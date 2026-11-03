//! C-Compatible FFI Layer for Bamboo Core.
//!
//! This module provides an `extern "C"` API for integrating Bamboo with
//! other languages like C, C++, Python, and IME frameworks (Fcitx5, `IBus`).
//! `include/bamboo_core.h` declares the supported C ABI. It exposes only the
//! composition engine: frontends own their preedit UI, event routing,
//! dictionaries, macros, and platform integration.

#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use crate::config::Config;
use crate::engine::{Engine, LastWordForm, ToneRefresh};
use crate::input_method::InputMethod;
use crate::mode::{Mode, OutputOptions};

/// C-compatible enum representing supported Vietnamese input methods for FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BambooMethod {
    /// Telex input method.
    #[default]
    Telex = 0,
    /// VNI input method.
    Vni = 1,
    /// VIQR input method.
    Viqr = 2,
    /// Microsoft Standard layout.
    MicrosoftLayout = 3,
    /// Telex 2 input method.
    Telex2 = 4,
    /// Telex W input method.
    TelexW = 5,
    /// Telex + VNI hybrid method.
    TelexVni = 6,
    /// Telex + VNI + VIQR combination method.
    TelexVniViqr = 7,
    /// VNI French layout method.
    VniFrenchLayout = 8,
}

impl BambooMethod {
    /// Converts an integer to a [`BambooMethod`], defaulting to [`BambooMethod::Telex`].
    pub const fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::Vni,
            2 => Self::Viqr,
            3 => Self::MicrosoftLayout,
            4 => Self::Telex2,
            5 => Self::TelexW,
            6 => Self::TelexVni,
            7 => Self::TelexVniViqr,
            8 => Self::VniFrenchLayout,
            _ => Self::Telex,
        }
    }

    /// Converts the enum variant to its corresponding [`InputMethod`].
    pub fn to_input_method(self) -> InputMethod {
        match self {
            Self::Telex => InputMethod::telex(),
            Self::Vni => InputMethod::vni(),
            Self::Viqr => InputMethod::viqr(),
            Self::MicrosoftLayout => InputMethod::microsoft_layout(),
            Self::Telex2 => InputMethod::telex_2(),
            Self::TelexW => InputMethod::telex_w(),
            Self::TelexVni => InputMethod::telex_vni(),
            Self::TelexVniViqr => InputMethod::telex_vni_viqr(),
            Self::VniFrenchLayout => InputMethod::vni_french_layout(),
        }
    }
}

static GLOBAL_ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

fn with_engine<F, R>(f: F) -> R
where
    F: FnOnce(&mut Engine) -> R,
    R: Default,
{
    let mut guard = match GLOBAL_ENGINE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if guard.is_none() {
        *guard = Some(Engine::new(InputMethod::telex()));
    }
    if let Some(engine) = guard.as_mut() { f(engine) } else { R::default() }
}

/// Initializes the global engine with the default Telex input method.
#[deprecated(since = "0.5.3", note = "use bamboo_engine_new for an independent engine instance")]
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_setup() {
    let mut guard = match GLOBAL_ENGINE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = Some(Engine::new(InputMethod::telex()));
}

/// Resets the global engine state, clearing the current composition.
#[deprecated(since = "0.5.3", note = "use bamboo_engine_reset for an independent engine instance")]
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_reset() {
    with_engine(|e| e.reset());
}

/// Sets the input method for the global engine.
///
/// # Arguments
///
/// * `method` - An integer representing the input method:
///     * 0: Telex
///     * 1: VNI
///     * 2: VIQR
///     * 3: Microsoft Layout
///     * 4: Telex 2
///     * 5: Telex W
///     * 6: Telex + VNI
///     * 7: Telex + VNI + VIQR
///     * 8: VNI French Layout
#[deprecated(
    since = "0.5.3",
    note = "use bamboo_engine_set_input_method for an independent engine instance"
)]
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_set_input_method(method: i32) {
    let im = BambooMethod::from_i32(method).to_input_method();
    let mut guard = match GLOBAL_ENGINE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = Some(Engine::new(im));
}

