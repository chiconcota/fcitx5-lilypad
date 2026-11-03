//! Internal utility functions for Vietnamese syllable analysis and transformation generation.

use super::tables::{TONE_ON_SECOND, TONE_PREFERRED, TONE_SECOND_ONSETS, UHO_PATTERNS};
use crate::config::Config;
use crate::engine::{MAX_ACTIVE_TRANS, Transformation, TransformationStack};
use crate::flattener::flatten_slice;
use crate::input_method::{EffectType, Mark, Rule, Tone};
use crate::mode::OutputOptions;
use crate::phonetics::{
    add_mark_to_char, add_tone_to_char, is_alpha, is_space, is_upper, is_vowel, lower,
};
use crate::spelling::{is_valid_cvc, is_valid_cvc_chars};

fn in_key_list(keys: Option<&[char]>, key: char) -> bool {
    keys.is_some_and(|ks| ks.contains(&key))
}

/// Extracts the raw (toneless, markless, lowercase) key chars from appending transformations.
/// Returns the number of chars written into `out`.
fn raw_keys_of(trans_slice: &[Transformation], out: &mut [char; 4]) -> usize {
    let mut len = 0;
    for t in trans_slice {
        if t.key != '\0' && len < 4 {
            out[len] = lower(crate::phonetics::add_tone_to_char(
                crate::phonetics::add_mark_to_toneless_char(t.key, 0),
                0,
            ));
            len += 1;
        }
    }
    len
}

/// Finds the last transformation in the composition that resulted in an appended character with its index.
#[inline]
pub(crate) fn find_last_appending_entry(
    composition: &[Transformation],
) -> Option<(u8, Transformation)> {
    composition
        .iter()
        .enumerate()
        .rev()
        .find(|(_, trans)| trans.effect_type == EffectType::Appending)
        .map(|(i, &t)| (i as u8, t))
}

/// Creates a new transformation that simply appends a character.
pub(crate) const fn new_appending_trans(key: char, is_upper_case: bool) -> Transformation {
    Transformation::new(key, key, key, None, 0, EffectType::Appending, is_upper_case)
}

/// Generates an appending transformation based on the provided rules and key.
pub(crate) fn generate_appending_trans(
    rules: &[Rule],
    lower_key: char,
    is_upper_case: bool,
) -> Transformation {
    for rule in rules {
        if rule.key == lower_key && rule.effect_type == EffectType::Appending {
            let effective_upper = is_upper_case || is_upper(rule.effect_on);
            let lower_eff_on = lower(rule.effect_on);
            return Transformation::new(
                rule.key,
                lower_eff_on,
                lower_eff_on,
                None,
                rule.effect,
                rule.effect_type,
                effective_upper,
            );
        }
    }

    new_appending_trans(lower_key, is_upper_case)
}

fn find_root_target(composition: &[Transformation], mut target: u8) -> u8 {
    // Guard against out-of-bounds access and cycles (max depth = composition length).
    let max_depth = composition.len();
    let mut depth = 0;
    while let Some(t) = composition.get(target as usize).and_then(|tr| tr.target()) {
        target = t;
        depth += 1;
        if depth >= max_depth {
            // Cycle detected or malformed chain — return current target to avoid infinite loop.
            debug_assert!(false, "find_root_target: cycle or invalid chain detected");
            break;
        }
    }
    target
}

/// Checks if the current composition represents a valid Vietnamese syllable.
pub(crate) fn is_valid(composition: &[Transformation], input_is_full_complete: bool) -> bool {
    check_validity(composition, input_is_full_complete).0
}

/// Validity plus the tone-check breakdown, when a tone transformation is
/// present. Lets validate-then-refresh call sites (engine slow path) reuse
/// one `extract_cvc_trans` instead of extracting twice per keystroke.
pub(crate) fn check_validity(
    composition: &[Transformation],
    input_is_full_complete: bool,
) -> (bool, Option<Cvc>) {
    let tone = composition.iter().rev().find(|t| t.effect_type == EffectType::ToneTransformation);
    if composition.len() <= 1 {
        let cvc = tone.map(|_| extract_cvc_trans(composition));
        return (true, cvc);
    }

    // last tone checking
    let mut tone_cvc = None;
    if let Some(trans) = tone {
        let last_tone = trans.tone();
        let cvc = extract_cvc_trans(composition);
        if !has_valid_tone(composition, &cvc, last_tone) {
            return (false, None);
        }
        tone_cvc = Some(cvc);
    }

    (spell_check(composition, input_is_full_complete), tone_cvc)
}

// spell checking (fast path: no heap for engine's bounded composition)
fn spell_check(composition: &[Transformation], input_is_full_complete: bool) -> bool {
    if composition.len() <= MAX_ACTIVE_TRANS {
        let mut app_abs = [0usize; MAX_ACTIVE_TRANS];
        let mut app_len = 0usize;

        for (abs_idx, t) in composition.iter().enumerate() {
            if !t.has_target() {
                app_abs[app_len] = abs_idx;
                app_len += 1;
            }
        }

        let app_indices = &app_abs[..app_len];
        let (fc_idxs, vo_idxs, lc_idxs) = extract_cvc_appending_indices(composition, app_indices);

        let mut fc_chars = ['\0'; MAX_ACTIVE_TRANS];
        let mut vo_chars = ['\0'; MAX_ACTIVE_TRANS];
        let mut lc_chars = ['\0'; MAX_ACTIVE_TRANS];

        // Resolve all characters in a single pass O(N)
        let mut resolved = ['\0'; MAX_ACTIVE_TRANS];
        for &abs_idx in app_indices.iter() {
            resolved[abs_idx] = composition[abs_idx].effect_on;
        }

        for t in composition {
            if let Some(target) = t.target()
                && (target as usize) < MAX_ACTIVE_TRANS
            {
                match t.effect_type {
                    EffectType::MarkTransformation => {
                        if t.effect == Mark::Raw as u8 {
                            resolved[target as usize] = composition[target as usize].key;
                        } else {
                            resolved[target as usize] =
                                add_mark_to_char(resolved[target as usize], t.effect);
                        }
                    }
                    EffectType::ToneTransformation => {
                        resolved[target as usize] =
                            add_tone_to_char(resolved[target as usize], t.effect);
                    }
                    _ => {}
                }
            }
        }

        for (i, &abs) in fc_idxs.iter().enumerate() {
            fc_chars[i] = lower(add_tone_to_char(resolved[abs], 0));
        }
        for (i, &abs) in vo_idxs.iter().enumerate() {
            vo_chars[i] = lower(add_tone_to_char(resolved[abs], 0));
        }
        for (i, &abs) in lc_idxs.iter().enumerate() {
            lc_chars[i] = lower(add_tone_to_char(resolved[abs], 0));
        }

        return is_valid_cvc_chars(
            &fc_chars[..fc_idxs.len()],
            &vo_chars[..vo_idxs.len()],
            &lc_chars[..lc_idxs.len()],
            input_is_full_complete,
        );
    }

    // fallback for uncommon long compositions
    let cvc = extract_cvc_trans(composition);
    let flatten_mode = OutputOptions::NONE | OutputOptions::LOWER_CASE | OutputOptions::TONE_LESS;
    is_valid_cvc(
        &flatten_slice(cvc.fc_slice(), flatten_mode),
        &flatten_slice(cvc.vo_slice(), flatten_mode),
        &flatten_slice(cvc.lc_slice(), flatten_mode),
        input_is_full_complete,
    )
}

