// Hint-label helpers extracted from data.rs.
// Owns the runtime hint resolver, authored-prefix label assignment, generated
// chord alphabet, permutation generator, and the small KeyBindings-aware
// helpers that ride alongside them. Loader, validation, and runtime-build
// helpers stay in data.rs (or their later sibling files); data.rs continues to
// be the single public surface via `pub use crate::data_hints::*`.

use crate::data_model::{default_hints, KeyBindings};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintResolveResult {
    Exact(usize),
    Partial(Vec<usize>),
    NoMatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintLabelAssignment {
    pub label: String,
    pub authored: bool,
}

pub fn resolve_hint(hints: &[&str], typed: &str) -> HintResolveResult {
    let matches: Vec<usize> = hints
        .iter()
        .enumerate()
        .filter_map(|(idx, hint)| hint.starts_with(typed).then_some(idx))
        .collect();

    match matches.as_slice() {
        [] => HintResolveResult::NoMatch,
        [idx] if hints[*idx] == typed => HintResolveResult::Exact(*idx),
        _ => HintResolveResult::Partial(matches),
    }
}

pub fn assign_hint_labels(
    base: &[String],
    explicit_prefixes: &[Option<&str>],
    case_sensitive: bool,
) -> Vec<HintLabelAssignment> {
    if explicit_prefixes.is_empty() {
        return Vec::new();
    }

    let generation_base = hint_generation_alphabet(base, case_sensitive);
    let mut assignments: Vec<Option<HintLabelAssignment>> = vec![None; explicit_prefixes.len()];
    let mut used_labels = HashSet::new();
    let mut reserved_prefixes = Vec::new();
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut group_order = Vec::new();

    for (idx, prefix) in explicit_prefixes.iter().enumerate() {
        let Some(prefix) = prefix else {
            continue;
        };
        let normalized = normalize_hint_value(prefix, case_sensitive);
        if !groups.contains_key(&normalized) {
            reserved_prefixes.push(normalized.clone());
            group_order.push(normalized.clone());
        }
        groups.entry(normalized).or_default().push(idx);
    }

    for prefix in group_order {
        let Some(group_indices) = groups.get(&prefix) else {
            continue;
        };
        if group_indices.len() == 1 {
            let label = prefix.clone();
            used_labels.insert(label.clone());
            assignments[group_indices[0]] = Some(HintLabelAssignment {
                label,
                authored: true,
            });
            continue;
        }

        let suffixes = take_available_generated_labels(
            &generation_base,
            group_indices.len(),
            &[],
            &HashSet::new(),
        );
        for (idx, suffix) in group_indices.iter().zip(suffixes.into_iter()) {
            let label = format!("{prefix}{suffix}");
            used_labels.insert(label.clone());
            assignments[*idx] = Some(HintLabelAssignment {
                label,
                authored: true,
            });
        }
    }

    let generated_needed = assignments.iter().filter(|entry| entry.is_none()).count();
    let generated = take_available_generated_labels(
        &generation_base,
        generated_needed,
        &reserved_prefixes,
        &used_labels,
    );
    let mut generated_iter = generated.into_iter();
    for entry in &mut assignments {
        if entry.is_none() {
            let label = generated_iter.next().unwrap_or_default();
            *entry = Some(HintLabelAssignment {
                label,
                authored: false,
            });
        }
    }

    assignments
        .into_iter()
        .map(|entry| {
            entry.unwrap_or(HintLabelAssignment {
                label: String::new(),
                authored: false,
            })
        })
        .collect()
}

fn hint_generation_alphabet(base: &[String], case_sensitive: bool) -> Vec<String> {
    let mut alphabet = Vec::new();
    let mut seen = HashSet::new();

    for candidate in base {
        let normalized = normalize_hint_value(candidate, case_sensitive);
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            alphabet.push(normalized);
        }
    }

    for candidate in default_hints() {
        let normalized = normalize_hint_value(&candidate, case_sensitive);
        if seen.insert(normalized.clone()) {
            alphabet.push(normalized);
        }
    }

    for ch in 'a'..='z' {
        let candidate = normalize_hint_value(&ch.to_string(), case_sensitive);
        if seen.insert(candidate.clone()) {
            alphabet.push(candidate);
        }
    }

    if alphabet.is_empty() {
        alphabet.push("1".to_string());
    }

    alphabet
}

fn take_available_generated_labels(
    base: &[String],
    needed: usize,
    reserved_prefixes: &[String],
    used_labels: &HashSet<String>,
) -> Vec<String> {
    if needed == 0 {
        return Vec::new();
    }

    let mut results = Vec::with_capacity(needed);
    let mut used = used_labels.clone();
    let mut length = 1usize;
    while results.len() < needed {
        let Some(total) = hint_label_count_for_length(base.len(), length) else {
            break;
        };
        for ordinal in 0..total {
            let candidate = encode_hint_label(base, length, ordinal);
            if used.contains(&candidate) {
                continue;
            }
            if reserved_prefixes
                .iter()
                .any(|prefix| !prefix.is_empty() && candidate.starts_with(prefix))
            {
                continue;
            }
            used.insert(candidate.clone());
            results.push(candidate);
            if results.len() >= needed {
                return results;
            }
        }
        length += 1;
    }
    results
}

fn hint_label_count_for_length(base_len: usize, length: usize) -> Option<usize> {
    if base_len == 0 || length == 0 {
        return Some(0);
    }
    let mut total = 1usize;
    for _ in 0..length {
        total = total.checked_mul(base_len)?;
    }
    Some(total)
}

fn encode_hint_label(base: &[String], chord_len: usize, ordinal: usize) -> String {
    let mut value = ordinal;
    let mut parts = vec![String::new(); chord_len];
    for slot in (0..chord_len).rev() {
        let idx = value % base.len();
        parts[slot] = base[idx].clone();
        value /= base.len();
    }
    parts.concat()
}

fn normalize_hint_value(value: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        value.to_string()
    } else {
        value.to_ascii_lowercase()
    }
}

pub fn generate_hint_permutations(base: &[String], count_needed: usize) -> Vec<String> {
    let n = base.len();
    if n == 0 || count_needed == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(count_needed);
    'outer: for dist in 0..n {
        for i in 0..n {
            if dist == 0 {
                result.push(format!("{}{}", base[i], base[i]));
                if result.len() >= count_needed {
                    break 'outer;
                }
            } else {
                let j = i + dist;
                if j < n {
                    result.push(format!("{}{}", base[i], base[j]));
                    if result.len() >= count_needed {
                        break 'outer;
                    }
                    result.push(format!("{}{}", base[j], base[i]));
                    if result.len() >= count_needed {
                        break 'outer;
                    }
                }
            }
        }
    }
    result.truncate(count_needed);
    result
}

pub fn ensure_hint_permutations(kb: &mut KeyBindings) {
    let count_needed = kb.hints.len() * kb.hints.len();
    if kb.hint_permutations.len() == count_needed {
        return;
    }
    kb.hint_permutations = generate_hint_permutations(&kb.hints, count_needed);
}

pub fn combined_hints(kb: &KeyBindings) -> Vec<&str> {
    kb.hints
        .iter()
        .map(String::as_str)
        .chain(kb.hint_permutations.iter().map(String::as_str))
        .collect()
}
