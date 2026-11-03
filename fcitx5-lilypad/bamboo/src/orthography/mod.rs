//! Vietnamese orthography, phonetics, and syllable analysis.
//!
//! This domain handles:
//! - Character-level phonetic properties, vowel/consonant tables, and diacritic marks (`phonetics`).
//! - Vietnamese orthographic rules and syllable validity checking (`spelling`).
//! - CVC syllable parsing, word boundary extraction, and transformation targeting (`syllable`).

pub mod packing;
pub mod phonetics;
pub mod spelling;
pub mod syllable;
pub mod tables;