/// Represents the broken-down parts of a Vietnamese syllable (Consonant-Vowel-Consonant).
#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct Cvc {
    /// Transformations in the first consonant part.
    pub fc: [Transformation; 8],
    /// Number of transformations in `fc`.
    pub fc_len: u8,
    /// Transformations in the vowel part.
    pub vo: [Transformation; 8],
    /// Number of transformations in `vo`.
    pub vo_len: u8,
    /// Transformations in the last consonant part.
    pub lc: [Transformation; 8],
    /// Number of transformations in `lc`.
    pub lc_len: u8,
    /// Absolute composition indices of the appending vowels (the `vo` split
    /// subset), in order. Locates vowels without value search, which mis-hits
    /// on identical twins (e.g. the second 'i' of "gii" after the gi-split
    /// moves the first into the onset).
    pub vo_abs: [u8; MAX_ACTIVE_TRANS],
    /// Number of indices in `vo_abs`.
    pub vo_abs_len: u8,
}

impl Cvc {
    /// Returns the transformations for the first consonant.
    pub fn fc_slice(&self) -> &[Transformation] {
        &self.fc[..self.fc_len as usize]
    }

    /// Returns the transformations for the vowel part.
    pub fn vo_slice(&self) -> &[Transformation] {
        &self.vo[..self.vo_len as usize]
    }

    /// Returns the transformations for the last consonant.
    pub fn lc_slice(&self) -> &[Transformation] {
        &self.lc[..self.lc_len as usize]
    }
}

/// Resolves the display char of an appending entry after applying its mark/tone chain.
fn resolve_appended_char(composition: &[Transformation], abs_idx: usize) -> char {
    let app = &composition[abs_idx];
    let mut c = app.effect_on;
    for t in composition {
        if t.target() != Some(abs_idx as u8) {
            continue;
        }
        match t.effect_type {
            EffectType::MarkTransformation => {
                if t.effect == Mark::Raw as u8 {
                    c = app.key;
                } else {
                    c = add_mark_to_char(c, t.effect);
                }
            }
            EffectType::ToneTransformation => {
                c = add_tone_to_char(c, t.effect);
            }
            _ => {}
        }
    }
    c
}

/// Index of the vowel that should carry the tone of `composition`.
pub(crate) fn tone_target(composition: &[Transformation], std_style: bool) -> Option<u8> {
    find_tone_target(composition, &extract_cvc_trans(composition), std_style)
}

fn find_tone_target(composition: &[Transformation], cvc: &Cvc, std_style: bool) -> Option<u8> {
    if composition.is_empty() {
        return None;
    }

    let vowels = cvc.vo_slice();
    let lc = cvc.lc_slice();

    // Absolute indices of the appending vowels, collected once by
    // `extract_cvc_trans`. Index-based: comparing Transformation values is
    // wrong when two vowels are identical (e.g. literal "oo").
    let vo_abs = &cvc.vo_abs[..cvc.vo_abs_len as usize];
    let appending_vowels_len = vo_abs.len();
    if appending_vowels_len == 0 {
        return None;
    }

    if appending_vowels_len == 1 {
        return Some(vo_abs[0]);
    }

    if appending_vowels_len == 2 && std_style {
        let mut target: Option<u8> = None;
        let has_u_horn = vowels.iter().any(|t| t.result == 'ư');
        let has_o_horn = vowels.iter().any(|t| t.result == 'ơ');
        if has_u_horn && has_o_horn {
            // Go's findToneTarget prefers the vowel carrying ơ/ê for the tone,
            // coda or not (`tuowf` -> `tườ`, not `từơ`). The first/second
            // heuristic remains as fallback for pairs without a preferred vowel.
            let preferred = vo_abs.iter().copied().find(|&abs| {
                TONE_PREFERRED.contains(&add_tone_to_char(
                    resolve_appended_char(composition, abs as usize),
                    0,
                ))
            });
            let is_th_or_h = {
                let fc = cvc.fc_slice();
                TONE_SECOND_ONSETS.iter().any(|onset| {
                    fc.len() == onset.len() && fc.iter().zip(*onset).all(|(t, &c)| t.key == c)
                })
            };
            target = Some(preferred.unwrap_or(if !lc.is_empty() || is_th_or_h {
                vo_abs[1]
            } else {
                vo_abs[0]
            }));
        } else {
            // Use the *resolved* base char so a cancelled mark chain (e.g. "ooo"
            // undoing "oo"→"ô" on the first "o") does not steal the tone target.
            for &abs in vo_abs {
                let base = add_tone_to_char(resolve_appended_char(composition, abs as usize), 0);
                if TONE_PREFERRED.contains(&base) {
                    target = Some(abs);
                }
            }
        }
        if target.is_none() {
            target = Some(if !lc.is_empty() { vo_abs[1] } else { vo_abs[0] });
        }
        return target;
    }

    if appending_vowels_len == 2 {
        if !lc.is_empty() {
            return Some(vo_abs[1]);
        }

        // Compare raw key chars directly — no heap allocation needed.
        let app_vowels = [composition[vo_abs[0] as usize], composition[vo_abs[1] as usize]];
        let mut raw = ['\0'; 4];
        let raw_len = raw_keys_of(&app_vowels, &mut raw);
        let tone_on_second =
            raw_len == 2 && TONE_ON_SECOND.iter().any(|&[a, b]| raw[0] == a && raw[1] == b);
        return Some(if tone_on_second { vo_abs[1] } else { vo_abs[0] });
    }

    if appending_vowels_len == 3 {
        let app_vowels = [
            composition[vo_abs[0] as usize],
            composition[vo_abs[1] as usize],
            composition[vo_abs[2] as usize],
        ];
        let mut raw = ['\0'; 4];
        let raw_len = raw_keys_of(&app_vowels, &mut raw);
        let is_uye = raw_len == 3 && raw[0] == 'u' && raw[1] == 'y' && raw[2] == 'e';
        return Some(if is_uye { vo_abs[2] } else { vo_abs[1] });
    }

    None
}

