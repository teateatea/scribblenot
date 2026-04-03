use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use crate::flat_file::FlatFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PartOption {
    Simple(String),
    Full { id: String, label: String, output: String, #[serde(default = "default_true")] default: bool },
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
    pub fn default_selected(&self) -> bool {
        match self {
            Self::Full { default, .. } => *default,
            _ => true,
        }
    }
}

fn default_true() -> bool { true }

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
    #[serde(default)]
    pub repeat_limit: Option<usize>,
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
    #[serde(default)]
    pub is_intake: bool,
    #[serde(default)]
    pub heading_search_text: Option<String>,
    #[serde(default)]
    pub heading_label: Option<String>,
    #[serde(default)]
    pub note_render_slot: Option<String>,
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
pub struct BlockSelectEntry {
    pub id: String,
    pub label: String,
    pub header: String,
    pub entries: Vec<PartOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockSelectFile {
    entries: Vec<BlockSelectEntry>,
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
    #[serde(default = "default_copy_note")]
    pub copy_note: Vec<String>,
}

fn default_copy_note() -> Vec<String> {
    vec!["c".to_string()]
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
            copy_note: default_copy_note(),
        }
    }
}

#[derive(Debug)]
pub struct AppData {
    pub groups: Vec<SectionGroup>,
    pub sections: Vec<SectionConfig>,
    pub list_data: HashMap<String, Vec<ListEntry>>,
    pub checklist_data: HashMap<String, Vec<String>>,
    pub block_select_data: HashMap<String, Vec<BlockSelectEntry>>,
    pub boilerplate_texts: HashMap<String, String>,
    pub keybindings: KeyBindings,
    pub data_dir: PathBuf,
}

