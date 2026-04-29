// Data model types extracted from data.rs.
// Owns authored config structs, runtime structs/enums, serde defaults, and
// lightweight inherent impls tied directly to those types. Loader, validation,
// runtime construction, source-index, and hint helpers stay in data.rs (or
// their later sibling files) and continue to be the single public API surface
// via `pub use crate::data_model::*` from data.rs.

use serde::{
    de::{self, value::MapAccessDeserializer, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFieldConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default)]
    pub fields: Vec<HeaderFieldConfig>,
    #[serde(default)]
    pub lists: Vec<HierarchyList>,
    #[serde(default)]
    pub collections: Vec<ResolvedCollectionConfig>,
    #[serde(default)]
    pub format_lists: Vec<HierarchyList>,
    #[serde(default)]
    pub joiner_style: Option<JoinerStyle>,
    #[serde(default)]
    pub max_entries: Option<usize>,
    #[serde(default)]
    pub max_actives: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeNodeKind {
    #[default]
    Section,
    Collection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SectionBodyMode {
    MultiField,
    #[default]
    FreeText,
    ListSelect,
    Checklist,
    Collection,
}

impl SectionBodyMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MultiField => "multi_field",
            Self::FreeText => "free_text",
            Self::ListSelect => "list_select",
            Self::Checklist => "checklist",
            Self::Collection => "collection",
        }
    }
}

