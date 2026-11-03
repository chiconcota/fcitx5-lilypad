//! Utility functions for Vietnamese character manipulation.
//!
//! This module provides low-level tools for identifying vowels, adding tones,
//! and managing diacritic marks using optimized lookup tables.

use std::sync::LazyLock;

/// A list of all Vietnamese vowels with their various tone marks.
pub const VOWELS: &[char] = &[
    'a', 'à', 'á', 'ả', 'ã', 'ạ', 'ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ', 'â', 'ầ', 'ấ', 'ẩ', 'ẫ', 'ậ', 'e',
    'è', 'é', 'ẻ', 'ẽ', 'ẹ', 'ê', 'ề', 'ế', 'ể', 'ễ', 'ệ', 'i', 'ì', 'í', 'ỉ', 'ĩ', 'ị', 'o', 'ò',
    'ó', 'ỏ', 'õ', 'ọ', 'ô', 'ồ', 'ố', 'ổ', 'ỗ', 'ộ', 'ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ', 'u', 'ù', 'ú',
    'ủ', 'ũ', 'ụ', 'ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự', 'y', 'ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ',
];

/// Number of Vietnamese vowels in [`VOWELS`]. Derived dimensions below
/// (`TONE_TABLE`, `MARK_TABLE`) follow this instead of a hardcoded 72 so the
/// tables stay in sync if the inventory ever grows.
const VOWEL_COUNT: usize = VOWELS.len();
/// Invariants the table code relies on: 6-tone groups (`tone = pos % 6`)
/// and `vowel_pos` fitting in `u8`.
const _: () = assert!(VOWEL_COUNT.is_multiple_of(6) && VOWEL_COUNT <= 250);

/// Mark-family rows shared by [`add_mark_to_toneless_char`] and `strip_mark`.
/// Row per family root: a / e / o / u / d. `MARK_ROWS[row][mark]` is the
/// marked char, `'_'` means "no such mark".
const MARK_ROWS: [[char; 5]; 5] = [
    ['a', 'â', 'ă', '_', '_'],
    ['e', 'ê', '_', '_', '_'],
    ['o', 'ô', '_', 'ơ', '_'],
    ['u', '_', '_', 'ư', '_'],
    ['d', '_', '_', '_', 'đ'],
];

/// Family roots parallel to [`MARK_ROWS`].
const MARK_ROOTS: [char; 5] = ['a', 'e', 'o', 'u', 'd'];

/// `(mark, mark_row)` for the 12 mark-bearing chars; `(MAX, MAX)` otherwise.
/// `mark` matches `find_mark_from_char` (position of `c` in its row).
const fn mark_info(c: char) -> (u8, u8) {
    match c {
        'a' => (0, 0),
        'â' => (1, 0),
        'ă' => (2, 0),
        'e' => (0, 1),
        'ê' => (1, 1),
        'o' => (0, 2),
        'ô' => (1, 2),
        'ơ' => (3, 2),
        'u' => (0, 3),
        'ư' => (3, 3),
        'd' => (0, 4),
        'đ' => (4, 4),
        _ => (u8::MAX, u8::MAX),
    }
}

/// Uppercase for the tabled Vietnamese set, const-evaluable: ASCII rule,
/// Latin-1 `c - 0x20` (`'ý'` → `'Ý'`), everything else in our set pairs at
/// `c - 1` (`'ă'` → `'Ă'`, `'ầ'` → `'Ầ'`, `'đ'` → `'Đ'`). Only ever applied
/// to tabled chars; verified against `char::to_uppercase` by
/// `decode_table_matches_legacy`.
const fn vn_upper(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_uppercase()
    } else {
        let u = c as u32;
        let up = if u <= 0xFF { u - 0x20 } else { u - 1 };
        match char::from_u32(up) {
            Some(ch) => ch,
            None => c,
        }
    }
}

/// Decoded per-character properties for one Vietnamese special char.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CharProps {
    /// Exact char claiming this slot (`'\0'` = empty). Guards against
    /// `& 0x7FF` aliases (e.g. U+08E2 vs `'â'` U+00E2).
    pub(crate) orig: char,
    /// Uppercase form (`upper(c)`).
    pub(crate) upper: char,
    /// Index in [`VOWELS`] (`u8::MAX` = not a vowel).
    pub(crate) vowel_pos: u8,
    /// `find_mark_from_char` value (`u8::MAX` = none).
    pub(crate) mark: u8,
    /// Index into [`MARK_ROWS`] (`u8::MAX` = no mark row).
    pub(crate) mark_row: u8,
}

