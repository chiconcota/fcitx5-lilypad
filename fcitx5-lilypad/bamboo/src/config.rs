//! Configuration options and builder for the Bamboo input method engine.
//!
//! Allows fine-tuning tone placement rules, free tone marking flexibility,
//! and orthographic syllable validation.

/// When a typed `w` that marks no vowel turns into `ư`.
///
/// Only a `w` the input method would otherwise insert as a plain letter is affected,
/// so `uw` still gives `ư` and presets that already map `w` to `ư` (Telex W) are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum W2uMode {
    /// `w` stays `w` (`nhw` -> `nhw`).
    #[default]
    Disabled,
    /// `w` becomes `ư` except at the start of a syllable, so English words starting
    /// with `w` survive (`nhw` -> `như`, `wow` -> `wơ`).
    NonStart,
    /// `w` always becomes `ư` (`w` -> `ư`).
    Everywhere,
}

/// When `[`, `]`, `{` and `}` type `ơ`, `ư`, `Ơ` and `Ư`.
///
/// Typing the same bracket again gives the bracket back (`[[` -> `[`). Input methods that
/// already map a bracket (Telex 2) are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BracketMode {
    /// Brackets stay brackets.
    #[default]
    Disabled,
    /// Brackets type vowels except at the start of a word (`t[` -> `tơ`, `[` -> `[`).
    NonStart,
    /// Brackets always type vowels (`[` -> `ơ`).
    Everywhere,
}

/// Placement style for tone marks on ambiguous vowel sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToneStyle {
    /// Modern placement (`hòa`, `khỏe`).
    #[default]
    Modern,
    /// Traditional placement (`hoà`, `khoẻ`).
    Traditional,
}

/// Configuration options for the Bamboo engine.
///
/// Use [`Config::default()`] for the standard modern Vietnamese input setup,
/// or [`Config::builder()`] / [`ConfigBuilder`] to customize individual flags.
///
/// # Example
/// ```rust
/// use bamboo_core::{Config, ToneStyle};
///
/// let config = Config::builder()
///     .free_tone_marking(true)
///     .tone_style(ToneStyle::Modern)
///     .auto_correct(false)
///     .build();
///
/// assert!(config.free_tone_marking);
/// assert!(config.std_tone_style);
/// assert!(!config.auto_correct);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// If `true`, allows typing tone marks at any position in the word (Free Tone Marking).
    /// For example, `hoangf` -> `hoàng`.
    ///
    /// Default: `true`.
    pub free_tone_marking: bool,
    /// If `true`, uses the standard (new) tone placement (e.g., `hòa`, `khỏe`).
    /// If `false`, uses the traditional (old) style (e.g., `hoà`, `khoẻ`).
    ///
    /// Default: `true`.
    pub std_tone_style: bool,
    /// If `true`, enables automatic spelling correction to ensure valid Vietnamese syllables.
    /// Invalid syllables (e.g. non-Vietnamese consonant clusters with marks) will automatically
    /// fall back to raw characters.
    ///
    /// Default: `true`.
    pub auto_correct: bool,
    /// When a plain `w` turns into `ư`; see [`W2uMode`].
    ///
    /// Default: [`W2uMode::Disabled`].
    pub w2u_mode: W2uMode,
    /// When brackets type `ơ` and `ư`; see [`BracketMode`].
    ///
    /// Default: [`BracketMode::Disabled`].
    pub bracket_mode: BracketMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            free_tone_marking: true,
            std_tone_style: true,
            auto_correct: true,
            w2u_mode: W2uMode::Disabled,
            bracket_mode: BracketMode::Disabled,
        }
    }
}

impl Config {
    /// Creates a new configuration with all standard defaults enabled.
    pub const fn new() -> Self {
        Self {
            free_tone_marking: true,
            std_tone_style: true,
            auto_correct: true,
            w2u_mode: W2uMode::Disabled,
            bracket_mode: BracketMode::Disabled,
        }
    }

    /// Returns a [`ConfigBuilder`] for constructing a custom configuration.
    pub const fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Returns the configured tone placement style.
    pub const fn tone_style(self) -> ToneStyle {
        if self.std_tone_style { ToneStyle::Modern } else { ToneStyle::Traditional }
    }

    /// Converts the configuration into an integer bitmask of flags.
    ///
    /// - Bit 0 (0x01): `free_tone_marking`
    /// - Bit 1 (0x02): `std_tone_style`
    /// - Bit 2 (0x04): `auto_correct`
    /// - Bit 3 (0x08): `w2u_mode` is [`W2uMode::NonStart`]
    /// - Bit 4 (0x10): `w2u_mode` is [`W2uMode::Everywhere`] (wins over bit 3)
    /// - Bit 5 (0x20): `bracket_mode` is [`BracketMode::NonStart`]
    /// - Bit 6 (0x40): `bracket_mode` is [`BracketMode::Everywhere`] (wins over bit 5)
    pub const fn to_flags(self) -> u32 {
        let mut flags = 0;
        if self.free_tone_marking {
            flags |= 1 << 0;
        }
        if self.std_tone_style {
            flags |= 1 << 1;
        }
        if self.auto_correct {
            flags |= 1 << 2;
        }
        match self.w2u_mode {
            W2uMode::Disabled => {}
            W2uMode::NonStart => flags |= 1 << 3,
            W2uMode::Everywhere => flags |= 1 << 4,
        }
        match self.bracket_mode {
            BracketMode::Disabled => {}
            BracketMode::NonStart => flags |= 1 << 5,
            BracketMode::Everywhere => flags |= 1 << 6,
        }
        flags
    }

