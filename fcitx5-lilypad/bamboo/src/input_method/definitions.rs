use phf::{Map, phf_map};

/// Maps a key to its corresponding Vietnamese input transformation name.
pub type InputMethodDef = Map<&'static str, &'static str>;

static TELEX: InputMethodDef = phf_map! {
    "z" => "XoaDauThanh",
    "s" => "DauSac",
    "f" => "DauHuyen",
    "r" => "DauHoi",
    "x" => "DauNga",
    "j" => "DauNang",
    "a" => "A_Â",
    "e" => "E_Ê",
    "o" => "O_Ô",
    "w" => "UOAÔ_ƯƠĂƠ",
    "d" => "D_Đ",
};

static VNI: InputMethodDef = phf_map! {
    "0" => "XoaDauThanh",
    "1" => "DauSac",
    "2" => "DauHuyen",
    "3" => "DauHoi",
    "4" => "DauNga",
    "5" => "DauNang",
    "6" => "AEO_ÂÊÔ",
    "7" => "UO_ƯƠ",
    "8" => "A_Ă",
    "9" => "D_Đ",
};

static VIQR: InputMethodDef = phf_map! {
    "0" => "XoaDauThanh",
    "'" => "DauSac",
    "`" => "DauHuyen",
    "?" => "DauHoi",
    "~" => "DauNga",
    "." => "DauNang",
    "^" => "AEO_ÂÊÔ",
    "+" => "UO_ƯƠ",
    "*" => "UO_ƯƠ",
    "(" => "A_Ă",
    "d" => "D_Đ",
};

static MICROSOFT_LAYOUT: InputMethodDef = phf_map! {
    "8" => "DauSac",
    "5" => "DauHuyen",
    "6" => "DauHoi",
    "7" => "DauNga",
    "9" => "DauNang",
    "1" => "__ă",
    "!" => "_Ă",
    "2" => "__â",
    "@" => "_Â",
    "3" => "__ê",
    "#" => "_Ê",
    "4" => "__ô",
    "$" => "_Ô",
    "0" => "__đ",
    ")" => "_Đ",
    "[" => "__ư",
    "{" => "_Ư",
    "]" => "__ơ",
    "}" => "_Ơ",
};

static TELEX_2: InputMethodDef = phf_map! {
    "z" => "XoaDauThanh",
    "s" => "DauSac",
    "f" => "DauHuyen",
    "r" => "DauHoi",
    "x" => "DauNga",
    "j" => "DauNang",
    "a" => "A_Â",
    "e" => "E_Ê",
    "o" => "O_Ô",
    "w" => "UOAÔ_ƯƠĂƠ__Ư",
    "d" => "D_Đ",
    "]" => "__ư",
    "[" => "__ơ",
    "}" => "_Ư",
    "{" => "_Ơ",
};

static TELEX_VNI: InputMethodDef = phf_map! {
    "z" => "XoaDauThanh",
    "s" => "DauSac",
    "f" => "DauHuyen",
    "r" => "DauHoi",
    "x" => "DauNga",
    "j" => "DauNang",
    "a" => "A_Â",
    "e" => "E_Ê",
    "o" => "O_Ô",
    "w" => "UOAÔ_ƯƠĂƠ",
    "d" => "D_Đ",
    "0" => "XoaDauThanh",
    "1" => "DauSac",
    "2" => "DauHuyen",
    "3" => "DauHoi",
    "4" => "DauNga",
    "5" => "DauNang",
    "6" => "AEO_ÂÊÔ",
    "7" => "UO_ƯƠ",
    "8" => "A_Ă",
    "9" => "D_Đ",
};

static TELEX_VNI_VIQR: InputMethodDef = phf_map! {
    "z" => "XoaDauThanh",
    "s" => "DauSac",
    "f" => "DauHuyen",
    "r" => "DauHoi",
    "x" => "DauNga",
    "j" => "DauNang",
    "a" => "A_Â",
    "e" => "E_Ê",
    "o" => "O_Ô",
    "w" => "UOAÔ_ƯƠĂƠ",
    "d" => "D_Đ",
    "0" => "XoaDauThanh",
    "1" => "DauSac",
    "2" => "DauHuyen",
    "3" => "DauHoi",
    "4" => "DauNga",
    "5" => "DauNang",
    "6" => "AEO_ÂÊÔ",
    "7" => "UO_ƯƠ",
    "8" => "A_Ă",
    "9" => "D_Đ",
    "'" => "DauSac",
    "`" => "DauHuyen",
    "?" => "DauHoi",
    "~" => "DauNga",
    "." => "DauNang",
    "^" => "AEO_ÂÊÔ",
    "+" => "UO_ƯƠ",
    "*" => "UO_ƯƠ",
    "(" => "A_Ă",
    "\\" => "D_Đ",
};

static VNI_FRENCH_LAYOUT: InputMethodDef = phf_map! {
    "&" => "XoaDauThanh",
    "é" => "DauSac",
    "\"" => "DauHuyen",
    "'" => "DauHoi",
    "(" => "DauNga",
    "-" => "DauNang",
    "è" => "AEO_ÂÊÔ",
    "_" => "UO_ƯƠ",
    "ç" => "A_Ă",
    "à" => "D_Đ",
};

static TELEX_W: InputMethodDef = phf_map! {
    "z" => "XoaDauThanh",
    "s" => "DauSac",
    "f" => "DauHuyen",
    "r" => "DauHoi",
    "x" => "DauNga",
    "j" => "DauNang",
    "a" => "A_Â",
    "e" => "E_Ê",
    "o" => "O_Ô",
    "w" => "UOAÔ_ƯƠĂƠ__Ư",
    "d" => "D_Đ",
};

static INPUT_METHOD_DEFS: Map<&'static str, &'static InputMethodDef> = phf_map! {
    "Telex" => &TELEX,
    "VNI" => &VNI,
    "VIQR" => &VIQR,
    "Microsoft layout" => &MICROSOFT_LAYOUT,
    "Telex 2" => &TELEX_2,
    "Telex + VNI" => &TELEX_VNI,
    "Telex + VNI + VIQR" => &TELEX_VNI_VIQR,
    "VNI Bàn phím tiếng Pháp" => &VNI_FRENCH_LAYOUT,
    "Telex W" => &TELEX_W,
};

/// Retrieves an input method definition by its name.
pub fn get_input_method(name: &str) -> Option<&'static InputMethodDef> {
    INPUT_METHOD_DEFS.get(name).copied()
}

/// Returns all available input method definitions.
pub fn get_input_method_definitions() -> &'static Map<&'static str, &'static InputMethodDef> {
    &INPUT_METHOD_DEFS
}
