//! DFA-based engine for high-performance Vietnamese input method.
//!
//! This module provides the Deterministic Finite Automaton (DFA) state management,
//! bitset fast-path transition lookups, arena allocation for transformations,
//! and the JIT compiler for pre-compiling common syllable transitions.

pub mod flattener;

use crate::engine::Transformation;
use crate::input_method::InputMethod;
use rustc_hash::FxHashMap;

/// Maximum transitions per DFA state. Vietnamese input typically uses ~12 keys per state.
const MAX_TRANS: usize = 16;

/// Maximum DFA state ID that fits in `trans_states: [u16; 16]`.
const MAX_STATE_ID: u32 = 0xFFFF;

/// Hard cap on the number of DFA states (including state 0).
///
/// Bounds long-lived Engine memory: `8192*64B states + ~8192*~5*16B arena +
/// flat + map ≈ 1.5–2MB used`. Valid Vietnamese needs ~4k states
/// (`fc*vowel*tone`), so 8192 leaves 2x headroom while freezing adversarial
/// unique spam (which previously grew unbounded past 1M states / 200MB+).
/// When full, [`Dfa::add_state`] returns 0 for new compositions (dedup hits
/// still succeed) and callers fall back to the rule-engine slow path, which
/// stays correct but slower for the long tail.
pub const DFA_MAX_STATES: usize = 8192;

/// A compact DFA state representing a unique syllable composition.
///
/// Transitions are stored as state IDs sorted by key (no key array: the
/// 128-bit bitset already records exactly which keys exist). A 128-bit
/// bitset enables $O(1)$ "has transition?" checks and fast rejection for
/// keys that don't have transitions.
///
/// Memory layout fits a single 64-byte cache line: the transition lookup
/// fields (bitset + state IDs) share line 0 with the arena offsets and
/// lengths. The composition hash lives in a side table on [`Dfa`]
/// (`comp_hashes`, cold dedup-only) instead of bloating the hot struct.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct State {
    /// 128-bit bitset: bit `i = 1` means key `i` has a transition (16 bytes: offset 0..16).
    pub bitset: [u64; 2],
    /// Destination state IDs, sorted by key (32 bytes: offset 16..48).
    /// Lookup rank = popcount of bits below the key.
    pub trans_states: [u16; MAX_TRANS],
    /// Start index in the DFA arena (4 bytes: offset 48..52).
    pub comp_offset: u32,
    /// Start index in the flattened-output arena (4 bytes: offset 52..56).
    pub flat_offset: u32,
    /// Number of valid transitions (1 byte: offset 56).
    pub trans_len: u8,
    /// Number of transformations in this state (1 byte: offset 57).
    pub comp_len: u8,
    /// Byte length of the flattened output in `flat_arena` (1 byte: offset 58).
    pub flat_len: u8,
    /// Explicit padding to 8-byte alignment (5 bytes: offset 59..64).
    pub _pad: [u8; 5],
}

const _: () = assert!(std::mem::size_of::<State>() == 64);

impl Default for State {
    fn default() -> Self {
        Self {
            bitset: [0; 2],
            trans_states: [0; MAX_TRANS],
            comp_offset: 0,
            flat_offset: 0,
            trans_len: 0,
            comp_len: 0,
            flat_len: 0,
            _pad: [0; 5],
        }
    }
}

