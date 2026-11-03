use crate::engine::EngineRules;
use std::sync::Arc;
use std::sync::LazyLock;

pub mod preset;
pub mod rule;

pub use preset::InputMethodPreset;
pub use rule::{EffectType, Mark, Rule, Tone};

static PRESET_TELEX_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> = LazyLock::new(|| {
    let im = Arc::new(parse_input_method("Telex"));
    let rules = Arc::new(EngineRules::from_input_method(&im));
    (im, rules)
});
static PRESET_VNI_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> = LazyLock::new(|| {
    let im = Arc::new(parse_input_method("VNI"));
    let rules = Arc::new(EngineRules::from_input_method(&im));
    (im, rules)
});
static PRESET_VIQR_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> = LazyLock::new(|| {
    let im = Arc::new(parse_input_method("VIQR"));
    let rules = Arc::new(EngineRules::from_input_method(&im));
    (im, rules)
});
static PRESET_MICROSOFT_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> =
    LazyLock::new(|| {
        let im = Arc::new(parse_input_method("Microsoft layout"));
        let rules = Arc::new(EngineRules::from_input_method(&im));
        (im, rules)
    });
static PRESET_TELEX_2_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> =
    LazyLock::new(|| {
        let im = Arc::new(parse_input_method("Telex 2"));
        let rules = Arc::new(EngineRules::from_input_method(&im));
        (im, rules)
    });
static PRESET_TELEX_VNI_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> =
    LazyLock::new(|| {
        let im = Arc::new(parse_input_method("Telex + VNI"));
        let rules = Arc::new(EngineRules::from_input_method(&im));
        (im, rules)
    });
static PRESET_TELEX_VNI_VIQR_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> =
    LazyLock::new(|| {
        let im = Arc::new(parse_input_method("Telex + VNI + VIQR"));
        let rules = Arc::new(EngineRules::from_input_method(&im));
        (im, rules)
    });
static PRESET_VNI_FRENCH_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> =
    LazyLock::new(|| {
        let im = Arc::new(parse_input_method("VNI Bàn phím tiếng Pháp"));
        let rules = Arc::new(EngineRules::from_input_method(&im));
        (im, rules)
    });
static PRESET_TELEX_W_SHARED: LazyLock<(Arc<InputMethod>, Arc<EngineRules>)> =
    LazyLock::new(|| {
        let im = Arc::new(parse_input_method("Telex W"));
        let rules = Arc::new(EngineRules::from_input_method(&im));
        (im, rules)
    });

pub(crate) fn get_preset_shared(preset: InputMethodPreset) -> (Arc<InputMethod>, Arc<EngineRules>) {
    let entry = match preset {
        InputMethodPreset::Telex => &PRESET_TELEX_SHARED,
        InputMethodPreset::Vni => &PRESET_VNI_SHARED,
        InputMethodPreset::Viqr => &PRESET_VIQR_SHARED,
        InputMethodPreset::MicrosoftLayout => &PRESET_MICROSOFT_SHARED,
        InputMethodPreset::Telex2 => &PRESET_TELEX_2_SHARED,
        InputMethodPreset::TelexVni => &PRESET_TELEX_VNI_SHARED,
        InputMethodPreset::TelexVniViqr => &PRESET_TELEX_VNI_VIQR_SHARED,
        InputMethodPreset::VniFrenchLayout => &PRESET_VNI_FRENCH_SHARED,
        InputMethodPreset::TelexW => &PRESET_TELEX_W_SHARED,
    };
    (Arc::clone(&entry.0), Arc::clone(&entry.1))
}

pub(crate) fn find_preset_shared(im: &InputMethod) -> Option<(Arc<InputMethod>, Arc<EngineRules>)> {
    let entry = match im.name {
        "Telex" => &PRESET_TELEX_SHARED,
        "VNI" => &PRESET_VNI_SHARED,
        "VIQR" => &PRESET_VIQR_SHARED,
        "Microsoft layout" => &PRESET_MICROSOFT_SHARED,
        "Telex 2" => &PRESET_TELEX_2_SHARED,
        "Telex + VNI" => &PRESET_TELEX_VNI_SHARED,
        "Telex + VNI + VIQR" => &PRESET_TELEX_VNI_VIQR_SHARED,
        "VNI Bàn phím tiếng Pháp" => &PRESET_VNI_FRENCH_SHARED,
        "Telex W" => &PRESET_TELEX_W_SHARED,
        _ => return None,
    };
    // A custom definition may reuse a preset name, so match on content.
    if im.rules == entry.0.rules && im.keys == entry.0.keys {
        Some((Arc::clone(&entry.0), Arc::clone(&entry.1)))
    } else {
        None
    }
}

