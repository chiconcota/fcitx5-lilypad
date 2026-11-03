//! Provides functions and tables for encoding Vietnamese Unicode text into legacy character sets.
//!
//! Supported character sets include:
//! - **Unicode** (precomposed NFC)
//! - **Unicode tổ hợp** (decomposed NFD)
//! - **TCVN3 (ABC)**
//! - **VNI Windows**
//! - **Windows 1258 codepage**
//! - **VIQR**
//! - **VISCII**
//! - **VPS**
//! - **BKHCM 1** / **BKHCM 2**
//! - **Vietware X** / **Vietware Full**
//! - **UTF-8**
//! - **NCR Decimal** / **NCR Hex**
//! - **Unicode C string Hex** / **Unicode C string Decimal**

pub mod tables;
use rustc_hash::FxHashSet;
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use rustc_hash::FxHashMap;
use tables::CharsetDefinition;
pub use tables::{get_charset_definition, get_charset_definitions};

static UNICODE: &str = "Unicode";

/// Strongly-typed Vietnamese character sets supported by Bamboo Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Charset {
    /// Canonical Unicode precomposed NFC (default).
    #[default]
    Unicode,
    /// TCVN3 (ABC standard).
    Tcvn3,
    /// VNI Windows legacy encoding.
    VniWindows,
    /// Unicode decomposed NFD (Unicode tổ hợp).
    UnicodeCompound,
    /// Windows 1258 Vietnamese codepage.
    Windows1258,
    /// VIQR (Vietnamese Quoted-Readable).
    Viqr,
    /// VISCII (Vietnamese Standard Code for Information Interchange).
    Viscii,
    /// VPS legacy encoding.
    Vps,
    /// BKHCM 1 legacy encoding.
    Bkhcm1,
    /// BKHCM 2 legacy encoding.
    Bkhcm2,
    /// Vietware X encoding.
    VietwareX,
    /// Vietware Full encoding.
    VietwareFull,
    /// UTF-8 literal byte escape sequences.
    Utf8,
    /// NCR Decimal (`&#...;`).
    NcrDecimal,
    /// NCR Hex (`&#x...;`).
    NcrHex,
    /// Unicode C string Hex (`\x...`).
    UnicodeCStringHex,
    /// Unicode C string Decimal (`\u...`).
    UnicodeCStringDecimal,
}

impl Charset {
    /// Returns the canonical display name of the charset definition.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unicode => "Unicode",
            Self::Tcvn3 => "TCVN3 (ABC)",
            Self::VniWindows => "VNI Windows",
            Self::UnicodeCompound => "Unicode tổ hợp",
            Self::Windows1258 => "Windows 1258 codepage",
            Self::Viqr => "VIQR",
            Self::Viscii => "VISCII",
            Self::Vps => "VPS",
            Self::Bkhcm1 => "BKHCM 1",
            Self::Bkhcm2 => "BKHCM 2",
            Self::VietwareX => "Vietware X",
            Self::VietwareFull => "Vietware Full",
            Self::Utf8 => "UTF-8",
            Self::NcrDecimal => "NCR Decimal",
            Self::NcrHex => "NCR Hex",
            Self::UnicodeCStringHex => "Unicode C string Hex",
            Self::UnicodeCStringDecimal => "Unicode C string Decimal",
        }
    }
}

impl fmt::Display for Charset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Charset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unicode" | "unicode" => Ok(Self::Unicode),
            "TCVN3 (ABC)" | "TCVN3" | "tcvn3" => Ok(Self::Tcvn3),
            "VNI Windows" | "VNI" | "vni" => Ok(Self::VniWindows),
            "Unicode tổ hợp" | "UnicodeToHop" => Ok(Self::UnicodeCompound),
            "Windows 1258 codepage" | "Windows-1258" | "windows-1258" => Ok(Self::Windows1258),
            "VIQR" | "viqr" => Ok(Self::Viqr),
            "VISCII" | "viscii" => Ok(Self::Viscii),
            "VPS" | "vps" => Ok(Self::Vps),
            "BKHCM 1" | "bkhcm1" => Ok(Self::Bkhcm1),
            "BKHCM 2" | "bkhcm2" => Ok(Self::Bkhcm2),
            "Vietware X" | "vietware_x" => Ok(Self::VietwareX),
            "Vietware Full" | "vietware_full" => Ok(Self::VietwareFull),
            "UTF-8" | "utf-8" | "utf8" => Ok(Self::Utf8),
            "NCR Decimal" | "ncr_decimal" => Ok(Self::NcrDecimal),
            "NCR Hex" | "ncr_hex" => Ok(Self::NcrHex),
            "Unicode C string Hex" => Ok(Self::UnicodeCStringHex),
            "Unicode C string Decimal" => Ok(Self::UnicodeCStringDecimal),
            _ => Err(()),
        }
    }
}