impl State {
    /// Performs a fast transition lookup on an ASCII key using bitset rank.
    ///
    /// `trans_states` is sorted by key and the bitset records exactly which
    /// keys exist, so lookup = one bit test plus the popcount rank of lower
    /// bits — constant time with no key scan:
    /// 1. `hit` = bit `key` of the 128-bit bitset (0/1 multiplier).
    /// 2. `rank` = popcount of bits below `key` = index of this key's slot.
    ///
    /// # Arguments
    /// * `key` - The ASCII byte of the key to look up (0..127).
    ///
    /// # Returns
    /// The destination state ID (non-zero), or `0` if no transition exists.
    #[inline]
    pub const fn get_transition(&self, key: u8) -> u32 {
        debug_assert!(key < 128);
        let idx = key as usize;
        let word = idx >> 6;
        let bit = idx & 63;
        let current = self.bitset[word];
        let hit = ((current >> bit) & 1) as u32;
        let lower_in_word = current & ((1u64 << bit) - 1);
        // `word` is 0 or 1. Turn it into an all-zero/all-one mask so lane
        // selection does not need a data-dependent branch.
        let prior_word_mask = 0usize.wrapping_sub(word);
        let rank = lower_in_word.count_ones() as usize
            + (self.bitset[0].count_ones() as usize & prior_word_mask);
        // On a hit, `rank` is the key's slot (`< trans_len <= 16`). On a miss
        // it can reach 16 (all keys below), so mask before indexing; the zero
        // `hit` multiplier discards the loaded value either way.
        self.trans_states[rank & 15] as u32 * hit
    }

    /// Sets or updates a transition from this state on a given key.
    ///
    /// Keeps `trans_states` sorted by key (rank = popcount of lower bits
    /// doubles as the update position and the insert position — no scan).
    /// State IDs above the `u16` address space are silently dropped, and a
    /// 17th distinct key is rejected WITHOUT setting its bitset bit,
    /// preserving the `popcount(bitset) == trans_len` invariant the rank
    /// lookup relies on.
    ///
    /// # Arguments
    /// * `key` - The ASCII key trigger.
    /// * `state_id` - The target DFA state ID.
    #[inline]
    pub fn set_transition(&mut self, key: u8, state_id: u32) {
        if state_id > MAX_STATE_ID {
            return;
        }
        let sid = state_id as u16;
        let idx = key as usize;
        let bits = (self.bitset[0] as u128) | ((self.bitset[1] as u128) << 64);
        let rank = (bits & ((1u128 << idx) - 1)).count_ones() as usize;
        if ((bits >> idx) & 1) != 0 {
            // Key already stored at `rank`: update in place, order kept.
            self.trans_states[rank] = sid;
            return;
        }
        let len = self.trans_len as usize;
        if len >= MAX_TRANS {
            // Full: reject and leave the bitset untouched.
            return;
        }
        self.trans_states.copy_within(rank..len, rank + 1);
        self.trans_states[rank] = sid;
        self.bitset[idx / 64] |= 1u64 << (idx % 64);
        self.trans_len += 1;
    }
}

/// The DFA core that manages states and transitions with Arena Allocation.
///
/// Stores all unique composition states and provides fast $O(1)$ lookups
/// without heap allocation during state transitions.
#[derive(Debug)]
pub struct Dfa {
    /// Array of all DFA states. State 0 is always the initial (empty) state.
    pub states: Vec<State>,
    /// Composition hashes parallel to `states` (cold dedup-only; kept out of
    /// [`State`] so hot states fit one cache line).
    pub comp_hashes: Vec<u64>,
    /// Continuous arena storing transformation slices for all states.
    pub arena: Vec<Transformation>,
    /// Flattened UTF-8 output cache, keyed by `State::flat_offset` / `flat_len`.
    pub flat_arena: Vec<u8>,
    /// Maps composition hash to `state_id` for $O(1)$ deduplication and lookup.
    pub hash_to_state: FxHashMap<u64, u32>,
}

impl Clone for Dfa {
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            comp_hashes: self.comp_hashes.clone(),
            arena: self.arena.clone(),
            flat_arena: self.flat_arena.clone(),
            hash_to_state: self.hash_to_state.clone(),
        }
    }
}