/// Dense decode page table: `PROPS[c as usize & 0x7FF]` → [`CharProps`].
/// One indexed load replaces the PHF hits in `is_vowel`,
/// `find_vowel_position`, `add_tone_to_char` and `add_mark_to_toneless_char`.
/// A guard miss (alias or untabled char) carries the legacy not-found
/// semantics (identity / `None` / `0` / `to_uppercase` fallback), exactly
/// like the old PHF miss.
const fn build_props() -> [CharProps; 2048] {
    let empty =
        CharProps { orig: '\0', upper: '\0', vowel_pos: u8::MAX, mark: u8::MAX, mark_row: u8::MAX };
    let mut table = [empty; 2048];
    let mut i = 0usize;
    while i < VOWELS.len() {
        let c = VOWELS[i];
        let (mark, mark_row) = mark_info(c);
        table[c as usize & 0x7FF] =
            CharProps { orig: c, upper: vn_upper(c), vowel_pos: i as u8, mark, mark_row };
        i += 1;
    }
    // 'd' has a mark row but is not a vowel; 'đ' likewise.
    let (mark, mark_row) = mark_info('d');
    table['d' as usize & 0x7FF] =
        CharProps { orig: 'd', upper: vn_upper('d'), vowel_pos: u8::MAX, mark, mark_row };
    let (mark, mark_row) = mark_info('đ');
    table['đ' as usize & 0x7FF] =
        CharProps { orig: 'đ', upper: vn_upper('đ'), vowel_pos: u8::MAX, mark, mark_row };
    table
}

/// `const` (not `static`) deliberately: `lookup_props`/`mark_root` are
/// `const fn` so `strip_mark` stays `const` for Task 7's const table build.
/// The compiler backs shared uses with one rodata image.
#[allow(clippy::large_const_arrays)]
const PROPS: [CharProps; 2048] = build_props();

/// Page-table lookup with the `orig` guard. `None` = untabled char:
/// callers apply the legacy not-found semantics.
#[inline]
pub(crate) const fn lookup_props(c: char) -> Option<CharProps> {
    let e = PROPS[c as usize & 0x7FF];
    if e.orig == c && e.orig != '\0' { Some(e) } else { None }
}

/// Mark-family root of `c` (`'â'` → `'a'`), `None` for unmarked chars.
/// Backs `strip_mark` in spelling with one table load.
#[inline]
pub(crate) const fn mark_root(c: char) -> Option<char> {
    let e = PROPS[c as usize & 0x7FF];
    if e.orig == c && e.orig != '\0' && e.mark_row != u8::MAX {
        Some(MARK_ROOTS[e.mark_row as usize])
    } else {
        None
    }
}

const fn build_tone_table() -> [[char; 6]; VOWEL_COUNT] {
    let mut t = [['\0'; 6]; VOWEL_COUNT];
    let mut pos = 0usize;
    while pos < VOWEL_COUNT {
        let base = pos - (pos % 6);
        let mut tone = 0usize;
        while tone < 6 {
            t[pos][tone] = VOWELS[base + tone];
            tone += 1;
        }
        pos += 1;
    }
    t
}

/// `TONE_TABLE[vowel_pos][tone]` = same-mark vowel carrying `tone`.
/// Mirrors [`MARK_TABLE`]; replaces the PHF + `% 6` arithmetic in
/// [`add_tone_to_char`].
const TONE_TABLE: [[char; 6]; VOWEL_COUNT] = build_tone_table();

// Flags for ASCII properties
const F_VOWEL: u8 = 1 << 0;
const F_ALPHA: u8 = 1 << 1;
const F_PUNCT: u8 = 1 << 2;

/// A pre-computed lookup table for ASCII characters (0-127).
static ASCII_PROPS: [u8; 128] = {
    let mut props = [0u8; 128];
    let mut i = 0;
    while i < 128 {
        let c = i as u8 as char;
        let is_alpha = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
        if is_alpha {
            props[i] |= F_ALPHA;
        }
        let is_vowel = c == 'a'
            || c == 'e'
            || c == 'i'
            || c == 'o'
            || c == 'u'
            || c == 'y'
            || c == 'A'
            || c == 'E'
            || c == 'I'
            || c == 'O'
            || c == 'U'
            || c == 'Y';
        if is_vowel {
            props[i] |= F_VOWEL;
        }
        let is_punct = (c >= '!' && c <= '/')
            || (c >= ':' && c <= '@')
            || (c >= '[' && c <= '`')
            || (c >= '{' && c <= '~')
            || c == ' ';
        if is_punct {
            props[i] |= F_PUNCT;
        }
        i += 1;
    }
    props
};