/// Encodes a Vietnamese Unicode string into a strongly-typed [`Charset`].
///
/// Returns [`Cow::Borrowed`] with **zero heap allocations** if `charset` is [`Charset::Unicode`],
/// if `input` is empty, or if `input` contains no Vietnamese characters requiring encoding.
///
/// VIQR puts a backslash before a diacritic character that would otherwise join the letter
/// in front of it, and before every backslash, so the text reads back the same:
/// `"mưa."` becomes `"mu+a\."`.
///
/// # Example
/// ```rust
/// use bamboo_core::{encode_charset, Charset};
///
/// let encoded = encode_charset(Charset::Viqr, "tiếng Việt");
/// assert_eq!(encoded, "tie^'ng Vie^.t");
///
/// // Zero-allocation on plain ASCII or standard Unicode
/// let passthrough = encode_charset(Charset::Viqr, "hello world");
/// assert!(matches!(passthrough, std::borrow::Cow::Borrowed(_)));
/// ```
pub fn encode_charset<'a>(charset: Charset, input: &'a str) -> Cow<'a, str> {
    if charset == Charset::Unicode || input.is_empty() {
        return Cow::Borrowed(input);
    }

    match get_charset_definition(charset.as_str()) {
        Some(charset_def) if charset == Charset::Viqr => encode_viqr(charset_def, input),
        Some(charset_def) => {
            if !input.chars().any(|c| !c.is_ascii() && charset_def.contains_key(&c)) {
                return Cow::Borrowed(input);
            }

            let mut output = String::with_capacity(input.len());
            for char in input.chars() {
                if char.is_ascii() {
                    output.push(char);
                } else {
                    match charset_def.get(&char) {
                        Some(encoded) => output.push_str(encoded),
                        None => output.push(char),
                    }
                }
            }
            Cow::Owned(output)
        }
        None => Cow::Borrowed(input),
    }
}

/// Every letter code in the VIQR table, to tell when a following character would join it.
static VIQR_CODES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    get_charset_definition(Charset::Viqr.as_str())
        .map(|def| def.values().copied().collect())
        .unwrap_or_default()
});

fn encode_viqr<'a>(charset_def: &CharsetDefinition, input: &'a str) -> Cow<'a, str> {
    let mut output: Option<String> = None;
    let mut prev_code = "";
    for (i, char) in input.char_indices() {
        let found = (!char.is_ascii()).then(|| charset_def.get(&char).copied()).flatten();
        let code = found.unwrap_or_else(|| &input[i..i + char.len_utf8()]);
        let escape =
            char == '\\' || (char.is_ascii_punctuation() && joins_viqr_code(prev_code, char));
        if found.is_some() || escape {
            let out = output.get_or_insert_with(|| input[..i].to_string());
            if escape {
                out.push('\\');
            }
            out.push_str(code);
        } else if let Some(out) = output.as_mut() {
            out.push(char);
        }
        // An escaped character stands alone, so nothing can join it.
        prev_code = if escape { "" } else { code };
    }
    output.map_or(Cow::Borrowed(input), Cow::Owned)
}

fn joins_viqr_code(prev_code: &str, mark: char) -> bool {
    let mut buf = [0u8; 4];
    let len = prev_code.len();
    if len >= buf.len() {
        return false;
    }
    buf[..len].copy_from_slice(prev_code.as_bytes());
    buf[len] = mark as u8;
    std::str::from_utf8(&buf[..=len]).is_ok_and(|joined| VIQR_CODES.contains(joined))
}

/// Encodes a Vietnamese Unicode string into a specific character set by string name.
///
/// If `charset_name` is `"Unicode"` or the input is empty, returns a clone of `input`.
/// Unrecognized characters or unknown charset names pass through unchanged.
///
/// For zero allocations, prefer [`encode_charset`] with the typed [`Charset`] enum.
///
/// # Arguments
/// * `charset_name` - Name of the target character set (e.g., `"TCVN3 (ABC)"`, `"VNI Windows"`, `"VIQR"`).
/// * `input` - The source Unicode string.
///
/// # Example
/// ```rust
/// use bamboo_core::advanced::encode;
///
/// let viqr = encode("VIQR", "tiếng Việt");
/// assert_eq!(viqr, "tie^'ng Vie^.t");
/// ```
pub fn encode(charset_name: &str, input: &str) -> String {
    if charset_name == UNICODE || input.is_empty() {
        return input.to_string();
    }

    match get_charset_definition(charset_name) {
        Some(charset_def) if charset_name == Charset::Viqr.as_str() => {
            encode_viqr(charset_def, input).into_owned()
        }
        Some(charset_def) => {
            let mut output = String::with_capacity(input.len());
            for char in input.chars() {
                if char.is_ascii() {
                    output.push(char);
                } else {
                    match charset_def.get(&char) {
                        Some(encoded) => output.push_str(encoded),
                        None => output.push(char),
                    }
                }
            }
            output
        }
        None => input.to_string(),
    }
}