impl Default for Dfa {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes a fast hash of a composition slice.
///
/// Uses the derived `Hash` impl, which hashes only the initialized fields and
/// never reads struct padding bytes. Hashing raw struct bytes directly would
/// read uninitialized padding (UB, flagged by Miri, and nondeterministic
/// across builds) — so the field-based hash is both correct and Miri-clean.
#[inline]
pub(crate) fn hash_composition(composition: &[Transformation]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    composition.hash(&mut hasher);
    hasher.finish()
}

impl Dfa {
    /// Creates a new DFA with an initial empty state (state ID 0).
    ///
    /// Arenas start unallocated and grow on demand: a fresh `Engine` costs no
    /// ~20 KiB upfront, which matters for short-lived engines (batch items,
    /// FFI create/use/free, tests). Long sessions amortize the growth
    /// reallocations during warmup.
    pub fn new() -> Self {
        let mut dfa = Self {
            states: Vec::new(),
            comp_hashes: Vec::new(),
            arena: Vec::new(),
            flat_arena: Vec::new(),
            hash_to_state: FxHashMap::default(),
        };
        dfa.states.push(State::default());
        dfa.comp_hashes.push(0); // state 0 (empty) hash = 0, as before
        dfa.hash_to_state.insert(0, 0); // empty composition hash = 0
        dfa
    }

    /// Returns a reference to the [`State`] with the given ID.
    ///
    /// # Panics
    /// Panics if `id as usize` is out of bounds of the `states` vector.
    pub fn get_state(&self, id: u32) -> &State {
        &self.states[id as usize]
    }

    /// Retrieves the slice of [`Transformation`]s belonging to the specified state.
    pub fn get_composition(&self, state_id: u32) -> &[Transformation] {
        let state = &self.states[state_id as usize];
        let start = state.comp_offset as usize;
        let end = start + state.comp_len as usize;
        &self.arena[start..end]
    }

    /// Returns the cached flattened UTF-8 output for the specified state.
    ///
    /// Returns an empty slice if the state has no cached flatten (e.g. state 0).
    pub fn get_flat(&self, state_id: u32) -> &str {
        let state = &self.states[state_id as usize];
        let start = state.flat_offset as usize;
        let end = start + state.flat_len as usize;
        std::str::from_utf8(&self.flat_arena[start..end]).unwrap_or("")
    }

