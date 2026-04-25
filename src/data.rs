use crate::diagnostics::{ErrorKind, ErrorReport, ErrorSource};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

// Pure data shape types live in `crate::data_model` after the data.rs split
// (slice 1). Re-exporting them keeps `crate::data::TypeName` callers and the
// internal data.rs code working unchanged.
pub use crate::data_model::*;
use crate::data_model::{default_hints, slug_source_for_item, slugify_id};

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

#[derive(Debug, Clone, Default)]
pub struct SourceIndex {
    pub nodes: HashMap<String, SourceNode>,
    child_refs: HashMap<ChildRefSourceKey, ErrorSource>,
}

impl SourceIndex {
    fn insert(&mut self, id: String, node: SourceNode) {
        self.nodes.entry(id).or_insert(node);
    }

    fn merge(&mut self, other: SourceIndex) {
        for (id, node) in other.nodes {
            self.insert(id, node);
        }
        for (key, source) in other.child_refs {
            self.child_refs.entry(key).or_insert(source);
        }
    }

    fn source_for(&self, id: &str) -> Option<ErrorSource> {
        self.nodes.get(id).map(|node| ErrorSource {
            file: node.file.clone(),
            line: node.line,
            quoted_line: node.quoted_line.clone(),
        })
    }

    fn insert_child_ref(&mut self, owner_id: &str, child: &HierarchyChildRef, source: ErrorSource) {
        self.child_refs
            .entry(ChildRefSourceKey {
                owner_id: owner_id.to_string(),
                child_kind: child.kind(),
                child_id: child.id().to_string(),
            })
            .or_insert(source);
    }