/// Processes a key and returns the full current word as a C-compatible string.
///
/// # Arguments
///
/// * `key` - The Unicode code point of the key to process.
/// * `is_vietnamese` - Non-zero if the key should be processed as Vietnamese, zero for English mode.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string.
/// **Note:** The caller is responsible for freeing the returned string using [`bamboo_free_string`].
#[deprecated(
    since = "0.5.3",
    note = "use bamboo_engine_process_key_mode or bamboo_engine_process_key_buf"
)]
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_process_key(key: u32, is_vietnamese: i32) -> *mut c_char {
    with_engine(|e| {
        let mode = if is_vietnamese != 0 { Mode::Vietnamese } else { Mode::English };
        if let Some(c) = std::char::from_u32(key) {
            e.process_key(c, mode);
        }
        let out = e.output();
        // Engine output is valid UTF-8 without null bytes; fallback to empty string if somehow invalid.
        CString::new(out.as_ref()).unwrap_or_default().into_raw()
    })
}

/// Processes a key and writes the delta output (inserted UTF-8 bytes) into a caller-provided buffer.
///
/// # Safety
/// - `out_buf` must be a valid pointer to a buffer of at least `out_cap` bytes if `out_cap > 0`.
/// - `out_len`, `backspaces_chars`, and `backspaces_bytes` must be valid, non-null pointers to `usize`.
/// - The caller must ensure that no other thread is accessing the global engine simultaneously (this function uses a Mutex internally for safety, but pointer validity is the caller's responsibility).
#[deprecated(
    since = "0.5.3",
    note = "use bamboo_engine_process_key_buf for an independent engine instance"
)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_process_key_buf(
    key: u32,
    is_vietnamese: i32,
    out_buf: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
    backspaces_chars: *mut usize,
    backspaces_bytes: *mut usize,
) -> i32 {
    if out_len.is_null() || backspaces_chars.is_null() || backspaces_bytes.is_null() {
        return -1;
    }

    // SAFETY: pointers were checked non-null and caller guarantees alignment and validity.
    let (out_len, backspaces_chars, backspaces_bytes) =
        unsafe { (&mut *out_len, &mut *backspaces_chars, &mut *backspaces_bytes) };

    let mode = if is_vietnamese != 0 { Mode::Vietnamese } else { Mode::English };

    let mut guard = match GLOBAL_ENGINE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if guard.is_none() {
        *guard = Some(Engine::new(InputMethod::telex()));
    }
    let Some(e) = guard.as_mut() else {
        return -1;
    };

    let Some(c) = std::char::from_u32(key) else {
        *out_len = 0;
        *backspaces_chars = 0;
        *backspaces_bytes = 0;
        return 0;
    };

    let edit = e.process_key_edit(c, mode);
    let (bs_chars, bs_bytes, inserted) = (edit.delete_chars, edit.delete_bytes, edit.insert);
    let bytes = inserted.as_bytes();

    *out_len = bytes.len();
    *backspaces_chars = bs_chars;
    *backspaces_bytes = bs_bytes;

    if bytes.len() > out_cap {
        return 1;
    }
    if !bytes.is_empty() {
        if out_buf.is_null() {
            return -1;
        }
        // SAFETY: caller guarantees out_buf points to at least out_cap bytes and bytes.len() <= out_cap.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf, bytes.len());
        }
    }

    0
}

/// Returns the current word output as a C-compatible string.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string.
/// **Note:** The caller is responsible for freeing the returned string using [`bamboo_free_string`].
#[deprecated(since = "0.5.3", note = "use bamboo_engine_output for an independent engine instance")]
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_output() -> *mut c_char {
    with_engine(|e| {
        let out = e.output();
        CString::new(out.as_ref()).unwrap_or_default().into_raw()
    })
}

/// Removes the last character from the current composition in the global engine.
#[deprecated(
    since = "0.5.3",
    note = "use bamboo_engine_undo_last_key with an explicit tone-refresh policy"
)]
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_remove_last_char() {
    with_engine(Engine::undo_last_key);
}

