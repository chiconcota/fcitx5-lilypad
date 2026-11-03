//! Fcitx5-Lilypad C-ABI compatibility layer for bamboo-core.
//!
//! Replaces the old Go cgo layer (fcitxbambooengine.go & bamboo-c.go)
//! with a pure Rust C-ABI implementation.

#![allow(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::ffi::{c_char, CStr, CString};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use crate::engine::{Engine, LastWordForm, ToneRefresh};
use crate::input_method::InputMethod;
use crate::mode::{Mode, OutputOptions};
use crate::orthography::phonetics::{is_vietnamese_rune, is_word_break_symbol};

// --- Fcitx KeySym & Modifier Constants ---
pub const FCITX_SHIFT_MASK: u32 = 1 << 0;
pub const FCITX_LOCK_MASK: u32 = 1 << 1;
pub const FCITX_CONTROL_MASK: u32 = 1 << 2;
pub const FCITX_MOD1_MASK: u32 = 1 << 3;
pub const FCITX_FORWARD_MASK: u32 = 1 << 25;
pub const FCITX_SUPER_MASK: u32 = 1 << 26;
pub const FCITX_HYPER_MASK: u32 = 1 << 27;
pub const FCITX_META_MASK: u32 = 1 << 28;

pub const FCITX_BACKSPACE: u32 = 0xff08;
pub const FCITX_SPACE: u32 = 0x0020;
pub const FCITX_TAB: u32 = 0xff09;

/// C-compatible FcitxBambooEngineOption matching the Fcitx5 struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FcitxBambooEngineOption {
    pub auto_non_vn_restore: bool,
    pub dd_free_style: bool,
    pub macro_enabled: bool,
    pub auto_capitalize_macro: bool,
    pub spell_check_with_dicts: bool,
    pub output_charset: *const c_char,
    pub modern_style: bool,
    pub free_marking: bool,
    pub w2u: i32,
    pub bracket_transform: i32,
    pub time_format: *const c_char,
    pub date_format: *const c_char,
}

/// Instance object stored in the global handle table
enum FcitxObject {
    Engine(Box<FcitxBambooEngine>),
    Macro(Arc<HashMap<String, String>>),
    Dict(Arc<HashSet<String>>),
}

static OBJECTS: LazyLock<Mutex<HashMap<usize, FcitxObject>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

fn alloc_object(obj: FcitxObject) -> usize {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let mut map = OBJECTS.lock().unwrap();
    map.insert(id, obj);
    id
}

fn get_dict(handle: usize) -> Option<Arc<HashSet<String>>> {
    if handle == 0 {
        return None;
    }
    let map = OBJECTS.lock().unwrap();
    if let Some(FcitxObject::Dict(d)) = map.get(&handle) {
        Some(Arc::clone(d))
    } else {
        None
    }
}

fn get_macro(handle: usize) -> Option<Arc<HashMap<String, String>>> {
    if handle == 0 {
        return None;
    }
    let map = OBJECTS.lock().unwrap();
    if let Some(FcitxObject::Macro(m)) = map.get(&handle) {
        Some(Arc::clone(m))
    } else {
        None
    }
}

fn with_engine_mut<F, R>(handle: usize, f: F) -> Option<R>
where
    F: FnOnce(&mut FcitxBambooEngine) -> R,
{
    let mut map = OBJECTS.lock().unwrap();
    if let Some(FcitxObject::Engine(e)) = map.get_mut(&handle) {
        Some(f(e))
    } else {
        None
    }
}

/// Parse input method name string to crate::InputMethod
fn parse_input_method(name: &str) -> InputMethod {
    match name {
        "VNI" => InputMethod::vni(),
        "VIQR" => InputMethod::viqr(),
        "Microsoft layout" => InputMethod::microsoft_layout(),
        "Telex 2" => InputMethod::telex_2(),
        "Telex W" => InputMethod::telex_w(),
        "Telex + VNI" => InputMethod::telex_vni(),
        "Telex + VNI + VIQR" => InputMethod::telex_vni_viqr(),
        "VNI Bàn phím tiếng Pháp" => InputMethod::vni_french_layout(),
        _ => InputMethod::telex(),
    }
}

/// Vietnamese Input Engine wrapped for Fcitx5
pub struct FcitxBambooEngine {
    engine: Engine,
    macro_table: Option<Arc<HashMap<String, String>>>,
    dictionary: Option<Arc<HashSet<String>>>,
    auto_non_vn_restore: bool,
    dd_free_style: bool,
    macro_enabled: bool,
    auto_capitalize_macro: bool,
    last_key_with_shift: bool,
    spell_check_with_dicts: bool,
    preedit_text: String,
    commit_text: String,
    pub should_restore_keystrokes: bool,
    output_charset: String,
    time_format: String,
    date_format: String,
}

