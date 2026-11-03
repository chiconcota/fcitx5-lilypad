//! The core engine that processes keypresses and maintains the IME state.

use crate::input_method::InputMethodPreset;
use std::sync::Arc;

use std::borrow::Cow;

use crate::config::Config;
use crate::input_method::{EffectType, InputMethod, Mark, Rule};
use crate::mode::{Mode, OutputOptions};
use crate::phonetics::{is_upper, lower, upper};

pub mod rebuild;
pub mod restore;
pub mod rules;
pub mod snapshot;
pub mod state;

pub use restore::LastWordForm;
pub use rules::EngineRules;
use snapshot::{FLAG_ENGLISH_BYPASS, FLAG_NEEDS_BUFFER, Snapshot};
#[allow(deprecated)]
pub use state::{MAX_ACTIVE_TRANS, RestoreMark, ToneRefresh, Transformation, TransformationStack};

/// Minimal suffix edit produced by processing one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditDelta<'a> {
    /// Number of Unicode scalar values to delete from the previous output suffix.
    pub delete_chars: usize,
    /// Number of UTF-8 bytes occupied by that deleted suffix.
    pub delete_bytes: usize,
    /// Replacement suffix to insert after the deletion.
    pub insert: &'a str,
}

/// The main stateful processor of the Vietnamese Input Method Engine.
///
/// It maintains an internal buffer of transformations and produces the correctly marked Vietnamese text.
/// The engine uses a hybrid approach combining a Rule Engine with a Lazy JIT DFA for peak performance.
#[derive(Debug)]
pub struct Engine {
    // Hot per-keystroke state first so it shares the first cache lines;
    // cold committed text and lazy boxes live at the tail.
    /// Stack-allocated buffer for the active composition: touched by every keystroke.
    active_buffer: [Transformation; MAX_ACTIVE_TRANS],
    /// Cached preedit string of the active composition for zero-allocation [`output()`](Self::output).
    cached_output: String,
    active_len: usize,
    /// Bit `i` = `active_buffer[i].is_upper_case`, maintained incrementally so
    /// the DFA-hit path and snapshots need no per-key fold. Invariant is
    /// checked by a `debug_assert` in [`Engine::update_cached_output`], which
    /// runs on every keystroke in test builds.
    active_upper_mask: u32,
    /// Snapshot stack for O(1) backspace — 8-byte metadata per slot, zero heap.
    snapshots: [Snapshot; MAX_ACTIVE_TRANS],
    snapshot_len: usize,
    current_state_id: u32,
    english_bypass: bool,
    /// True once the DFA refused a new composition (frozen at
    /// [`crate::dfa::DFA_MAX_STATES`]).
    ///
    /// Perf hint only: while detached, the JIT cache attempt is skipped and
    /// keystrokes take the rule-engine slow path (correct, uncached) until
    /// `commit`/`reset` or a backspace back into cached state.
    /// Correctness does NOT rely on this flag — the fast path is structurally
    /// guarded by `(active_len == 0 || current_state_id != 0)`, so a lookup
    /// from state 0 can only run on empty active text (true word start).
    dfa_detached: bool,
    dfa: crate::dfa::Dfa,
    /// Resume hint for `last_syllable_start`: consecutive slow-path keys see
    /// the same prefix plus a short tail, so only suffix windows are
    /// re-validated. Self-validating (stale ⇒ full rescan), 16 bytes.
    syll_hint: crate::syllable::SyllHint,

    pub(crate) input_method: Arc<InputMethod>,
    pub(crate) rules: Arc<EngineRules>,
    config: Config,

    committed_text: String,
    /// Keys behind `committed_text`, so `RAW | FULL_TEXT` can cover committed words.
    committed_raw: String,
    prev_preedit: String,
    delta_buf: String,
    /// Lazily-allocated side buffer for snapshots whose composition is not in
    /// the DFA (English bypass / fallback). Only written when needed.
    bypass_buffers: Option<Box<[[Transformation; MAX_ACTIVE_TRANS]; MAX_ACTIVE_TRANS]>>,

    /// Lazily-initialized scratch engine for `restore_last_word` to avoid repeated `with_config`.
    pub(crate) scratch_engine: Option<Box<Engine>>,
}

impl Engine {
    /// Creates a new engine with the specified input method and default configuration.
    pub fn new(input_method: InputMethod) -> Self {
        Self::with_config(input_method, Config::default())
    }

    /// Creates a new engine using a standard [`InputMethodPreset`] with zero-allocation rule sharing.
    pub fn from_preset(preset: InputMethodPreset) -> Self {
        Self::from_preset_with_config(preset, Config::default())
    }

    /// Creates a new engine using a standard [`InputMethodPreset`] and configuration with zero-allocation rule sharing.
    pub fn from_preset_with_config(preset: InputMethodPreset, config: Config) -> Self {
        let (im, rules) = crate::input_method::get_preset_shared(preset);
        Self::with_shared_rules(im, rules, config)
    }

    /// Creates a new engine with a specific input method and configuration.
    pub fn with_config(input_method: InputMethod, config: Config) -> Self {
        if let Some((preset_im, preset_rules)) =
            crate::input_method::find_preset_shared(&input_method)
        {
            return Self::with_shared_rules(preset_im, preset_rules, config);
        }
        let rules = Arc::new(EngineRules::from_input_method(&input_method));
        Self::with_shared_rules(Arc::new(input_method), rules, config)
    }

    pub(crate) fn with_shared_rules(
        input_method: Arc<InputMethod>,
        rules: Arc<EngineRules>,
        config: Config,
    ) -> Self {
        Self {
            committed_text: String::with_capacity(128),
            committed_raw: String::new(),
            cached_output: String::with_capacity(32),
            active_buffer: [Transformation::default(); MAX_ACTIVE_TRANS],
            active_len: 0,
            active_upper_mask: 0,
            input_method,
            rules,
            config,

            prev_preedit: String::with_capacity(32),
            delta_buf: String::with_capacity(32),
            dfa: crate::dfa::Dfa::new(),
            current_state_id: 0,

            snapshots: [Snapshot::default(); MAX_ACTIVE_TRANS],
            snapshot_len: 0,
            bypass_buffers: None,
            scratch_engine: None,
            syll_hint: crate::syllable::SyllHint::NONE,
            english_bypass: false,
            dfa_detached: false,
        }
    }

    #[inline]
    pub(crate) fn active_slice(&self) -> &[Transformation] {
        &self.active_buffer[..self.active_len]
    }

    pub(crate) fn take_active_into(&mut self, out: &mut TransformationStack) {
        out.clear();
        out.extend_from_slice(self.active_slice());
        self.active_len = 0;
        self.active_upper_mask = 0;
    }

    #[inline]
    fn update_cached_output(&mut self) {
        // The incremental mask must mirror the buffer on every keystroke;
        // this fires in test/debug builds if any mutation site forgets it.
        debug_assert_eq!(
            self.active_upper_mask,
            self.active_slice()
                .iter()
                .enumerate()
                .fold(0u32, |m, (i, t)| { if t.is_upper_case { m | (1u32 << i) } else { m } }),
            "active_upper_mask out of sync with active_buffer"
        );
        self.cached_output.clear();
        if self.active_len == 0 {
            return;
        }
        // P1 fast path: reuse cached lowercase flatten from DFA state and
        // re-apply case bits — avoids full effect-chain + PHF resolution.
        if self.current_state_id != 0 {
            let flat = self.dfa.get_flat_cached(self.current_state_id);
            if !flat.is_empty() {
                // DFA output is stored lowercase-canonical. The overwhelmingly
                // common path has no uppercase entries, so copy its UTF-8 in
                // one operation instead of decoding chars and walking the
                // transformation buffer merely to reproduce the same string.
                if self.active_upper_mask == 0 {
                    self.cached_output.push_str(flat);
                    return;
                }
                let mut flat_chars = flat.chars();
                for t in &self.active_buffer[..self.active_len] {
                    if t.effect_type == EffectType::Appending
                        && t.key != '\0'
                        && let Some(c) = flat_chars.next()
                    {
                        self.cached_output.push(if t.is_upper_case { upper(c) } else { c });
                    }
                }
                return;
            }
        }
        // Slow path: full flatten with effect-chain resolution.
        crate::flattener::append_flatten_slice(
            &self.active_buffer[..self.active_len],
            OutputOptions::NONE,
            &mut self.cached_output,
        );
    }