/// Frees a string allocated by the engine and returned via FFI.
///
/// # Safety
///
/// The provided pointer must have been returned by a `bamboo_*` function and not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_free_string(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: caller guarantees s was allocated via CString::into_raw by this library.
        let _ = unsafe { CString::from_raw(s) };
    }
}

// --- Instance-based API for multi-context support ---

/// Opaque handle to a Bamboo Engine instance.
pub type BambooEngine = Engine;

const fn mode_from_i32(mode: i32) -> Mode {
    if mode == 0 { Mode::English } else { Mode::Vietnamese }
}

const fn tone_refresh_from_i32(policy: i32) -> ToneRefresh {
    if policy == 1 { ToneRefresh::PreserveSnapshot } else { ToneRefresh::Recompute }
}

const fn last_word_form_from_i32(form: i32) -> LastWordForm {
    if form == 1 { LastWordForm::RawKeystrokes } else { LastWordForm::Vietnamese }
}

/// Creates a new Bamboo Engine instance.
///
/// # Arguments
///
/// * `method` - An integer representing the input method (0: Telex, 1: VNI, 2: VIQR, 3: Microsoft layout, 4: Telex 2, 5: Telex W, 6: Telex+VNI, 7: Telex+VNI+VIQR, 8: VNI French).
///
/// # Returns
///
/// A pointer to the new [`BambooEngine`] instance.
/// **Note:** The caller is responsible for freeing the engine using [`bamboo_engine_free`].
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_engine_new(method: i32) -> *mut BambooEngine {
    let im = BambooMethod::from_i32(method).to_input_method();
    Box::into_raw(Box::new(Engine::new(im)))
}

/// Creates a new Bamboo Engine instance with configuration flags.
///
/// # Arguments
///
/// * `method` - The input method, numbered as in [`bamboo_engine_new`].
/// * `flags` - A bitmask read by [`Config::from_flags`]:
///     * 0x01: free tone marking
///     * 0x02: standard tone style (`hòa`; clear for `hoà`)
///     * 0x04: auto-correct
///     * 0x08: `w` becomes `ư` except at the start of a syllable
///     * 0x10: `w` always becomes `ư` (wins over 0x08)
///     * 0x20: `[ ] { }` type `ơ ư Ơ Ư` except at the start of a word
///     * 0x40: `[ ] { }` always type `ơ ư Ơ Ư` (wins over 0x20)
///
///   [`bamboo_engine_new`] uses `0x07`.
///
/// # Returns
///
/// A pointer to the new [`BambooEngine`] instance.
/// **Note:** The caller is responsible for freeing the engine using [`bamboo_engine_free`].
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_engine_new_with_flags(method: i32, flags: u32) -> *mut BambooEngine {
    let im = BambooMethod::from_i32(method).to_input_method();
    Box::into_raw(Box::new(Engine::with_config(im, Config::from_flags(flags))))
}

/// Frees a Bamboo Engine instance created with [`bamboo_engine_new`].
///
/// # Safety
///
/// The provided pointer must be a valid pointer to a [`BambooEngine`] instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_free(engine: *mut BambooEngine) {
    if !engine.is_null() {
        // SAFETY: caller guarantees engine was created by bamboo_engine_new and not previously freed.
        let _ = unsafe { Box::from_raw(engine) };
    }
}

/// Processes a key using a specific engine instance.
///
/// Each call allocates the returned string; per-keystroke polling loops
/// should prefer [`bamboo_engine_process_key_buf`], which writes into a
/// caller-provided buffer instead.
///
/// # Safety
/// - `engine` must be a valid, non-null pointer to a `BambooEngine` instance.
/// - The caller is responsible for freeing the returned string using `bamboo_free_string`.
#[deprecated(
    since = "0.5.3",
    note = "use bamboo_engine_process_key_mode or bamboo_engine_process_key_buf"
)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_process(engine: *mut BambooEngine, key: u32) -> *mut c_char {
    // SAFETY: engine pointer checked for non-null and safely converted to a mutable reference.
    let Some(e) = (unsafe { engine.as_mut() }) else {
        return ptr::null_mut();
    };
    if let Some(c) = std::char::from_u32(key) {
        e.process_key(c, Mode::Vietnamese);
    }
    let out = e.output();
    CString::new(out.as_ref()).unwrap_or_default().into_raw()
}