impl AppData {
    pub fn load(data_dir: PathBuf) -> Result<Self> {
        let base = load_data_dir(&data_dir)
            .map_err(|e| anyhow::anyhow!(e))?;
        let groups = base.groups;
        let sections = base.sections;

        let mut list_data: HashMap<String, Vec<ListEntry>> = HashMap::new();
        let mut checklist_data: HashMap<String, Vec<String>> = HashMap::new();
        let mut block_select_data: HashMap<String, Vec<BlockSelectEntry>> = HashMap::new();

        for section in &sections {
            if let Some(ref data_file) = section.data_file {
                let path = data_dir.join(data_file);
                if path.exists() {
                    let content = fs::read_to_string(&path)?;
                    match section.section_type.as_str() {
                        "list_select" => {
                            let flat: crate::flat_file::FlatFile = serde_yaml::from_str(&content)?;
                            let mut entries: Vec<ListEntry> = Vec::new();
                            for block in flat.blocks {
                                if let crate::flat_file::FlatBlock::OptionsList { entries: opts, .. } = block {
                                    for opt in opts {
                                        entries.push(ListEntry {
                                            label: opt.label().to_string(),
                                            output: opt.output().to_string(),
                                        });
                                    }
                                }
                            }
                            list_data.insert(data_file.clone(), entries);
                        }
                        "checklist" => {
                            let flat: crate::flat_file::FlatFile = serde_yaml::from_str(&content)?;
                            let mut items: Vec<String> = Vec::new();
                            for block in flat.blocks {
                                if let crate::flat_file::FlatBlock::OptionsList { entries: opts, .. } = block {
                                    for opt in opts {
                                        items.push(opt.label().to_string());
                                    }
                                }
                            }
                            checklist_data.insert(data_file.clone(), items);
                        }
                        "block_select" => {
                            let file: BlockSelectFile = serde_yaml::from_str(&content)?;
                            block_select_data.insert(data_file.clone(), file.entries);
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
            block_select_data,
            boilerplate_texts: base.boilerplate_texts,
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

#[cfg(test)]
mod rename_tests {
    use super::*;
    use std::collections::HashMap;

    // Verify BlockSelectEntry is defined and has an `entries` field of Vec<PartOption>.
    // This test will FAIL until TechniqueConfig is renamed to PartOption-based BlockSelectEntry.
    #[test]
    fn block_select_entry_exists_with_entries_field() {
        let _entry: BlockSelectEntry = BlockSelectEntry {
            id: "test-id".to_string(),
            label: "Test".to_string(),
            header: "Header".to_string(),
            entries: vec![],
        };
    }

    // Verify BlockSelectFile is defined and has an `entries` field of Vec<BlockSelectEntry>.
    // This test will FAIL until RegionsFile is renamed to BlockSelectFile.
    #[test]
    fn block_select_file_exists_with_entries_field() {
        let _file: BlockSelectFile = BlockSelectFile {
            entries: vec![],
        };
    }

    // Verify TechniqueConfig no longer exists as a public type.
    // This test will FAIL to compile if TechniqueConfig still exists,
    // because the compile-time type check approach confirms absence by expecting
    // BlockSelectEntry (the replacement) to be the canonical name.
    // Since we cannot directly assert a type does NOT exist in a passing test,
    // we assert AppData uses `block_select_data` (not `region_data`).
    #[test]
    fn app_data_has_block_select_data_not_region_data() {
        // Construct a minimal AppData manually to verify field names.
        // This will fail to compile while `region_data` exists and `block_select_data` does not.
        let _map: HashMap<String, Vec<BlockSelectEntry>> = HashMap::new();
        // The following line will only compile after `block_select_data` replaces `region_data`.
        let _check = std::mem::size_of::<AppData>();
        // Access the field name via a closure that borrows it -- compile-time proof.
        fn check_field(ad: &AppData) -> &HashMap<String, Vec<BlockSelectEntry>> {
            &ad.block_select_data
        }
        let _ = check_field;
    }
}

#[cfg(test)]
mod part_option_default_tests {
    use super::*;

    #[test]
    fn full_without_default_field_yields_default_true() {
        let yaml = "id: opt1\nlabel: Option One\noutput: out1\n";
        let parsed: PartOption = serde_yaml::from_str(yaml).expect("deserialize failed");
        match parsed {
            PartOption::Full { default, .. } => {
                assert!(default, "expected default == true when `default:` key is absent");
            }
            other => panic!("expected PartOption::Full, got {:?}", other),
        }
    }

    #[test]
    fn full_with_default_false_yields_false() {
        let yaml = "id: opt2\nlabel: Option Two\noutput: out2\ndefault: false\n";
        let parsed: PartOption = serde_yaml::from_str(yaml).expect("deserialize failed");
        match parsed {
            PartOption::Full { default, .. } => {
                assert!(!default, "expected default == false when `default: false` is set");
            }
            other => panic!("expected PartOption::Full, got {:?}", other),
        }
    }
}

#[cfg(test)]
mod tx_regions_default_tests {
    use super::*;

    // ST47-3-TEST-1: The `fascial_l4l5` entry in `back_lower_prone` (LOWER BACK Prone, index 3)
    // must start UNSELECTED (default: false) because it is rarely used per user context.
    // This test FAILS before the yml change (no `default` field means default=true -> selected).
    // It PASSES after `default: false` is added to the fascial_l4l5 entry in tx_regions.yml.
    #[test]
    fn lower_back_prone_fascial_l4l5_starts_unselected() {
        let yaml_content = include_str!("../data/tx_regions.yml");
        let file: BlockSelectFile =
            serde_yaml::from_str(yaml_content).expect("tx_regions.yml must parse as BlockSelectFile");

        let region = file
            .entries
            .iter()
            .find(|e| e.id == "back_lower_prone")
            .expect("back_lower_prone region must exist in tx_regions.yml");

        // Entries order: swedish(0), spec_comp_ql(1), muscle_strip_es(2), fascial_l4l5(3)
        let fascial_entry = region
            .entries
            .iter()
            .find(|e| e.option_id() == Some("fascial_l4l5"))
            .expect("fascial_l4l5 entry must exist in back_lower_prone");

        assert!(
            !fascial_entry.default_selected(),
            "fascial_l4l5 in LOWER BACK (Prone) must have default: false and start unselected"
        );
    }
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

fn block_type_tag(b: &crate::flat_file::FlatBlock) -> &'static str {
    use crate::flat_file::FlatBlock::*;
    match b {
        Box {..} => "box",
        Group {..} => "group",
        Section {..} => "section",
        Field {..} => "field",
        OptionsList {..} => "options-list",
        Boilerplate {..} => "boilerplate",
    }
}

fn block_id(b: &crate::flat_file::FlatBlock) -> &str {
    use crate::flat_file::FlatBlock::*;
    match b {
        Box { id, .. } | Group { id, .. } | Section { id, .. }
            | Field { id, .. } | OptionsList { id, .. }
            | Boilerplate { id, .. } => id.as_str(),
    }
}

fn block_children(b: &crate::flat_file::FlatBlock) -> &[String] {
    use crate::flat_file::FlatBlock::*;
    match b {
        Box { children, .. } | Group { children, .. } | Section { children, .. }
            | Field { children, .. } | OptionsList { children, .. } => children.as_slice(),
        Boilerplate {..} => &[],
    }
}

/// Identifies the structural level of a hierarchy node for scoped (TypeTag, id) uniqueness.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeTag {
    Template,
    Group,
    Section,
    Field,
    List,
    Item,
    Boilerplate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyItem {
    pub id: String,
    pub label: String,
    pub default: Option<bool>,
    pub output: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyList {
    pub id: String,
    pub label: Option<String>,
    pub items: Vec<HierarchyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyField {
    pub id: String,
    pub label: String,
    pub field_type: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub list_id: Option<String>,
    pub data_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchySection {
    pub id: String,
    pub nav_label: String,
    pub map_label: String,
    pub section_type: String,
    pub fields: Option<Vec<HierarchyField>>,
    pub lists: Option<Vec<HierarchyList>>,
    pub date_prefix: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyGroup {
    pub id: String,
    pub nav_label: String,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyTemplate {
    pub id: Option<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoilerplateEntry {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyFile {
    pub template: Option<HierarchyTemplate>,
    pub groups: Option<Vec<HierarchyGroup>>,
    pub sections: Option<Vec<HierarchySection>>,
    pub fields: Option<Vec<HierarchyField>>,
    pub lists: Option<Vec<HierarchyList>>,
    pub items: Option<Vec<HierarchyItem>>,
    #[serde(default)]
    pub boilerplate: Vec<BoilerplateEntry>,
}

pub fn load_data_dir(path: &Path) -> Result<AppData, String> {
    // Collect all *.yml files, skipping keybindings.yml
    let entries = fs::read_dir(path)
        .map_err(|e| format!("failed to read directory {:?}: {}", path, e))?;

    let mut pool: Vec<crate::flat_file::FlatBlock> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("directory entry error: {}", e))?;
        let file_path = entry.path();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if !file_name.ends_with(".yml") {
            continue;
        }
        if file_name == "keybindings.yml" || file_name == "config.yml" || file_name == "tx_regions.yml" {
            continue;
        }
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("failed to read {:?}: {}", file_path, e))?;
        let flat_file: FlatFile = serde_yaml::from_str(&content)
            .map_err(|e| format!("parse error in {:?}: {}", file_path, e))?;
        pool.extend(flat_file.blocks);
    }

    // Duplicate check: (type_tag, id) must be unique
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for block in &pool {
        let key = (block_type_tag(block).to_string(), block_id(block).to_string());
        if !seen.insert(key) {
            return Err(format!(
                "duplicate block: type={} id={}",
                block_type_tag(block),
                block_id(block)
            ));
        }
    }

    // Build id -> index lookup (any type) for reference resolution
    let mut id_map: HashMap<String, usize> = HashMap::new();
    for (i, block) in pool.iter().enumerate() {
        id_map.insert(block_id(block).to_string(), i);
    }

    // Missing-ref check: every child ID must exist in the pool
    for block in &pool {
        for child_id in block_children(block) {
            if !id_map.contains_key(child_id.as_str()) {
                return Err(format!(
                    "block {:?} references unknown child id {:?}",
                    block_id(block),
                    child_id
                ));
            }
        }
    }

    // Cycle detection: standard DFS with visited/in-stack sets
    let n = pool.len();
    let mut visited: HashSet<usize> = HashSet::new();
    let mut in_stack: HashSet<usize> = HashSet::new();

    fn dfs(
        node: usize,
        pool: &[crate::flat_file::FlatBlock],
        id_map: &HashMap<String, usize>,
        visited: &mut HashSet<usize>,
        in_stack: &mut HashSet<usize>,
    ) -> Result<(), String> {
        if in_stack.contains(&node) {
            return Err(format!("cycle detected at block id={}", block_id(&pool[node])));
        }
        if visited.contains(&node) {
            return Ok(());
        }
        visited.insert(node);
        in_stack.insert(node);
        for child_id in block_children(&pool[node]) {
            if let Some(&child_idx) = id_map.get(child_id.as_str()) {
                dfs(child_idx, pool, id_map, visited, in_stack)?;
            }
        }
        in_stack.remove(&node);
        Ok(())
    }

    for i in 0..n {
        dfs(i, &pool, &id_map, &mut visited, &mut in_stack)?;
    }

    // Reconstruction pass: walk Group blocks to build runtime SectionGroup/SectionConfig.
    let mut groups: Vec<SectionGroup> = Vec::new();
    let mut all_sections: Vec<SectionConfig> = Vec::new();

    for block in &pool {
        if let crate::flat_file::FlatBlock::Group { id, name, num, children, .. } = block {
            let mut group_sections: Vec<SectionConfig> = Vec::new();
            for child_id in children {
                let Some(&sec_idx) = id_map.get(child_id.as_str()) else { continue };
                if let crate::flat_file::FlatBlock::Section {
                    id: sid, name: sname, map_label, section_type,
                    data_file, date_prefix, children: field_ids,
                    is_intake, heading_search_text, heading_label, note_render_slot,
                } = &pool[sec_idx] {
                    let fields = if section_type.as_deref() == Some("multi_field") {
                        let mut hfields: Vec<HeaderFieldConfig> = Vec::new();
                        for fid in field_ids {
                            let Some(&fidx) = id_map.get(fid.as_str()) else { continue };
                            if let crate::flat_file::FlatBlock::Field {
                                id: field_id, name: field_name,
                                options, composite, default, repeat_limit, ..
                            } = &pool[fidx] {
                                hfields.push(HeaderFieldConfig {
                                    id: field_id.clone(),
                                    name: field_name.clone().unwrap_or_default(),
                                    options: options.clone(),
                                    composite: composite.clone(),
                                    default: default.clone(),
                                    repeat_limit: *repeat_limit,
                                });
                            }
                        }
                        Some(hfields)
                    } else {
                        None
                    };
                    let sc = SectionConfig {
                        id: sid.clone(),
                        name: sname.clone().unwrap_or_default(),
                        map_label: map_label.clone().unwrap_or_default(),
                        section_type: section_type.clone().unwrap_or_default(),
                        data_file: data_file.clone(),
                        date_prefix: *date_prefix,
                        options: vec![],
                        composite: None,
                        fields,
                        is_intake: *is_intake,
                        heading_search_text: heading_search_text.clone(),
                        heading_label: heading_label.clone(),
                        note_render_slot: note_render_slot.clone(),
                    };
                    group_sections.push(sc.clone());
                    all_sections.push(sc);
                }
            }
            groups.push(SectionGroup {
                id: id.clone(),
                name: name.clone().unwrap_or_default(),
                num: *num,
                sections: group_sections,
            });
        }
    }

    // Boilerplate extraction: collect id -> text.
    // Duplicate ids are already rejected by the general duplicate check above.
    let mut boilerplate_texts: HashMap<String, String> = HashMap::new();
    for block in &pool {
        if let crate::flat_file::FlatBlock::Boilerplate { id, text } = block {
            boilerplate_texts.insert(id.clone(), text.clone());
        }
    }

    Ok(AppData {
        groups,
        sections: all_sections,
        list_data: HashMap::new(),
        checklist_data: HashMap::new(),
        block_select_data: HashMap::new(),
        boilerplate_texts,
        keybindings: KeyBindings::default(),
        data_dir: path.to_path_buf(),
    })
}

pub fn load_hierarchy_dir(dir: &std::path::Path) -> Result<HierarchyFile, String> {
    // --- Phase 1: scan and parse ---
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory {:?}: {}", dir, e))?;

    let mut merged = HierarchyFile {
        template: None,
        groups: None,
        sections: None,
        fields: None,
        lists: None,
        items: None,
        boilerplate: Vec::new(),
    };
    let mut template_count = 0usize;

    for entry in entries {
        let entry = entry.map_err(|e| format!("directory entry error: {}", e))?;
        let file_path = entry.path();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if !file_name.ends_with(".yml") {
            continue;
        }
        if file_name == "keybindings.yml" || file_name == "config.yml" {
            continue;
        }
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("failed to read {:?}: {}", file_path, e))?;
        let hf: HierarchyFile = serde_yaml::from_str(&content)
            .map_err(|e| format!("parse error in {:?}: {}", file_path, e))?;

        // Merge template (count occurrences for cardinality check)
        if hf.template.is_some() {
            template_count += 1;
            if merged.template.is_none() {
                merged.template = hf.template;
            }
        }
        // Merge Option<Vec<>> fields
        if let Some(v) = hf.groups {
            merged.groups.get_or_insert_with(Vec::new).extend(v);
        }
        if let Some(v) = hf.sections {
            merged.sections.get_or_insert_with(Vec::new).extend(v);
        }
        if let Some(v) = hf.fields {
            merged.fields.get_or_insert_with(Vec::new).extend(v);
        }
        if let Some(v) = hf.lists {
            merged.lists.get_or_insert_with(Vec::new).extend(v);
        }
        if let Some(v) = hf.items {
            merged.items.get_or_insert_with(Vec::new).extend(v);
        }
        merged.boilerplate.extend(hf.boilerplate);
    }

    // --- Phase 2: template cardinality ---
    match template_count {
        0 => return Err("no template defined: exactly 1 template is required across all hierarchy files".to_string()),
        1 => {}
        n => return Err(format!("multiple templates defined: found {}, expected exactly 1", n)),
    }

    // --- Phase 3: typed ID uniqueness ---
    let mut seen: HashSet<(TypeTag, String)> = HashSet::new();
    for g in merged.groups.as_deref().unwrap_or(&[]) {
        let key = (TypeTag::Group, g.id.clone());
        if !seen.insert(key) {
            return Err(format!("duplicate group id: {}", g.id));
        }
    }
    for s in merged.sections.as_deref().unwrap_or(&[]) {
        let key = (TypeTag::Section, s.id.clone());
        if !seen.insert(key) {
            return Err(format!("duplicate section id: {}", s.id));
        }
    }
    for f in merged.fields.as_deref().unwrap_or(&[]) {
        let key = (TypeTag::Field, f.id.clone());
        if !seen.insert(key) {
            return Err(format!("duplicate field id: {}", f.id));
        }
    }
    for l in merged.lists.as_deref().unwrap_or(&[]) {
        let key = (TypeTag::List, l.id.clone());
        if !seen.insert(key) {
            return Err(format!("duplicate list id: {}", l.id));
        }
    }

    // --- Phase 4: boilerplate ID uniqueness ---
    let mut bp_seen: HashSet<String> = HashSet::new();
    for bp in &merged.boilerplate {
        if !bp_seen.insert(bp.id.clone()) {
            return Err(format!("duplicate boilerplate id: {}", bp.id));
        }
    }

    // --- Phase 5: cross-reference validation ---
    // Build typed lookup sets for O(1) existence checks
    let group_ids: HashSet<&str> = merged.groups.as_deref().unwrap_or(&[])
        .iter().map(|g| g.id.as_str()).collect();
    let section_ids: HashSet<&str> = merged.sections.as_deref().unwrap_or(&[])
        .iter().map(|s| s.id.as_str()).collect();
    let _field_ids: HashSet<&str> = merged.fields.as_deref().unwrap_or(&[])
        .iter().map(|f| f.id.as_str()).collect();
    let list_ids: HashSet<&str> = merged.lists.as_deref().unwrap_or(&[])
        .iter().map(|l| l.id.as_str()).collect();

    // Template -> group refs
    let template = merged.template.as_ref().unwrap(); // safe: cardinality already checked
    for gref in &template.groups {
        if !group_ids.contains(gref.as_str()) {
            return Err(format!("unresolved template group ref: {}", gref));
        }
    }
    // Group -> section refs
    for g in merged.groups.as_deref().unwrap_or(&[]) {
        for sref in &g.sections {
            if !section_ids.contains(sref.as_str()) {
                return Err(format!("unresolved section ref '{}' in group '{}'", sref, g.id));
            }
        }
    }
    // Field -> list_id ref
    for f in merged.fields.as_deref().unwrap_or(&[]) {
        if let Some(ref lid) = f.list_id {
            if !list_ids.contains(lid.as_str()) {
                return Err(format!("unresolved list_id ref '{}' in field '{}'", lid, f.id));
            }
        }
    }

    // --- Phase 6: DFS cycle detection over group->section->field->list refs ---
    fn dfs_hier(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
    ) -> Result<(), String> {
        if in_stack.contains(node) {
            return Err(format!("cycle detected at node id={}", node));
        }
        if visited.contains(node) {
            return Ok(());
        }
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());
        if let Some(children) = adj.get(node) {
            for child in children {
                dfs_hier(child, adj, visited, in_stack)?;
            }
        }
        in_stack.remove(node);
        Ok(())
    }

    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for g in merged.groups.as_deref().unwrap_or(&[]) {
        adj.entry(g.id.clone()).or_default().extend(g.sections.iter().cloned());
    }
    for f in merged.fields.as_deref().unwrap_or(&[]) {
        if let Some(ref lid) = f.list_id {
            adj.entry(f.id.clone()).or_default().push(lid.clone());
        }
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut in_stack: HashSet<String> = HashSet::new();
    for g in merged.groups.as_deref().unwrap_or(&[]) {
        dfs_hier(&g.id, &adj, &mut visited, &mut in_stack)?;
    }
    for f in merged.fields.as_deref().unwrap_or(&[]) {
        dfs_hier(&f.id, &adj, &mut visited, &mut in_stack)?;
    }

    Ok(merged)
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
                is_intake: false,
                heading_search_text: None,
                heading_label: None,
                note_render_slot: None,
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

    // ---- sections.yml migration test (Task #45 sub-task 4) ----
    //
    // This test verifies that the real data/sections.yml (and all other *.yml
    // files in the data directory) can be parsed by load_data_dir as flat-format
    // blocks.  It FAILS until the migration is complete because sections.yml is
    // still in the old nested `groups:` format, which is not a valid FlatFile.
    //
    // When sections.yml has been fully migrated the test will pass.

    /// The real data directory must load without errors after migration.
    ///
    /// Failure mode before migration: serde_yaml returns a parse error because
    /// sections.yml starts with `groups:` instead of `blocks:`.
    #[test]
    fn real_data_dir_loads_as_flat_format() {
        // Locate the project's data/ directory relative to CARGO_MANIFEST_DIR
        // so the test is not affected by the working directory at test-run time.
        let manifest_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR must be set when running cargo test"),
        );
        let data_dir = manifest_dir.join("data");

        assert!(
            data_dir.exists(),
            "data directory not found at {:?}",
            data_dir
        );

        let result = load_data_dir(&data_dir);
        assert!(
            result.is_ok(),
            "load_data_dir on the real data directory failed (migration not yet complete?): {}",
            result.unwrap_err()
        );
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
                        is_intake: false,
                        heading_search_text: None,
                        heading_label: None,
                        note_render_slot: None,
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

    // ---- load_data_dir tests (Task #45 sub-task 2) ----
    //
    // These tests verify the new flat-file loader that replaces the old
    // SectionsFile nested-struct path.  The function under test is:
    //
    //   pub fn load_data_dir(path: &std::path::Path) -> Result<AppData, String>
    //
    // It scans `path` for all *.yml files, deserialises each as a
    // Vec<FlatBlock> (via FlatFile), merges them into a single pool, resolves
    // parent->children references, detects cycles and duplicate IDs, and
    // returns AppData.  The function does NOT yet exist - these tests are
    // written first so that they fail until the implementation is added.

    use std::path::{Path, PathBuf};

    /// Create a unique temp subdirectory under the system temp folder.
    /// Returns the path; the caller is responsible for cleanup (best-effort).
    fn make_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("scribblenot_tests")
            .join(name);
        std::fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    /// Remove the test directory after the test (best-effort, never panics).
    fn cleanup_test_dir(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Write content into `dir/name`.
    fn write_yml(dir: &Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).expect("write yml");
    }

    #[test]
    fn load_data_dir_returns_app_data_for_valid_directory() {
        // A directory with a single valid flat-block yml should produce AppData.
        let dir = make_test_dir("valid_single");
        write_yml(
            &dir,
            "forms.yml",
            "blocks:\n  - type: box\n    id: root_box\n  - type: section\n    id: sec_a\n  - type: field\n    id: fld_a\n",
        );
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_ok(),
            "expected Ok(AppData) for valid directory, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn load_data_dir_merges_blocks_from_multiple_yml_files() {
        // Blocks across two files must both appear in the merged pool.
        let dir = make_test_dir("multi_file");
        write_yml(&dir, "file_a.yml", "blocks:\n  - type: box\n    id: box_a\n");
        write_yml(&dir, "file_b.yml", "blocks:\n  - type: section\n    id: sec_b\n");
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(result.is_ok(), "merge of two valid files should succeed");
    }

    #[test]
    fn load_data_dir_errors_on_duplicate_id_and_type() {
        // Two blocks with the same id AND the same type must produce an error.
        let dir = make_test_dir("dupe_same_file");
        write_yml(
            &dir,
            "dupe.yml",
            "blocks:\n  - type: section\n    id: duplicated_id\n  - type: section\n    id: duplicated_id\n",
        );
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected an error for duplicate id+type combination"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("duplicated_id"),
            "error message should mention the duplicate id; got: {err_msg}"
        );
    }

    #[test]
    fn load_data_dir_errors_on_duplicate_id_and_type_across_files() {
        // Cross-file duplicates must also be caught.
        let dir = make_test_dir("dupe_cross_file");
        write_yml(&dir, "alpha.yml", "blocks:\n  - type: field\n    id: shared_id\n");
        write_yml(&dir, "beta.yml", "blocks:\n  - type: field\n    id: shared_id\n");
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_err(),
            "cross-file duplicate id+type should produce an error"
        );
    }

    #[test]
    fn load_data_dir_allows_same_id_different_type() {
        // Same id but different types is NOT a duplicate.
        let dir = make_test_dir("same_id_diff_type");
        write_yml(
            &dir,
            "ok.yml",
            "blocks:\n  - type: field\n    id: shared_name\n  - type: section\n    id: shared_name\n",
        );
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_ok(),
            "same id with different types should be allowed; got: {:?}",
            result.err()
        );
    }

    #[test]
    fn load_data_dir_errors_on_missing_child_id_reference() {
        // A children list that references an ID not in the pool must error.
        let dir = make_test_dir("missing_child");
        write_yml(
            &dir,
            "missing_ref.yml",
            "blocks:\n  - type: box\n    id: parent_box\n    children:\n      - nonexistent_child_id\n",
        );
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected an error when a children reference points to a missing id"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("nonexistent_child_id"),
            "error message should mention the missing id; got: {err_msg}"
        );
    }

    #[test]
    fn load_data_dir_errors_on_direct_cycle() {
        // A -> B -> A is a cycle and must produce an error.
        let dir = make_test_dir("direct_cycle");
        write_yml(
            &dir,
            "cycle.yml",
            "blocks:\n  - type: box\n    id: node_a\n    children:\n      - node_b\n  - type: box\n    id: node_b\n    children:\n      - node_a\n",
        );
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected an error for a direct cycle between node_a and node_b"
        );
    }

    #[test]
    fn load_data_dir_errors_on_indirect_cycle() {
        // A -> B -> C -> A must also produce an error.
        let dir = make_test_dir("indirect_cycle");
        write_yml(
            &dir,
            "long_cycle.yml",
            "blocks:\n  - type: box\n    id: cx_a\n    children:\n      - cx_b\n  - type: box\n    id: cx_b\n    children:\n      - cx_c\n  - type: box\n    id: cx_c\n    children:\n      - cx_a\n",
        );
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected an error for an indirect 3-node cycle"
        );
    }

    #[test]
    fn load_data_dir_accepts_acyclic_tree() {
        // A -> B -> C with no back-edges should succeed.
        let dir = make_test_dir("acyclic_tree");
        write_yml(
            &dir,
            "tree.yml",
            "blocks:\n  - type: box\n    id: tree_root\n    children:\n      - tree_child\n  - type: section\n    id: tree_child\n    children:\n      - tree_leaf\n  - type: field\n    id: tree_leaf\n",
        );
        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_ok(),
            "acyclic tree should be accepted; got: {:?}",
            result.err()
        );
    }

    // ---- reconstruction pass tests (Task #45 sub-task 3) ----
    //
    // These tests verify that load_data_dir performs the reconstruction pass:
    // after validation it must walk Group blocks to build Vec<SectionGroup>,
    // resolve children IDs to Section blocks -> SectionConfig values, and
    // resolve each Section's children to Field blocks.
    //
    // Both tests FAIL until the reconstruction pass is implemented because
    // load_data_dir currently returns AppData { groups: vec![], sections: vec![] }.

    /// After loading the real data directory, AppData.groups must be non-empty.
    ///
    /// Failure mode before implementation: load_data_dir returns groups: vec![]
    /// even though the data files contain Group blocks.
    #[test]
    fn real_data_dir_has_non_empty_groups() {
        let manifest_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR must be set when running cargo test"),
        );
        let data_dir = manifest_dir.join("data");

        assert!(data_dir.exists(), "data directory not found at {:?}", data_dir);

        let result = load_data_dir(&data_dir);
        assert!(
            result.is_ok(),
            "load_data_dir on the real data directory failed: {}",
            result.unwrap_err()
        );
        let app_data = result.unwrap();
        assert!(
            app_data.groups.len() > 0,
            "expected groups.len() > 0 after reconstruction pass, got {}",
            app_data.groups.len()
        );
    }

    /// After loading the real data directory, AppData.sections must be non-empty.
    ///
    /// Failure mode before implementation: load_data_dir returns sections: vec![]
    /// even though the data files contain Section blocks.
    #[test]
    fn real_data_dir_has_non_empty_sections() {
        let manifest_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR must be set when running cargo test"),
        );
        let data_dir = manifest_dir.join("data");

        assert!(data_dir.exists(), "data directory not found at {:?}", data_dir);

        let result = load_data_dir(&data_dir);
        assert!(
            result.is_ok(),
            "load_data_dir on the real data directory failed: {}",
            result.unwrap_err()
        );
        let app_data = result.unwrap();
        assert!(
            app_data.sections.len() > 0,
            "expected sections.len() > 0 after reconstruction pass, got {}",
            app_data.sections.len()
        );
    }

    /// Hybrid inline+ID-reference: a parent block whose children list mixes IDs defined
    /// in the same file (co-located, "inline" in spirit) with IDs defined in a separate
    /// file (cross-file ID reference).  All three must resolve correctly.
    ///
    /// File layout:
    ///   hybrid_parent.yml - defines `hybrid_root` (box) with children [local_child, remote_child]
    ///                       and `local_child` (section) - co-located with its parent
    ///   hybrid_remote.yml - defines `remote_child` (field) - referenced by ID from the other file
    ///
    /// Expected: Ok(AppData) - the loader merges both files, resolves all three IDs, and
    /// confirms the reference graph is acyclic.
    #[test]
    fn load_data_dir_hybrid_inline_and_cross_file_id_reference_resolves_correctly() {
        let dir = make_test_dir("hybrid_inline_crossfile");

        // Parent file: root block + one child defined in the same file
        write_yml(
            &dir,
            "hybrid_parent.yml",
            "blocks:\n  - type: box\n    id: hybrid_root\n    children:\n      - local_child\n      - remote_child\n  - type: section\n    id: local_child\n",
        );

        // Remote file: the other child, defined separately
        write_yml(
            &dir,
            "hybrid_remote.yml",
            "blocks:\n  - type: field\n    id: remote_child\n",
        );

        let result = load_data_dir(&dir);
        cleanup_test_dir(&dir);
        assert!(
            result.is_ok(),
            "hybrid inline+cross-file ID reference should resolve correctly; got: {:?}",
            result.err()
        );
    }
}

pub fn find_data_dir() -> PathBuf {
    // Try cwd/data first (development)
    let cwd_data = std::env::current_dir()
        .unwrap_or_default()
        .join("data");
    if cwd_data.exists() && cwd_data.is_dir() {
        return cwd_data;
    }

    // Try exe parent/data
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_data = parent.join("data");
            if exe_data.exists() && exe_data.is_dir() {
                return exe_data;
            }
        }
    }

    // Fallback to cwd/data
    cwd_data
}