    pub(crate) fn set_active_from_stack(&mut self, src: &mut TransformationStack) {
        self.active_len = src.len().min(MAX_ACTIVE_TRANS);
        self.active_buffer[..self.active_len].copy_from_slice(src.as_slice());
        src.clear();
        // Cold paths only (slow key / restore / rebuild): recompute directly.
        self.active_upper_mask = self
            .active_slice()
            .iter()
            .enumerate()
            .fold(0u32, |m, (i, t)| if t.is_upper_case { m | (1u32 << i) } else { m });
    }

    #[inline]
    fn push_snapshot(&mut self) {
        if self.snapshot_len < MAX_ACTIVE_TRANS {
            // Empty actives are restored by length alone: skip the buffer copy
            // at word starts (`state_id == 0`), which used to copy the whole
            // active buffer on every first key of a word.
            let needs_buffer =
                self.english_bypass || (self.active_len > 0 && self.current_state_id == 0);
            let snap = &mut self.snapshots[self.snapshot_len];
            snap.state_id = self.current_state_id as u16;
            snap.active_len = self.active_len as u8;
            // Pre-key mask, maintained incrementally (no per-key fold).
            snap.upper_mask = self.active_upper_mask;
            snap.flags = if self.english_bypass { FLAG_ENGLISH_BYPASS } else { 0 }
                | if needs_buffer { FLAG_NEEDS_BUFFER } else { 0 };
            if needs_buffer {
                // Out-of-line so the 16 KB array temp never inflates the
                // inlined `push_snapshot` frame on the hot keystroke path.
                #[inline(never)]
                fn alloc_bypass_buffers()
                -> Box<[[Transformation; MAX_ACTIVE_TRANS]; MAX_ACTIVE_TRANS]> {
                    Box::new([[Transformation::default(); MAX_ACTIVE_TRANS]; MAX_ACTIVE_TRANS])
                }
                let bufs = self.bypass_buffers.get_or_insert_with(alloc_bypass_buffers);
                bufs[self.snapshot_len] = self.active_buffer;
            }
            self.snapshot_len += 1;
        }
    }

    #[inline]
    fn pop_snapshot(&mut self) -> Option<()> {
        if self.snapshot_len == 0 {
            return None;
        }
        self.snapshot_len -= 1;
        let snap = &self.snapshots[self.snapshot_len];
        self.current_state_id = snap.state_id as u32;
        self.english_bypass = snap.flags & FLAG_ENGLISH_BYPASS != 0;
        if snap.state_id != 0 {
            // Restored from the DFA arena (or a bypass buffer holding exactly
            // that state's composition): the state identifies the active
            // composition again, so fast-path lookups are sound.
            self.dfa_detached = false;
        } else if snap.active_len == 0 {
            // Restored to empty: state 0 soundly identifies empty composition.
            self.dfa_detached = false;
        }
        if snap.flags & FLAG_NEEDS_BUFFER != 0 {
            if let Some(bufs) = &self.bypass_buffers {
                self.active_buffer = bufs[self.snapshot_len];
            }
            self.active_len = snap.active_len as usize;
        } else if snap.state_id != 0 {
            // Restore from DFA arena + re-apply case bits.
            let comp = self.dfa.get_composition(snap.state_id as u32);
            let len = comp.len().min(MAX_ACTIVE_TRANS);
            self.active_buffer[..len].copy_from_slice(comp);
            self.active_len = len;
            for (i, t) in self.active_buffer[..len].iter_mut().enumerate() {
                t.is_upper_case = (snap.upper_mask >> i) & 1 != 0;
            }
        } else {
            self.active_len = snap.active_len as usize;
        }
        self.active_upper_mask = self
            .active_slice()
            .iter()
            .enumerate()
            .fold(0u32, |m, (i, t)| if t.is_upper_case { m | (1u32 << i) } else { m });
        Some(())
    }

    /// Returns the current configuration of the engine.
    pub const fn config(&self) -> Config {
        self.config
    }

    /// Updates the engine configuration.
    pub fn set_config(&mut self, config: Config) {
        if config == self.config {
            return;
        }
        self.config = config;
        // Cached transitions were computed with the old settings.
        self.dfa = crate::dfa::Dfa::new();
        self.current_state_id = 0;
        self.dfa_detached = false;
        self.snapshot_len = 0;
        self.scratch_engine = None;
    }

    /// Returns a reference to the current input method.
    #[must_use]
    pub fn input_method(&self) -> &InputMethod {
        &self.input_method
    }

