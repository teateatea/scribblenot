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
        let keybindings = if kb_path.exists() {
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