    /// Returns cached output for an engine-owned DFA without revalidating UTF-8.
    ///
    /// This stays crate-private because the public `Dfa` fields can be mutated
    /// by advanced callers. An `Engine` owns its DFA privately, and every byte
    /// in its flat arena is appended by `char::encode_utf8` through
    /// `append_flatten_bytes`, so the selected slice is always valid UTF-8.
    #[inline]
    pub(crate) fn get_flat_cached(&self, state_id: u32) -> &str {
        let state = &self.states[state_id as usize];
        let start = state.flat_offset as usize;
        let end = start + state.flat_len as usize;
        let bytes = &self.flat_arena[start..end];
        debug_assert!(std::str::from_utf8(bytes).is_ok());
        // SAFETY: Engine-owned flat_arena bytes are produced exclusively by
        // encoding Rust `char` values as UTF-8; offsets and lengths are stored
        // at the same append operation in Dfa::add_state.
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    /// Returns true when the DFA has hit [`DFA_MAX_STATES`] and is frozen.
    ///
    /// Frozen DFA still serves all cached transitions (fast path untouched);
    /// new compositions fall back to the rule-engine slow path.
    pub const fn is_full(&self) -> bool {
        self.states.len() >= DFA_MAX_STATES
    }

    /// Adds a new composition state to the DFA or returns the existing state ID if already present.
    ///
    /// Uses the precomputed hash and arena verification for fast collision-free deduplication.
    /// On hash collision, scans backwards through earlier states sharing the same
    /// hash (`comp_hashes`) so previously inserted compositions remain reachable.
    /// Also caches the flattened lowercase output for zero-cost preedit reconstruction.
    ///
    /// When the DFA is full ([`DFA_MAX_STATES`]) or the next ID would exceed the
    /// `u16` transition address space, new compositions are **refused**: dedup
    /// hits still return their existing ID, but misses return `0` without
    /// allocating. Callers must skip `set_transition` on `0` and fall back to
    /// `find_state(...).unwrap_or(0)` (rule-engine slow path stays correct).
    pub fn add_state(&mut self, composition: &[Transformation]) -> u32 {
        // Arena-bounded: `comp_len`/`flat_len` below truncate into `u8`.
        debug_assert!(composition.len() <= u8::MAX as usize);
        let hash = hash_composition(composition);

        // Fast path: hash match -> verify arena equality (no heap allocation).
        if let Some(&id) = self.hash_to_state.get(&hash) {
            if self.get_composition(id) == composition {
                return id;
            }
            // Hash collision: `hash_to_state` stores only the most recent state for
            // this hash. Scan backwards so older compositions with the same hash
            // are still found (and not duplicated).
            for old_id in (0..id).rev() {
                if self.comp_hashes[old_id as usize] == hash
                    && self.get_composition(old_id) == composition
                {
                    return old_id;
                }
            }
        }

        // Freeze: refuse new storage (also fixes the old zombie leak where
        // states kept pushing past u16 addressability with dropped edges).
        if self.states.len() >= DFA_MAX_STATES || self.states.len() > MAX_STATE_ID as usize {
            return 0;
        }

        let id = self.states.len() as u32;
        let comp_offset = self.arena.len() as u32;
        let comp_len = composition.len() as u8;

        // Cache the flattened lowercase output for P1 fast preedit reconstruction,
        // writing UTF-8 straight into the arena (no temporary `String`).
        let flat_offset = self.flat_arena.len() as u32;
        crate::flattener::append_flatten_bytes(
            composition,
            crate::mode::OutputOptions::LOWER_CASE,
            &mut self.flat_arena,
        );
        let flat_len = (self.flat_arena.len() - flat_offset as usize) as u8;

        self.arena.extend_from_slice(composition);
        self.states.push(State {
            comp_offset,
            comp_len,
            flat_offset,
            flat_len,
            ..State::default()
        });
        self.comp_hashes.push(hash);

        self.hash_to_state.insert(hash, id);
        id
    }

    /// Returns the number of bytes stored in the flattened-output arena.
    pub const fn flat_len(&self) -> usize {
        self.flat_arena.len()
    }

    /// Returns Vec capacities (allocated slots) for memory accounting.
    pub const fn states_capacity(&self) -> usize {
        self.states.capacity()
    }

    /// Returns arena capacity in [`Transformation`] slots.
    pub const fn arena_capacity(&self) -> usize {
        self.arena.capacity()
    }

    /// Returns flat-arena capacity in bytes.
    pub const fn flat_capacity(&self) -> usize {
        self.flat_arena.capacity()
    }

    /// Returns the hash-map item capacity (≈7/8 of `SwissTable` buckets) for memory accounting.
    pub fn map_capacity(&self) -> usize {
        self.hash_to_state.capacity()
    }

    /// Estimates DFA heap usage (`used` bytes, without Vec slack).
    ///
    /// Formula: `states.len()*64 + arena.len()*16 + flat.len()*1 + map.len()*16`.
    /// Map per-entry cost uses 16 B (8 B key + 4 B value + alignment) + ~1 B
    /// `SwissTable` control byte, rounded to 16 B/entry for a conservative floor.
    /// Real allocator usage is slightly higher (control bytes + group overhead).
    pub fn memory_used_bytes(&self) -> usize {
        self.states.len() * std::mem::size_of::<State>()
            + self.arena.len() * std::mem::size_of::<Transformation>()
            + self.flat_arena.len()
            + self.hash_to_state.len() * 16
    }

    /// Estimates DFA heap allocation (`allocated` bytes, with Vec slack).
    ///
    /// Same formula as [`Self::memory_used_bytes`] but with capacities.
    ///
    /// This is a deliberate floor, not the allocator's real number:
    /// `capacity()` counts items (≈7/8 of `SwissTable` buckets), so the true
    /// map allocation is ~20% higher (`buckets × (16 B data + 1 B ctrl)`).
    /// Use it for relative before/after comparisons, not absolute RSS.
    pub fn memory_allocated_bytes(&self) -> usize {
        self.states.capacity() * std::mem::size_of::<State>()
            + self.arena.capacity() * std::mem::size_of::<Transformation>()
            + self.flat_arena.capacity()
            + self.hash_to_state.capacity() * 16
    }

    /// Checks whether `composition` equals the stored composition of `state_id`,
    /// ignoring `is_upper_case` bits.
    ///
    /// The DFA stores lowercase-canonical compositions (the engine lowercases
    /// before caching and re-applies case on fast-path restore), so an active
    /// buffer carrying uppercase bits still identifies the same state.
    /// Used to gate JIT edge linking: an edge may only be written when the
    /// source state identifies the pre-key composition.
    /// Packs the compared [`Transformation`] fields into one `u128` lane set:
    /// `key` @ bytes 0..4, `effect_on` @ 4..8, `result` @ 8..12, target @ 12,
    /// `effect` @ 13, `effect_type` @ 14. Byte 15 (`is_upper_case`) is left
    /// out so one masked XOR compares a whole record branchlessly.
    #[inline]
    const fn canonical_word(t: &Transformation) -> u128 {
        (t.key as u128)
            | ((t.effect_on as u128) << 32)
            | ((t.result as u128) << 64)
            | ((t.target_raw() as u128) << 96)
            | ((t.effect as u128) << 104)
            | ((t.effect_type as u128) << 112)
    }

    /// Checks whether `composition` equals the stored composition of `state_id`,
    /// ignoring `is_upper_case` bits.
    ///
    /// The DFA stores lowercase-canonical compositions (the engine lowercases
    /// before caching and re-applies case on fast-path restore), so an active
    /// buffer carrying uppercase bits still identifies the same state.
    /// Used to gate JIT edge linking: an edge may only be written when the
    /// source state identifies the pre-key composition.
    pub fn composition_matches_canonical(
        &self,
        state_id: u32,
        composition: &[Transformation],
    ) -> bool {
        let stored = self.get_composition(state_id);
        if stored.len() != composition.len() {
            return false;
        }
        // Mask out byte 15 (`is_upper_case`): one XOR per record, no per-field
        // branches. `Transformation` is 16 B with no padding, and the packed
        // layout above mirrors its field order.
        const CASE_MASK: u128 = 0x00FF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;
        stored
            .iter()
            .zip(composition.iter())
            .all(|(a, b)| ((Self::canonical_word(a) ^ Self::canonical_word(b)) & CASE_MASK) == 0)
    }

    /// Finds the state ID corresponding to an existing composition slice, if present.
    ///
    /// On hash collision, falls back to a backward scan over earlier states with
    /// the same hash (`comp_hashes`; mirrors [`Dfa::add_state`]).
    pub fn find_state(&self, composition: &[Transformation]) -> Option<u32> {
        let hash = hash_composition(composition);
        let id = *self.hash_to_state.get(&hash)?;
        if self.get_composition(id) == composition {
            return Some(id);
        }
        // Hash collision: scan backwards for an earlier state with the same hash.
        (0..id).rev().find(|&old_id| {
            self.comp_hashes[old_id as usize] == hash && self.get_composition(old_id) == composition
        })
    }
}

/// A DFA compiler that pre-initializes common syllable states into a [`Dfa`].
pub struct DfaCompiler {
    /// The compiled DFA instance.
    pub dfa: Dfa,
    engine: crate::Engine,
}

impl DfaCompiler {
    /// Creates a new compiler instance for a given input method and configuration.
    pub fn new(im: &InputMethod, config: crate::Config) -> Self {
        let engine = crate::Engine::with_config(im.clone(), config);
        Self { dfa: Dfa::new(), engine }
    }

