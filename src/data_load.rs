// YAML load and merge helpers extracted from data.rs.
// Owns directory scanning (`read_hierarchy_dir`), the multi-file merge
// entrypoint (`load_hierarchy_dir`), per-document parsing
// (`parse_hierarchy_file_documents`, `split_yaml_documents`,
// `is_doc_boundary`), authored-error decoration (`yaml_doc_error`,
// `authored_yaml_doc_error`), the `YamlDocument` view, and the source-index
// builder (`build_source_index`) that ties parsed YAML back to source anchors.
//
// Items are kept `pub(crate)` so data.rs can re-export them via
// `pub use crate::data_load::*` and validation/runtime code in data.rs can
// continue to call them by their bare names.

use crate::data::*;
use crate::data_model::HierarchyFile;
use crate::data_source::{
    EntryAnchor, SourceAnchor, SourceIndex, SourceNode, collect_child_ref_anchors,
    collect_top_level_entry_anchors, child_ref_from_value, find_mapping_anchor, quoted_line,
    top_level_block_range,
};
use crate::diagnostics::{ErrorReport, ErrorSource};
use std::fs;
use std::path::Path;

pub(crate) fn read_hierarchy_dir(
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

pub(crate) fn parse_hierarchy_file_documents(
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

pub(crate) struct YamlDocument<'a> {
    pub(crate) text: &'a str,
    pub(crate) start_line: usize,
}

pub(crate) fn split_yaml_documents(content: &str) -> Vec<YamlDocument<'_>> {
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

pub(crate) fn yaml_doc_error(
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

pub(crate) fn authored_yaml_doc_error(
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

pub(crate) fn build_source_index(
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
