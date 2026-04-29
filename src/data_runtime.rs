// Runtime conversion helpers extracted from data.rs.
// Owns hierarchy-to-runtime transformation, flattening and navigation helpers,
// child resolution, placeholder scanning used during runtime build, and the
// small list/checklist initialization helpers that support runtime state.
//
// Validation/reporting helpers remain in data.rs for slice 5 and are imported
// here through `crate::data::*` until the later validation split.

use crate::data::*;
use crate::diagnostics::ErrorReport;
use std::collections::HashMap;

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
        let collection_matches = collections.iter().any(|collection| collection.id == list_id);
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
    for list in &lists {
        for item in &list.items {
            for list_id in referenced_placeholder_ids(item.output.as_deref()) {
                let list_is_primary = lists.iter().any(|l| l.id == list_id);
                let format_list_is_primary = format_lists.iter().any(|l| l.id == list_id);
                let collection_matches = collections.iter().any(|c| c.id == list_id);
                let field_matches = fields.iter().any(|f| f.id == list_id);
                if list_is_primary || format_list_is_primary || collection_matches || field_matches
                {
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

pub(crate) fn referenced_placeholder_ids(format: Option<&str>) -> Vec<String> {
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