/// Processes a key in an explicit mode and returns the active composition.
///
/// `mode` is 0 for English and non-zero for Vietnamese. The returned string
/// must be released with [`bamboo_free_string`]. This is the allocating
/// convenience form; use [`bamboo_engine_process_key_buf`] in a key loop.
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_process_key_mode(
    engine: *mut BambooEngine,
    key: u32,
    mode: i32,
) -> *mut c_char {
    // SAFETY: the caller supplies a live exclusive engine pointer.
    let Some(e) = (unsafe { engine.as_mut() }) else {
        return ptr::null_mut();
    };
    if let Some(c) = char::from_u32(key) {
        e.process_key(c, mode_from_i32(mode));
    }
    CString::new(e.output_str()).unwrap_or_default().into_raw()
}

/// Resets a specific engine, discarding active and committed text.
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_reset(engine: *mut BambooEngine) {
    // SAFETY: the caller supplies a live exclusive engine pointer.
    if let Some(e) = unsafe { engine.as_mut() } {
        e.reset();
    }
}

/// Commits the active composition into the engine's full-text stream.
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_commit(engine: *mut BambooEngine) {
    // SAFETY: the caller supplies a live exclusive engine pointer.
    if let Some(e) = unsafe { engine.as_mut() } {
        e.commit();
    }
}

/// Replaces an engine's configuration using [`Config::from_flags`].
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_set_config_flags(engine: *mut BambooEngine, flags: u32) {
    // SAFETY: the caller supplies a live exclusive engine pointer.
    if let Some(e) = unsafe { engine.as_mut() } {
        e.set_config(Config::from_flags(flags));
    }
}

/// Returns the current configuration in the stable [`Config::to_flags`] form.
/// A null engine yields zero.
///
/// # Safety
/// `engine` must be a valid engine pointer when non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_config_flags(engine: *const BambooEngine) -> u32 {
    // SAFETY: the caller supplies a live engine pointer when non-null.
    unsafe { engine.as_ref() }.map_or(0, |e| e.config().to_flags())
}

/// Replaces the input method while preserving the engine configuration.
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_set_input_method(engine: *mut BambooEngine, method: i32) {
    // SAFETY: the caller supplies a live exclusive engine pointer.
    if let Some(e) = unsafe { engine.as_mut() } {
        let config = e.config();
        *e = Engine::with_config(BambooMethod::from_i32(method).to_input_method(), config);
    }
}

/// Returns the processed text for the requested [`OutputOptions`] bit mask.
/// The returned string must be released with [`bamboo_free_string`].
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_output(
    engine: *const BambooEngine,
    options: u32,
) -> *mut c_char {
    // SAFETY: the caller supplies a live engine pointer.
    let Some(e) = (unsafe { engine.as_ref() }) else {
        return ptr::null_mut();
    };
    CString::new(e.get_processed_str_cow(OutputOptions::from_bits_truncate(options)).as_ref())
        .unwrap_or_default()
        .into_raw()
}

/// Returns non-zero when `key` can affect the active composition.
///
/// # Safety
/// `engine` must be a valid engine pointer when non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_can_process_key(
    engine: *const BambooEngine,
    key: u32,
) -> i32 {
    // SAFETY: the caller supplies a live engine pointer when non-null.
    unsafe { engine.as_ref() }.zip(char::from_u32(key)).is_some_and(|(e, c)| e.can_process_key(c))
        as i32
}

/// Returns non-zero when the active composition is a complete valid word.
///
/// # Safety
/// `engine` must be a valid engine pointer when non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_is_valid_word(engine: *const BambooEngine) -> i32 {
    // SAFETY: the caller supplies a live engine pointer when non-null.
    unsafe { engine.as_ref() }.is_some_and(Engine::is_valid_word) as i32
}

