/// Flag bit: snapshot was taken while `english_bypass` was active.
pub(crate) const FLAG_ENGLISH_BYPASS: u8 = 1;
/// Flag bit: snapshot's composition is stored in `bypass_buffers`
/// (not recoverable from the DFA arena).
pub(crate) const FLAG_NEEDS_BUFFER: u8 = 2;

/// A compact 8-byte snapshot of engine state before a keystroke, used for O(1) backspace.
///
/// The active composition itself is **not** stored here when it is recoverable
/// from the DFA arena (`state_id != 0`). Only the metadata needed to restore is
/// kept, shrinking the snapshot from 528 B to 8 B. When the composition is not
/// in the DFA (English bypass, word-break, or slow-path fallback), the caller
/// copies the buffer into `Engine::bypass_buffers` and sets [`FLAG_NEEDS_BUFFER`].
///
/// Field order packs exactly 8 bytes: state IDs fit `u16` (DFA is capped at
/// [`crate::dfa::DFA_MAX_STATES`] ≪ `u16::MAX`).
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub(crate) struct Snapshot {
    /// Bitmask of `is_upper_case` flags: bit *i* = `active_buffer[i].is_upper_case`.
    pub(crate) upper_mask: u32,
    /// DFA state id at snapshot time (0 = empty / not in DFA).
    pub(crate) state_id: u16,
    /// Length of `active_buffer` at snapshot time.
    pub(crate) active_len: u8,
    /// Combination of [`FLAG_ENGLISH_BYPASS`] and [`FLAG_NEEDS_BUFFER`].
    pub(crate) flags: u8,
}

const _: () = assert!(std::mem::size_of::<Snapshot>() == 8);