impl FcitxBambooEngine {
    pub fn new(
        im_name: &str,
        dict: Option<Arc<HashSet<String>>>,
        macro_table: Option<Arc<HashMap<String, String>>>,
    ) -> Self {
        let im = parse_input_method(im_name);
        Self::new_with_im(im, dict, macro_table)
    }

    pub fn new_with_im(
        im: InputMethod,
        dict: Option<Arc<HashSet<String>>>,
        macro_table: Option<Arc<HashMap<String, String>>>,
    ) -> Self {
        Self {
            engine: Engine::new(im),
            macro_table,
            dictionary: dict,
            auto_non_vn_restore: true,
            dd_free_style: true,
            macro_enabled: false,
            auto_capitalize_macro: false,
            last_key_with_shift: false,
            spell_check_with_dicts: true,
            preedit_text: String::new(),
            commit_text: String::new(),
            should_restore_keystrokes: false,
            output_charset: "Unicode".to_string(),
            time_format: "%H:%M".to_string(),
            date_format: "%d/%m/%Y".to_string(),
        }
    }

    pub fn set_option(&mut self, opt: &FcitxBambooEngineOption) {
        self.auto_non_vn_restore = opt.auto_non_vn_restore;
        self.dd_free_style = opt.dd_free_style;
        self.macro_enabled = opt.macro_enabled;
        self.auto_capitalize_macro = opt.auto_capitalize_macro;
        self.spell_check_with_dicts = opt.spell_check_with_dicts;

        if !opt.output_charset.is_null() {
            self.output_charset = unsafe { CStr::from_ptr(opt.output_charset) }
                .to_string_lossy()
                .to_string();
        }
        if !opt.time_format.is_null() {
            self.time_format = unsafe { CStr::from_ptr(opt.time_format) }
                .to_string_lossy()
                .to_string();
        }
        if !opt.date_format.is_null() {
            self.date_format = unsafe { CStr::from_ptr(opt.date_format) }
                .to_string_lossy()
                .to_string();
        }

        let mut flags = 0u32;
        if opt.free_marking {
            flags |= 1 << 0; // BAMBOO_CONFIG_FREE_TONE_MARKING
        }
        if opt.modern_style {
            flags |= 1 << 1; // BAMBOO_CONFIG_MODERN_TONE_STYLE
        }
        if opt.auto_non_vn_restore {
            flags |= 1 << 2; // BAMBOO_CONFIG_AUTO_CORRECT
        }
        if opt.w2u == 1 {
            flags |= 1 << 3; // BAMBOO_CONFIG_W2U_NON_START
        } else if opt.w2u == 2 {
            flags |= 1 << 4; // BAMBOO_CONFIG_W2U_EVERYWHERE
        }
        if opt.bracket_transform == 1 {
            flags |= 1 << 5; // BAMBOO_CONFIG_BRACKET_NON_START
        } else if opt.bracket_transform == 2 {
            flags |= 1 << 6; // BAMBOO_CONFIG_BRACKET_EVERYWHERE
        }
        self.engine.set_config(crate::Config::from_flags(flags));
    }

    pub fn reset(&mut self) {
        self.engine.reset();
        self.preedit_text.clear();
        self.commit_text.clear();
    }

    pub fn rebuild_from_text(&mut self, text: &str) {
        self.engine.reset();
        self.engine.rebuild_from_text(text);
        let s = self.get_preedit_string();
        self.update_preedit(&s);
    }

    pub fn commit_preedit_and_reset(&mut self, s: String) {
        self.commit_text = s;
        self.preedit_text.clear();
        self.engine.reset();
    }

    fn encode_text(&self, text: &str) -> String {
        crate::encoder::encode(&self.output_charset, text)
    }

    fn update_preedit(&mut self, processed_str: &str) {
        let encoded = self.encode_text(processed_str);
        if encoded.is_empty() {
            self.preedit_text.clear();
            self.commit_text.clear();
        } else {
            self.preedit_text = encoded;
        }
    }

    pub fn pull_commit(&mut self) -> String {
        let text = std::mem::take(&mut self.commit_text);
        self.encode_text(&text)
    }

    pub fn pull_preedit(&self) -> String {
        self.preedit_text.clone()
    }

