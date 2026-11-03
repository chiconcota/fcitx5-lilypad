//! Compile-time Vietnamese phonology tables — the single source of truth for
//! spelling data.
//!
//! How to extend (keep engine logic out of here):
//! - New rime/onset/coda token → add a line to the matching row ([`FC_0`]..
//!   [`FC_4`], [`VO_0`]..[`VO_7`], [`LC_0`]..[`LC_4`]) **and bump the row
//!   length in the `*_PACKED` parallel** in [`crate::orthography::packing`]
//!   (two edits, but the compiler enforces both). A 6th onset class would
//!   also widen `CV_OK` (`[u16; 32]` → 64) and its `0x1F` mask there and in
//!   `spelling.rs`.
//! - New allowed/forbidden combination → edit [`VC_ALLOWED_MASKS`] /
//!   [`CV_ALLOWED_MASKS`] (one row per class, bit = allowed partner row).
//! - New tone-placement preference → edit [`TONE_PREFERRED`] /
//!   [`TONE_ON_SECOND`] / [`TONE_SECOND_ONSETS`].
//! - New intermediate rime pattern → edit [`UHO_PATTERNS`].
//! - A behavior that is an algorithm (e.g. "spread the mark only when the
//!   syllable is incomplete without it") stays code in `syllable.rs`.

// Static token representation: (chars, length)
pub(crate) type Token = ([char; 4], u8);

// Onset classes, mirroring Go bamboo-core's `firstConsonantSeqs` rows. Tokens
// may appear in several classes (e.g. `đ` in rows 0 and 3): the lookup sets one
// bit per matching row and `CV_ALLOWED_MASKS` unions their rime allowances.
// `kr` is a bamboo extension (loanword onsets) and rides with the generic class.
pub(crate) static FC_0: [Token; 17] = [
    (['b', '\0', '\0', '\0'], 1),
    (['d', '\0', '\0', '\0'], 1),
    (['đ', '\0', '\0', '\0'], 1),
    (['g', '\0', '\0', '\0'], 1),
    (['g', 'h', '\0', '\0'], 2),
    (['k', 'r', '\0', '\0'], 2),
    (['m', '\0', '\0', '\0'], 1),
    (['n', '\0', '\0', '\0'], 1),
    (['n', 'h', '\0', '\0'], 2),
    (['p', '\0', '\0', '\0'], 1),
    (['p', 'h', '\0', '\0'], 2),
    (['r', '\0', '\0', '\0'], 1),
    (['s', '\0', '\0', '\0'], 1),
    (['t', '\0', '\0', '\0'], 1),
    (['t', 'r', '\0', '\0'], 2),
    (['v', '\0', '\0', '\0'], 1),
    (['z', '\0', '\0', '\0'], 1),
];
pub(crate) static FC_1: [Token; 6] = [
    (['c', '\0', '\0', '\0'], 1),
    (['h', '\0', '\0', '\0'], 1),
    (['k', '\0', '\0', '\0'], 1),
    (['k', 'h', '\0', '\0'], 2),
    (['q', 'u', '\0', '\0'], 2),
    (['t', 'h', '\0', '\0'], 2),
];
pub(crate) static FC_2: [Token; 6] = [
    (['c', 'h', '\0', '\0'], 2),
    (['g', 'i', '\0', '\0'], 2),
    (['l', '\0', '\0', '\0'], 1),
    (['n', 'g', '\0', '\0'], 2),
    (['n', 'g', 'h', '\0'], 3),
    (['x', '\0', '\0', '\0'], 1),
];
pub(crate) static FC_3: [Token; 2] = [(['đ', '\0', '\0', '\0'], 1), (['l', '\0', '\0', '\0'], 1)];
pub(crate) static FC_4: [Token; 1] = [(['h', '\0', '\0', '\0'], 1)];

/// Char-row aggregates, kept as the readable source of truth and the legacy
/// oracle input. Production matches on the packed parallels below.
#[cfg(test)]
pub(crate) static FC_ROWS: &[&[Token]] = &[&FC_0, &FC_1, &FC_2, &FC_3, &FC_4];