#[cfg(test)]
mod boilerplate_load_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn make_bp_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("scribblenot_bp_tests")
            .join(name);
        std::fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    fn cleanup_bp_test_dir(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    fn write_bp_yml(dir: &Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).expect("write yml");
    }

    // ST52-3-TEST-1: AppData must have a `boilerplate_texts` field of type
    // HashMap<String, String>. This test fails to compile until the field is added.
    #[test]
    fn app_data_has_boilerplate_texts_field() {
        fn check_field(ad: &AppData) -> &HashMap<String, String> {
            &ad.boilerplate_texts
        }
        let _ = check_field;
    }

    // ST52-3-TEST-2: After loading a directory containing a boilerplate.yml with
    // the treatment_plan_disclaimer block, boilerplate_texts must contain that key
    // with the correct text.
    #[test]
    fn boilerplate_texts_contains_treatment_plan_disclaimer() {
        let dir = make_bp_test_dir("bp_disclaimer");
        write_bp_yml(
            &dir,
            "boilerplate.yml",
            "blocks:\n  - type: boilerplate\n    id: treatment_plan_disclaimer\n    text: |\n      Regions and locations are bilateral unless indicated otherwise.\n      Patient is pillowed under ankles when prone, and under knees when supine.\n",
        );
        let result = load_data_dir(&dir);
        cleanup_bp_test_dir(&dir);
        let app_data = result.expect("load_data_dir should succeed with valid boilerplate yml");
        assert!(
            app_data.boilerplate_texts.contains_key("treatment_plan_disclaimer"),
            "boilerplate_texts must contain treatment_plan_disclaimer key"
        );
        let text = &app_data.boilerplate_texts["treatment_plan_disclaimer"];
        assert!(
            text.contains("bilateral unless indicated otherwise"),
            "treatment_plan_disclaimer text must contain expected content; got: {text:?}"
        );
        assert!(
            text.contains("pillowed under ankles when prone"),
            "treatment_plan_disclaimer text must contain expected content; got: {text:?}"
        );
    }

    // ST52-3-TEST-3: After loading a directory containing a boilerplate.yml with
    // the informed_consent block, boilerplate_texts must contain that key with the
    // correct text.
    #[test]
    fn boilerplate_texts_contains_informed_consent() {
        let dir = make_bp_test_dir("bp_consent");
        write_bp_yml(
            &dir,
            "boilerplate.yml",
            "blocks:\n  - type: boilerplate\n    id: informed_consent\n    text: Patient has been informed of the risks and benefits of massage therapy, and has given informed consent to assessment and treatment.\n",
        );
        let result = load_data_dir(&dir);
        cleanup_bp_test_dir(&dir);
        let app_data = result.expect("load_data_dir should succeed with valid boilerplate yml");
        assert!(
            app_data.boilerplate_texts.contains_key("informed_consent"),
            "boilerplate_texts must contain informed_consent key"
        );
        let text = &app_data.boilerplate_texts["informed_consent"];
        assert!(
            text.contains("informed consent to assessment and treatment"),
            "informed_consent text must contain expected content; got: {text:?}"
        );
    }

    // ST52-3-TEST-4: Loading a directory with two boilerplate blocks sharing the
    // same id must return an error (duplicate detection).
    #[test]
    fn load_data_dir_errors_on_duplicate_boilerplate_id() {
        let dir = make_bp_test_dir("bp_dupe");
        write_bp_yml(
            &dir,
            "boilerplate.yml",
            "blocks:\n  - type: boilerplate\n    id: duplicate_bp\n    text: First text.\n  - type: boilerplate\n    id: duplicate_bp\n    text: Second text.\n",
        );
        let result = load_data_dir(&dir);
        cleanup_bp_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected an error for duplicate boilerplate id, but got Ok"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("duplicate_bp"),
            "error message should mention the duplicate id; got: {err_msg}"
        );
    }
}

