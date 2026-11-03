//! Compile-time packing of the char tables into `u16`-lane matchers.
//!
//! Everything here derives from [`crate::orthography::tables`] at compile
//! time (`const fn` + `static`); nothing is parsed or allocated at runtime.

use super::tables::{
    CV_ALLOWED_MASKS, FC_0, FC_1, FC_2, FC_3, FC_4, LC_0, LC_1, LC_2, LC_3, LC_4, Token,
    VC_ALLOWED_MASKS, VO_0, VO_1, VO_2, VO_3, VO_4, VO_5, VO_6, VO_7,
};

/// A spelling token with its characters pre-packed into `u16` lanes.
/// `raw[k]` / `stripped[k]` hold lane `k` (0 for unused lanes); `len` is the
/// token length (always 1..=3, verified by `packed_token_tables_wellformed`).
/// All token chars are BMP (max U+1EF9), so `as u16` is exact — also verified
/// there. Lets the matcher test whole lanes with integer compares instead of
/// per-character branches plus `strip_mark` calls.
#[derive(Clone, Copy)]
pub(crate) struct PackedToken {
    pub(crate) raw: [u16; 3],
    pub(crate) stripped: [u16; 3],
    pub(crate) len: u8,
}

impl PackedToken {
    const fn pack(chars: [char; 4], len: u8) -> Self {
        let mut raw = [0u16; 3];
        let mut stripped = [0u16; 3];
        let mut k = 0usize;
        while k < 3 {
            let c = chars[k];
            raw[k] = c as u16;
            stripped[k] = strip_mark(c) as u16;
            k += 1;
        }
        Self { raw, stripped, len }
    }
}

/// Builds a packed row from a char token row at compile time, keeping the
/// char lists the single source of truth.
pub(crate) const fn pack_row<const N: usize>(src: &[Token; N]) -> [PackedToken; N] {
    let mut out = [PackedToken { raw: [0; 3], stripped: [0; 3], len: 0 }; N];
    let mut i = 0usize;
    while i < N {
        let (chars, len) = src[i];
        out[i] = PackedToken::pack(chars, len);
        i += 1;
    }
    out
}

/// Single token-lane test with no branches: raw hit, or stripped hit while
/// the input is still incomplete (mid-typing bases like 'u' match 'ư').
#[inline(always)]
pub(crate) const fn lane_eq(input: u16, raw: u16, stripped: u16, incomplete: bool) -> bool {
    (input == raw) | (incomplete & (input == stripped))
}

/// Strips diacritic marks from a character (mark 0 = canonical base).
/// Used in the innermost token-match loop — one decode-table load.
/// Only the 7 bare marked chars strip; toned variants (e.g. 'ầ') and
/// everything else are identity.
#[inline(always)]
pub(crate) const fn strip_mark(c: char) -> char {
    match crate::phonetics::mark_root(c) {
        Some(root) => root,
        None => c,
    }
}

// Packed parallels, derived from the char rows above at compile time.
pub(crate) static FC_0_PACKED: [PackedToken; 17] = pack_row(&FC_0);
pub(crate) static FC_1_PACKED: [PackedToken; 6] = pack_row(&FC_1);
pub(crate) static FC_2_PACKED: [PackedToken; 6] = pack_row(&FC_2);
pub(crate) static FC_3_PACKED: [PackedToken; 2] = pack_row(&FC_3);
pub(crate) static FC_4_PACKED: [PackedToken; 1] = pack_row(&FC_4);
pub(crate) static VO_0_PACKED: [PackedToken; 6] = pack_row(&VO_0);
pub(crate) static VO_1_PACKED: [PackedToken; 5] = pack_row(&VO_1);
pub(crate) static VO_2_PACKED: [PackedToken; 13] = pack_row(&VO_2);
pub(crate) static VO_3_PACKED: [PackedToken; 1] = pack_row(&VO_3);
pub(crate) static VO_4_PACKED: [PackedToken; 1] = pack_row(&VO_4);
pub(crate) static VO_5_PACKED: [PackedToken; 30] = pack_row(&VO_5);
pub(crate) static VO_6_PACKED: [PackedToken; 2] = pack_row(&VO_6);
pub(crate) static VO_7_PACKED: [PackedToken; 1] = pack_row(&VO_7);
pub(crate) static LC_0_PACKED: [PackedToken; 2] = pack_row(&LC_0);
pub(crate) static LC_1_PACKED: [PackedToken; 2] = pack_row(&LC_1);
pub(crate) static LC_2_PACKED: [PackedToken; 4] = pack_row(&LC_2);
pub(crate) static LC_3_PACKED: [PackedToken; 1] = pack_row(&LC_3);
pub(crate) static LC_4_PACKED: [PackedToken; 1] = pack_row(&LC_4);
pub(crate) static FC_PACKED_ROWS: &[&[PackedToken]] =
    &[&FC_0_PACKED, &FC_1_PACKED, &FC_2_PACKED, &FC_3_PACKED, &FC_4_PACKED];
pub(crate) static VO_PACKED_ROWS: &[&[PackedToken]] = &[
    &VO_0_PACKED,
    &VO_1_PACKED,
    &VO_2_PACKED,
    &VO_3_PACKED,
    &VO_4_PACKED,
    &VO_5_PACKED,
    &VO_6_PACKED,
    &VO_7_PACKED,
];
pub(crate) static LC_PACKED_ROWS: &[&[PackedToken]] =
    &[&LC_0_PACKED, &LC_1_PACKED, &LC_2_PACKED, &LC_3_PACKED, &LC_4_PACKED];

pub(crate) const fn build_vc_ok() -> [u16; 256] {
    let mut t = [0u16; 256];
    let mut m = 0usize;
    while m < 256 {
        let mut acc = 0u16;
        let mut bits = m;
        let mut b = 0usize;
        while bits != 0 {
            if bits & 1 != 0 && b < VC_ALLOWED_MASKS.len() {
                acc |= VC_ALLOWED_MASKS[b];
            }
            bits >>= 1;
            b += 1;
        }
        t[m] = acc;
        m += 1;
    }
    t
}

/// `VC_OK[mask]` = OR of [`VC_ALLOWED_MASKS`] over the mask's set bits.
/// Replaces the trailing-zeros loop in [`is_valid_vc`] with one load.
/// Bits at index 8+ contribute nothing, exactly like the loop's guard.
pub(crate) const VC_OK: [u16; 256] = build_vc_ok();

pub(crate) const fn build_cv_ok() -> [u16; 32] {
    let mut t = [0u16; 32];
    let mut m = 0usize;
    while m < 32 {
        let mut acc = 0u16;
        let mut bits = m;
        let mut b = 0usize;
        while bits != 0 {
            if bits & 1 != 0 && b < CV_ALLOWED_MASKS.len() {
                acc |= CV_ALLOWED_MASKS[b];
            }
            bits >>= 1;
            b += 1;
        }
        t[m] = acc;
        m += 1;
    }
    t
}

/// `CV_OK[mask]` = OR of [`CV_ALLOWED_MASKS`] over the mask's set bits (onset
/// classes are 5 bits; overlapping tokens like `đ` union their allowances).
pub(crate) const CV_OK: [u16; 32] = build_cv_ok();
