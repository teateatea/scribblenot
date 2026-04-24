use crate::diagnostics::report::ErrorReport;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MessageEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub fix: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct Messages {
    entries: HashMap<String, MessageEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedError {
    pub id: String,
    pub title: String,
    pub description: String,
    pub description_segments: Vec<RenderedTextSegment>,
    pub fix: String,
    pub fix_segments: Vec<RenderedTextSegment>,
    pub source: Option<RenderedErrorSource>,
    pub source_blocks: Vec<RenderedErrorSourceBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedErrorSource {
    pub location: String,
    pub quoted_line: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedErrorSourceBlock {
    pub file_name: String,
    pub file_path: String,
    pub lines: Vec<RenderedErrorSourceLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedErrorSourceLine {
    pub line: usize,
    pub quoted_line: String,
    pub role: RenderedErrorSourceRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderedErrorSourceRole {
    Owner,
    Reference,
    Found,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTextSegment {
    pub text: String,
    pub is_param: bool,
}

impl Messages {
    pub fn load() -> Self {
        let mut messages = Self::default();
        for entry in builtin_entries() {
            messages.entries.insert(entry.id.to_string(), entry);
        }
        messages
    }

    pub fn render(&self, report: &ErrorReport) -> RenderedError {
        let params = report_params(report);
        let kind_id = report.kind_id();
        let Some(entry) = entry_for_report(self, report, &params) else {
            return RenderedError {
                id: kind_id.to_string(),
                title: title_from_kind(kind_id),
                description: report.message.clone(),
                description_segments: vec![RenderedTextSegment {
                    text: report.message.clone(),
                    is_param: false,
                }],
                fix: String::new(),
                fix_segments: Vec::new(),
                source: render_source(report),
                source_blocks: render_source_blocks(report),
            };
        };

        let description_segments = substitute_segments(entry.description, &params);
        let fix_segments = substitute_segments(entry.fix, &params);

        RenderedError {
            id: kind_id.to_string(),
            title: substitute(entry.title, &params),
            description: flatten_segments(&description_segments),
            description_segments,
            fix: flatten_segments(&fix_segments),
            fix_segments,
            source: render_source(report),
            source_blocks: render_source_blocks(report),
        }
    }
}

fn entry_for_report<'a>(
    messages: &'a Messages,
    report: &ErrorReport,
    params: &HashMap<String, String>,
) -> Option<&'a MessageEntry> {
    if report.kind_id() == "invalid_child_kind"
        && params.get("owner_kind").map(String::as_str) == Some("section")
        && params.get("referenced_kind").map(String::as_str) == Some("collection")
    {
        return Some(&SECTION_COLLECTION_INVALID_ENTRY);
    }
    messages.entries.get(report.kind_id())
}

static SECTION_COLLECTION_INVALID_ENTRY: MessageEntry = MessageEntry {
    id: "invalid_child_kind",
    title: "Sections Cannot Directly Contain Collections (Yet)",
    description:
        "{owner_label} may not contain *collection* '{referenced_id}' directly.\n\nSections currently accept **fields** and **lists** only.",
    fix:
        "The recommended path is **section -> field -> collection**.\n\n{owner_label} should contain a **field**, and that field should contain `collection: {referenced_id}`.\n\n(Direct **section -> collection** support is later on the roadmap. If you have a concrete use case for it, consider moving it up.)",
};

fn builtin_entries() -> Vec<MessageEntry> {
    vec![
        MessageEntry {
            id: "missing_child",
            title: "ID Not Found",
            description: "{owner_label} references {referenced_kind} '{referenced_id}', but no {referenced_kind} with that id was found.",
            fix: "a) Update the ID: *'{referenced_id}'* -> **existing {referenced_kind}**:\n     ln {source_line} `{source_quoted_line}`\n     ...\n     **ln {referenced_line}** `  - {referenced_kind}: **correct_{referenced_kind}_id**`\n     \nb) Create the '{referenced_id}' **{referenced_kind}**:\n     `**{referenced_kind}:**`\n     `  **- id:** {referenced_id}`\n",
        },
        MessageEntry {
            id: "wrong_kind_reference",
            title: "[Type: ID] Mismatch",
            description: "{owner_label} references '{referenced_id}' as a {referenced_kind}, but that id is registered as a {actual_kind}.",
            fix: "a) Update the type: *{referenced_kind}* -> **{actual_kind}**:\n     ln {source_line} `{source_quoted_line}`\n     ...\n     **ln {referenced_line}** `  - **{actual_kind}**: {referenced_id}`\n  \nb) Update the ID: *'{referenced_id}'* -> **existing {referenced_kind}**:\n     ln {source_line} `{source_quoted_line}`\n     ...\n     **ln {referenced_line}** `  - {referenced_kind}: **correct_{referenced_kind}_id**`\n",
        },
        MessageEntry {
            id: "invalid_child_kind",
            title: "Invalid Child Kind",
            description: "{owner_label} may not contain {referenced_kind} '{referenced_id}'. Allowed child kinds here: {allowed_kinds}.",
            fix: "Remove `{referenced_kind}: {referenced_id}` from this `contains:` block, or move it under a parent that accepts {referenced_kind} references.",
        },
        MessageEntry {
            id: "looks_like_list_missing_items",
            title: "List-Like Block Is Registered As The Wrong Kind",
            description: "'{id}' is referenced as a list, but is registered as a {registered_as}. Its YAML uses list-only fields: {found_fingerprints}.",
            fix: "Add `items:` to '{id}', or move the block under top-level `lists:`.",
        },
        MessageEntry {
            id: "looks_like_collection_missing_key",
            title: "Collection-Like Block Is Registered As The Wrong Kind",
            description: "'{id}' is referenced as a collection, but is registered as a {registered_as}. Its YAML uses collection-only fields: {found_fingerprints}.",
            fix: "Restore `contains:` on '{id}', or move the block under top-level `collections:`.",
        },
        MessageEntry {
            id: "looks_like_section_or_group_missing_key",
            title: "Section Or Group-Like Block Is Registered As The Wrong Kind",
            description: "'{id}' is referenced as a {inferred_kind}, but is registered as a {registered_as}. Its YAML uses {inferred_kind}-only fields: {found_fingerprints}.",
            fix: "Restore `contains:` on '{id}', or move the block under top-level `{inferred_kind}s:`.",
        },
        MessageEntry {
            id: "runtime_unknown_group",
            title: "Unknown Group",
            description: "{owner_label} references group '{referenced_id}', but no group with that id was found.",
            fix: "Add group '{referenced_id}', or update line {source_line} to reference an existing group.",
        },
        MessageEntry {
            id: "runtime_unknown_section",
            title: "Unknown Section",
            description: "{owner_label} references section '{referenced_id}', but no section with that id was found.",
            fix: "Add section '{referenced_id}', or update line {source_line} to reference an existing section.",
        },
        MessageEntry {
            id: "runtime_unknown_collection",
            title: "Unknown Collection",
            description: "{owner_label} references collection '{referenced_id}', but no collection with that id was found.",
            fix: "Add collection '{referenced_id}', or update line {source_line} to reference an existing collection.",
        },
        MessageEntry {
            id: "runtime_unknown_field",
            title: "Unknown Field",
            description: "{owner_label} references field '{referenced_id}', but no field with that id was found.",
            fix: "Add field '{referenced_id}', or update line {source_line} to reference an existing field.",
        },
        MessageEntry {
            id: "runtime_unknown_list",
            title: "Unknown List",
            description: "{owner_label} references list '{referenced_id}', but no list with that id was found.",
            fix: "Add list '{referenced_id}', or update line {source_line} to reference an existing list.",
        },
        MessageEntry {
            id: "runtime_group_child_invalid",
            title: "Invalid Group Child",
            description: "{owner_label} cannot contain {referenced_kind} '{referenced_id}' at runtime. Allowed child kinds here: {allowed_kinds}.",
            fix: "Move '{referenced_id}' under a compatible parent, or change this `contains:` entry to a section or collection.",
        },
        MessageEntry {
            id: "runtime_section_child_invalid",
            title: "Invalid Section Child",
            description: "{owner_label} cannot contain {referenced_kind} '{referenced_id}' at runtime. Allowed child kinds here: {allowed_kinds}.",
            fix: "Move '{referenced_id}' under a compatible parent, or change this `contains:` entry to a field or list.",
        },
        MessageEntry {
            id: "runtime_collection_child_invalid",
            title: "Invalid Collection Child",
            description: "{owner_label} cannot contain {referenced_kind} '{referenced_id}' at runtime. Allowed child kinds here: {allowed_kinds}.",
            fix: "Move '{referenced_id}' under a compatible parent, or change this `contains:` entry to a list.",
        },
        MessageEntry {
            id: "runtime_field_child_invalid",
            title: "Invalid Field Child",
            description: "{owner_label} cannot contain {referenced_kind} '{referenced_id}' at runtime. Allowed child kinds here: {allowed_kinds}.",
            fix: "Move '{referenced_id}' under a compatible parent, or change this `contains:` entry to a field, list, or collection.",
        },
        MessageEntry {
            id: "duplicate_id",
            title: "Duplicate ID",
            description: "{message}",
            fix: "Rename one of the conflicting hierarchy ids so every group, section, collection, field, and list id is globally unique.",
        },
        MessageEntry {
            id: "duplicate_boilerplate_id",
            title: "Duplicate Boilerplate ID",
            description: "{message}",
            fix: "Rename one boilerplate entry so each boilerplate id is unique.",
        },
        MessageEntry {
            id: "runtime_field_cycle",
            title: "Field Cycle",
            description: "{message}",
            fix: "Break the cycle by removing or changing one nested field reference.",
        },
        MessageEntry {
            id: "field_expected_list_wrong_kind",
            title: "Field List Reference Has Wrong Kind",
            description: "{message}",
            fix: "Point the field at an existing list id.",
        },
        MessageEntry {
            id: "field_unknown_list",
            title: "Field References Unknown List",
            description: "{message}",
            fix: "Add the list or update the field's `lists:` reference.",
        },
        MessageEntry {
            id: "field_explicit_format_list_wrong_kind",
            title: "Format List Has Wrong Kind",
            description: "{message}",
            fix: "Point `format_lists:` at an existing list id.",
        },
        MessageEntry {
            id: "field_unknown_explicit_format_list",
            title: "Unknown Explicit Format List",
            description: "{message}",
            fix: "Add the list or update `format_lists:`.",
        },
        MessageEntry {
            id: "field_expected_collection_wrong_kind",
            title: "Field Collection Reference Has Wrong Kind",
            description: "{message}",
            fix: "Point the field at an existing collection id.",
        },
        MessageEntry {
            id: "field_unknown_collection",
            title: "Field References Unknown Collection",
            description: "{message}",
            fix: "Add the collection or update the field's `collections:` reference.",
        },
        MessageEntry {
            id: "field_expected_format_list_wrong_kind",
            title: "Format Placeholder Has Wrong Kind",
            description: "{message}",
            fix: "Use a list id for this format placeholder.",
        },
        MessageEntry {
            id: "field_unknown_format_list",
            title: "Unknown Format Placeholder",
            description: "{message}",
            fix: "Attach the referenced list to the field or correct the placeholder.",
        },
        MessageEntry {
            id: "runtime_unknown_format_list",
            title: "Unknown Runtime Format List",
            description: "{message}",
            fix: "Attach the referenced format list to the field or correct the placeholder.",
        },
        MessageEntry {
            id: "runtime_unknown_branch_field",
            title: "Unknown Branch Field",
            description: "{message}",
            fix: "Add the branch field or update the list item's `fields:` / `branch_fields:` reference.",
        },
        MessageEntry {
            id: "empty_hotkey",
            title: "Empty Hotkey",
            description: "{message}",
            fix: "Use a single visible character or remove `hotkey`.",
        },
        MessageEntry {
            id: "invalid_hotkey",
            title: "Invalid Hotkey",
            description: "{message}",
            fix: "Use exactly one character in `hotkey`.",
        },
        MessageEntry {
            id: "read_data_dir_failed",
            title: "Could Not Read Data Folder",
            description: "{message}",
            fix: "Check that the data folder exists and that Scribblenot has permission to read it.",
        },
        MessageEntry {
            id: "enumerate_data_dir_failed",
            title: "Could Not List Data Files",
            description: "{message}",
            fix: "Check the data folder permissions, then try loading again.",
        },
        MessageEntry {
            id: "read_hierarchy_file_failed",
            title: "Could Not Read Data File",
            description: "{message}",
            fix: "Check that the file exists, is not locked by another program, and can be read.",
        },
        MessageEntry {
            id: "data_load_failed",
            title: "Data Load Failed",
            description: "{message}",
            fix: "Fix the reported problem, then reload data or restart Scribblenot.",
        },
        MessageEntry {
            id: "assign_self_reference",
            title: "List Assigns To Itself",
            description: "{message}",
            fix: "Remove the self-assignment or target a different list.",
        },
        MessageEntry {
            id: "assign_unknown_list",
            title: "Assigns Unknown List",
            description: "{message}",
            fix: "Point `assigns` at an existing list id.",
        },
        MessageEntry {
            id: "assign_unknown_item",
            title: "Assigns Unknown Item",
            description: "{message}",
            fix: "Use an existing item id in the target list.",
        },
        MessageEntry {
            id: "runtime_assign_unknown_list",
            title: "Runtime Assigns Unknown List",
            description: "{message}",
            fix: "Point `assigns` at an existing list id.",
        },
        MessageEntry {
            id: "runtime_assign_unknown_item",
            title: "Runtime Assigns Unknown Item",
            description: "{message}",
            fix: "Use an existing item id in the target list.",
        },
        MessageEntry {
            id: "runtime_build_failed",
            title: "Runtime Build Failed",
            description: "{message}",
            fix: "Fix the reported hierarchy issue, then reload data.",
        },
        MessageEntry {
            id: "multiple_templates_across_files",
            title: "Multiple Templates",
            description: "{message}",
            fix: "Keep exactly one `template:` block across all hierarchy data files.",
        },
        MessageEntry {
            id: "template_count_invalid",
            title: "Template Count Problem",
            description: "{message}",
            fix: "Add one `template:` block or remove extras so exactly one template remains.",
        },
        MessageEntry {
            id: "missing_template",
            title: "Missing Template",
            description: "{message}",
            fix: "Add a `template:` block that contains the top-level group references.",
        },
        MessageEntry {
            id: "template_runtime_child_invalid",
            title: "Invalid Template Child",
            description: "{message}",
            fix: "Template children must be group references.",
        },
        MessageEntry {
            id: "yaml_parse_failed",
            title: "YAML Parse Error",
            description: "{message}",
            fix: "Fix the YAML syntax or unsupported key shown above, then reload data.",
        },
        MessageEntry {
            id: "legacy_repeating_key",
            title: "Deprecated YAML Key",
            description: "{message}",
            fix: "Replace `repeating:` with `joiner_style:` in the reported file.",
        },
        MessageEntry {
            id: "multiple_templates_in_file",
            title: "Multiple Templates In One File",
            description: "{message}",
            fix: "Keep only one `template:` block in each YAML file.",
        },
        MessageEntry {
            id: "keybindings_invalid",
            title: "Invalid Keybindings",
            description: "{message}",
            fix: "Restore valid keybinding lists, for example `confirm: [enter]`.",
        },
    ]
}

fn report_params(report: &ErrorReport) -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("message".to_string(), report.message.clone());
    params.insert(
        "message_without_fix".to_string(),
        message_without_trailing_fix(&report.message).to_string(),
    );
    params.insert("kind_id".to_string(), report.kind_id().to_string());
    if let Some(source) = &report.source {
        params.insert("file".to_string(), source.file.display().to_string());
        params.insert("line".to_string(), source.line.to_string());
        params.insert("source_file".to_string(), source.file.display().to_string());
        params.insert("source_line".to_string(), source.line.to_string());
        params.insert(
            "quoted_line".to_string(),
            source.quoted_line.clone().unwrap_or_default(),
        );
        params.insert(
            "source_quoted_line".to_string(),
            source.quoted_line.clone().unwrap_or_default(),
        );
    }
    for (key, value) in report.params() {
        params.insert(key.to_string(), value);
    }
    params
}

fn message_without_trailing_fix(message: &str) -> &str {
    message
        .split_once(". Fix:")
        .map(|(summary, _)| summary)
        .unwrap_or(message)
}

fn render_source(report: &ErrorReport) -> Option<RenderedErrorSource> {
    report.source.as_ref().map(|source| RenderedErrorSource {
        location: format!("{}:{}", source.file.display(), source.line),
        quoted_line: source.quoted_line.clone(),
    })
}

fn render_source_blocks(report: &ErrorReport) -> Vec<RenderedErrorSourceBlock> {
    let params = report_params(report);
    let mut blocks = Vec::new();

    if let Some(source) = &report.source {
        push_source_block_line(
            &mut blocks,
            source.file.display().to_string(),
            source.line,
            source.quoted_line.clone().unwrap_or_default(),
            RenderedErrorSourceRole::Owner,
        );
    }

    push_source_block_line_from_params(
        &mut blocks,
        &params,
        "referenced_file",
        "referenced_line",
        "referenced_quoted_line",
        RenderedErrorSourceRole::Reference,
    );
    push_source_block_line_from_params(
        &mut blocks,
        &params,
        "found_file",
        "found_line",
        "found_quoted_line",
        RenderedErrorSourceRole::Found,
    );

    for block in &mut blocks {
        block.lines.sort_by_key(|line| line.line);
        block.lines.dedup_by(|left, right| {
            left.line == right.line
                && left.quoted_line == right.quoted_line
                && left.role == right.role
        });
    }

    blocks
}

fn push_source_block_line_from_params(
    blocks: &mut Vec<RenderedErrorSourceBlock>,
    params: &HashMap<String, String>,
    file_key: &str,
    line_key: &str,
    quote_key: &str,
    role: RenderedErrorSourceRole,
) {
    let Some(file_path) = params.get(file_key).cloned() else {
        return;
    };
    if file_path.trim().is_empty() {
        return;
    }
    let Some(line) = params
        .get(line_key)
        .and_then(|line| line.parse::<usize>().ok())
    else {
        return;
    };
    let quoted_line = params.get(quote_key).cloned().unwrap_or_default();
    push_source_block_line(blocks, file_path, line, quoted_line, role);
}

fn push_source_block_line(
    blocks: &mut Vec<RenderedErrorSourceBlock>,
    file_path: String,
    line: usize,
    quoted_line: String,
    role: RenderedErrorSourceRole,
) {
    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&file_path)
        .to_string();
    let line_entry = RenderedErrorSourceLine {
        line,
        quoted_line,
        role,
    };

    if let Some(block) = blocks.iter_mut().find(|block| block.file_path == file_path) {
        block.lines.push(line_entry);
        return;
    }

    blocks.push(RenderedErrorSourceBlock {
        file_name,
        file_path,
        lines: vec![line_entry],
    });
}

fn substitute(template: &str, params: &HashMap<String, String>) -> String {
    flatten_segments(&substitute_segments(template, params))
}

fn substitute_segments(
    template: &str,
    params: &HashMap<String, String>,
) -> Vec<RenderedTextSegment> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    while let Some(start) = template[cursor..].find('{') {
        let start = cursor + start;
        if start > cursor {
            segments.push(RenderedTextSegment {
                text: template[cursor..start].to_string(),
                is_param: false,
            });
        }
        let Some(end_rel) = template[start + 1..].find('}') else {
            segments.push(RenderedTextSegment {
                text: template[start..].to_string(),
                is_param: false,
            });
            return segments;
        };
        let end = start + 1 + end_rel;
        let key = &template[start + 1..end];
        let value = params
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("{{{key}}}"));
        segments.push(RenderedTextSegment {
            text: value,
            is_param: true,
        });
        cursor = end + 1;
    }
    if cursor < template.len() {
        segments.push(RenderedTextSegment {
            text: template[cursor..].to_string(),
            is_param: false,
        });
    }
    segments
}

