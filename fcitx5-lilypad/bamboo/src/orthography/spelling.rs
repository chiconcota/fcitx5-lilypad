//! Vietnamese spelling validation logic based on CVC (Consonant-Vowel-Consonant) structure.

#[cfg(test)]
use super::packing::strip_mark;
use super::packing::{
    CV_OK, FC_PACKED_ROWS, LC_PACKED_ROWS, PackedToken, VC_OK, VO_PACKED_ROWS, lane_eq,
};
#[cfg(test)]
use super::tables::{FC_ROWS, LC_ROWS, Token, VC_ALLOWED_MASKS, VO_ROWS};

fn lookup_mask_optimized(
    rows: &[&[PackedToken]],
    input: &[char],
    input_is_full: bool,
    input_is_complete: bool,
) -> u16 {
    let input_len = input.len();
    // No token exceeds 3 chars, so longer inputs match nothing. Astral input
    // chars exceed the u16 lanes yet can never equal a BMP token lane, so an
    // input containing one matches nothing either (both verified by parity).
    if input_len == 0 || input_len > 3 || input.iter().any(|&c| (c as u32) > 0xFFFF) {
        return 0;
    }
    let incomplete = !input_is_complete;
    let mut packed = [0u16; 3];
    for (k, &c) in input.iter().enumerate() {
        packed[k] = c as u16;
    }
    let need = input_len as u8;
    let mut ret = 0u16;
    for (index, tokens) in rows.iter().enumerate() {
        for t in *tokens {
            if t.len < need || (input_is_full && t.len != need) {
                continue;
            }
            // Branchless lanes; a single branch per token keeps the early
            // exit that pays off on the dominant valid-syllable path.
            let mut hit = true;
            for (k, &in_lane) in packed[..input_len].iter().enumerate() {
                hit &= lane_eq(in_lane, t.raw[k], t.stripped[k], incomplete);
            }
            if hit {
                ret |= 1u16 << index;
                break;
            }
        }
    }
    ret
}

const fn is_valid_vc(vo_mask: u16, lc_mask: u16) -> bool {
    (VC_OK[(vo_mask & 0xFF) as usize] & lc_mask) != 0
}

const fn is_valid_cv(fc_mask: u16, vo_mask: u16) -> bool {
    (CV_OK[(fc_mask & 0x1F) as usize] & vo_mask) != 0
}

pub fn is_valid_cvc(fc: &str, vo: &str, lc: &str, full: bool) -> bool {
    let mut fc_chars = ['\0'; 8];
    let mut vo_chars = ['\0'; 8];
    let mut lc_chars = ['\0'; 8];

    let mut fc_len = 0;
    for c in fc.chars().take(8) {
        fc_chars[fc_len] = c;
        fc_len += 1;
    }
    let mut vo_len = 0;
    for c in vo.chars().take(8) {
        vo_chars[vo_len] = c;
        vo_len += 1;
    }
    let mut lc_len = 0;
    for c in lc.chars().take(8) {
        lc_chars[lc_len] = c;
        lc_len += 1;
    }

    is_valid_cvc_chars(&fc_chars[..fc_len], &vo_chars[..vo_len], &lc_chars[..lc_len], full)
}

