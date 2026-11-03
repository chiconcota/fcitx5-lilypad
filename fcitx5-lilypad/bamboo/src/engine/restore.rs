use super::Engine;
use crate::engine::state::TransformationStack;
use crate::mode::Mode;
use std::sync::Arc;

/// Desired representation of the active last word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastWordForm {
    /// Reapply the configured Vietnamese input method to the stored keystrokes.
    Vietnamese,
    /// Break transformations back into their raw keystrokes.
    RawKeystrokes,
}

impl Engine {
    /// Restores the last word in the composition to its un-transformed state.
    ///
    /// If `to_vietnamese` is true, it attempts to re-apply Vietnamese transformations.
    #[deprecated(
        since = "0.5.0",
        note = "use restore_last_word_as(LastWordForm) instead; planned removal in 1.0.0"
    )]
    pub fn restore_last_word(&mut self, to_vietnamese: bool) {
        let form =
            if to_vietnamese { LastWordForm::Vietnamese } else { LastWordForm::RawKeystrokes };
        self.restore_last_word_as(form);
    }

    /// Restores the active last word in the requested representation.
    pub fn restore_last_word_as(&mut self, form: LastWordForm) {
        let mut work = TransformationStack::new();

        self.take_active_into(&mut work);
        if work.is_empty() {
            self.set_active_from_stack(&mut work);
            self.current_state_id = 0;
            self.update_cached_output();
            return;
        }

        let (prev_slice, last) =
            crate::syllable::extract_last_word(work.as_slice(), Some(&self.input_method.keys));

        let mut previous = TransformationStack::new();
        previous.extend_from_slice(prev_slice);

        if last.is_empty() {
            self.set_active_from_stack(&mut work);
            self.current_state_id = 0;
            self.update_cached_output();
            return;
        }
        if form == LastWordForm::RawKeystrokes {
            previous.extend_from_slice(&crate::syllable::break_composition_slice(last));
            self.set_active_from_stack(&mut previous);
            self.current_state_id = 0;
            self.update_cached_output();
            return;
        }

        let mut new_comp = TransformationStack::new();
        if self.scratch_engine.is_none() {
            self.scratch_engine = Some(Box::new(Self::with_shared_rules(
                Arc::clone(&self.input_method),
                Arc::clone(&self.rules),
                self.config,
            )));
        }
        let Some(temp_engine) = self.scratch_engine.as_mut() else {
            return;
        };
        temp_engine.reset();

        for t in last {
            if t.key == '\0' {
                continue;
            }
            temp_engine.process_key(t.key, Mode::Vietnamese);
        }
        new_comp.extend_from_slice(temp_engine.active_slice());

        previous.extend_from_slice(new_comp.as_slice());

        self.set_active_from_stack(&mut previous);
        self.current_state_id = 0;
        self.update_cached_output();
    }
}