    pub fn commit_preedit(&mut self) {
        let pre = self.get_preedit_string();
        self.commit_preedit_and_reset(pre);
    }

    fn get_processed_string(&self, options: OutputOptions) -> String {
        self.engine.get_processed_str(options)
    }

    fn should_fallback_to_english(&self, check_vn_rune: bool) -> bool {
        if !self.auto_non_vn_restore {
            return false;
        }
        let vn_seq = self.get_processed_string(OutputOptions::LOWER_CASE);
        if vn_seq.is_empty() {
            return false;
        }
        if self.get_macro_text().is_some() {
            return false;
        }
        if self.dd_free_style && vn_seq.contains('đ') {
            return false;
        }
        if check_vn_rune && !vn_seq.chars().any(is_vietnamese_rune) {
            return false;
        }
        !self.engine.is_valid_prefix()
    }

    fn must_fallback_to_english(&self) -> bool {
        if !self.auto_non_vn_restore {
            return false;
        }
        let vn_seq = self.get_processed_string(OutputOptions::LOWER_CASE);
        if vn_seq.is_empty() {
            return false;
        }
        if self.dd_free_style && vn_seq.contains('đ') {
            return false;
        }
        if self.spell_check_with_dicts {
            if let Some(ref dict) = self.dictionary {
                if !dict.is_empty() {
                    return !dict.contains(&vn_seq);
                }
            }
        }
        !self.engine.is_valid_word()
    }

    fn get_preedit_string(&self) -> String {
        if self.macro_enabled {
            if self.auto_non_vn_restore && self.should_fallback_to_english(true) {
                return self.get_processed_string(OutputOptions::RAW);
            }
            return self.get_processed_string(OutputOptions::PUNCTUATION_MODE);
        }
        if self.should_fallback_to_english(true) {
            return self.get_processed_string(OutputOptions::RAW);
        }
        self.engine.output_str().to_string()
    }

    fn get_macro_text(&self) -> Option<String> {
        if !self.macro_enabled {
            return None;
        }
        let table = self.macro_table.as_ref()?;
        let text = self.get_processed_string(OutputOptions::PUNCTUATION_MODE);
        table.get(&text).map(|v| self.expand_macro(&text, v))
    }

    fn expand_macro(&self, raw: &str, replacement: &str) -> String {
        let mut result = replacement.to_string();
        // Time / Date expansions can be added if needed
        if self.auto_capitalize_macro {
            if let Some(first) = raw.chars().next() {
                if first.is_lowercase() {
                    result = result.to_lowercase();
                } else if raw.chars().all(|c| c.is_uppercase()) {
                    result = result.to_uppercase();
                }
            }
        }
        result
    }

    fn is_valid_state(&self, state: u32) -> bool {
        (state
            & (FCITX_CONTROL_MASK
                | FCITX_MOD1_MASK
                | FCITX_FORWARD_MASK
                | FCITX_SUPER_MASK
                | FCITX_HYPER_MASK
                | FCITX_META_MASK))
            == 0
    }

    fn can_process_key(&self, key_val: u32) -> bool {
        if key_val == FCITX_SPACE || key_val == FCITX_BACKSPACE {
            return true;
        }
        if let Some(c) = char::from_u32(key_val) {
            if is_word_break_symbol(c) {
                return true;
            }
            if self.get_macro_text().is_some() && key_val == FCITX_TAB {
                return true;
            }
            self.engine.can_process_key(c)
        } else {
            false
        }
    }