/// Low-level variant of [`is_valid_cvc`] that works directly with character slices.
pub fn is_valid_cvc_chars(fc: &[char], vo: &[char], lc: &[char], full: bool) -> bool {
    let fc_mask = if !fc.is_empty() {
        let m = lookup_mask_optimized(FC_PACKED_ROWS, fc, full || !vo.is_empty(), true);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let vo_mask = if !vo.is_empty() {
        let m = lookup_mask_optimized(VO_PACKED_ROWS, vo, full || !lc.is_empty(), full);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let lc_mask = if !lc.is_empty() {
        let m = lookup_mask_optimized(LC_PACKED_ROWS, lc, full, true);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    if vo_mask == 0 {
        if fc_mask != 0 {
            return true;
        }
        // Consonant abbreviations containing 'đ' (e.g., "đc", "vcđ", "cđ", "bcđ", "nđm", "ađ")
        if fc.iter().any(|&c| matches!(c, 'đ' | 'Đ')) || lc.iter().any(|&c| matches!(c, 'đ' | 'Đ'))
        {
            return true;
        }
        return false;
    }

    if fc_mask != 0 && !is_valid_cv(fc_mask, vo_mask) {
        return false;
    }

    if lc_mask != 0 {
        return is_valid_vc(vo_mask, lc_mask);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_simple_syllables() {
        // "ba", "me", "di", "co", "tu"
        assert!(is_valid_cvc("b", "a", "", false));
        assert!(is_valid_cvc("m", "e", "", false));
        assert!(is_valid_cvc("d", "i", "", false));
        assert!(is_valid_cvc("c", "o", "", false));
        assert!(is_valid_cvc("t", "u", "", false));
    }

    #[test]
    fn valid_cvc_with_final_consonant() {
        // "ban", "con", "mat", "toc"
        assert!(is_valid_cvc("b", "a", "n", true));
        assert!(is_valid_cvc("c", "o", "n", true));
        assert!(is_valid_cvc("m", "a", "t", true));
        assert!(is_valid_cvc("t", "o", "c", true));
    }

    #[test]
    fn invalid_consonant_vowel_combinations() {
        // Unknown initial consonant cluster
        assert!(!is_valid_cvc("bb", "a", "", true));
        // Unknown initial consonant cluster
        assert!(!is_valid_cvc("px", "a", "", true));
        // Vowel that doesn't exist in Vietnamese
        assert!(!is_valid_cvc("b", "z", "", true));
    }

    #[test]
    fn valid_no_initial_consonant() {
        // "an", "em", "oi"
        assert!(is_valid_cvc("", "a", "n", true));
        assert!(is_valid_cvc("", "e", "m", true));
        assert!(is_valid_cvc("", "oi", "", false));
    }

    /// Frozen copies of the pre-packed implementations, kept as the oracle
    /// for the packed-token rewrite. Do NOT rewire these to tables/packing.
    mod legacy {
        use super::{Token, VC_ALLOWED_MASKS};

        pub fn strip_mark(c: char) -> char {
            match c {
                'ă' | 'â' => 'a',
                'ê' => 'e',
                'ô' | 'ơ' => 'o',
                'ư' => 'u',
                'đ' => 'd',
                _ => c,
            }
        }

        pub fn lookup_mask(
            rows: &[&[Token]],
            input: &[char],
            input_is_full: bool,
            input_is_complete: bool,
        ) -> u16 {
            let input_len = input.len() as u8;
            if input_len == 0 {
                return 0;
            }
            let mut ret = 0u16;
            for (index, tokens) in rows.iter().enumerate() {
                for (t_chars, t_len) in *tokens {
                    if *t_len < input_len || (input_is_full && *t_len > input_len) {
                        continue;
                    }
                    let hit = input
                        .iter()
                        .zip(t_chars.iter())
                        .all(|(&ic, &tc)| ic == tc || (!input_is_complete && strip_mark(tc) == ic));
                    if hit {
                        ret |= 1u16 << index;
                        break;
                    }
                }
            }
            ret
        }

        pub const fn is_valid_vc(vo_mask: u16, lc_mask: u16) -> bool {
            let mut mask = vo_mask;
            let mut ok = false;
            while mask != 0 {
                let idx = mask.trailing_zeros() as usize;
                if idx < VC_ALLOWED_MASKS.len() && (VC_ALLOWED_MASKS[idx] & lc_mask) != 0 {
                    ok = true;
                }
                mask &= mask - 1;
            }
            ok
        }
    }

    /// Frozen copy of the pre-table `strip_mark` match, kept as the oracle
    /// for the table rewrite. Only the 7 bare marked chars strip; toned
    /// variants (e.g. 'ầ') are identity.
    fn legacy_strip_mark(c: char) -> char {
        legacy::strip_mark(c)
    }

    /// Parity: table-backed `strip_mark` must match the legacy match chain,
    /// including the exact-match-only semantics for toned variants.
    #[test]
    fn strip_mark_matches_legacy() {
        use crate::phonetics::VOWELS;
        let mut inputs: Vec<char> = (' '..='~').collect();
        for &c in VOWELS {
            inputs.push(c);
        }
        inputs.extend(['đ', '中', '–']);
        inputs.push('\u{8E2}');
        for c in inputs {
            assert_eq!(strip_mark(c), legacy_strip_mark(c), "strip_mark({c:?})");
        }
    }

    /// Parity: packed lookup must answer exactly like the legacy per-char
    /// matcher on every input shape, including mixed raw/stripped lanes,
    /// astral chars (never match BMP tokens), and over-long inputs.
    ///
    /// Characterization harness for a behavior-preserving rewrite: passes on
    /// the current code (validating the harness) and must stay green after
    /// the packing.
    #[test]
    fn packed_matches_legacy_lookup() {
        const SET: [char; 12] = ['a', 'n', 'g', 'â', 'ê', 'ô', 'ơ', 'ư', 'đ', 'o', 'u', 'e'];
        let packed_groups: &[&[&[PackedToken]]] = &[FC_PACKED_ROWS, VO_PACKED_ROWS, LC_PACKED_ROWS];
        let groups: &[&[&[Token]]] = &[FC_ROWS, VO_ROWS, LC_ROWS];
        // Length 1..=3 exhaustive over SET x {full, complete} x row groups.
        for len in 1..=3usize {
            let mut idx = vec![0usize; len];
            loop {
                let input: Vec<char> = idx.iter().map(|&k| SET[k]).collect();
                for full in [false, true] {
                    for complete in [false, true] {
                        for (packed_rows, rows) in packed_groups.iter().zip(groups.iter()) {
                            assert_eq!(
                                lookup_mask_optimized(packed_rows, &input, full, complete),
                                legacy::lookup_mask(rows, &input, full, complete),
                                "lookup {input:?} full={full} complete={complete}"
                            );
                        }
                    }
                }
                let mut pos = 0;
                loop {
                    idx[pos] += 1;
                    if idx[pos] < SET.len() {
                        break;
                    }
                    idx[pos] = 0;
                    pos += 1;
                    if pos >= len {
                        break;
                    }
                }
                if pos >= len {
                    break;
                }
            }
        }

        // Explicit mixed-lane case: token ('u','â','y') matches ['u','a','y']
        // only via the stripped middle lane when incomplete.
        let mixed = ['u', 'a', 'y'];
        let hit = lookup_mask_optimized(VO_PACKED_ROWS, &mixed, false, false);
        assert_eq!(hit, legacy::lookup_mask(VO_ROWS, &mixed, false, false));
        assert_ne!(hit & (1u16 << 5), 0, "VO_5 mixed row must hit");
        let hit_complete = lookup_mask_optimized(VO_PACKED_ROWS, &mixed, false, true);
        assert_eq!(hit_complete, legacy::lookup_mask(VO_ROWS, &mixed, false, true));
        assert_eq!(hit_complete & (1u16 << 5), 0, "VO_5 must miss when complete");

        // Astral chars never match BMP tokens (both sides must agree on 0
        // for any length; guards future packing truncation).
        for astral in ['\u{10061}', '\u{10437}', '\u{1F600}'] {
            for input in [&[astral][..], &[astral, 'i'][..], &['a', astral][..]] {
                for (packed_rows, rows) in packed_groups.iter().zip(groups.iter()) {
                    assert_eq!(
                        lookup_mask_optimized(packed_rows, input, false, false),
                        0,
                        "astral {input:?} must match nothing"
                    );
                    assert_eq!(
                        legacy::lookup_mask(rows, input, false, false),
                        0,
                        "legacy astral {input:?} must match nothing"
                    );
                }
            }
        }

        // Over-long inputs match nothing (all tokens are shorter).
        for input in [&['a', 'b', 'c', 'd'][..], &['ư', 'ơ', 'i', 'a', 'b'][..]] {
            for (packed_rows, rows) in packed_groups.iter().zip(groups.iter()) {
                assert_eq!(lookup_mask_optimized(packed_rows, input, false, false), 0);
                assert_eq!(legacy::lookup_mask(rows, input, false, false), 0);
            }
        }

        // VC table parity over the full u8 vo-mask space x lc samples.
        for vo in 0..=255u16 {
            for lc in [0u16, 1, 3, 7, 15, 31, 63, 255, 0xFFFF] {
                assert_eq!(is_valid_vc(vo, lc), legacy::is_valid_vc(vo, lc), "vc vo={vo} lc={lc}");
            }
        }
    }

    /// Guards the packing preconditions for future token edits: every token
    /// is 1..=3 chars, in-lane chars are non-NUL BMP, padding lanes are NUL.
    #[test]
    fn packed_token_tables_wellformed() {
        let groups: &[&[&[Token]]] = &[FC_ROWS, VO_ROWS, LC_ROWS];
        for rows in groups {
            for tokens in *rows {
                for (chars, len) in *tokens {
                    assert!((1..=3u8).contains(len), "token len {len} out of range");
                    for (k, &c) in chars.iter().enumerate() {
                        if (k as u8) < *len {
                            assert_ne!(c, '\0', "in-lane NUL");
                            assert!((c as u32) <= 0xFFFF, "non-BMP token char {c:?}");
                        } else {
                            assert_eq!(c, '\0', "padding lane must be NUL");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn valid_chars_variant_matches_str_variant() {
        let cases =
            [("b", "a", "n"), ("t", "oa", ""), ("", "oi", ""), ("kh", "u", ""), ("", "uye", "")];
        for (fc, vo, lc) in cases {
            let str_result = is_valid_cvc(fc, vo, lc, false);
            let fc_chars: Vec<char> = fc.chars().collect();
            let vo_chars: Vec<char> = vo.chars().collect();
            let lc_chars: Vec<char> = lc.chars().collect();
            let chars_result = is_valid_cvc_chars(&fc_chars, &vo_chars, &lc_chars, false);
            assert_eq!(str_result, chars_result, "Mismatch for ({fc:?}, {vo:?}, {lc:?})");
        }
    }
}