/// Returns non-zero when the active composition is a valid incomplete prefix.
///
/// # Safety
/// `engine` must be a valid engine pointer when non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_is_valid_prefix(engine: *const BambooEngine) -> i32 {
    // SAFETY: the caller supplies a live engine pointer when non-null.
    unsafe { engine.as_ref() }.is_some_and(Engine::is_valid_prefix) as i32
}

/// Undoes one physical keypress. `tone_refresh` is 0 to recompute tone
/// placement and 1 to preserve the prior snapshot's placement.
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_undo_last_key(engine: *mut BambooEngine, tone_refresh: i32) {
    // SAFETY: the caller supplies a live exclusive engine pointer.
    if let Some(e) = unsafe { engine.as_mut() } {
        e.undo_last_key_with(tone_refresh_from_i32(tone_refresh));
    }
}

/// Restores the active last word. `form` is 0 for Vietnamese processing and
/// 1 for raw keystrokes.
///
/// # Safety
/// `engine` must be a valid, non-null engine pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_restore_last_word(engine: *mut BambooEngine, form: i32) {
    // SAFETY: the caller supplies a live exclusive engine pointer.
    if let Some(e) = unsafe { engine.as_mut() } {
        e.restore_last_word_as(last_word_form_from_i32(form));
    }
}

/// Removes the last output character (grapheme) from the active composition
/// using a specific engine instance, keeping mark/tone transformations on
/// earlier characters intact.
///
/// # Safety
/// - `engine` must be a valid, non-null pointer to a `BambooEngine` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_remove_last_output_char(engine: *mut BambooEngine) {
    // SAFETY: engine pointer checked for non-null and safely converted to a mutable reference.
    let Some(e) = (unsafe { engine.as_mut() }) else {
        return;
    };
    e.remove_last_output_char();
}

/// Resets a specific engine instance and loads `text` as if it had been typed
/// (see [`Engine::rebuild_from_text`]). A null `text` only resets the engine.
///
/// # Safety
/// - `engine` must be a valid, non-null pointer to a `BambooEngine` instance.
/// - `text` must be null or a valid NUL-terminated string; invalid UTF-8 is replaced.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_rebuild_from_text(
    engine: *mut BambooEngine,
    text: *const c_char,
) {
    // SAFETY: engine pointer checked for non-null and safely converted to a mutable reference.
    let Some(e) = (unsafe { engine.as_mut() }) else {
        return;
    };
    if text.is_null() {
        e.reset();
        return;
    }
    // SAFETY: text is non-null and the caller guarantees it is NUL-terminated.
    let text = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    e.rebuild_from_text(&text);
}

