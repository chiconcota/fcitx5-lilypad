//! Functions for converting a sequence of transformations into a final string.

use crate::engine::{MAX_ACTIVE_TRANS, Transformation};
use crate::input_method::{EffectType, Mark};
use crate::mode::OutputOptions;
use crate::phonetics::{add_mark_to_char, add_tone_to_char, lower, upper};

/// Converts a slice of transformations into a string based on the provided options.
pub(crate) fn flatten_slice(composition: &[Transformation], options: OutputOptions) -> String {
    // Exact upper bound in O(1): each transformation emits at most one char
    // (at most 4 UTF-8 bytes), replacing the estimate pre-pass.
    let mut out = String::with_capacity(composition.len() * 4);
    write_canvas_slice(composition, options, &mut out);
    out
}

/// Appends the flattened composition to an existing string buffer without clearing it.
/// Used by `commit()` to avoid allocating a temporary String.
pub(crate) fn append_flatten_slice(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut String,
) {
    out.reserve(composition.len() * 4);
    write_canvas_slice(composition, options, out);
}

/// Destination for flattened characters. The generic canvas below
/// monomorphizes per sink, so the hot [`String`] path codegens exactly like
/// the direct `push` it replaces.
pub(crate) trait CanvasSink {
    fn push_char(&mut self, c: char);
}

impl CanvasSink for String {
    #[inline(always)]
    fn push_char(&mut self, c: char) {
        self.push(c);
    }
}

impl CanvasSink for Vec<u8> {
    #[inline(always)]
    fn push_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        self.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    }
}

/// Appends the flattened composition as UTF-8 bytes without clearing `out`.
/// Used by [`Dfa::add_state`](crate::dfa::Dfa::add_state) to fill the flat
/// arena directly instead of round-tripping through a temporary `String`
/// (one allocation plus a copy per new JIT state).
pub(crate) fn append_flatten_bytes(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut Vec<u8>,
) {
    out.reserve(composition.len() * 4);
    write_canvas_slice(composition, options, out);
}

/// Same output as flattening with [`OutputOptions::RAW`], without the canvas
/// setup, since `commit()` runs it on every word.
pub(crate) fn append_raw_keys(composition: &[Transformation], out: &mut String) {
    for t in composition.iter().filter(|t| t.key != '\0') {
        out.push(if t.is_upper_case { upper(t.key) } else { t.key });
    }
}

/// Appends already flattened text, applying the per-character options the
/// flattener would have applied (committed words keep no transformations).
pub(crate) fn append_text_with_options(text: &str, options: OutputOptions, out: &mut String) {
    if options.is_empty() {
        out.push_str(text);
        return;
    }
    let tone_less = options.contains(OutputOptions::TONE_LESS);
    let mark_less = options.contains(OutputOptions::MARK_LESS);
    let lower_case = options.contains(OutputOptions::LOWER_CASE);
    for c in text.chars() {
        // Tone and mark tables are lowercase-only. `lowered` is computed
        // once and reused for both normalization and the case check below.
        let lowered = lower(c);
        let mut chr = lowered;
        if tone_less {
            chr = add_tone_to_char(chr, 0);
        }
        if mark_less {
            chr = crate::phonetics::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
        }
        let keep_upper = !lower_case && c != lowered;
        out.push(if keep_upper { upper(chr) } else { chr });
    }
}

fn write_canvas_slice<S: CanvasSink>(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut S,
) {
    if composition.is_empty() {
        return;
    }

    let len = composition.len();
    debug_assert!(len <= MAX_ACTIVE_TRANS, "composition too long for stack canvas: {len}");
    if len > MAX_ACTIVE_TRANS {
        return;
    }

    // Loop-invariant options, hoisted so the loops below contain no
    // per-character option tests.
    let raw = options.contains(OutputOptions::RAW);
    let tone_less = options.contains(OutputOptions::TONE_LESS);
    let mark_less = options.contains(OutputOptions::MARK_LESS);
    let lower_case = options.contains(OutputOptions::LOWER_CASE);

    // Intrusive effect chains in application order: `head_effect[t]` is the
    // first transform targeting `t`, `tail_effect[t]` the last one appended,
    // `next_effect` links them oldest-first. Forward links always point to a
    // later insertion, so cycles are impossible by construction.
    const NO_EFFECT: u8 = 0xFF;
    let mut next_effect = [NO_EFFECT; MAX_ACTIVE_TRANS];
    let mut head_effect = [NO_EFFECT; MAX_ACTIVE_TRANS];
    let mut tail_effect = [NO_EFFECT; MAX_ACTIVE_TRANS];
    let mut appending_idxs = [0u8; MAX_ACTIVE_TRANS];
    let mut appending_len = 0usize;

    for (idx, trans) in composition.iter().enumerate() {
        if (raw || trans.effect_type == EffectType::Appending) && trans.key != '\0' {
            // Dead-guard removed: `len <= MAX_ACTIVE_TRANS` above bounds
            // `appending_len` to the array size.
            appending_idxs[appending_len] = idx as u8;
            appending_len += 1;
        } else if let Some(target) = trans.target()
            && (target as usize) < len
        {
            let t = target as usize;
            if head_effect[t] == NO_EFFECT {
                head_effect[t] = idx as u8;
            } else {
                next_effect[tail_effect[t] as usize] = idx as u8;
            }
            tail_effect[t] = idx as u8;
        }
    }

    for &abs_u8 in appending_idxs.iter().take(appending_len) {
        let abs_idx = abs_u8 as usize;
        let appending_trans = &composition[abs_idx];

        let mut chr: char;
        if raw {
            chr = appending_trans.key;
        } else {
            chr = appending_trans.effect_on;

            // Application order straight from the chain: no collect buffer,
            // no reverse pass. Bounded defensively (unreachable by construction).
            let mut curr = head_effect[abs_idx];
            let mut guard = 0usize;
            while curr != NO_EFFECT && guard < MAX_ACTIVE_TRANS {
                let t = &composition[curr as usize];

                match t.effect_type {
                    EffectType::MarkTransformation => {
                        if t.effect == Mark::Raw as u8 {
                            chr = appending_trans.key;
                        } else {
                            chr = add_mark_to_char(chr, t.effect);
                        }
                    }
                    EffectType::ToneTransformation => {
                        chr = add_tone_to_char(chr, t.effect);
                    }
                    _ => {}
                }
                curr = next_effect[curr as usize];
                guard += 1;
            }
            debug_assert!(
                curr == NO_EFFECT,
                "flattener: effect chain exceeded MAX_ACTIVE_TRANS — possible cycle"
            );
        }

        if tone_less {
            chr = add_tone_to_char(chr, 0);
        }
        if mark_less {
            chr = crate::phonetics::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
        }

        let final_chr = if lower_case {
            lower(chr)
        } else if appending_trans.is_upper_case {
            upper(chr)
        } else {
            chr
        };
        out.push_char(final_chr);
    }
}