    fn source_for_child_ref(
        &self,
        owner_id: &str,
        child: &HierarchyChildRef,
    ) -> Option<ErrorSource> {
        self.child_refs
            .get(&ChildRefSourceKey {
                owner_id: owner_id.to_string(),
                child_kind: child.kind(),
                child_id: child.id().to_string(),
            })
            .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChildRefSourceKey {
    owner_id: String,
    child_kind: TypeTag,
    child_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceNode {
    pub file: PathBuf,
    pub line: usize,
    pub quoted_line: Option<String>,
    pub raw: serde_yaml::Value,
}

#[derive(Debug, Clone)]
pub struct LoadedHierarchy {
    pub hierarchy: HierarchyFile,
    pub source_index: SourceIndex,
}

impl AppData {
    pub fn load(data_dir: PathBuf) -> Result<Self> {
        let LoadedHierarchy {
            hierarchy,
            source_index,
        } = load_hierarchy_dir(&data_dir).map_err(anyhow::Error::new)?;
        let hotkeys = collect_authored_hotkeys(&hierarchy);
        let runtime = hierarchy_to_runtime(hierarchy, &source_index).map_err(anyhow::Error::new)?;

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

pub fn flat_sections_from_template(template: &RuntimeTemplate) -> Vec<SectionConfig> {
    template
        .children
        .iter()
        .flat_map(|group| group.children.iter())
        .filter_map(|node| node.as_config().cloned())
        .collect()
}

pub fn runtime_navigation(template: &RuntimeTemplate) -> Vec<NavigationEntry> {
    let mut entries = Vec::new();
    for (group_index, group) in template.children.iter().enumerate() {
        for node in &group.children {
            let Some(config) = node.as_config() else {
                continue;
            };
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

#[derive(Clone)]
struct ValidationOwner {
    label: String,
    kind: &'static str,
    id: Option<String>,
    source_id: Option<String>,
}

impl ValidationOwner {
    fn new(kind: &'static str, id: Option<&str>) -> Self {
        let id = id.map(str::to_string);
        Self {
            label: if kind == "template" {
                "template".to_string()
            } else {
                id.as_ref()
                    .map(|id| format!("{kind} '{id}'"))
                    .unwrap_or_else(|| kind.to_string())
            },
            kind,
            id: id.clone(),
            source_id: id,
        }
    }

    fn source(&self, index: &SourceIndex) -> Option<ErrorSource> {
        self.source_id
            .as_deref()
            .and_then(|id| index.source_for(id))
    }
}

fn report(
    kind_id: &'static str,
    message: impl Into<String>,
    source: Option<ErrorSource>,
) -> ErrorReport {
    ErrorReport::generic(kind_id, message).with_source(source)
}

fn format_placeholder_report(
    kind_id: &'static str,
    field_id: &str,
    placeholder_id: &str,
    message: impl Into<String>,
    source: Option<ErrorSource>,
) -> ErrorReport {
    report(kind_id, message, source)
        .with_param("owner_label", format!("field '{field_id}'"))
        .with_param("owner_id", field_id)
        .with_param("placeholder_id", placeholder_id)
}

fn assign_rule_report(
    kind_id: &'static str,
    source_list_id: &str,
    source_item_id: &str,
    target_list_id: &str,
    target_item_id: &str,
    message: impl Into<String>,
    source: Option<ErrorSource>,
) -> ErrorReport {
    report(kind_id, message, source)
        .with_param(
            "owner_label",
            format!("item '{source_item_id}' in list '{source_list_id}'"),
        )
        .with_param("source_list_id", source_list_id)
        .with_param("source_item_id", source_item_id)
        .with_param("target_list_id", target_list_id)
        .with_param("target_item_id", target_item_id)
}

fn child_reference_report(
    kind_id: &'static str,
    message: impl Into<String>,
    source: Option<ErrorSource>,
    owner: &ValidationOwner,
    referenced_kind: TypeTag,
    referenced_id: &str,
) -> ErrorReport {
    report(kind_id, message, source)
        .with_param("owner_label", owner.label.clone())
        .with_param("owner_kind", owner.kind)
        .with_param("owner_id", owner.id.clone().unwrap_or_default())
        .with_param("referenced_kind", kind_label(referenced_kind))
        .with_param("referenced_id", referenced_id)
        .with_param("actual_kind", "")
        .with_param("allowed_kinds", "")
        .with_param("found_file", "")
        .with_param("found_line", "")
        .with_param("found_quoted_line", "")
        .with_param("referenced_file", "")
        .with_param("referenced_line", "")
        .with_param("referenced_quoted_line", "")
}

fn child_reference_with_actual_kind(mut report: ErrorReport, actual_kind: TypeTag) -> ErrorReport {
    let actual_kind = kind_label(actual_kind).to_string();
    report.extra_params.retain(|(key, _)| key != "actual_kind");
    report.with_param("actual_kind", actual_kind)
}

fn child_reference_with_allowed_kinds(
    mut report: ErrorReport,
    allowed_kinds: &[TypeTag],
) -> ErrorReport {
    report
        .extra_params
        .retain(|(key, _)| key != "allowed_kinds");
    report.with_param("allowed_kinds", expected_kind_labels(allowed_kinds))
}

fn child_reference_with_found_source(
    mut report: ErrorReport,
    found_source: Option<ErrorSource>,
) -> ErrorReport {
    report.extra_params.retain(|(key, _)| {
        key != "found_file" && key != "found_line" && key != "found_quoted_line"
    });
    let Some(found_source) = found_source else {
        return report
            .with_param("found_file", "")
            .with_param("found_line", "")
            .with_param("found_quoted_line", "");
    };

    let found_file = found_source.file.display().to_string();
    let found_line = found_source.line.to_string();
    let found_quoted_line = found_source.quoted_line.unwrap_or_default();
    report
        .with_param("found_file", found_file)
        .with_param("found_line", found_line.clone())
        .with_param("found_quoted_line", found_quoted_line.clone())
}

fn child_reference_with_reference_source(
    mut report: ErrorReport,
    reference_source: Option<ErrorSource>,
) -> ErrorReport {
    report.extra_params.retain(|(key, _)| {
        key != "referenced_file" && key != "referenced_line" && key != "referenced_quoted_line"
    });
    let Some(reference_source) = reference_source else {
        return report
            .with_param("referenced_file", "")
            .with_param("referenced_line", "")
            .with_param("referenced_quoted_line", "");
    };

    report
        .with_param(
            "referenced_file",
            reference_source.file.display().to_string(),
        )
        .with_param("referenced_line", reference_source.line.to_string())
        .with_param(
            "referenced_quoted_line",
            reference_source.quoted_line.unwrap_or_default(),
        )
}

fn reference_source_for_child(
    owner: &ValidationOwner,
    child: &HierarchyChildRef,
    index: &SourceIndex,
) -> Option<ErrorSource> {
    owner
        .id
        .as_deref()
        .and_then(|owner_id| index.source_for_child_ref(owner_id, child))
}

pub fn hierarchy_to_runtime(
    hf: HierarchyFile,
    index: &SourceIndex,
) -> std::result::Result<RuntimeHierarchy, ErrorReport> {
    let template = hf.template.clone().ok_or_else(|| {
        report(
            "missing_template",
            "merged hierarchy is missing template",
            None,
        )
    })?;
    let template_id = template
        .id
        .clone()
        .unwrap_or_else(|| "default_template".to_string());
    let template_owner = ValidationOwner::new("template", template.id.as_deref());

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
            return Err(
                report(
                    "template_runtime_child_invalid",
                    "template runtime build expected only group refs",
                    template_owner.source(index),
                )
                .with_param("template_child_kind", kind_label(child.kind()))
                .with_param("template_child_id", child.id()),
            );
        };
        let hierarchy_group = groups_by_id.get(group.as_str()).ok_or_else(|| {
            child_reference_with_reference_source(
                child_reference_report(
                    "runtime_unknown_group",
                    format!("unknown group '{}'", group),
                    template_owner.source(index),
                    &template_owner,
                    TypeTag::Group,
                    group,
                ),
                reference_source_for_child(&template_owner, child, index),
            )
        })?;
        let group_note_label = hierarchy_group.note_label.clone();
        let group_nav_label = hierarchy_group
            .nav_label
            .clone()
            .unwrap_or_else(|| hierarchy_group.id.clone());
        let group_note = GroupNoteMeta {
            note_label: group_note_label.clone(),
        };
        let child_fallback_name = group_note_label
            .clone()
            .unwrap_or_else(|| group_nav_label.clone());
        let mut runtime_children = Vec::new();
        for child in &hierarchy_group.contains {
            match child {
                HierarchyChildRef::Section { section } => {
                    let section_data = sections_by_id.get(section.as_str()).ok_or_else(|| {
                        child_reference_with_reference_source(
                            child_reference_report(
                                "runtime_unknown_section",
                                format!("unknown section '{}'", section),
                                index.source_for(&hierarchy_group.id),
                                &ValidationOwner::new("group", Some(&hierarchy_group.id)),
                                TypeTag::Section,
                                section,
                            ),
                            reference_source_for_child(
                                &ValidationOwner::new("group", Some(&hierarchy_group.id)),
                                child,
                                index,
                            ),
                        )
                    })?;
                    let section_config = section_to_config(
                        section_data,
                        &child_fallback_name,
                        hierarchy_group.id.as_str(),
                        &fields_by_id,
                        &collections_by_id,
                        &lists_by_id,
                        index,
                    )?;
                    runtime_children.push(RuntimeNode::Section(section_config.clone()));
                    maybe_record_section_lists(
                        &section_config,
                        &mut list_data,
                        &mut checklist_data,
                    );
                }
                HierarchyChildRef::Collection { collection } => {
                    let collection_def =
                        collections_by_id.get(collection.as_str()).ok_or_else(|| {
                            child_reference_with_reference_source(
                                child_reference_report(
                                    "runtime_unknown_collection",
                                    format!("unknown collection '{}'", collection),
                                    index.source_for(&hierarchy_group.id),
                                    &ValidationOwner::new("group", Some(&hierarchy_group.id)),
                                    TypeTag::Collection,
                                    collection,
                                ),
                                reference_source_for_child(
                                    &ValidationOwner::new("group", Some(&hierarchy_group.id)),
                                    child,
                                    index,
                                ),
                            )
                        })?;
                    let collection_config = collection_to_config(
                        collection_def,
                        &child_fallback_name,
                        hierarchy_group.id.as_str(),
                        &fields_by_id,
                        &collections_by_id,
                        &lists_by_id,
                        index,
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
                            index,
                        )?],
                    );
                }
                HierarchyChildRef::Boilerplate { boilerplate } => {
                    runtime_children.push(RuntimeNode::Boilerplate(boilerplate.clone()));
                }
                other => {
                    return Err(child_reference_with_allowed_kinds(
                        child_reference_with_reference_source(
                            child_reference_report(
                                "runtime_group_child_invalid",
                                format!(
                                    "group '{}' cannot contain {:?} at runtime",
                                    hierarchy_group.id,
                                    other.kind()
                                ),
                                index.source_for(&hierarchy_group.id),
                                &ValidationOwner::new("group", Some(&hierarchy_group.id)),
                                other.kind(),
                                other.id(),
                            ),
                            reference_source_for_child(
                                &ValidationOwner::new("group", Some(&hierarchy_group.id)),
                                other,
                                index,
                            ),
                        ),
                        &[TypeTag::Section, TypeTag::Collection],
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
        .boilerplates
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
    index: &SourceIndex,
) -> std::result::Result<SectionConfig, ErrorReport> {
    let mut field_configs = Vec::new();
    let mut attached_lists = Vec::new();
    let section_owner = ValidationOwner::new("section", Some(&section.id));

    for child in &section.contains {
        match child {
            HierarchyChildRef::Field { field } => {
                let field_data = fields_by_id.get(field.as_str()).ok_or_else(|| {
                    child_reference_with_reference_source(
                        child_reference_report(
                            "runtime_unknown_field",
                            format!("unknown field '{}'", field),
                            index.source_for(&section.id),
                            &section_owner,
                            TypeTag::Field,
                            field,
                        ),
                        reference_source_for_child(&section_owner, child, index),
                    )
                })?;
                field_configs.push(resolve_field(
                    field_data,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    &mut Vec::new(),
                    index,
                )?);
            }
            HierarchyChildRef::List { list } => {
                attached_lists.push(resolve_runtime_list(
                    lists_by_id.get(list.as_str()).ok_or_else(|| {
                        child_reference_with_reference_source(
                            child_reference_report(
                                "runtime_unknown_list",
                                format!("unknown list '{}'", list),
                                index.source_for(&section.id),
                                &section_owner,
                                TypeTag::List,
                                list,
                            ),
                            reference_source_for_child(&section_owner, child, index),
                        )
                    })?,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    index,
                )?);
            }
            other => {
                return Err(child_reference_with_allowed_kinds(
                    child_reference_with_reference_source(
                        child_reference_report(
                            "runtime_section_child_invalid",
                            format!("section '{}' cannot contain {:?}", section.id, other.kind()),
                            index.source_for(&section.id),
                            &section_owner,
                            other.kind(),
                            other.id(),
                        ),
                        reference_source_for_child(&section_owner, other, index),
                    ),
                    &[TypeTag::Field, TypeTag::List],
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
    index: &SourceIndex,
) -> std::result::Result<SectionConfig, ErrorReport> {
    let resolved = resolve_collection(
        collection,
        fallback_name,
        fields_by_id,
        collections_by_id,
        lists_by_id,
        index,
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
    index: &SourceIndex,
) -> std::result::Result<ResolvedCollectionConfig, ErrorReport> {
    let mut lists = Vec::new();
    let collection_owner = ValidationOwner::new("collection", Some(&collection.id));
    for child in &collection.contains {
        match child {
            HierarchyChildRef::List { list } => lists.push(resolve_runtime_list(
                lists_by_id.get(list.as_str()).ok_or_else(|| {
                    child_reference_with_reference_source(
                        child_reference_report(
                            "runtime_unknown_list",
                            format!("unknown list '{}'", list),
                            index.source_for(&collection.id),
                            &collection_owner,
                            TypeTag::List,
                            list,
                        ),
                        reference_source_for_child(&collection_owner, child, index),
                    )
                })?,
                fields_by_id,
                collections_by_id,
                lists_by_id,
                index,
            )?),
            other => {
                return Err(child_reference_with_allowed_kinds(
                    child_reference_with_reference_source(
                        child_reference_report(
                            "runtime_collection_child_invalid",
                            format!(
                                "collection '{}' cannot contain {:?}",
                                collection.id,
                                other.kind()
                            ),
                            index.source_for(&collection.id),
                            &collection_owner,
                            other.kind(),
                            other.id(),
                        ),
                        reference_source_for_child(&collection_owner, other, index),
                    ),
                    &[TypeTag::List],
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
        .or_else(|| collection.note_label.clone())
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
    index: &SourceIndex,
) -> std::result::Result<HeaderFieldConfig, ErrorReport> {
    if visiting.iter().any(|existing| existing == &field.id) {
        let mut path = visiting.clone();
        path.push(field.id.clone());
        let cycle_path = path.join(" -> ");
        return Err(
            report(
                "runtime_field_cycle",
                format!("field cycle detected: {cycle_path}"),
                index.source_for(&field.id),
            )
            .with_param("cycle_path", cycle_path),
        );
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
        index,
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
    index: &SourceIndex,
) -> std::result::Result<HeaderFieldConfig, ErrorReport> {
    let mut fields = Vec::new();
    let mut lists = Vec::new();
    let mut collections = Vec::new();
    let mut format_lists: Vec<HierarchyList> = Vec::new();
    let field_owner = ValidationOwner::new("field", Some(&field.id));
    for child in &field.contains {
        match child {
            HierarchyChildRef::Field { field: child_id } => {
                let child = fields_by_id.get(child_id.as_str()).ok_or_else(|| {
                    child_reference_with_reference_source(
                        child_reference_report(
                            "runtime_unknown_field",
                            format!(
                                "field '{}' references unknown field '{}'",
                                field.id, child_id
                            ),
                            index.source_for(&field.id),
                            &field_owner,
                            TypeTag::Field,
                            child_id,
                        ),
                        reference_source_for_child(&field_owner, child, index),
                    )
                })?;
                fields.push(resolve_field(
                    child,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    visiting,
                    index,
                )?);
            }
            HierarchyChildRef::List { list } => {
                let list = resolve_runtime_list(
                    lists_by_id.get(list.as_str()).ok_or_else(|| {
                        child_reference_with_reference_source(
                            child_reference_report(
                                "runtime_unknown_list",
                                format!("field '{}' references unknown list '{}'", field.id, list),
                                index.source_for(&field.id),
                                &field_owner,
                                TypeTag::List,
                                list,
                            ),
                            reference_source_for_child(&field_owner, child, index),
                        )
                    })?,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    index,
                )?;
                if has_nested_fields {
                    fields.push(wrap_list_as_field(&list));
                } else {
                    lists.push(list);
                }
            }
            HierarchyChildRef::Collection { collection } => {
                let collection = collections_by_id.get(collection.as_str()).ok_or_else(|| {
                    child_reference_with_reference_source(
                        child_reference_report(
                            "runtime_unknown_collection",
                            format!(
                                "field '{}' references unknown collection '{}'",
                                field.id, collection
                            ),
                            index.source_for(&field.id),
                            &field_owner,
                            TypeTag::Collection,
                            collection,
                        ),
                        reference_source_for_child(&field_owner, child, index),
                    )
                })?;
                let resolved = resolve_collection(
                    collection,
                    &field.label,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    index,
                )?;
                if has_nested_fields {
                    fields.push(wrap_collection_as_field(&resolved));
                } else {
                    collections.push(resolved);
                }
            }
            other => {
                return Err(child_reference_with_allowed_kinds(
                    child_reference_with_reference_source(
                        child_reference_report(
                            "runtime_field_child_invalid",
                            format!("field '{}' cannot contain {:?}", field.id, other.kind()),
                            index.source_for(&field.id),
                            &field_owner,
                            other.kind(),
                            other.id(),
                        ),
                        reference_source_for_child(&field_owner, other, index),
                    ),
                    &[TypeTag::Field, TypeTag::List, TypeTag::Collection],
                ));
            }
        }
    }
    for list_id in referenced_placeholder_ids(field.format.as_deref()) {
        let list_is_primary = lists.iter().any(|list| list.id == list_id);
        let format_list_is_primary = format_lists.iter().any(|list| list.id == list_id);
        let collection_matches = collections
            .iter()
            .any(|collection| collection.id == list_id);
        let field_matches = fields.iter().any(|child| child.id == list_id);
        if list_is_primary || format_list_is_primary || collection_matches || field_matches {
            continue;
        }
        format_lists.push(resolve_runtime_list(
            lists_by_id.get(list_id.as_str()).ok_or_else(|| {
                format_placeholder_report(
                    "runtime_unknown_format_list",
                    &field.id,
                    &list_id,
                    format!(
                        "field '{}' references unknown format list '{}'",
                        field.id, list_id
                    ),
                    index.source_for(&field.id),
                )
            })?,
            fields_by_id,
            collections_by_id,
            lists_by_id,
            index,
        )?);
    }
    // Also discover format lists referenced in item output strings.
    // Only IDs that resolve to an actual list are added; branch-field references and
    // other non-list placeholders in item outputs are silently ignored.
    for list in &lists {
        for item in &list.items {
            for list_id in referenced_placeholder_ids(item.output.as_deref()) {
                let list_is_primary = lists.iter().any(|l| l.id == list_id);
                let format_list_is_primary = format_lists.iter().any(|l| l.id == list_id);
                let collection_matches = collections.iter().any(|c| c.id == list_id);
                let field_matches = fields.iter().any(|f| f.id == list_id);
                if list_is_primary || format_list_is_primary || collection_matches || field_matches {
                    continue;
                }
                let Some(list_def) = lists_by_id.get(list_id.as_str()) else {
                    continue;
                };
                format_lists.push(resolve_runtime_list(
                    list_def,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    index,
                )?);
            }
        }
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

fn read_hierarchy_dir(
    dir: &Path,
) -> std::result::Result<(HierarchyFile, SourceIndex, usize), ErrorReport> {
    let mut merged = HierarchyFile::default();
    let mut source_index = SourceIndex::default();
    let mut template_count = 0usize;
    let mut hierarchy_file_count = 0usize;

    let mut entries = fs::read_dir(dir)
        .map_err(|err| {
            ErrorReport::generic(
                "read_data_dir_failed",
                format!("failed to read data dir '{}': {err}", dir.display()),
            )
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| {
            ErrorReport::generic(
                "enumerate_data_dir_failed",
                format!("failed to enumerate data dir '{}': {err}", dir.display()),
            )
        })?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }
        if matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("keybindings.yml" | "config.yml" | "default-theme.yml")
        ) {
            continue;
        }
        hierarchy_file_count += 1;

        let content = fs::read_to_string(&path).map_err(|err| {
            ErrorReport::generic(
                "read_hierarchy_file_failed",
                format!("failed to read '{}': {err}", path.display()),
            )
        })?;
        let (file, file_sources) = parse_hierarchy_file_documents(&content, &path)?;

        if file.template.is_some() {
            template_count += 1;
            if merged.template.is_some() {
                return Err(
                    ErrorReport::generic(
                        "multiple_templates_across_files",
                        format!(
                            "multiple templates found while loading '{}'",
                            path.display()
                        ),
                    )
                    .with_param("conflicting_file", path.display().to_string()),
                );
            }
            merged.template = file.template;
        }

        merged.groups.extend(file.groups);
        merged.sections.extend(file.sections);
        merged.collections.extend(file.collections);
        merged.fields.extend(file.fields);
        merged.lists.extend(file.lists);
        merged.boilerplates.extend(file.boilerplates);
        source_index.merge(file_sources);
    }

    if template_count != 1 {
        return Err(
            ErrorReport::generic(
                "template_count_invalid",
                format!(
                    "expected exactly 1 template across data files, found {}",
                    template_count
                ),
            )
            .with_param("found_count", template_count.to_string()),
        );
    }

    Ok((merged, source_index, hierarchy_file_count))
}

pub fn load_hierarchy_dir(dir: &Path) -> std::result::Result<LoadedHierarchy, ErrorReport> {
    let (merged, source_index, _) = read_hierarchy_dir(dir)?;
    validate_merged_hierarchy(&merged, &source_index)?;
    Ok(LoadedHierarchy {
        hierarchy: merged,
        source_index,
    })
}

pub fn validate_data_dir(dir: &Path) -> std::result::Result<DataValidationSummary, ErrorReport> {
    let (merged, source_index, hierarchy_file_count) = read_hierarchy_dir(dir)?;
    validate_merged_hierarchy(&merged, &source_index)?;
    let summary = DataValidationSummary {
        hierarchy_file_count,
        keybindings_present: validate_keybindings_file(&dir.join("keybindings.yml"))
            .map_err(|message| ErrorReport::generic("keybindings_invalid", message))?,
        group_count: merged.groups.len(),
        section_count: merged.sections.len(),
        collection_count: merged.collections.len(),
        field_count: merged.fields.len(),
        list_count: merged.lists.len(),
        boilerplate_count: merged.boilerplates.len(),
    };
    hierarchy_to_runtime(merged, &source_index).map_err(|err| {
        ErrorReport::generic(
            "runtime_build_failed",
            format!("validated data could not build runtime hierarchy: {err}"),
        )
    })?;
    Ok(summary)
}

fn parse_hierarchy_file_documents(
    content: &str,
    path: &Path,
) -> std::result::Result<(HierarchyFile, SourceIndex), ErrorReport> {
    let mut merged = HierarchyFile::default();
    let mut source_index = SourceIndex::default();

    for (doc_idx, doc) in split_yaml_documents(content).into_iter().enumerate() {
        let value: serde_yaml::Value = serde_yaml::from_str(doc.text)
            .map_err(|err| yaml_doc_error(path, &doc, doc_idx + 1, err))?;
        if contains_legacy_repeating_key(&value) {
            return Err(ErrorReport::generic(
                "legacy_repeating_key",
                format!(
                    "failed to parse '{}' document {}: deprecated key 'repeating' found; use 'joiner_style'",
                    path.display(),
                    doc_idx + 1
                ),
            ));
        }
        if let Some((field_id, key_name)) = find_legacy_field_child_key(&value) {
            return Err(ErrorReport::generic(
                "legacy_field_child_key",
                format!(
                    "failed to parse '{}' document {}: field '{}' uses deprecated key '{}'; use `contains:` with typed child refs such as `- {{ list: some_list_id }}` instead.",
                    path.display(),
                    doc_idx + 1,
                    field_id,
                    key_name
                ),
            ));
        }
        if let Some(report) = find_unknown_top_level_key_report(path, &doc, doc_idx + 1, &value) {
            return Err(report);
        }
        let mut file: HierarchyFile = serde_yaml::from_str(doc.text)
            .map_err(|err| authored_yaml_doc_error(path, &doc, doc_idx + 1, &value, err))?;
        normalize_items(&mut file);
        file.item_hotkeys = extract_item_hotkeys_from_value(&value, &file);
        let doc_sources = build_source_index(&value, path, &doc);

        if file.template.is_some() {
            if merged.template.is_some() {
                return Err(ErrorReport::generic(
                    "multiple_templates_in_file",
                    format!("multiple templates found inside '{}'", path.display()),
                ));
            }
            merged.template = file.template.take();
        }

        merged.groups.extend(file.groups);
        merged.sections.extend(file.sections);
        merged.collections.extend(file.collections);
        merged.fields.extend(file.fields);
        merged.lists.extend(file.lists);
        merged.boilerplates.extend(file.boilerplates);
        for (list_id, item_hotkeys) in file.item_hotkeys {
            merged
                .item_hotkeys
                .entry(list_id)
                .or_default()
                .extend(item_hotkeys);
        }
        source_index.merge(doc_sources);
    }

    Ok((merged, source_index))
}

struct YamlDocument<'a> {
    text: &'a str,
    start_line: usize,
}

fn split_yaml_documents(content: &str) -> Vec<YamlDocument<'_>> {
    let mut docs = Vec::new();
    let mut current_start = 0usize;
    let mut line_start = 0usize;
    let mut line_no = 1usize;
    let mut first_content_line = 1usize;
    let mut seen_content = false;

    for line in content.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if is_doc_boundary(trimmed) {
            if current_start < line_start {
                let doc = &content[current_start..line_start];
                if !doc.trim().is_empty() {
                    docs.push(YamlDocument {
                        text: doc,
                        start_line: first_content_line,
                    });
                }
            }
            current_start = line_end;
            first_content_line = line_no + 1;
            seen_content = false;
        } else if !seen_content && !trimmed.trim().is_empty() {
            first_content_line = line_no;
            seen_content = true;
        }
        line_start = line_end;
        line_no += 1;
    }

    if current_start < content.len() {
        let doc = &content[current_start..];
        if !doc.trim().is_empty() {
            docs.push(YamlDocument {
                text: doc,
                start_line: first_content_line,
            });
        }
    }

    docs
}

fn is_doc_boundary(line: &str) -> bool {
    if line.starts_with(' ') || line.starts_with('\t') {
        return false;
    }
    let trimmed = line.trim();
    trimmed == "---" || trimmed == "..."
}

fn yaml_doc_error(
    path: &Path,
    doc: &YamlDocument<'_>,
    doc_number: usize,
    err: serde_yaml::Error,
) -> ErrorReport {
    let source = err.location().map(|location| ErrorSource {
        file: path.to_path_buf(),
        line: doc.start_line + location.line().saturating_sub(1),
        quoted_line: quoted_line(doc.text, location.line()),
    });
    let message = err.to_string();

    if let Some(details) = parse_unclosed_yaml_structure_error(&message) {
        return unclosed_yaml_structure_report(
            details,
            normalize_unclosed_yaml_source(doc, source),
            path,
            doc_number,
        );
    }

    if let Some(raw_line) = err
        .location()
        .and_then(|loc| doc.text.lines().nth(loc.line().saturating_sub(1)))
    {
        if let Some(key) = detect_indented_top_level_key(&message, raw_line) {
            return indented_top_level_key_report(key, source, path, doc_number);
        }
    }

    ErrorReport::generic(
        "yaml_parse_failed",
        format!(
            "failed to parse '{}' document {}: {err}",
            path.display(),
            doc_number
        ),
    )
    .with_source(source)
}

fn authored_yaml_doc_error(
    path: &Path,
    doc: &YamlDocument<'_>,
    doc_number: usize,
    value: &serde_yaml::Value,
    err: serde_yaml::Error,
) -> ErrorReport {
    let source = err.location().map(|location| ErrorSource {
        file: path.to_path_buf(),
        line: doc.start_line + location.line().saturating_sub(1),
        quoted_line: quoted_line(doc.text, location.line()),
    });
    let message = err.to_string();

    if let Some(details) = parse_unknown_field_error(&message) {
        if let Some(context) = authored_unknown_key_context(value, details.path.as_deref()) {
            return unsupported_authored_key_report(
                context,
                &details.key_name,
                source,
                path,
                doc_number,
            );
        }
    }

    if let Some(details) = parse_missing_field_error(&message) {
        if let Some(context) = authored_unknown_key_context(value, details.path.as_deref()) {
            return missing_required_authored_key_report(
                context,
                &details.key_name,
                source,
                path,
                doc_number,
            );
        }
    }

    if let Some(details) = parse_invalid_type_error(&message) {
        if let Some((context, key_name)) = authored_property_context(value, details.path.as_deref())
        {
            return invalid_authored_value_type_report(
                context,
                &key_name,
                &details.actual_type,
                &details.expected_type,
                value,
                source,
                path,
                doc_number,
            );
        }
    }

    if let Some(details) = parse_unknown_variant_error(&message) {
        if details.field_name.as_deref() == Some("joiner_style") {
            return joiner_style_unknown_variant_report(&details.provided, source, path, doc_number);
        }
    }

    if let Some(details) = parse_unclosed_yaml_structure_error(&message) {
        return unclosed_yaml_structure_report(
            details,
            normalize_unclosed_yaml_source(doc, source),
            path,
            doc_number,
        );
    }

    if let Some(rel_line) = detect_invalid_child_ref_line(&message, doc, &err) {
        return ErrorReport::generic(
            "yaml_parse_failed",
            format!(
                "failed to parse '{}' document {}: {err}",
                path.display(),
                doc_number
            ),
        )
        .with_source(Some(ErrorSource {
            file: path.to_path_buf(),
            line: doc.start_line + rel_line - 1,
            quoted_line: quoted_line(doc.text, rel_line),
        }));
    }

    ErrorReport::generic(
        "yaml_parse_failed",
        format!(
            "failed to parse '{}' document {}: {err}",
            path.display(),
            doc_number
        ),
    )
    .with_source(source)
}

fn quoted_line(text: &str, relative_line: usize) -> Option<String> {
    text.lines()
        .nth(relative_line.saturating_sub(1))
        .map(|line| line.trim().to_string())
}

fn build_source_index(
    value: &serde_yaml::Value,
    path: &Path,
    doc: &YamlDocument<'_>,
) -> SourceIndex {
    let mut index = SourceIndex::default();
    let Some(root) = value.as_mapping() else {
        return index;
    };
    let lines: Vec<&str> = doc.text.lines().collect();

    if let Some(template) = root
        .get(serde_yaml::Value::String("template".to_string()))
        .and_then(serde_yaml::Value::as_mapping)
    {
        if let Some(template_id) = template
            .get(serde_yaml::Value::String("id".to_string()))
            .and_then(serde_yaml::Value::as_str)
        {
            let anchor = find_mapping_anchor(doc.text, doc.start_line, "template", template_id);
            index.insert(
                template_id.to_string(),
                SourceNode {
                    file: path.to_path_buf(),
                    line: anchor.line,
                    quoted_line: anchor.quoted_line,
                    raw: serde_yaml::Value::Mapping(template.clone()),
                },
            );
            if let Some(contains) = template
                .get(serde_yaml::Value::String("contains".to_string()))
                .and_then(serde_yaml::Value::as_sequence)
            {
                if let Some((start_idx, end_idx)) = top_level_block_range(&lines, "template") {
                    let child_anchors = collect_child_ref_anchors(
                        &lines[start_idx..end_idx],
                        doc.start_line + start_idx,
                    );
                    for (child_value, child_anchor) in contains.iter().zip(child_anchors.iter()) {
                        if let Some(child) = child_ref_from_value(child_value) {
                            index.insert_child_ref(
                                template_id,
                                &child,
                                ErrorSource {
                                    file: path.to_path_buf(),
                                    line: child_anchor.line,
                                    quoted_line: child_anchor.quoted_line.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    let anchors = collect_top_level_entry_anchors(doc.text, doc.start_line);
    for key in [
        "groups",
        "sections",
        "collections",
        "fields",
        "lists",
        "boilerplate",
    ] {
        let Some(raw_entries) = root
            .get(serde_yaml::Value::String(key.to_string()))
            .and_then(serde_yaml::Value::as_sequence)
        else {
            continue;
        };
        let entry_anchors = anchors.get(key);
        for (idx, raw_entry) in raw_entries.iter().enumerate() {
            let Some(id) = raw_entry
                .as_mapping()
                .and_then(|mapping| mapping.get(serde_yaml::Value::String("id".to_string())))
                .and_then(serde_yaml::Value::as_str)
            else {
                continue;
            };
            let anchor = entry_anchors
                .and_then(|entries| entries.get(idx))
                .cloned()
                .unwrap_or_else(|| EntryAnchor {
                    anchor: SourceAnchor {
                        line: doc.start_line,
                        quoted_line: None,
                    },
                    start_idx: 0,
                    end_idx: 0,
                });
            index.insert(
                id.to_string(),
                SourceNode {
                    file: path.to_path_buf(),
                    line: anchor.anchor.line,
                    quoted_line: anchor.anchor.quoted_line.clone(),
                    raw: raw_entry.clone(),
                },
            );
            if let Some(contains) = raw_entry
                .as_mapping()
                .and_then(|mapping| mapping.get(serde_yaml::Value::String("contains".to_string())))
                .and_then(serde_yaml::Value::as_sequence)
            {
                let child_anchors = collect_child_ref_anchors(
                    &lines[anchor.start_idx..anchor.end_idx],
                    doc.start_line + anchor.start_idx,
                );
                for (child_value, child_anchor) in contains.iter().zip(child_anchors.iter()) {
                    if let Some(child) = child_ref_from_value(child_value) {
                        index.insert_child_ref(
                            id,
                            &child,
                            ErrorSource {
                                file: path.to_path_buf(),
                                line: child_anchor.line,
                                quoted_line: child_anchor.quoted_line.clone(),
                            },
                        );
                    }
                }
            }
        }
    }

    index
}

fn top_level_block_range(lines: &[&str], key: &str) -> Option<(usize, usize)> {
    let mut start_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if leading_spaces(line) == 0 && trimmed.starts_with(&format!("{key}:")) {
            start_idx = Some(idx);
            continue;
        }
        if start_idx.is_some()
            && leading_spaces(line) == 0
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            return Some((start_idx?, idx));
        }
    }
    start_idx.map(|start| (start, lines.len()))
}

#[derive(Debug, Clone)]
struct SourceAnchor {
    line: usize,
    quoted_line: Option<String>,
}

#[derive(Debug, Clone)]
struct EntryAnchor {
    anchor: SourceAnchor,
    start_idx: usize,
    end_idx: usize,
}

fn find_mapping_anchor(
    doc_text: &str,
    start_line: usize,
    top_level_key: &str,
    id: &str,
) -> SourceAnchor {
    let mut current_key = None::<&str>;
    for (idx, line) in doc_text.lines().enumerate() {
        let absolute_line = start_line + idx;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if leading_spaces(line) == 0 {
            current_key = top_level_key_name(trimmed);
            continue;
        }
        if current_key == Some(top_level_key) && trimmed.starts_with("id:") {
            let maybe_id = trimmed
                .trim_start_matches("id:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if maybe_id == id {
                return SourceAnchor {
                    line: absolute_line,
                    quoted_line: Some(trimmed.to_string()),
                };
            }
        }
    }
    SourceAnchor {
        line: start_line,
        quoted_line: None,
    }
}

fn collect_top_level_entry_anchors(
    doc_text: &str,
    start_line: usize,
) -> HashMap<String, Vec<EntryAnchor>> {
    let lines: Vec<&str> = doc_text.lines().collect();
    let mut anchors: HashMap<String, Vec<EntryAnchor>> = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            idx += 1;
            continue;
        }

        if leading_spaces(line) == 0 {
            current_key = top_level_key_name(trimmed)
                .filter(|key| {
                    matches!(
                        *key,
                        "groups" | "sections" | "collections" | "fields" | "lists" | "boilerplate"
                    )
                })
                .map(str::to_string);
            idx += 1;
            continue;
        }

        if leading_spaces(line) == 2 && trimmed.starts_with("- ") {
            if let Some(key) = current_key.clone() {
                let start_idx = idx;
                idx += 1;
                while idx < lines.len() {
                    let next = lines[idx];
                    let next_trimmed = next.trim();
                    if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                        idx += 1;
                        continue;
                    }
                    if leading_spaces(next) == 0
                        || (leading_spaces(next) == 2 && next_trimmed.starts_with("- "))
                    {
                        break;
                    }
                    idx += 1;
                }
                let anchor = find_entry_anchor(&lines[start_idx..idx], start_line + start_idx)
                    .unwrap_or(SourceAnchor {
                        line: start_line + start_idx,
                        quoted_line: Some(lines[start_idx].trim().to_string()),
                    });
                anchors.entry(key).or_default().push(EntryAnchor {
                    anchor,
                    start_idx,
                    end_idx: idx,
                });
                continue;
            }
        }

        idx += 1;
    }

    anchors
}

fn collect_child_ref_anchors(entry_lines: &[&str], start_line: usize) -> Vec<SourceAnchor> {
    entry_lines
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| {
            let trimmed = line.trim();
            let matched = [
                "- group:",
                "- section:",
                "- collection:",
                "- field:",
                "- list:",
            ]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix));
            matched.then(|| SourceAnchor {
                line: start_line + offset,
                quoted_line: Some(trimmed.to_string()),
            })
        })
        .collect()
}

fn child_ref_from_value(value: &serde_yaml::Value) -> Option<HierarchyChildRef> {
    serde_yaml::from_value(value.clone()).ok()
}

fn find_entry_anchor(entry_lines: &[&str], start_line: usize) -> Option<SourceAnchor> {
    for (offset, line) in entry_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("- id:") || trimmed.starts_with("id:") || trimmed.contains("{ id:") {
            return Some(SourceAnchor {
                line: start_line + offset,
                quoted_line: Some(trimmed.to_string()),
            });
        }
    }
    None
}

fn top_level_key_name(line: &str) -> Option<&str> {
    line.split_once(':').map(|(key, _)| key)
}

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
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

struct UnknownFieldParse {
    path: Option<String>,
    key_name: String,
}

struct MissingFieldParse {
    path: Option<String>,
    key_name: String,
}

struct InvalidTypeParse {
    path: Option<String>,
    actual_type: String,
    expected_type: String,
}

#[derive(Debug)]
struct ResolvedAuthoredOwnerContext {
    owner_kind: &'static str,
    owner_label: String,
    owner_id: Option<String>,
}

#[derive(Clone, Copy)]
struct UnclosedYamlStructure {
    structure_label: &'static str,
    opening_token: &'static str,
    closing_token: &'static str,
}

fn parse_unknown_field_error(message: &str) -> Option<UnknownFieldParse> {
    let (prefix, rest) = message.split_once(": unknown field `")?;
    let (key_name, _) = rest.split_once('`')?;
    let path = if prefix.trim().is_empty() {
        None
    } else {
        Some(prefix.trim().to_string())
    };
    Some(UnknownFieldParse {
        path,
        key_name: key_name.to_string(),
    })
}

fn parse_missing_field_error(message: &str) -> Option<MissingFieldParse> {
    let (prefix, rest) = message.split_once(": missing field `")?;
    let (key_name, _) = rest.split_once('`')?;
    let path = if prefix.trim().is_empty() {
        None
    } else {
        Some(prefix.trim().to_string())
    };
    Some(MissingFieldParse {
        path,
        key_name: key_name.to_string(),
    })
}

fn parse_invalid_type_error(message: &str) -> Option<InvalidTypeParse> {
    let (prefix, rest) = message.split_once(": invalid type: ")?;
    let (actual_type, expected_rest) = rest.split_once(", expected ")?;
    let expected_type = expected_rest
        .split_once(" at line ")
        .map(|(expected, _)| expected)
        .unwrap_or(expected_rest);
    let path = if prefix.trim().is_empty() {
        None
    } else {
        Some(prefix.trim().to_string())
    };
    Some(InvalidTypeParse {
        path,
        actual_type: actual_type.trim().to_string(),
        expected_type: expected_type.trim().to_string(),
    })
}

struct UnknownVariantParse {
    field_name: Option<String>,
    provided: String,
}

fn parse_unknown_variant_error(message: &str) -> Option<UnknownVariantParse> {
    let (prefix, rest) = message.split_once(": unknown variant `")
        .or_else(|| message.split_once(": unknown variant '"))?;
    let (provided, _) = rest.split_once(['`', '\''])?;
    let field_name = prefix
        .rsplit_once('.')
        .map(|(_, field)| field.to_string())
        .or_else(|| if !prefix.is_empty() { Some(prefix.trim().to_string()) } else { None });
    Some(UnknownVariantParse {
        field_name,
        provided: provided.to_string(),
    })
}

fn joiner_style_unknown_variant_report(
    provided: &str,
    source: Option<ErrorSource>,
    path: &Path,
    doc_number: usize,
) -> ErrorReport {
    ErrorReport::generic(
        "joiner_style_unknown_variant",
        format!(
            "failed to parse '{}' document {}: '{}' is not a valid joiner_style.",
            path.display(),
            doc_number,
            provided,
        ),
    )
    .with_source(source)
    .with_param("provided", provided)
}

fn parse_unclosed_yaml_structure_error(message: &str) -> Option<UnclosedYamlStructure> {
    if !message.contains("unexpected end of stream") {
        return None;
    }

    if message.contains("quoted scalar") {
        return Some(UnclosedYamlStructure {
            structure_label: "quoted value",
            opening_token: "\"",
            closing_token: "\"",
        });
    }

    if message.contains("flow sequence") {
        return Some(UnclosedYamlStructure {
            structure_label: "flow list",
            opening_token: "[",
            closing_token: "]",
        });
    }

    if message.contains("flow mapping") {
        return Some(UnclosedYamlStructure {
            structure_label: "flow mapping",
            opening_token: "{",
            closing_token: "}",
        });
    }

    None
}

fn detect_indented_top_level_key<'a>(message: &str, raw_line: &'a str) -> Option<&'static str> {
    if !message.contains("did not find expected key") {
        return None;
    }
    if !raw_line.starts_with(' ') && !raw_line.starts_with('\t') {
        return None;
    }
    let trimmed = raw_line.trim();
    allowed_keys_for_owner_kind("document")
        .iter()
        .copied()
        .find(|&key| trimmed == key || trimmed.starts_with(&format!("{key}:")))
}

fn indented_top_level_key_report(
    key: &'static str,
    source: Option<ErrorSource>,
    path: &Path,
    doc_number: usize,
) -> ErrorReport {
    ErrorReport::generic(
        "yaml_indented_top_level_key",
        format!(
            "failed to parse '{}' document {}: `{key}:` is a top-level block but has leading indentation.",
            path.display(),
            doc_number,
        ),
    )
    .with_source(source)
    .with_param("key", key)
}

fn detect_invalid_child_ref_line(
    message: &str,
    doc: &YamlDocument<'_>,
    err: &serde_yaml::Error,
) -> Option<usize> {
    if !message.contains("did not match any variant of untagged enum HierarchyChildRef") {
        return None;
    }
    let location = err.location()?;
    let start = location.line().saturating_sub(1);
    let indent = location.column().saturating_sub(1);
    let valid = [
        "group", "section", "collection", "field", "list", "boilerplate",
    ];
    doc.text
        .lines()
        .enumerate()
        .skip(start)
        .find(|(_, line)| {
            let leading = line.chars().take_while(|c| *c == ' ').count();
            if leading != indent {
                return false;
            }
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") {
                return false;
            }
            let after_dash = trimmed[2..].trim();
            after_dash
                .split_once(':')
                .map(|(key, _)| !valid.contains(&key.trim()))
                .unwrap_or(false)
        })
        .map(|(idx, _)| idx + 1)
}

fn authored_unknown_key_context(
    value: &serde_yaml::Value,
    path: Option<&str>,
) -> Option<ResolvedAuthoredOwnerContext> {
    let Some(root) = value.as_mapping() else {
        return None;
    };
    let path = path?;
    if path == "template" {
        return Some(ResolvedAuthoredOwnerContext {
            owner_kind: "template",
            owner_label: "template".to_string(),
            owner_id: root
                .get(serde_yaml::Value::String("template".to_string()))
                .and_then(serde_yaml::Value::as_mapping)
                .and_then(|mapping| mapping.get(serde_yaml::Value::String("id".to_string())))
                .and_then(serde_yaml::Value::as_str)
                .map(str::to_string),
        });
    }

    let parts = path.split('.').collect::<Vec<_>>();
    match parts.as_slice() {
        [entry] => context_for_top_level_entry(root, entry),
        [entry, "note"] => context_for_note_block(root, entry),
        [list_entry, item_entry] if item_entry.starts_with("items[") => {
            context_for_item(root, list_entry, item_entry)
        }
        [list_entry, item_entry, assign_entry]
            if item_entry.starts_with("items[") && assign_entry.starts_with("assigns[") =>
        {
            context_for_assign(root, list_entry, item_entry)
        }
        _ => None,
    }
}

fn authored_property_context(
    value: &serde_yaml::Value,
    path: Option<&str>,
) -> Option<(ResolvedAuthoredOwnerContext, String)> {
    let path = path?;
    let mut parts = path.split('.').collect::<Vec<_>>();
    let key_name = parts.pop()?.to_string();
    let owner_path = if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    };
    let context = authored_unknown_key_context(value, owner_path.as_deref())?;
    Some((context, key_name))
}

fn context_for_top_level_entry(
    root: &serde_yaml::Mapping,
    entry: &str,
) -> Option<ResolvedAuthoredOwnerContext> {
    let (bucket, idx) = parse_indexed_path_segment(entry)?;
    let owner_kind = match bucket {
        "groups" => "group",
        "sections" => "section",
        "collections" => "collection",
        "fields" => "field",
        "lists" => "list",
        "boilerplate" => "boilerplate entry",
        _ => return None,
    };
    let mapping = root
        .get(serde_yaml::Value::String(bucket.to_string()))?
        .as_sequence()?
        .get(idx)?
        .as_mapping()?;
    let owner_id = mapping
        .get(serde_yaml::Value::String("id".to_string()))
        .and_then(serde_yaml::Value::as_str)
        .map(str::to_string);
    let owner_label = match owner_kind {
        "boilerplate entry" => owner_id
            .as_ref()
            .map(|id| format!("boilerplate entry '{id}'"))
            .unwrap_or_else(|| "boilerplate entry".to_string()),
        _ => owner_id
            .as_ref()
            .map(|id| format!("{owner_kind} '{id}'"))
            .unwrap_or_else(|| owner_kind.to_string()),
    };
    Some(ResolvedAuthoredOwnerContext {
        owner_kind,
        owner_label,
        owner_id,
    })
}

fn context_for_note_block(
    root: &serde_yaml::Mapping,
    entry: &str,
) -> Option<ResolvedAuthoredOwnerContext> {
    let parent = context_for_top_level_entry(root, entry)?;
    let owner_kind = match parent.owner_kind {
        "section" => "section_note",
        "collection" => "collection_note",
        _ => return None,
    };
    Some(ResolvedAuthoredOwnerContext {
        owner_kind,
        owner_label: format!("note block on {}", parent.owner_label),
        owner_id: parent.owner_id,
    })
}

fn context_for_item(
    root: &serde_yaml::Mapping,
    list_entry: &str,
    item_entry: &str,
) -> Option<ResolvedAuthoredOwnerContext> {
    let list_ctx = context_for_top_level_entry(root, list_entry)?;
    let (_, list_idx) = parse_indexed_path_segment(list_entry)?;
    let (_, item_idx) = parse_indexed_path_segment(item_entry)?;
    let list_mapping = root
        .get(serde_yaml::Value::String("lists".to_string()))?
        .as_sequence()?
        .get(list_idx)?
        .as_mapping()?;
    let items = list_mapping
        .get(serde_yaml::Value::String("items".to_string()))?
        .as_sequence()?;
    let item = items.get(item_idx)?;
    let item_label = item
        .as_mapping()
        .and_then(|mapping| {
            mapping
                .get(serde_yaml::Value::String("id".to_string()))
                .and_then(serde_yaml::Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    mapping
                        .get(serde_yaml::Value::String("label".to_string()))
                        .and_then(serde_yaml::Value::as_str)
                        .map(str::to_string)
                })
        })
        .or_else(|| item.as_str().map(str::to_string))
        .unwrap_or_else(|| format!("item {}", item_idx + 1));
    Some(ResolvedAuthoredOwnerContext {
        owner_kind: "item",
        owner_label: format!("item '{item_label}' in {}", list_ctx.owner_label),
        owner_id: Some(item_label),
    })
}

fn context_for_assign(
    root: &serde_yaml::Mapping,
    list_entry: &str,
    item_entry: &str,
) -> Option<ResolvedAuthoredOwnerContext> {
    let item_ctx = context_for_item(root, list_entry, item_entry)?;
    Some(ResolvedAuthoredOwnerContext {
        owner_kind: "assign",
        owner_label: format!("assign entry on {}", item_ctx.owner_label),
        owner_id: item_ctx.owner_id,
    })
}

fn parse_indexed_path_segment(segment: &str) -> Option<(&str, usize)> {
    let (key, remainder) = segment.split_once('[')?;
    let idx = remainder.strip_suffix(']')?.parse().ok()?;
    Some((key, idx))
}

fn unsupported_authored_key_report(
    context: ResolvedAuthoredOwnerContext,
    key_name: &str,
    source: Option<ErrorSource>,
    path: &Path,
    doc_number: usize,
) -> ErrorReport {
    ErrorReport::generic(
        "unsupported_authored_key",
        format!(
            "failed to parse '{}' document {}: {} uses unsupported key `{}`.",
            path.display(),
            doc_number,
            context.owner_label,
            key_name
        ),
    )
    .with_source(source)
    .with_param("owner_kind", context.owner_kind)
    .with_param("owner_label", context.owner_label)
    .with_param("owner_id", context.owner_id.unwrap_or_default())
    .with_param("key_name", key_name)
    .with_param("expected_keys", format_key_list(allowed_keys_for_owner_kind(context.owner_kind)))
}

fn missing_required_authored_key_report(
    context: ResolvedAuthoredOwnerContext,
    key_name: &str,
    source: Option<ErrorSource>,
    path: &Path,
    doc_number: usize,
) -> ErrorReport {
    ErrorReport::generic(
        "missing_required_authored_key",
        format!(
            "failed to parse '{}' document {}: {} is missing required key `{}`.",
            path.display(),
            doc_number,
            context.owner_label,
            key_name
        ),
    )
    .with_source(source)
    .with_param("owner_kind", context.owner_kind)
    .with_param("owner_label", context.owner_label)
    .with_param("owner_id", context.owner_id.unwrap_or_default())
    .with_param("key_name", key_name)
    .with_param("required_keys", format_key_list(required_keys_for_owner_kind(context.owner_kind)))
}

fn invalid_authored_value_type_report(
    context: ResolvedAuthoredOwnerContext,
    key_name: &str,
    actual_type: &str,
    expected_type: &str,
    value: &serde_yaml::Value,
    source: Option<ErrorSource>,
    path: &Path,
    doc_number: usize,
) -> ErrorReport {
    let inline_map_details = source.as_ref().and_then(|source| {
        source
            .quoted_line
            .as_deref()
            .and_then(parse_inline_map_token_from_source_line)
    });
    let mut report = ErrorReport::generic(
        "invalid_authored_value_type",
        format!(
            "failed to parse '{}' document {}: {} expects {}, but `{}` was written as {}.",
            path.display(),
            doc_number,
            context.owner_label,
            expected_type,
            key_name,
            actual_type
        ),
    )
    .with_source(source)
    .with_param("owner_kind", context.owner_kind)
    .with_param("owner_label", context.owner_label)
    .with_param("owner_id", context.owner_id.unwrap_or_default())
    .with_param("key_name", key_name)
    .with_param("actual_type", actual_type)
    .with_param("expected_type", expected_type);

    if let Some((inline_map_token, inline_map_identifier)) = inline_map_details {
        let list_exists = raw_value_contains_list_id(value, &inline_map_identifier);
        report = report
            .with_param("inline_map_token", inline_map_token)
            .with_param("inline_map_identifier", inline_map_identifier)
            .with_param("inline_map_list_exists", list_exists.to_string());
    }

    report
}

fn parse_inline_map_token_from_source_line(source_line: &str) -> Option<(String, String)> {
    let (_, value) = source_line.split_once(':')?;
    let value = value.trim();
    if !value.starts_with('{') || !value.ends_with('}') || value.contains(':') {
        return None;
    }
    let identifier = value
        .strip_prefix('{')?
        .strip_suffix('}')?
        .trim()
        .to_string();
    if identifier.is_empty() {
        return None;
    }
    Some((value.to_string(), identifier))
}

fn raw_value_contains_list_id(value: &serde_yaml::Value, list_id: &str) -> bool {
    value.get("lists")
        .and_then(serde_yaml::Value::as_sequence)
        .map(|lists| {
            lists.iter().any(|list| {
                list.as_mapping()
                    .and_then(|mapping| mapping.get(serde_yaml::Value::String("id".to_string())))
                    .and_then(serde_yaml::Value::as_str)
                    == Some(list_id)
            })
        })
        .unwrap_or(false)
}

fn unclosed_yaml_structure_report(
    details: UnclosedYamlStructure,
    source: Option<ErrorSource>,
    path: &Path,
    doc_number: usize,
) -> ErrorReport {
    ErrorReport::generic(
        "yaml_unclosed_structure",
        format!(
            "failed to parse '{}' document {}: this YAML {} starts with `{}` but never closes with `{}`.",
            path.display(),
            doc_number,
            details.structure_label,
            details.opening_token,
            details.closing_token
        ),
    )
    .with_source(source)
    .with_param("structure_label", details.structure_label)
    .with_param("opening_token", details.opening_token)
    .with_param("closing_token", details.closing_token)
}

fn normalize_unclosed_yaml_source(
    doc: &YamlDocument<'_>,
    source: Option<ErrorSource>,
) -> Option<ErrorSource> {
    source.map(|mut source| {
        if source.quoted_line.is_none() && source.line > doc.start_line {
            source.line -= 1;
            source.quoted_line = quoted_line(doc.text, source.line - doc.start_line + 1);
        }
        source
    })
}

fn find_unknown_top_level_key_report(
    path: &Path,
    doc: &YamlDocument<'_>,
    doc_number: usize,
    value: &serde_yaml::Value,
) -> Option<ErrorReport> {
    let root = value.as_mapping()?;
    for key in root.keys().filter_map(serde_yaml::Value::as_str) {
        if allowed_keys_for_owner_kind("document").contains(&key) {
            continue;
        }
        let source = doc
            .text
            .lines()
            .enumerate()
            .find(|(_, line)| leading_spaces(line) == 0 && line.trim().starts_with(&format!("{key}:")))
            .map(|(idx, _)| ErrorSource {
                file: path.to_path_buf(),
                line: doc.start_line + idx,
                quoted_line: quoted_line(doc.text, idx + 1),
            });
        return Some(
            unsupported_authored_key_report(
                ResolvedAuthoredOwnerContext {
                    owner_kind: "document",
                    owner_label: "document root".to_string(),
                    owner_id: None,
                },
                key,
                source,
                path,
                doc_number,
            ),
        );
    }
    None
}

fn allowed_keys_for_owner_kind(owner_kind: &str) -> &'static [&'static str] {
    match owner_kind {
        "document" => &[
            "template",
            "groups",
            "sections",
            "collections",
            "fields",
            "lists",
            "boilerplates",
        ],
        "template" => &["id", "contains"],
        "group" => &["id", "nav_label", "note_label", "contains"],
        "section" => &[
            "id",
            "label",
            "nav_label",
            "hotkey",
            "show_field_labels",
            "contains",
            "note",
        ],
        "section_note" | "collection_note" => &["note_label"],
        "collection" => &[
            "id",
            "label",
            "nav_label",
            "note_label",
            "default_enabled",
            "joiner_style",
            "contains",
            "note",
        ],
        "field" => &[
            "id",
            "label",
            "nav_label",
            "hotkey",
            "format",
            "preview",
            "contains",
            "joiner_style",
            "max_entries",
            "max_actives",
        ],
        "list" => &[
            "id",
            "label",
            "preview",
            "sticky",
            "default",
            "modal_start",
            "joiner_style",
            "max_entries",
            "items",
        ],
        "item" => &["id", "label", "default_enabled", "output", "hotkey", "fields", "assigns"],
        "assign" => &["list", "item"],
        "boilerplate entry" => &["id", "text"],
        _ => &[],
    }
}

fn required_keys_for_owner_kind(owner_kind: &str) -> &'static [&'static str] {
    match owner_kind {
        "group" | "section" | "collection" | "list" => &["id"],
        "field" => &["id", "label"],
        "assign" => &["list", "item"],
        "boilerplate entry" => &["id", "text"],
        _ => &[],
    }
}

fn format_key_list(keys: &[&str]) -> String {
    keys.iter()
        .map(|key| format!("`{key}`"))
        .collect::<Vec<_>>()
        .join(", ")
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
    index: &SourceIndex,
) -> std::result::Result<HierarchyList, ErrorReport> {
    let mut resolved = list.clone();
    for item in &mut resolved.items {
        if let Some(field_ids) = item.fields.as_ref() {
            let mut branch_fields = Vec::new();
            for field_id in field_ids {
                let field = fields_by_id.get(field_id.as_str()).ok_or_else(|| {
                    report(
                        "runtime_unknown_branch_field",
                        format!(
                            "list '{}' item '{}' references unknown branch field '{}'",
                            list.id, item.id, field_id
                        ),
                        index.source_for(&list.id),
                    )
                })?;
                branch_fields.push(resolve_field(
                    field,
                    fields_by_id,
                    collections_by_id,
                    lists_by_id,
                    &mut Vec::new(),
                    index,
                )?);
            }
            item.branch_fields = branch_fields;
        }
        for assign in &mut item.assigns {
            let target_list = lists_by_id.get(assign.list_id.as_str()).ok_or_else(|| {
                assign_rule_report(
                    "runtime_assign_unknown_list",
                    &list.id,
                    &item.id,
                    &assign.list_id,
                    &assign.item_id,
                    format!(
                        "list '{}' item '{}' assigns unknown list '{}'",
                        list.id, item.id, assign.list_id
                    ),
                    index.source_for(&list.id),
                )
            })?;
            let target_item = target_list
                .items
                .iter()
                .find(|target| target.id == assign.item_id)
                .ok_or_else(|| {
                    assign_rule_report(
                        "runtime_assign_unknown_item",
                        &list.id,
                        &item.id,
                        &assign.list_id,
                        &assign.item_id,
                        format!(
                            "list '{}' item '{}' assigns unknown item '{}' in list '{}'",
                            list.id, item.id, assign.item_id, assign.list_id
                        ),
                        index.source_for(&list.id),
                    )
                })?;
            assign.output = target_item.output().to_string();
        }
    }
    Ok(resolved)
}

fn validate_merged_hierarchy(
    file: &HierarchyFile,
    index: &SourceIndex,
) -> std::result::Result<(), ErrorReport> {
    let template = file.template.as_ref().ok_or_else(|| {
        report(
            "missing_template",
            "merged hierarchy is missing template",
            None,
        )
    })?;

    let mut global_ids: HashMap<String, TypeTag> = HashMap::new();
    register_global_ids(
        &mut global_ids,
        &file.groups,
        TypeTag::Group,
        |item| &item.id,
        index,
    )?;
    register_global_ids(
        &mut global_ids,
        &file.sections,
        TypeTag::Section,
        |item| &item.id,
        index,
    )?;
    register_global_ids(
        &mut global_ids,
        &file.collections,
        TypeTag::Collection,
        |item| &item.id,
        index,
    )?;
    register_global_ids(
        &mut global_ids,
        &file.fields,
        TypeTag::Field,
        |item| &item.id,
        index,
    )?;
    register_global_ids(
        &mut global_ids,
        &file.lists,
        TypeTag::List,
        |item| &item.id,
        index,
    )?;

    register_global_ids(
        &mut global_ids,
        &file.boilerplates,
        TypeTag::Boilerplate,
        |item| &item.id,
        index,
    )?;

    validate_children(
        ValidationOwner::new("template", template.id.as_deref()),
        &[TypeTag::Group],
        &template.contains,
        &global_ids,
        index,
    )?;
    for group in &file.groups {
        validate_children(
            ValidationOwner::new("group", Some(&group.id)),
            &[TypeTag::Section, TypeTag::Collection, TypeTag::Boilerplate],
            &group.contains,
            &global_ids,
            index,
        )?;
    }
    for section in &file.sections {
        validate_children(
            ValidationOwner::new("section", Some(&section.id)),
            &[TypeTag::Field, TypeTag::List],
            &section.contains,
            &global_ids,
            index,
        )?;
    }
    for collection in &file.collections {
        validate_children(
            ValidationOwner::new("collection", Some(&collection.id)),
            &[TypeTag::List],
            &collection.contains,
            &global_ids,
            index,
        )?;
    }

    for section in &file.sections {
        validate_explicit_hotkey(
            &format!("section '{}'", section.id),
            section.hotkey.as_deref(),
            index.source_for(&section.id),
        )?;
    }

    for field in &file.fields {
        validate_explicit_hotkey(
            &format!("field '{}'", field.id),
            field.hotkey.as_deref(),
            index.source_for(&field.id),
        )?;
    }

    for list in &file.lists {
        for item in &list.items {
            let hotkey = file
                .item_hotkeys
                .get(&list.id)
                .and_then(|items| items.get(&item.id))
                .map(String::as_str);
            validate_explicit_hotkey(
                &format!("list '{}' item '{}'", list.id, item.id),
                hotkey,
                index.source_for(&list.id),
            )?;
        }
    }

    for field in &file.fields {
        if !field.contains.is_empty() {
            validate_children(
                ValidationOwner::new("field", Some(&field.id)),
                &[TypeTag::Field, TypeTag::List, TypeTag::Collection],
                &field.contains,
                &global_ids,
                index,
            )?;
        }
        for list_id in referenced_placeholder_ids(field.format.as_deref()) {
            let field_has_list = field
                .contains
                .iter()
                .any(|child| matches!(child, HierarchyChildRef::List { list } if list == &list_id));
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
                    return Err(
                        format_placeholder_report(
                            "field_expected_format_list_wrong_kind",
                            &field.id,
                            &list_id,
                            format!(
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
                            ),
                            index.source_for(&field.id),
                        )
                        .with_param("actual_kind", kind_label(*other)),
                    )
                }
                None => {
                    return Err(format_placeholder_report(
                        "field_unknown_format_list",
                        &field.id,
                        &list_id,
                        format!(
                            "field '{}' references unknown format list '{}'. {}",
                            field.id,
                            list_id,
                            missing_reference_kind_fix_hint(&field.id, TypeTag::List, &list_id)
                        ),
                        index.source_for(&field.id),
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
                            return Err(report(
                                "item_field_wrong_kind",
                                format!(
                                    "list '{}' item '{}' references '{}' as field, but that id is registered as {}. Fix: update `fields:` on that item to reference field ids only.",
                                    list.id,
                                    item.id,
                                    field_id,
                                    kind_label(*other)
                                ),
                                index.source_for(&list.id),
                            ));
                        }
                        None => {
                            return Err(report(
                                "item_field_unknown",
                                format!(
                                    "list '{}' item '{}' references unknown field '{}'. Fix: add a field with that id or remove it from the item's `fields:` list.",
                                    list.id,
                                    item.id,
                                    field_id
                                ),
                                index.source_for(&list.id),
                            ));
                        }
                    }
                }
            }
            for assign in &item.assigns {
                if assign.list_id == list.id {
                    return Err(assign_rule_report(
                        "assign_self_reference",
                        &list.id,
                        &item.id,
                        &assign.list_id,
                        &assign.item_id,
                        format!(
                            "list '{}' item '{}' cannot assign back into the same list '{}'. Fix: remove that self-assignment or target a different list.",
                            list.id, item.id, assign.list_id
                        ),
                        index.source_for(&list.id),
                    ));
                }
                let Some(target_list) = lists_by_id.get(assign.list_id.as_str()) else {
                    return Err(assign_rule_report(
                        "assign_unknown_list",
                        &list.id,
                        &item.id,
                        &assign.list_id,
                        &assign.item_id,
                        format!(
                            "list '{}' item '{}' assigns unknown list '{}'. Fix: point `assigns` at an existing list id.",
                            list.id, item.id, assign.list_id
                        ),
                        index.source_for(&list.id),
                    ));
                };
                if !target_list
                    .items
                    .iter()
                    .any(|target| target.id == assign.item_id)
                {
                    return Err(assign_rule_report(
                        "assign_unknown_item",
                        &list.id,
                        &item.id,
                        &assign.list_id,
                        &assign.item_id,
                        format!(
                            "list '{}' item '{}' assigns unknown item '{}' in list '{}'. Fix: use an existing target item id.",
                            list.id, item.id, assign.item_id, assign.list_id
                        ),
                        index.source_for(&list.id),
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
        TypeTag::Boilerplate => "boilerplate",
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
    index: &SourceIndex,
) -> std::result::Result<(), ErrorReport>
where
    F: Fn(&T) -> &str,
{
    for item in items {
        let id = get_id(item);
        if let Some(existing) = registry.insert(id.to_string(), tag) {
            return Err(report(
                "duplicate_id",
                format!(
                    "duplicate id '{}' across {} and {}; ids must be globally unique across hierarchy kinds. {}",
                    id,
                    kind_label(existing),
                    kind_label(tag),
                    duplicate_id_fix_hint()
                ),
                index.source_for(id),
            ));
        }
    }
    Ok(())
}

fn validate_explicit_hotkey(
    owner: &str,
    hotkey: Option<&str>,
    source: Option<ErrorSource>,
) -> std::result::Result<(), ErrorReport> {
    let Some(hotkey) = hotkey else {
        return Ok(());
    };

    if hotkey.is_empty() {
        return Err(report(
            "empty_hotkey",
            format!(
                "{owner} has an empty hotkey. Fix: use a single visible character such as `g`, or remove `hotkey`."
            ),
            source.clone(),
        ));
    }

    if hotkey.chars().count() != 1 {
        return Err(report(
            "invalid_hotkey",
            format!(
                "{owner} has invalid hotkey '{}'. Fix: use exactly one character in `hotkey`.",
                hotkey
            ),
            source,
        ));
    }

    Ok(())
}

fn validate_children(
    owner: ValidationOwner,
    expected: &[TypeTag],
    children: &[HierarchyChildRef],
    global_ids: &HashMap<String, TypeTag>,
    index: &SourceIndex,
) -> std::result::Result<(), ErrorReport> {
    for child in children {
        validate_child_exists(child, global_ids, &owner, index)?;
        if !expected.contains(&child.kind()) {
            return Err(child_reference_with_allowed_kinds(
                child_reference_with_reference_source(
                    child_reference_report(
                        "invalid_child_kind",
                        format!(
                            "{} may not contain {} '{}'; allowed child kinds: {}. {}",
                            owner.label,
                            kind_label(child.kind()),
                            child.id(),
                            expected_kind_labels(expected),
                            invalid_child_fix_hint(
                                &owner.label,
                                expected,
                                child.kind(),
                                child.id()
                            )
                        ),
                        owner.source(index),
                        &owner,
                        child.kind(),
                        child.id(),
                    ),
                    reference_source_for_child(&owner, child, index),
                ),
                expected,
            ));
        }
    }
    Ok(())
}

fn validate_child_exists(
    child: &HierarchyChildRef,
    global_ids: &HashMap<String, TypeTag>,
    owner: &ValidationOwner,
    index: &SourceIndex,
) -> std::result::Result<(), ErrorReport> {
    match global_ids.get(child.id()) {
        Some(tag) if *tag == child.kind() => Ok(()),
        Some(tag) => Err(route_wrong_kind_error(child, *tag, owner, index)),
        None => Err(child_reference_with_reference_source(
            child_reference_report(
                "missing_child",
                format!(
                    "{} references missing {} '{}'. {}",
                    owner.label,
                    kind_label(child.kind()),
                    child.id(),
                    missing_child_fix_hint(&owner.label, child.kind(), child.id())
                ),
                owner.source(index),
                owner,
                child.kind(),
                child.id(),
            ),
            reference_source_for_child(owner, child, index),
        )),
    }
}

fn route_wrong_kind_error(
    child: &HierarchyChildRef,
    actual_kind: TypeTag,
    owner: &ValidationOwner,
    index: &SourceIndex,
) -> ErrorReport {
    let source = owner.source(index);
    let found_source = index.source_for(child.id());
    let reference_source = reference_source_for_child(owner, child, index);
    let Some(node) = index.nodes.get(child.id()) else {
        return child_reference_with_actual_kind(
            child_reference_with_found_source(
                child_reference_with_reference_source(
                    child_reference_report(
                        "wrong_kind_reference",
                        format!(
                            "{} references '{}' as {}, but that id is registered as {}. {}",
                            owner.label,
                            child.id(),
                            kind_label(child.kind()),
                            kind_label(actual_kind),
                            wrong_kind_fix_hint(
                                &owner.label,
                                child.kind(),
                                actual_kind,
                                child.id()
                            )
                        ),
                        source,
                        owner,
                        child.kind(),
                        child.id(),
                    ),
                    reference_source,
                ),
                found_source,
            ),
            actual_kind,
        );
    };

    let found = fingerprint_kind(&node.raw);
    let Some((inferred_kind, found_names)) = inferred_fingerprint_kind(&found) else {
        return child_reference_with_actual_kind(
            child_reference_with_found_source(
                child_reference_with_reference_source(
                    child_reference_report(
                        "wrong_kind_reference",
                        format!(
                            "{} references '{}' as {}, but that id is registered as {}. {}",
                            owner.label,
                            child.id(),
                            kind_label(child.kind()),
                            kind_label(actual_kind),
                            wrong_kind_fix_hint(
                                &owner.label,
                                child.kind(),
                                actual_kind,
                                child.id()
                            )
                        ),
                        source,
                        owner,
                        child.kind(),
                        child.id(),
                    ),
                    reference_source.clone(),
                ),
                found_source,
            ),
            actual_kind,
        );
    };

    if inferred_kind != child.kind() {
        return child_reference_with_actual_kind(
            child_reference_with_found_source(
                child_reference_with_reference_source(
                    child_reference_report(
                        "wrong_kind_reference",
                        format!(
                            "{} references '{}' as {}, but that id is registered as {}. {}",
                            owner.label,
                            child.id(),
                            kind_label(child.kind()),
                            kind_label(actual_kind),
                            wrong_kind_fix_hint(
                                &owner.label,
                                child.kind(),
                                actual_kind,
                                child.id()
                            )
                        ),
                        source,
                        owner,
                        child.kind(),
                        child.id(),
                    ),
                    reference_source.clone(),
                ),
                found_source,
            ),
            actual_kind,
        );
    }

    let fingerprints: Vec<String> = found_names.iter().map(|name| (*name).to_string()).collect();
    match inferred_kind {
        TypeTag::List => ErrorReport {
            kind: ErrorKind::LooksLikeListMissingItems {
                id: child.id().to_string(),
                registered_as: kind_label(actual_kind).to_string(),
                found_fingerprints: fingerprints.clone(),
            },
            message: format!(
                "'{}' is referenced as a list, but is registered as a {}. Its YAML has {} which are list-only fields. It looks like a list that is missing its `items:` key. Fix: add `items:` to '{}' or check that the block is under `lists:` in your data file.",
                child.id(),
                kind_label(actual_kind),
                format_fingerprint_names(&fingerprints),
                child.id(),
            ),
            source,
            extra_params: Vec::new(),
        },
        TypeTag::Collection => ErrorReport {
            kind: ErrorKind::LooksLikeCollectionMissingKey {
                id: child.id().to_string(),
                registered_as: kind_label(actual_kind).to_string(),
                found_fingerprints: fingerprints.clone(),
            },
            message: format!(
                "'{}' is referenced as a collection, but is registered as a {}. Its YAML has {} which are collection-only fields. It looks like a collection that lost its `contains:` key or ended up under the wrong top-level key. Fix: restore `contains:` and make sure '{}' lives under `collections:`.",
                child.id(),
                kind_label(actual_kind),
                format_fingerprint_names(&fingerprints),
                child.id(),
            ),
            source,
            extra_params: Vec::new(),
        },
        TypeTag::Section | TypeTag::Group => ErrorReport {
            kind: ErrorKind::LooksLikeSectionOrGroupMissingKey {
                id: child.id().to_string(),
                inferred_kind: kind_label(inferred_kind).to_string(),
                registered_as: kind_label(actual_kind).to_string(),
                found_fingerprints: fingerprints.clone(),
            },
            message: format!(
                "'{}' is referenced as a {}, but is registered as a {}. Its YAML has {} which are {}-only fields. It looks like a {} that lost its `contains:` key or ended up under the wrong top-level key. Fix: restore `contains:` and make sure '{}' lives under `{}s:`.",
                child.id(),
                kind_label(child.kind()),
                kind_label(actual_kind),
                format_fingerprint_names(&fingerprints),
                kind_label(inferred_kind),
                kind_label(inferred_kind),
                child.id(),
                kind_label(inferred_kind),
            ),
            source,
            extra_params: Vec::new(),
        },
        TypeTag::Boilerplate => unreachable!("boilerplate refs are skipped before wrong-kind checks"),
        TypeTag::Field => child_reference_with_actual_kind(
            child_reference_with_found_source(
                child_reference_with_reference_source(
                    child_reference_report(
                        "wrong_kind_reference",
                        format!(
                            "{} references '{}' as {}, but that id is registered as {}. {}",
                            owner.label,
                            child.id(),
                            kind_label(child.kind()),
                            kind_label(actual_kind),
                            wrong_kind_fix_hint(&owner.label, child.kind(), actual_kind, child.id())
                        ),
                        source,
                        owner,
                        child.kind(),
                        child.id(),
                    ),
                    reference_source,
                ),
                found_source,
            ),
            actual_kind,
        ),
    }
}

fn fingerprint_kind(raw: &serde_yaml::Value) -> Vec<(&'static str, TypeTag)> {
    let Some(mapping) = raw.as_mapping() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    if mapping.contains_key(serde_yaml::Value::String("items".to_string())) {
        found.push(("items", TypeTag::List));
    }
    if mapping.contains_key(serde_yaml::Value::String("modal_start".to_string())) {
        found.push(("modal_start", TypeTag::List));
    }
    if mapping.contains_key(serde_yaml::Value::String("sticky".to_string())) {
        found.push(("sticky", TypeTag::List));
    }
    if mapping.contains_key(serde_yaml::Value::String("default_enabled".to_string())) {
        found.push(("default_enabled", TypeTag::Collection));
    }
    if mapping.contains_key(serde_yaml::Value::String("show_field_labels".to_string())) {
        found.push(("show_field_labels", TypeTag::Section));
    }
    if mapping.contains_key(serde_yaml::Value::String("note_label".to_string())) {
        found.push(("note_label", TypeTag::Group));
    }
    if mapping.contains_key(serde_yaml::Value::String("format".to_string())) {
        found.push(("format", TypeTag::Field));
    }
    if mapping.contains_key(serde_yaml::Value::String("max_actives".to_string())) {
        found.push(("max_actives", TypeTag::Field));
    }
    found
}

fn inferred_fingerprint_kind(
    found: &[(&'static str, TypeTag)],
) -> Option<(TypeTag, Vec<&'static str>)> {
    let first = found.first()?.1;
    if found.iter().all(|(_, tag)| *tag == first) {
        Some((first, found.iter().map(|(name, _)| *name).collect()))
    } else {
        None
    }
}

fn format_fingerprint_names(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
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

    fn parse_with_index(yaml: &str) -> (HierarchyFile, SourceIndex) {
        parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml")).expect("parses")
    }

    fn validate_with_index(
        file: &HierarchyFile,
        index: &SourceIndex,
    ) -> std::result::Result<(), ErrorReport> {
        validate_merged_hierarchy(file, index)
    }

    fn runtime_with_index(
        file: HierarchyFile,
        index: &SourceIndex,
    ) -> std::result::Result<RuntimeHierarchy, ErrorReport> {
        hierarchy_to_runtime(file, index)
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
        let (file, index) = parse_with_index(concat!(
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

        validate_with_index(&file, &index).expect("nested field hierarchy should validate");
        let runtime = runtime_with_index(file, &index).expect("runtime build should succeed");
        let request = runtime
            .template
            .children
            .iter()
            .flat_map(|group| group.children.iter())
            .filter_map(|node| node.as_config())
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
        let (file, _) =
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
    fn parser_rejects_authored_format_lists_key_with_specific_guidance() {
        let yaml = concat!(
            "fields:\n",
            "  - id: consent_pecs_field\n",
            "    label: Consent\n",
            "    format: \"{year}-{month}-{day}: {patientconsents}\"\n",
            "    contains:\n",
            "      - list: patientconsents\n",
            "    format_lists:\n",
            "      - year\n",
            "      - month\n",
            "      - day\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("authored format_lists key should fail");

        assert_eq!(err.kind_id(), "unsupported_authored_key");
        assert!(err.contains("field 'consent_pecs_field' uses unsupported key `format_lists`"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(7));
        assert_eq!(
            err.source
                .as_ref()
                .and_then(|source| source.quoted_line.as_deref()),
            Some("format_lists:")
        );
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
        assert_eq!(err.kind_id(), "unsupported_authored_key");
        assert!(err.contains("section 'subjective' uses unsupported key `body`"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(4));
        assert_eq!(
            err.source
                .as_ref()
                .and_then(|source| source.quoted_line.as_deref()),
            Some("body: checklist")
        );
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
        assert_eq!(err.kind_id(), "unsupported_authored_key");
        assert!(err.contains("item 'alpha' in list 'demo' uses unsupported key `bogus`"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(6));
    }

    #[test]
    fn parser_accepts_authored_item_hotkey() {
        let (file, _) = parse_hierarchy_file_documents(
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
        assert_eq!(err.kind_id(), "unsupported_authored_key");
        assert!(err.contains("item 'alpha' in list 'demo' uses unsupported key `branch_fields`"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(6));
    }

    #[test]
    fn parser_rejects_unknown_top_level_key() {
        let yaml = concat!("widgets:\n", "  - id: nope\n");
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("unknown top-level key should fail");

        assert_eq!(err.kind_id(), "unsupported_authored_key");
        assert!(err.contains("document root uses unsupported key `widgets`"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(1));
        assert_eq!(
            err.source
                .as_ref()
                .and_then(|source| source.quoted_line.as_deref()),
            Some("widgets:")
        );
    }

    #[test]
    fn parser_rejects_missing_required_field_key() {
        let yaml = concat!("fields:\n", "  - id: consent_glutes_field\n");
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("missing field label should fail");

        assert_eq!(err.kind_id(), "missing_required_authored_key");
        assert!(err.contains("field 'consent_glutes_field' is missing required key `label`"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(2));
        assert_eq!(
            err.source
                .as_ref()
                .and_then(|source| source.quoted_line.as_deref()),
            Some("- id: consent_glutes_field")
        );
    }

    #[test]
    fn parser_rejects_inline_map_where_field_label_expects_string() {
        let yaml = concat!(
            "fields:\n",
            "  - id: pt_info\n",
            "    label: {pt_pronouns}\n",
            "    contains:\n",
            "      - list: pt_pronouns\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("inline map field label should fail");

        assert_eq!(err.kind_id(), "invalid_authored_value_type");
        assert!(err.contains("field 'pt_info' expects a string"));
        assert!(err.contains("`label` was written as map"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(3));
        assert_eq!(
            err.source
                .as_ref()
                .and_then(|source| source.quoted_line.as_deref()),
            Some("label: {pt_pronouns}")
        );
    }

    #[test]
    fn parser_rejects_unclosed_quoted_yaml_value() {
        let yaml = concat!(
            "fields:\n",
            "  - id: consent_glutes_field\n",
            "    label: \"Consent Glutes\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("unterminated quoted scalar should fail");

        assert_eq!(err.kind_id(), "yaml_unclosed_structure");
        assert!(err.contains("quoted value starts with `\"` but never closes with `\"`"));
        assert_eq!(err.source.as_ref().map(|source| source.line), Some(3));
        assert_eq!(
            err.source
                .as_ref()
                .and_then(|source| source.quoted_line.as_deref()),
            Some("label: \"Consent Glutes")
        );
    }

    #[test]
    fn parser_rejects_indented_top_level_key() {
        let yaml = concat!(
            "sections:\n",
            "  - id: s\n",
            "    label: S\n",
            " boilerplates:\n",
            "  - id: b\n",
            "    text: hi\n",
        );
        let err = parse_hierarchy_file_documents(yaml, Path::new("inline-test.yml"))
            .expect_err("indented top-level key should fail");

        assert_eq!(err.kind_id(), "yaml_indented_top_level_key");
        assert!(err.contains("`boilerplates:`"));
    }

    #[test]
    fn fingerprint_kind_reports_list_only_fields() {
        let raw: serde_yaml::Value = serde_yaml::from_str(concat!(
            "id: demo\n",
            "modal_start: search\n",
            "sticky: true\n",
        ))
        .expect("raw yaml parses");

        assert_eq!(
            fingerprint_kind(&raw),
            vec![("modal_start", TypeTag::List), ("sticky", TypeTag::List)]
        );
    }

    #[test]
    fn wrong_kind_error_routes_to_missing_items_diagnostic_when_fingerprints_agree() {
        let mut index = SourceIndex::default();
        index.insert(
            "demo".to_string(),
            SourceNode {
                file: PathBuf::from("inline-test.yml"),
                line: 7,
                quoted_line: Some("id: demo".to_string()),
                raw: serde_yaml::from_str(concat!(
                    "id: demo\n",
                    "modal_start: search\n",
                    "sticky: true\n",
                ))
                .expect("raw yaml parses"),
            },
        );
        let mut global_ids = HashMap::new();
        global_ids.insert("demo".to_string(), TypeTag::Field);

        let err = validate_child_exists(
            &HierarchyChildRef::List {
                list: "demo".to_string(),
            },
            &global_ids,
            &ValidationOwner::new("section", Some("appointment")),
            &index,
        )
        .expect_err("wrong kind should fail");

        assert!(matches!(
            err.kind,
            ErrorKind::LooksLikeListMissingItems { ref id, ref registered_as, ref found_fingerprints }
                if id == "demo"
                    && registered_as == "field"
                    && found_fingerprints == &vec!["modal_start".to_string(), "sticky".to_string()]
        ));
        assert!(err.message.contains("missing its `items:` key"));
    }

    #[test]
    fn source_index_records_top_level_entry_id_line() {
        let (_, index) = parse_with_index(concat!(
            "fields:\n",
            "  - id: alpha\n",
            "    label: Alpha\n",
            "  - id: beta\n",
            "    label: Beta\n",
        ));

        assert_eq!(index.nodes["alpha"].line, 2);
        assert_eq!(index.nodes["beta"].line, 4);
    }

    #[test]
    fn parser_error_reports_actual_source_line() {
        let err = parse_hierarchy_file_documents(
            concat!(
                "sections:\n",
                "  - id: subjective\n",
                "    label: Subjective\n",
                "    body: checklist\n",
                "    contains: []\n",
            ),
            Path::new("inline-test.yml"),
        )
        .expect_err("unknown section key should fail");

        assert_eq!(
            err.source.as_ref().map(|source| source.line),
            Some(4),
            "error should point at the authored line with the unknown key"
        );
    }

    #[test]
    fn list_max_entries_without_joiner_style_is_allowed() {
        let (file, index) = parse_with_index(concat!(
            "template:\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: s\n",
            "sections:\n  - id: s\n    contains:\n      - list: demo\n",
            "lists:\n  - id: demo\n    max_entries: 2\n    items:\n      - Alpha\n",
        ));
        validate_with_index(&file, &index).expect("max_entries without joiner_style should load");
    }

    #[test]
    fn loader_rejects_duplicate_ids_across_kinds() {
        let (file, index) = parse_with_index(concat!(
            "template:\n  contains:\n    - group: shared\n",
            "groups:\n  - id: shared\n    contains: []\n",
            "sections:\n  - id: shared\n    contains: []\n",
        ));
        let err = validate_with_index(&file, &index).expect_err("duplicate id must fail");
        assert!(err.contains("duplicate id 'shared'"));
        assert!(err.contains("globally unique"));
        assert!(err.contains("Fix: rename one of the conflicting ids"));
    }

    #[test]
    fn loader_rejects_wrong_child_kind() {
        let (file, index) = parse_with_index(concat!(
            "template:\n  contains:\n    - group: intake\n",
            "groups:\n  - id: intake\n    contains:\n      - list: bad\n",
            "lists:\n  - id: bad\n    items: []\n",
        ));
        let err = validate_with_index(&file, &index).expect_err("bad child kind must fail");
        assert!(err.contains("may not contain"));
        assert!(err.contains("allowed child kinds"));
        assert!(err.contains("Fix: remove list 'bad'"));
    }

    #[test]
    fn loader_missing_child_error_includes_fix_hint() {
        let (file, index) = parse_with_index(concat!(
            "template:\n  id: template\n  contains:\n    - group: fake_group\n",
            "groups: []\n",
        ));
        let err = validate_with_index(&file, &index).expect_err("missing child must fail");
        assert!(err.contains("template references missing group 'fake_group'"));
        assert!(err.contains("Fix: add a group with id 'fake_group'"));
        let params = err.params().into_iter().collect::<HashMap<_, _>>();
        assert_eq!(
            params.get("owner_label").map(String::as_str),
            Some("template")
        );
        assert_eq!(
            params.get("referenced_kind").map(String::as_str),
            Some("group")
        );
        assert_eq!(
            params.get("referenced_id").map(String::as_str),
            Some("fake_group")
        );
        assert_eq!(params.get("referenced_line").map(String::as_str), Some("4"));
        assert_eq!(
            params.get("referenced_quoted_line").map(String::as_str),
            Some("- group: fake_group")
        );
    }

    #[test]
    fn wrong_kind_child_error_records_found_source_params() {
        let mut index = SourceIndex::default();
        index.insert(
            "demo".to_string(),
            SourceNode {
                file: PathBuf::from("inline-test.yml"),
                line: 7,
                quoted_line: Some("- id: demo".to_string()),
                raw: serde_yaml::from_str(
                    concat!("id: demo\n", "label: Demo\n", "contains: []\n",),
                )
                .expect("raw yaml parses"),
            },
        );
        index.insert_child_ref(
            "appointment",
            &HierarchyChildRef::Field {
                field: "demo".to_string(),
            },
            ErrorSource {
                file: PathBuf::from("inline-test.yml"),
                line: 12,
                quoted_line: Some("- field: demo".to_string()),
            },
        );
        let mut global_ids = HashMap::new();
        global_ids.insert("demo".to_string(), TypeTag::Collection);

        let err = validate_child_exists(
            &HierarchyChildRef::Field {
                field: "demo".to_string(),
            },
            &global_ids,
            &ValidationOwner::new("section", Some("appointment")),
            &index,
        )
        .expect_err("wrong kind should fail");

        let params = err.params().into_iter().collect::<HashMap<_, _>>();
        assert_eq!(
            params.get("owner_label").map(String::as_str),
            Some("section 'appointment'")
        );
        assert_eq!(
            params.get("referenced_kind").map(String::as_str),
            Some("field")
        );
        assert_eq!(
            params.get("actual_kind").map(String::as_str),
            Some("collection")
        );
        assert_eq!(
            params.get("referenced_line").map(String::as_str),
            Some("12")
        );
        assert_eq!(
            params.get("referenced_quoted_line").map(String::as_str),
            Some("- field: demo")
        );
        assert_eq!(
            params.get("found_file").map(String::as_str),
            Some("inline-test.yml")
        );
        assert_eq!(params.get("found_line").map(String::as_str), Some("7"));
        assert_eq!(
            params.get("found_quoted_line").map(String::as_str),
            Some("- id: demo")
        );
    }

    #[test]
    fn field_wrong_kind_error_includes_fix_hint() {
        let (file, index) = parse_with_index(concat!(
            "template:\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: s\n",
            "sections:\n  - id: s\n    contains:\n      - field: f\n",
            "fields:\n  - id: f\n    label: Demo\n    contains:\n      - list: demo\n",
            "collections:\n  - id: demo\n    contains: []\n",
        ));
        let err = validate_with_index(&file, &index).expect_err("wrong kind must fail");
        assert!(err.contains(
            "field 'f' references 'demo' as list, but that id is registered as collection"
        ));
        assert!(err.contains("Fix: update field 'f'"));
        assert!(err.contains("list"));
    }

    #[test]
    fn validate_merged_hierarchy_rejects_missing_item_field_ref() {
        let (file, index) = parse_with_index(concat!(
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
        let err = validate_with_index(&file, &index).expect_err("missing item field ref must fail");
        assert!(err.contains("list 'demo' item 'alpha' references unknown field 'missing_field'"));
    }

    #[test]
    fn item_fields_resolve_into_runtime_branch_fields() {
        let (file, index) = parse_with_index(concat!(
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
        validate_with_index(&file, &index).expect("item field refs should validate");
        let runtime = runtime_with_index(file, &index).expect("runtime build should succeed");
        let section = runtime
            .template
            .children
            .iter()
            .flat_map(|group| group.children.iter())
            .filter_map(|node| node.as_config())
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
        let (file, index) = parse_with_index(
            concat!(
                "template:\n  id: template\n  contains:\n    - group: treatment\n",
                "groups:\n  - id: treatment\n    contains:\n      - collection: tx_regions\n",
                "collections:\n  - id: tx_regions\n    label: Treatment Regions\n    contains:\n      - list: back\n",
                "lists:\n",
                "  - id: back\n    label: Back\n    items:\n      - Alpha\n",
                "  - id: unrelated\n    label: Unrelated\n    items:\n      - Beta\n",
            ),
        );
        validate_with_index(&file, &index).expect("valid merged hierarchy");
        let runtime = runtime_with_index(file, &index).expect("runtime build succeeds");
        let collection = runtime
            .template
            .children
            .iter()
            .flat_map(|group| group.children.iter())
            .filter_map(|node| node.as_config())
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
        let (file, index) = parse_with_index(concat!(
            "template:\n  id: template\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: empty\n      - section: picker\n      - section: form\n",
            "sections:\n",
            "  - id: empty\n    contains: []\n",
            "  - id: picker\n    contains:\n      - list: choices\n",
            "  - id: form\n    contains:\n      - field: field_one\n",
            "fields:\n  - id: field_one\n    label: Field One\n",
            "lists:\n  - id: choices\n    items:\n      - Alpha\n",
        ));
        validate_with_index(&file, &index).expect("valid merged hierarchy");
        let runtime = runtime_with_index(file, &index).expect("runtime build succeeds");
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
        let (file, index) = parse_with_index(concat!(
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
        validate_with_index(&file, &index).expect("valid merged hierarchy");
        let runtime = runtime_with_index(file, &index).expect("runtime build succeeds");
        let sections = flat_sections_from_template(&runtime.template);
        let ids: Vec<&str> = sections.iter().map(|section| section.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "c", "b"]);
    }

    #[test]
    fn runtime_navigation_matches_authored_tree_order_and_groups() {
        let (file, index) = parse_with_index(concat!(
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
        validate_with_index(&file, &index).expect("valid merged hierarchy");
        let runtime = runtime_with_index(file, &index).expect("runtime build succeeds");

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
        let (file, index) = parse_with_index(concat!(
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

        let err = validate_with_index(&file, &index).expect_err("multi-char hotkey must fail");
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
            .hierarchy
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
        assert_eq!(kb.theme_reload, vec!["/".to_string()]);
        assert_eq!(kb.data_reload, vec!["\\".to_string()]);
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
            "theme_reload: [/]\n",
            "data_reload: ['\\']\n",
        ))
        .expect("new nav field names should deserialize");

        assert_eq!(kb.nav_down, vec!["down".to_string(), "n".to_string()]);
        assert_eq!(kb.nav_right, vec!["right".to_string(), "i".to_string()]);
        assert_eq!(kb.theme_reload, vec!["/".to_string()]);
        assert_eq!(kb.data_reload, vec!["\\".to_string()]);
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
