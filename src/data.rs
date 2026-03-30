use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PartOption {
    Simple(String),
    Full { id: String, label: String, output: String },
    Labeled { label: String, output: String },
}

impl PartOption {
    pub fn label(&self) -> &str {
        match self {
            Self::Simple(s) => s,
            Self::Full { label, .. } => label,
            Self::Labeled { label, .. } => label,
        }
    }
    pub fn output(&self) -> &str {
        match self {
            Self::Simple(s) => s,
            Self::Full { output, .. } => output,
            Self::Labeled { output, .. } => output,
        }
    }
    pub fn option_id(&self) -> Option<&str> {
        match self {
            Self::Full { id, .. } => Some(id.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositePart {
    pub id: String,
    pub label: String,
    pub preview: Option<String>,
    #[serde(default)]
    pub options: Vec<PartOption>,
    pub data_file: Option<String>,
    #[serde(default)]
    pub sticky: bool,
    pub default: Option<String>,
}

impl CompositePart {
    pub fn default_cursor(&self) -> usize {
        let Some(ref default) = self.default else { return 0; };
        if let Some(pos) = self.options.iter().position(|o| o.option_id() == Some(default.as_str())) {
            return pos;
        }
        if let Some(pos) = self.options.iter().position(|o| o.label() == default.as_str()) {
            return pos;
        }
        0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeConfig {
    pub format: String,
    pub parts: Vec<CompositePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFieldConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub composite: Option<CompositeConfig>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionGroup {
    pub id: String,
    pub num: Option<usize>,
    pub name: String,
    pub sections: Vec<SectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionConfig {
    pub id: String,
    pub name: String,
    pub map_label: String,
    #[serde(rename = "type")]
    pub section_type: String,
    pub data_file: Option<String>,
    pub date_prefix: Option<bool>,
    #[serde(default)]
    pub options: Vec<String>,
    pub composite: Option<CompositeConfig>,
    pub fields: Option<Vec<HeaderFieldConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SectionsFile {
    groups: Vec<SectionGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEntry {
    pub label: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListFile {
    entries: Vec<ListEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChecklistFile {
    items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechniqueConfig {
    pub id: String,
    pub label: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub id: String,
    pub label: String,
    pub header: String,
    pub techniques: Vec<TechniqueConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegionsFile {
    regions: Vec<RegionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub navigate_down: Vec<String>,
    pub navigate_up: Vec<String>,
    pub select: Vec<String>,
    pub confirm: Vec<String>,
    pub add_entry: Vec<String>,
    pub back: Vec<String>,
    pub swap_panes: Vec<String>,
    pub help: Vec<String>,
    pub quit: Vec<String>,
    #[serde(default = "default_focus_left")]
    pub focus_left: Vec<String>,
    #[serde(default = "default_focus_right")]
    pub focus_right: Vec<String>,
    #[serde(default = "default_hints")]
    pub hints: Vec<String>,
    #[serde(default = "default_super_confirm")]
    pub super_confirm: Vec<String>,
    #[serde(default)]
    pub hint_permutations: Vec<String>,
}

fn default_super_confirm() -> Vec<String> {
    vec!["shift+enter".to_string()]
}

fn default_focus_left() -> Vec<String> {
    vec!["left".to_string(), "h".to_string()]
}
fn default_focus_right() -> Vec<String> {
    vec!["right".to_string(), "i".to_string()]
}
fn default_hints() -> Vec<String> {
    ["1","2","3","4","5","6","7","8","9"].iter().map(|s| s.to_string()).collect()
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            navigate_down: vec!["down".to_string(), "n".to_string()],
            navigate_up: vec!["up".to_string(), "e".to_string()],
            select: vec!["space".to_string(), "s".to_string()],
            confirm: vec!["enter".to_string(), "t".to_string()],
            add_entry: vec!["a".to_string(), "d".to_string()],
            back: vec!["esc".to_string()],
            swap_panes: vec!["`".to_string(), "\\".to_string()],
            help: vec!["?".to_string()],
            quit: vec!["q".to_string()],
            focus_left: default_focus_left(),
            focus_right: default_focus_right(),
            hints: default_hints(),
            super_confirm: default_super_confirm(),
            hint_permutations: vec![],
        }
    }
}

pub struct AppData {
    pub groups: Vec<SectionGroup>,
    pub sections: Vec<SectionConfig>,
    pub list_data: HashMap<String, Vec<ListEntry>>,
    pub checklist_data: HashMap<String, Vec<String>>,
    pub region_data: HashMap<String, Vec<RegionConfig>>,
    pub keybindings: KeyBindings,
    pub data_dir: PathBuf,
}

impl AppData {
    pub fn load(data_dir: PathBuf) -> Result<Self> {
        let sections_path = data_dir.join("sections.yml");
        let sections_content = fs::read_to_string(&sections_path)?;
        let sections_file: SectionsFile = serde_yaml::from_str(&sections_content)?;

        let groups = sections_file.groups.clone();
        let sections: Vec<SectionConfig> = groups
            .iter()
            .flat_map(|g| g.sections.iter().cloned())
            .collect();

        let mut list_data: HashMap<String, Vec<ListEntry>> = HashMap::new();
        let mut checklist_data: HashMap<String, Vec<String>> = HashMap::new();
        let mut region_data: HashMap<String, Vec<RegionConfig>> = HashMap::new();

        for section in &sections {
            if let Some(ref data_file) = section.data_file {
                let path = data_dir.join(data_file);
                if path.exists() {
                    let content = fs::read_to_string(&path)?;
                    match section.section_type.as_str() {
                        "list_select" => {
                            let file: ListFile = serde_yaml::from_str(&content)?;
                            list_data.insert(data_file.clone(), file.entries);
                        }
                        "checklist" => {
                            let file: ChecklistFile = serde_yaml::from_str(&content)?;
                            checklist_data.insert(data_file.clone(), file.items);
                        }
                        "block_select" => {
                            let file: RegionsFile = serde_yaml::from_str(&content)?;
                            region_data.insert(data_file.clone(), file.regions);
                        }
                        _ => {}
                    }
                }
            }
        }

        let kb_path = data_dir.join("keybindings.yml");
        let mut keybindings = if kb_path.exists() {
            let kb_content = fs::read_to_string(&kb_path)?;
            match serde_yaml::from_str(&kb_content) {
                Ok(kb) => kb,
                Err(e) => {
                    eprintln!("Warning: keybindings.yml parse error ({}), using defaults", e);
                    KeyBindings::default()
                }
            }
        } else {
            KeyBindings::default()
        };

        ensure_hint_permutations(&mut keybindings);

        Ok(Self {
            groups,
            sections,
            list_data,
            checklist_data,
            region_data,
            keybindings,
            data_dir,
        })
    }

    pub fn reload_list(&mut self, data_file: &str) -> Result<()> {
        let path = self.data_dir.join(data_file);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let file: ListFile = serde_yaml::from_str(&content)?;
            self.list_data.insert(data_file.to_string(), file.entries);
        }
        Ok(())
    }

    pub fn append_list_entry(&mut self, data_file: &str, entry: ListEntry) -> Result<()> {
        let path = self.data_dir.join(data_file);
        let mut entries = if path.exists() {
            let content = fs::read_to_string(&path)?;
            let file: ListFile = serde_yaml::from_str(&content)?;
            file.entries
        } else {
            vec![]
        };
        entries.push(entry);
        let file = ListFile { entries };
        let content = serde_yaml::to_string(&file)?;
        fs::write(&path, content)?;
        self.reload_list(data_file)?;
        Ok(())
    }
}

pub fn generate_hint_permutations(base: &[String], count_needed: usize) -> Vec<String> {
    let n = base.len();
    if n == 0 || count_needed == 0 {
        return vec![];
    }

    let mut result: Vec<String> = Vec::with_capacity(count_needed);

    // r=1: single characters (band 0 only - each char is its own "pair")
    // Skip r=1; hints field already covers single chars.
    // r=2: iterate distance bands 0..n
    'outer: for dist in 0..n {
        for i in 0..n {
            // j = i + dist (wrap is not meaningful for linear adjacency - skip wrapping)
            if dist == 0 {
                // Same-index pairs: "qq", "ww", etc.
                let entry = format!("{}{}", base[i], base[i]);
                result.push(entry);
                if result.len() >= count_needed {
                    break 'outer;
                }
            } else {
                // (i, i+dist) and (i+dist, i) - both directions
                let j = i + dist;
                if j < n {
                    result.push(format!("{}{}", base[i], base[j]));
                    if result.len() >= count_needed { break 'outer; }
                    result.push(format!("{}{}", base[j], base[i]));
                    if result.len() >= count_needed { break 'outer; }
                }
            }
        }
    }

    if result.len() >= count_needed {
        return result;
    }

    // r=3 fallback: extend each r=2 entry with all base chars in adjacency order
    let r2_complete = result.clone();
    'r3: for prefix in &r2_complete {
        for dist in 0..n {
            for i in 0..n {
                if dist == 0 {
                    let entry = format!("{}{}", prefix, base[i]);
                    result.push(entry);
                    if result.len() >= count_needed { break 'r3; }
                } else {
                    let j = i + dist;
                    if j < n {
                        result.push(format!("{}{}", prefix, base[i]));
                        if result.len() >= count_needed { break 'r3; }
                        result.push(format!("{}{}", prefix, base[j]));
                        if result.len() >= count_needed { break 'r3; }
                    }
                }
            }
        }
    }

    result.truncate(count_needed);
    result
}

/// Ensures `kb.hint_permutations` is populated and up-to-date.
///
/// count_needed is `hints.len()^2` (the full r=2 space). Regeneration is triggered when
/// `hint_permutations` is empty or its length does not match count_needed (staleness).
pub fn ensure_hint_permutations(kb: &mut KeyBindings) {
    let n = kb.hints.len();
    if n == 0 {
        return;
    }
    let count_needed = n * n;
    if kb.hint_permutations.len() == count_needed {
        return; // already fresh
    }
    kb.hint_permutations = generate_hint_permutations(&kb.hints, count_needed);
}

/// Returns a combined ordered slice of all hints followed by all hint_permutations.
/// Use this wherever hints are assigned to groups, sections, fields, or modal rows.
pub fn combined_hints(kb: &KeyBindings) -> Vec<&str> {
    kb.hints.iter().map(String::as_str)
        .chain(kb.hint_permutations.iter().map(String::as_str))
        .collect()
}

/// Returns the flat section index of the first section in `groups[g_idx]`.
///
/// - If `g_idx` is in bounds, returns the sum of `sections.len()` for all preceding groups.
/// - If the group exists but has 0 sections, returns the same start index (which equals the
///   next group's start, or total section count if it is the last group).
/// - If `g_idx >= groups.len()`, returns the total section count (past-the-end sentinel).
pub fn group_jump_target(groups: &[SectionGroup], g_idx: usize) -> usize {
    if g_idx >= groups.len() {
        return groups.iter().map(|g| g.sections.len()).sum();
    }
    groups.iter().take(g_idx).map(|g| g.sections.len()).sum()
}

#[derive(Debug, PartialEq)]
pub enum HintResolveResult {
    Exact(usize),
    Partial(Vec<usize>),
    NoMatch,
}

/// Returns the indices of all hints that start with `prefix`.
/// An empty `prefix` matches every hint.
pub fn filter_hints_by_prefix(hints: &[&str], prefix: &str) -> Vec<usize> {
    hints
        .iter()
        .enumerate()
        .filter_map(|(i, h)| if h.starts_with(prefix) { Some(i) } else { None })
        .collect()
}

/// Resolves the current typed string against the hint list.
///
/// - `NoMatch`   - no hint starts with `typed`
/// - `Exact(i)`  - exactly one hint starts with `typed` AND equals `typed` in full
/// - `Partial(v)`- one or more hints share the prefix but none is an exact full match,
///                 or more than one match exists
pub fn resolve_hint(hints: &[&str], typed: &str) -> HintResolveResult {
    let matches = filter_hints_by_prefix(hints, typed);
    match matches.as_slice() {
        [] => HintResolveResult::NoMatch,
        [idx] if hints[*idx] == typed => HintResolveResult::Exact(*idx),
        _ => HintResolveResult::Partial(matches),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keybindings_default_has_super_confirm_shift_enter() {
        let kb = KeyBindings::default();
        assert_eq!(
            kb.super_confirm,
            vec!["shift+enter".to_string()],
            "KeyBindings::default() should have super_confirm = [\"shift+enter\"]"
        );
    }

    #[test]
    fn keybindings_super_confirm_serde_default() {
        // When deserializing a KeyBindings that omits super_confirm, it should default to ["shift+enter"]
        let yaml = "navigate_down: [down]\nnavigate_up: [up]\nselect: [space]\nconfirm: [enter]\nadd_entry: [a]\nback: [esc]\nswap_panes: ['`']\nhelp: ['?']\nquit: [q]\n";
        let kb: KeyBindings = serde_yaml::from_str(yaml).expect("should parse keybindings");
        assert_eq!(
            kb.super_confirm,
            vec!["shift+enter".to_string()],
            "super_confirm should default to [\"shift+enter\"] when absent from YAML"
        );
    }

    // ---- hint_permutations tests (Task #23 sub-task 1) ----

    /// The output must be capped at count_needed.
    #[test]
    fn hint_permutations_capped_at_count_needed() {
        let base: Vec<String> = vec!["q", "w", "f", "p"]
            .into_iter().map(|s| s.to_string()).collect();
        let result = generate_hint_permutations(&base, 5);
        assert_eq!(result.len(), 5, "output should be capped at count_needed=5");
    }

    /// r=2 permutations from a 4-element base produce 4^2 = 16 combos when count_needed >= 16.
    #[test]
    fn hint_permutations_r2_from_4_element_base() {
        let base: Vec<String> = vec!["q", "w", "f", "p"]
            .into_iter().map(|s| s.to_string()).collect();
        // Ask for exactly 16 - the full r=2 space
        let result = generate_hint_permutations(&base, 16);
        assert_eq!(result.len(), 16, "4-element base should yield 16 r=2 permutations");
        // Every entry should be exactly 2 characters (single-char keys concatenated)
        for entry in result.iter() {
            let entry: &String = entry;
            assert_eq!(entry.len(), 2, "each r=2 entry should have length 2, got: {entry}");
        }
    }

    /// Adjacent pairs appear before distant pairs in adjacency-priority ordering.
    /// For base [q, w, f, p] the adjacent pairs are qq, qw, wq, ww (indices 0-1 are neighbours).
    /// The distant pair qp (indices 0 and 3) must appear later.
    #[test]
    fn hint_permutations_adjacency_ordering_adjacent_before_distant() {
        let base: Vec<String> = vec!["q", "w", "f", "p"]
            .into_iter().map(|s| s.to_string()).collect();
        let result = generate_hint_permutations(&base, 16);

        let pos_qq = result.iter().position(|s| s == "qq").expect("qq should be present");
        let pos_qw = result.iter().position(|s| s == "qw").expect("qw should be present");
        let pos_wq = result.iter().position(|s| s == "wq").expect("wq should be present");
        let pos_ww = result.iter().position(|s| s == "ww").expect("ww should be present");
        let pos_qp = result.iter().position(|s| s == "qp").expect("qp should be present");

        assert!(pos_qq < pos_qp, "qq (adjacent) should appear before qp (distant)");
        assert!(pos_qw < pos_qp, "qw (adjacent) should appear before qp (distant)");
        assert!(pos_wq < pos_qp, "wq (adjacent) should appear before qp (distant)");
        assert!(pos_ww < pos_qp, "ww (adjacent) should appear before qp (distant)");
    }

    /// When count_needed > base^2 (r=2 space exhausted), r=3 entries must appear to fill the gap.
    #[test]
    fn hint_permutations_r3_fallback_when_r2_not_enough() {
        let base: Vec<String> = vec!["q", "w", "f", "p"]
            .into_iter().map(|s| s.to_string()).collect();
        // 4^2=16 r=2 entries; ask for 20 to force r=3 entries
        let result = generate_hint_permutations(&base, 20);
        assert_eq!(result.len(), 20, "should produce 20 entries when r=3 fallback is needed");
        // At least one entry should have length 3 (an r=3 permutation)
        let has_r3 = result.iter().any(|s: &String| s.len() == 3);
        assert!(has_r3, "result should contain at least one r=3 entry when count_needed > 4^2");
    }

    /// KeyBindings must have a hint_permutations field that defaults to empty vec.
    #[test]
    fn keybindings_hint_permutations_field_defaults_empty() {
        let kb = KeyBindings::default();
        assert!(
            kb.hint_permutations.is_empty(),
            "KeyBindings::default() hint_permutations should be empty"
        );
    }

    /// hint_permutations deserializes correctly from YAML when absent (defaults to empty).
    #[test]
    fn keybindings_hint_permutations_serde_default_empty() {
        let yaml = "navigate_down: [down]\nnavigate_up: [up]\nselect: [space]\nconfirm: [enter]\nadd_entry: [a]\nback: [esc]\nswap_panes: ['`']\nhelp: ['?']\nquit: [q]\n";
        let kb: KeyBindings = serde_yaml::from_str(yaml).expect("should parse keybindings");
        assert!(
            kb.hint_permutations.is_empty(),
            "hint_permutations should default to empty vec when absent from YAML"
        );
    }

    /// hint_permutations deserializes correctly from YAML when explicitly provided.
    #[test]
    fn keybindings_hint_permutations_serde_explicit_value() {
        let yaml = "navigate_down: [down]\nnavigate_up: [up]\nselect: [space]\nconfirm: [enter]\nadd_entry: [a]\nback: [esc]\nswap_panes: ['`']\nhelp: ['?']\nquit: [q]\nhint_permutations: [qq, qw, wq]\n";
        let kb: KeyBindings = serde_yaml::from_str(yaml).expect("should parse keybindings");
        assert_eq!(
            kb.hint_permutations,
            vec!["qq".to_string(), "qw".to_string(), "wq".to_string()],
            "hint_permutations should deserialize the provided YAML values"
        );
    }

    // ---- ensure_hint_permutations tests (Task #23 sub-task 2) ----

    /// Regeneration is triggered when hint_permutations is empty.
    #[test]
    fn ensure_hint_permutations_populates_when_empty() {
        let mut kb = KeyBindings::default(); // hint_permutations = []
        assert!(kb.hint_permutations.is_empty(), "precondition: starts empty");
        ensure_hint_permutations(&mut kb);
        let expected_count = kb.hints.len() * kb.hints.len();
        assert_eq!(
            kb.hint_permutations.len(),
            expected_count,
            "hint_permutations should be populated after ensure call"
        );
    }

    /// Regeneration is triggered when hints list changes (staleness by hints.len() change).
    #[test]
    fn ensure_hint_permutations_regenerates_when_hints_change() {
        let mut kb = KeyBindings::default();
        ensure_hint_permutations(&mut kb);
        let original_len = kb.hint_permutations.len();

        // Simulate hints list change: add an extra hint
        kb.hints.push("z".to_string());
        // hint_permutations.len() is now stale (doesn't equal new hints.len()^2)
        ensure_hint_permutations(&mut kb);

        let new_expected = kb.hints.len() * kb.hints.len();
        assert_ne!(
            kb.hint_permutations.len(),
            original_len,
            "hint_permutations should be regenerated after hints list change"
        );
        assert_eq!(
            kb.hint_permutations.len(),
            new_expected,
            "regenerated hint_permutations should match new count_needed"
        );
    }

    /// No regeneration when hint_permutations is already fresh (idempotent).
    #[test]
    fn ensure_hint_permutations_no_regen_when_fresh() {
        let mut kb = KeyBindings::default();
        ensure_hint_permutations(&mut kb);
        let populated = kb.hint_permutations.clone();

        // Call again - should not change anything
        ensure_hint_permutations(&mut kb);
        assert_eq!(
            kb.hint_permutations,
            populated,
            "ensure_hint_permutations should be idempotent when already fresh"
        );
    }

    // ---- combined_hints tests (Task #23 sub-task 3) ----

    /// First n entries match hints, remaining match hint_permutations.
    #[test]
    fn combined_hints_returns_hints_then_permutations() {
        let mut kb = KeyBindings::default();
        ensure_hint_permutations(&mut kb);
        let combined = combined_hints(&kb);
        let n = kb.hints.len();
        for (i, h) in kb.hints.iter().enumerate() {
            assert_eq!(combined[i], h.as_str(), "combined[{}] should match hints[{}]", i, i);
        }
        for (i, p) in kb.hint_permutations.iter().enumerate() {
            assert_eq!(combined[n + i], p.as_str(), "combined[{}] should match hint_permutations[{}]", n + i, i);
        }
    }

    /// combined.len() == hints.len() + hint_permutations.len()
    #[test]
    fn combined_hints_total_length() {
        let mut kb = KeyBindings::default();
        ensure_hint_permutations(&mut kb);
        let combined = combined_hints(&kb);
        assert_eq!(
            combined.len(),
            kb.hints.len() + kb.hint_permutations.len(),
            "combined length should equal hints.len() + hint_permutations.len()"
        );
    }

    /// When hint_permutations is empty, combined == hints.
    #[test]
    fn combined_hints_empty_permutations() {
        let kb = KeyBindings::default(); // hint_permutations starts empty
        let combined = combined_hints(&kb);
        assert_eq!(combined.len(), kb.hints.len(), "combined length should equal hints.len() when permutations are empty");
        for (i, h) in kb.hints.iter().enumerate() {
            assert_eq!(combined[i], h.as_str(), "combined[{}] should match hints[{}]", i, i);
        }
    }

    /// With explicit [a, b] hints and [aa, ab] permutations, combined == [a, b, aa, ab].
    #[test]
    fn combined_hints_order_hints_before_permutations() {
        let kb = KeyBindings {
            hints: vec!["a".to_string(), "b".to_string()],
            hint_permutations: vec!["aa".to_string(), "ab".to_string()],
            ..KeyBindings::default()
        };
        let combined = combined_hints(&kb);
        assert_eq!(combined, vec!["a", "b", "aa", "ab"]);
    }

    // ---- filter_hints_by_prefix / resolve_hint tests (Task #22 sub-task 1) ----

    #[test]
    fn filter_hints_by_prefix_returns_matching_indices() {
        let hints = ["q", "w", "qq", "qw"];
        let result = filter_hints_by_prefix(&hints, "q");
        assert_eq!(result, vec![0usize, 2, 3]);
    }

    #[test]
    fn filter_hints_by_prefix_empty_prefix_returns_all() {
        let hints = ["q", "w", "qq", "qw"];
        let result = filter_hints_by_prefix(&hints, "");
        assert_eq!(result, vec![0usize, 1, 2, 3]);
    }

    #[test]
    fn filter_hints_by_prefix_no_match_returns_empty() {
        let hints = ["q", "w", "qq", "qw"];
        let result = filter_hints_by_prefix(&hints, "z");
        assert_eq!(result, Vec::<usize>::new());
    }

    #[test]
    fn resolve_hint_single_char_exact() {
        let hints = ["q", "w"];
        let result = resolve_hint(&hints, "q");
        assert_eq!(result, HintResolveResult::Exact(0));
    }

    #[test]
    fn resolve_hint_partial_match() {
        let hints = ["qq", "qw", "ww"];
        let result = resolve_hint(&hints, "q");
        assert_eq!(result, HintResolveResult::Partial(vec![0, 1]));
    }

    #[test]
    fn resolve_hint_exact_multichar() {
        let hints = ["qq", "qw"];
        let result = resolve_hint(&hints, "qq");
        assert_eq!(result, HintResolveResult::Exact(0));
    }

    #[test]
    fn resolve_hint_no_match() {
        let hints = ["qq", "qw"];
        let result = resolve_hint(&hints, "z");
        assert_eq!(result, HintResolveResult::NoMatch);
    }

    #[test]
    fn resolve_hint_no_match_resets() {
        let hints = ["qq", "qw"];
        let result = resolve_hint(&hints, "qz");
        assert_eq!(result, HintResolveResult::NoMatch);
    }

    #[test]
    fn resolve_hint_partial_one_match_longer_than_typed() {
        // Only one hint remains but it is longer than typed — must be Partial, not Exact
        let hints = ["q", "w", "zz"];
        let result = resolve_hint(&hints, "z");
        assert_eq!(result, HintResolveResult::Partial(vec![2]));
    }

    // ---- group_jump_target tests (Task #21 sub-task 1) ----

    fn make_group(id: &str, section_count: usize) -> SectionGroup {
        let sections = (0..section_count)
            .map(|i| SectionConfig {
                id: format!("{}-s{}", id, i),
                name: format!("Section {}", i),
                map_label: format!("s{}", i),
                section_type: "text".to_string(),
                data_file: None,
                date_prefix: None,
                options: vec![],
                composite: None,
                fields: None,
            })
            .collect();
        SectionGroup {
            id: id.to_string(),
            num: None,
            name: id.to_string(),
            sections,
        }
    }

    #[test]
    fn group_jump_target_first_group() {
        // 3 groups with [3, 2, 1] sections; g_idx=0 -> 0 (start of first group)
        let groups = vec![make_group("a", 3), make_group("b", 2), make_group("c", 1)];
        assert_eq!(group_jump_target(&groups, 0), 0);
    }

    #[test]
    fn group_jump_target_second_group() {
        // 3 groups with [3, 2, 1] sections; g_idx=1 -> 3 (after first group's 3 sections)
        let groups = vec![make_group("a", 3), make_group("b", 2), make_group("c", 1)];
        assert_eq!(group_jump_target(&groups, 1), 3);
    }

    #[test]
    fn group_jump_target_third_group() {
        // 3 groups with [3, 2, 1] sections; g_idx=2 -> 5 (after 3+2 sections)
        let groups = vec![make_group("a", 3), make_group("b", 2), make_group("c", 1)];
        assert_eq!(group_jump_target(&groups, 2), 5);
    }

    #[test]
    fn group_jump_target_out_of_bounds() {
        // g_idx=3 is past-the-end; should return total section count (3+2+1=6)
        let groups = vec![make_group("a", 3), make_group("b", 2), make_group("c", 1)];
        assert_eq!(group_jump_target(&groups, 3), 6);
    }

    #[test]
    fn group_jump_target_empty_group() {
        // groups with [2, 0, 1] sections; g_idx=1 -> 2 (empty group starts where it starts)
        let groups = vec![make_group("a", 2), make_group("b", 0), make_group("c", 1)];
        assert_eq!(group_jump_target(&groups, 1), 2);
    }

    // ---- group_jump_target additional tests using make_groups helper (Task #21 sub-task 1) ----

    fn make_groups(sizes: &[usize]) -> Vec<SectionGroup> {
        sizes
            .iter()
            .enumerate()
            .map(|(i, &n)| SectionGroup {
                id: format!("g{i}"),
                num: None,
                name: format!("Group {i}"),
                sections: (0..n)
                    .map(|j| SectionConfig {
                        id: format!("s{i}_{j}"),
                        name: format!("Section {i}/{j}"),
                        map_label: format!("{i}/{j}"),
                        section_type: "free_text".to_string(),
                        data_file: None,
                        date_prefix: None,
                        options: vec![],
                        composite: None,
                        fields: None,
                    })
                    .collect(),
            })
            .collect()
    }

    #[test]
    fn group_jump_target_group0_returns_0() {
        let groups = make_groups(&[3, 2, 4]);
        assert_eq!(group_jump_target(&groups, 0), 0);
    }

    #[test]
    fn group_jump_target_group1_returns_sum_of_group0() {
        let groups = make_groups(&[3, 2, 4]);
        assert_eq!(group_jump_target(&groups, 1), 3);
    }

    #[test]
    fn group_jump_target_group2_returns_sum_of_groups_0_and_1() {
        let groups = make_groups(&[3, 2, 4]);
        assert_eq!(group_jump_target(&groups, 2), 5);
    }

    #[test]
    fn group_jump_target_out_of_bounds_returns_total_count() {
        let groups = make_groups(&[3, 2, 4]);
        // total = 9; g_idx = 3 is out of bounds
        assert_eq!(group_jump_target(&groups, 3), 9);
    }

    #[test]
    fn group_jump_target_far_out_of_bounds_returns_total_count() {
        let groups = make_groups(&[3, 2, 4]);
        assert_eq!(group_jump_target(&groups, 100), 9);
    }

    #[test]
    fn group_jump_target_empty_group_returns_same_as_next_start() {
        // group 1 has 0 sections; its start == group 0's end == 3
        let groups = make_groups(&[3, 0, 4]);
        assert_eq!(group_jump_target(&groups, 1), 3);
        // group 2's start == 3 + 0 == 3 as well
        assert_eq!(group_jump_target(&groups, 2), 3);
    }

    #[test]
    fn group_jump_target_all_empty_groups() {
        let groups = make_groups(&[0, 0, 0]);
        assert_eq!(group_jump_target(&groups, 0), 0);
        assert_eq!(group_jump_target(&groups, 1), 0);
        assert_eq!(group_jump_target(&groups, 2), 0);
        assert_eq!(group_jump_target(&groups, 3), 0); // out of bounds, total = 0
    }

    #[test]
    fn group_jump_target_single_group() {
        let groups = make_groups(&[5]);
        assert_eq!(group_jump_target(&groups, 0), 0);
        assert_eq!(group_jump_target(&groups, 1), 5); // out of bounds
    }

    #[test]
    fn group_jump_target_empty_slice() {
        let groups: Vec<SectionGroup> = vec![];
        assert_eq!(group_jump_target(&groups, 0), 0); // out of bounds, total = 0
    }
}

pub fn find_data_dir() -> PathBuf {
    // Try cwd/data first (development)
    let cwd_data = std::env::current_dir()
        .unwrap_or_default()
        .join("data");
    if cwd_data.join("sections.yml").exists() {
        return cwd_data;
    }

    // Try exe parent/data
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_data = parent.join("data");
            if exe_data.join("sections.yml").exists() {
                return exe_data;
            }
        }
    }

    // Fallback to cwd/data
    cwd_data
}