#[cfg(test)]
mod header_field_repeat_limit_tests {
    use super::*;

    // ST49-1-TEST-1: HeaderFieldConfig with repeat_limit: 5 deserializes to Some(5)
    #[test]
    fn repeat_limit_some_when_present() {
        let yaml = "id: foo\nname: Foo\nrepeat_limit: 5\n";
        let cfg: HeaderFieldConfig = serde_yaml::from_str(yaml)
            .expect("should deserialize HeaderFieldConfig with repeat_limit");
        assert_eq!(
            cfg.repeat_limit,
            Some(5),
            "repeat_limit should be Some(5) when specified as 5 in YAML"
        );
    }

    // ST49-1-TEST-2: HeaderFieldConfig without repeat_limit deserializes to None
    #[test]
    fn repeat_limit_none_when_absent() {
        let yaml = "id: bar\nname: Bar\n";
        let cfg: HeaderFieldConfig = serde_yaml::from_str(yaml)
            .expect("should deserialize HeaderFieldConfig without repeat_limit");
        assert_eq!(
            cfg.repeat_limit,
            None,
            "repeat_limit should be None when not specified in YAML"
        );
    }

    // ST49-1-TEST-3: repeat_limit: 0 is valid and deserializes to Some(0)
    #[test]
    fn repeat_limit_some_zero_when_zero() {
        let yaml = "id: baz\nname: Baz\nrepeat_limit: 0\n";
        let cfg: HeaderFieldConfig = serde_yaml::from_str(yaml)
            .expect("should deserialize HeaderFieldConfig with repeat_limit 0");
        assert_eq!(
            cfg.repeat_limit,
            Some(0),
            "repeat_limit should be Some(0) when specified as 0 in YAML"
        );
    }
}