    pub fn process_key_event(&mut self, key_val: u32, state: u32) -> bool {
        let raw_len = self.engine.active_len();
        let c = match char::from_u32(key_val) {
            Some(c) => c,
            None => return false,
        };
        let old_text = self.get_preedit_string();
        self.last_key_with_shift = (state & FCITX_SHIFT_MASK) != 0;

        if !self.should_restore_keystrokes {
            if !self.is_valid_state(state)
                || !self.can_process_key(key_val)
                || (!self.macro_enabled && raw_len == 0 && !self.engine.can_process_key(c))
            {
                if raw_len > 0 {
                    let pre = self.get_preedit_string();
                    self.commit_preedit_and_reset(pre);
                }
                return false;
            }
        }

        // Backspace
        if key_val == FCITX_BACKSPACE {
            if self.preedit_text.chars().count() <= 1 {
                self.commit_preedit_and_reset(String::new());
                return true;
            }
            if raw_len > 0 {
                self.engine.undo_last_key_with(ToneRefresh::Recompute);
                let pre = self.get_preedit_string();
                self.update_preedit(&pre);
                return true;
            } else {
                return false;
            }
        }

        // Tab (Macro expansion)
        if key_val == FCITX_TAB {
            if let Some(macro_text) = self.get_macro_text() {
                self.commit_preedit_and_reset(macro_text);
            } else {
                let comp = if old_text.chars().any(is_vietnamese_rune)
                    && self.must_fallback_to_english()
                {
                    self.get_processed_string(OutputOptions::RAW)
                } else {
                    old_text
                };
                self.commit_preedit_and_reset(comp);
                return false;
            }
            return true;
        }

        // Shift + Space restore
        if self.should_restore_keystrokes {
            self.should_restore_keystrokes = false;
            let has_vn = old_text.chars().any(is_vietnamese_rune);
            self.engine.restore_last_word_as(if has_vn {
                LastWordForm::Vietnamese
            } else {
                LastWordForm::RawKeystrokes
            });
            let pre = self.get_preedit_string();
            self.update_preedit(&pre);
            return false;
        }

        // Standard typing
        if self.engine.can_process_key(c) {
            let processed_c = if (state & FCITX_LOCK_MASK) != 0 && (state & FCITX_SHIFT_MASK) == 0 {
                c.to_uppercase().next().unwrap_or(c)
            } else {
                c
            };

            let mode = if self.should_fallback_to_english(false) {
                Mode::English
            } else {
                Mode::Vietnamese
            };

            self.engine.process_key(processed_c, mode);
            let new_text = self.get_preedit_string();
            self.update_preedit(&new_text);
            return true;
        } else if is_word_break_symbol(c) {
            // Word break commit
            if self.macro_enabled {
                if let Some(table) = self.macro_table.as_ref() {
                    if let Some(mac) = table.get(&old_text) {
                        let expanded = self.expand_macro(&old_text, mac);
                        let commit_str = format!("{}{}", expanded, c);
                        self.commit_preedit_and_reset(commit_str);
                        return true;
                    }
                }
            }

            if old_text.chars().any(is_vietnamese_rune) && self.must_fallback_to_english() {
                self.engine.restore_last_word_as(LastWordForm::RawKeystrokes);
                let fallback = self.get_processed_string(OutputOptions::RAW);
                let commit_str = format!("{}{}", fallback, c);
                self.engine.process_key(c, Mode::English);
                self.commit_preedit_and_reset(commit_str);
                return true;
            }

            self.engine.process_key(c, Mode::English);
            let commit_str = format!("{}{}", old_text, c);
            self.commit_preedit_and_reset(commit_str);
            return true;
        }

        true
    }
}

// =========================================================================
// C-ABI EXPORTS (Matching bamboo-core.h and fcitx5-lilypad C++ requirements)
// =========================================================================