fn has_valid_tone(composition: &[Transformation], cvc: &Cvc, tone: Tone) -> bool {
    if matches!(tone, Tone::None | Tone::Acute | Tone::Dot) {
        return true;
    }
    if composition.is_empty() {
        return true;
    }
    if cvc.lc_len == 0 {
        return true;
    }

    // Compare raw key chars directly — no String allocation needed.
    let lc = cvc.lc_slice();
    let mut raw = ['\0'; 4];
    let raw_len = raw_keys_of(lc, &mut raw);
    let lc_str = &raw[..raw_len];
    !matches!(lc_str, ['c'] | ['k'] | ['p'] | ['t'] | ['c', 'h'])
}

fn get_last_tone_transformation(composition: &[Transformation]) -> Option<Transformation> {
    composition
        .iter()
        .rev()
        .find(|t| t.effect_type == EffectType::ToneTransformation && t.has_target())
        .copied()
}

fn is_free(composition: &[Transformation], trans_idx: usize, effect_type: EffectType) -> bool {
    composition
        .iter()
        .all(|t| !(t.target() == Some(trans_idx as u8) && t.effect_type == effect_type))
}

fn extract_cvc_appending_indices<'a>(
    _composition: &[Transformation],
    app_indices: &'a [usize],
) -> (&'a [usize], &'a [usize], &'a [usize]) {
    let mut results = ['\0'; MAX_ACTIVE_TRANS];
    for (i, &idx) in app_indices.iter().enumerate() {
        results[i] = _composition[idx].result;
    }

    let (head, lc) = {
        let mut idx = app_indices.len();
        while idx > 0 {
            if is_vowel(results[idx - 1]) {
                break;
            }
            idx -= 1;
        }
        (&app_indices[..idx], &app_indices[idx..])
    };

    let (fc, vo) = {
        let mut idx = head.len();
        while idx > 0 {
            if !is_vowel(results[idx - 1]) {
                break;
            }
            idx -= 1;
        }
        (&head[..idx], &head[idx..])
    };

    if fc.is_empty() && vo.is_empty() && !lc.is_empty() {
        return (lc, &[], &[]);
    }

    let mut fc_final = fc;
    let mut vo_final = vo;
    // `results[i]` is parallel to `app_indices[i]` (app_indices-space).
    // `fc`/`vo` are slices of `app_indices` (absolute composition indices),
    // so lookups into `results` must use positions 0..app_len, not absolute indices.
    if (fc.len() == 1
        && vo.len() > 1
        && (lc.is_empty() || (fc.len() + 1 < app_indices.len() && results[fc.len() + 1] != 'e'))
        && results[fc.len()] == 'i'
        && results[0] == 'g')
        || (fc.len() == 1 && !vo.is_empty() && results[0] == 'q' && results[fc.len()] == 'u')
    {
        fc_final = &app_indices[..fc.len() + 1];
        vo_final = &vo[1..];
    }

    (fc_final, vo_final, lc)
}

pub(crate) fn extract_cvc_trans(composition: &[Transformation]) -> Cvc {
    let mut app_indices = [0usize; MAX_ACTIVE_TRANS];
    let mut app_len = 0usize;
    for (i, t) in composition.iter().enumerate() {
        if !t.has_target() && app_len < MAX_ACTIVE_TRANS {
            app_indices[app_len] = i;
            app_len += 1;
        }
    }

    let (fc_idxs, vo_idxs, lc_idxs) =
        extract_cvc_appending_indices(composition, &app_indices[..app_len]);

    let mut res = Cvc::default();

    // Record the absolute vowel indices for index-based (not value-based)
    // vowel location by callers.
    for (k, &abs) in vo_idxs.iter().enumerate() {
        if k < res.vo_abs.len() {
            debug_assert!(abs < MAX_ACTIVE_TRANS);
            res.vo_abs[k] = abs as u8;
            res.vo_abs_len += 1;
        }
    }

    for &i in fc_idxs {
        if (res.fc_len as usize) < res.fc.len() {
            res.fc[res.fc_len as usize] = composition[i];
            res.fc_len += 1;
        }
    }
    for &i in vo_idxs {
        if (res.vo_len as usize) < res.vo.len() {
            res.vo[res.vo_len as usize] = composition[i];
            res.vo_len += 1;
        }
    }
    for &i in lc_idxs {
        if (res.lc_len as usize) < res.lc.len() {
            res.lc[res.lc_len as usize] = composition[i];
            res.lc_len += 1;
        }
    }

    // Pre-compute bitmasks once for O(1) lookup in the loop below.
    let fc_mask: u32 = fc_idxs.iter().fold(0u32, |m, &i| m | (1u32 << i));
    let vo_mask: u32 = vo_idxs.iter().fold(0u32, |m, &i| m | (1u32 << i));
    let lc_mask: u32 = lc_idxs.iter().fold(0u32, |m, &i| m | (1u32 << i));

    for trans in composition {
        if let Some(target_idx) = trans.target() {
            let bit = if (target_idx as usize) < MAX_ACTIVE_TRANS { 1u32 << target_idx } else { 0 };
            if (fc_mask & bit) != 0 {
                if (res.fc_len as usize) < res.fc.len() {
                    res.fc[res.fc_len as usize] = *trans;
                    res.fc_len += 1;
                }
            } else if (vo_mask & bit) != 0 {
                if (res.vo_len as usize) < res.vo.len() {
                    res.vo[res.vo_len as usize] = *trans;
                    res.vo_len += 1;
                }
            } else if (lc_mask & bit) != 0 && (res.lc_len as usize) < res.lc.len() {
                res.lc[res.lc_len as usize] = *trans;
                res.lc_len += 1;
            }
        }
    }

    res
}

/// Extracts the last word along with its punctuation marks from the composition.
pub(crate) fn extract_last_word_with_punctuation_marks<'a>(
    composition: &'a [Transformation],
    _effect_keys: &[char],
) -> (&'a [Transformation], &'a [Transformation]) {
    for i in (0..composition.len()).rev() {
        let Some(c) =
            crate::flattener::first_canvas_char_in_suffix(composition, i, OutputOptions::RAW)
        else {
            continue;
        };
        if is_space(c) {
            if i == composition.len() - 1 {
                return (composition, &[]);
            }
            return (&composition[..i + 1], &composition[i + 1..]);
        }
    }

    (&[], composition)
}