#[cfg(test)]
mod tx_mods_multi_field_tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        let manifest_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR must be set when running cargo test"),
        );
        manifest_dir.join("data")
    }

    fn load() -> AppData {
        let dir = data_dir();
        load_data_dir(&dir).expect("load_data_dir on real data directory must succeed")
    }

    fn find_tx_mods(app: &AppData) -> &SectionConfig {
        app.sections
            .iter()
            .find(|s| s.id == "tx_mods")
            .expect("tx_mods section must exist in sections")
    }

    // ST50-1-TEST-1: tx_mods section_type must be "multi_field" after sub-task 1.
    // FAILS before implementation because sections.yml still has section_type: list_select.
    #[test]
    fn tx_mods_section_type_is_multi_field() {
        let app = load();
        let sec = find_tx_mods(&app);
        assert_eq!(
            sec.section_type, "multi_field",
            "tx_mods section_type must be 'multi_field' after sub-task 1 implementation; \
             got '{}'",
            sec.section_type
        );
    }

    // ST50-1-TEST-2: tx_mods must have exactly 5 field children after sub-task 1.
    // FAILS before implementation because the section has no fields (it uses data_file instead).
    #[test]
    fn tx_mods_has_five_field_children() {
        let app = load();
        let sec = find_tx_mods(&app);
        let fields = sec
            .fields
            .as_ref()
            .expect("tx_mods.fields must be Some(Vec<HeaderFieldConfig>) after implementation");
        assert_eq!(
            fields.len(),
            5,
            "tx_mods must have exactly 5 inline field children \
             (pressure, challenge, mood, communication, modifications); got {}",
            fields.len()
        );
    }

    // ST50-1-TEST-3: the 'modifications' field in tx_mods must have repeat_limit: Some(10).
    // FAILS before implementation because FlatBlock::Field has no repeat_limit field
    // and the loader hardcodes repeat_limit: None for all fields.
    #[test]
    fn tx_mods_modifications_field_has_repeat_limit_10() {
        let app = load();
        let sec = find_tx_mods(&app);
        let fields = sec
            .fields
            .as_ref()
            .expect("tx_mods.fields must be Some after implementation");
        let modifications = fields
            .iter()
            .find(|f| f.id == "modifications")
            .expect("a field with id 'modifications' must exist in tx_mods fields");
        assert_eq!(
            modifications.repeat_limit,
            Some(10),
            "tx_mods modifications field must have repeat_limit: Some(10); \
             got {:?}",
            modifications.repeat_limit
        );
    }

    // ST50-1-TEST-4: the 'modifications' field options must include the value "PREGNANCY".
    // FAILS before implementation because the field doesn't exist yet.
    // The options are simple strings matching the label values from tx_mods.yml.
    #[test]
    fn tx_mods_modifications_field_contains_pregnancy_option() {
        let app = load();
        let sec = find_tx_mods(&app);
        let fields = sec
            .fields
            .as_ref()
            .expect("tx_mods.fields must be Some after implementation");
        let modifications = fields
            .iter()
            .find(|f| f.id == "modifications")
            .expect("a field with id 'modifications' must exist in tx_mods fields");
        let has_pregnancy = modifications
            .options
            .iter()
            .any(|o| o.contains("PREGNANCY"));
        assert!(
            has_pregnancy,
            "tx_mods modifications field options must include an entry containing 'PREGNANCY'; \
             options found: {:?}",
            modifications.options
        );
    }

    // ST50-3-TEST-1: the tx_mods SectionConfig must NOT have a data_file field set.
    // This verifies that the migration to inline fields is complete and data/tx_mods.yml
    // has been properly removed -- any remaining data_file reference would mean the
    // migration is incomplete.
    // FAILS if tx_mods still has a data_file value pointing to the deleted file.
    #[test]
    fn tx_mods_section_has_no_data_file() {
        let app = load();
        let sec = find_tx_mods(&app);
        assert!(
            sec.data_file.is_none(),
            "tx_mods section must NOT have a data_file set after migration to inline fields; \
             data/tx_mods.yml was deleted and all options are now inline. \
             Found data_file: {:?}",
            sec.data_file
        );
    }

    // ST50-3-TEST-2: PREGNANCY option must be present as an inline field option in sections.yml,
    // not loaded from an external file. This verifies the migration moved all options inline
    // and the now-deleted data/tx_mods.yml is no longer needed as a source.
    // FAILS if the modifications field is missing or PREGNANCY is absent from inline options.
    #[test]
    fn tx_mods_pregnancy_option_is_inline_not_from_external_file() {
        let app = load();
        let sec = find_tx_mods(&app);

        // The section must NOT use a data_file (external file was deleted).
        assert!(
            sec.data_file.is_none(),
            "tx_mods must not reference an external data file; migration to inline is incomplete"
        );

        // The PREGNANCY option must be reachable directly from the inline fields.
        let fields = sec
            .fields
            .as_ref()
            .expect("tx_mods.fields must be Some -- inline fields are required after migration");
        let modifications = fields
            .iter()
            .find(|f| f.id == "modifications")
            .expect("'modifications' field must exist as an inline child of tx_mods");
        let has_pregnancy = modifications
            .options
            .iter()
            .any(|o| o.contains("PREGNANCY"));
        assert!(
            has_pregnancy,
            "PREGNANCY option must be present in the inline 'modifications' field options \
             (not loaded from the deleted data/tx_mods.yml); \
             inline options found: {:?}",
            modifications.options
        );
    }

    // ST50-4-TEST-1: the communication field must contain exactly two STOIC entries.
    // STOIC intentionally appears twice: one entry for pts who suppress pain responses
    // and one for pts who respond well to frequent verbal check-ins. Both are distinct
    // in the full option text even though they start with "- STOIC:".
    // FAILS if communication only has one STOIC or the two entries are collapsed into one.
    #[test]
    fn communication_has_exactly_two_stoic_entries() {
        let app = load();
        let sec = find_tx_mods(&app);
        let fields = sec
            .fields
            .as_ref()
            .expect("tx_mods.fields must be Some after implementation");
        let communication = fields
            .iter()
            .find(|f| f.id == "communication")
            .expect("a field with id 'communication' must exist in tx_mods fields");
        let stoic_count = communication
            .options
            .iter()
            .filter(|o| o.contains("STOIC"))
            .count();
        assert_eq!(
            stoic_count, 2,
            "communication field must have exactly 2 STOIC entries \
             (one for pain-suppressing pts, one for frequent check-in pts); \
             found {} entry/entries. Options: {:?}",
            stoic_count, communication.options
        );
    }

    // ST50-4-TEST-2: pressure, challenge, mood, and communication fields must be single-select
    // (repeat_limit: None). Only the modifications field has repeat_limit: Some(10).
    // FAILS if any of these four fields accidentally have repeat_limit set.
    #[test]
    fn single_select_fields_have_no_repeat_limit() {
        let app = load();
        let sec = find_tx_mods(&app);
        let fields = sec
            .fields
            .as_ref()
            .expect("tx_mods.fields must be Some after implementation");
        for field_id in &["pressure", "challenge", "mood", "communication"] {
            let field = fields
                .iter()
                .find(|f| &f.id.as_str() == field_id)
                .unwrap_or_else(|| panic!("field '{}' must exist in tx_mods fields", field_id));
            assert_eq!(
                field.repeat_limit, None,
                "field '{}' must be single-select (repeat_limit: None); \
                 only 'modifications' should have a repeat_limit. Got: {:?}",
                field_id, field.repeat_limit
            );
        }
    }

    // ST50-4-TEST-3: tx_mods must have exactly the 5 expected field IDs in order:
    // pressure, challenge, mood, communication, modifications.
    // FAILS if any field is missing, misspelled, or in wrong order.
    #[test]
    fn tx_mods_field_ids_are_correct() {
        let app = load();
        let sec = find_tx_mods(&app);
        let fields = sec
            .fields
            .as_ref()
            .expect("tx_mods.fields must be Some after implementation");
        let ids: Vec<&str> = fields.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["pressure", "challenge", "mood", "communication", "modifications"],
            "tx_mods field IDs must be exactly ['pressure', 'challenge', 'mood', \
             'communication', 'modifications'] in that order; got {:?}",
            ids
        );
    }
}