/// Converts a character to lowercase, fast-path for ASCII.
#[inline]
pub fn lower(c: char) -> char {
    if c.is_ascii() { c.to_ascii_lowercase() } else { c.to_lowercase().next().unwrap_or(c) }
}

/// Converts a character to uppercase: ASCII fast-path, precomputed table
/// form for Vietnamese specials, `to_uppercase` fallback otherwise.
#[inline]
pub fn upper(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_uppercase();
    }
    match lookup_props(c) {
        Some(p) => p.upper,
        None => c.to_uppercase().next().unwrap_or(c),
    }
}

/// Returns true if the character is uppercase.
#[inline]
pub fn is_upper(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_uppercase() } else { lower(c) != c }
}

/// Returns true if the character is a space.
#[inline]
pub const fn is_space(c: char) -> bool {
    c == ' '
}

/// Returns true if the character is a common punctuation mark.
#[inline]
pub fn is_punctuation(c: char) -> bool {
    if c.is_ascii() {
        return (ASCII_PROPS[c as usize] & F_PUNCT) != 0;
    }
    c.is_ascii_punctuation() // Best approximation
}

/// Returns true if the character should trigger a word break.
#[inline]
pub fn is_word_break_symbol(c: char) -> bool {
    is_punctuation(c) || c.is_ascii_digit()
}

/// Returns true if the character is a Vietnamese vowel (with or without tone/diacritic).
#[inline]
pub fn is_vowel(c: char) -> bool {
    if c.is_ascii() {
        return (ASCII_PROPS[c as usize] & F_VOWEL) != 0;
    }
    matches!(lookup_props(c), Some(p) if p.vowel_pos != u8::MAX)
}

/// Returns true if the character is a basic ASCII alphabetic character.
#[inline]
pub fn is_alpha(c: char) -> bool {
    if c.is_ascii() {
        return (ASCII_PROPS[c as usize] & F_ALPHA) != 0;
    }
    false
}

#[inline]
fn find_vowel_position(c: char) -> Option<usize> {
    lookup_props(c).and_then(
        |p| {
            if p.vowel_pos == u8::MAX { None } else { Some(p.vowel_pos as usize) }
        },
    )
}

#[inline]
pub(crate) fn find_tone_from_char(c: char) -> u8 {
    lookup_props(c).map_or(0, |p| if p.vowel_pos == u8::MAX { 0 } else { p.vowel_pos % 6 })
}

/// Returns the diacritic mark of a toneless character (`Some(0)` for its plain base).
pub(crate) fn find_mark_from_char(c: char) -> Option<u8> {
    lookup_props(c).and_then(|p| if p.mark == u8::MAX { None } else { Some(p.mark) })
}

/// Adds or changes the tone mark of a Vietnamese vowel.
///
/// `tone` should be a value from 0 to 5.
#[inline]
pub fn add_tone_to_char(c: char, tone: u8) -> char {
    // NOTE: `tone == 0` is NOT identity for non-ASCII vowels: it strips the
    // tone (`'ệ'` → `'ê'`), so there is no tone-0 fast path here.
    match lookup_props(c) {
        Some(p) if p.vowel_pos != u8::MAX => {
            TONE_TABLE[p.vowel_pos as usize].get(tone as usize).copied().unwrap_or(c)
        }
        _ => c,
    }
}

/// Adds a diacritic mark to a toneless character.
#[inline]
pub fn add_mark_to_toneless_char(c: char, mark: u8) -> char {
    match lookup_props(c) {
        Some(p) if p.mark_row != u8::MAX => MARK_ROWS[p.mark_row as usize]
            .get(mark as usize)
            .copied()
            .filter(|&ch| ch != '_')
            .unwrap_or(c),
        _ => c,
    }
}

/// Slow reference implementation used only to build [`MARK_TABLE`].
/// Performs the original multi-step PHF lookup chain.
fn add_mark_to_char_slow(c: char, mark: u8) -> char {
    let tone = find_tone_from_char(c);
    let base = add_tone_to_char(c, 0);
    let marked = add_mark_to_toneless_char(base, mark);
    add_tone_to_char(marked, tone)
}

