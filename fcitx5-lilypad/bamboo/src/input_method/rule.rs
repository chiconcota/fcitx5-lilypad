use phf::{Map, phf_map};

/// Represents a Vietnamese tone mark.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    /// No tone.
    None = 0,
    /// Grave accent.
    Grave = 1,
    /// Acute accent.
    Acute = 2,
    /// Hook above.
    Hook = 3,
    /// Tilde.
    Tilde = 4,
    /// Dot below.
    Dot = 5,
}

/// Represents a Vietnamese diacritic mark (marks that change the vowel/consonant).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mark {
    /// No diacritic.
    None = 0,
    /// Circumflex (Â, Ê, Ô).
    Hat = 1,
    /// Breve (Ă).
    Breve = 2,
    /// Horn (Ư, Ơ).
    Horn = 3,
    /// Dash (Đ).
    Dash = 4,
    /// Special mark for raw character restoration.
    Raw = 5,
}

/// The type of transformation a rule applies.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EffectType {
    /// Appends a character (standard typing).
    #[default]
    Appending = 0,
    /// Adds/changes a diacritic mark.
    MarkTransformation = 1,
    /// Adds/changes a tone mark.
    ToneTransformation = 2,
    /// Replaces a character with another.
    Replacing = 3,
}

pub(crate) static TONES: Map<&'static str, Tone> = phf_map! {
    "XoaDauThanh" => Tone::None,
    "DauSac" => Tone::Acute,
    "DauHuyen" => Tone::Grave,
    "DauNga" => Tone::Tilde,
    "DauNang" => Tone::Dot,
    "DauHoi" => Tone::Hook,
};

/// A transformation rule that defines how a key press affects the composition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rule {
    /// The key that triggers this rule.
    pub key: char,
    /// The character that this rule targets (to be replaced or marked).
    pub effect_on: char,
    /// The resulting character after applying the transformation.
    pub result: char,
    /// Additional characters to append immediately after this one (used for multi-character shortcuts).
    pub appended: [char; 2],
    /// Effect value:
    /// - if `effect_type == ToneTransformation`: this is a [`Tone`] as `u8`
    /// - if `effect_type == MarkTransformation`: this is a [`Mark`] as `u8`
    pub effect: u8,
    /// The type of transformation to apply.
    pub effect_type: EffectType,
    /// Number of characters in `appended`.
    pub appended_len: u8,
}

const _: () = assert!(std::mem::size_of::<Rule>() <= 24);

impl Rule {
    /// Sets the effect value from a [`Tone`].
    pub const fn set_tone(&mut self, tone: Tone) {
        self.effect = tone as u8;
    }

    /// Sets the effect value from a [`Mark`].
    pub const fn set_mark(&mut self, mark: Mark) {
        self.effect = mark as u8;
    }

    /// Retrieves the effect value as a [`Tone`].
    pub const fn get_tone(&self) -> Tone {
        match self.effect {
            1 => Tone::Grave,
            2 => Tone::Acute,
            3 => Tone::Hook,
            4 => Tone::Tilde,
            5 => Tone::Dot,
            _ => Tone::None,
        }
    }

    /// Retrieves the effect value as a [`Mark`].
    pub const fn get_mark(&self) -> Mark {
        match self.effect {
            1 => Mark::Hat,
            2 => Mark::Breve,
            3 => Mark::Horn,
            4 => Mark::Dash,
            5 => Mark::Raw,
            _ => Mark::None,
        }
    }
}