/// Extracts the last word from the composition.
pub(crate) fn extract_last_word<'a>(
    composition: &'a [Transformation],
    effect_keys: Option<&[char]>,
) -> (&'a [Transformation], &'a [Transformation]) {
    for i in (0..composition.len()).rev() {
        let Some(c) = crate::flattener::first_canvas_char_in_suffix(
            composition,
            i,
            OutputOptions::NONE
                | OutputOptions::LOWER_CASE
                | OutputOptions::TONE_LESS
                | OutputOptions::MARK_LESS,
        ) else {
            continue;
        };
        if !is_alpha(c) && !in_key_list(effect_keys, c) {
            if i == composition.len() - 1 {
                return (composition, &[]);
            }
            return (&composition[..i + 1], &composition[i + 1..]);
        }
    }

    (&[], composition)
}

/// Opaque resume hint for [`last_syllable_start`].
///
/// Typing extends the composition a few transformations at a time, so
/// consecutive calls observe the same prefix plus a short new tail. The hint
/// lets a call skip re-validating windows it already verified and only check
/// the suffix windows covering new content: O(new) validity checks instead of
/// O(len).
///
/// Self-validating: every field is re-checked against the current composition
/// before use, so a stale hint only costs time (full rescan), never changes
/// the result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SyllHint {
    /// `hash_composition` of `composition[..len]` when stored.
    hash: u64,
    /// Length of the composition when stored.
    len: u8,
    /// Main-loop start when stored, *before* the effect fixup below (the
    /// fixup always re-runs fully, so resuming must continue from the
    /// pre-fixup value or the replay diverges).
    start: u8,
    /// Whether the hint holds stored values.
    valid: bool,
}

impl SyllHint {
    pub(crate) const NONE: Self = Self { hash: 0, len: 0, start: 0, valid: false };
}

/// Returns where the syllable that the next key edits starts: the last word,
/// cut before the first letter that made it an invalid syllable. A tone key
/// after "enl" thus only sees "l", and is typed as a letter.
///
/// Returns the start together with a hint for the next call. `hint` should be
/// the hint returned by the previous call on the prefix of this composition;
/// [`SyllHint::NONE`] disables resuming. The result is identical with or
/// without a usable hint.
pub(crate) fn last_syllable_start(
    composition: &[Transformation],
    hint: SyllHint,
) -> (usize, SyllHint) {
    let len = composition.len();
    // Resume: the stored prefix is identical (hash match), the composition
    // only grew (no delete/replace), and the new tail cannot move the last
    // word start (appended letters keep an all-letter tail a single word;
    // mark/tone transformations add no canvas character of their own in the
    // non-RAW mode the word split uses). Earlier windows then replay
    // identically and end at the stored start, so only suffix windows past
    // the stored length need checking.
    if hint.valid
        && (hint.len as usize) <= len
        && len <= MAX_ACTIVE_TRANS
        && composition[hint.len as usize..].iter().all(|t| {
            if t.effect_type == EffectType::Appending {
                // An appended letter keeps the tail one word; anything else
                // (digit/punctuation) may split it.
                is_alpha(t.key)
            } else {
                // A pure effect adds no canvas character, but only when it
                // points at a letter (a stray effect without target is not
                // worth reasoning about — rescan instead).
                t.has_target()
            }
        })
        && crate::dfa::hash_composition(&composition[..hint.len as usize]) == hint.hash
    {
        let mut start = hint.start as usize;
        for end in hint.len as usize..len {
            // A one-char window is always valid (`is_valid` returns true for
            // len <= 1); skip the copy and the call.
            if end + 1 - start <= 1 {
                continue;
            }
            if !is_valid_from(composition, start, end + 1) {
                start = end;
            }
        }
        let fixed = pull_effects_back(composition, start);
        return (fixed, store_hint(composition, start));
    }
    let (previous, _) = extract_last_word(composition, None);
    let mut start = previous.len();
    for end in start + 1..len {
        if !is_valid_from(composition, start, end + 1) {
            start = end;
        }
    }
    let fixed = pull_effects_back(composition, start);
    (fixed, store_hint(composition, start))
}

/// Stores a resume hint for `composition`/`start` (`NONE` when too long).
fn store_hint(composition: &[Transformation], start: usize) -> SyllHint {
    if composition.len() > MAX_ACTIVE_TRANS || start > MAX_ACTIVE_TRANS {
        return SyllHint::NONE;
    }
    SyllHint {
        hash: crate::dfa::hash_composition(composition),
        len: composition.len() as u8,
        start: start as u8,
        valid: true,
    }
}

/// Index targets cannot point before the slice, so keep effects with their letters.
fn pull_effects_back(composition: &[Transformation], mut start: usize) -> usize {
    while let Some(target) = composition[start..]
        .iter()
        .filter_map(Transformation::target)
        .filter(|&t| (t as usize) < start)
        .min()
    {
        start = target as usize;
    }
    start
}

/// `is_valid` on `composition[start..end]`, ignoring effects on letters before `start`.
fn is_valid_from(composition: &[Transformation], start: usize, end: usize) -> bool {
    let mut part = [Transformation::default(); MAX_ACTIVE_TRANS];
    let len = end - start;
    part[..len].copy_from_slice(&composition[start..end]);
    for t in &mut part[..len] {
        if let Some(target) = t.target() {
            let outside = (target as usize) < start;
            // Out of range, so it matches no letter, like Go's pointer that is in no list.
            t.set_target(Some(if outside { MAX_ACTIVE_TRANS as u8 } else { target - start as u8 }));
        }
    }
    is_valid(&part[..len], false)
}

fn is_effective(composition: &[Transformation], target_idx: usize, new_rule: &Rule) -> bool {
    // Undo pattern: the same tone/mark key pressed twice in a row on the same
    // target (e.g., "ss", "xx"). Deliberately left "not effective" so the
    // caller falls back to undo + literal key.
    if let Some(last) = composition.last()
        && last.target() == Some(target_idx as u8)
        && last.effect_type == new_rule.effect_type
        && last.effect == new_rule.effect
        && last.key != '\0'
        && last.key == new_rule.key
    {
        return false;
    }

    let appending_idx = find_root_target(composition, target_idx as u8);
    let appending = &composition[appending_idx as usize];

    let mut current_char = appending.effect_on;
    for t in composition {
        if t.target() == Some(appending_idx) {
            match t.effect_type {
                EffectType::MarkTransformation => {
                    if t.effect == Mark::Raw as u8 {
                        current_char = appending.key;
                    } else {
                        current_char = add_mark_to_char(current_char, t.effect);
                    }
                }
                EffectType::ToneTransformation => {
                    current_char = add_tone_to_char(current_char, t.effect);
                }
                _ => {}
            }
        }
    }

    let mut next_char = current_char;
    match new_rule.effect_type {
        EffectType::MarkTransformation => {
            if new_rule.effect == Mark::Raw as u8 {
                next_char = appending.key;
            } else {
                next_char = add_mark_to_char(current_char, new_rule.effect);
            }
        }
        EffectType::ToneTransformation => {
            next_char = add_tone_to_char(current_char, new_rule.effect);
        }
        _ => {}
    }

    next_char != current_char
}