    /// Creates a configuration from an integer bitmask of flags.
    ///
    /// - Bit 0 (0x01): `free_tone_marking`
    /// - Bit 1 (0x02): `std_tone_style`
    /// - Bit 2 (0x04): `auto_correct`
    /// - Bit 3 (0x08): `w2u_mode` is [`W2uMode::NonStart`]
    /// - Bit 4 (0x10): `w2u_mode` is [`W2uMode::Everywhere`] (wins over bit 3)
    /// - Bit 5 (0x20): `bracket_mode` is [`BracketMode::NonStart`]
    /// - Bit 6 (0x40): `bracket_mode` is [`BracketMode::Everywhere`] (wins over bit 5)
    pub const fn from_flags(flags: u32) -> Self {
        Self {
            free_tone_marking: (flags & (1 << 0)) != 0,
            std_tone_style: (flags & (1 << 1)) != 0,
            auto_correct: (flags & (1 << 2)) != 0,
            w2u_mode: if flags & (1 << 4) != 0 {
                W2uMode::Everywhere
            } else if flags & (1 << 3) != 0 {
                W2uMode::NonStart
            } else {
                W2uMode::Disabled
            },
            bracket_mode: if flags & (1 << 6) != 0 {
                BracketMode::Everywhere
            } else if flags & (1 << 5) != 0 {
                BracketMode::NonStart
            } else {
                BracketMode::Disabled
            },
        }
    }
}

/// A fluent builder for constructing a [`Config`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Creates a new builder initialized with default settings.
    pub const fn new() -> Self {
        Self { config: Config::new() }
    }

    /// Sets whether free tone marking is allowed.
    pub const fn free_tone_marking(mut self, enabled: bool) -> Self {
        self.config.free_tone_marking = enabled;
        self
    }

    /// Sets whether standard (new) tone style is enabled (`hòa` vs `hoà`).
    #[deprecated(since = "0.5.0", note = "use tone_style(ToneStyle) instead")]
    pub const fn std_tone_style(mut self, enabled: bool) -> Self {
        self.config.std_tone_style = enabled;
        self
    }

    /// Sets the tone placement style.
    pub const fn tone_style(mut self, style: ToneStyle) -> Self {
        self.config.std_tone_style = matches!(style, ToneStyle::Modern);
        self
    }

    /// Sets whether automatic spelling correction is enabled.
    pub const fn auto_correct(mut self, enabled: bool) -> Self {
        self.config.auto_correct = enabled;
        self
    }

    /// Sets when a plain `w` turns into `ư`.
    pub const fn w2u_mode(mut self, mode: W2uMode) -> Self {
        self.config.w2u_mode = mode;
        self
    }

    /// Sets when brackets type `ơ` and `ư`.
    pub const fn bracket_mode(mut self, mode: BracketMode) -> Self {
        self.config.bracket_mode = mode;
        self
    }

    /// Builds and returns the final [`Config`].
    pub const fn build(self) -> Config {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_flags_from_flags_roundtrip() {
        let configs = [
            (true, true, true, W2uMode::Disabled, BracketMode::Disabled),
            (false, false, false, W2uMode::NonStart, BracketMode::NonStart),
            (true, false, true, W2uMode::Everywhere, BracketMode::Everywhere),
            (false, true, false, W2uMode::Disabled, BracketMode::NonStart),
            (true, false, false, W2uMode::NonStart, BracketMode::Disabled),
        ];
        for (free_tone_marking, std_tone_style, auto_correct, w2u_mode, bracket_mode) in configs {
            let original =
                Config { free_tone_marking, std_tone_style, auto_correct, w2u_mode, bracket_mode };
            let flags = original.to_flags();
            let restored = Config::from_flags(flags);
            assert_eq!(original, restored, "Round-trip failed for {original:?}");
        }
    }

    #[test]
    fn default_config_flags() {
        let cfg = Config::default();
        // Default: all three enabled -> flags = 0b111 = 7
        assert_eq!(cfg.to_flags(), 7);
    }

    #[test]
    fn config_builder() {
        let cfg = Config::builder()
            .free_tone_marking(false)
            .tone_style(ToneStyle::Modern)
            .auto_correct(false)
            .build();

        assert_eq!(
            cfg,
            Config {
                free_tone_marking: false,
                std_tone_style: true,
                auto_correct: false,
                w2u_mode: W2uMode::Disabled,
                bracket_mode: BracketMode::Disabled,
            }
        );
    }

    #[test]
    fn w2u_flag_bits() {
        assert_eq!(Config::builder().w2u_mode(W2uMode::NonStart).build().to_flags(), 0x0f);
        assert_eq!(Config::builder().w2u_mode(W2uMode::Everywhere).build().to_flags(), 0x17);
        assert_eq!(Config::from_flags(0x18).w2u_mode, W2uMode::Everywhere);
        // Flags written before the w2u bits existed keep w2u off.
        assert_eq!(Config::from_flags(7).w2u_mode, W2uMode::Disabled);
    }
}