#[cfg(test)]
mod section_metadata_fields_tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        let manifest_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR must be set when running cargo test"),
        );
        manifest_dir.join("data")
    }

    fn load() -> AppData {
        let dir = data_dir();
        load_data_dir(&dir).expect("load_data_dir on real data directory must succeed")
    }

    fn find_section<'a>(app: &'a AppData, id: &str) -> &'a SectionConfig {
        app.sections
            .iter()
            .find(|s| s.id == id)
            .unwrap_or_else(|| panic!("section with id '{}' must exist in sections", id))
    }

    // ST51-1-TEST-1: the 'adl' section must have is_intake == true.
    // FAILS before implementation because SectionConfig does not have an is_intake field yet.
    #[test]
    fn adl_is_intake_is_true() {
        let app = load();
        let sec = find_section(&app, "adl");
        assert!(
            sec.is_intake,
            "adl section must have is_intake == true after sub-task 51.1 implementation; \
             SectionConfig.is_intake field does not exist yet"
        );
    }

    // ST51-1-TEST-2: the 'tx_mods' section must have heading_search_text == Some("TREATMENT MODIFICATIONS").
    // FAILS before implementation because SectionConfig does not have a heading_search_text field yet.
    #[test]
    fn tx_mods_heading_search_text_is_set() {
        let app = load();
        let sec = find_section(&app, "tx_mods");
        assert_eq!(
            sec.heading_search_text,
            Some("TREATMENT MODIFICATIONS".to_string()),
            "tx_mods section must have heading_search_text == Some(\"TREATMENT MODIFICATIONS\") \
             after sub-task 51.1 implementation; \
             SectionConfig.heading_search_text field does not exist yet. Got: {:?}",
            sec.heading_search_text
        );
    }

    // ST51-1-TEST-3: the 'adl' section must have heading_label == Some("#### ACTIVITIES OF DAILY LIVING").
    // FAILS before implementation because SectionConfig does not have a heading_label field yet.
    #[test]
    fn adl_heading_label_is_set() {
        let app = load();
        let sec = find_section(&app, "adl");
        assert_eq!(
            sec.heading_label,
            Some("#### ACTIVITIES OF DAILY LIVING".to_string()),
            "adl section must have heading_label == Some(\"#### ACTIVITIES OF DAILY LIVING\") \
             after sub-task 51.1 implementation; \
             SectionConfig.heading_label field does not exist yet. Got: {:?}",
            sec.heading_label
        );
    }

    // ST51-1-TEST-4: the 'tx_mods' section must have note_render_slot == Some("tx_mods").
    // FAILS before implementation because SectionConfig does not have a note_render_slot field yet.
    #[test]
    fn tx_mods_note_render_slot_is_set() {
        let app = load();
        let sec = find_section(&app, "tx_mods");
        assert_eq!(
            sec.note_render_slot,
            Some("tx_mods".to_string()),
            "tx_mods section must have note_render_slot == Some(\"tx_mods\") \
             after sub-task 51.1 implementation; \
             SectionConfig.note_render_slot field does not exist yet. Got: {:?}",
            sec.note_render_slot
        );
    }
}

#[cfg(test)]
mod section_metadata_complete_tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        let manifest_dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR must be set when running cargo test"),
        );
        manifest_dir.join("data")
    }

    fn load() -> AppData {
        let dir = data_dir();
        load_data_dir(&dir).expect("load_data_dir on real data directory must succeed")
    }

    fn find_section<'a>(app: &'a AppData, id: &str) -> &'a SectionConfig {
        app.sections
            .iter()
            .find(|s| s.id == id)
            .unwrap_or_else(|| panic!("section with id '{}' must exist in sections", id))
    }

    // ST51-2-TEST-1: all remaining intake sections (exercise, sleep_diet, social, history,
    // specialists) must have is_intake == true.
    // FAILS before implementation because sections.yml has not been populated for these sections.
    #[test]
    fn all_intake_sections_have_is_intake_true() {
        let app = load();
        for id in &["exercise", "sleep_diet", "social", "history", "specialists"] {
            let sec = find_section(&app, id);
            assert!(
                sec.is_intake,
                "section '{}' must have is_intake == true after sub-task 51.2 implementation; \
                 sections.yml has not been populated for this section yet",
                id
            );
        }
    }

    // ST51-2-TEST-2: all remaining intake sections must have the correct heading_label.
    // FAILS before implementation because sections.yml has not been populated for these sections.
    #[test]
    fn all_intake_sections_have_heading_label() {
        let app = load();
        let expected: &[(&str, &str)] = &[
            ("exercise",    "#### EXERCISE HABITS"),
            ("sleep_diet",  "#### SLEEP & DIET"),
            ("social",      "#### SOCIAL & STRESS"),
            ("history",     "#### HISTORY & PREVIOUS DIAGNOSES"),
            ("specialists", "#### SPECIALISTS & TREATMENT"),
        ];
        for &(id, label) in expected {
            let sec = find_section(&app, id);
            assert_eq!(
                sec.heading_label,
                Some(label.to_string()),
                "section '{}' must have heading_label == Some({:?}) after sub-task 51.2 \
                 implementation; sections.yml has not been populated for this section yet. \
                 Got: {:?}",
                id, label, sec.heading_label
            );
        }
    }

    // ST51-2-TEST-3: a representative set of sections must have the correct heading_search_text.
    // FAILS before implementation because sections.yml has not been populated for most sections.
    #[test]
    fn all_sections_with_search_text_are_set() {
        let app = load();
        let expected: &[(&str, &str)] = &[
            ("adl",               "ACTIVITIES OF DAILY LIVING"),
            ("exercise",          "EXERCISE HABITS"),
            ("tx_regions",        "TREATMENT / PLAN"),
            ("objective_section", "## OBJECTIVE / OBSERVATIONS"),
            ("post_treatment",    "## POST-TREATMENT"),
        ];
        for &(id, text) in expected {
            let sec = find_section(&app, id);
            assert_eq!(
                sec.heading_search_text,
                Some(text.to_string()),
                "section '{}' must have heading_search_text == Some({:?}) after sub-task 51.2 \
                 implementation; sections.yml has not been populated for this section yet. \
                 Got: {:?}",
                id, text, sec.heading_search_text
            );
        }
    }

    // ST51-2-TEST-4: all sections that map to a note render slot must have the correct
    // note_render_slot value.
    // FAILS before implementation because sections.yml has not been populated for most sections.
    #[test]
    fn remaining_sections_have_note_render_slot() {
        let app = load();
        let expected: &[(&str, &str)] = &[
            ("header",                   "header"),
            ("subjective_section",       "subjective_section"),
            ("tx_regions",               "tx_regions"),
            ("objective_section",        "objective_section"),
            ("post_treatment",           "post_treatment"),
            ("remedial_section",         "remedial_section"),
            ("tx_plan",                  "tx_plan"),
            ("infection_control_section","infection_control_section"),
        ];
        for &(id, slot) in expected {
            let sec = find_section(&app, id);
            assert_eq!(
                sec.note_render_slot,
                Some(slot.to_string()),
                "section '{}' must have note_render_slot == Some({:?}) after sub-task 51.2 \
                 implementation; sections.yml has not been populated for this section yet. \
                 Got: {:?}",
                id, slot, sec.note_render_slot
            );
        }
    }
}