fn find_mark_target_excluding(
    composition: &[Transformation],
    rules: &[Rule],
    exclude_target: Option<u8>,
) -> (Option<u8>, Option<Rule>) {
    let mut tmp = [Transformation::default(); MAX_ACTIVE_TRANS];
    for (idx, trans) in composition.iter().enumerate().rev() {
        for rule in rules {
            if rule.effect_type != EffectType::MarkTransformation || rule.effect == 0 {
                continue;
            }
            if trans.result != rule.effect_on {
                continue;
            }
            let target = find_root_target(composition, idx as u8);
            if Some(target) == exclude_target {
                continue;
            }
            if !is_effective(composition, target as usize, rule) {
                continue;
            }

            let base_len = composition.len();
            if base_len >= MAX_ACTIVE_TRANS {
                continue;
            }
            let tmp_len = base_len + 1;
            tmp[..base_len].copy_from_slice(composition);
            tmp[base_len] = Transformation::from_rule(*rule, Some(target), false);

            if rule.get_mark() == Mark::Dash || is_valid(&tmp[..tmp_len], false) {
                return (Some(target), Some(*rule));
            }
        }
    }

    (None, None)
}

/// Finds the target for a given transformation rule within the current composition.
pub(crate) fn find_target(
    composition: &[Transformation],
    applicable_rules: &[Rule],
    config: Config,
    cvc: &Cvc,
) -> (Option<u8>, Option<Rule>) {
    find_target_excluding(composition, applicable_rules, config, None, cvc)
}

/// Finds the target excluding a specific target index (e.g. for companion vowel in dual transformations).
pub(crate) fn find_target_excluding(
    composition: &[Transformation],
    applicable_rules: &[Rule],
    config: Config,
    exclude_target: Option<u8>,
    cvc: &Cvc,
) -> (Option<u8>, Option<Rule>) {
    for applicable_rule in applicable_rules {
        if applicable_rule.effect_type != EffectType::ToneTransformation {
            continue;
        }

        let mut target: Option<u8> = None;
        if config.free_tone_marking {
            let tone = applicable_rule.get_tone();
            if has_valid_tone(composition, cvc, tone) {
                target = find_tone_target(composition, cvc, config.std_tone_style);
            }
        } else if let Some((idx, last_appending)) = find_last_appending_entry(composition)
            && is_vowel(last_appending.effect_on)
        {
            target = Some(idx);
        }

        let Some(t_idx) = target else { continue };
        if Some(t_idx) == exclude_target {
            continue;
        }
        let effective = is_effective(composition, t_idx as usize, applicable_rule);
        if !effective {
            continue;
        }

        if applicable_rule.effect == Tone::None as u8
            && is_free(composition, t_idx as usize, EffectType::ToneTransformation)
            && add_tone_to_char(composition[t_idx as usize].result, 0)
                == composition[t_idx as usize].result
        {
            target = None;
        }

        if target.is_some() {
            return (target, Some(*applicable_rule));
        }
    }

    find_mark_target_excluding(composition, applicable_rules, exclude_target)
}

fn generate_undo_transformations(
    composition: &[Transformation],
    rules: &[Rule],
    config: Config,
    out: &mut TransformationStack,
    cvc: &Cvc,
) {
    for rule in rules {
        if rule.effect_type == EffectType::ToneTransformation {
            let mut target: Option<u8> = None;
            if config.free_tone_marking {
                let tone = rule.get_tone();
                if has_valid_tone(composition, cvc, tone) {
                    target = find_tone_target(composition, cvc, config.std_tone_style);
                }
            } else if let Some((idx, last_appending)) = find_last_appending_entry(composition)
                && is_vowel(last_appending.effect_on)
            {
                target = Some(idx);
            }

            let Some(target) = target else { continue };
            let undo_rule = Rule {
                effect_type: EffectType::ToneTransformation,
                effect: 0,
                key: '\0',
                effect_on: '\0',
                result: '\0',
                appended: ['\0'; 2],
                appended_len: 0,
            };

            if is_effective(composition, target as usize, &undo_rule) {
                out.push(Transformation::new(
                    '\0',
                    '\0',
                    '\0',
                    Some(target),
                    0,
                    EffectType::ToneTransformation,
                    false,
                ));
            }
        } else if rule.effect_type == EffectType::MarkTransformation {
            for (idx, trans) in composition.iter().enumerate().rev() {
                if trans.result == rule.effect_on {
                    let target = find_root_target(composition, idx as u8);

                    let undo_rule = Rule {
                        key: '\0',
                        effect_type: EffectType::MarkTransformation,
                        effect: 0,
                        effect_on: '\0',
                        result: '\0',
                        appended: ['\0'; 2],
                        appended_len: 0,
                    };

                    if is_effective(composition, target as usize, &undo_rule) {
                        out.push(Transformation::new(
                            '\0',
                            '\0',
                            '\0',
                            Some(target),
                            0,
                            EffectType::MarkTransformation,
                            false,
                        ));
                    }
                }
            }
        }
    }
}

/// Checks if composition contains "uơ" or "ưo" pattern followed by an alphabetic
/// char — a stack-only replacement for `flatten_slice` + `uoh_tail_match`
/// that avoids heap allocation on the super-key fallback path.
pub(crate) fn uho_tail_match_composition(composition: &[Transformation]) -> bool {
    // Resolve effect chains in a single O(n) pass: start with appending `result`
    // chars, then overlay mark/tone effects from targeted transforms.
    let mut chars = ['\0'; MAX_ACTIVE_TRANS];
    let mut is_app = [false; MAX_ACTIVE_TRANS];
    let mut len = 0;
    for t in composition {
        if !t.has_target() && t.effect_type == EffectType::Appending && t.key != '\0' {
            chars[len] = t.result;
            is_app[len] = true;
            len += 1;
        }
    }
    // Map: composition index -> slot in chars[] for appending transforms.
    let mut slot_of = [usize::MAX; MAX_ACTIVE_TRANS];
    let mut slot = 0;
    for (ci, t) in composition.iter().enumerate() {
        if !t.has_target() && t.effect_type == EffectType::Appending && t.key != '\0' {
            slot_of[ci] = slot;
            slot += 1;
        }
    }
    for t in composition {
        if let Some(target) = t.target()
            && (target as usize) < MAX_ACTIVE_TRANS
            && slot_of[target as usize] != usize::MAX
        {
            let s = slot_of[target as usize];
            match t.effect_type {
                EffectType::MarkTransformation if t.effect == Mark::Raw as u8 => {
                    chars[s] = composition[target as usize].key;
                }
                EffectType::MarkTransformation => {
                    chars[s] = add_mark_to_char(chars[s], t.effect);
                }
                EffectType::ToneTransformation => {
                    chars[s] = add_tone_to_char(chars[s], t.effect);
                }
                _ => {}
            }
        }
    }
    // Normalize to toneless lowercase for pattern matching.
    for c in chars.iter_mut().take(len) {
        *c = lower(add_tone_to_char(*c, 0));
    }
    for i in 0..len.saturating_sub(1) {
        let (c1, c2) = (chars[i], chars[i + 1]);
        let is_pattern = UHO_PATTERNS.iter().any(|&(a, b)| c1 == a && c2 == b);
        if is_pattern && i + 2 < len && chars[i + 2].is_alphabetic() {
            return true;
        }
    }
    false
}