    /// Warms up the DFA by pre-compiling common Vietnamese syllables.
    ///
    /// This API is intentionally unstable and currently uses a Telex-biased
    /// heuristic corpus. It can help long-lived Telex sessions, but may hurt
    /// cold-start latency or non-Telex/custom input methods.
    ///
    /// Prefer relying on the default lazy JIT behavior unless you have benchmark
    /// data for your production workload.
    #[deprecated(
        since = "0.3.4",
        note = "Engine::warm_up() is unstable and may be removed. It uses a Telex-biased heuristic and may regress cold-start or non-Telex workloads."
    )]
    pub fn warm_up(&mut self) {
        let mut compiler = crate::dfa::DfaCompiler::new(&self.input_method, self.config);
        compiler.compile_common();
        self.dfa = compiler.dfa;
        self.current_state_id = 0;
        self.dfa_detached = false;
        // Snapshot state IDs refer to the discarded DFA — restoring them would
        // read foreign compositions (or panic) in the replacement.
        self.snapshot_len = 0;
    }

    /// `lower_key` must already be lowercased (callers hold it); the extra
    /// `lower()` per slow keystroke is hoisted out.
    fn get_applicable_rules(&self, lower_key: char) -> &[Rule] {
        let key = lower_key;
        if key.is_ascii() {
            let (start, end) = self.rules.ascii_rule_indices[key as usize];
            &self.rules.all_rules[start as usize..end as usize]
        } else {
            self.rules
                .non_ascii_rule_indices
                .binary_search_by_key(&key, |(k, _)| *k)
                .map(|idx| {
                    let (start, end) = self.rules.non_ascii_rule_indices[idx].1;
                    &self.rules.all_rules[start as usize..end as usize]
                })
                .unwrap_or(&[])
        }
    }

    /// Returns true if `key` takes part in Vietnamese composition at this point, as
    /// opposed to being typed as is (and ending the word, for punctuation).
    #[must_use]
    pub fn can_process_key(&self, key: char) -> bool {
        let lower_key = lower(key);
        self.can_process_key_raw(lower_key)
            || self.bracket_vowel(lower_key, self.active_len == 0).is_some()
    }

    /// The vowel a bracket key types under [`crate::BracketMode`], if it applies here.
    fn bracket_vowel(&self, lower_key: char, at_word_start: bool) -> Option<char> {
        let vowel = match lower_key {
            '[' | '{' => 'ơ',
            ']' | '}' => 'ư',
            _ => return None,
        };
        let enabled = match self.config.bracket_mode {
            crate::BracketMode::Disabled => false,
            crate::BracketMode::NonStart => !at_word_start,
            crate::BracketMode::Everywhere => true,
        };
        // Input methods with their own bracket rules (Telex 2) keep them.
        (enabled && !self.is_input_method_key(lower_key)).then_some(vowel)
    }

    fn is_input_method_key(&self, lower_key: char) -> bool {
        (lower_key.is_ascii() && self.rules.ascii_effect_keys[lower_key as usize])
            || self.rules.non_ascii_effect_keys.binary_search(&lower_key).is_ok()
    }

    fn can_process_key_raw(&self, lower_key: char) -> bool {
        if crate::phonetics::is_alpha(lower_key) || self.is_input_method_key(lower_key) {
            return true;
        }
        if crate::phonetics::is_word_break_symbol(lower_key) {
            return false;
        }
        crate::phonetics::is_vietnamese_rune(lower_key)
    }

    fn generate_transformations(
        &self,
        composition: &mut TransformationStack,
        key: char,
        is_upper_case: bool,
    ) -> bool {
        let lower_key = lower(key);
        let rules = self.get_applicable_rules(lower_key);
        let mut trans_buf = TransformationStack::new();

        crate::syllable::generate_transformations(
            composition.as_slice(),
            rules,
            self.config,
            lower_key,
            is_upper_case,
            &mut trans_buf,
        );

        // `syllable` is empty only at the start of a word.
        let bracket = self.bracket_vowel(lower_key, composition.is_empty());
        // Stack-bounded: indices below truncate into `u8` targets.
        debug_assert!(composition.len() <= MAX_ACTIVE_TRANS);
        if let Some(vowel) = bracket
            && trans_buf.is_empty()
            && let Some(last) = composition.as_slice().last()
            && last.effect_type == EffectType::Appending
            && matches!(last.key, '[' | ']' | '{' | '}')
            && last.result == vowel
        {
            // `[{` undoes like `[[`; the generic undo only matches the same key.
            trans_buf.push(Transformation::new(
                '\0',
                '\0',
                '\0',
                Some((composition.len() - 1) as u8),
                Mark::Raw as u8,
                EffectType::MarkTransformation,
                false,
            ));
        }

        if trans_buf.is_empty() {
            crate::syllable::generate_fallback_transformations(
                rules,
                lower_key,
                is_upper_case,
                &mut trans_buf,
            );
            if let Some(vowel) = bracket
                && let Some(first) = trans_buf.as_mut_slice().first_mut()
            {
                first.result = vowel;
                first.effect_on = vowel;
            }
            if lower_key == 'w'
                && self.w2u_applies(composition.as_slice())
                && let Some(first) = trans_buf.as_mut_slice().first_mut()
                && first.result == 'w'
            {
                first.result = 'ư';
                first.effect_on = 'ư';
            }
        }

        // Any key, not only a letter, can complete "uơ"/"ưo" + letter (e.g. a
        // horn on "luộc"), so the second horn is checked after every key.
        self.maybe_apply_uho_horn(composition, &mut trans_buf);

        let has_undo = trans_buf.as_slice().iter().any(|t| {
            t.has_target()
                && ((t.effect_type == EffectType::ToneTransformation && t.effect == 0)
                    || (t.effect_type == EffectType::MarkTransformation
                        && (t.effect == 0 || t.effect == Mark::Raw as u8)))
        });

        composition.extend_from_slice(trans_buf.as_slice());
        if self.config.free_tone_marking {
            // `check_validity` hands back the tone-check breakdown so the
            // refresh below reuses a single `extract_cvc_trans` per keystroke.
            let (valid, tone_cvc) = crate::syllable::check_validity(composition.as_slice(), false);
            if valid && let Some(cvc) = tone_cvc {
                crate::syllable::refresh_with_cvc(
                    composition.as_mut_slice(),
                    &cvc,
                    self.config.std_tone_style,
                );
            }
        }
        has_undo
    }

    /// Spreads a horn mark across a `uơ`/`ưo` + letter tail when the input
    /// method defines super keys (Telex `w`). Runs after every key because a
    /// mark key can complete the pattern, not just a letter.
    fn maybe_apply_uho_horn(
        &self,
        composition: &TransformationStack,
        trans_buf: &mut TransformationStack,
    ) {
        let combined_len = composition.len() + trans_buf.len();
        if combined_len > MAX_ACTIVE_TRANS || self.input_method.super_keys.is_empty() {
            return;
        }
        let mut tmp_data = [Transformation::default(); MAX_ACTIVE_TRANS];
        tmp_data[..composition.len()].copy_from_slice(composition.as_slice());
        tmp_data[composition.len()..combined_len].copy_from_slice(trans_buf.as_slice());

        if crate::syllable::uho_tail_match_composition(&tmp_data[..combined_len]) {
            let tmp_cvc = crate::syllable::extract_cvc_trans(&tmp_data[..combined_len]);
            let (target, rule) = crate::syllable::find_target(
                &tmp_data[..combined_len],
                self.get_applicable_rules(lower(self.input_method.super_keys[0])),
                self.config,
                &tmp_cvc,
            );
            if let (Some(target), Some(mut rule)) = (target, rule) {
                rule.key = '\0';
                trans_buf.push(Transformation::from_rule(rule, Some(target), false));
            }
        }
    }

    // `syllable` is the composition before the key, so empty means the key starts a syllable.
    const fn w2u_applies(&self, syllable: &[Transformation]) -> bool {
        match self.config.w2u_mode {
            crate::W2uMode::Disabled => false,
            crate::W2uMode::NonStart => !syllable.is_empty(),
            crate::W2uMode::Everywhere => true,
        }
    }

    fn new_composition_in_place(
        &mut self,
        composition: &mut TransformationStack,
        scratch: &mut TransformationStack,
        key: char,
        is_upper_case: bool,
    ) -> bool {
        // Stack-bounded: target indices below truncate into `u8`.
        debug_assert!(composition.len() <= MAX_ACTIVE_TRANS);
        let (syllable_abs_start, hint) =
            crate::syllable::last_syllable_start(composition.as_slice(), self.syll_hint);
        self.syll_hint = hint;

        composition.drain_to(syllable_abs_start, scratch);

        let offset = syllable_abs_start;
        if offset != 0 {
            for t in scratch.as_mut_slice().iter_mut() {
                if let Some(target) = t.target() {
                    t.set_target(Some(target.saturating_sub(offset as u8)));
                }
            }
        }

        let has_undo = self.generate_transformations(scratch, key, is_upper_case);

        if offset != 0 {
            for t in scratch.as_mut_slice().iter_mut() {
                if let Some(target) = t.target() {
                    t.set_target(Some(target + offset as u8));
                }
            }
        }

        composition.extend_from_slice(scratch.as_slice());
        has_undo
    }
    /// Processes a string of characters and returns the resulting active word.
    ///
    /// This is a convenience wrapper around [`Self::process_str`] and [`Self::output`].
    ///
    /// # ⚠️ Not Recommended for Production
    ///
    /// This method is primarily intended for **testing and convenience purposes**.
    /// For production IME integration, use:
    /// - [`process_key`](Self::process_key) or [`process_key_edit`](Self::process_key_edit) for real-time input
    /// - [`process_str`](Self::process_str) + [`output`](Self::output) for batch processing
    ///
    /// This method may be deprecated or removed in a future version.
    #[must_use]
    pub fn process(&mut self, s: &str, mode: Mode) -> String {
        self.process_str(s, mode).output().into_owned()
    }

    /// Processes a string of characters and returns a reference to the engine.
    pub fn process_str(&mut self, s: &str, mode: Mode) -> &Self {
        for key in s.chars() {
            self.process_key(key, mode);
        }
        self
    }

    fn lcp_chars_and_bytes(a: &str, b: &str) -> (usize, usize) {
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        let min_len = a_bytes.len().min(b_bytes.len());
        let mut lcp_bytes = 0;

        // 8-byte chunk comparison
        while lcp_bytes + 8 <= min_len {
            let chunk_a = u64::from_ne_bytes(
                a_bytes[lcp_bytes..lcp_bytes + 8]
                    .try_into()
                    .expect("lcp_bytes + 8 <= min_len guaranteed by loop bound"),
            );
            let chunk_b = u64::from_ne_bytes(
                b_bytes[lcp_bytes..lcp_bytes + 8]
                    .try_into()
                    .expect("lcp_bytes + 8 <= min_len guaranteed by loop bound"),
            );
            let diff = chunk_a ^ chunk_b;
            if diff != 0 {
                #[cfg(target_endian = "little")]
                let mismatch_byte = (diff.trailing_zeros() / 8) as usize;
                #[cfg(target_endian = "big")]
                let mismatch_byte = (diff.leading_zeros() / 8) as usize;
                lcp_bytes += mismatch_byte;
                break;
            }
            lcp_bytes += 8;
        }

        while lcp_bytes < min_len && a_bytes[lcp_bytes] == b_bytes[lcp_bytes] {
            lcp_bytes += 1;
        }

        while lcp_bytes > 0 && !a.is_char_boundary(lcp_bytes) {
            lcp_bytes -= 1;
        }

        let prefix = &a[..lcp_bytes];
        let lcp_chars = if prefix.is_ascii() {
            lcp_bytes
        } else {
            prefix.as_bytes().iter().filter(|&&b| (b & 0xC0) != 0x80).count()
        };
        (lcp_chars, lcp_bytes)
    }

    /// Processes a single key and returns a **3-way diff** for efficient text editor updates.
    ///
    /// Instead of rewriting the entire preedit, the frontend only needs to apply:
    /// 1. Keep the common prefix unchanged.
    /// 2. Delete `backspace_count` characters from the end of the previous preedit.
    /// 3. Append `inserted_suffix`.
    ///
    /// ```text
    /// previous_preedit = [common_prefix] + [backspace_count chars to delete]
    /// new_preedit      = [common_prefix] + [inserted_suffix]
    /// ```
    ///
    /// The common prefix length is implicit: `previous_preedit.len() - backspace_count`
    /// (in characters). The frontend does not need to compute LCP/LCS — the engine does it.
    ///
    /// # Returns
    ///
    /// `(backspace_count, backspaces_bytes, inserted_suffix)`:
    /// - `backspace_count`: Number of **characters** to delete from the end of the previous preedit.
    /// - `backspaces_bytes`: Number of **UTF-8 bytes** to delete (for byte-oriented editors).
    /// - `inserted_suffix`: The new string to append after deletion.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bamboo_core::{Engine, Mode, InputMethod};
    ///
    /// let mut engine = Engine::new(InputMethod::telex());
    ///
    /// let edit = engine.process_key_edit('a', Mode::Vietnamese);
    /// assert_eq!(edit.delete_chars, 0);
    /// assert_eq!(edit.insert, "a");
    ///
    /// let edit = engine.process_key_edit('s', Mode::Vietnamese);
    /// // previous = "a", new = "á"
    /// // keep prefix = 1 - 1 = 0, delete = 1 ("a"), insert = "á"
    /// assert_eq!(edit.delete_chars, 1);
    /// assert_eq!(edit.insert, "á");
    /// ```
    #[must_use]
    pub fn process_key_edit(&mut self, key: char, mode: Mode) -> EditDelta<'_> {
        self.process_key(key, mode);

        let (_prefix_len, lcp_bytes) =
            Self::lcp_chars_and_bytes(&self.prev_preedit, &self.cached_output);

        let prev_bytes = self.prev_preedit.len();

        // Count only the suffix chars after the common prefix — O(suffix_len) instead of O(total).
        let backspace_count = self.prev_preedit[lcp_bytes..].chars().count();
        let backspaces_bytes = prev_bytes.saturating_sub(lcp_bytes);

        self.delta_buf.clear();
        self.delta_buf.push_str(&self.cached_output);
        std::mem::swap(&mut self.prev_preedit, &mut self.delta_buf);
        let inserted_suffix = &self.prev_preedit[lcp_bytes..];
        EditDelta {
            delete_chars: backspace_count,
            delete_bytes: backspaces_bytes,
            insert: inserted_suffix,
        }
    }

    /// Processes one key and returns the changed output suffix as a legacy tuple.
    #[deprecated(
        since = "0.5.0",
        note = "use process_key_edit() and named EditDelta fields instead; planned removal in 1.0.0"
    )]
    #[must_use]
    pub fn process_key_delta(&mut self, key: char, mode: Mode) -> (usize, usize, &str) {
        let edit = self.process_key_edit(key, mode);
        (edit.delete_chars, edit.delete_bytes, edit.insert)
    }

    /// Similar to [`Self::process_key_edit`], but writes the inserted string into a provided buffer.
    ///
    /// # Returns
    /// `backspace_count` — number of characters to delete from the end of the previous preedit.
    #[must_use]
    pub fn process_key_edit_into(&mut self, key: char, mode: Mode, inserted: &mut String) -> usize {
        let edit = self.process_key_edit(key, mode);
        inserted.clear();
        inserted.push_str(edit.insert);
        edit.delete_chars
    }

    /// Processes one key into a caller-provided insertion buffer using the legacy name.
    #[deprecated(
        since = "0.5.0",
        note = "use process_key_edit_into() instead; planned removal in 1.0.0"
    )]
    #[must_use]
    pub fn process_key_delta_into(
        &mut self,
        key: char,
        mode: Mode,
        inserted: &mut String,
    ) -> usize {
        self.process_key_edit_into(key, mode, inserted)
    }

    /// Processes a single character.
    ///
    /// The `mode` determines whether to apply Vietnamese transformation rules.
    ///
    /// Tone keys always override the previous tone on the same vowel (the last
    /// tone key wins, e.g. `looixfsx` → `lỗi`). Typing the same tone key twice
    /// in a row undoes the tone and types the key as a literal letter
    /// (e.g. `ass` → `as`).
    pub fn process_key(&mut self, key: char, mode: Mode) {
        if !self.process_key_internal(key, mode) {
            self.update_cached_output();
        }
    }

    /// Returns `true` when the preedit cache is already up to date (the key
    /// went through the raw-append path, which maintains it incrementally).
    fn process_key_internal(&mut self, key: char, mode: Mode) -> bool {
        let lower_key = lower(key);
        let is_upper_case = is_upper(key);

        // English mode or English bypass active: skip all Vietnamese processing.
        // Direct buffer append — no DFA lookup, snapshot saved for backspace.
        if mode == Mode::English || self.english_bypass {
            self.push_english_append(lower_key, is_upper_case, mode);
            return true;
        }

        let bracket = self.bracket_vowel(lower_key, self.active_len == 0);
        // `{` and `}` are the shifted brackets, so they type capitals.
        let is_upper_case = is_upper_case || (bracket.is_some() && matches!(lower_key, '{' | '}'));

        // DFA fast path: returns true when a cached transition handled the key.
        if self.try_dfa_hit(lower_key, is_upper_case) {
            return false;
        }

        // Slow path: validate key and handle word breaks
        if bracket.is_none() && !self.can_process_key_raw(lower_key) {
            self.handle_word_break_key(lower_key, is_upper_case);
            return false;
        }

        self.process_key_slow(lower_key, is_upper_case);
        false
    }

    /// Rule-engine slow path for keys the DFA could not resolve.
    ///
    /// Kept out of [`Engine::process_key_internal`] (and never inlined) so the
    /// hot DFA-hit path never pays the stack frame of the two scratch
    /// [`TransformationStack`]s.
    #[inline(never)]
    fn process_key_slow(&mut self, lower_key: char, is_upper_case: bool) {
        // Snapshot only needed before slow-path mutations (new_composition_in_place).
        // Window full: silently commit the head first so no key is ever dropped
        // (TransformationStack::push would discard past MAX_ACTIVE_TRANS).
        if self.active_len >= MAX_ACTIVE_TRANS {
            self.commit();
        }
        self.push_snapshot();

        // The JIT edge cur--key-->next below is only sound when `cur`
        // identifies the pre-key active composition (case-insensitive: the DFA
        // is lowercase-canonical). Residue states — frozen refusal, digit /
        // non-ASCII / English residue, retargeted tone — must not link edges
        // (e.g. a word-start edge must never point at a mid-word composition).
        // Slow path only: one memcmp over <=32 transformations.
        let prefix_known = if self.active_len == 0 {
            // Word start: only state 0 identifies empty text.
            self.current_state_id == 0
        } else {
            !self.dfa_detached
                && self.current_state_id != 0
                && self
                    .dfa
                    .composition_matches_canonical(self.current_state_id, self.active_slice())
        };

        // Local scratch stacks: the previous persisted copies were never
        // read (every use clears first via `take_active_into`/`drain_to`),
        // so keeping them as `Engine` fields only cost 528 B of footprint.
        let mut work = TransformationStack::new();
        let mut scratch = TransformationStack::new();

        self.take_active_into(&mut work);
        let has_undo =
            self.new_composition_in_place(&mut work, &mut scratch, lower_key, is_upper_case);
        if has_undo {
            // Only lock into english bypass if the result of the undo is NOT a valid Vietnamese prefix.
            // This allows words like "thoòng" (typed as "thooongf") to continue receiving tones
            // ("oo" is a valid nucleus in VO_2), while "res" after undo still falls back to English.
            if !self.is_valid_internal(work.as_slice(), false) {
                // Keep the tone: "uwfw" is "ùw", which callers can tell apart
                // from plain "uw" and show as the raw keys instead.
                self.english_bypass = true;
            }
        }

        // Real-time Auto-restore when auto_correct is enabled:
        self.apply_auto_correct(&mut work, has_undo);

        // Try to update DFA (Lazy JIT).
        // Always cache using lowercase key so uppercase keys reuse the same DFA transitions.
        // Skipped while detached: the DFA is frozen, slow path stays correct uncached.
        self.link_jit_state(lower_key, &work, &mut scratch, prefix_known);

        self.set_active_from_stack(&mut work);
    }

    /// English-mode (or bypassed) key: appends raw, committing on word breaks.
    ///
    /// Maintains `cached_output` incrementally (O(1) per key) so long raw runs
    /// do not re-flatten the whole preedit on every keystroke.
    fn push_english_append(&mut self, lower_key: char, is_upper_case: bool, mode: Mode) {
        // VNI tone keys are digits: they belong to the word, so restore,
        // raw output and backspace must still reach them.
        let ends_word = crate::phonetics::is_word_break_symbol(lower_key)
            && !self.is_input_method_key(lower_key);
        if ends_word && self.active_len > 0 {
            self.commit();
        }
        if self.active_len >= MAX_ACTIVE_TRANS {
            self.commit();
        }
        if mode != Mode::English && self.active_len > 0 {
            self.push_snapshot();
        }
        if is_upper_case {
            self.active_upper_mask |= 1u32 << self.active_len;
        }
        self.active_buffer[self.active_len] =
            crate::syllable::new_appending_trans(lower_key, is_upper_case);
        self.active_len += 1;
        if ends_word {
            self.commit();
        } else {
            // Pure append: the preedit grows by exactly one cased char.
            // Any `commit()` above already cleared `cached_output`.
            self.cached_output.push(if is_upper_case { upper(lower_key) } else { lower_key });
        }
        self.current_state_id = 0;
    }

    /// DFA fast path: applies a cached transition when one exists.
    ///
    /// Uses the lowercase key so uppercase shares the same DFA cache.
    /// Structural guard: state 0 only identifies empty active text. With
    /// non-empty residue (frozen/refused composition, digit residue from
    /// `push_active`, non-ASCII/English residue) a lookup from state 0
    /// could false-hit a word-start edge and wipe the composition, so those
    /// keys take the slow path. Skipped while detached (perf: the frozen
    /// DFA would miss anyway).
    ///
    /// Returns true when a cached transition handled the key.
    fn try_dfa_hit(&mut self, lower_key: char, is_upper_case: bool) -> bool {
        if self.dfa_detached
            || !lower_key.is_ascii()
            || (self.active_len != 0 && self.current_state_id == 0)
        {
            return false;
        }
        let next_state_id =
            self.dfa.get_state(self.current_state_id).get_transition(lower_key as u8);
        if next_state_id == 0 {
            return false;
        }
        self.push_snapshot();
        let prev_len = self.active_len;
        // Pre-key mask, maintained incrementally (no per-key fold).
        let prev_upper = self.active_upper_mask;

        self.current_state_id = next_state_id;
        let comp = self.dfa.get_composition(next_state_id);
        self.active_len = comp.len().min(MAX_ACTIVE_TRANS);
        self.active_buffer[..self.active_len].copy_from_slice(comp);

        if prev_upper != 0 {
            for i in 0..prev_len.min(self.active_len) {
                self.active_buffer[i].is_upper_case = (prev_upper >> i) & 1 != 0;
            }
        }
        // Maintain the mask with bit ops: keep shared prefix bits, set the
        // new tail when the fresh key is uppercase.
        let shared = prev_len.min(self.active_len);
        let keep = if shared >= 32 { u32::MAX } else { (1u32 << shared) - 1 };
        let mut new_mask = prev_upper & keep;
        if is_upper_case && prev_len <= self.active_len {
            for (k, t) in self.active_buffer[prev_len..self.active_len].iter_mut().enumerate() {
                t.is_upper_case = true;
                new_mask |= 1u32 << (prev_len + k);
            }
        }
        self.active_upper_mask = new_mask;
        true
    }

    /// Non-processable key on the slow path: snapshots, appends raw, and
    /// commits on word breaks.
    fn handle_word_break_key(&mut self, lower_key: char, is_upper_case: bool) {
        if crate::phonetics::is_word_break_symbol(lower_key) {
            self.commit();
        }
        // Snapshot before push_active so backspace can restore previous state.
        // Word breaks trigger commit() which clears snapshots — that's correct
        // (committed text can't be undone via backspace).
        self.push_snapshot();
        let trans = crate::syllable::new_appending_trans(lower_key, is_upper_case);
        self.push_active(trans);
        if crate::phonetics::is_word_break_symbol(lower_key) {
            self.commit();
        }
        self.current_state_id = 0;
    }

    /// Restores a mistyped word to raw keys when `auto_correct` is on and the
    /// result is not a valid Vietnamese prefix (no-op otherwise).
    fn apply_auto_correct(&mut self, work: &mut TransformationStack, has_undo: bool) {
        if has_undo || !self.config.auto_correct {
            return;
        }
        let has_transforms =
            work.as_slice().iter().any(|t| t.has_target() || (t.key != '\0' && t.result != t.key));

        if has_transforms && !self.is_valid_internal(work.as_slice(), false) {
            let raw_comp = crate::syllable::break_composition_slice(work.as_slice());
            let raw_len =
                work.as_slice().iter().filter(|t| t.key != '\0').count().min(MAX_ACTIVE_TRANS);
            work.clear();
            work.extend_from_slice(&raw_comp[..raw_len]);
            self.english_bypass = true;
        }
    }

    /// Links the post-key composition into the JIT trie (or resolves the
    /// current state when detached/bypassed), using lowercase-canonical keys
    /// so `'a'` and `'A'` share transitions.
    fn link_jit_state(
        &mut self,
        lower_key: char,
        work: &TransformationStack,
        scratch: &mut TransformationStack,
        prefix_known: bool,
    ) {
        if self.english_bypass
            || self.dfa_detached
            || !lower_key.is_ascii()
            || work.len() > MAX_ACTIVE_TRANS
        {
            // Canonical (lowercase) lookup: the map stores lowercase-canonical
            // compositions, so work carrying uppercase bits would miss an
            // existing state on a raw `find_state`.
            let mut lower = *work;
            for t in lower.as_mut_slice() {
                t.is_upper_case = false;
            }
            self.current_state_id = self.dfa.find_state(lower.as_slice()).unwrap_or(0);
            return;
        }
        // For uppercase: create a lowercase copy of the composition for DFA caching.
        // This ensures both 'a' and 'A' share the same DFA transition from the same state.
        let cache_comp = if work.as_slice().iter().any(|t| t.is_upper_case) {
            scratch.clear();
            scratch.extend_from_slice(work.as_slice());
            for t in scratch.as_mut_slice() {
                t.is_upper_case = false;
            }
            scratch.as_slice()
        } else {
            work.as_slice()
        };
        let next_id = self.dfa.add_state(cache_comp);
        if next_id != 0 {
            // Link only on a known prefix; otherwise `current_state_id`
            // (e.g. state 0 over residue) does not describe the pre-key
            // composition and the edge would corrupt the trie. The new
            // state itself still identifies the post-key active text, so
            // `current_state_id` is always updated.
            if prefix_known {
                self.dfa.states[self.current_state_id as usize]
                    .set_transition(lower_key as u8, next_id);
            }
            self.current_state_id = next_id;
        } else if self.dfa.is_full() {
            // DFA frozen: detach so later keys take the slow path instead
            // of false-hitting word-start edges from state 0. Snapshots
            // already cover backspace via bypass_buffers (current == 0).
            self.dfa_detached = true;
            self.current_state_id = self.dfa.find_state(cache_comp).unwrap_or(0);
        } else {
            self.current_state_id = self.dfa.find_state(cache_comp).unwrap_or(0);
        }
    }

    fn push_active(&mut self, trans: Transformation) {
        if self.active_len >= MAX_ACTIVE_TRANS {
            // Buffer full, auto-commit to make room
            self.commit();
        }
        if trans.is_upper_case {
            self.active_upper_mask |= 1u32 << self.active_len;
        }
        self.active_buffer[self.active_len] = trans;
        self.active_len += 1;
        self.current_state_id = self.dfa.find_state(self.active_slice()).unwrap_or(0);
    }

    /// Clears the active syllable buffer and appends it to the committed text.
    pub fn commit(&mut self) {
        if self.active_len == 0 {
            return;
        }
        crate::flattener::append_flatten_slice(
            &self.active_buffer[..self.active_len],
            OutputOptions::NONE,
            &mut self.committed_text,
        );
        crate::flattener::append_raw_keys(
            &self.active_buffer[..self.active_len],
            &mut self.committed_raw,
        );
        self.cached_output.clear();
        self.active_len = 0;
        self.active_upper_mask = 0;
        self.current_state_id = 0;
        self.snapshot_len = 0;
        self.english_bypass = false;
        self.dfa_detached = false;
    }

    /// Returns the currently active syllable as a borrowed string slice.
    ///
    /// This performs **zero heap allocations**, borrowing directly from the engine's internal preedit cache.
    #[inline]
    #[must_use]
    pub fn output_str(&self) -> &str {
        &self.cached_output
    }

    /// Returns the currently active syllable as a string slice or owned string.
    ///
    /// This returns a zero-allocation [`Cow::Borrowed`] pointing to the internal preedit cache.
    #[inline]
    #[must_use]
    pub fn output(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.cached_output)
    }

    /// Returns the processed string as a [`Cow<str>`] according to the specified options.
    ///
    /// Avoids heap allocations when the active buffer is empty or directly borrowable.
    #[must_use]
    pub fn get_processed_str_cow(&self, options: OutputOptions) -> Cow<'_, str> {
        let active = self.active_slice();
        if options.contains(OutputOptions::FULL_TEXT) {
            // Like Go, FULL_TEXT ignores PUNCTUATION_MODE.
            let options = options - OutputOptions::FULL_TEXT - OutputOptions::PUNCTUATION_MODE;
            let committed = if options.contains(OutputOptions::RAW) {
                &self.committed_raw
            } else {
                &self.committed_text
            };
            let char_options = options - OutputOptions::RAW;
            if active.is_empty() && char_options.is_empty() {
                return Cow::Borrowed(committed);
            }
            let mut result = String::with_capacity(committed.len() + self.cached_output.len());
            crate::flattener::append_text_with_options(committed, char_options, &mut result);
            if options == OutputOptions::NONE {
                result.push_str(&self.cached_output);
            } else {
                crate::flattener::append_flatten_slice(active, options, &mut result);
            }
            return Cow::Owned(result);
        }
        if options.contains(OutputOptions::PUNCTUATION_MODE) {
            if active.is_empty() {
                return Cow::Borrowed("");
            }
            let (_, tail) = crate::syllable::extract_last_word_with_punctuation_marks(
                active,
                &self.input_method.keys,
            );
            return Cow::Owned(crate::flattener::flatten_slice(tail, OutputOptions::NONE));
        }
        if active.is_empty() {
            Cow::Borrowed("")
        } else if options == OutputOptions::NONE {
            Cow::Borrowed(&self.cached_output)
        } else if options == OutputOptions::LOWER_CASE
            && self.current_state_id != 0
            && !self.dfa.get_flat_cached(self.current_state_id).is_empty()
        {
            // The DFA caches the lowercase flatten of the canonical
            // composition, which is exactly `LOWER_CASE` output (case bits do
            // not matter once lowered). Polling readers such as per-keystroke
            // validity checks borrow it with zero allocation or flattening.
            Cow::Borrowed(self.dfa.get_flat_cached(self.current_state_id))
        } else {
            Cow::Owned(crate::flattener::flatten_slice(active, options))
        }
    }

    /// Returns the processed string according to the specified options.
    ///
    /// This can be used to get the full text (committed + active) or variations like toneless text.
    ///
    /// Polling callers (e.g. per-keystroke validity checks) should prefer
    /// [`get_processed_str_cow`](Self::get_processed_str_cow): it borrows the
    /// cached output for `NONE`, `FULL_TEXT`-without-options and warmed
    /// `LOWER_CASE` reads instead of allocating.
    pub fn get_processed_str(&self, options: OutputOptions) -> String {
        self.get_processed_str_cow(options).into_owned()
    }

    /// Checks if the current composition forms a complete Vietnamese syllable.
    #[must_use]
    pub fn is_valid_word(&self) -> bool {
        self.is_valid_internal(self.active_slice(), true)
    }

    /// Checks if the current composition is a valid, potentially incomplete syllable prefix.
    #[must_use]
    pub fn is_valid_prefix(&self) -> bool {
        self.is_valid_internal(self.active_slice(), false)
    }

    /// Checks syllable validity using the legacy completeness boolean.
    #[deprecated(
        since = "0.5.0",
        note = "use is_valid_word() or is_valid_prefix() instead; planned removal in 1.0.0"
    )]
    #[must_use]
    pub fn is_valid(&self, input_is_full_complete: bool) -> bool {
        self.is_valid_internal(self.active_slice(), input_is_full_complete)
    }

    // Like Go's RemoveLastChar: an invalid word keeps its tone where it was
    // typed, so deleting a key brings back the text shown before that key.
    fn may_move_tone(&self) -> bool {
        // Refresh is a no-op without a tone to move (mirrors
        // `refresh_last_tone_target_into`'s early exit on a missing tone
        // transformation), so the spelling check is skipped and backspacing a
        // toneless word stays on the previous fast path.
        self.config.free_tone_marking
            && self.active_len > 0
            && self
                .active_slice()
                .iter()
                .any(|t| t.effect_type == EffectType::ToneTransformation && t.has_target())
            && self.is_valid_prefix()
    }

    fn is_valid_internal(
        &self,
        composition: &[Transformation],
        input_is_full_complete: bool,
    ) -> bool {
        crate::syllable::is_valid(composition, input_is_full_complete)
    }

    /// Undoes the most recent physical key and recomputes the tone target when appropriate.
    pub fn undo_last_key(&mut self) {
        self.undo_last_key_with(ToneRefresh::Recompute);
    }

    /// Undoes the most recent physical key using an explicit tone refresh policy.
    pub fn undo_last_key_with(&mut self, tone_refresh: ToneRefresh) {
        self.undo_last_key_impl(tone_refresh);
    }

    /// Removes the last key using the legacy tone refresh option.
    #[deprecated(
        since = "0.5.0",
        note = "use undo_last_key() or undo_last_key_with(ToneRefresh) instead; planned removal in 1.0.0"
    )]
    #[allow(deprecated)]
    pub fn remove_last_char(&mut self, restore_mark: impl Into<RestoreMark>) {
        let refresh = ToneRefresh::from(restore_mark.into());
        self.undo_last_key_impl(refresh);
    }

    fn undo_last_key_impl(&mut self, tone_refresh: ToneRefresh) {
        if self.pop_snapshot().is_none() {
            return;
        }

        let refresh_last_tone_target = matches!(tone_refresh, ToneRefresh::Recompute);

        if refresh_last_tone_target && self.may_move_tone() {
            crate::syllable::refresh_last_tone_target_into(
                &mut self.active_buffer[..self.active_len],
                self.config.std_tone_style,
            );
        }
        self.update_cached_output();
    }

    /// Removes the last output character (grapheme) from the active composition,
    /// keeping mark/tone transformations on earlier characters intact.
    ///
    /// No-op if the composition is empty. Invalidates the keystroke snapshot stack
    /// and the DFA fast-path state.
    pub fn remove_last_output_char(&mut self) {
        let last = self
            .active_slice()
            .iter()
            .enumerate()
            .rev()
            .find(|(_, t)| t.effect_type == EffectType::Appending && t.key != '\0');
        let Some((l, _)) = last else { return };

        // Compact the buffer, dropping the grapheme at `l` together with every
        // transformation targeting it. Transformations targeting earlier graphemes
        // keep their positions (and targets) — they are what survives the delete.
        let mut write = 0;
        for read in 0..self.active_len {
            let t = self.active_buffer[read];
            if read == l || t.target() == Some(l as u8) {
                continue;
            }
            if write != read {
                self.active_buffer[write] = t;
            }
            write += 1;
        }
        self.active_len = write;

        // Keystroke snapshots are stale after compaction; the DFA fast path no longer matches.
        self.snapshot_len = 0;
        self.current_state_id = 0;
        // Detach only with non-empty residue: state 0 soundly identifies empty.
        self.dfa_detached = self.active_len > 0;

        if self.may_move_tone() {
            crate::syllable::refresh_last_tone_target_into(
                &mut self.active_buffer[..self.active_len],
                self.config.std_tone_style,
            );
        }
        // Deleting back to a valid word ("eete" -> "et") lets the next key
        // add marks again, as if the invalid part was never typed.
        if self.english_bypass && self.is_valid_prefix() {
            self.english_bypass = false;
        }
        // Compaction shifts case bits: recompute directly (cold path).
        self.active_upper_mask = self
            .active_slice()
            .iter()
            .enumerate()
            .fold(0u32, |m, (i, t)| if t.is_upper_case { m | (1u32 << i) } else { m });
        self.update_cached_output();
    }

    /// Resets the engine state, clearing committed and active text.
    pub fn reset(&mut self) {
        self.committed_text.clear();
        self.committed_raw.clear();
        self.cached_output.clear();
        self.active_len = 0;
        self.active_upper_mask = 0;
        self.prev_preedit.clear();
        self.delta_buf.clear();
        self.current_state_id = 0;
        self.snapshot_len = 0;
        self.english_bypass = false;
        self.dfa_detached = false;
    }

    /// Returns the number of DFA states currently cached.
    #[must_use]
    pub const fn dfa_state_count(&self) -> usize {
        self.dfa.states.len()
    }

    /// Returns the number of Transformations stored in the DFA arena.
    #[must_use]
    pub const fn dfa_arena_len(&self) -> usize {
        self.dfa.arena.len()
    }

    /// Returns the number of entries in the DFA composition-to-state map.
    #[must_use]
    pub fn dfa_composition_count(&self) -> usize {
        self.dfa.hash_to_state.len()
    }

    /// Returns the number of bytes in the DFA flattened-output arena.
    #[must_use]
    pub const fn dfa_flat_len(&self) -> usize {
        self.dfa.flat_len()
    }

    /// Returns DFA Vec capacities for memory accounting (states slots).
    #[must_use]
    pub const fn dfa_states_capacity(&self) -> usize {
        self.dfa.states_capacity()
    }

    /// Returns DFA arena capacity in [`Transformation`] slots.
    #[must_use]
    pub const fn dfa_arena_capacity(&self) -> usize {
        self.dfa.arena_capacity()
    }

    /// Returns DFA flat-arena capacity in bytes.
    #[must_use]
    pub const fn dfa_flat_capacity(&self) -> usize {
        self.dfa.flat_capacity()
    }

    /// Returns DFA hash-map capacity (buckets).
    #[must_use]
    pub fn dfa_map_capacity(&self) -> usize {
        self.dfa.map_capacity()
    }

    /// Estimates DFA heap `used` bytes (no slack). See [`crate::dfa::Dfa::memory_used_bytes`].
    #[must_use]
    pub fn dfa_memory_used(&self) -> usize {
        self.dfa.memory_used_bytes()
    }

    /// Estimates DFA heap `allocated` bytes (with Vec slack).
    #[must_use]
    pub fn dfa_memory_allocated(&self) -> usize {
        self.dfa.memory_allocated_bytes()
    }

    /// Returns the capacity (in bytes) of the `committed_text` buffer.
    #[must_use]
    pub const fn committed_text_capacity(&self) -> usize {
        self.committed_text.capacity()
    }

    /// Returns the length (in bytes) of the committed text.
    #[must_use]
    pub const fn committed_text_len(&self) -> usize {
        self.committed_text.len()
    }

    /// Returns true if the lazy snapshot bypass buffer (4 KiB) has been allocated.
    #[must_use]
    pub const fn bypass_allocated(&self) -> bool {
        self.bypass_buffers.is_some()
    }

    /// Returns the number of active transformations in the current syllable.
    #[must_use]
    pub const fn active_len(&self) -> usize {
        self.active_len
    }

    /// Returns the number of snapshots stored for backspace.
    #[must_use]
    pub const fn snapshot_len(&self) -> usize {
        self.snapshot_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luoojcw() {
        let mut e = Engine::new(InputMethod::telex());
        e.process_str("luoojcw", Mode::Vietnamese);
        assert_eq!(e.output(), "lược");
    }

    #[test]
    fn delta_backspaces_and_inserted() {
        let telex = InputMethod::telex();
        let mut e = Engine::new(telex);

        let edit1 = e.process_key_edit('a', Mode::Vietnamese);
        assert_eq!(edit1.delete_chars, 0, "First 'a' should have 0 backspaces");
        assert_eq!(edit1.insert, "a");

        let edit2 = e.process_key_edit('s', Mode::Vietnamese);
        assert_eq!(edit2.delete_chars, 1, "Adding 's' to 'a' should have 1 backspace for 'á'");
        assert_eq!(edit2.insert, "á");

        let edit3 = e.process_key_edit(' ', Mode::Vietnamese);
        assert_eq!(edit3.delete_chars, 1, "Space should clear the preedit 'á'");
        assert_eq!(edit3.insert, "");
    }

    #[test]
    fn remove_last_output_char_telex() {
        let mut e = Engine::new(InputMethod::telex());

        // `tiếng` -> drops `g`, keeps the `s` tone on `ê`.
        e.process_str("tieesng", Mode::Vietnamese);
        assert_eq!(e.output(), "tiếng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");

        // Same shape with plain `e`: `tiéng` -> `tién`.
        e.reset();
        e.process_str("tiengs", Mode::Vietnamese);
        assert_eq!(e.output(), "tiéng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tién");

        // `việt` -> drops `t`, keeps ê mark and nặng tone.
        e.reset();
        e.process_str("vietej", Mode::Vietnamese);
        assert_eq!(e.output(), "việt");
        e.remove_last_output_char();
        assert_eq!(e.output(), "việ");

        // Tone targets the last grapheme: both are dropped.
        e.reset();
        e.process_str("baf", Mode::Vietnamese);
        assert_eq!(e.output(), "bà");
        e.remove_last_output_char();
        assert_eq!(e.output(), "b");

        // Doubled letters: `â` -> `""`.
        e.reset();
        e.process_str("aa", Mode::Vietnamese);
        assert_eq!(e.output(), "â");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");

        // `đ` -> `""`.
        e.reset();
        e.process_str("dd", Mode::Vietnamese);
        assert_eq!(e.output(), "đ");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");

        // Tone typed early: still drops `g`, keeps tone.
        e.reset();
        e.process_str("tieesng", Mode::Vietnamese);
        assert_eq!(e.output(), "tiếng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");

        // Empty composition: no-op.
        e.reset();
        e.remove_last_output_char();
        assert_eq!(e.output(), "");
    }

    #[test]
    fn remove_last_output_char_vni() {
        let mut e = Engine::new(InputMethod::vni());

        // `việt` -> `việ`.
        e.process_str("viet65", Mode::Vietnamese);
        assert_eq!(e.output(), "việt");
        e.remove_last_output_char();
        assert_eq!(e.output(), "việ");

        // `bà` -> `b`.
        e.reset();
        e.process_str("ba2", Mode::Vietnamese);
        assert_eq!(e.output(), "bà");
        e.remove_last_output_char();
        assert_eq!(e.output(), "b");
    }

    #[test]
    fn remove_last_output_char_invalidates_snapshots() {
        let mut e = Engine::new(InputMethod::telex());

        e.process_str("tiếng", Mode::Vietnamese);
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");

        // remove_last_char must not "undo" past the grapheme delete to a stale snapshot.
        e.undo_last_key();
        assert_eq!(e.output(), "tiến");
    }

    #[test]
    fn compose_after_remove_last_output_char() {
        let mut e = Engine::new(InputMethod::telex());

        // `toàn`: the huyền tone targets the vowel `a`, not the final consonant `n`,
        // so deleting `n` keeps the tone: `toàn` -> `tòa`.
        e.process_str("toanf", Mode::Vietnamese);
        assert_eq!(e.output(), "toàn");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tòa");

        // The engine must accept keystrokes normally after compaction:
        // re-typing the deleted consonant recomposes the word since tone was kept.
        e.process_key('n', Mode::Vietnamese);
        assert_eq!(e.output(), "toàn");

        // And keep composing fresh words after a commit.
        e.process_key(' ', Mode::Vietnamese);
        e.process_key('a', Mode::Vietnamese);
        e.process_key('n', Mode::Vietnamese);
        e.process_key('h', Mode::Vietnamese);
        assert_eq!(e.output(), "anh");
    }

    #[test]
    fn remove_last_output_char_double_delete() {
        let mut e = Engine::new(InputMethod::telex());

        // Stress-test compaction on an already-compacted buffer.
        e.process_str("tieesng", Mode::Vietnamese);
        assert_eq!(e.output(), "tiếng");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiến");
        e.remove_last_output_char();
        assert_eq!(e.output(), "tiế");

        // Deleting past the end is a no-op.
        e.reset();
        e.process_str("baf", Mode::Vietnamese);
        assert_eq!(e.output(), "bà");
        e.remove_last_output_char();
        assert_eq!(e.output(), "b");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");
        e.remove_last_output_char();
        assert_eq!(e.output(), "");
    }

    #[test]
    fn test_zero_allocation_output() {
        let mut e = Engine::new(InputMethod::telex());
        assert_eq!(e.output(), "");
        assert!(matches!(e.output(), Cow::Borrowed(_)));
        assert_eq!(e.output_str(), "");

        e.process_str("tieengs", Mode::Vietnamese);
        assert_eq!(e.output(), "ti\u{1ebf}ng");
        assert!(matches!(e.output(), Cow::Borrowed(_)));
        assert_eq!(e.output_str(), "ti\u{1ebf}ng");

        // commit() clears cache
        e.commit();
        assert_eq!(e.output(), "");
        assert!(matches!(e.output(), Cow::Borrowed(_)));
        assert_eq!(e.output_str(), "");
        assert_eq!(e.get_processed_str(OutputOptions::FULL_TEXT), "ti\u{1ebf}ng");
    }

    #[test]
    fn test_engine_from_preset_and_shared_rules() {
        let mut e = Engine::from_preset(InputMethodPreset::Telex);
        e.process_str("tieengs", Mode::Vietnamese);
        assert_eq!(e.output_str(), "tiếng");

        let mut evni = Engine::from_preset(InputMethodPreset::Vni);
        evni.process_str("vie6t5", Mode::Vietnamese);
        assert_eq!(evni.output_str(), "việt");

        // Verify shared rules pointer equivalence for presets
        let e1 = Engine::from_preset(InputMethodPreset::Telex);
        let e2 = Engine::from_preset(InputMethodPreset::Telex);
        assert!(Arc::ptr_eq(&e1.rules, &e2.rules));
        assert!(Arc::ptr_eq(&e1.input_method, &e2.input_method));

        // Verify Engine::new also picks up shared rules automatically
        let e3 = Engine::new(InputMethod::telex());
        assert!(Arc::ptr_eq(&e1.rules, &e3.rules));
    }

    #[test]
    fn word_start_edge_never_wipes_residue() {
        // English-mode typing leaves state 0 + non-empty active text (bypass
        // branch appends raw without DFA lookup). Switching back to Vietnamese
        // mode and typing a key with a pre-cached word-start edge must extend
        // the residue via the slow path, not wipe it via a false fast-path
        // hit from state 0.
        let mut e = Engine::new(InputMethod::telex());
        e.process_str("namf", Mode::Vietnamese); // "nàm", caches state0--'n'
        e.process_key(' ', Mode::Vietnamese);
        e.commit();

        e.process_str("hello", Mode::English);
        assert_eq!(e.output_str(), "hello");
        e.process_key('n', Mode::Vietnamese);
        assert_eq!(e.output_str(), "hellon");
    }

    #[test]
    fn jit_never_overwrites_word_start_edge_from_residue() {
        // Full trace: 'n' from residue must not repoint the word-start edge.
        let mut e = Engine::new(InputMethod::telex());
        e.process_str("namf", Mode::Vietnamese); // caches 0--'n'
        e.process_key(' ', Mode::Vietnamese);
        e.commit();

        e.process_str("hello", Mode::English); // residue, state 0
        e.process_key('n', Mode::Vietnamese); // slow path: "hellon"
        assert_eq!(e.output_str(), "hellon");
        e.process_key(' ', Mode::Vietnamese);
        e.commit();

        // A fresh word starting with 'n' must use the original edge:
        // 'n' alone -> "n", then "namf" -> "nàm" (not "hellon...").
        e.process_key('n', Mode::Vietnamese);
        assert_eq!(e.output_str(), "n");
        e.process_str("amf", Mode::Vietnamese);
        assert_eq!(e.output_str(), "nàm");
    }

    #[test]
    fn uppercase_prefix_still_links_edges() {
        // The DFA is lowercase-canonical: an uppercase active buffer whose
        // canonical form is cached must still qualify as a known prefix and
        // link JIT edges (case-insensitive match), which then serve typing.
        let mut e = Engine::new(InputMethod::telex());
        e.process_str("namf", Mode::Vietnamese);
        e.process_key(' ', Mode::Vietnamese);
        e.commit();

        e.process_str("Nam", Mode::Vietnamese); // fast path, restores with case
        assert_eq!(e.output_str(), "Nam");
        let nam_id = e.current_state_id;
        assert_ne!(nam_id, 0);

        e.process_str("s", Mode::Vietnamese); // slow path from an uppercase prefix
        assert_eq!(e.output_str(), "Nám");
        assert_ne!(
            e.dfa.get_state(nam_id).get_transition(b's'),
            0,
            "uppercase prefix must still link the JIT edge"
        );

        e.process_key(' ', Mode::Vietnamese);
        e.commit();
        // The edge written from the uppercase round must serve later typing.
        e.process_str("nams", Mode::Vietnamese);
        assert_eq!(e.output_str(), "nám");
    }

    #[test]
    fn warm_up_invalidates_stale_snapshots() {
        let mut e = Engine::new(InputMethod::telex());
        e.process_str("tieengs", Mode::Vietnamese);
        assert!(e.snapshot_len() > 0);
        #[allow(deprecated)]
        e.warm_up();
        // Snapshot state IDs refer to the replaced DFA and must not be restored.
        assert_eq!(e.snapshot_len(), 0);
    }

    #[test]
    fn frozen_dfa_stays_correct_via_slow_path() {
        use crate::dfa::DFA_MAX_STATES;

        let mut e = Engine::new(InputMethod::telex());
        // Saturate the DFA with distinct pure-consonant spam (worst case:
        // appending-only, never triggers auto_correct bypass).
        let alph: &[u8] = b"bcdfghjklmnpqrstvwxz";
        let mut i = 0usize;
        while e.dfa_state_count() < DFA_MAX_STATES {
            let mut w = String::with_capacity(10);
            let mut x = i;
            for _ in 0..10 {
                w.push(alph[x % alph.len()] as char);
                x /= alph.len();
            }
            e.process_str(&w, Mode::Vietnamese);
            e.process_key(' ', Mode::Vietnamese);
            e.commit();
            i += 1;
            assert!(i < DFA_MAX_STATES + 1000, "freeze must trigger");
        }
        assert_eq!(e.dfa_state_count(), DFA_MAX_STATES);
        let used = e.dfa_memory_used();
        assert!(used < 4 * 1024 * 1024, "frozen used={} too big", used);

        // After freeze, ordinary Vietnamese must still compose correctly
        // through the rule-engine slow path, and must not grow the DFA.
        let before = e.dfa_state_count();
        for (keys, expect) in
            [("tieengs", "tiếng"), ("vieetj", "việt"), ("namf", "nàm"), ("dduowngf", "đường")]
        {
            e.reset();
            e.process_str(keys, Mode::Vietnamese);
            assert_eq!(e.output_str(), expect, "wrong output for {keys} when frozen");
        }
        assert_eq!(e.dfa_state_count(), before, "frozen DFA must not grow");
    }
}