/// Dense precomputed table: `MARK_TABLE[vowel_index][mark]` → resulting char.
/// Indexed by position in [`VOWELS`] and mark (0..4).
/// Eliminates 4 PHF lookups per `add_mark_to_char` call down to 1–2 array loads.
static MARK_TABLE: LazyLock<[[char; 5]; VOWEL_COUNT]> = LazyLock::new(|| {
    let mut table = [['\0'; 5]; VOWEL_COUNT];
    for pos in 0..VOWEL_COUNT {
        for mark in 0..5u8 {
            table[pos][mark as usize] = add_mark_to_char_slow(VOWELS[pos], mark);
        }
    }
    table
});

/// Adds a diacritic mark to a character while preserving its current tone mark.
///
/// `mark = 0` strips the diacritic (maps to the plain base), **not** identity
/// for characters that carry a mark (ă, â, ê, ô, ơ, ư, đ).
///
/// Optimized with a dense lookup table — only 1–2 array loads for vowels
/// (down from 4 PHF lookups in the reference implementation).
#[inline]
pub fn add_mark_to_char(c: char, mark: u8) -> char {
    if let Some(pos) = find_vowel_position(c) {
        return MARK_TABLE[pos].get(mark as usize).copied().unwrap_or(c);
    }
    add_mark_to_toneless_char(c, mark)
}