    /// Compiles common Vietnamese syllables into the DFA.
    pub fn compile_common(&mut self) {
        let fc = [
            "", "b", "c", "ch", "d", "dd", "g", "gh", "h", "k", "kh", "l", "m", "n", "nh", "ng",
            "ngh", "p", "ph", "q", "r", "s", "t", "th", "tr", "v", "x",
        ];
        let vowels = [
            "a", "e", "i", "o", "u", "y", "aa", "ee", "oo", "aw", "ow", "uw", "ai", "ao", "au",
            "ay", "ie", "oa", "oe", "oi", "ua", "ue", "ui", "uo", "uy",
        ];
        let tones = ["", "s", "f", "r", "x", "j"];

        // Stack-allocated buffer: max prefix "ngh" (3) + max vowel "uay" (3) + tone (1) = 7
        let mut buf = [0u8; 8];
        for &f in &fc {
            for &v in &vowels {
                for &t in &tones {
                    let mut pos = 0;
                    for &b in f.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    for &b in v.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    for &b in t.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    if let Ok(seq) = std::str::from_utf8(&buf[..pos]) {
                        self.simulate_str(seq);
                    }
                }
            }
        }
    }

    fn simulate_str(&mut self, s: &str) {
        self.engine.reset();

        let mut current_state = 0u32;
        // State 0 identifies empty active text at word start. Afterwards the
        // previous state only identifies the pre-key composition when its own
        // add succeeded (a refusal leaves an unidentified residue that must
        // not gain word-start — or any — edges).
        let mut prev_known = true;
        for k in s.chars() {
            if !k.is_ascii() {
                continue;
            }

            let prev_state = current_state;
            let link_ok = prev_known;
            self.engine.process_key(k, crate::Mode::Vietnamese);

            let comp = self.engine.active_slice();
            current_state = self.dfa.add_state(comp);

            // Link the transition (skip when frozen: add_state returns 0 for
            // new compositions; 0 already means "no transition" on lookup).
            // Skip equally when the source state does not identify the
            // pre-key composition.
            if current_state != 0 {
                if link_ok {
                    self.dfa.states[prev_state as usize].set_transition(k as u8, current_state);
                }
                prev_known = true;
            } else {
                prev_known = comp.is_empty();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Transformation;
    use crate::input_method::EffectType;
    use proptest::prelude::*;

    fn appending(ch: char) -> Transformation {
        Transformation::new(ch, ch, ch, None, 0, EffectType::Appending, false)
    }

    #[test]
    fn add_state_deduplicates_identical_composition() {
        let mut dfa = Dfa::new();
        let comp = [appending('a'), appending('b')];
        let id1 = dfa.add_state(&comp);
        let id2 = dfa.add_state(&comp);
        assert_eq!(id1, id2);
        assert_eq!(dfa.states.len(), 2); // state 0 (empty) + one real
    }

    #[test]
    fn find_state_roundtrip() {
        let mut dfa = Dfa::new();
        let comp_a = [appending('x')];
        let comp_b = [appending('y')];
        let id_a = dfa.add_state(&comp_a);
        let id_b = dfa.add_state(&comp_b);
        assert_ne!(id_a, id_b);
        assert_eq!(dfa.find_state(&comp_a), Some(id_a));
        assert_eq!(dfa.find_state(&comp_b), Some(id_b));
    }

    #[test]
    fn engine_flat_cache_is_valid_utf8_while_public_reader_handles_corruption() {
        let mut dfa = Dfa::new();
        let comp = [appending('ư'), appending('ơ')];
        let id = dfa.add_state(&comp);
        assert_eq!(dfa.get_flat_cached(id), dfa.get_flat(id));
        assert_eq!(dfa.get_flat_cached(id), "ươ");

        // Advanced users can mutate public DFA storage, so the public reader
        // must remain checked even though Engine's private DFA uses the faster
        // invariant-based accessor.
        let offset = dfa.states[id as usize].flat_offset as usize;
        dfa.flat_arena[offset] = 0xFF;
        assert_eq!(dfa.get_flat(id), "");
    }

    /// Regression: on hash collision, older compositions must remain reachable
    /// via `find_state` and `add_state` must not create duplicates.
    #[test]
    fn hash_collision_preserves_both_states() {
        let mut dfa = Dfa::new();
        let comp_a = [appending('p'), appending('q')];
        let comp_b = [appending('r'), appending('s')];

        let id_a = dfa.add_state(&comp_a);
        let id_b = dfa.add_state(&comp_b);
        assert_ne!(id_a, id_b);

        // Simulate a hash collision: force both states to share the same hash.
        let forced_hash = 0xDEAD_BEEF_CAFE_F00D_u64;
        dfa.comp_hashes[id_a as usize] = forced_hash;
        dfa.comp_hashes[id_b as usize] = forced_hash;
        dfa.hash_to_state.insert(forced_hash, id_b); // map points at newer state

        // Both compositions must still be findable.
        assert_eq!(dfa.find_state(&comp_a), Some(id_a));
        assert_eq!(dfa.find_state(&comp_b), Some(id_b));

        // Re-adding must not create duplicates.
        assert_eq!(dfa.add_state(&comp_a), id_a);
        assert_eq!(dfa.add_state(&comp_b), id_b);
        assert_eq!(dfa.states.len(), 3); // state 0 + id_a + id_b, no duplicate
    }

    #[test]
    fn freeze_bounds_memory_and_keeps_dedup() {
        let mut dfa = Dfa::new();
        // Fill to the cap with distinct 2-char compositions.
        // ('a'+hi, 'a'+lo) over i < 8192 gives fully distinct pairs.
        let mut i = 0u32;
        while !dfa.is_full() {
            let hi = char::from_u32('a' as u32 + (i / 64)).unwrap_or('x');
            let lo = char::from_u32('a' as u32 + (i % 64)).unwrap_or('y');
            let comp = [appending(hi), appending(lo)];
            let id = dfa.add_state(&comp);
            assert_ne!(id, 0, "distinct comp must allocate before full");
            i += 1;
            assert!(i < DFA_MAX_STATES as u32 + 10, "cap must trigger");
        }
        assert_eq!(dfa.states.len(), DFA_MAX_STATES);

        let used = dfa.memory_used_bytes();
        // 8192*64 states + arena + flat + map must stay in low single-digit MB.
        assert!(used < 4 * 1024 * 1024, "frozen DFA used={} too big", used);

        // A fresh composition is refused without allocating.
        // Char 0x10FFFF never appears in the fill range above.
        let fresh = [appending('\u{10FFFF}'), appending('\u{10FFFE}')];
        assert_eq!(dfa.find_state(&fresh), None);
        let before = dfa.states.len();
        assert_eq!(dfa.add_state(&fresh), 0);
        assert_eq!(dfa.states.len(), before);

        // Dedup hits still succeed when full.
        let dup = [appending('a'), appending('a')]; // i == 0
        let dup_id = dfa.add_state(&dup);
        assert_ne!(dup_id, 0);
        assert_eq!(dfa.states.len(), before);
        assert!(dfa.is_full());
    }

    #[test]
    fn memory_accounting_is_internally_consistent() {
        let mut dfa = Dfa::new();
        let comp = [appending('a'), appending('b')];
        dfa.add_state(&comp);
        assert!(dfa.memory_allocated_bytes() >= dfa.memory_used_bytes());
        assert!(dfa.states_capacity() >= dfa.states.len());
        assert!(dfa.arena_capacity() >= dfa.arena.len());
        assert!(dfa.flat_capacity() >= dfa.flat_len());
        assert!(dfa.map_capacity() >= dfa.hash_to_state.len());
    }

    #[test]
    fn composition_matches_canonical_ignores_case_only() {
        let mut dfa = Dfa::new();
        let lower = [appending('h'), appending('i')];
        let id = dfa.add_state(&lower);

        // Identical and case-only differences match (DFA is lowercase-canonical).
        assert!(dfa.composition_matches_canonical(id, &lower));
        let mut upper = lower;
        upper[0].is_upper_case = true;
        assert!(dfa.composition_matches_canonical(id, &upper));
        let mut all_upper = lower;
        for t in &mut all_upper {
            t.is_upper_case = true;
        }
        assert!(dfa.composition_matches_canonical(id, &all_upper));

        // Length, key, result, and target differences must not match.
        assert!(!dfa.composition_matches_canonical(id, &[]));
        assert!(!dfa.composition_matches_canonical(id, &[appending('h')]));
        let mut other_key = lower;
        other_key[1] = appending('o');
        assert!(!dfa.composition_matches_canonical(id, &other_key));
        let mut other_result = lower;
        other_result[1].result = 'î';
        assert!(!dfa.composition_matches_canonical(id, &other_result));
        let mut other_target = lower;
        other_target[1].set_target(Some(0));
        assert!(!dfa.composition_matches_canonical(id, &other_target));
    }

    // Invariant for the popcount-rank lookup: the bitset population must
    // equal the stored transition count, and every stored key must resolve
    // to its latest id. FAILS on the current code: the 17th distinct key
    // sets its bitset bit without storing (bit set before the capacity
    // check in `set_transition`).
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]
        #[test]
        fn bitset_len_invariant(ops in prop::collection::vec((0u8..128u8, 1u32..100u32), 0..32)) {
            let mut state = State::default();
            for (key, id) in &ops {
                state.set_transition(*key, *id);
            }
            let pop = state.bitset[0].count_ones() + state.bitset[1].count_ones();
            prop_assert_eq!(pop, state.trans_len as u32, "bitset population == trans_len");
            // Replay model: updates stick, only the first 16 distinct keys store.
            let mut expected: Vec<(u8, u32)> = Vec::new();
            for (key, id) in &ops {
                if let Some(slot) = expected.iter_mut().find(|(k, _)| k == key) {
                    slot.1 = *id;
                } else if expected.len() < MAX_TRANS {
                    expected.push((*key, *id));
                }
            }
            for (key, id) in &expected {
                prop_assert_eq!(state.get_transition(*key), *id);
            }
        }
    }

    /// The 17th distinct key must be rejected WITHOUT setting its bitset bit
    /// (otherwise the rank scheme miscounts). FAILS on the current code.
    #[test]
    fn set_transition_full_capacity_rejects_17th_key() {
        let mut state = State::default();
        for i in 0..16u8 {
            state.set_transition(b'a' + i, 100 + i as u32);
        }
        assert_eq!(state.trans_len, 16);
        state.set_transition(b'z', 200);
        let bit = (state.bitset[b'z' as usize / 64] >> (b'z' as usize % 64)) & 1;
        assert_eq!(bit, 0, "17th key must not set its bitset bit");
        assert_eq!(state.get_transition(b'z'), 0);
        assert_eq!(state.trans_len, 16);
        for i in 0..16u8 {
            assert_eq!(state.get_transition(b'a' + i), 100 + i as u32, "stored key survives");
        }
    }

    /// Characterization: `get_transition` semantics across every lane of the
    /// 16-key scan (lanes 0, 7, 8, 15 are the old two-chunk boundaries) plus
    /// miss cases. Pins behavior before the branchless rewrite.
    #[test]
    fn get_transition_all_lanes_and_misses() {
        let mut state = State::default();

        // Empty state: any key misses.
        assert_eq!(state.get_transition(b'a'), 0);
        assert_eq!(state.get_transition(0), 0);

        // Fill all 16 lanes with distinct destinations (keys 'a'..='p').
        for i in 0..16u32 {
            state.set_transition(b'a' + i as u8, 100 + i);
        }

        // Every lane hits with its own destination, including the old
        // chunk boundaries (lane 0, 7, 8) and the last lane (15).
        for i in 0..16u32 {
            assert_eq!(state.get_transition(b'a' + i as u8), 100 + i, "lane {i}");
        }

        // Absent keys miss: before range, between range and tail, after range.
        assert_eq!(state.get_transition(b'`'), 0);
        assert_eq!(state.get_transition(b'q'), 0);
        assert_eq!(state.get_transition(b'z'), 0);

        // Single-transition states at lane 0 and lane 15.
        let mut lane0 = State::default();
        lane0.set_transition(b'a', 7);
        assert_eq!(lane0.get_transition(b'a'), 7);
        assert_eq!(lane0.get_transition(b'b'), 0);

        let mut lane15 = State::default();
        lane15.set_transition(b'p', 9);
        assert_eq!(lane15.get_transition(b'p'), 9);
        assert_eq!(lane15.get_transition(b'o'), 0);
    }
}