struct DecodeTable {
    chars: FxHashMap<&'static str, char>,
    starts: FxHashSet<char>,
    max_len: usize,
}

// Capitals VISCII puts on control codes and 0xA0. The encode table folds them into lowercase so
// typing never sends control characters, but real VISCII files contain them.
const VISCII_CAPITALS: [(&str, char); 7] = [
    ("\u{2}", 'Ẳ'),
    ("\u{5}", 'Ẵ'),
    ("\u{6}", 'Ẫ'),
    ("\u{14}", 'Ỷ'),
    ("\u{19}", 'Ỹ'),
    ("\u{1e}", 'Ỵ'),
    ("\u{a0}", 'Õ'),
];

static DECODE_TABLES: LazyLock<FxHashMap<&'static str, DecodeTable>> = LazyLock::new(|| {
    get_charset_definitions()
        .entries()
        .map(|(&name, def)| {
            let extra: &[(&str, char)] =
                if name == Charset::Viscii.as_str() { &VISCII_CAPITALS } else { &[] };
            (name, build_decode_table(def, extra))
        })
        .collect()
});

fn build_decode_table(
    def: &tables::CharsetDefinition,
    extra: &[(&'static str, char)],
) -> DecodeTable {
    let mut groups: FxHashMap<&'static str, Vec<char>> = FxHashMap::default();
    for (&c, &encoded) in def.entries() {
        groups.entry(encoded).or_default().push(c);
    }
    let mut chars: FxHashMap<&'static str, char> = groups
        .into_iter()
        .filter_map(|(encoded, letters)| match letters[..] {
            [c] => Some((encoded, c)),
            // Shared by case only: the capitals font that told them apart is lost on copy.
            // Shared by two different letters: leave the code as it is rather than guess.
            [a, b] if a.to_lowercase().eq(b.to_lowercase()) => {
                Some((encoded, if a.is_lowercase() { a } else { b }))
            }
            _ => None,
        })
        .collect();
    chars.extend(extra.iter().copied());
    let starts = chars.keys().filter_map(|k| k.chars().next()).collect();
    let max_len = chars.keys().map(|k| k.len()).max().unwrap_or(0);
    DecodeTable { chars, starts, max_len }
}

/// Decodes text in a legacy [`Charset`] back into Unicode; the inverse of [`encode_charset`].
///
/// Sequences are matched longest first, so multi-character encodings such as VNI `eá` or
/// VIQR `e^'` decode to one letter. Anything that is not an encoded sequence passes through.
///
/// Returns [`Cow::Borrowed`] when nothing in `input` needs decoding.
///
/// Some limits come from the charsets themselves:
/// - TCVN3 (ABC) and Vietware Full use the same code for most lowercase and uppercase toned
///   vowels and relied on a separate font for capitals; VPS lacks some capitals. Such shared
///   codes decode to lowercase. VISCII's own codes for `Ẳ Ẵ Ẫ Õ Ỷ Ỹ Ỵ` decode to capitals,
///   although [`encode_charset`] writes the lowercase codes for them. A code shared by two different letters (`Ặ` and
///   `Ụ` in BKHCM 1) is left undecoded.
/// - Charsets that use ASCII for letters also decode ASCII text that happens to match: VIQR
///   (`dd` becomes `đ`, a full stop after a vowel becomes a dot below unless written `\.`,
///   the VIQR escape), NCR, C string, and
///   BKHCM 1, which encodes `Đ Ấ Ý Ỳ Ỷ Ỹ` as `}`, `~`, `{`, `^`, `` ` `` and `|`.
/// - [`Charset::UnicodeCompound`] is the letter with its hat or horn plus a combining tone
///   mark, as [`encode_charset`] writes it, not full NFD.
///
/// # Example
/// ```rust
/// use bamboo_core::{decode_charset, encode_charset, Charset};
///
/// assert_eq!(decode_charset(Charset::VniWindows, "tieáng Vieät"), "tiếng Việt");
///
/// let viqr = encode_charset(Charset::Viqr, "tiếng Việt");
/// assert_eq!(decode_charset(Charset::Viqr, &viqr), "tiếng Việt");
/// ```
pub fn decode_charset<'a>(charset: Charset, input: &'a str) -> Cow<'a, str> {
    let Some(table) = DECODE_TABLES.get(charset.as_str()) else {
        return Cow::Borrowed(input);
    };

    let mut output: Option<String> = None;
    let mut i = 0;
    while i < input.len() {
        let rest = &input[i..];
        let first = rest.chars().next().unwrap_or_default();
        if charset == Charset::Viqr
            && first == '\\'
            && let Some(next) = rest[1..].chars().next().filter(char::is_ascii_punctuation)
        {
            output.get_or_insert_with(|| input[..i].to_string()).push(next);
            i += 1 + next.len_utf8();
            continue;
        }
        let found = if table.starts.contains(&first) {
            (1..=table.max_len.min(rest.len()))
                .rev()
                .filter(|&len| rest.is_char_boundary(len))
                .find_map(|len| table.chars.get(&rest[..len]).map(|&c| (c, len)))
        } else {
            None
        };
        match found {
            Some((c, len)) if output.is_some() || rest[..len].chars().ne(std::iter::once(c)) => {
                output.get_or_insert_with(|| input[..i].to_string()).push(c);
                i += len;
            }
            _ => {
                if let Some(out) = output.as_mut() {
                    out.push(first);
                }
                i += first.len_utf8();
            }
        }
    }
    output.map_or(Cow::Borrowed(input), Cow::Owned)
}