fn to_c_string_array(strs: &[&str]) -> *mut *mut c_char {
    unsafe {
        let layout = std::alloc::Layout::array::<*mut c_char>(strs.len() + 1).unwrap();
        let ptr = libc::malloc(layout.size()) as *mut *mut c_char;
        for (i, &s) in strs.iter().enumerate() {
            let cs = CString::new(s).unwrap();
            *ptr.add(i) = libc::strdup(cs.as_ptr());
        }
        *ptr.add(strs.len()) = std::ptr::null_mut();
        ptr
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Init() {
    // Initialization hook
}

#[unsafe(no_mangle)]
pub extern "C" fn GetInputMethodNames() -> *mut *mut c_char {
    let names = [
        "Telex",
        "VNI",
        "Telex + VNI",
        "Telex + VNI + VIQR",
        "VIQR",
        "Microsoft layout",
        "Telex 2",
        "Telex W",
        "VNI Bàn phím tiếng Pháp",
    ];
    to_c_string_array(&names)
}

#[unsafe(no_mangle)]
pub extern "C" fn GetCharsetNames() -> *mut *mut c_char {
    let names = [
        "Unicode",
        "TCVN3",
        "VNI Windows",
        "VIQR",
        "Unicode C string Hex",
    ];
    to_c_string_array(&names)
}

#[unsafe(no_mangle)]
pub extern "C" fn NewDictionary(fd: usize) -> usize {
    use std::io::{BufRead, BufReader};
    use std::os::unix::io::FromRawFd;

    let mut dict = HashSet::new();
    if fd > 0 {
        let file = unsafe { std::fs::File::from_raw_fd(fd as i32) };
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line.trim().to_lowercase();
            if !trimmed.is_empty() {
                dict.insert(trimmed);
            }
        }
    }
    alloc_object(FcitxObject::Dict(Arc::new(dict)))
}

#[unsafe(no_mangle)]
pub extern "C" fn NewMacroTable(definition: *mut *mut c_char) -> usize {
    let mut table = HashMap::new();
    if !definition.is_null() {
        let mut i = 0;
        unsafe {
            while !(*definition.add(i)).is_null() {
                let k_ptr = *definition.add(i);
                let v_ptr = *definition.add(i + 1);
                if v_ptr.is_null() {
                    break;
                }
                let k = CStr::from_ptr(k_ptr).to_string_lossy().to_string();
                let v = CStr::from_ptr(v_ptr).to_string_lossy().to_string();
                table.insert(k, v);
                i += 2;
            }
        }
    }
    alloc_object(FcitxObject::Macro(Arc::new(table)))
}

#[unsafe(no_mangle)]
pub extern "C" fn NewEngine(
    name: *const c_char,
    dict_handle: usize,
    table_handle: usize,
) -> usize {
    let im_name = if name.is_null() {
        "Telex"
    } else {
        unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("Telex")
    };
    let dict = get_dict(dict_handle);
    let macro_table = get_macro(table_handle);
    let engine = FcitxBambooEngine::new(im_name, dict, macro_table);
    alloc_object(FcitxObject::Engine(Box::new(engine)))
}

#[unsafe(no_mangle)]
pub extern "C" fn NewCustomEngine(
    definition: *mut *mut c_char,
    dict_handle: usize,
    table_handle: usize,
) -> usize {
    let mut entries = Vec::new();
    if !definition.is_null() {
        let mut i = 0;
        unsafe {
            while !(*definition.add(i)).is_null() {
                let k_ptr = *definition.add(i);
                let v_ptr = *definition.add(i + 1);
                if v_ptr.is_null() {
                    break;
                }
                let k = CStr::from_ptr(k_ptr).to_string_lossy().to_string();
                let v = CStr::from_ptr(v_ptr).to_string_lossy().to_string();
                entries.push((k, v));
                i += 2;
            }
        }
    }
    let dict = get_dict(dict_handle);
    let macro_table = get_macro(table_handle);
    let im = InputMethod::from_definition("Custom", entries);
    let engine = FcitxBambooEngine::new_with_im(im, dict, macro_table);
    alloc_object(FcitxObject::Engine(Box::new(engine)))
}

#[unsafe(no_mangle)]
pub extern "C" fn DeleteObject(handle: usize) {
    let mut map = OBJECTS.lock().unwrap();
    map.remove(&handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn ResetEngine(engine: usize) {
    with_engine_mut(engine, |e| {
        e.reset();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn EngineRebuildFromText(engine: usize, text: *const c_char) {
    if text.is_null() {
        return;
    }
    let s = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    with_engine_mut(engine, |e| {
        e.rebuild_from_text(&s);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn EnginePullCommit(engine: usize) -> *mut c_char {
    with_engine_mut(engine, |e| {
        let commit = e.pull_commit();
        if commit.is_empty() {
            std::ptr::null_mut()
        } else {
            let cs = CString::new(commit).unwrap_or_default();
            unsafe { libc::strdup(cs.as_ptr()) }
        }
    })
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn EnginePullPreedit(engine: usize) -> *mut c_char {
    with_engine_mut(engine, |e| {
        let preedit = e.pull_preedit();
        if preedit.is_empty() {
            std::ptr::null_mut()
        } else {
            let cs = CString::new(preedit).unwrap_or_default();
            unsafe { libc::strdup(cs.as_ptr()) }
        }
    })
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn EngineCommitPreedit(engine: usize) {
    with_engine_mut(engine, |e| {
        e.commit_preedit();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn EngineSetOption(engine: usize, option: *const FcitxBambooEngineOption) {
    if option.is_null() {
        return;
    }
    let opt = unsafe { &*option };
    with_engine_mut(engine, |e| {
        e.set_option(opt);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn EngineSetRestoreKeyStroke(engine: usize) {
    with_engine_mut(engine, |e| {
        e.should_restore_keystrokes = true;
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn EngineProcessKeyEvent(engine: usize, key_val: u32, state: u32) -> u8 {
    with_engine_mut(engine, |e| {
        if e.process_key_event(key_val, state) {
            1
        } else {
            0
        }
    })
    .unwrap_or(0)
}
