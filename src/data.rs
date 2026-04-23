use anyhow::Result;
use serde::{
    de::{self, value::MapAccessDeserializer, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

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
}

fn default_copy_note() -> Vec<String> {
    vec!["c".to_string()]
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

fn default_hints() -> Vec<String> {
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
        }
    }
}

#[derive(Debug)]
pub struct AppData {
    #[allow(dead_code)]
    pub template: RuntimeTemplate,
    pub list_data: HashMap<String, Vec<ListEntry>>,
    pub checklist_data: HashMap<String, Vec<String>>,
    pub collection_data: HashMap<String, Vec<ResolvedCollectionConfig>>,
    pub boilerplate_texts: HashMap<String, String>,
    pub keybindings: KeyBindings,
    pub hotkeys: AuthoredHotkeys,
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

impl AppData {
    pub fn load(data_dir: PathBuf) -> Result<Self> {
        let hierarchy = load_hierarchy_dir(&data_dir).map_err(anyhow::Error::msg)?;
        let hotkeys = collect_authored_hotkeys(&hierarchy);
        let runtime = hierarchy_to_runtime(hierarchy).map_err(anyhow::Error::msg)?;

        let kb_path = data_dir.join("keybindings.yml");
        let mut keybindings = if kb_path.exists() {
            let kb_content = fs::read_to_string(&kb_path)?;
            serde_yaml::from_str(&kb_content).unwrap_or_else(|err| {
                eprintln!(
                    "Warning: keybindings.yml parse error ({}), using defaults",
                    err
                );
                KeyBindings::default()
            })
        } else {
            KeyBindings::default()
        };
        ensure_hint_permutations(&mut keybindings);

        Ok(Self {
            template: runtime.template,
            list_data: runtime.list_data,
            checklist_data: runtime.checklist_data,
            collection_data: runtime.collection_data,
            boilerplate_texts: runtime.boilerplate_texts,
            keybindings,
            hotkeys,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupNoteMeta {
    #[serde(default)]
    pub note_label: Option<String>,
    #[serde(default)]
    pub boilerplate_refs: Vec<String>,
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

fn default_item_enabled() -> bool {
    true
}

fn default_show_field_labels() -> bool {
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

fn slug_source_for_item(item: &HierarchyItem) -> &str {
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
    pub boilerplate_refs: Vec<String>,
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
}

impl HierarchyChildRef {
    pub fn kind(&self) -> TypeTag {
        match self {
            Self::Group { .. } => TypeTag::Group,
            Self::Section { .. } => TypeTag::Section,
            Self::Collection { .. } => TypeTag::Collection,
            Self::Field { .. } => TypeTag::Field,
            Self::List { .. } => TypeTag::List,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Group { group } => group,
            Self::Section { section } => section,
            Self::Collection { collection } => collection,
            Self::Field { field } => field,
            Self::List { list } => list,
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
    pub boilerplate: Vec<BoilerplateEntry>,
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
}

impl RuntimeNode {
    pub fn config(&self) -> &SectionConfig {
        match self {
            Self::Section(config) | Self::Collection(config) => config,
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

pub fn flat_sections_from_template(template: &RuntimeTemplate) -> Vec<SectionConfig> {
    template
        .children
        .iter()
        .flat_map(|group| group.children.iter())
        .map(|node| node.config().clone())
        .collect()
}

pub fn runtime_navigation(template: &RuntimeTemplate) -> Vec<NavigationEntry> {
    let mut entries = Vec::new();
    for (group_index, group) in template.children.iter().enumerate() {
        for node in &group.children {
            let config = node.config();
            entries.push(NavigationEntry {
                node_id: config.id.clone(),
                group_id: group.id.clone(),
                group_index,
                node_kind: config.node_kind,
            });
        }
    }
    entries
}

pub fn hierarchy_to_runtime(hf: HierarchyFile) -> Result<RuntimeHierarchy, String> {
    let template = hf
        .template
        .clone()
        .ok_or_else(|| "merged hierarchy is missing template".to_string())?;
    let template_id = template
        .id
        .clone()
        .unwrap_or_else(|| "default_template".to_string());

    let groups_by_id: HashMap<&str, &HierarchyGroup> = hf
        .groups
        .iter()
        .map(|group| (group.id.as_str(), group))
        .collect();
    let sections_by_id: HashMap<&str, &HierarchySection> = hf
        .sections
        .iter()
        .map(|section| (section.id.as_str(), section))
        .collect();
    let collections_by_id: HashMap<&str, &HierarchyCollection> = hf
        .collections
        .iter()
        .map(|collection| (collection.id.as_str(), collection))
        .collect();
    let fields_by_id: HashMap<&str, &HierarchyField> = hf
        .fields
        .iter()
        .map(|field| (field.id.as_str(), field))
        .collect();
    let lists_by_id: HashMap<&str, &HierarchyList> = hf
        .lists
        .iter()
        .map(|list| (list.id.as_str(), list))
        .collect();

    let mut runtime_groups = Vec::new();
    let mut list_data = HashMap::new();
    let mut checklist_data = HashMap::new();
    let mut collection_data = HashMap::new();

    for child in &template.contains {
        let HierarchyChildRef::Group { group } = child else {
            return Err("template runtime build expected only group refs".to_string());
        };
        let hierarchy_group = groups_by_id
            .get(group.as_str())
            .ok_or_else(|| format!("unknown group '{}'", group))?;
        let group_note_label = hierarchy_group.note_label.clone();
        let group_nav_label = hierarchy_group
            .nav_label
            .clone()
            .unwrap_or_else(|| hierarchy_group.id.clone());
        let group_note = GroupNoteMeta {
            note_label: group_note_label.clone(),
            boilerplate_refs: hierarchy_group.boilerplate_refs.clone(),
        };
        let child_fallback_name = group_note_label
            .clone()
            .unwrap_or_else(|| group_nav_label.clone());
        let mut runtime_children = Vec::new();
        for child in &hierarchy_group.contains {
            match child {
                HierarchyChildRef::Section { section } => {
                    let section_data = sections_by_id
                        .get(section.as_str())
                        .ok_or_else(|| format!("unknown section '{}'", section))?;
                    let section_config = section_to_config(
                        section_data,
                        &child_fallback_name,
                        hierarchy_group.id.as_str(),
                        &fields_by_id,
                        &collections_by_id,
                        &lists_by_id,
                    )?;
                    runtime_children.push(RuntimeNode::Section(section_config.clone()));
                    maybe_record_section_lists(
                        &section_config,
                        &mut list_data,
                        &mut checklist_data,
                    );
                }
                HierarchyChildRef::Collection { collection } => {
                    let collection_def = collections_by_id
                        .get(collection.as_str())
                        .ok_or_else(|| format!("unknown collection '{}'", collection))?;
                    let collection_config = collection_to_config(
                        collection_def,
                        &child_fallback_name,
                        hierarchy_group.id.as_str(),
                        &fields_by_id,
                        &collections_by_id,
                        &lists_by_id,
                    )?;
                    runtime_children.push(RuntimeNode::Collection(collection_config.clone()));
                    collection_data.insert(
                        collection_config.id.clone(),
                        vec![resolve_collection(
                            collection_def,
                            &child_fallback_name,
                            &fields_by_id,
                            &collections_by_id,
                            &lists_by_id,
                        )?],
                    );
                }
                other => {
                    return Err(format!(
                        "group '{}' cannot contain {:?} at runtime",
                        hierarchy_group.id,
                        other.kind()
                    ));
                }
            }
        }

        runtime_groups.push(RuntimeGroup {
            id: hierarchy_group.id.clone(),
            nav_label: group_nav_label.clone(),
            note: group_note.clone(),
            children: runtime_children,
        });
    }

    let boilerplate_texts = hf
        .boilerplate
        .into_iter()
        .map(|entry| (entry.id, entry.text))
        .collect();

    Ok(RuntimeHierarchy {
        template: RuntimeTemplate {
            id: template_id,
            children: runtime_groups,
        },
        list_data,
        checklist_data,
        collection_data,
        boilerplate_texts,
    })
}

fn section_to_config(
    section: &HierarchySection,
    fallback_name: &str,
    group_id: &str,
    fields_by_id: &HashMap<&str, &HierarchyField>,
    collections_by_id: &HashMap<&str, &HierarchyCollection>,
    lists_by_id: &HashMap<&str, &HierarchyList>,
) -> Result<SectionConfig, String> {
    let mut field_configs = Vec::new();
    let mut attached_lists = Vec::new();

    for child in &section.contains {
        match child {
            HierarchyChildRef::Field { field } => {
                let field_data = fields_by_id
                    .get(field.as_str())
                    .ok_or_else(|| format!("unknown field '{}'", field))?;
                field_configs.push(resolve_field(
                    field_data,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    &mut Vec::new(),
                )?);
            }
            HierarchyChildRef::List { list } => {
                attached_lists.push(resolve_runtime_list(
                    lists_by_id
                        .get(list.as_str())
                        .ok_or_else(|| format!("unknown list '{}'", list))?,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                )?);
            }
            other => {
                return Err(format!(
                    "section '{}' cannot contain {:?}",
                    section.id,
                    other.kind()
                ));
            }
        }
    }

    let name = section
        .label
        .clone()
        .or_else(|| section.nav_label.clone())
        .unwrap_or_else(|| fallback_name.to_string());
    let map_label = section.nav_label.clone().unwrap_or_else(|| name.clone());
    let heading_label = section
        .note
        .note_label
        .clone()
        .or_else(|| Some(format!("#### {}", name.to_uppercase())));
    Ok(SectionConfig {
        id: section.id.clone(),
        name,
        map_label,
        section_type: infer_body_mode(&field_configs, &attached_lists),
        show_field_labels: section.show_field_labels,
        data_file: None,
        fields: (!field_configs.is_empty()).then_some(field_configs),
        lists: attached_lists,
        note_label: heading_label,
        group_id: group_id.to_string(),
        node_kind: RuntimeNodeKind::Section,
    })
}

fn collection_to_config(
    collection: &HierarchyCollection,
    fallback_name: &str,
    group_id: &str,
    fields_by_id: &HashMap<&str, &HierarchyField>,
    collections_by_id: &HashMap<&str, &HierarchyCollection>,
    lists_by_id: &HashMap<&str, &HierarchyList>,
) -> Result<SectionConfig, String> {
    let resolved = resolve_collection(
        collection,
        fallback_name,
        fields_by_id,
        collections_by_id,
        lists_by_id,
    )?;
    let map_label = collection
        .nav_label
        .clone()
        .unwrap_or_else(|| resolved.label.clone());

    Ok(SectionConfig {
        id: resolved.id.clone(),
        name: resolved.label.clone(),
        map_label,
        section_type: SectionBodyMode::Collection,
        show_field_labels: true,
        data_file: None,
        fields: None,
        lists: resolved.lists.clone(),
        note_label: resolved.note_label.clone(),
        group_id: group_id.to_string(),
        node_kind: RuntimeNodeKind::Collection,
    })
}

fn resolve_collection(
    collection: &HierarchyCollection,
    fallback_name: &str,
    fields_by_id: &HashMap<&str, &HierarchyField>,
    collections_by_id: &HashMap<&str, &HierarchyCollection>,
    lists_by_id: &HashMap<&str, &HierarchyList>,
) -> Result<ResolvedCollectionConfig, String> {
    let mut lists = Vec::new();
    for child in &collection.contains {
        match child {
            HierarchyChildRef::List { list } => lists.push(resolve_runtime_list(
                lists_by_id
                    .get(list.as_str())
                    .ok_or_else(|| format!("unknown list '{}'", list))?,
                fields_by_id,
                collections_by_id,
                lists_by_id,
            )?),
            other => {
                return Err(format!(
                    "collection '{}' cannot contain {:?}",
                    collection.id,
                    other.kind()
                ));
            }
        }
    }

    let label = collection
        .label
        .clone()
        .or_else(|| collection.nav_label.clone())
        .unwrap_or_else(|| fallback_name.to_string());
    let note_label = collection
        .note
        .note_label
        .clone()
        .or_else(|| Some(format!("#### {}", label.to_uppercase())));

    Ok(ResolvedCollectionConfig {
        id: collection.id.clone(),
        label,
        note_label,
        default_enabled: collection.default_enabled,
        joiner_style: collection.joiner_style.clone(),
        lists,
    })
}

fn infer_body_mode(fields: &[HeaderFieldConfig], lists: &[HierarchyList]) -> SectionBodyMode {
    if !fields.is_empty() {
        SectionBodyMode::MultiField
    } else if lists.is_empty() {
        SectionBodyMode::FreeText
    } else {
        SectionBodyMode::ListSelect
    }
}

fn resolve_field(
    field: &HierarchyField,
    fields_by_id: &HashMap<&str, &HierarchyField>,
    collections_by_id: &HashMap<&str, &HierarchyCollection>,
    lists_by_id: &HashMap<&str, &HierarchyList>,
    visiting: &mut Vec<String>,
) -> Result<HeaderFieldConfig, String> {
    if visiting.iter().any(|existing| existing == &field.id) {
        let mut path = visiting.clone();
        path.push(field.id.clone());
        return Err(format!("field cycle detected: {}", path.join(" -> ")));
    }
    visiting.push(field.id.clone());

    let has_nested_fields = field
        .contains
        .iter()
        .any(|child| matches!(child, HierarchyChildRef::Field { .. }));

    let result = resolve_field_inner(
        field,
        has_nested_fields,
        fields_by_id,
        collections_by_id,
        lists_by_id,
        visiting,
    );
    visiting.pop();
    result
}

fn resolve_field_inner(
    field: &HierarchyField,
    has_nested_fields: bool,
    fields_by_id: &HashMap<&str, &HierarchyField>,
    collections_by_id: &HashMap<&str, &HierarchyCollection>,
    lists_by_id: &HashMap<&str, &HierarchyList>,
    visiting: &mut Vec<String>,
) -> Result<HeaderFieldConfig, String> {
    let mut fields = Vec::new();
    let mut lists = Vec::new();
    let mut collections = Vec::new();
    let mut format_lists = Vec::new();
    for child in &field.contains {
        match child {
            HierarchyChildRef::Field { field: child_id } => {
                let child = fields_by_id.get(child_id.as_str()).ok_or_else(|| {
                    format!(
                        "field '{}' references unknown field '{}'",
                        field.id, child_id
                    )
                })?;
                fields.push(resolve_field(
                    child,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    visiting,
                )?);
            }
            HierarchyChildRef::List { list } => {
                let list = resolve_runtime_list(
                    lists_by_id.get(list.as_str()).ok_or_else(|| {
                        format!("field '{}' references unknown list '{}'", field.id, list)
                    })?,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                )?;
                if has_nested_fields {
                    fields.push(wrap_list_as_field(&list));
                } else {
                    lists.push(list);
                }
            }
            HierarchyChildRef::Collection { collection } => {
                let collection = collections_by_id.get(collection.as_str()).ok_or_else(|| {
                    format!(
                        "field '{}' references unknown collection '{}'",
                        field.id, collection
                    )
                })?;
                let resolved = resolve_collection(
                    collection,
                    &field.label,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                )?;
                if has_nested_fields {
                    fields.push(wrap_collection_as_field(&resolved));
                } else {
                    collections.push(resolved);
                }
            }
            other => {
                return Err(format!(
                    "field '{}' cannot contain {:?}",
                    field.id,
                    other.kind()
                ));
            }
        }
    }
    for list_id in referenced_placeholder_ids(field.format.as_deref()) {
        let list_is_primary = lists.iter().any(|list| list.id == list_id);
        let collection_matches = collections
            .iter()
            .any(|collection| collection.id == list_id);
        let field_matches = fields.iter().any(|child| child.id == list_id);
        if list_is_primary || collection_matches || field_matches {
            continue;
        }
        format_lists.push(resolve_runtime_list(
            lists_by_id.get(list_id.as_str()).ok_or_else(|| {
                format!(
                    "field '{}' references unknown format list '{}'",
                    field.id, list_id
                )
            })?,
            fields_by_id,
            collections_by_id,
            lists_by_id,
        )?);
    }
    Ok(HeaderFieldConfig {
        id: field.id.clone(),
        name: field.label.clone(),
        format: field.format.clone(),
        preview: field.preview.clone(),
        fields,
        lists,
        collections,
        format_lists,
        joiner_style: field.joiner_style.clone(),
        max_entries: field.max_entries,
        max_actives: field.max_actives,
    })
}

fn wrap_list_as_field(list: &HierarchyList) -> HeaderFieldConfig {
    HeaderFieldConfig {
        id: list.id.clone(),
        name: list.label.clone().unwrap_or_else(|| list.id.clone()),
        format: None,
        preview: list.preview.clone(),
        fields: Vec::new(),
        lists: vec![list.clone()],
        collections: Vec::new(),
        format_lists: Vec::new(),
        joiner_style: None,
        max_entries: None,
        max_actives: None,
    }
}

fn wrap_collection_as_field(collection: &ResolvedCollectionConfig) -> HeaderFieldConfig {
    HeaderFieldConfig {
        id: collection.id.clone(),
        name: collection.label.clone(),
        format: None,
        preview: None,
        fields: Vec::new(),
        lists: Vec::new(),
        collections: vec![collection.clone()],
        format_lists: Vec::new(),
        joiner_style: None,
        max_entries: None,
        max_actives: None,
    }
}

fn referenced_placeholder_ids(format: Option<&str>) -> Vec<String> {
    let Some(format) = format else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    let mut chars = format.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' {
            continue;
        }
        let mut id = String::new();
        for next in chars.by_ref() {
            if next == '}' {
                break;
            }
            id.push(next);
        }
        if !id.is_empty() && !ids.iter().any(|existing| existing == &id) {
            ids.push(id);
        }
    }
    ids
}

fn maybe_record_section_lists(
    section: &SectionConfig,
    list_data: &mut HashMap<String, Vec<ListEntry>>,
    checklist_data: &mut HashMap<String, Vec<String>>,
) {
    match section.section_type {
        SectionBodyMode::ListSelect => {
            list_data.insert(section.id.clone(), list_entries_from_lists(&section.lists));
        }
        SectionBodyMode::Checklist => {
            checklist_data.insert(
                section.id.clone(),
                checklist_items_from_lists(&section.lists),
            );
        }
        _ => {}
    }
}

fn list_entries_from_lists(lists: &[HierarchyList]) -> Vec<ListEntry> {
    lists
        .iter()
        .flat_map(|list| {
            list.items.iter().map(|item| ListEntry {
                label: item.ui_label().to_string(),
                output: item.output().to_string(),
            })
        })
        .collect()
}

fn checklist_items_from_lists(lists: &[HierarchyList]) -> Vec<String> {
    lists
        .iter()
        .flat_map(|list| list.items.iter().map(|item| item.ui_label().to_string()))
        .collect()
}

fn read_hierarchy_dir(dir: &Path) -> Result<(HierarchyFile, usize), String> {
    let mut merged = HierarchyFile::default();
    let mut template_count = 0usize;
    let mut hierarchy_file_count = 0usize;

    let mut entries = fs::read_dir(dir)
        .map_err(|err| format!("failed to read data dir '{}': {err}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to enumerate data dir '{}': {err}", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("keybindings.yml") {
            continue;
        }
        hierarchy_file_count += 1;

        let content = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read '{}': {err}", path.display()))?;
        let file = parse_hierarchy_file_documents(&content, &path)?;

        if file.template.is_some() {
            template_count += 1;
            if merged.template.is_some() {
                return Err(format!(
                    "multiple templates found while loading '{}'",
                    path.display()
                ));
            }
            merged.template = file.template;
        }

        merged.groups.extend(file.groups);
        merged.sections.extend(file.sections);
        merged.collections.extend(file.collections);
        merged.fields.extend(file.fields);
        merged.lists.extend(file.lists);
        merged.boilerplate.extend(file.boilerplate);
    }

    if template_count != 1 {
        return Err(format!(
            "expected exactly 1 template across data files, found {}",
            template_count
        ));
    }

    Ok((merged, hierarchy_file_count))
}

pub fn load_hierarchy_dir(dir: &Path) -> Result<HierarchyFile, String> {
    let (merged, _) = read_hierarchy_dir(dir)?;
    validate_merged_hierarchy(&merged)?;
    Ok(merged)
}

pub fn validate_data_dir(dir: &Path) -> Result<DataValidationSummary, String> {
    let (merged, hierarchy_file_count) = read_hierarchy_dir(dir)?;
    validate_merged_hierarchy(&merged)?;
    let summary = DataValidationSummary {
        hierarchy_file_count,
        keybindings_present: validate_keybindings_file(&dir.join("keybindings.yml"))?,
        group_count: merged.groups.len(),
        section_count: merged.sections.len(),
        collection_count: merged.collections.len(),
        field_count: merged.fields.len(),
        list_count: merged.lists.len(),
        boilerplate_count: merged.boilerplate.len(),
    };
    hierarchy_to_runtime(merged)
        .map_err(|err| format!("validated data could not build runtime hierarchy: {err}"))?;
    Ok(summary)
}

fn parse_hierarchy_file_documents(content: &str, path: &Path) -> Result<HierarchyFile, String> {
    let mut merged = HierarchyFile::default();
    let mut docs = serde_yaml::Deserializer::from_str(content).peekable();

    if docs.peek().is_none() {
        return Ok(merged);
    }

    for (doc_idx, doc) in docs.enumerate() {
        let value = serde_yaml::Value::deserialize(doc).map_err(|err| {
            format!(
                "failed to parse '{}' document {}: {err}",
                path.display(),
                doc_idx + 1
            )
        })?;
        if contains_legacy_repeating_key(&value) {
            return Err(format!(
                "failed to parse '{}' document {}: deprecated key 'repeating' found; use 'joiner_style'",
                path.display(),
                doc_idx + 1
            ));
        }
        if let Some((field_id, key_name)) = find_legacy_field_child_key(&value) {
            return Err(format!(
                "failed to parse '{}' document {}: field '{}' uses deprecated key '{}'; use `contains:` with typed child refs such as `- {{ list: some_list_id }}` instead.",
                path.display(),
                doc_idx + 1,
                field_id,
                key_name
            ));
        }
        let raw_value = value.clone();
        let mut file = HierarchyFile::deserialize(value).map_err(|err| {
            format!(
                "failed to parse '{}' document {}: {err}",
                path.display(),
                doc_idx + 1
            )
        })?;
        normalize_items(&mut file);
        file.item_hotkeys = extract_item_hotkeys_from_value(&raw_value, &file);

        if file.template.is_some() {
            if merged.template.is_some() {
                return Err(format!(
                    "multiple templates found inside '{}'",
                    path.display()
                ));
            }
            merged.template = file.template.take();
        }

        merged.groups.extend(file.groups);
        merged.sections.extend(file.sections);
        merged.collections.extend(file.collections);
        merged.fields.extend(file.fields);
        merged.lists.extend(file.lists);
        merged.boilerplate.extend(file.boilerplate);
        for (list_id, item_hotkeys) in file.item_hotkeys {
            merged
                .item_hotkeys
                .entry(list_id)
                .or_default()
                .extend(item_hotkeys);
        }
    }

    Ok(merged)
}

fn contains_legacy_repeating_key(value: &serde_yaml::Value) -> bool {
    match value {
        serde_yaml::Value::Mapping(map) => map.iter().any(|(key, value)| {
            matches!(key, serde_yaml::Value::String(name) if name == "repeating")
                || contains_legacy_repeating_key(value)
        }),
        serde_yaml::Value::Sequence(seq) => seq.iter().any(contains_legacy_repeating_key),
        _ => false,
    }
}

fn find_legacy_field_child_key(value: &serde_yaml::Value) -> Option<(String, &'static str)> {
    let fields = value.get("fields")?.as_sequence()?;
    for field in fields {
        let mapping = field.as_mapping()?;
        let field_id = mapping
            .get(serde_yaml::Value::String("id".to_string()))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or("<unknown>")
            .to_string();
        if mapping.contains_key(serde_yaml::Value::String("lists".to_string())) {
            return Some((field_id, "lists"));
        }
        if mapping.contains_key(serde_yaml::Value::String("collections".to_string())) {
            return Some((field_id, "collections"));
        }
    }
    None
}

fn validate_keybindings_file(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read '{}': {err}", path.display()))?;
    let mut keybindings: KeyBindings = serde_yaml::from_str(&content)
        .map_err(|err| {
            format!(
                "failed to parse '{}': {err}. Fix: restore valid YAML keybinding lists such as `confirm: [enter]`.",
                path.display()
            )
        })?;
    ensure_hint_permutations(&mut keybindings);
    Ok(true)
}

fn normalize_items(file: &mut HierarchyFile) {
    for list in &mut file.lists {
        for item in &mut list.items {
            if item.id.is_empty() {
                item.id = slugify_id(slug_source_for_item(item));
            }
        }
    }
}

fn extract_item_hotkeys_from_value(
    value: &serde_yaml::Value,
    file: &HierarchyFile,
) -> HashMap<String, HashMap<String, String>> {
    let mut item_hotkeys = HashMap::new();
    let Some(root) = value.as_mapping() else {
        return item_hotkeys;
    };
    let Some(raw_lists) = root
        .get(serde_yaml::Value::String("lists".to_string()))
        .and_then(serde_yaml::Value::as_sequence)
    else {
        return item_hotkeys;
    };

    for (list_idx, raw_list) in raw_lists.iter().enumerate() {
        let Some(list) = file.lists.get(list_idx) else {
            continue;
        };
        let Some(raw_items) = raw_list
            .as_mapping()
            .and_then(|mapping| mapping.get(serde_yaml::Value::String("items".to_string())))
            .and_then(serde_yaml::Value::as_sequence)
        else {
            continue;
        };

        let mut hotkeys_for_list = HashMap::new();
        for (item_idx, raw_item) in raw_items.iter().enumerate() {
            let Some(item) = list.items.get(item_idx) else {
                continue;
            };
            let Some(mapping) = raw_item.as_mapping() else {
                continue;
            };
            let Some(hotkey) = mapping
                .get(serde_yaml::Value::String("hotkey".to_string()))
                .and_then(serde_yaml::Value::as_str)
            else {
                continue;
            };
            hotkeys_for_list.insert(item.id.clone(), hotkey.to_string());
        }

        if !hotkeys_for_list.is_empty() {
            item_hotkeys.insert(list.id.clone(), hotkeys_for_list);
        }
    }

    item_hotkeys
}

fn collect_authored_hotkeys(file: &HierarchyFile) -> AuthoredHotkeys {
    let mut hotkeys = AuthoredHotkeys::default();

    for section in &file.sections {
        if let Some(hotkey) = section.hotkey.clone() {
            hotkeys.sections.insert(section.id.clone(), hotkey);
        }
    }

    for field in &file.fields {
        if let Some(hotkey) = field.hotkey.clone() {
            hotkeys.fields.insert(field.id.clone(), hotkey);
        }
    }

    for list in &file.lists {
        let item_hotkeys = file.item_hotkeys.get(&list.id).cloned().unwrap_or_default();
        if !item_hotkeys.is_empty() {
            hotkeys.items.insert(list.id.clone(), item_hotkeys);
        }
    }

    hotkeys
}

fn resolve_runtime_list(
    list: &HierarchyList,
    fields_by_id: &HashMap<&str, &HierarchyField>,
    collections_by_id: &HashMap<&str, &HierarchyCollection>,
    lists_by_id: &HashMap<&str, &HierarchyList>,
) -> Result<HierarchyList, String> {
    let mut resolved = list.clone();
    for item in &mut resolved.items {
        if let Some(field_ids) = item.fields.as_ref() {
            let mut branch_fields = Vec::new();
            for field_id in field_ids {
                let field = fields_by_id.get(field_id.as_str()).ok_or_else(|| {
                    format!(
                        "list '{}' item '{}' references unknown field '{}'",
                        list.id, item.id, field_id
                    )
                })?;
                branch_fields.push(resolve_field(
                    field,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    &mut Vec::new(),
                )?);
            }
            item.branch_fields = branch_fields;
        }
        for assign in &mut item.assigns {
            let target_list = lists_by_id.get(assign.list_id.as_str()).ok_or_else(|| {
                format!(
                    "list '{}' item '{}' assigns unknown list '{}'",
                    list.id, item.id, assign.list_id
                )
            })?;
            let target_item = target_list
                .items
                .iter()
                .find(|target| target.id == assign.item_id)
                .ok_or_else(|| {
                    format!(
                        "list '{}' item '{}' assigns unknown item '{}' in list '{}'",
                        list.id, item.id, assign.item_id, assign.list_id
                    )
                })?;
            assign.output = target_item.output().to_string();
        }
    }
    Ok(resolved)
}

fn validate_merged_hierarchy(file: &HierarchyFile) -> Result<(), String> {
    let template = file
        .template
        .as_ref()
        .ok_or_else(|| "merged hierarchy is missing template".to_string())?;

    let mut global_ids: HashMap<String, TypeTag> = HashMap::new();
    register_global_ids(&mut global_ids, &file.groups, TypeTag::Group, |item| {
        &item.id
    })?;
    register_global_ids(&mut global_ids, &file.sections, TypeTag::Section, |item| {
        &item.id
    })?;
    register_global_ids(
        &mut global_ids,
        &file.collections,
        TypeTag::Collection,
        |item| &item.id,
    )?;
    register_global_ids(&mut global_ids, &file.fields, TypeTag::Field, |item| {
        &item.id
    })?;
    register_global_ids(&mut global_ids, &file.lists, TypeTag::List, |item| &item.id)?;

    let mut boilerplate_ids = HashSet::new();
    for entry in &file.boilerplate {
        if !boilerplate_ids.insert(entry.id.clone()) {
            return Err(format!(
                "duplicate boilerplate id '{}'. Fix: rename one boilerplate entry so each boilerplate id is unique.",
                entry.id
            ));
        }
    }

    validate_children(
        "template",
        &[TypeTag::Group],
        &template.contains,
        &global_ids,
    )?;
    for group in &file.groups {
        validate_children(
            &format!("group '{}'", group.id),
            &[TypeTag::Section, TypeTag::Collection],
            &group.contains,
            &global_ids,
        )?;
    }
    for section in &file.sections {
        validate_children(
            &format!("section '{}'", section.id),
            &[TypeTag::Field, TypeTag::List],
            &section.contains,
            &global_ids,
        )?;
    }
    for collection in &file.collections {
        validate_children(
            &format!("collection '{}'", collection.id),
            &[TypeTag::List],
            &collection.contains,
            &global_ids,
        )?;
    }

    for section in &file.sections {
        validate_explicit_hotkey(
            &format!("section '{}'", section.id),
            section.hotkey.as_deref(),
        )?;
    }

    for field in &file.fields {
        validate_explicit_hotkey(&format!("field '{}'", field.id), field.hotkey.as_deref())?;
    }

    for list in &file.lists {
        for item in &list.items {
            let hotkey = file
                .item_hotkeys
                .get(&list.id)
                .and_then(|items| items.get(&item.id))
                .map(String::as_str);
            validate_explicit_hotkey(&format!("list '{}' item '{}'", list.id, item.id), hotkey)?;
        }
    }

    for field in &file.fields {
        if !field.contains.is_empty() {
            validate_children(
                &format!("field '{}'", field.id),
                &[TypeTag::Field, TypeTag::List, TypeTag::Collection],
                &field.contains,
                &global_ids,
            )?;
        }
        for list_id in referenced_placeholder_ids(field.format.as_deref()) {
            let field_has_list = field.contains.iter().any(
                |child| matches!(child, HierarchyChildRef::List { list } if list == &list_id),
            );
            let field_has_collection = field.contains.iter().any(
                    |child| matches!(child, HierarchyChildRef::Collection { collection } if collection == &list_id),
                );
            let field_has_field = field.contains.iter().any(
                |child| matches!(child, HierarchyChildRef::Field { field } if field == &list_id),
            );
            if field_has_list || field_has_collection || field_has_field {
                continue;
            }
            match global_ids.get(list_id.as_str()) {
                Some(TypeTag::List) => {}
                Some(other) => {
                    return Err(format!(
                        "field '{}' expected format list '{}', found {}. {}",
                        field.id,
                        list_id,
                        kind_label(*other),
                        expected_reference_kind_fix_hint(
                            &field.id,
                            TypeTag::List,
                            *other,
                            &list_id
                        )
                    ))
                }
                None => {
                    return Err(format!(
                        "field '{}' references unknown format list '{}'. {}",
                        field.id,
                        list_id,
                        missing_reference_kind_fix_hint(&field.id, TypeTag::List, &list_id)
                    ))
                }
            }
        }
    }

    let lists_by_id: HashMap<&str, &HierarchyList> = file
        .lists
        .iter()
        .map(|list| (list.id.as_str(), list))
        .collect();
    for list in &file.lists {
        for item in &list.items {
            if let Some(field_ids) = item.fields.as_ref() {
                for field_id in field_ids {
                    match global_ids.get(field_id.as_str()) {
                        Some(TypeTag::Field) => {}
                        Some(other) => {
                            return Err(format!(
                                "list '{}' item '{}' references '{}' as field, but that id is registered as {}. Fix: update `fields:` on that item to reference field ids only.",
                                list.id,
                                item.id,
                                field_id,
                                kind_label(*other)
                            ));
                        }
                        None => {
                            return Err(format!(
                                "list '{}' item '{}' references unknown field '{}'. Fix: add a field with that id or remove it from the item's `fields:` list.",
                                list.id,
                                item.id,
                                field_id
                            ));
                        }
                    }
                }
            }
            for assign in &item.assigns {
                if assign.list_id == list.id {
                    return Err(format!(
                        "list '{}' item '{}' cannot assign back into the same list '{}'. Fix: remove that self-assignment or target a different list.",
                        list.id, item.id, assign.list_id
                    ));
                }
                let Some(target_list) = lists_by_id.get(assign.list_id.as_str()) else {
                    return Err(format!(
                        "list '{}' item '{}' assigns unknown list '{}'. Fix: point `assigns` at an existing list id.",
                        list.id, item.id, assign.list_id
                    ));
                };
                if !target_list
                    .items
                    .iter()
                    .any(|target| target.id == assign.item_id)
                {
                    return Err(format!(
                        "list '{}' item '{}' assigns unknown item '{}' in list '{}'. Fix: use an existing target item id.",
                        list.id, item.id, assign.item_id, assign.list_id
                    ));
                }
            }
        }
    }

    Ok(())
}

fn kind_label(tag: TypeTag) -> &'static str {
    match tag {
        TypeTag::Group => "group",
        TypeTag::Section => "section",
        TypeTag::Collection => "collection",
        TypeTag::Field => "field",
        TypeTag::List => "list",
    }
}

fn expected_kind_labels(expected: &[TypeTag]) -> String {
    expected
        .iter()
        .map(|tag| kind_label(*tag))
        .collect::<Vec<_>>()
        .join(", ")
}

fn missing_child_fix_hint(owner: &str, child_kind: TypeTag, child_id: &str) -> String {
    match owner {
        "template" => format!(
            "Fix: add a {} with id '{}' or update template.contains to use an existing {} id.",
            kind_label(child_kind),
            child_id,
            kind_label(child_kind)
        ),
        _ => format!(
            "Fix: add a {} with id '{}' or update {} to reference an existing {} id.",
            kind_label(child_kind),
            child_id,
            owner,
            kind_label(child_kind)
        ),
    }
}

fn wrong_kind_fix_hint(
    owner: &str,
    expected_kind: TypeTag,
    actual_kind: TypeTag,
    child_id: &str,
) -> String {
    format!(
        "Fix: update {} so '{}' is referenced as a {} or point it at an existing {} id instead of a {} id.",
        owner,
        child_id,
        kind_label(expected_kind),
        kind_label(expected_kind),
        kind_label(actual_kind)
    )
}

fn invalid_child_fix_hint(
    owner: &str,
    expected: &[TypeTag],
    child_kind: TypeTag,
    child_id: &str,
) -> String {
    format!(
        "Fix: remove {} '{}' from {} or move it under a parent that accepts {} refs. Allowed here: {}.",
        kind_label(child_kind),
        child_id,
        owner,
        kind_label(child_kind),
        expected_kind_labels(expected)
    )
}

fn duplicate_id_fix_hint() -> &'static str {
    "Fix: rename one of the conflicting ids so every group, section, collection, field, and list id is globally unique."
}

fn expected_reference_kind_fix_hint(
    field_id: &str,
    reference_kind: TypeTag,
    actual_kind: TypeTag,
    ref_id: &str,
) -> String {
    format!(
        "Fix: update field '{}' so '{}' points to a {} id, not a {} id.",
        field_id,
        ref_id,
        kind_label(reference_kind),
        kind_label(actual_kind)
    )
}

fn missing_reference_kind_fix_hint(
    field_id: &str,
    reference_kind: TypeTag,
    ref_id: &str,
) -> String {
    format!(
        "Fix: add a {} with id '{}' or update field '{}' to reference an existing {} id.",
        kind_label(reference_kind),
        ref_id,
        field_id,
        kind_label(reference_kind)
    )
}

fn register_global_ids<T, F>(
    registry: &mut HashMap<String, TypeTag>,
    items: &[T],
    tag: TypeTag,
    get_id: F,
) -> Result<(), String>
where
    F: Fn(&T) -> &str,
{
    for item in items {
        let id = get_id(item);
        if let Some(existing) = registry.insert(id.to_string(), tag) {
            return Err(format!(
                "duplicate id '{}' across {} and {}; ids must be globally unique across hierarchy kinds. {}",
                id,
                kind_label(existing),
                kind_label(tag),
                duplicate_id_fix_hint()
            ));
        }
    }
    Ok(())
}

fn validate_explicit_hotkey(owner: &str, hotkey: Option<&str>) -> Result<(), String> {
    let Some(hotkey) = hotkey else {
        return Ok(());
    };

    if hotkey.is_empty() {
        return Err(format!(
            "{owner} has an empty hotkey. Fix: use a single visible character such as `g`, or remove `hotkey`."
        ));
    }

    if hotkey.chars().count() != 1 {
        return Err(format!(
            "{owner} has invalid hotkey '{}'. Fix: use exactly one character in `hotkey`.",
            hotkey
        ));
    }

    Ok(())
}

fn validate_children(
    owner: &str,
    expected: &[TypeTag],
    children: &[HierarchyChildRef],
    global_ids: &HashMap<String, TypeTag>,
) -> Result<(), String> {
    for child in children {
        validate_child_exists(child, global_ids, owner)?;
        if !expected.contains(&child.kind()) {
            return Err(format!(
                "{} may not contain {} '{}'; allowed child kinds: {}. {}",
                owner,
                kind_label(child.kind()),
                child.id(),
                expected_kind_labels(expected),
                invalid_child_fix_hint(owner, expected, child.kind(), child.id())
            ));
        }
    }
    Ok(())
}

fn validate_child_exists(
    child: &HierarchyChildRef,
    global_ids: &HashMap<String, TypeTag>,
    owner: &str,
) -> Result<(), String> {
    match global_ids.get(child.id()) {
        Some(tag) if *tag == child.kind() => Ok(()),
        Some(tag) => Err(format!(
            "{} references '{}' as {}, but that id is registered as {}. {}",
            owner,
            child.id(),
            kind_label(child.kind()),
            kind_label(*tag),
            wrong_kind_fix_hint(owner, child.kind(), *tag, child.id())
        )),
        None => Err(format!(
            "{} references missing {} '{}'. {}",
            owner,
            kind_label(child.kind()),
            child.id(),
            missing_child_fix_hint(owner, child.kind(), child.id())
        )),
    }
}

fn slugify_id(label: &str) -> String {
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

pub fn find_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempTestDir {
        path: PathBuf,
    }

    impl TempTestDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "scribblenot-{prefix}-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("temp test dir should be created");
            Self { path }
        }
    }

    impl Drop for TempTestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn parse(yaml: &str) -> HierarchyFile {
        let mut file: HierarchyFile = serde_yaml::from_str(yaml).expect("yaml parses");
        normalize_items(&mut file);
        let raw: serde_yaml::Value = serde_yaml::from_str(yaml).expect("yaml value parses");
        file.item_hotkeys = extract_item_hotkeys_from_value(&raw, &file);
        file
    }

    #[test]
    fn child_ref_short_yaml_deserializes() {
        let refs: Vec<HierarchyChildRef> =
            serde_yaml::from_str("- section: tx_mods\n- collection: tx_regions\n")
                .expect("typed refs parse");
        assert!(matches!(
            refs.as_slice(),
            [HierarchyChildRef::Section { section }, HierarchyChildRef::Collection { collection }]
                if section == "tx_mods" && collection == "tx_regions"
        ));
    }

    #[test]
    fn item_string_deserializes_as_simple_item() {
        let list: HierarchyList =
            serde_yaml::from_str("id: demo\nitems:\n  - Alpha\n  - id: beta\n    label: Beta\n")
                .expect("list parses");
        assert_eq!(list.items[0].label.as_deref(), Some("Alpha"));
        assert_eq!(list.items[0].id, "alpha");
        assert_eq!(list.items[1].id, "beta");
    }

    #[test]
    fn field_contains_deserializes_typed_children() {
        let file = parse(concat!(
            "fields:\n",
            "  - id: requested_regions\n",
            "    label: Requested Regions\n",
            "    contains:\n",
            "      - list: appointment_type_list\n",
            "      - collection: tx_regions\n",
        ));
        let field = file.fields.first().expect("field exists");
        assert!(matches!(
            field.contains.as_slice(),
            [
                HierarchyChildRef::List { list },
                HierarchyChildRef::Collection { collection }
            ] if list == "appointment_type_list" && collection == "tx_regions"
        ));
    }

    #[test]
    fn runtime_field_contains_mixed_fields_and_lists_in_authored_order() {
        let file = parse(concat!(
            "template:\n  id: template\n  contains:\n    - group: intake\n",
            "groups:\n  - id: intake\n    contains:\n      - section: appointment\n",
            "sections:\n  - id: appointment\n    contains:\n      - field: request\n",
            "fields:\n",
            "  - id: request\n",
            "    label: Request\n",
            "    format: \"{appointment_type}{requested_regions}\"\n",
            "    contains:\n",
            "      - list: appointment_type\n",
            "      - field: requested_regions\n",
            "  - id: requested_regions\n",
            "    label: Requested Regions\n",
            "    format: \"{requested_region}\"\n",
            "    joiner_style: comma_and_the\n",
            "    max_entries: 3\n",
            "    contains:\n",
            "      - field: requested_region\n",
            "  - id: requested_region\n",
            "    label: Requested Region\n",
            "    format: \"{side}{body_part}\"\n",
            "    contains:\n",
            "      - field: side\n",
            "      - list: body_part\n",
            "  - id: side\n",
            "    label: Side\n",
            "    contains:\n",
            "      - list: side_list\n",
            "lists:\n",
            "  - id: appointment_type\n",
            "    items:\n",
            "      - Treatment massage, focusing on \n",
            "  - id: side_list\n",
            "    items:\n",
            "      - { id: left, label: Left, output: \"left \" }\n",
            "  - id: body_part\n",
            "    items:\n",
            "      - Shoulder\n",
        ));

        validate_merged_hierarchy(&file).expect("nested field hierarchy should validate");
        let runtime = hierarchy_to_runtime(file).expect("runtime build should succeed");
        let request = runtime
            .template
            .children
            .iter()
            .flat_map(|group| group.children.iter())
            .map(|node| node.config())
            .find(|section| section.id == "appointment")
            .and_then(|section| section.fields.as_ref())
            .and_then(|fields| fields.iter().find(|field| field.id == "request"))
            .expect("request field should resolve");

        assert_eq!(request.fields.len(), 2);
        assert_eq!(request.fields[0].id, "appointment_type");
        assert_eq!(request.fields[1].id, "requested_regions");
        assert_eq!(
            request.fields[1].joiner_style,
            Some(JoinerStyle::CommaAndThe)
        );
        assert_eq!(request.fields[1].max_entries, Some(3));
        assert_eq!(request.fields[1].fields.len(), 1);
        assert_eq!(request.fields[1].fields[0].id, "requested_region");
        assert_eq!(request.fields[1].fields[0].fields.len(), 2);
        assert_eq!(request.fields[1].fields[0].fields[0].id, "side");
        assert_eq!(request.fields[1].fields[0].fields[1].id, "body_part");
    }

    #[test]
    fn collection_default_enabled_deserializes() {
        let file = parse(concat!(
            "collections:\n",
            "  - id: tx_regions\n",
            "    label: Treatment Regions\n",
            "    default_enabled: true\n",
            "    contains: []\n",
        ));
        assert!(file
            .collections
            .first()
            .is_some_and(|collection| collection.default_enabled));
    }

    #[test]
    fn multi_document_file_merges_top_level_blocks() {
        let yaml = concat!(
            "fields:\n",
            "  - id: f1\n",
            "    label: First\n",
            "---\n",
            "lists:\n",
            "  - id: l1\n",
            "    items:\n",
            "      - Alpha\n",
            "---\n",
            "collections:\n",
            "  - id: c1\n",
            "    label: Demo Collection\n",
            "    contains:\n",
            "      - list: l1\n",
        );
        let file =
            parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml")).expect("parses");

        assert_eq!(file.fields.len(), 1);
        assert_eq!(file.lists.len(), 1);
        assert_eq!(file.collections.len(), 1);
        assert_eq!(file.fields[0].id, "f1");
        assert_eq!(file.lists[0].id, "l1");
        assert_eq!(file.collections[0].id, "c1");
    }

    #[test]
    fn parser_rejects_legacy_repeating_key() {
        let yaml = concat!(
            "lists:\n",
            "  - id: demo\n",
            "    repeating: comma\n",
            "    items:\n",
            "      - Alpha\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("legacy repeating key should fail");
        assert!(err.contains("deprecated key 'repeating'"));
    }

    #[test]
    fn parser_rejects_legacy_field_lists_key() {
        let yaml = concat!(
            "fields:\n",
            "  - id: appointment_requested_field\n",
            "    label: Request\n",
            "    lists: [appointment_type_list]\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("legacy field lists key should fail");
        assert!(err.contains("deprecated key 'lists'"));
        assert!(err.contains("use `contains:`"));
    }

    #[test]
    fn parser_rejects_unknown_section_key() {
        let yaml = concat!(
            "sections:\n",
            "  - id: subjective\n",
            "    label: Subjective\n",
            "    body: checklist\n",
            "    contains: []\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("unknown section key should fail");
        assert!(err.contains("unknown field `body`"));
    }

    #[test]
    fn parser_rejects_unknown_item_key() {
        let yaml = concat!(
            "lists:\n",
            "  - id: demo\n",
            "    items:\n",
            "      - id: alpha\n",
            "        label: Alpha\n",
            "        bogus: true\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("unknown item key should fail");
        assert!(err.contains("unknown field `bogus`"));
    }

    #[test]
    fn parser_accepts_authored_item_hotkey() {
        let file = parse_hierarchy_file_documents(
            concat!(
                "lists:\n",
                "  - id: demo\n",
                "    items:\n",
                "      - id: alpha\n",
                "        label: Alpha\n",
                "        hotkey: a\n",
            ),
            Path::new("inline-test.yml"),
        )
        .expect("item hotkey should still parse");

        assert_eq!(
            file.item_hotkeys
                .get("demo")
                .and_then(|hotkeys| hotkeys.get("alpha"))
                .map(String::as_str),
            Some("a")
        );
    }

    #[test]
    fn parser_rejects_authored_branch_fields_key() {
        let yaml = concat!(
            "lists:\n",
            "  - id: demo\n",
            "    items:\n",
            "      - id: alpha\n",
            "        label: Alpha\n",
            "        branch_fields: [child_field]\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("authored branch_fields should fail");
        assert!(err.contains("unknown field `branch_fields`"));
    }

    #[test]
    fn list_max_entries_without_joiner_style_is_allowed() {
        let file = parse(concat!(
            "template:\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: s\n",
            "sections:\n  - id: s\n    contains:\n      - list: demo\n",
            "lists:\n  - id: demo\n    max_entries: 2\n    items:\n      - Alpha\n",
        ));
        validate_merged_hierarchy(&file).expect("max_entries without joiner_style should load");
    }

    #[test]
    fn loader_rejects_duplicate_ids_across_kinds() {
        let file = parse(concat!(
            "template:\n  contains:\n    - group: shared\n",
            "groups:\n  - id: shared\n    contains: []\n",
            "sections:\n  - id: shared\n    contains: []\n",
        ));
        let err = validate_merged_hierarchy(&file).expect_err("duplicate id must fail");
        assert!(err.contains("duplicate id 'shared'"));
        assert!(err.contains("globally unique"));
        assert!(err.contains("Fix: rename one of the conflicting ids"));
    }

    #[test]
    fn loader_rejects_wrong_child_kind() {
        let file = parse(concat!(
            "template:\n  contains:\n    - group: intake\n",
            "groups:\n  - id: intake\n    contains:\n      - list: bad\n",
            "lists:\n  - id: bad\n    items: []\n",
        ));
        let err = validate_merged_hierarchy(&file).expect_err("bad child kind must fail");
        assert!(err.contains("may not contain"));
        assert!(err.contains("allowed child kinds"));
        assert!(err.contains("Fix: remove list 'bad'"));
    }

    #[test]
    fn loader_missing_child_error_includes_fix_hint() {
        let file = parse(concat!(
            "template:\n  contains:\n    - group: fake_group\n",
            "groups: []\n",
        ));
        let err = validate_merged_hierarchy(&file).expect_err("missing child must fail");
        assert!(err.contains("template references missing group 'fake_group'"));
        assert!(err.contains("Fix: add a group with id 'fake_group'"));
    }

    #[test]
    fn field_wrong_kind_error_includes_fix_hint() {
        let file = parse(concat!(
            "template:\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: s\n",
            "sections:\n  - id: s\n    contains:\n      - field: f\n",
            "fields:\n  - id: f\n    label: Demo\n    contains:\n      - list: demo\n",
            "collections:\n  - id: demo\n    contains: []\n",
        ));
        let err = validate_merged_hierarchy(&file).expect_err("wrong kind must fail");
        assert!(err.contains("field 'f' references 'demo' as list, but that id is registered as collection"));
        assert!(err.contains("Fix: update field 'f'"));
        assert!(err.contains("list"));
    }

    #[test]
    fn validate_merged_hierarchy_rejects_missing_item_field_ref() {
        let file = parse(concat!(
            "template:\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: s\n",
            "sections:\n  - id: s\n    contains:\n      - list: demo\n",
            "lists:\n",
            "  - id: demo\n",
            "    items:\n",
            "      - id: alpha\n",
            "        label: Alpha\n",
            "        fields: [missing_field]\n",
        ));
        let err = validate_merged_hierarchy(&file).expect_err("missing item field ref must fail");
        assert!(err.contains("list 'demo' item 'alpha' references unknown field 'missing_field'"));
    }

    #[test]
    fn item_fields_resolve_into_runtime_branch_fields() {
        let file = parse(concat!(
            "template:\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: s\n",
            "sections:\n  - id: s\n    contains:\n      - list: demo\n",
            "fields:\n",
            "  - id: child_field\n",
            "    label: Child\n",
            "    contains:\n",
            "      - list: child_list\n",
            "lists:\n",
            "  - id: demo\n",
            "    items:\n",
            "      - id: alpha\n",
            "        label: Alpha\n",
            "        fields: [child_field]\n",
            "  - id: child_list\n",
            "    items:\n",
            "      - Beta\n",
        ));
        validate_merged_hierarchy(&file).expect("item field refs should validate");
        let runtime = hierarchy_to_runtime(file).expect("runtime build should succeed");
        let section = runtime
            .template
            .children
            .iter()
            .flat_map(|group| group.children.iter())
            .map(|node| node.config())
            .find(|section| section.id == "s")
            .expect("section exists");
        let item = section.lists[0]
            .items
            .iter()
            .find(|item| item.id == "alpha")
            .expect("item exists");

        assert_eq!(item.branch_fields.len(), 1);
        assert_eq!(item.branch_fields[0].id, "child_field");
        assert_eq!(item.branch_fields[0].lists[0].id, "child_list");
    }

    #[test]
    fn collection_only_resolves_named_lists() {
        let file = parse(
            concat!(
                "template:\n  id: template\n  contains:\n    - group: treatment\n",
                "groups:\n  - id: treatment\n    contains:\n      - collection: tx_regions\n",
                "collections:\n  - id: tx_regions\n    label: Treatment Regions\n    contains:\n      - list: back\n",
                "lists:\n",
                "  - id: back\n    label: Back\n    items:\n      - Alpha\n",
                "  - id: unrelated\n    label: Unrelated\n    items:\n      - Beta\n",
            ),
        );
        validate_merged_hierarchy(&file).expect("valid merged hierarchy");
        let runtime = hierarchy_to_runtime(file).expect("runtime build succeeds");
        let collection = runtime
            .template
            .children
            .iter()
            .flat_map(|group| group.children.iter())
            .map(|node| node.config())
            .find(|section| section.id == "tx_regions")
            .expect("collection exists");
        let list_ids: Vec<&str> = collection
            .lists
            .iter()
            .map(|list| list.id.as_str())
            .collect();
        assert_eq!(list_ids, vec!["back"]);
        assert_eq!(collection.section_type, SectionBodyMode::Collection);
    }

    #[test]
    fn runtime_uses_typed_section_body_modes() {
        let file = parse(concat!(
            "template:\n  id: template\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: empty\n      - section: picker\n      - section: form\n",
            "sections:\n",
            "  - id: empty\n    contains: []\n",
            "  - id: picker\n    contains:\n      - list: choices\n",
            "  - id: form\n    contains:\n      - field: field_one\n",
            "fields:\n  - id: field_one\n    label: Field One\n",
            "lists:\n  - id: choices\n    items:\n      - Alpha\n",
        ));
        validate_merged_hierarchy(&file).expect("valid merged hierarchy");
        let runtime = hierarchy_to_runtime(file).expect("runtime build succeeds");
        let sections = flat_sections_from_template(&runtime.template);
        let modes: HashMap<&str, SectionBodyMode> = sections
            .iter()
            .map(|section| (section.id.as_str(), section.section_type))
            .collect();

        assert_eq!(modes.get("empty"), Some(&SectionBodyMode::FreeText));
        assert_eq!(modes.get("picker"), Some(&SectionBodyMode::ListSelect));
        assert_eq!(modes.get("form"), Some(&SectionBodyMode::MultiField));
    }

    #[test]
    fn runtime_preserves_authored_order() {
        let file = parse(concat!(
            "template:\n  id: template\n  contains:\n    - group: first\n    - group: second\n",
            "groups:\n",
            "  - id: first\n    contains:\n      - section: a\n      - collection: c\n",
            "  - id: second\n    contains:\n      - section: b\n",
            "sections:\n",
            "  - id: a\n    contains: []\n",
            "  - id: b\n    contains: []\n",
            "collections:\n",
            "  - id: c\n    contains:\n      - list: list_one\n",
            "lists:\n  - id: list_one\n    items: []\n",
        ));
        validate_merged_hierarchy(&file).expect("valid merged hierarchy");
        let runtime = hierarchy_to_runtime(file).expect("runtime build succeeds");
        let sections = flat_sections_from_template(&runtime.template);
        let ids: Vec<&str> = sections.iter().map(|section| section.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "c", "b"]);
    }

    #[test]
    fn runtime_navigation_matches_authored_tree_order_and_groups() {
        let file = parse(concat!(
            "template:\n  id: template\n  contains:\n    - group: first\n    - group: second\n",
            "groups:\n",
            "  - id: first\n    contains:\n      - section: a\n      - collection: c\n",
            "  - id: second\n    contains:\n      - section: b\n",
            "sections:\n",
            "  - id: a\n    contains: []\n",
            "  - id: b\n    contains: []\n",
            "collections:\n",
            "  - id: c\n    contains:\n      - list: list_one\n",
            "lists:\n  - id: list_one\n    items: []\n",
        ));
        validate_merged_hierarchy(&file).expect("valid merged hierarchy");
        let runtime = hierarchy_to_runtime(file).expect("runtime build succeeds");

        let navigation = runtime_navigation(&runtime.template);
        let entries: Vec<(&str, &str, usize)> = navigation
            .iter()
            .map(|entry| {
                (
                    entry.node_id.as_str(),
                    entry.group_id.as_str(),
                    entry.group_index,
                )
            })
            .collect();

        assert_eq!(
            entries,
            vec![("a", "first", 0), ("c", "first", 0), ("b", "second", 1)]
        );
    }

    #[test]
    fn resolve_hint_reports_exact_and_partial() {
        let hints = vec!["aa", "ab", "ba"];
        assert_eq!(resolve_hint(&hints, "aa"), HintResolveResult::Exact(0));
        assert_eq!(
            resolve_hint(&hints, "a"),
            HintResolveResult::Partial(vec![0, 1])
        );
        assert_eq!(resolve_hint(&hints, "z"), HintResolveResult::NoMatch);
    }

    #[test]
    fn assign_hint_labels_expands_duplicate_authored_prefixes() {
        let base = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let assignments =
            assign_hint_labels(&base, &[Some("n"), Some("n"), None, Some("x")], false);

        let labels = assignments
            .iter()
            .map(|assignment| assignment.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(labels[0], "n1");
        assert_eq!(labels[1], "n2");
        assert_eq!(labels[3], "x");
        assert_eq!(labels[2], "1");
        assert!(assignments[0].authored);
        assert!(assignments[1].authored);
        assert!(!assignments[2].authored);
    }

    #[test]
    fn validate_merged_hierarchy_rejects_multichar_item_hotkey() {
        let file = parse(concat!(
            "template:\n  id: demo\n  contains:\n    - group: intake\n",
            "groups:\n  - id: intake\n    contains:\n      - section: subjective\n",
            "sections:\n  - id: subjective\n    contains:\n      - list: regions\n",
            "lists:\n",
            "  - id: regions\n",
            "    items:\n",
            "      - id: shoulder\n",
            "        label: Shoulder\n",
            "        hotkey: ab\n",
        ));

        let err = validate_merged_hierarchy(&file).expect_err("multi-char hotkey must fail");
        assert!(err.contains("list 'regions' item 'shoulder'"));
        assert!(err.contains("exactly one character"));
    }

    #[test]
    fn real_data_directory_loads_under_typed_contains_schema() {
        let dir = find_data_dir();
        let app_data = AppData::load(dir).expect("real data should load");
        assert!(
            !app_data.template.children.is_empty(),
            "real authored data should load at least one group"
        );
        let group_ids: HashSet<&str> = app_data
            .template
            .children
            .iter()
            .map(|group| group.id.as_str())
            .collect();
        assert!(
            flat_sections_from_template(&app_data.template)
                .iter()
                .all(|section| group_ids.contains(section.group_id.as_str())),
            "every runtime section should belong to a loaded runtime group"
        );
    }

    #[test]
    fn real_data_groups_preserve_authored_note_labels_and_boilerplate() {
        let dir = find_data_dir();
        let hierarchy = load_hierarchy_dir(&dir).expect("real hierarchy should load");
        let app_data = AppData::load(dir).expect("real data should load");
        let authored_groups: HashMap<&str, &HierarchyGroup> = hierarchy
            .groups
            .iter()
            .map(|group| (group.id.as_str(), group))
            .collect();

        for runtime_group in &app_data.template.children {
            let authored_group = authored_groups
                .get(runtime_group.id.as_str())
                .expect("runtime group should come from authored group");
            let expected_note_label = authored_group.note_label.clone();
            let expected_nav_label = authored_group
                .nav_label
                .clone()
                .unwrap_or_else(|| authored_group.id.clone());

            assert_eq!(runtime_group.nav_label, expected_nav_label);
            assert_eq!(
                runtime_group.note.note_label.as_deref(),
                expected_note_label.as_deref()
            );
            assert_eq!(
                runtime_group.note.boilerplate_refs,
                authored_group.boilerplate_refs
            );
        }
    }

    #[test]
    fn validate_data_dir_reports_real_data_summary() {
        let summary = validate_data_dir(&find_data_dir()).expect("real data should validate");

        assert!(summary.hierarchy_file_count > 0);
        assert!(summary.keybindings_present);
        assert!(summary.group_count > 0);
        assert!(summary.section_count > 0);
        assert!(summary.list_count > 0);
    }

    #[test]
    fn validate_data_dir_rejects_invalid_keybindings_file() {
        let dir = TempTestDir::new("validate-data");
        fs::write(
            dir.path.join("sections.yml"),
            concat!(
                "template:\n",
                "  id: template\n",
                "  contains:\n",
                "    - group: intake\n",
                "groups:\n",
                "  - id: intake\n",
                "    contains:\n",
                "      - section: appointment\n",
                "sections:\n",
                "  - id: appointment\n",
                "    contains:\n",
                "      - list: appointment_type\n",
                "lists:\n",
                "  - id: appointment_type\n",
                "    items:\n",
                "      - Treatment massage\n",
            ),
        )
        .expect("hierarchy fixture should be written");
        fs::write(
            dir.path.join("keybindings.yml"),
            "nav_down: down\nconfirm: [enter]\n",
        )
        .expect("keybindings fixture should be written");

        let err =
            validate_data_dir(&dir.path).expect_err("invalid keybindings should fail validation");

        assert!(err.contains("keybindings.yml"));
        assert!(err.contains("failed to parse"));
    }

    #[test]
    fn keybindings_default_uses_nav_field_names() {
        let kb = KeyBindings::default();
        assert_eq!(kb.nav_down, vec!["down".to_string(), "n".to_string()]);
        assert_eq!(kb.nav_up, vec!["up".to_string(), "e".to_string()]);
        assert_eq!(kb.nav_left, vec!["left".to_string(), "h".to_string()]);
        assert_eq!(kb.nav_right, vec!["right".to_string(), "i".to_string()]);
    }

    #[test]
    fn keybindings_nav_fields_deserialize() {
        let kb: KeyBindings = serde_yaml::from_str(concat!(
            "nav_down: [down, n]\n",
            "nav_up: [up, e]\n",
            "select: [space]\n",
            "confirm: [enter]\n",
            "add_entry: [d]\n",
            "back: [esc]\n",
            "swap_panes: ['`']\n",
            "help: ['?']\n",
            "quit: [ctrl+q]\n",
            "nav_left: [left, h]\n",
            "nav_right: [right, i]\n",
            "hints: [a]\n",
            "super_confirm: [shift+enter]\n",
            "copy_note: [c]\n",
        ))
        .expect("new nav field names should deserialize");

        assert_eq!(kb.nav_down, vec!["down".to_string(), "n".to_string()]);
        assert_eq!(kb.nav_right, vec!["right".to_string(), "i".to_string()]);
    }

    #[test]
    fn keybindings_legacy_directional_names_fail_to_deserialize() {
        let err = serde_yaml::from_str::<KeyBindings>(concat!(
            "navigate_down: [down, n]\n",
            "navigate_up: [up, e]\n",
            "select: [space]\n",
            "confirm: [enter]\n",
            "add_entry: [d]\n",
            "back: [esc]\n",
            "swap_panes: ['`']\n",
            "help: ['?']\n",
            "quit: [ctrl+q]\n",
            "focus_left: [left, h]\n",
            "focus_right: [right, i]\n",
            "hints: [a]\n",
            "super_confirm: [shift+enter]\n",
            "copy_note: [c]\n",
        ))
        .expect_err("legacy directional field names should fail");

        assert!(err.to_string().contains("nav_down"));
    }
}
