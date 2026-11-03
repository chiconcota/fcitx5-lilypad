use crate::input_method::InputMethod;

/// Predefined standard Vietnamese input methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum InputMethodPreset {
    /// Standard Telex (most common input method in Vietnam).
    #[default]
    Telex,
    /// VNI input method (uses numeric keys 1-9 for accents).
    Vni,
    /// VIQR input method (uses punctuation for accents).
    Viqr,
    /// Microsoft standard layout.
    MicrosoftLayout,
    /// Telex 2 (supports `[` and `]` brackets).
    Telex2,
    /// Combined Telex and VNI.
    TelexVni,
    /// Combined Telex, VNI, and VIQR.
    TelexVniViqr,
    /// VNI for French keyboard layouts (AZERTY).
    VniFrenchLayout,
    /// Telex W layout (uses `w` for marks and `z` for tone removal).
    TelexW,
}

impl InputMethodPreset {
    /// Returns the canonical display name of the preset.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Telex => "Telex",
            Self::Vni => "VNI",
            Self::Viqr => "VIQR",
            Self::MicrosoftLayout => "Microsoft layout",
            Self::Telex2 => "Telex 2",
            Self::TelexVni => "Telex + VNI",
            Self::TelexVniViqr => "Telex + VNI + VIQR",
            Self::VniFrenchLayout => "VNI Bàn phím tiếng Pháp",
            Self::TelexW => "Telex W",
        }
    }

    /// Converts this preset into an [`InputMethod`].
    pub fn to_input_method(self) -> InputMethod {
        (*super::get_preset_shared(self).0).clone()
    }
}

impl std::fmt::Display for InputMethodPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for InputMethodPreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "telex" => Ok(Self::Telex),
            "vni" => Ok(Self::Vni),
            "viqr" => Ok(Self::Viqr),
            "microsoft" | "microsoft layout" | "microsoft_layout" => Ok(Self::MicrosoftLayout),
            "telex2" | "telex 2" | "telex_2" => Ok(Self::Telex2),
            "telex+vni" | "telex + vni" | "telex_vni" => Ok(Self::TelexVni),
            "telex+vni+viqr" | "telex + vni + viqr" | "telex_vni_viqr" => Ok(Self::TelexVniViqr),
            "vni french" | "vni_french" | "vni bàn phím tiếng pháp" | "vni ban phim tieng phap" => {
                Ok(Self::VniFrenchLayout)
            }
            "telexw" | "telex w" | "telex_w" => Ok(Self::TelexW),
            _ => Err(format!("Unknown input method preset: '{}'", s)),
        }
    }
}
