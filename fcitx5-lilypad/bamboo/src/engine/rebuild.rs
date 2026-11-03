use super::Engine;
use crate::engine::state::{MAX_ACTIVE_TRANS, Transformation, TransformationStack};
use crate::input_method::EffectType;
use crate::mode::OutputOptions;
use crate::phonetics::{
    add_mark_to_char, add_mark_to_toneless_char, add_tone_to_char, find_mark_from_char,
    find_tone_from_char, is_upper, is_word_break_symbol, lower,
};

impl Engine {
    /// Resets the engine and loads `text` as if it had been typed, so that the
    /// next keys can change the tone and marks of its last word.
    ///
    /// Text up to the last word break is committed and the word after it
    /// becomes the active composition. A word too long for the composition is
    /// committed as is. Tones and marks are rebuilt without keystrokes, so
    /// [`restore_last_word`](Self::restore_last_word) gives back the letters.
    pub fn rebuild_from_text(&mut self, text: &str) {
        self.reset();
        let std_style = self.config.std_tone_style;
        let mut comp = TransformationStack::new();
        let mut word_start = 0;
        for (i, c) in text.char_indices() {
            if !is_word_break_symbol(c) {
                continue;
            }
            let word = &text[word_start..i];
            if build_word(word, std_style, &mut comp) {
                crate::flattener::append_flatten_slice(
                    comp.as_slice(),
                    OutputOptions::NONE,
                    &mut self.committed_text,
                );
                // Keep `committed_raw` in step like `commit()` does: the
                // original keystrokes are unknowable for pre-existing text,
                // so the rebuilt roots stand in for the raw keys.
                crate::flattener::append_raw_keys(comp.as_slice(), &mut self.committed_raw);
            } else {
                self.committed_text.push_str(word);
                self.committed_raw.push_str(word);
            }
            self.committed_text.push(c);
            self.committed_raw.push(c);
            word_start = i + c.len_utf8();
        }
        let word = &text[word_start..];
        if build_word(word, std_style, &mut comp) {
            self.set_active_from_stack(&mut comp);
        } else {
            self.committed_text.push_str(word);
            self.committed_raw.push_str(word);
        }
        self.update_cached_output();
    }
}

/// Decomposes `word` into appended letters, their marks and one tone, the way
/// Go bamboo-core `RebuildCompositionFromText` does. Returns false if it does
/// not fit in a composition.
fn build_word(word: &str, std_style: bool, out: &mut TransformationStack) -> bool {
    out.clear();
    let mut tone = 0;
    for c in word.chars() {
        let lower_c = lower(c);
        let char_tone = find_tone_from_char(lower_c);
        // Like Go, the mark is read from the toned char, so a toned ê or ư is
        // appended whole; keys then act on it as they do on a typed word.
        let mark = find_mark_from_char(lower_c).unwrap_or(0);
        let mut root = lower_c;
        if char_tone != 0 {
            root = add_tone_to_char(root, 0);
            tone = char_tone;
        }
        if mark != 0 {
            root = add_mark_to_char(root, 0);
        }
        if out.len() + 1 + usize::from(mark != 0) > MAX_ACTIVE_TRANS {
            return false;
        }
        let idx = out.len() as u8;
        out.push(Transformation::new(
            root,
            root,
            root,
            None,
            0,
            EffectType::Appending,
            is_upper(c),
        ));
        if mark != 0 {
            let result = add_mark_to_toneless_char(root, mark);
            out.push(Transformation::new(
                '\0',
                root,
                result,
                Some(idx),
                mark,
                EffectType::MarkTransformation,
                false,
            ));
        }
    }
    if tone != 0
        && let Some(target) = crate::syllable::tone_target(out.as_slice(), std_style)
    {
        if out.len() >= MAX_ACTIVE_TRANS {
            return false;
        }
        out.push(Transformation::new(
            '\0',
            '\0',
            '\0',
            Some(target),
            tone,
            EffectType::ToneTransformation,
            false,
        ));
    }
    true
}