pub mod definitions;
use crate::phonetics::{add_mark_to_toneless_char, add_tone_to_char, is_vowel};
pub use definitions::InputMethodDef;

use rule::TONES;

/// A collection of rules defining how keys transform text.
///
/// Use the provided static methods (e.g., [`InputMethod::telex()`]) to get
/// standard Vietnamese input methods.
#[derive(Clone, Debug, Default)]
pub struct InputMethod {
    /// The name of the input method.
    pub name: &'static str,
    /// The complete list of transformation rules.
    pub rules: Vec<Rule>,
    /// Keys that can affect multiple vowels at once (e.g., 'w' in Telex).
    pub super_keys: Vec<char>,
    /// Keys that apply tone marks.
    pub tone_keys: Vec<char>,
    /// Keys that primarily append characters.
    pub appending_keys: Vec<char>,
    /// All keys that have at least one rule associated with them.
    pub keys: Vec<char>,
}

impl InputMethod {
    /// Returns the name of the input method.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the transformation rules.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Returns the super keys (keys that can affect multiple vowels).
    pub fn super_keys(&self) -> &[char] {
        &self.super_keys
    }

    /// Returns the tone mark keys.
    pub fn tone_keys(&self) -> &[char] {
        &self.tone_keys
    }

    /// Returns the appending keys.
    pub fn appending_keys(&self) -> &[char] {
        &self.appending_keys
    }

    /// Returns all keys with associated rules.
    pub fn keys(&self) -> &[char] {
        &self.keys
    }

    /// Creates an input method from a known preset.
    pub fn from_preset(preset: InputMethodPreset) -> Self {
        preset.to_input_method()
    }

    /// Standard Telex input method (lazily cached across calls).
    pub fn telex() -> Self {
        (*PRESET_TELEX_SHARED.0).clone()
    }

    /// Standard VNI input method (using number keys, lazily cached across calls).
    pub fn vni() -> Self {
        (*PRESET_VNI_SHARED.0).clone()
    }

    /// Standard VIQR input method (lazily cached across calls).
    pub fn viqr() -> Self {
        (*PRESET_VIQR_SHARED.0).clone()
    }

    /// Microsoft Standard Vietnamese keyboard layout (lazily cached across calls).
    pub fn microsoft_layout() -> Self {
        (*PRESET_MICROSOFT_SHARED.0).clone()
    }

    /// Telex variant that also supports `[` and `]` keys (lazily cached across calls).
    pub fn telex_2() -> Self {
        (*PRESET_TELEX_2_SHARED.0).clone()
    }

    /// Combined Telex and VNI (lazily cached across calls).
    pub fn telex_vni() -> Self {
        (*PRESET_TELEX_VNI_SHARED.0).clone()
    }

    /// Combined Telex, VNI, and VIQR (lazily cached across calls).
    pub fn telex_vni_viqr() -> Self {
        (*PRESET_TELEX_VNI_VIQR_SHARED.0).clone()
    }

    /// VNI for French keyboard layouts (lazily cached across calls).
    pub fn vni_french_layout() -> Self {
        (*PRESET_VNI_FRENCH_SHARED.0).clone()
    }

    /// Telex variant using `w` for marks and `z` for tone removal (lazily cached across calls).
    pub fn telex_w() -> Self {
        (*PRESET_TELEX_W_SHARED.0).clone()
    }

    /// Builds an input method from a runtime key → rule definition, in the
    /// format of the built-in presets (e.g. `("s", "DauSac")`, `("d", "D_Đ")`).
    ///
    /// Only the first character of each key is used; empty keys are skipped.
    /// Rules are stored in entry order, which can only matter when two
    /// entries share a key up to case (e.g. `w` and `W`).
    pub fn from_definition<K, V>(
        name: &'static str,
        entries: impl IntoIterator<Item = (K, V)>,
    ) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        parse_entries(name, entries)
    }
}