// ST70-1: Hierarchy struct tests.
// These tests FAIL before implementation because HierarchyItem, HierarchyList, HierarchyField,
// HierarchySection, HierarchyGroup, HierarchyTemplate, and HierarchyFile do not exist yet.
#[cfg(test)]
mod hierarchy_struct_tests {
    use super::*;

    // ST70-1-TEST-1: HierarchyItem deserializes from YAML with required fields (id, label)
    // and optional fields (default, output, note).
    // FAILS because HierarchyItem does not exist yet.
    #[test]
    fn hierarchy_item_deserializes_basic() {
        let yaml = "id: opt_a\nlabel: Option A\n";
        let item: HierarchyItem = serde_yaml::from_str(yaml)
            .expect("HierarchyItem must deserialize from YAML with id and label");
        assert_eq!(item.id, "opt_a");
        assert_eq!(item.label, "Option A");
        assert!(item.default.is_none(), "default should be None when not specified");
        assert!(item.output.is_none(), "output should be None when not specified");
        assert!(item.note.is_none(), "note should be None when not specified");
    }

    // ST70-1-TEST-2: HierarchyItem deserializes with all optional fields present.
    // FAILS because HierarchyItem does not exist yet.
    #[test]
    fn hierarchy_item_deserializes_with_optional_fields() {
        let yaml = "id: opt_b\nlabel: Option B\ndefault: true\noutput: B output\nnote: a note\n";
        let item: HierarchyItem = serde_yaml::from_str(yaml)
            .expect("HierarchyItem must deserialize from YAML with all fields");
        assert_eq!(item.id, "opt_b");
        assert_eq!(item.label, "Option B");
        assert_eq!(item.default, Some(true));
        assert_eq!(item.output, Some("B output".to_string()));
        assert_eq!(item.note, Some("a note".to_string()));
    }

    // ST70-1-TEST-3: HierarchyList deserializes from YAML with id and items.
    // FAILS because HierarchyList does not exist yet.
    #[test]
    fn hierarchy_list_deserializes() {
        let yaml = "id: list_one\nitems:\n  - id: x\n    label: X\n  - id: y\n    label: Y\n";
        let list: HierarchyList = serde_yaml::from_str(yaml)
            .expect("HierarchyList must deserialize from YAML with id and items");
        assert_eq!(list.id, "list_one");
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].id, "x");
        assert_eq!(list.items[1].label, "Y");
    }

    // ST70-1-TEST-4: HierarchyField deserializes with id, label, field_type, and optional
    // options, list_id, and data_file fields.
    // FAILS because HierarchyField does not exist yet.
    #[test]
    fn hierarchy_field_deserializes() {
        let yaml = "id: f1\nlabel: Field One\nfield_type: select\noptions:\n  - alpha\n  - beta\n";
        let field: HierarchyField = serde_yaml::from_str(yaml)
            .expect("HierarchyField must deserialize from YAML with id, label, field_type, options");
        assert_eq!(field.id, "f1");
        assert_eq!(field.label, "Field One");
        assert_eq!(field.field_type, "select");
        assert_eq!(field.options, vec!["alpha".to_string(), "beta".to_string()]);
        assert!(field.list_id.is_none());
        assert!(field.data_file.is_none());
    }

    // ST70-1-TEST-5: HierarchyField deserializes with list_id and data_file.
    // FAILS because HierarchyField does not exist yet.
    #[test]
    fn hierarchy_field_deserializes_with_list_id_and_data_file() {
        let yaml = "id: f2\nlabel: Field Two\nfield_type: list\nlist_id: my_list\ndata_file: data.yml\n";
        let field: HierarchyField = serde_yaml::from_str(yaml)
            .expect("HierarchyField must deserialize from YAML with list_id and data_file");
        assert_eq!(field.id, "f2");
        assert_eq!(field.list_id, Some("my_list".to_string()));
        assert_eq!(field.data_file, Some("data.yml".to_string()));
    }

    // ST70-1-TEST-6: HierarchySection deserializes with id, nav_label, map_label, section_type,
    // and optional fields and lists children.
    // FAILS because HierarchySection does not exist yet.
    #[test]
    fn hierarchy_section_deserializes() {
        let yaml = concat!(
            "id: sec1\n",
            "nav_label: Section One\n",
            "map_label: SEC 1\n",
            "section_type: composite\n",
            "fields:\n",
            "  - id: f1\n",
            "    label: Field One\n",
            "    field_type: select\n",
        );
        let section: HierarchySection = serde_yaml::from_str(yaml)
            .expect("HierarchySection must deserialize from YAML");
        assert_eq!(section.id, "sec1");
        assert_eq!(section.nav_label, "Section One");
        assert_eq!(section.map_label, "SEC 1");
        assert_eq!(section.section_type, "composite");
        let fields = section.fields.as_ref().expect("fields must be Some");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].id, "f1");
        assert!(section.lists.is_none());
    }

    // ST70-1-TEST-7: HierarchyGroup deserializes with id, nav_label, and sections (Vec<String> IDs).
    // FAILS because HierarchyGroup does not exist yet.
    #[test]
    fn hierarchy_group_deserializes() {
        let yaml = "id: grp1\nnav_label: Group One\nsections:\n  - sec_a\n  - sec_b\n";
        let group: HierarchyGroup = serde_yaml::from_str(yaml)
            .expect("HierarchyGroup must deserialize from YAML");
        assert_eq!(group.id, "grp1");
        assert_eq!(group.nav_label, "Group One");
        assert_eq!(group.sections, vec!["sec_a".to_string(), "sec_b".to_string()]);
    }

    // ST70-1-TEST-8: HierarchyTemplate deserializes with groups (Vec<String> IDs).
    // FAILS because HierarchyTemplate does not exist yet.
    #[test]
    fn hierarchy_template_deserializes() {
        let yaml = "groups:\n  - grp1\n  - grp2\n";
        let template: HierarchyTemplate = serde_yaml::from_str(yaml)
            .expect("HierarchyTemplate must deserialize from YAML");
        assert_eq!(template.groups, vec!["grp1".to_string(), "grp2".to_string()]);
    }

    // ST70-1-TEST-9: HierarchyFile (top-level container) deserializes with optional
    // template, groups, sections, fields, lists, items, and boilerplate entries.
    // FAILS because HierarchyFile does not exist yet.
    #[test]
    fn hierarchy_file_deserializes_minimal() {
        let yaml = "template:\n  groups:\n    - g1\n";
        let file: HierarchyFile = serde_yaml::from_str(yaml)
            .expect("HierarchyFile must deserialize from YAML with just a template");
        let tmpl = file.template.as_ref().expect("template must be Some");
        assert_eq!(tmpl.groups, vec!["g1".to_string()]);
        assert!(file.groups.is_none());
        assert!(file.sections.is_none());
        assert!(file.fields.is_none());
        assert!(file.lists.is_none());
        assert!(file.items.is_none());
    }

    // ST70-1-TEST-10: HierarchyFile deserializes with all optional collections present.
    // FAILS because HierarchyFile does not exist yet.
    #[test]
    fn hierarchy_file_deserializes_full() {
        let yaml = concat!(
            "template:\n  groups:\n    - g1\n",
            "groups:\n  - id: g1\n    nav_label: G1\n    sections:\n      - s1\n",
            "sections:\n  - id: s1\n    nav_label: S1\n    map_label: S1\n    section_type: list\n",
            "fields:\n  - id: fi1\n    label: Fi1\n    field_type: text\n",
            "lists:\n  - id: l1\n    items:\n      - id: i1\n        label: I1\n",
            "items:\n  - id: i2\n    label: I2\n",
        );
        let file: HierarchyFile = serde_yaml::from_str(yaml)
            .expect("HierarchyFile must deserialize from YAML with all collections");
        assert!(file.template.is_some());
        let groups = file.groups.as_ref().expect("groups must be Some");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id, "g1");
        let sections = file.sections.as_ref().expect("sections must be Some");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].id, "s1");
        let fields = file.fields.as_ref().expect("fields must be Some");
        assert_eq!(fields.len(), 1);
        let lists = file.lists.as_ref().expect("lists must be Some");
        assert_eq!(lists.len(), 1);
        let items = file.items.as_ref().expect("items must be Some");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "i2");
    }
}