/// Checks if composition contains "ưo" or "ươ" pattern directly from transformations,
/// avoiding the expensive flatten + string search allocation.
fn contains_uho_in_composition(composition: &[Transformation]) -> bool {
    for i in 0..composition.len() {
        let t = &composition[i];
        if t.has_target() || t.effect_type != EffectType::Appending || t.key == '\0' {
            continue;
        }
        // Get the toneless result char.
        let c = add_tone_to_char(t.result, 0);
        if c == 'ư' {
            // Look ahead for 'o' or 'ơ'.
            for t2 in composition.iter().skip(i + 1) {
                if t2.has_target() || t2.effect_type != EffectType::Appending || t2.key == '\0' {
                    continue;
                }
                let c2 = add_tone_to_char(t2.result, 0);
                if c2 == 'o' || c2 == 'ơ' {
                    return true;
                }
                break; // Found next appending char, not o/ơ.
            }
        }
    }
    false
}

/// Generates a list of transformations to apply based on the current composition and rules.
pub(crate) fn generate_transformations(
    composition: &[Transformation],
    applicable_rules: &[Rule],
    config: Config,
    lower_key: char,
    is_upper_case: bool,
    out: &mut TransformationStack,
) {
    if let Some(last) = composition.last()
        && last.effect_type == EffectType::Appending
        && last.key == lower_key
        && last.key != last.result
    {
        out.push(Transformation::new(
            '\0',
            '\0',
            '\0',
            Some((composition.len() - 1) as u8),
            Mark::Raw as u8,
            EffectType::MarkTransformation,
            false,
        ));
    }

    // Single split per composition revision: threaded through every query
    // below instead of re-extracting per call.
    let cvc = extract_cvc_trans(composition);

    if let (Some(target), Some(applicable_rule)) =
        find_target(composition, applicable_rules, config, &cvc)
    {
        out.push(Transformation::from_rule(applicable_rule, Some(target), is_upper_case));

        if applicable_rule.effect_type != EffectType::MarkTransformation {
            if applicable_rule.effect_type == EffectType::ToneTransformation {
                for trans in composition {
                    if trans.effect_type == EffectType::ToneTransformation
                        && let Some(prev_target) = trans.target()
                        && prev_target != target
                        && trans.effect != 0
                    {
                        out.push(Transformation::new(
                            '\0',
                            '\0',
                            '\0',
                            Some(prev_target),
                            0,
                            EffectType::ToneTransformation,
                            false,
                        ));
                    }
                }
            }
            return;
        }

        let mut new_comp = [Transformation::default(); MAX_ACTIVE_TRANS];
        let base_len = composition.len();
        // Only attempt the validity check if there is room for one more transformation.
        if base_len < MAX_ACTIVE_TRANS {
            let new_len = base_len + 1;
            new_comp[..base_len].copy_from_slice(composition);
            new_comp[base_len] = out.as_slice()[0];

            // Only spread the mark when the syllable is incomplete without it:
            // "khuơ" stays as typed, while "huơu" needs the second horn.
            // `new_comp` is a new revision, so it gets its own split.
            if !is_valid(&new_comp[..new_len], true)
                && let (Some(target2), Some(mut virtual_rule)) = {
                    let new_cvc = extract_cvc_trans(&new_comp[..new_len]);
                    find_target(&new_comp[..new_len], applicable_rules, config, &new_cvc)
                }
            {
                virtual_rule.key = '\0';
                out.push(Transformation::from_rule(virtual_rule, Some(target2), false));
            }
        }
    } else {
        if contains_uho_in_composition(composition) {
            // Absolute index of the first appending vowel, collected by the
            // entry split. A value search here mis-hits on identical twins
            // (e.g. the fc 'i' shadows vo's 'i' in "gii...").
            let vo_abs = &cvc.vo_abs[..cvc.vo_abs_len as usize];
            if let Some(&target_idx) = vo_abs.first() {
                let trans = Transformation::new(
                    '\0',
                    '\0',
                    '\0',
                    Some(target_idx),
                    0,
                    EffectType::MarkTransformation,
                    false,
                );

                let mut tmp = [Transformation::default(); MAX_ACTIVE_TRANS];
                let base_len = composition.len();
                // Only attempt if there is room for one more transformation.
                if base_len < MAX_ACTIVE_TRANS {
                    let tmp_len = base_len + 1;
                    tmp[..base_len].copy_from_slice(composition);
                    tmp[base_len] = trans;

                    // `tmp` is a new revision, so it gets its own split.
                    if let (Some(target), Some(applicable_rule)) = {
                        let tmp_cvc = extract_cvc_trans(&tmp[..tmp_len]);
                        find_target(&tmp[..tmp_len], applicable_rules, config, &tmp_cvc)
                    } && target_idx != target
                    {
                        out.push(trans);
                        out.push(Transformation::from_rule(
                            applicable_rule,
                            Some(target),
                            is_upper_case,
                        ));
                        return;
                    }
                }
            }
        }

        generate_undo_transformations(composition, applicable_rules, config, out, &cvc);
        if !out.is_empty() {
            let has_raw_cancel = out.as_slice().iter().any(|t| {
                t.effect_type == EffectType::MarkTransformation && t.effect == Mark::Raw as u8
            });
            if !has_raw_cancel {
                out.push(new_appending_trans(lower_key, is_upper_case));
            }
        }
    }
}

/// Generates fallback transformations when no specific rules match.
pub(crate) fn generate_fallback_transformations(
    applicable_rules: &[Rule],
    lower_key: char,
    is_upper_case: bool,
    out: &mut TransformationStack,
) {
    let trans = generate_appending_trans(applicable_rules, lower_key, is_upper_case);
    out.push(trans);

    for rule in applicable_rules {
        if rule.key == lower_key && rule.effect_type == EffectType::Appending {
            for i in 0..rule.appended_len {
                let appended_char = rule.appended[i as usize];
                let _is_upper_case = is_upper_case || is_upper(appended_char);
                out.push(Transformation::new(
                    '\0',
                    lower(appended_char),
                    lower(appended_char),
                    None,
                    0,
                    EffectType::Appending,
                    _is_upper_case,
                ));
            }
            break;
        }
    }
}

