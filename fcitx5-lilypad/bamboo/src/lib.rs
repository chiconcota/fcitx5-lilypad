//! # Bamboo Core
//!
//! Bamboo Core is an ultra-high-performance Vietnamese Input Method Engine (IME) written in Rust.
//! It is designed for near-zero allocation on hot paths, deterministic execution, and seamless integration
//! into native applications, text editors, OS-level input handlers, WebAssembly, and C/C++ runtimes.
//!
//! ## Key Capabilities
//!
//! - **Real-time Keystroke Processing**: Sub-microsecond response per key (`< 1 µs / keystroke`),
//!   orders of magnitude faster than conventional engines.
//! - **Multi-Input Method Support**: First-class support for **Telex**, **VNI**, **VIQR**,
//!   and user-defined input methods.
//! - **Orthographic Validation**: Built-in Vietnamese consonant-vowel-consonant (CVC) phonotactic
//!   verification prevents creating invalid Vietnamese words.
//! - **Deterministic Finite Automaton (DFA) Cache**: Pre-compiled common syllable transitions
//!   with branchless popcount-rank lookup over single-cache-line states.
//! - **Rich Deletion Semantics**: Comprehensive support for both character-level and grapheme-level
//!   backspace undoing.
//!
//! ## Quick Start
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! // Create an engine configured with Telex input method
//! let mut engine = Engine::new(InputMethod::telex());
//!
//! // Process keystrokes one by one
//! engine.process_key('t', Mode::Vietnamese);
//! engine.process_key('i', Mode::Vietnamese);
//! engine.process_key('e', Mode::Vietnamese);
//! engine.process_key('e', Mode::Vietnamese);
//! engine.process_key('n', Mode::Vietnamese);
//! engine.process_key('g', Mode::Vietnamese);
//! engine.process_key('s', Mode::Vietnamese);
//!
//! assert_eq!(engine.output(), "tiếng");
//! ```
//!
//! ## API Selection Guide
//!
//! | Use Case | Primary API | Description |
//! |---|---|---|
//! | **IME Frontend (Fcitx5 / `IBus` / Native)** | [`Engine::process_key`], [`Engine::output`] | Feeds keys one-by-one, updates internal buffer, queries current composition word |
//! | **Text Editor / Terminal / IDE** | [`Engine::process_key_edit`] | Returns a named suffix edit to apply to the previous output |
//! | **Keystroke-level Backspace** | [`Engine::undo_last_key`] | $O(1)$ undo of the last physical keypress using stack snapshots |
//! | **Grapheme-level Backspace** | [`Engine::remove_last_output_char`] | Deletes preceding character while retaining marks/tones on remaining vowels (e.g. `tiếng` $\rightarrow$ `tiến`) |
//! | **Word Finalization** | [`Engine::commit`] | Clears composing state and appends text to committed stream |
//! | **Batch / Convenience (Testing)** | [`Engine::process`] | Processes a full string and returns output (**convenience only, not for real-time IME**) |
//! | **Inline Word Repair** | [`Engine::rebuild_from_text`] | Loads existing text (e.g. the word before the cursor) as if typed, so tone/mark/backspace edit it |
//! | **Charset Conversion** | [`encode_charset`], [`decode_charset`] | Converts between Unicode and supported legacy encodings with the typed [`Charset`] enum |
//!
//! ## Migrating from 0.4.x to 0.5
//!
//! Version 0.5 replaces boolean arguments whose meaning depended on their position with named
//! methods and enums. Every 0.4.x API listed below remains available and source-compatible in
//! 0.5.x, but emits a deprecation warning. It is scheduled for removal in 1.0.0.
//!
//! | 0.4.x API | 0.5 replacement |
//! |---|---|
//! | [`Engine::remove_last_char`]`(true)` | [`Engine::undo_last_key`] or [`Engine::undo_last_key_with`]`(`[`ToneRefresh::Recompute`]`)` |
//! | [`Engine::remove_last_char`]`(false)` | [`Engine::undo_last_key_with`]`(`[`ToneRefresh::PreserveSnapshot`]`)` |
//! | [`Engine::restore_last_word`]`(true)` | [`Engine::restore_last_word_as`]`(`[`LastWordForm::Vietnamese`]`)` |
//! | [`Engine::restore_last_word`]`(false)` | [`Engine::restore_last_word_as`]`(`[`LastWordForm::RawKeystrokes`]`)` |
//! | [`Engine::is_valid`]`(true)` | [`Engine::is_valid_word`] |
//! | [`Engine::is_valid`]`(false)` | [`Engine::is_valid_prefix`] |
//! | [`ConfigBuilder::std_tone_style`]`(true)` | [`ConfigBuilder::tone_style`]`(`[`ToneStyle::Modern`]`)` |
//! | [`ConfigBuilder::std_tone_style`]`(false)` | [`ConfigBuilder::tone_style`]`(`[`ToneStyle::Traditional`]`)` |
//! | [`Engine::process_key_delta`] | [`Engine::process_key_edit`], which returns [`EditDelta`] with named fields |
//! | [`Engine::process_key_delta_into`] | [`Engine::process_key_edit_into`] |
//!
//! Configuration switches with a direct enabled/disabled meaning, including
//! [`ConfigBuilder::auto_correct`] and [`ConfigBuilder::free_tone_marking`], are unchanged.
//!
//! ## Legacy Charset Conversion
//!
//! Use [`encode_charset`] and [`decode_charset`] with [`Charset`] for new code. The older
//! string-name API [`advanced::encode`] remains available for compatibility, but typed charsets
//! avoid runtime name lookup. VIQR uses a backslash to escape punctuation that would otherwise
//! extend the encoded letter, preserving ordinary prose through a round trip.
//!
//! ```rust
//! use bamboo_core::{decode_charset, encode_charset, Charset};
//!
//! let encoded = encode_charset(Charset::Viqr, "mưa.");
//! assert_eq!(encoded, "mu+a\\.");
//! assert_eq!(decode_charset(Charset::Viqr, &encoded), "mưa.");
//! ```
//!
//! ## Architecture Overview
//!
//! The engine operates on an internal canvas of [`Transformation`] structs representing individual
//! graphemic steps. These transformations track rule applications, tone marks, and character modifications:
//!
//! ```text
//! Raw Keys:    ['t', 'i', 'e', 'e', 'n', 'g', 's']
//!                   │
//!                   ▼
//! Engine:       DFA Lookup ───► Phonetic Rule Engine ───► CVC Spelling Check
//!                   │
//!                   ▼
//! Composition:  ['t', 'i', 'ê' (+circumflex), 'n', 'g'] + Tone::Sac
//!                   │
//!                   ▼
//! Flattener:    "tiếng"
//! ```
//!
//! ## Deletion Semantics
//!
//! Bamboo Core distinguishes two types of backspace actions:
//!
//! 1. **Keystroke Backspace ([`Engine::undo_last_key`])**:
//!    Undoes the single most recent keystroke transformation, reverting diacritic additions in reverse order.
//!
//!    ```rust
//!    use bamboo_core::{Engine, Mode, InputMethod};
//!
//!    let mut engine = Engine::new(InputMethod::telex());
//!    engine.process_str("tieengs", Mode::Vietnamese);
//!    assert_eq!(engine.output(), "tiếng");
//!
//!    // Drops the tone mark 's', leaving the circumflex on 'ê'
//!    engine.undo_last_key();
//!    assert_eq!(engine.output(), "tiêng");
//!    ```
//!
//! 2. **Grapheme Deletion ([`Engine::remove_last_output_char`])**:
//!    Deletes the whole preceding letter before the caret while preserving diacritics and tone marks
//!    on the earlier characters.
//!
//!    ```rust
//!    use bamboo_core::{Engine, Mode, InputMethod};
//!
//!    let mut engine = Engine::new(InputMethod::telex());
//!    engine.process_str("tieesng", Mode::Vietnamese);
//!    assert_eq!(engine.output(), "tiếng");
//!
//!    // Drops 'g', keeps 'ê' and 's' tone mark -> "tiến"
//!    engine.remove_last_output_char();
//!    assert_eq!(engine.output(), "tiến");
//!    ```
//!
//! ## Tone Marking & Orthography Semantics
//!
//! - **Last Tone Key Wins**: Typing a new tone key replaces any existing tone on that syllable (`looixfsx` $\rightarrow$ `lỗi`).
//! - **Tone Undo**: Typing the exact same tone key consecutively removes the tone and outputs the raw character (`ass` $\rightarrow$ `as`).
//! - **Free Tone Marking**: Tones can be typed at any point during word composition (`hoangf` $\rightarrow$ `hoàng`).
//! - **Spelling Auto-Correction**: When [`Config::auto_correct`](crate::Config::auto_correct) is enabled, invalid syllable compositions
//!   automatically fall back to raw characters.