/// Parse a known input method by name from the built-in definitions.
pub(crate) fn parse_input_method(im_name: &'static str) -> InputMethod {
    let defs = crate::input_method_def::get_input_method_definitions();
    defs.get(im_name).copied().map(|def| parse_input_method_def(im_name, def)).unwrap_or_default()
}

/// Parses an input method definition from its structured format.
pub(crate) fn parse_input_method_def(
    im_name: &'static str,
    im_def: &InputMethodDef,
) -> InputMethod {
    parse_entries(im_name, im_def.entries())
}

fn parse_entries<K, V>(
    im_name: &'static str,
    entries: impl IntoIterator<Item = (K, V)>,
) -> InputMethod
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut im = InputMethod { name: im_name, ..Default::default() };

    for (key_str, line) in entries {
        let (key_str, line) = (key_str.as_ref(), line.as_ref());
        let Some(key) = key_str.chars().next() else {
            continue;
        };

        im.rules.extend(parse_rules(key, line));

        if contains_uo_case_insensitive(line) {
            im.super_keys.push(key);
        }
        im.keys.push(key);
    }

    for rule in &im.rules {
        if rule.effect_type == EffectType::Appending {
            im.appending_keys.push(rule.key);
        }
        if rule.effect_type == EffectType::ToneTransformation {
            im.tone_keys.push(rule.key);
        }
    }

    im
}

#[inline]
fn contains_uo_case_insensitive(s: &str) -> bool {
    let mut prev_u = false;
    for c in s.chars() {
        let lc = c.to_ascii_lowercase();
        if prev_u && lc == 'o' {
            return true;
        }
        prev_u = lc == 'u';
    }
    false
}

/// Parses rules for a given key based on its definition line.
pub(crate) fn parse_rules(key: char, line: &str) -> Vec<Rule> {
    if let Some(tone) = TONES.get(line).copied() {
        return vec![Rule {
            key,
            effect_type: EffectType::ToneTransformation,
            effect: tone as u8,
            effect_on: '\0',
            result: '\0',
            appended: ['\0'; 2],
            appended_len: 0,
        }];
    }

    parse_toneless_rules(key, line)
}

/// Parses rules that don't involve tone transformations.
pub(crate) fn parse_toneless_rules(key: char, line: &str) -> Vec<Rule> {
    let lower = line.to_lowercase();

    if let Some((effective_ons, results, rest)) = parse_dsl(&lower) {
        let mut rules = Vec::new();
        for (effective_on, result) in effective_ons.into_iter().zip(results) {
            let Some(effect) = find_mark_from_char(result) else {
                continue;
            };
            rules.extend(parse_toneless_rule(key, effective_on, result, effect));
        }

        if let Some(rule) = get_appending_rule(key, rest) {
            rules.push(rule);
        }

        return rules;
    }

    if let Some(rule) = get_appending_rule(key, line) {
        return vec![rule];
    }

    Vec::new()
}

fn parse_toneless_rule(key: char, effective_on: char, result: char, effect: Mark) -> Vec<Rule> {
    let mut rules = Vec::new();

    for chr in get_mark_family(effective_on) {
        if chr == result {
            rules.push(Rule {
                key,
                effect_type: EffectType::MarkTransformation,
                effect: 0,
                effect_on: result,
                result: effective_on,
                appended: ['\0'; 2],
                appended_len: 0,
            });
            continue;
        }

        if is_vowel(chr) {
            for tone in 0u8..=5 {
                rules.push(Rule {
                    key,
                    effect_type: EffectType::MarkTransformation,
                    effect_on: add_tone_to_char(chr, tone),
                    effect: effect as u8,
                    result: add_tone_to_char(result, tone),
                    appended: ['\0'; 2],
                    appended_len: 0,
                });
            }
        } else {
            rules.push(Rule {
                key,
                effect_type: EffectType::MarkTransformation,
                effect_on: chr,
                effect: effect as u8,
                result,
                appended: ['\0'; 2],
                appended_len: 0,
            });
        }
    }

    rules
}