fn flatten_segments(segments: &[RenderedTextSegment]) -> String {
    segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect()
}

fn title_from_kind(kind_id: &str) -> String {
    kind_id
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::report::{ErrorReport, ErrorSource};
    use std::path::PathBuf;

    #[test]
    fn render_substitutes_raw_message_and_source() {
        let messages = Messages::load();
        let report =
            ErrorReport::generic("yaml_parse_failed", "bad yaml").with_source(Some(ErrorSource {
                file: PathBuf::from("data/demo.yml"),
                line: 3,
                quoted_line: Some("bad: [".to_string()),
            }));

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "YAML Parse Error");
        assert_eq!(rendered.id, "yaml_parse_failed");
        assert_eq!(rendered.description, "bad yaml");
        assert_eq!(
            rendered.description_segments,
            vec![RenderedTextSegment {
                text: "bad yaml".to_string(),
                is_param: true,
            }]
        );
        assert_eq!(
            rendered.fix,
            "Fix the YAML syntax or unsupported key shown above, then reload data."
        );
        assert_eq!(
            rendered
                .source
                .as_ref()
                .map(|source| source.location.as_str()),
            Some("data/demo.yml:3")
        );
        assert_eq!(rendered.source_blocks.len(), 1);
        assert_eq!(rendered.source_blocks[0].file_name, "demo.yml");
        assert_eq!(rendered.source_blocks[0].lines[0].line, 3);
    }

    #[test]
    fn render_can_strip_trailing_fix_from_description() {
        let messages = Messages::load();
        let report = ErrorReport::generic(
            "missing_child",
            "section 'demo' references missing field 'missing'. Fix: add it.",
        )
        .with_param("owner_label", "section 'demo'")
        .with_param("referenced_kind", "field")
        .with_param("referenced_id", "missing");

        let rendered = messages.render(&report);

        assert_eq!(
            rendered.description,
            "section 'demo' references field 'missing', but no field with that id was found."
        );
    }

    #[test]
    fn render_substitutes_structured_child_reference_params() {
        let messages = Messages::load();
        let report = ErrorReport::generic(
            "missing_child",
            "section 'demo' references missing field 'missing'. Fix: add it.",
        )
        .with_source(Some(ErrorSource {
            file: PathBuf::from("data/demo.yml"),
            line: 12,
            quoted_line: Some("- field: missing".to_string()),
        }))
        .with_param("owner_label", "section 'demo'")
        .with_param("referenced_kind", "field")
        .with_param("referenced_id", "missing");

        let rendered = messages.render(&report);

        assert!(rendered
            .description
            .contains("section 'demo' references field 'missing'"));
        assert!(rendered.fix.contains("existing field"));
        assert!(rendered.fix.contains("missing"));
        assert!(rendered
            .description_segments
            .iter()
            .any(|segment| { segment.is_param && segment.text == "missing" }));
    }

    #[test]
    fn render_groups_related_source_lines_by_file() {
        let messages = Messages::load();
        let report = ErrorReport::generic("wrong_kind_reference", "raw")
            .with_source(Some(ErrorSource {
                file: PathBuf::from("data/subjective.yml"),
                line: 2,
                quoted_line: Some("- id: subjective_section".to_string()),
            }))
            .with_param("owner_label", "section 'subjective_section'")
            .with_param("referenced_kind", "field")
            .with_param("referenced_id", "back_all_prone_collection")
            .with_param("actual_kind", "collection")
            .with_param("referenced_file", "data/subjective.yml")
            .with_param("referenced_line", "8")
            .with_param(
                "referenced_quoted_line",
                "- field: back_all_prone_collection".to_string(),
            )
            .with_param("found_file", "data/treatment.yml")
            .with_param("found_line", "73")
            .with_param(
                "found_quoted_line",
                "- id: back_all_prone_collection".to_string(),
            );

        let rendered = messages.render(&report);

        assert_eq!(rendered.description, "section 'subjective_section' references 'back_all_prone_collection' as a field, but that id is registered as a collection.");
        assert_eq!(rendered.source_blocks.len(), 2);
        assert_eq!(rendered.source_blocks[0].file_name, "subjective.yml");
        assert_eq!(rendered.source_blocks[0].lines.len(), 2);
        assert_eq!(rendered.source_blocks[1].file_name, "treatment.yml");
        assert_eq!(rendered.source_blocks[1].lines[0].line, 73);
    }

    #[test]
    fn render_special_cases_section_collection_invalid_child_kind() {
        let messages = Messages::load();
        let report = ErrorReport::generic("invalid_child_kind", "raw")
            .with_param("owner_label", "section 'subjective_section'")
            .with_param("owner_kind", "section")
            .with_param("referenced_kind", "collection")
            .with_param("referenced_id", "muscle_regions")
            .with_param("allowed_kinds", "field, list");

        let rendered = messages.render(&report);

        assert_eq!(
            rendered.title,
            "Sections Cannot Directly Contain Collections (Yet)"
        );
        assert!(rendered.description.contains("Sections currently accept"));
        assert!(rendered.description.contains("fields"));
        assert!(rendered.description.contains("lists"));
        assert!(rendered.fix.contains("section -> field -> collection"));
        assert!(rendered.fix.contains("on the roadmap"));
    }
}