pub(crate) static VO_0: [Token; 6] = [
    (['ê', '\0', '\0', '\0'], 1),
    (['i', '\0', '\0', '\0'], 1),
    (['u', 'a', '\0', '\0'], 2),
    (['u', 'ê', '\0', '\0'], 2),
    (['u', 'y', '\0', '\0'], 2),
    (['y', '\0', '\0', '\0'], 1),
];
pub(crate) static VO_1: [Token; 5] = [
    (['a', '\0', '\0', '\0'], 1),
    (['i', 'ê', '\0', '\0'], 2),
    (['o', 'a', '\0', '\0'], 2),
    (['u', 'y', 'ê', '\0'], 3),
    (['y', 'ê', '\0', '\0'], 2),
];
pub(crate) static VO_2: [Token; 13] = [
    (['â', '\0', '\0', '\0'], 1),
    (['ă', '\0', '\0', '\0'], 1),
    (['e', '\0', '\0', '\0'], 1),
    (['o', '\0', '\0', '\0'], 1),
    (['o', 'o', '\0', '\0'], 2),
    (['ô', '\0', '\0', '\0'], 1),
    (['ơ', '\0', '\0', '\0'], 1),
    (['o', 'e', '\0', '\0'], 2),
    (['u', '\0', '\0', '\0'], 1),
    (['ư', '\0', '\0', '\0'], 1),
    (['u', 'â', '\0', '\0'], 2),
    (['u', 'ô', '\0', '\0'], 2),
    (['ư', 'ơ', '\0', '\0'], 2),
];
pub(crate) static VO_3: [Token; 1] = [(['o', 'ă', '\0', '\0'], 2)];
pub(crate) static VO_4: [Token; 1] = [(['u', 'ơ', '\0', '\0'], 2)];
pub(crate) static VO_5: [Token; 30] = [
    (['a', 'i', '\0', '\0'], 2),
    (['a', 'o', '\0', '\0'], 2),
    (['a', 'u', '\0', '\0'], 2),
    (['â', 'u', '\0', '\0'], 2),
    (['a', 'y', '\0', '\0'], 2),
    (['â', 'y', '\0', '\0'], 2),
    (['e', 'o', '\0', '\0'], 2),
    (['ê', 'u', '\0', '\0'], 2),
    (['i', 'a', '\0', '\0'], 2),
    (['i', 'ê', 'u', '\0'], 3),
    (['i', 'u', '\0', '\0'], 2),
    (['o', 'a', 'i', '\0'], 3),
    (['o', 'a', 'o', '\0'], 3),
    (['o', 'a', 'y', '\0'], 3),
    (['o', 'e', 'o', '\0'], 3),
    (['o', 'i', '\0', '\0'], 2),
    (['ô', 'i', '\0', '\0'], 2),
    (['ơ', 'i', '\0', '\0'], 2),
    (['ư', 'a', '\0', '\0'], 2),
    (['u', 'â', 'y', '\0'], 3),
    (['u', 'i', '\0', '\0'], 2),
    (['ư', 'i', '\0', '\0'], 2),
    (['u', 'ô', 'i', '\0'], 3),
    (['ư', 'ơ', 'i', '\0'], 3),
    (['ư', 'ơ', 'u', '\0'], 3),
    (['ư', 'u', '\0', '\0'], 2),
    (['u', 'y', 'a', '\0'], 3),
    (['u', 'y', 'u', '\0'], 3),
    (['u', 'ê', 'u', '\0'], 3),
    (['y', 'ê', 'u', '\0'], 3),
];
pub(crate) static VO_6: [Token; 2] = [(['ă', '\0', '\0', '\0'], 1), (['u', '\0', '\0', '\0'], 1)];
pub(crate) static VO_7: [Token; 1] = [(['i', '\0', '\0', '\0'], 1)];

#[cfg(test)]
pub(crate) static VO_ROWS: &[&[Token]] = &[&VO_0, &VO_1, &VO_2, &VO_3, &VO_4, &VO_5, &VO_6, &VO_7];

pub(crate) static LC_0: [Token; 2] = [(['c', 'h', '\0', '\0'], 2), (['n', 'h', '\0', '\0'], 2)];
pub(crate) static LC_1: [Token; 2] = [(['c', '\0', '\0', '\0'], 1), (['n', 'g', '\0', '\0'], 2)];
pub(crate) static LC_2: [Token; 4] = [
    (['m', '\0', '\0', '\0'], 1),
    (['n', '\0', '\0', '\0'], 1),
    (['p', '\0', '\0', '\0'], 1),
    (['t', '\0', '\0', '\0'], 1),
];
pub(crate) static LC_3: [Token; 1] = [(['k', '\0', '\0', '\0'], 1)];
pub(crate) static LC_4: [Token; 1] = [(['c', '\0', '\0', '\0'], 1)];