#![warn(
    missing_docs,
    clippy::undocumented_unsafe_blocks,
    clippy::doc_markdown,
    clippy::manual_let_else,
    clippy::semicolon_if_nothing_returned,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::perf,
    clippy::trivially_copy_pass_by_ref,
    clippy::large_types_passed_by_value,
    clippy::needless_collect,
    clippy::or_fun_call,
    clippy::format_push_string,
    clippy::unnecessary_to_owned,
    clippy::redundant_clone
)]

mod config;
mod dfa;
mod encoder;
mod engine;
mod input_method;
mod mode;
mod orthography;
pub(crate) use dfa::flattener;
pub(crate) use orthography::{phonetics, spelling, syllable};

pub mod ffi;
pub mod wasm;

/// Parallel batch processing utilities for bulk Vietnamese text transformations.
///
/// Available when the `parallel` feature is enabled.
#[cfg(feature = "parallel")]
pub mod parallel {
    use crate::{Engine, InputMethod, Mode};
    use rayon::prelude::*;

    /// Processes multiple input strings in parallel using Rayon work-stealing.
    ///
    /// One engine is reused per worker thread (reset between items), so
    /// per-item construction cost is paid once per thread and later items
    /// reuse the warmed JIT cache. Output order matches input order.
    ///
    /// # Example
    /// ```rust
    /// use bamboo_core::{parallel::process_batch, Mode, InputMethod};
    ///
    /// let inputs = vec!["tieengs", "vieetj", "nam"];
    /// let results = process_batch(&inputs, &InputMethod::telex(), Mode::Vietnamese);
    /// assert_eq!(results, vec!["tiếng", "việt", "nam"]);
    /// ```
    pub fn process_batch<S: AsRef<str> + Sync>(
        inputs: &[S],
        input_method: &InputMethod,
        mode: Mode,
    ) -> Vec<String> {
        inputs
            .par_iter()
            .map_init(
                || Engine::new(input_method.clone()),
                |engine, s| {
                    engine.reset();
                    engine.process(s.as_ref(), mode)
                },
            )
            .collect()
    }
}