/// Returns an iterator over all supported character set names without heap allocations.
///
/// # Example
/// ```rust
/// use bamboo_core::advanced::charset_names;
///
/// let names: Vec<&'static str> = charset_names().collect();
/// assert!(names.contains(&"Unicode"));
/// assert!(names.contains(&"TCVN3 (ABC)"));
/// assert!(names.contains(&"VIQR"));
/// ```
pub fn charset_names() -> impl Iterator<Item = &'static str> {
    std::iter::once(UNICODE).chain(get_charset_definitions().keys().copied())
}

/// Returns a newly allocated list of all supported character set names.
pub fn get_charset_name() -> Vec<String> {
    charset_names().map(String::from).collect()
}

/// Alias for [`get_charset_name`].
pub fn get_charset_names() -> Vec<String> {
    get_charset_name()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_charset_zero_alloc() {
        // Plain ASCII returns Borrowed (0 heap allocation)
        let res = encode_charset(Charset::Viqr, "hello world");
        assert_eq!(res, "hello world");
        assert!(matches!(res, Cow::Borrowed(_)));

        // Unicode charset returns Borrowed (0 heap allocation)
        let res_uni = encode_charset(Charset::Unicode, "ti\u{1ebf}ng Vi\u{1ec7}t");
        assert_eq!(res_uni, "ti\u{1ebf}ng Vi\u{1ec7}t");
        assert!(matches!(res_uni, Cow::Borrowed(_)));

        // Converted charset returns Owned with correct transformed string
        let res_viqr = encode_charset(Charset::Viqr, "ti\u{1ebf}ng Vi\u{1ec7}t");
        assert_eq!(res_viqr, "tie^'ng Vie^.t");
        assert!(matches!(res_viqr, Cow::Owned(_)));
    }

    #[test]
    fn test_windows_1258_matches_the_codepage() {
        // Bytes DD EC and D0, as glibc iconv writes them, shown as cp1252 like the rest of the table.
        assert_eq!(encode_charset(Charset::Windows1258, "Ứ"), "Ýì");
        assert_eq!(encode_charset(Charset::Windows1258, "Đ"), "\u{D0}");

        // Every code must be bytes of the codepage, i.e. characters cp1252 can show.
        let cp1252_high = "€‚ƒ„…†‡ˆ‰Š‹ŒŽ‘’“”•–—˜™š›œžŸ";
        let def = get_charset_definition(Charset::Windows1258.as_str()).unwrap();
        for (letter, code) in def.entries() {
            for c in code.chars() {
                assert!(c <= '\u{FF}' || cp1252_high.contains(c), "{letter} => {code:?}");
            }
        }
    }

    #[test]
    fn test_viqr_escapes_marks_that_would_join_a_letter() {
        let cases = [
            ("m\u{1b0}a.", "mu+a\\."),
            ("H\u{e0} N\u{1ed9}i?", "Ha` No^.i\\?"),
            ("xa.", "xa\\."),
            ("Y?", "Y\\?"),
            ("\u{1a1}'", "o+\\'"),
            ("e^", "e\\^"),
            ("a(", "a\\("),
            ("o+", "o\\+"),
            ("a\\b", "a\\\\b"),
            ("\u{1b0}.", "u+\\."),
            ("a..", "a\\.."),
            // Nothing to join: a toned letter takes no second tone.
            ("\u{1ea5}.", "a^'."),
            ("\u{1ea1}.", "a.."),
            ("\u{ea}^", "e^^"),
            ("x.", "x."),
            ("a .", "a ."),
            ("a)", "a)"),
        ];
        for (input, expected) in cases {
            assert_eq!(encode_charset(Charset::Viqr, input), expected, "{input:?}");
            assert_eq!(encode("VIQR", input), expected, "{input:?}");
        }
        assert!(matches!(encode_charset(Charset::Viqr, "x. a)"), Cow::Borrowed(_)));
    }
}