impl std::fmt::Display for SectionBodyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionConfig {
    pub id: String,
    pub name: String,
    pub map_label: String,
    #[serde(rename = "type")]
    pub section_type: SectionBodyMode,
    #[serde(default = "default_show_field_labels")]
    pub show_field_labels: bool,
    #[serde(default)]
    pub data_file: Option<String>,
    #[serde(default)]
    pub fields: Option<Vec<HeaderFieldConfig>>,
    #[serde(default)]
    pub lists: Vec<HierarchyList>,
    #[serde(default)]
    pub note_label: Option<String>,
    #[serde(default)]
    pub group_id: String,
    #[serde(default)]
    pub node_kind: RuntimeNodeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEntry {
    pub label: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub nav_down: Vec<String>,
    pub nav_up: Vec<String>,
    pub select: Vec<String>,
    pub confirm: Vec<String>,
    pub add_entry: Vec<String>,
    pub back: Vec<String>,
    pub swap_panes: Vec<String>,
    pub help: Vec<String>,
    pub quit: Vec<String>,
    #[serde(default = "default_nav_left")]
    pub nav_left: Vec<String>,
    #[serde(default = "default_nav_right")]
    pub nav_right: Vec<String>,
    #[serde(default = "default_hints")]
    pub hints: Vec<String>,
    #[serde(default = "default_super_confirm")]
    pub super_confirm: Vec<String>,
    #[serde(default)]
    pub hint_permutations: Vec<String>,
    #[serde(default = "default_copy_note")]
    pub copy_note: Vec<String>,
    #[serde(default = "default_theme_reload")]
    pub theme_reload: Vec<String>,
    #[serde(default = "default_data_reload")]
    pub data_reload: Vec<String>,
}

fn default_copy_note() -> Vec<String> {
    vec!["c".to_string()]
}

fn default_theme_reload() -> Vec<String> {
    vec!["/".to_string()]
}

fn default_data_reload() -> Vec<String> {
    vec!["\\".to_string()]
}

fn default_super_confirm() -> Vec<String> {
    vec!["shift+enter".to_string()]
}

fn default_nav_left() -> Vec<String> {
    vec!["left".to_string(), "h".to_string()]
}

fn default_nav_right() -> Vec<String> {
    vec!["right".to_string(), "i".to_string()]
}

pub(crate) fn default_hints() -> Vec<String> {
    ["1", "2", "3", "4", "5", "6", "7", "8", "9"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            nav_down: vec!["down".to_string(), "n".to_string()],
            nav_up: vec!["up".to_string(), "e".to_string()],
            select: vec!["space".to_string(), "s".to_string()],
            confirm: vec!["enter".to_string(), "t".to_string()],
            add_entry: vec!["a".to_string(), "d".to_string()],
            back: vec!["esc".to_string()],
            swap_panes: vec!["`".to_string()],
            help: vec!["?".to_string()],
            quit: vec!["ctrl+q".to_string()],
            nav_left: default_nav_left(),
            nav_right: default_nav_right(),
            hints: default_hints(),
            super_confirm: default_super_confirm(),
            hint_permutations: vec![],
            copy_note: default_copy_note(),
            theme_reload: default_theme_reload(),
            data_reload: default_data_reload(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthoredHotkeys {
    pub sections: HashMap<String, String>,
    pub fields: HashMap<String, String>,
    pub items: HashMap<String, HashMap<String, String>>,
}

impl AuthoredHotkeys {
    pub fn section(&self, section_id: &str) -> Option<&str> {
        self.sections.get(section_id).map(String::as_str)
    }

    pub fn field(&self, field_id: &str) -> Option<&str> {
        self.fields.get(field_id).map(String::as_str)
    }

    pub fn item(&self, list_id: &str, item_id: &str) -> Option<&str> {
        self.items
            .get(list_id)
            .and_then(|items| items.get(item_id))
            .map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataValidationSummary {
    pub hierarchy_file_count: usize,
    pub keybindings_present: bool,
    pub group_count: usize,
    pub section_count: usize,
    pub collection_count: usize,
    pub field_count: usize,
    pub list_count: usize,
    pub boilerplate_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupNoteMeta {
    #[serde(default)]
    pub note_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NoteNodeMeta {
    #[serde(default)]
    pub note_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModalStart {
    List,
    Search,
}

impl Default for ModalStart {
    fn default() -> Self {
        Self::List
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinerStyle {
    CommaAnd,
    CommaAndThe,
    CommaOr,
    Comma,
    Space,
    Semicolon,
    Slash,
    Newline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HierarchyList {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default)]
    pub sticky: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub modal_start: ModalStart,
    #[serde(default)]
    pub joiner_style: Option<JoinerStyle>,
    #[serde(default)]
    pub max_entries: Option<usize>,
    #[serde(default)]
    pub items: Vec<HierarchyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ItemAssignment {
    #[serde(rename = "list")]
    pub list_id: String,
    #[serde(rename = "item")]
    pub item_id: String,
    #[serde(skip)]
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HierarchyItem {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_item_enabled")]
    pub default_enabled: bool,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub fields: Option<Vec<String>>,
    #[serde(skip)]
    pub branch_fields: Vec<HeaderFieldConfig>,
    #[serde(default)]
    pub assigns: Vec<ItemAssignment>,
}

pub(crate) fn default_item_enabled() -> bool {
    true
}

pub(crate) fn default_show_field_labels() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HierarchyItemRaw {
    #[serde(default)]
    id: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default = "default_item_enabled")]
    default_enabled: bool,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    hotkey: Option<String>,
    #[serde(default)]
    fields: Option<Vec<String>>,
    #[serde(default)]
    assigns: Vec<ItemAssignment>,
}

impl<'de> Deserialize<'de> for HierarchyItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HierarchyItemVisitor;

        impl<'de> Visitor<'de> for HierarchyItemVisitor {
            type Value = HierarchyItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string item label or an item mapping")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(HierarchyItem::from_simple(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(HierarchyItem::from_simple(value))
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let raw = HierarchyItemRaw::deserialize(MapAccessDeserializer::new(map))?;
                Ok(HierarchyItem::from_raw(raw))
            }
        }

        deserializer.deserialize_any(HierarchyItemVisitor)
    }
}

impl HierarchyItem {
    fn from_raw(item: HierarchyItemRaw) -> Self {
        Self {
            id: if item.id.is_empty() {
                slugify_id(
                    item.label
                        .as_deref()
                        .or(item.output.as_deref())
                        .unwrap_or(""),
                )
            } else {
                item.id
            },
            label: item.label,
            default_enabled: item.default_enabled,
            output: item.output,
            fields: item.fields,
            branch_fields: Vec::new(),
            assigns: item.assigns,
        }
    }

    fn from_simple(label: String) -> Self {
        Self {
            id: slugify_id(&label),
            label: Some(label.clone()),
            default_enabled: true,
            output: Some(label),
            fields: None,
            branch_fields: Vec::new(),
            assigns: Vec::new(),
        }
    }

    pub fn ui_label(&self) -> &str {
        self.label
            .as_deref()
            .or(self.output.as_deref())
            .unwrap_or("")
    }

    pub fn output(&self) -> &str {
        self.output
            .as_deref()
            .or(self.label.as_deref())
            .unwrap_or("")
    }

    pub fn default_enabled(&self) -> bool {
        self.default_enabled
    }
}

pub(crate) fn slug_source_for_item(item: &HierarchyItem) -> &str {
    item.label
        .as_deref()
        .or(item.output.as_deref())
        .unwrap_or("")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoilerplateEntry {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HierarchyTemplate {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub contains: Vec<HierarchyChildRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HierarchyGroup {
    pub id: String,
    #[serde(default)]
    pub nav_label: Option<String>,
    #[serde(default)]
    pub note_label: Option<String>,
    #[serde(default)]
    pub contains: Vec<HierarchyChildRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HierarchySection {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub nav_label: Option<String>,
    #[serde(default)]
    pub hotkey: Option<String>,
    #[serde(default = "default_show_field_labels")]
    pub show_field_labels: bool,
    #[serde(default)]
    pub contains: Vec<HierarchyChildRef>,
    #[serde(default)]
    pub note: NoteNodeMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HierarchyCollection {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub nav_label: Option<String>,
    #[serde(default)]
    pub note_label: Option<String>,
    #[serde(default = "default_item_enabled")]
    pub default_enabled: bool,
    #[serde(default)]
    pub joiner_style: Option<JoinerStyle>,
    #[serde(default)]
    pub contains: Vec<HierarchyChildRef>,
    #[serde(default)]
    pub note: NoteNodeMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HierarchyField {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub nav_label: Option<String>,
    #[serde(default)]
    pub hotkey: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default)]
    pub contains: Vec<HierarchyChildRef>,
    #[serde(default)]
    pub joiner_style: Option<JoinerStyle>,
    #[serde(default)]
    pub max_entries: Option<usize>,
    #[serde(default)]
    pub max_actives: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCollectionConfig {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub note_label: Option<String>,
    #[serde(default = "default_item_enabled")]
    pub default_enabled: bool,
    #[serde(default)]
    pub joiner_style: Option<JoinerStyle>,
    #[serde(default)]
    pub lists: Vec<HierarchyList>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum HierarchyChildRef {
    Group { group: String },
    Section { section: String },
    Collection { collection: String },
    Field { field: String },
    List { list: String },
    Boilerplate { boilerplate: String },
}

impl HierarchyChildRef {
    pub fn kind(&self) -> TypeTag {
        match self {
            Self::Group { .. } => TypeTag::Group,
            Self::Section { .. } => TypeTag::Section,
            Self::Collection { .. } => TypeTag::Collection,
            Self::Field { .. } => TypeTag::Field,
            Self::List { .. } => TypeTag::List,
            Self::Boilerplate { .. } => TypeTag::Boilerplate,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Group { group } => group,
            Self::Section { section } => section,
            Self::Collection { collection } => collection,
            Self::Field { field } => field,
            Self::List { list } => list,
            Self::Boilerplate { boilerplate } => boilerplate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HierarchyFile {
    #[serde(default)]
    pub template: Option<HierarchyTemplate>,
    #[serde(default)]
    pub groups: Vec<HierarchyGroup>,
    #[serde(default)]
    pub sections: Vec<HierarchySection>,
    #[serde(default)]
    pub collections: Vec<HierarchyCollection>,
    #[serde(default)]
    pub fields: Vec<HierarchyField>,
    #[serde(default)]
    pub lists: Vec<HierarchyList>,
    #[serde(default)]
    pub boilerplates: Vec<BoilerplateEntry>,
    #[serde(skip)]
    pub item_hotkeys: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeTag {
    Group,
    Section,
    Collection,
    Field,
    List,
    Boilerplate,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RuntimeTemplate {
    pub id: String,
    pub children: Vec<RuntimeGroup>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RuntimeGroup {
    pub id: String,
    pub nav_label: String,
    pub note: GroupNoteMeta,
    pub children: Vec<RuntimeNode>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RuntimeNode {
    Section(SectionConfig),
    Collection(SectionConfig),
    Boilerplate(String),
}

impl RuntimeNode {
    pub fn config(&self) -> &SectionConfig {
        match self {
            Self::Section(config) | Self::Collection(config) => config,
            Self::Boilerplate(_) => panic!("config() called on Boilerplate node"),
        }
    }

    pub fn as_config(&self) -> Option<&SectionConfig> {
        match self {
            Self::Section(config) | Self::Collection(config) => Some(config),
            Self::Boilerplate(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationEntry {
    pub node_id: String,
    pub group_id: String,
    pub group_index: usize,
    pub node_kind: RuntimeNodeKind,
}

#[derive(Debug, Clone)]
pub struct RuntimeHierarchy {
    pub template: RuntimeTemplate,
    pub list_data: HashMap<String, Vec<ListEntry>>,
    pub checklist_data: HashMap<String, Vec<String>>,
    pub collection_data: HashMap<String, Vec<ResolvedCollectionConfig>>,
    pub boilerplate_texts: HashMap<String, String>,
}

pub(crate) fn slugify_id(label: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;
    for c in label.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    if slug.is_empty() {
        "item".to_string()
    } else {
        slug
    }
}