/// Returns true if the character is a Vietnamese vowel with a tone mark
/// or a diacritic mark.
#[inline]
pub fn is_vietnamese_rune(c: char) -> bool {
    if c.is_ascii() {
        return false;
    }
    find_tone_from_char(c) != 0 || c != add_mark_to_toneless_char(c, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the dense `MARK_TABLE` matches the reference PHF chain for
    /// every vowel (72) × every mark (0..5).
    #[test]
    fn mark_table_matches_slow_reference() {
        for (pos, &c) in VOWELS.iter().enumerate() {
            for mark in 0..5u8 {
                let fast = add_mark_to_char(c, mark);
                let slow = add_mark_to_char_slow(c, mark);
                assert_eq!(
                    fast, slow,
                    "MARK_TABLE mismatch at VOWELS[{pos}]='{c}' mark={mark}: fast={fast:?} slow={slow:?}"
                );
            }
        }
    }

    #[test]
    fn add_mark_to_char_consonant_d() {
        assert_eq!(add_mark_to_char('d', 4), 'đ');
        assert_eq!(add_mark_to_char('d', 0), 'd');
    }

    /// Frozen copies of the pre-table PHF/match implementations, kept as the
    /// oracle for [`decode_table_matches_legacy`]. Do NOT rewire these to the
    /// decode table — they must keep answering with the legacy semantics.
    mod legacy {
        use super::VOWELS;

        const MARKS_MAPS: [(char, [char; 5]); 12] = [
            ('a', ['a', 'â', 'ă', '_', '_']),
            ('â', ['a', 'â', 'ă', '_', '_']),
            ('ă', ['a', 'â', 'ă', '_', '_']),
            ('e', ['e', 'ê', '_', '_', '_']),
            ('ê', ['e', 'ê', '_', '_', '_']),
            ('o', ['o', 'ô', '_', 'ơ', '_']),
            ('ô', ['o', 'ô', '_', 'ơ', '_']),
            ('ơ', ['o', 'ô', '_', 'ơ', '_']),
            ('u', ['u', '_', '_', 'ư', '_']),
            ('ư', ['u', '_', '_', 'ư', '_']),
            ('d', ['d', '_', '_', '_', 'đ']),
            ('đ', ['d', '_', '_', '_', 'đ']),
        ];

        fn marks_of(c: char) -> Option<&'static [char; 5]> {
            MARKS_MAPS.iter().find(|(k, _)| *k == c).map(|(_, v)| v)
        }

        pub fn is_vowel(c: char) -> bool {
            if c.is_ascii() {
                return matches!(
                    c,
                    'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'A' | 'E' | 'I' | 'O' | 'U' | 'Y'
                );
            }
            VOWELS.contains(&c)
        }

        pub fn find_vowel_position(c: char) -> Option<usize> {
            if c.is_ascii() {
                match c {
                    'a' => Some(0),
                    'e' => Some(18),
                    'i' => Some(30),
                    'o' => Some(36),
                    'u' => Some(54),
                    'y' => Some(66),
                    _ => None,
                }
            } else {
                VOWELS.iter().position(|&v| v == c)
            }
        }

        pub fn find_tone_from_char(c: char) -> u8 {
            if c.is_ascii() { 0 } else { find_vowel_position(c).map_or(0, |pos| (pos % 6) as u8) }
        }

        pub fn find_mark_from_char(c: char) -> Option<u8> {
            marks_of(c)?.iter().position(|&m| m == c).map(|pos| pos as u8)
        }

        pub fn add_tone_to_char(c: char, tone: u8) -> char {
            if c.is_ascii() {
                if tone == 0 {
                    return c;
                }
                let pos = match c {
                    'a' => 0,
                    'e' => 18,
                    'i' => 30,
                    'o' => 36,
                    'u' => 54,
                    'y' => 66,
                    _ => return c,
                };
                return VOWELS[pos + tone as usize];
            }
            find_vowel_position(c)
                .and_then(|pos| {
                    let new_pos = pos as isize + (tone as isize - (pos % 6) as isize);
                    VOWELS.get(new_pos as usize).copied()
                })
                .unwrap_or(c)
        }

        pub fn add_mark_to_toneless_char(c: char, mark: u8) -> char {
            if mark == 0 && c.is_ascii() {
                return c;
            }
            marks_of(c)
                .and_then(|arr| arr.get(mark as usize))
                .copied()
                .filter(|&ch| ch != '_')
                .unwrap_or(c)
        }

        pub fn add_mark_to_char(c: char, mark: u8) -> char {
            let tone = find_tone_from_char(c);
            let base = add_tone_to_char(c, 0);
            let marked = add_mark_to_toneless_char(base, mark);
            add_tone_to_char(marked, tone)
        }

        pub fn upper(c: char) -> char {
            if c.is_ascii() { c.to_ascii_uppercase() } else { c.to_uppercase().next().unwrap_or(c) }
        }
    }

    /// Parity: the decode-table implementations must answer exactly like the
    /// legacy PHF/match implementations for every reachable character.
    ///
    /// Characterization harness for a behavior-preserving refactor: it passes
    /// on the legacy code (validating the harness) and must stay green after
    /// the table rewrite.
    #[test]
    fn decode_table_matches_legacy() {
        let mut inputs: Vec<char> =
            (0x20u32..=0x7Eu32).map(|u| char::from_u32(u).unwrap()).collect();
        for &c in VOWELS {
            inputs.push(c);
            for t in 0..6u8 {
                inputs.push(legacy::add_tone_to_char(c, t));
            }
            for m in 0..5u8 {
                inputs.push(legacy::add_mark_to_char(c, m));
            }
        }
        // '中' (CJK), U+08E2 (aliases 'â' U+00E2 under `& 0x7FF` — must miss
        // the table via the `orig` guard), '–' (en dash).
        inputs.extend(['d', 'đ', 'D', '中', '–']);
        inputs.push('\u{8E2}');

        for c in inputs {
            assert_eq!(is_vowel(c), legacy::is_vowel(c), "is_vowel({c:?})");
            assert_eq!(
                find_vowel_position(c),
                legacy::find_vowel_position(c),
                "find_vowel_position({c:?})"
            );
            assert_eq!(
                find_tone_from_char(c),
                legacy::find_tone_from_char(c),
                "find_tone_from_char({c:?})"
            );
            assert_eq!(
                find_mark_from_char(c),
                legacy::find_mark_from_char(c),
                "find_mark_from_char({c:?})"
            );
            assert_eq!(upper(c), legacy::upper(c), "upper({c:?})");
            for t in 0..6u8 {
                assert_eq!(
                    add_tone_to_char(c, t),
                    legacy::add_tone_to_char(c, t),
                    "add_tone_to_char({c:?},{t})"
                );
            }
            for m in 0..5u8 {
                assert_eq!(
                    add_mark_to_toneless_char(c, m),
                    legacy::add_mark_to_toneless_char(c, m),
                    "add_mark_to_toneless_char({c:?},{m})"
                );
                assert_eq!(
                    add_mark_to_char(c, m),
                    legacy::add_mark_to_char(c, m),
                    "add_mark_to_char({c:?},{m})"
                );
            }
        }
    }
}
