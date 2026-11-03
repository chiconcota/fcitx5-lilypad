use crate::input_method::{InputMethod, Rule};
use crate::phonetics::lower;
use std::sync::Arc;

/// Precomputed, partitioned rules and lookup tables for fast key evaluation.
#[derive(Clone, Debug)]
pub struct EngineRules {
    pub(crate) all_rules: Arc<[Rule]>,
    pub(crate) ascii_rule_indices: [(u16, u16); 128],
    pub(crate) non_ascii_rule_indices: Arc<[(char, (u16, u16))]>,
    pub(crate) ascii_effect_keys: [bool; 128],
    pub(crate) non_ascii_effect_keys: Arc<[char]>,
}

impl EngineRules {
    /// Constructs partitioned engine rules from an [`InputMethod`].
    pub fn from_input_method(im: &InputMethod) -> Self {
        let mut ascii_counts = [0u16; 128];
        let mut non_ascii_keys: Vec<char> = Vec::new();

        for rule in &im.rules {
            let key = lower(rule.key);
            if (key as u32) < 128 {
                ascii_counts[key as usize] += 1;
            } else if !non_ascii_keys.contains(&key) {
                non_ascii_keys.push(key);
            }
        }

        let mut all_rules = vec![Rule::default(); im.rules.len()];
        let mut ascii_rule_indices = [(0u16, 0u16); 128];
        let mut ascii_write_pos = [0u16; 128];
        let mut offset = 0u16;

        for (ascii_char, &count) in ascii_counts.iter().enumerate() {
            if count > 0 {
                ascii_rule_indices[ascii_char] = (offset, offset + count);
                ascii_write_pos[ascii_char] = offset;
                offset += count;
            }
        }

        let mut non_ascii_indices = Vec::with_capacity(non_ascii_keys.len());
        let mut non_ascii_write_pos = Vec::with_capacity(non_ascii_keys.len());
        for &k in &non_ascii_keys {
            let count = im.rules.iter().filter(|r| lower(r.key) == k).count() as u16;
            non_ascii_indices.push((k, (offset, offset + count)));
            non_ascii_write_pos.push(offset);
            offset += count;
        }

        for rule in &im.rules {
            let key = lower(rule.key);
            if (key as u32) < 128 {
                let pos = ascii_write_pos[key as usize] as usize;
                all_rules[pos] = *rule;
                ascii_write_pos[key as usize] += 1;
            } else if let Some(idx) = non_ascii_keys.iter().position(|&k| k == key) {
                let pos = non_ascii_write_pos[idx] as usize;
                all_rules[pos] = *rule;
                non_ascii_write_pos[idx] += 1;
            }
        }

        let mut ascii_effect_keys = [false; 128];
        let mut non_ascii_effect_keys: Vec<char> = Vec::new();
        for key in &im.keys {
            if key.is_ascii() {
                ascii_effect_keys[*key as usize] = true;
            } else {
                non_ascii_effect_keys.push(*key);
            }
        }
        non_ascii_effect_keys.sort_unstable();
        non_ascii_effect_keys.dedup();

        Self {
            all_rules: all_rules.into(),
            ascii_rule_indices,
            non_ascii_rule_indices: non_ascii_indices.into(),
            ascii_effect_keys,
            non_ascii_effect_keys: non_ascii_effect_keys.into(),
        }
    }
}
