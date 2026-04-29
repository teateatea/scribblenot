// Validation and author-facing reporting helpers extracted from data.rs.
// Owns hierarchy validation, authored parse/report helpers, fix-hint
// generation, and the validation entrypoint used by `--validate-data`.
//
// A small shared reporting layer remains in data.rs for now so the loader and
// runtime modules can keep depending on the same diagnostics helpers without
// extra churn.

use crate::data::*;
use crate::diagnostics::{ErrorKind, ErrorReport, ErrorSource};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

pub(crate) struct UnknownFieldParse {
    pub(crate) path: Option<String>,
    pub(crate) key_name: String,
}

pub(crate) struct MissingFieldParse {
    pub(crate) path: Option<String>,
    pub(crate) key_name: String,
}

pub(crate) struct InvalidTypeParse {
    pub(crate) path: Option<String>,
    pub(crate) actual_type: String,
    pub(crate) expected_type: String,
}

#[derive(Debug)]
pub(crate) struct ResolvedAuthoredOwnerContext {
    pub(crate) owner_kind: &'static str,
    pub(crate) owner_label: String,
    pub(crate) owner_id: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct UnclosedYamlStructure {
    pub(crate) structure_label: &'static str,
    pub(crate) opening_token: &'static str,
    pub(crate) closing_token: &'static str,
}

pub(crate) fn parse_unknown_field_error(message: &str) -> Option<UnknownFieldParse> {
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

pub(crate) fn parse_missing_field_error(message: &str) -> Option<MissingFieldParse> {
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

pub(crate) fn parse_invalid_type_error(message: &str) -> Option<InvalidTypeParse> {
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

pub(crate) struct UnknownVariantParse {
    pub(crate) field_name: Option<String>,
    pub(crate) provided: String,
}

pub(crate) fn parse_unknown_variant_error(message: &str) -> Option<UnknownVariantParse> {
    let (prefix, rest) = message
        .split_once(": unknown variant `")
        .or_else(|| message.split_once(": unknown variant '"))?;
    let (provided, _) = rest.split_once(['`', '\''])?;
    let field_name = prefix
        .rsplit_once('.')
        .map(|(_, field)| field.to_string())
        .or_else(|| {
            if !prefix.is_empty() {
                Some(prefix.trim().to_string())
            } else {
                None
            }
        });
    Some(UnknownVariantParse {
        field_name,
        provided: provided.to_string(),
    })
}

pub(crate) fn joiner_style_unknown_variant_report(
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

pub(crate) fn parse_unclosed_yaml_structure_error(message: &str) -> Option<UnclosedYamlStructure> {
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

pub(crate) fn detect_indented_top_level_key<'a>(
    message: &str,
    raw_line: &'a str,
) -> Option<&'static str> {
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

pub(crate) fn indented_top_level_key_report(
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

pub(crate) fn detect_invalid_child_ref_line(
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

pub(crate) fn authored_unknown_key_context(
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

pub(crate) fn authored_property_context(
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

pub(crate) fn unsupported_authored_key_report(
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

pub(crate) fn missing_required_authored_key_report(
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
    .with_param(
        "required_keys",
        format_key_list(required_keys_for_owner_kind(context.owner_kind)),
    )
}

pub(crate) fn invalid_authored_value_type_report(
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
    value
        .get("lists")
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

pub(crate) fn unclosed_yaml_structure_report(
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

pub(crate) fn normalize_unclosed_yaml_source(
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

pub(crate) fn find_unknown_top_level_key_report(
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
        return Some(unsupported_authored_key_report(
            ResolvedAuthoredOwnerContext {
                owner_kind: "document",
                owner_label: "document root".to_string(),
                owner_id: None,
            },
            key,
            source,
            path,
            doc_number,
        ));
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
        "item" => &[
            "id",
            "label",
            "default_enabled",
            "output",
            "hotkey",
            "fields",
            "assigns",
        ],
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

fn validate_keybindings_file(path: &Path) -> std::result::Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read '{}': {err}", path.display()))?;
    let mut keybindings: KeyBindings = serde_yaml::from_str(&content).map_err(|err| {
        format!(
            "failed to parse '{}': {err}. Fix: restore valid YAML keybinding lists such as `confirm: [enter]`.",
            path.display()
        )
    })?;
    ensure_hint_permutations(&mut keybindings);
    Ok(true)
}

pub(crate) fn validate_merged_hierarchy(
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
    register_global_ids(&mut global_ids, &file.groups, TypeTag::Group, |item| &item.id, index)?;
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
    register_global_ids(&mut global_ids, &file.fields, TypeTag::Field, |item| &item.id, index)?;
    register_global_ids(&mut global_ids, &file.lists, TypeTag::List, |item| &item.id, index)?;
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
        let format = field.format.as_deref().unwrap_or_default();
        for list_id in referenced_placeholder_ids(field.format.as_deref()) {
            let field_has_list = field
                .contains
                .iter()
                .any(|child| matches!(child, HierarchyChildRef::List { list } if list == &list_id));
            let field_has_collection = field.contains.iter().any(
                |child| matches!(child, HierarchyChildRef::Collection { collection } if collection == &list_id),
            );
            let field_has_field = field
                .contains
                .iter()
                .any(|child| matches!(child, HierarchyChildRef::Field { field } if field == &list_id));
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
                    );
                }
                None => {
                    if looks_like_double_brace_placeholder(format, &list_id) {
                        let normalized_id = list_id.trim_start_matches('{');
                        return Err(double_brace_format_placeholder_report(
                            &field.id,
                            normalized_id,
                            format!(
                                "field '{}' uses invalid double-brace placeholder '{{{{{}}}}}' in `format:`. Scribblenot placeholders use single braces like '{{{}}}'.",
                                field.id, normalized_id, normalized_id
                            ),
                            index.source_for(&field.id),
                        ));
                    }
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
                    ));
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
                if !target_list.items.iter().any(|target| target.id == assign.item_id) {
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

pub(crate) fn kind_label(tag: TypeTag) -> &'static str {
    match tag {
        TypeTag::Group => "group",
        TypeTag::Section => "section",
        TypeTag::Collection => "collection",
        TypeTag::Field => "field",
        TypeTag::List => "list",
        TypeTag::Boilerplate => "boilerplate",
    }
}

pub(crate) fn expected_kind_labels(expected: &[TypeTag]) -> String {
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
                            invalid_child_fix_hint(&owner.label, expected, child.kind(), child.id())
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

pub(crate) fn validate_child_exists(
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
                            wrong_kind_fix_hint(&owner.label, child.kind(), actual_kind, child.id())
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
                            wrong_kind_fix_hint(&owner.label, child.kind(), actual_kind, child.id())
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

pub(crate) fn fingerprint_kind(raw: &serde_yaml::Value) -> Vec<(&'static str, TypeTag)> {
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