pub use config::{BracketMode, Config, ConfigBuilder, ToneStyle, W2uMode};
pub(crate) use encoder::tables as charset_def;
pub use encoder::{Charset, decode_charset, encode_charset};
#[allow(deprecated)]
pub use engine::{
    EditDelta, Engine, EngineRules, LastWordForm, RestoreMark, ToneRefresh, Transformation,
    TransformationStack,
};
pub(crate) use input_method::definitions as input_method_def;
pub use input_method::{InputMethod, InputMethodPreset};
pub use mode::{Mode, OutputOptions};
pub mod fcitx;

/// Advanced types for low-level interaction with the engine.
///
/// This module exposes internal structures and raw definitions
/// for users who need to build custom input methods or analyze the composition state.
pub mod advanced {
    pub use crate::engine::{EngineRules, MAX_ACTIVE_TRANS, Transformation, TransformationStack};
    pub use crate::input_method::{EffectType, InputMethodPreset, Mark, Rule, Tone};
    pub use crate::mode::OutputOptions;

    pub use crate::charset_def::{
        CharsetDefinition, get_charset_definition, get_charset_definitions,
    };
    pub use crate::dfa::{DFA_MAX_STATES, Dfa, State};
    pub use crate::encoder::{
        Charset, charset_names, decode_charset, encode, encode_charset, get_charset_name,
        get_charset_names,
    };
    pub use crate::input_method_def::{
        InputMethodDef, get_input_method, get_input_method_definitions,
    };
}