/// "Breaks" a composition by converting all non-virtual transformations into simple appending ones.
pub(crate) fn break_composition_slice(
    composition: &[Transformation],
) -> [Transformation; MAX_ACTIVE_TRANS] {
    let mut result = [Transformation::default(); MAX_ACTIVE_TRANS];
    let mut len = 0;
    for trans in composition {
        if trans.key == '\0' {
            continue;
        }
        if len < MAX_ACTIVE_TRANS {
            result[len] = new_appending_trans(trans.key, trans.is_upper_case);
            len += 1;
        }
    }
    result
}

/// Updates the tone target in the composition based on the current syllable structure and tone style.
pub(crate) fn refresh_last_tone_target_into(composition: &mut [Transformation], std_style: bool) {
    // Cheap guard first: without a tone to move there is nothing to refresh.
    // This skips `extract_cvc_trans` on toneless compositions (same early exit
    // as before, only reordered).
    if get_last_tone_transformation(composition).is_none() {
        return;
    }
    let cvc = extract_cvc_trans(composition);
    refresh_with_cvc(composition, &cvc, std_style);
}

/// Tone-target refresh reusing a caller-provided breakdown (see
/// [`check_validity`]), saving a second `extract_cvc_trans` per keystroke on
/// the validate-then-refresh slow path.
pub(crate) fn refresh_with_cvc(composition: &mut [Transformation], cvc: &Cvc, std_style: bool) {
    let (new_tone_target, last_tone_idx) = {
        if cvc.vo_len == 0 {
            return;
        }

        let new_tone_target = find_tone_target(composition, cvc, std_style);

        let last_tone_idx = composition
            .iter()
            .enumerate()
            .rev()
            .find(|(_, t)| t.effect_type == EffectType::ToneTransformation && t.has_target())
            .map(|(i, _)| i);

        (new_tone_target, last_tone_idx)
    };

    let Some(last_idx) = last_tone_idx else { return };

    // Clear all earlier tone transformations in the composition so only the last tone is active.
    for (i, trans) in composition.iter_mut().enumerate() {
        if i != last_idx && trans.effect_type == EffectType::ToneTransformation {
            trans.effect = 0;
        }
    }

    let last_target = composition[last_idx].target();
    if last_target == new_tone_target {
        return;
    }

    composition[last_idx].set_target(new_tone_target);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn appending(ch: char) -> Transformation {
        new_appending_trans(ch, false)
    }

    fn mark_trans(target: u8) -> Transformation {
        Transformation::new(
            '\0',
            '\0',
            '\0',
            Some(target),
            Mark::Horn as u8,
            EffectType::MarkTransformation,
            false,
        )
    }

    /// `Cvc.vo_abs` pins the absolute composition indices of the appending
    /// vowels, in vo order. "giiưo": the gi-split moves the first 'i' into
    /// fc, so vo = [i#2, ư#3, o#4] — a value search for vo[0] mis-hits the
    /// fc twin at index 1, while the collected index is 2.
    #[test]
    fn cvc_vo_abs_indices() {
        let comp = [appending('g'), appending('i'), appending('i'), appending('ư'), appending('o')];
        let cvc = extract_cvc_trans(&comp);
        assert_eq!(&cvc.vo_abs[..cvc.vo_abs_len as usize], &[2u8, 3, 4]);
        // Structural invariant: vo_abs[i] indexes the vowel equal to vo[i].
        for (k, &abs) in cvc.vo_abs[..cvc.vo_abs_len as usize].iter().enumerate() {
            assert_eq!(comp[abs as usize], cvc.vo_slice()[k]);
        }

        // Real mobile shapes (precomposed ư): no twins, indices are direct.
        let comp2 = [appending('g'), appending('ư'), appending('o')];
        let cvc2 = extract_cvc_trans(&comp2);
        assert_eq!(&cvc2.vo_abs[..cvc2.vo_abs_len as usize], &[1u8, 2]);

        let comp3 = [appending('r'), appending('ư'), appending('ợ')];
        let cvc3 = extract_cvc_trans(&comp3);
        assert_eq!(&cvc3.vo_abs[..cvc3.vo_abs_len as usize], &[1u8, 2]);
    }

    /// Revision discipline: the threaded `&Cvc` must never cross a composition
    /// revision. `generate_transformations` (revision A) and `apply_auto_correct`
    /// (revision B, post-transform) each use their own split. Pinned through
    /// the public engine on sequences whose tone/mark placement depends on
    /// per-revision validity: "uwfw" keeps the tone (`ùw`), and "eete" +
    /// backspace + "e" rebuilds the mark (`êt`).
    #[test]
    fn cvc_shared_within_single_revision_only() {
        use crate::engine::Engine;
        use crate::input_method::preset::InputMethodPreset;
        use crate::mode::Mode;
        let mut e = Engine::from_preset(InputMethodPreset::Telex);
        for ch in "uwfw".chars() {
            e.process_key(ch, Mode::Vietnamese);
        }
        assert_eq!(e.output_str(), "ùw");

        let mut e2 = Engine::from_preset(InputMethodPreset::Telex);
        for ch in "eete".chars() {
            e2.process_key(ch, Mode::Vietnamese);
        }
        e2.remove_last_output_char();
        e2.process_key('e', Mode::Vietnamese);
        assert_eq!(e2.output_str(), "êt");
    }

    /// Ground truth: real Vietnamese ư-words typed with precomposed ư/ơ
    /// (mobile VN keyboards send these raw). This is the only input shape
    /// reaching the uho horn fallback with pre-resolved results; it must
    /// keep producing the exact words through the Cvc refactor.
    #[test]
    fn mobile_precomposed_uo_words() {
        use crate::engine::Engine;
        use crate::input_method::preset::InputMethodPreset;
        use crate::mode::Mode;
        let cases = [
            ("rượu", "rượu"),
            ("ngươi", "ngươi"),
            ("gương", "gương"),
            ("đường", "đường"),
            ("bươu", "bươu"),
            ("sườn", "sườn"),
            ("vươn", "vươn"),
            ("thương", "thương"),
        ];
        for (input, expected) in cases {
            let mut e = Engine::from_preset(InputMethodPreset::Telex);
            for ch in input.chars() {
                e.process_key(ch, Mode::Vietnamese);
            }
            assert_eq!(e.output_str(), expected, "mobile input {input:?}");
        }
    }

    /// Regression: `results[fc[0]]` / `results[vo[0]]` indexed `results[]` by
    /// absolute composition index instead of app_indices-space. Wrong whenever a
    /// non-appending transform precedes the first appending one.
    #[test]
    fn extract_cvc_appending_indices_with_leading_mark() {
        // Composition: [mark(target=1), g, i, a]
        //   app_indices = [1, 2, 3]  (absolute positions of appending transforms)
        //   results     = ['g', 'i', 'a']
        // Onset-split must read results[0]=='g' and results[1]=='i', not
        // results[1]=='i' / results[2]=='a' (the old absolute-index bug).
        let comp = [mark_trans(1), appending('g'), appending('i'), appending('a')];
        let app_indices: Vec<usize> = (0..comp.len()).filter(|&i| !comp[i].has_target()).collect();
        let (fc, vo, lc) = extract_cvc_appending_indices(&comp, &app_indices);

        // 'g' + 'i' + 'a': onset-split should absorb 'i' into onset ('gi'),
        // leaving 'a' as the vowel and nothing as the coda.
        assert_eq!(fc.len(), 2, "onset should be 'g'+'i' (absorbed by qi/gi rule)");
        assert_eq!(vo.len(), 1, "vowel should be 'a'");
        assert_eq!(lc.len(), 0);
    }

    #[test]
    fn extract_cvc_appending_indices_qu_onset() {
        // Composition: [mark(target=1), q, u, a]
        let comp = [mark_trans(1), appending('q'), appending('u'), appending('a')];
        let app_indices: Vec<usize> = (0..comp.len()).filter(|&i| !comp[i].has_target()).collect();
        let (fc, vo, lc) = extract_cvc_appending_indices(&comp, &app_indices);

        assert_eq!(fc.len(), 2, "onset should be 'q'+'u' (absorbed by qu rule)");
        assert_eq!(vo.len(), 1, "vowel should be 'a'");
        assert_eq!(lc.len(), 0);
    }

    /// The resume hint is a pure optimization: for any composition and any
    /// hint (real, stale, or adversarial garbage), the resumed result must
    /// equal the full scan.
    #[test]
    fn last_syllable_start_resume_matches_full_scan() {
        fn rng_next(state: &mut u64) -> u64 {
            *state = state.wrapping_mul(6364136229).wrapping_add(1442695041);
            *state >> 33
        }
        // Letter pool with word breaks (space, digits) mixed in.
        const KEYS: &[u8] = b"aeioubcdghklmnprstvxzaeioubcdghklmnprstvxz  012 settled";
        let mut state: u64 = 0x9E3779B97F4A7C15;
        for _ in 0..300 {
            let mut comp = Vec::new();
            let total = 1 + (rng_next(&mut state) as usize % MAX_ACTIVE_TRANS);
            while comp.len() < total {
                let r = rng_next(&mut state);
                if r.is_multiple_of(4) && !comp.is_empty() {
                    // Effect transformation targeting an existing index.
                    let target = (r as usize) % comp.len();
                    let is_tone = (r >> 8).is_multiple_of(2);
                    comp.push(Transformation::new(
                        '\0',
                        'a',
                        'a',
                        Some(target as u8),
                        (r >> 16) as u8 % 6,
                        if is_tone {
                            EffectType::ToneTransformation
                        } else {
                            EffectType::MarkTransformation
                        },
                        false,
                    ));
                } else {
                    let c = KEYS[(r as usize) % KEYS.len()] as char;
                    comp.push(appending(c));
                }
            }
            // Incremental typing with real chained hints.
            let mut hint = SyllHint::NONE;
            for end in 1..=comp.len() {
                let (full, _) = last_syllable_start(&comp[..end], SyllHint::NONE);
                let (resumed, next) = last_syllable_start(&comp[..end], hint);
                assert_eq!(resumed, full, "chained resume diverged for {comp:?}[..{end}]");
                hint = next;
            }
            let (full, _) = last_syllable_start(&comp, SyllHint::NONE);
            // Adversarial hints must fall back to the same result.
            for _ in 0..4 {
                let flag = rng_next(&mut state);
                let evil = SyllHint {
                    hash: rng_next(&mut state),
                    len: (rng_next(&mut state) % 20) as u8,
                    start: (rng_next(&mut state) % 20) as u8,
                    valid: flag.is_multiple_of(2),
                };
                let (resumed, _) = last_syllable_start(&comp, evil);
                assert_eq!(resumed, full, "evil hint {evil:?} diverged for {comp:?}");
            }
        }
    }

    #[test]
    fn extract_cvc_appending_indices_plain_no_leading_transform() {
        // Without a leading non-appending transform the old code also worked
        // (fc[0]==0 == app_indices[0]==0). Guard against regression.
        let comp = [appending('g'), appending('i'), appending('a')];
        let app_indices: Vec<usize> = (0..comp.len()).filter(|&i| !comp[i].has_target()).collect();
        let (fc, vo, lc) = extract_cvc_appending_indices(&comp, &app_indices);

        assert_eq!(fc.len(), 2);
        assert_eq!(vo.len(), 1);
        assert_eq!(lc.len(), 0);
    }

    /// The resume hint is a pure fast path: replaying from it must give the
    /// same syllable start as a full rescan on every keystroke shape
    /// (valid words, invalid tails, VNI digits, backspace, uppercase).
    #[test]
    fn syll_hint_matches_full_rescan() {
        use crate::engine::Engine;
        use crate::input_method::InputMethod;
        use crate::mode::Mode;

        const SEQS: &[&str] = &[
            "tieengs",
            "nguwowif",
            "dduwowngf",
            "khuyeens",
            "nghieengs",
            "truwowngf",
            "thuyeens",
            "mymfyk",
            "craxyuk",
            "enlf",
            "hoafn",
            "xre6po6c",
            "go366",
            "eete",
            "uwfw",
            "tieengsBB",
            "eeteBe",
            "ing3",
            "TIEENGS",
            "VieetjNam",
        ];
        for seq in SEQS {
            for im in [InputMethod::telex(), InputMethod::vni()] {
                let mut engine = Engine::new(im);
                let mut hint = SyllHint::NONE;
                for k in seq.chars() {
                    if k == 'B' {
                        engine.remove_last_output_char();
                    } else {
                        engine.process_key(k, Mode::Vietnamese);
                    }
                    let comp = engine.active_slice();
                    let (resumed, next) = last_syllable_start(comp, hint);
                    let (fresh, _) = last_syllable_start(comp, SyllHint::NONE);
                    assert_eq!(resumed, fresh, "hint diverged on {seq}");
                    hint = next;
                }
            }
        }
    }
}