// ST70-2: Hierarchy loader tests.
// These tests FAIL before implementation because `load_hierarchy_dir` does not exist yet.
// The function under test has this signature:
//   pub fn load_hierarchy_dir(dir: &std::path::Path) -> Result<HierarchyFile, String>
//
// It scans `dir` for all *.yml files, merges them into a single HierarchyFile,
// validates template cardinality (exactly 1 template across all files),
// detects duplicate (TypeTag, id) pairs, verifies that all group->section and
// template->group cross-references point to IDs that exist in the merged pool,
// detects reference cycles, and returns the merged HierarchyFile on success.
#[cfg(test)]
mod hierarchy_loader_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn make_hier_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("scribblenot_hier_tests")
            .join(name);
        std::fs::create_dir_all(&dir).expect("create hierarchy test dir");
        dir
    }

    fn cleanup_hier_test_dir(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    fn write_hier_yml(dir: &Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).expect("write hierarchy yml");
    }

    // ST70-2-TEST-1: A directory containing a single valid YAML file (one template, one group,
    // one section) returns Ok and the merged HierarchyFile has the correct group count.
    // FAILS because load_hierarchy_dir does not exist yet.
    #[test]
    fn load_hierarchy_dir_returns_ok_for_valid_single_file() {
        let dir = make_hier_test_dir("valid_single");
        write_hier_yml(
            &dir,
            "hierarchy.yml",
            concat!(
                "template:\n",
                "  groups:\n",
                "    - grp1\n",
                "groups:\n",
                "  - id: grp1\n",
                "    nav_label: Group One\n",
                "    sections:\n",
                "      - sec1\n",
                "sections:\n",
                "  - id: sec1\n",
                "    nav_label: Section One\n",
                "    map_label: SEC 1\n",
                "    section_type: composite\n",
            ),
        );
        let result = load_hierarchy_dir(&dir);
        cleanup_hier_test_dir(&dir);
        assert!(
            result.is_ok(),
            "expected Ok(HierarchyFile) for valid single-file directory, got: {:?}",
            result.err()
        );
        let file = result.unwrap();
        let groups = file.groups.as_ref().expect("merged file must have groups");
        assert_eq!(groups.len(), 1, "merged file should have exactly 1 group");
    }

    // ST70-2-TEST-2: A directory containing a YAML file with no template at all returns Err
    // (template cardinality violation: 0 templates found).
    // FAILS because load_hierarchy_dir does not exist yet.
    #[test]
    fn load_hierarchy_dir_errors_when_zero_templates() {
        let dir = make_hier_test_dir("zero_templates");
        write_hier_yml(
            &dir,
            "no_template.yml",
            concat!(
                "groups:\n",
                "  - id: grp1\n",
                "    nav_label: Group One\n",
                "    sections: []\n",
            ),
        );
        let result = load_hierarchy_dir(&dir);
        cleanup_hier_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected Err when no template is defined across all files, got Ok"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.to_lowercase().contains("template"),
            "error message must mention 'template', got: {:?}",
            msg
        );
    }

    // ST70-2-TEST-3: A directory containing two YAML files that each define a template returns
    // Err (template cardinality violation: 2 templates found).
    // FAILS because load_hierarchy_dir does not exist yet.
    #[test]
    fn load_hierarchy_dir_errors_when_two_templates() {
        let dir = make_hier_test_dir("two_templates");
        write_hier_yml(
            &dir,
            "file_a.yml",
            "template:\n  groups:\n    - grp1\n",
        );
        write_hier_yml(
            &dir,
            "file_b.yml",
            "template:\n  groups:\n    - grp2\n",
        );
        let result = load_hierarchy_dir(&dir);
        cleanup_hier_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected Err when two templates are defined across files, got Ok"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.to_lowercase().contains("template"),
            "error message must mention 'template', got: {:?}",
            msg
        );
    }

    // ST70-2-TEST-4: A directory containing two files that define a group with the same id
    // returns Err (duplicate (TypeTag::Group, id) pair).
    // FAILS because load_hierarchy_dir does not exist yet.
    #[test]
    fn load_hierarchy_dir_errors_on_duplicate_group_id() {
        let dir = make_hier_test_dir("duplicate_group_id");
        write_hier_yml(
            &dir,
            "file_a.yml",
            concat!(
                "template:\n",
                "  groups:\n",
                "    - grp1\n",
                "groups:\n",
                "  - id: grp1\n",
                "    nav_label: Group One\n",
                "    sections: []\n",
            ),
        );
        write_hier_yml(
            &dir,
            "file_b.yml",
            concat!(
                "groups:\n",
                "  - id: grp1\n",
                "    nav_label: Group One Duplicate\n",
                "    sections: []\n",
            ),
        );
        let result = load_hierarchy_dir(&dir);
        cleanup_hier_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected Err for duplicate group id 'grp1' across files, got Ok"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("grp1") || msg.to_lowercase().contains("duplicate"),
            "error message must mention 'grp1' or 'duplicate', got: {:?}",
            msg
        );
    }

    // ST70-2-TEST-5: A directory where a group's sections list references a section id that
    // does not exist in any file returns Err (missing cross-reference).
    // FAILS because load_hierarchy_dir does not exist yet.
    #[test]
    fn load_hierarchy_dir_errors_on_missing_section_reference() {
        let dir = make_hier_test_dir("missing_section_ref");
        write_hier_yml(
            &dir,
            "hierarchy.yml",
            concat!(
                "template:\n",
                "  groups:\n",
                "    - grp1\n",
                "groups:\n",
                "  - id: grp1\n",
                "    nav_label: Group One\n",
                "    sections:\n",
                "      - sec_nonexistent\n",
            ),
        );
        let result = load_hierarchy_dir(&dir);
        cleanup_hier_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected Err when group references section id 'sec_nonexistent' that does not exist, got Ok"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("sec_nonexistent") || msg.to_lowercase().contains("missing") || msg.to_lowercase().contains("not found"),
            "error message must mention 'sec_nonexistent' or 'missing'/'not found', got: {:?}",
            msg
        );
    }

    // ST70-2-TEST-6: A directory where two boilerplate entries share the same id returns Err
    // (duplicate boilerplate ID).
    // FAILS because load_hierarchy_dir does not exist yet.
    #[test]
    fn load_hierarchy_dir_errors_on_duplicate_boilerplate_id() {
        let dir = make_hier_test_dir("duplicate_boilerplate_id");
        write_hier_yml(
            &dir,
            "file_a.yml",
            concat!(
                "template:\n",
                "  groups: []\n",
                "boilerplate:\n",
                "  - id: bp1\n",
                "    text: First boilerplate entry\n",
            ),
        );
        write_hier_yml(
            &dir,
            "file_b.yml",
            concat!(
                "boilerplate:\n",
                "  - id: bp1\n",
                "    text: Duplicate boilerplate entry\n",
            ),
        );
        let result = load_hierarchy_dir(&dir);
        cleanup_hier_test_dir(&dir);
        assert!(
            result.is_err(),
            "expected Err for duplicate boilerplate id 'bp1' across files, got Ok"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("bp1") || msg.to_lowercase().contains("duplicate") || msg.to_lowercase().contains("boilerplate"),
            "error message must mention 'bp1', 'duplicate', or 'boilerplate', got: {:?}",
            msg
        );
    }
}
