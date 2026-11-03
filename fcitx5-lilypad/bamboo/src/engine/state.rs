#![allow(deprecated)]

use crate::input_method::{EffectType, Rule, Tone};
use std::mem::MaybeUninit;

/// Whether undoing a key recomputes the target of an existing tone mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToneRefresh {
    /// Recompute the tone target when the remaining syllable is valid.
    #[default]
    Recompute,
    /// Preserve the exact tone target stored in the previous snapshot.
    PreserveSnapshot,
}

/// Legacy option for refreshing tone targets when removing characters.
#[deprecated(since = "0.5.0", note = "use ToneRefresh instead; planned removal in 1.0.0")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RestoreMark {
    /// Refresh the last tone target to maintain valid tone placement.
    #[default]
    Yes,
    /// Do not refresh the last tone target.
    No,
}

#[allow(deprecated)]
impl From<RestoreMark> for ToneRefresh {
    fn from(value: RestoreMark) -> Self {
        match value {
            RestoreMark::Yes => Self::Recompute,
            RestoreMark::No => Self::PreserveSnapshot,
        }
    }
}

impl From<bool> for RestoreMark {
    fn from(v: bool) -> Self {
        if v { Self::Yes } else { Self::No }
    }
}

impl From<RestoreMark> for bool {
    fn from(r: RestoreMark) -> Self {
        matches!(r, RestoreMark::Yes)
    }
}

/// Maximum number of active transformations in a single syllable.
pub const MAX_ACTIVE_TRANS: usize = 32;

/// Sentinel value representing `None` for a transformation target index.
pub const TARGET_NONE: u8 = 0xFF;

/// Represents a single keypress or a transformation derived from it (e.g., adding a mark or tone).
///
/// Compacted to exactly 16 bytes (128-bit aligned) for optimal CPU cache utilization and zero heap waste.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Transformation {
    /// The key that triggered this transformation (or `\0` if synthetic/virtual).
    pub key: char,
    /// The character that this transformation targets / replaces.
    pub effect_on: char,
    /// The resulting character after transformation.
    pub result: char,
    /// The index of the targeted transformation in the composition (`0xFF` = None).
    pub(crate) target_idx: u8,
    /// Transformation effect value (Tone or Mark enum value).
    pub effect: u8,
    /// Type of transformation (Appending, Mark, Tone, Replacing).
    pub effect_type: EffectType,
    /// Whether the resulting character should be rendered uppercase.
    pub is_upper_case: bool,
}

const _: () = assert!(std::mem::size_of::<Transformation>() == 16);

impl Default for Transformation {
    #[inline]
    fn default() -> Self {
        Self {
            key: '\0',
            effect_on: '\0',
            result: '\0',
            target_idx: TARGET_NONE,
            effect: 0,
            effect_type: EffectType::Appending,
            is_upper_case: false,
        }
    }
}

impl Transformation {
    /// Creates a new transformation from explicit fields.
    #[inline]
    pub const fn new(
        key: char,
        effect_on: char,
        result: char,
        target: Option<u8>,
        effect: u8,
        effect_type: EffectType,
        is_upper_case: bool,
    ) -> Self {
        let target_idx = match target {
            Some(idx) => idx,
            None => TARGET_NONE,
        };
        Self { key, effect_on, result, target_idx, effect, effect_type, is_upper_case }
    }

    /// Creates a new transformation from a [`Rule`].
    #[inline]
    pub const fn from_rule(rule: Rule, target: Option<u8>, is_upper_case: bool) -> Self {
        Self::new(
            rule.key,
            rule.effect_on,
            rule.result,
            target,
            rule.effect,
            rule.effect_type,
            is_upper_case,
        )
    }

    /// Returns the target transformation index, if any.
    #[inline]
    pub const fn target(&self) -> Option<u8> {
        if self.target_idx == TARGET_NONE { None } else { Some(self.target_idx) }
    }

    /// Returns the raw target index (`0xFF` for None).
    #[inline]
    pub const fn target_raw(&self) -> u8 {
        self.target_idx
    }

    /// Sets or clears the target transformation index.
    #[inline]
    pub fn set_target(&mut self, target: Option<u8>) {
        self.target_idx = target.unwrap_or(TARGET_NONE);
    }

    /// Returns true if this transformation has a target.
    #[inline]
    pub const fn has_target(&self) -> bool {
        self.target_idx != TARGET_NONE
    }

    /// Retrieves the effect value as a [`Tone`].
    #[inline]
    pub const fn tone(&self) -> Tone {
        match self.effect {
            1 => Tone::Grave,
            2 => Tone::Acute,
            3 => Tone::Hook,
            4 => Tone::Tilde,
            5 => Tone::Dot,
            _ => Tone::None,
        }
    }
}

/// A stack-allocated buffer for transformations to avoid heap allocations in the hot path.
///
/// This structure uses a fixed-size array and is extremely fast for frequent updates.
/// Slots are [`MaybeUninit`]: constructing a stack is free (no zero-fill of the
/// whole array on every slow-path keystroke — 512 B at [`MAX_ACTIVE_TRANS`] 32),
/// and only the `len` used slots are ever read.
#[derive(Clone, Copy)]
pub struct TransformationStack {
    pub(crate) data: [MaybeUninit<Transformation>; MAX_ACTIVE_TRANS],
    pub(crate) len: usize,
}

impl TransformationStack {
    /// Creates a new, empty transformation stack.
    pub const fn new() -> Self {
        Self { data: [const { MaybeUninit::uninit() }; MAX_ACTIVE_TRANS], len: 0 }
    }

    /// Pushes a new transformation onto the stack.
    /// Does nothing if the stack is full.
    pub fn push(&mut self, t: Transformation) {
        debug_assert!(
            self.len < MAX_ACTIVE_TRANS,
            "TransformationStack overflow: max {MAX_ACTIVE_TRANS} reached"
        );
        if self.len < MAX_ACTIVE_TRANS {
            self.data[self.len].write(t);
            self.len += 1;
        }
    }

    /// Clears all transformations from the stack.
    pub const fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns the number of transformations currently in the stack.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the stack contains no transformations.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a slice containing all transformations in the stack.
    pub const fn as_slice(&self) -> &[Transformation] {
        // SAFETY: slots `[..len]` are initialized by `push`/`extend_from_slice`.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().cast::<Transformation>(), self.len) }
    }

    /// Returns a mutable slice containing all transformations in the stack.
    pub const fn as_mut_slice(&mut self) -> &mut [Transformation] {
        // SAFETY: slots `[..len]` are initialized by `push`/`extend_from_slice`.
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().cast::<Transformation>(),
                self.len,
            )
        }
    }

    /// Appends a slice of transformations to the stack.
    pub fn extend_from_slice(&mut self, other: &[Transformation]) {
        let to_copy = other.len().min(MAX_ACTIVE_TRANS - self.len);
        for (slot, &t) in self.data[self.len..self.len + to_copy].iter_mut().zip(other) {
            slot.write(t);
        }
        self.len += to_copy;
    }

    /// Drains transformations from a starting index into another stack.
    pub fn drain_to(&mut self, start: usize, target: &mut TransformationStack) {
        target.clear();
        if start < self.len {
            target.extend_from_slice(&self.as_slice()[start..]);
            self.len = start;
        }
    }
}

impl Default for TransformationStack {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TransformationStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransformationStack")
            .field("data", &self.as_slice())
            .field("len", &self.len)
            .finish()
    }
}

impl PartialEq for TransformationStack {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for TransformationStack {}

impl std::hash::Hash for TransformationStack {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}