/// Parse: `([a-zA-Z]+)_(\p{L}+)([_\p{L}]*)`.
fn parse_dsl(s: &str) -> Option<(Vec<char>, Vec<char>, &str)> {
    let (left, right) = s.split_once('_')?;
    if left.is_empty() || !left.chars().all(|c| c.is_alphabetic()) {
        return None;
    }

    let mut results = Vec::new();
    let mut rest_start_byte = right.len();

    for (byte_idx, ch) in right.char_indices() {
        if ch.is_alphabetic() {
            results.push(ch);
            continue;
        }
        rest_start_byte = byte_idx;
        break;
    }

    if results.is_empty() {
        return None;
    }

    let rest = &right[rest_start_byte..];
    Some((left.chars().collect(), results, rest))
}

/// Parse: `(_?)_(\p{L}+)`.
fn get_appending_rule(key: char, value: &str) -> Option<Rule> {
    if !value.starts_with('_') {
        return None;
    }

    // "_x" or "__x" forms.
    let start = if value.starts_with("__") { 2 } else { 1 };
    let tail = value.get(start..)?;

    let mut letters = Vec::new();
    for ch in tail.chars() {
        if ch.is_alphabetic() {
            letters.push(ch);
        } else {
            break;
        }
    }

    let first = *letters.first()?;

    let mut appended = ['\0'; 2];
    let mut appended_len = 0u8;
    for &ch in letters.iter().skip(1) {
        if (appended_len as usize) < appended.len() {
            appended[appended_len as usize] = ch;
            appended_len += 1;
        }
    }

    Some(Rule {
        key,
        effect_type: EffectType::Appending,
        effect: 0,
        effect_on: first,
        result: first,
        appended,
        appended_len,
    })
}

fn get_mark_family(c: char) -> Vec<char> {
    let base = add_tone_to_char(c, 0);
    let canonical = add_mark_to_toneless_char(base, 0);

    // Marks are 0..=4 in utils' internal mark table.
    let mut family: Vec<char> =
        (0u8..=4).map(|m| add_mark_to_toneless_char(canonical, m)).collect();

    family.sort_unstable();
    family.dedup();
    family
}

