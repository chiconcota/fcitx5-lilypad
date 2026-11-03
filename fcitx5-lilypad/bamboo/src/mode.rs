use bitflags::bitflags;

/// Represents the processing mode of the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Process input using Vietnamese transformation rules.
    #[default]
    Vietnamese,
    /// Treat input as plain English (no transformations).
    English,
}

bitflags! {
    /// Customization options for the flattened string output.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct OutputOptions: u32 {
        /// Default output with no extra transformations.
        const NONE = 0;
        /// Strip tone marks from the output.
        const TONE_LESS = 1 << 0;
        /// Strip diacritic marks (marks that change the vowel/consonant) from the output.
        const MARK_LESS = 1 << 1;
        /// Convert output to lowercase.
        const LOWER_CASE = 1 << 2;
        /// Return the full text including committed and active text.
        const FULL_TEXT = 1 << 3;
        /// Handle punctuation marks specifically for IME usage.
        const PUNCTUATION_MODE = 1 << 4;
        /// Reserved for future use.
        const IN_REVERSE_ORDER = 1 << 5;
        /// Return raw input keys instead of transformed text.
        const RAW = 1 << 6;
    }
}