pub(crate) fn first_canvas_char_in_suffix(
    composition: &[Transformation],
    start: usize,
    options: OutputOptions,
) -> Option<char> {
    let mut first: Option<(usize, &Transformation)> = None;
    for (idx, trans) in composition[start..].iter().enumerate() {
        let abs_idx = start + idx;
        if options.contains(OutputOptions::RAW) {
            if trans.key == '\0' {
                continue;
            }
            first = Some((abs_idx, trans));
            break;
        }
        if trans.effect_type == EffectType::Appending && trans.key != '\0' {
            first = Some((abs_idx, trans));
            break;
        }
    }

    let (target_abs_idx, appending_trans) = first?;
    let mut chr = if options.contains(OutputOptions::RAW) {
        appending_trans.key
    } else {
        let mut c = appending_trans.effect_on;
        for trans in &composition[start..] {
            if trans.target() != Some(target_abs_idx as u8) {
                continue;
            }
            match trans.effect_type {
                EffectType::MarkTransformation => {
                    if trans.effect == Mark::Raw as u8 {
                        c = appending_trans.key;
                    } else {
                        c = add_mark_to_char(c, trans.effect);
                    }
                }
                EffectType::ToneTransformation => {
                    c = add_tone_to_char(c, trans.effect);
                }
                _ => {}
            }
        }
        c
    };

    if options.contains(OutputOptions::TONE_LESS) {
        chr = add_tone_to_char(chr, 0);
    }
    if options.contains(OutputOptions::MARK_LESS) {
        chr = crate::phonetics::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
    }
    if options.contains(OutputOptions::LOWER_CASE) {
        chr = lower(chr);
    } else if appending_trans.is_upper_case {
        chr = upper(chr);
    }

    Some(chr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_method::EffectType;

    #[test]
    fn first_canvas_char_in_suffix_handles_offsets() {
        let t1 = Transformation::new('a', 'a', 'a', None, 0, EffectType::Appending, false);
        let t2 = Transformation::new(' ', ' ', ' ', None, 0, EffectType::Appending, false);
        let t3 = Transformation::new('w', 'w', 'w', None, 0, EffectType::Appending, false);
        let comp = vec![t1, t2, t3];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, OutputOptions::RAW), Some(' '));
        assert_eq!(first_canvas_char_in_suffix(&comp, 2, OutputOptions::NONE), Some('w'));
    }

    #[test]
    fn first_canvas_char_in_suffix_resolves_absolute_targets() {
        let x = Transformation::new('x', 'x', 'x', None, 0, EffectType::Appending, false);
        let o = Transformation::new('o', 'o', 'o', None, 0, EffectType::Appending, false);
        let mark_hat =
            Transformation::new('o', 'o', 'ô', Some(1), 1, EffectType::MarkTransformation, false);
        let comp = vec![x, o, mark_hat];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, OutputOptions::NONE), Some('ô'));
    }

    #[test]
    fn flatten_applies_mark_and_tone_in_order() {
        let o = Transformation::new('o', 'o', 'o', None, 0, EffectType::Appending, false);
        let hat =
            Transformation::new('o', 'o', 'ô', Some(0), 1, EffectType::MarkTransformation, false);
        let acute =
            Transformation::new('s', '\0', '\0', Some(0), 2, EffectType::ToneTransformation, false);
        let comp = vec![o, hat, acute];
        assert_eq!(flatten_slice(&comp, OutputOptions::NONE), "ố");
    }

    #[test]
    fn byte_sink_matches_string_sink() {
        let o = Transformation::new('o', 'o', 'o', None, 0, EffectType::Appending, true);
        let hat =
            Transformation::new('o', 'o', 'ô', Some(0), 1, EffectType::MarkTransformation, false);
        let acute =
            Transformation::new('s', '\0', '\0', Some(0), 2, EffectType::ToneTransformation, false);
        let comp = vec![o, hat, acute];
        let options = [
            OutputOptions::NONE,
            OutputOptions::RAW,
            OutputOptions::LOWER_CASE,
            OutputOptions::TONE_LESS,
            OutputOptions::MARK_LESS,
            OutputOptions::FULL_TEXT,
        ];
        for option in options {
            let mut bytes = Vec::new();
            append_flatten_bytes(&comp, option, &mut bytes);
            assert_eq!(bytes, flatten_slice(&comp, option).as_bytes(), "{option:?}");
        }
    }
}