#[cfg(test)]
pub(crate) static LC_ROWS: &[&[Token]] = &[&LC_0, &LC_1, &LC_2, &LC_3, &LC_4];
/// Which codas each vowel row may take — Go bamboo-core's `vcMatrix`,
/// row-for-row with [`VO_0`]..[`VO_7`], with one known divergence: row 4
/// (`uơ`) here accepts c/ng/m/n/p/t while Go allows none (`vcMatrix[4] = {}`).
/// Deliberate to leave as-is: `uho_tail_match_composition` rewrites `uơ`→`ươ`
/// as soon as a letter follows (e.g. `thuowc` → `thươc`), so the divergence is
/// unreachable from typing; revisit only if a direct-`uơ`+coda path appears.
pub(crate) const VC_ALLOWED_MASKS: [u16; 8] = [
    (1 << 0) | (1 << 2),
    (1 << 0) | (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2) | (1 << 4),
    0,
    1 << 3,
    1 << 4,
];

/// Which vowel rows each onset class may pair with — Go bamboo-core's
/// `cvMatrix`, row-for-row with the [`FC_0`]..[`FC_4`] classes, with one
/// deliberate extension: class 0 also takes row 3 (`oă`) so real syllables
/// like `boặm`, `toắt`, `nhoằng` stay valid (commit `1dffe18` — pairing an
/// onset with a vowel is free in Vietnamese).
///
/// The operative constraint is row 4 (`uơ`): legal only after class 1
/// (`c h k kh qu th`). `uơ` is the Telex intermediate of the `uow` shortcut,
/// not a standalone rime — so `duơ` is incomplete (its second horn spreads to
/// `dươ`) while `khuơ` stands and bare `uơ` (`uowr` → `uở`) keeps one horn.
/// Partial inputs stay valid through the union check (e.g. bare `u` matches
/// rows 2/4/6 and validates via row 2).
pub(crate) const CV_ALLOWED_MASKS: [u16; 5] = [
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 5),
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5),
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 5),
    1 << 6,
    1 << 7,
];

/// Vowels that attract the tone when a vowel pair has no other cue
/// (`find_tone_target`, std style).
pub(crate) const TONE_PREFERRED: [char; 5] = ['ơ', 'ê', 'ô', 'â', 'ă'];

/// Raw vowel pairs whose tone lands on the *second* vowel.
pub(crate) const TONE_ON_SECOND: [[char; 2]; 5] =
    [['o', 'a'], ['o', 'e'], ['u', 'y'], ['u', 'e'], ['u', 'o']];

/// Onset keys after which a horned vowel pair takes its tone on the second
/// vowel even without a coda (`is_th_or_h` in `syllable.rs`): `th`, `h`.
pub(crate) const TONE_SECOND_ONSETS: [&[char]; 2] = [&['t', 'h'], &['h']];

/// Intermediate rime pairs (`uơ` / `ưo`) that resolve to `ươ` once a letter
/// follows (`uho_tail_match_composition`).
pub(crate) const UHO_PATTERNS: [(char, char); 2] = [('u', 'ơ'), ('ư', 'o')];

#[cfg(test)]
mod tests {
    use super::*;

    /// Duplicate tokens within a row would silently shadow later entries
    /// (lookup stops at the first hit per row), and a mask bit past the row
    /// count would grant pairings that no class can express.
    #[test]
    fn tables_wellformed() {
        let groups: &[&[&[Token]]] = &[FC_ROWS, VO_ROWS, LC_ROWS];
        for rows in groups {
            for (i, row) in rows.iter().enumerate() {
                for (j, a) in row.iter().enumerate() {
                    for b in &row[j + 1..] {
                        assert_ne!(a.0, b.0, "duplicate token {:?} in row {i}", a.0);
                    }
                }
            }
        }
        assert_eq!(CV_ALLOWED_MASKS.len(), FC_ROWS.len(), "cvMatrix rows = onset classes");
        assert_eq!(VC_ALLOWED_MASKS.len(), VO_ROWS.len(), "vcMatrix rows = vowel rows");
        let vo_span = (1u16 << VO_ROWS.len()) - 1;
        let lc_span = (1u16 << LC_ROWS.len()) - 1;
        for (i, &m) in CV_ALLOWED_MASKS.iter().enumerate() {
            assert_eq!(m & !vo_span, 0, "cv row {i} bit past vowel rows");
        }
        for (i, &m) in VC_ALLOWED_MASKS.iter().enumerate() {
            assert_eq!(m & !lc_span, 0, "vc row {i} bit past coda rows");
        }
    }
}