fn find_mark_from_char(c: char) -> Option<Mark> {
    let c = c.to_lowercase().next().unwrap_or(c);
    let toneless = add_tone_to_char(c, 0);
    let base = add_mark_to_toneless_char(toneless, 0);

    for m in 0u8..=4 {
        if add_mark_to_toneless_char(base, m) == toneless {
            return Some(match m {
                1 => Mark::Hat,
                2 => Mark::Breve,
                3 => Mark::Horn,
                4 => Mark::Dash,
                _ => Mark::None,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tone_rules() {
        let rules = parse_rules('z', "XoaDauThanh");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].effect_type, EffectType::ToneTransformation);
        assert_eq!(rules[0].effect, Tone::None as u8);

        let rules = parse_rules('x', "DauNga");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].effect_type, EffectType::ToneTransformation);
        assert_eq!(rules[0].get_tone(), Tone::Tilde);
    }

    #[test]
    fn parse_toneless_rules_cases() {
        let rules = parse_toneless_rules('d', "D_Đ");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[0].effect, Mark::Dash as u8);
        assert_eq!(rules[0].effect_on, 'd');

        let rules = parse_toneless_rules('{', "_Ư");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].effect_type, EffectType::Appending);
        assert_eq!(rules[0].effect_on, 'Ư');

        let rules = parse_toneless_rules('w', "UOA_ƯƠĂ");
        assert_eq!(rules.len(), 33);
        assert_eq!(rules[0].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[0].get_mark(), Mark::Horn);
        assert_eq!(rules[0].effect_on, 'u');
        assert_eq!(rules[7].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[7].get_mark(), Mark::Horn);
        assert_eq!(rules[7].effect_on, 'o');
        assert_eq!(rules[20].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[20].get_mark(), Mark::Breve);
        assert_eq!(rules[20].effect_on, 'a');

        let rules = parse_toneless_rules('w', "UOA_ƯƠĂ__Ư");
        assert_eq!(rules.len(), 34);
        assert_eq!(rules[20].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[20].get_mark(), Mark::Breve);
        assert_eq!(rules[20].effect_on, 'a');
        assert_eq!(rules[33].effect_type, EffectType::Appending);
        assert_eq!(rules[33].effect_on, 'ư');
    }

    #[test]
    fn parse_append_rule() {
        let rules = parse_toneless_rules('[', "__ươ");
        assert_eq!(rules.len(), 1);
        let appended_len = rules[0].appended_len;
        assert_eq!(appended_len, 1);
        assert_eq!(rules[0].appended[0], 'ơ');

        let rules = parse_toneless_rules('{', "__ƯƠ");
        assert_eq!(rules.len(), 1);
        let appended_len = rules[0].appended_len;
        assert_eq!(appended_len, 1);
        assert_eq!(rules[0].appended[0], 'Ơ');
    }

    #[test]
    fn parse_input_method_super_key_detection() {
        let im = parse_input_method("Telex");
        assert!(im.super_keys.contains(&'w'));
    }

    #[test]
    fn parse_telex_o_hat_rule_exists() {
        // In Telex, typing 'o' after an existing 'o' should be able to mark it as 'ô'.
        let rules = parse_toneless_rules('o', "O_Ô");
        assert!(rules.iter().any(|r| {
            r.effect_type == EffectType::MarkTransformation
                && r.get_mark() == Mark::Hat
                && r.effect_on == 'o'
                && r.result == 'ô'
        }));
        assert!(!rules.iter().any(|r| r.effect_type == EffectType::Appending));
    }

    #[test]
    fn telex2_has_no_appending_rule_for_o() {
        let im = parse_input_method("Telex 2");
        let o_rules: Vec<_> = im.rules.iter().filter(|r| r.key == 'o').collect();
        assert!(!o_rules.is_empty());
        assert!(!o_rules.iter().any(|r| r.effect_type == EffectType::Appending));
    }
    #[test]
    fn test_input_method_presets_roundtrip() {
        let presets = [
            InputMethodPreset::Telex,
            InputMethodPreset::Vni,
            InputMethodPreset::Viqr,
            InputMethodPreset::MicrosoftLayout,
            InputMethodPreset::Telex2,
            InputMethodPreset::TelexVni,
            InputMethodPreset::TelexVniViqr,
            InputMethodPreset::VniFrenchLayout,
            InputMethodPreset::TelexW,
        ];

        for preset in presets {
            let name = preset.name();
            let parsed: InputMethodPreset = name.parse().expect("Failed to parse preset name");
            assert_eq!(parsed, preset);
            assert_eq!(preset.to_string(), name);

            let im = preset.to_input_method();
            assert!(!im.rules.is_empty(), "Preset {} has empty rules", name);
            assert_eq!(im.name(), name);
        }
    }

    #[test]
    fn test_input_method_from_preset() {
        let im_telex = InputMethod::from_preset(InputMethodPreset::Telex);
        assert_eq!(im_telex.name(), "Telex");
        assert!(!im_telex.rules.is_empty());

        let im_vni = InputMethod::from_preset(InputMethodPreset::Vni);
        assert_eq!(im_vni.name(), "VNI");
        assert!(!im_vni.rules.is_empty());
    }

    #[test]
    fn test_preset_from_str_case_insensitive() {
        assert_eq!("telex".parse::<InputMethodPreset>().unwrap(), InputMethodPreset::Telex);
        assert_eq!("VNI".parse::<InputMethodPreset>().unwrap(), InputMethodPreset::Vni);
        assert_eq!("viqr".parse::<InputMethodPreset>().unwrap(), InputMethodPreset::Viqr);
        assert_eq!(
            "microsoft layout".parse::<InputMethodPreset>().unwrap(),
            InputMethodPreset::MicrosoftLayout
        );
        assert_eq!("telex 2".parse::<InputMethodPreset>().unwrap(), InputMethodPreset::Telex2);
        assert_eq!(
            "telex + vni".parse::<InputMethodPreset>().unwrap(),
            InputMethodPreset::TelexVni
        );
        assert_eq!("telex w".parse::<InputMethodPreset>().unwrap(), InputMethodPreset::TelexW);
    }
}