/// Instance-based variant of [`bamboo_process_key_buf`].
///
/// # Safety
/// - `engine` must be a valid, non-null pointer to a `BambooEngine` instance.
/// - `out_buf` must be a valid pointer to a buffer of at least `out_cap` bytes if `out_cap > 0`.
/// - `out_len`, `backspaces_chars`, and `backspaces_bytes` must be valid, non-null pointers to `usize`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_process_key_buf(
    engine: *mut BambooEngine,
    key: u32,
    is_vietnamese: i32,
    out_buf: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
    backspaces_chars: *mut usize,
    backspaces_bytes: *mut usize,
) -> i32 {
    // SAFETY: engine pointer checked for non-null and safely converted to a mutable reference.
    let Some(e) = (unsafe { engine.as_mut() }) else {
        return -2;
    };
    if out_len.is_null() || backspaces_chars.is_null() || backspaces_bytes.is_null() {
        return -1;
    }

    // SAFETY: pointers were verified non-null and caller guarantees proper alignment and lifetime.
    let (out_len, backspaces_chars, backspaces_bytes) =
        unsafe { (&mut *out_len, &mut *backspaces_chars, &mut *backspaces_bytes) };

    let mode = if is_vietnamese != 0 { Mode::Vietnamese } else { Mode::English };

    let Some(c) = std::char::from_u32(key) else {
        *out_len = 0;
        *backspaces_chars = 0;
        *backspaces_bytes = 0;
        return 0;
    };

    let edit = e.process_key_edit(c, mode);
    let (bs_chars, bs_bytes, inserted) = (edit.delete_chars, edit.delete_bytes, edit.insert);
    let bytes = inserted.as_bytes();

    *out_len = bytes.len();
    *backspaces_chars = bs_chars;
    *backspaces_bytes = bs_bytes;

    if bytes.len() > out_cap {
        return 1;
    }
    if !bytes.is_empty() {
        if out_buf.is_null() {
            return -1;
        }
        // SAFETY: caller guarantees out_buf points to at least out_cap bytes and bytes.len() <= out_cap.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf, bytes.len());
        }
    }

    0
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_bamboo_method_roundtrip() {
        let methods = [
            (BambooMethod::Telex, 0),
            (BambooMethod::Vni, 1),
            (BambooMethod::Viqr, 2),
            (BambooMethod::MicrosoftLayout, 3),
            (BambooMethod::Telex2, 4),
            (BambooMethod::TelexW, 5),
            (BambooMethod::TelexVni, 6),
            (BambooMethod::TelexVniViqr, 7),
            (BambooMethod::VniFrenchLayout, 8),
        ];

        for (method, id) in methods {
            assert_eq!(BambooMethod::from_i32(id), method);
            let im = method.to_input_method();
            assert!(!im.rules.is_empty());
        }
    }

    #[test]
    fn test_ffi_engine_lifecycle_and_process() {
        let engine = bamboo_engine_new(0); // Telex
        assert!(!engine.is_null());

        // SAFETY: `engine` was verified non-null and valid.
        unsafe {
            // Type "tieengs" -> "tiếng"
            for ch in "tieengs".chars() {
                let res_ptr = bamboo_engine_process(engine, ch as u32);
                assert!(!res_ptr.is_null());
                bamboo_free_string(res_ptr);
            }

            // Remove last output char
            bamboo_engine_remove_last_output_char(engine);

            // Null engine safety check (should not crash)
            assert!(bamboo_engine_process(ptr::null_mut(), 'a' as u32).is_null());
            bamboo_engine_remove_last_output_char(ptr::null_mut());

            bamboo_engine_free(engine);
            bamboo_engine_free(ptr::null_mut()); // Freeing null should be a no-op
        }
    }

    #[test]
    fn test_ffi_engine_new_with_flags() {
        let typed = |flags: u32, keys: &str| {
            let engine = bamboo_engine_new_with_flags(0, flags);
            assert!(!engine.is_null());
            let mut last = String::new();
            // SAFETY: `engine` is non-null and freed once below.
            unsafe {
                for ch in keys.chars() {
                    let res_ptr = bamboo_engine_process(engine, ch as u32);
                    last = CStr::from_ptr(res_ptr).to_string_lossy().into_owned();
                    bamboo_free_string(res_ptr);
                }
                bamboo_engine_free(engine);
            }
            last
        };
        assert_eq!(typed(0x07, "nhw"), "nhw");
        assert_eq!(typed(0x07 | 0x08, "nhw"), "như");
        // A bracket the engine does not process ends the word.
        assert_eq!(typed(0x07, "m[f"), "f");
        assert_eq!(typed(0x07 | 0x20, "m[f"), "mờ");
    }

    #[test]
    fn test_ffi_rebuild_from_text() {
        let engine = bamboo_engine_new(0); // Telex
        let text = CString::new("xin tiếng").unwrap();

        // SAFETY: `engine` and `text` are valid for the duration of the calls.
        unsafe {
            bamboo_engine_rebuild_from_text(engine, text.as_ptr());
            let res_ptr = bamboo_engine_process(engine, 'f' as u32);
            assert_eq!(CStr::from_ptr(res_ptr).to_str().unwrap(), "tiềng");
            bamboo_free_string(res_ptr);

            bamboo_engine_rebuild_from_text(engine, ptr::null());
            assert_eq!((*engine).output(), "");
            bamboo_engine_rebuild_from_text(ptr::null_mut(), text.as_ptr());

            bamboo_engine_free(engine);
        }
    }

    #[test]
    fn test_ffi_process_key_buf() {
        let engine = bamboo_engine_new(0); // Telex
        assert!(!engine.is_null());

        let mut buf = [0u8; 64];
        let mut out_len = 0usize;
        let mut bs_chars = 0usize;
        let mut bs_bytes = 0usize;

        // SAFETY: `engine` and buffers are valid stack/allocated memory.
        unsafe {
            let ret = bamboo_engine_process_key_buf(
                engine,
                'a' as u32,
                1,
                buf.as_mut_ptr(),
                buf.len(),
                &mut out_len,
                &mut bs_chars,
                &mut bs_bytes,
            );
            assert_eq!(ret, 0);
            assert_eq!(out_len, 1);
            assert_eq!(buf[0], b'a');

            // Null checks
            assert_eq!(
                bamboo_engine_process_key_buf(
                    ptr::null_mut(),
                    'a' as u32,
                    1,
                    buf.as_mut_ptr(),
                    buf.len(),
                    &mut out_len,
                    &mut bs_chars,
                    &mut bs_bytes,
                ),
                -2
            );

            bamboo_engine_free(engine);
        }
    }

    #[test]
    fn test_ffi_current_engine_api() {
        let engine = bamboo_engine_new_with_flags(0, 0x07);
        assert!(!engine.is_null());

        // SAFETY: the engine is valid until the final free, and strings are
        // released through the matching FFI function.
        unsafe {
            assert_eq!(bamboo_engine_config_flags(engine), 0x07);
            assert_eq!(bamboo_engine_can_process_key(engine, 's' as u32), 1);
            assert_eq!(bamboo_engine_can_process_key(engine, '!' as u32), 0);

            for c in "tieengs".chars() {
                let text = bamboo_engine_process_key_mode(engine, c as u32, 1);
                assert!(!text.is_null());
                bamboo_free_string(text);
            }
            assert_eq!(bamboo_engine_is_valid_word(engine), 1);
            let output = bamboo_engine_output(engine, OutputOptions::NONE.bits());
            assert_eq!(CStr::from_ptr(output).to_str().unwrap(), "tiếng");
            bamboo_free_string(output);

            bamboo_engine_undo_last_key(engine, 0);
            let output = bamboo_engine_output(engine, OutputOptions::NONE.bits());
            assert_eq!(CStr::from_ptr(output).to_str().unwrap(), "tiêng");
            bamboo_free_string(output);

            bamboo_engine_commit(engine);
            let output = bamboo_engine_output(engine, OutputOptions::FULL_TEXT.bits());
            assert_eq!(CStr::from_ptr(output).to_str().unwrap(), "tiêng");
            bamboo_free_string(output);

            bamboo_engine_reset(engine);
            assert_eq!(bamboo_engine_is_valid_prefix(engine), 1);
            bamboo_engine_set_config_flags(engine, 0x07 | 0x08);
            assert_eq!(bamboo_engine_config_flags(engine), 0x0F);
            bamboo_engine_set_input_method(engine, BambooMethod::Vni as i32);
            let text = bamboo_engine_process_key_mode(engine, 'a' as u32, 0);
            assert_eq!(CStr::from_ptr(text).to_str().unwrap(), "a");
            bamboo_free_string(text);
            bamboo_engine_free(engine);
        }
    }

    #[test]
    fn test_ffi_global_engine() {
        bamboo_setup();
        bamboo_reset();

        let s1 = bamboo_process_key('a' as u32, 1);
        let s2 = bamboo_process_key('s' as u32, 1);

        // SAFETY: `s2` is a valid C-string returned by `bamboo_process_key`.
        unsafe {
            let cstr = CStr::from_ptr(s2);
            assert_eq!(cstr.to_str().unwrap(), "á");
            bamboo_free_string(s1);
            bamboo_free_string(s2);
        }

        let out = bamboo_output();
        // SAFETY: `out` is a valid C-string returned by `bamboo_output`.
        unsafe {
            let cstr = CStr::from_ptr(out);
            assert_eq!(cstr.to_str().unwrap(), "á");
            bamboo_free_string(out);
        }

        bamboo_remove_last_char();
        bamboo_reset();
    }
}
