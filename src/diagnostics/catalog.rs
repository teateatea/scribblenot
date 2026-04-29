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
    if report.kind_id() == "unsupported_authored_key"
        && params.get("owner_kind").map(String::as_str) == Some("field")
        && params.get("key_name").map(String::as_str) == Some("format_lists")
    {
        return Some(&FIELD_FORMAT_LISTS_UNSUPPORTED_ENTRY);
    }
    if report.kind_id() == "unsupported_authored_key"
        && params.get("owner_kind").map(String::as_str) == Some("section")
        && params.get("key_name").map(String::as_str) == Some("body")
    {
        return Some(&SECTION_BODY_UNSUPPORTED_ENTRY);
    }
    if report.kind_id() == "unsupported_authored_key"
        && params.get("owner_kind").map(String::as_str) == Some("item")
        && params.get("key_name").map(String::as_str) == Some("branch_fields")
    {
        return Some(&ITEM_BRANCH_FIELDS_UNSUPPORTED_ENTRY);
    }
    if report.kind_id() == "invalid_authored_value_type"
        && params.get("owner_kind").map(String::as_str) == Some("field")
        && params.get("key_name").map(String::as_str) == Some("label")
        && params.get("actual_type").map(String::as_str) == Some("map")
        && params.get("expected_type").map(String::as_str) == Some("a string")
        && params.contains_key("inline_map_token")
    {
        return Some(
            if params.get("inline_map_list_exists").map(String::as_str) == Some("true") {
                &FIELD_LABEL_INLINE_MAP_LIST_EXISTS_ENTRY
            } else {
                &FIELD_LABEL_INLINE_MAP_LIST_MISSING_ENTRY
            },
        );
    }
    if report.kind_id() == "invalid_authored_value_type"
        && params.get("actual_type").map(String::as_str) == Some("map")
        && params.get("expected_type").map(String::as_str) == Some("a string")
        && params.contains_key("inline_map_token")
    {
        return Some(&STRING_PROPERTY_INLINE_MAP_ENTRY);
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

static FIELD_FORMAT_LISTS_UNSUPPORTED_ENTRY: MessageEntry = MessageEntry {
    id: "unsupported_authored_key",
    title: "Unsupported Field Property: format_lists",
    description: "{owner_label} uses unsupported key `format_lists`.",
    fix: "Remove `format_lists:` from this field. Keep the placeholders in `format:` and make sure the matching list ids are available through `contains:`; Scribblenot resolves those extra placeholder lists automatically.",
};

static SECTION_BODY_UNSUPPORTED_ENTRY: MessageEntry = MessageEntry {
    id: "unsupported_authored_key",
    title: "Unsupported Section Property: body",
    description: "{owner_label} uses unsupported key `body`.",
    fix: "Remove `body:` from this section. Scribblenot infers section behaviour from `contains:`: empty = `free_text`, field children = `multi_field`, list children = `list_select`.",
};

static ITEM_BRANCH_FIELDS_UNSUPPORTED_ENTRY: MessageEntry = MessageEntry {
    id: "unsupported_authored_key",
    title: "Unsupported Item Property: branch_fields",
    description: "{owner_label} uses unsupported key `branch_fields`.",
    fix: "Replace `branch_fields:` with `fields:` on this item. Item-driven branching is authored through `fields:` now.",
};

static FIELD_LABEL_INLINE_MAP_LIST_EXISTS_ENTRY: MessageEntry = MessageEntry {
    id: "invalid_authored_value_type",
    title: "Field Label Needs Quotes",
    description: "{owner_label} expects plain text for label, but {inline_map_token} is YAML inline-map syntax, not a string.",
    fix: "a) If you meant the literal text, add quotes:\n```yaml\n{key_name}: {quoted_inline_map_token}\n```\n\nb) If you meant to use the resolved list value instead, keep the label as normal text and move the dynamic part into format:\n```yaml\nfields:\n  - id: {owner_id}\n    label: \"Field Label\"\n    format: \"{inline_map_token}\"\n    contains:\n      - list: {inline_map_identifier}\n```",
};

static FIELD_LABEL_INLINE_MAP_LIST_MISSING_ENTRY: MessageEntry = MessageEntry {
    id: "invalid_authored_value_type",
    title: "Field Label Needs Quotes",
    description: "{owner_label} expects plain text for label, but {inline_map_token} is YAML inline-map syntax, not a string.",
    fix: "a) If you meant the literal text, add quotes:\n```yaml\n{key_name}: {quoted_inline_map_token}\n```\n\nb) If you meant to use the resolved list value instead, keep the label as normal text and move the dynamic part into format:\n```yaml\nfields:\n  - id: {owner_id}\n    label: \"Field Label\"\n    format: \"{inline_map_token}\"\n    contains:\n      - list: {inline_map_identifier}\n```\n\nc) No list with id {inline_map_identifier} was found in this document yet, so create it first:\n```yaml\n{list_creation_example}\n```",
};

static STRING_PROPERTY_INLINE_MAP_ENTRY: MessageEntry = MessageEntry {
    id: "invalid_authored_value_type",
    title: "This Needs Quotes",
    description: "{owner_label} expects {expected_type} for {key_name}, but {inline_map_token} is YAML inline-map syntax, not plain text.",
    fix: "a) If you meant the literal text, add quotes:\n```yaml\n{key_name}: {quoted_inline_map_token}\n```",
};

fn builtin_entries() -> Vec<MessageEntry> {
    vec![
        MessageEntry {
            id: "missing_child",
            title: "ID Not Found: {referenced_id}",
            description: "{owner_label} references {referenced_kind} '{referenced_id}', but no {referenced_kind} with that id was found.",
            fix: "a) Update the ID: *'{referenced_id}'* -> **existing {referenced_kind}**:\n     ln {source_line} `{source_quoted_line}`\n     ...\n     **ln {referenced_line}** `  - {referenced_kind}: **correct_{referenced_kind}_id**`\n\nb) Create the '{referenced_id}' **{referenced_kind}** with a full scaffold, then trim anything you do not need:\n```yaml\n{creation_example}\n```\n",
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
            title: "This Block Looks Like A List",
            description: "`{id}` is being used as a list, but it is currently registered as a {registered_as}.\n\nScribblenot inferred that this block was probably meant to be a list because it uses list-only fields: {found_fingerprints}.",
            fix: "Two valid repairs:\n1. Keep this as a **list** by adding `items:` and moving the block under top-level `lists:`.\n2. Or leave the block where it is and change the reference so it points at a real list instead.",
        },
        MessageEntry {
            id: "looks_like_collection_missing_key",
            title: "This Block Looks Like A Collection",
            description: "`{id}` is being used as a collection, but it is currently registered as a {registered_as}.\n\nScribblenot inferred that this block was probably meant to be a collection because it uses collection-only fields: {found_fingerprints}.",
            fix: "Two valid repairs:\n1. Keep this as a **collection** by restoring `contains:` and moving the block under top-level `collections:`.\n2. Or change the reference so it points at a real collection instead.",
        },
        MessageEntry {
            id: "looks_like_section_or_group_missing_key",
            title: "This Block Looks Like A {inferred_kind}",
            description: "`{id}` is being used as a {inferred_kind}, but it is currently registered as a {registered_as}.\n\nScribblenot inferred that this block was probably meant to be a {inferred_kind} because it uses {inferred_kind}-only fields: {found_fingerprints}.",
            fix: "Two valid repairs:\n1. Keep this as a **{inferred_kind}** by restoring `contains:` and placing it under top-level `{inferred_kind}s:`.\n2. Or change the reference so it points at a real {inferred_kind} instead.",
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
            description: "These field references form a cycle:\n\n`{cycle_path}`\n\nThat means the wizard can never finish walking this field tree, because one of the nested fields eventually asks for the starting field again.",
            fix: "Break the cycle in one of two ways:\n1. Remove or change one `contains:` field reference in the cycle.\n2. Flatten one step into a list or format-driven value instead of another nested field.",
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
            description: "{owner_label} exposes `{placeholder_token}` through `format_lists:`, but that id resolves to a {actual_kind}, not a list.",
            fix: "In Scribblenot, placeholders in `format:` resolve through lists attached to the same field.\n\nUse a real list id here, or rename the placeholder so it matches the list you intended.",
        },
        MessageEntry {
            id: "field_unknown_explicit_format_list",
            title: "Unknown Explicit Format List",
            description: "{owner_label} exposes `{placeholder_token}` through `format_lists:`, but no list with id `{placeholder_id}` was found.",
            fix: "In Scribblenot, placeholders in `format:` resolve through lists attached to the same field.\n\nAdd a list with id `{placeholder_id}`, or update `format_lists:` so it points at the list id you meant.",
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
            description: "{owner_label} uses placeholder `{placeholder_token}` in `format:`, but that id currently resolves to a {actual_kind}, not a list.",
            fix: "In Scribblenot, `format:` placeholders resolve through lists attached to the same field.\n\nThis field needs a matching pair:\n```yaml\nformat: \"{placeholder_token}\"\ncontains:\n  - list: {placeholder_id}\n```\nIf `{placeholder_id}` is meant to stay a {actual_kind}, rename the placeholder instead.",
        },
        MessageEntry {
            id: "field_unknown_format_list",
            title: "Unknown Format Placeholder",
            description: "{owner_label} uses placeholder `{placeholder_token}` in `format:`, but this field does not expose a list with id `{placeholder_id}`.",
            fix: "In Scribblenot, `format:` placeholders resolve through lists attached to the same field.\n\nThis field needs a matching pair:\n```yaml\nformat: \"{placeholder_token}\"\ncontains:\n  - list: {placeholder_id}\n```\nIf the real list id is different, rename the placeholder to match it.",
        },
        MessageEntry {
            id: "field_double_brace_format_placeholder",
            title: "Format Placeholder Uses Double Braces",
            description: "{owner_label} uses `{actual_placeholder_token}` in `format:`, but Scribblenot placeholders use single braces such as `{placeholder_token}`.",
            fix: "Use single braces in `format:` and keep the matching list attached to the same field.\n\nThis field should look like:\n```yaml\nformat: \"{placeholder_token}\"\ncontains:\n  - list: {placeholder_id}\n```\n`{{...}}` is treated as a typo, not as a valid placeholder form.",
        },
        MessageEntry {
            id: "runtime_unknown_format_list",
            title: "Unknown Runtime Format List",
            description: "{owner_label} still needs placeholder `{placeholder_token}` at runtime, but no list with id `{placeholder_id}` is available on that field.",
            fix: "Runtime placeholder resolution follows the same rule as authored validation: `{placeholder_token}` must line up with a list attached to the same field.\n\nAdd that list to the field, or rename the placeholder so it matches the list id you intended.",
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
            description: "When `{source_item_id}` is chosen in `{source_list_id}`, this `assigns:` rule points back into the same list `{target_list_id}`.\n\n`assigns:` is meant to be a side effect that selects an item in another list, not the current one.",
            fix: "Remove this self-assignment, or point it at a different target list that should be auto-selected when `{source_item_id}` is chosen.",
        },
        MessageEntry {
            id: "assign_unknown_list",
            title: "Assigns Unknown List",
            description: "When `{source_item_id}` is chosen in `{source_list_id}`, this item tries to also select `{target_item_id}` in list `{target_list_id}`.\n\nThat target list does not exist.",
            fix: "In Scribblenot, `assigns:` means \"when this item is chosen, also choose that item in another list.\"\n\nFix the target list id, or remove the assignment if this side effect is no longer needed.",
        },
        MessageEntry {
            id: "assign_unknown_item",
            title: "Assigns Unknown Item",
            description: "When `{source_item_id}` is chosen in `{source_list_id}`, this item tries to also select `{target_item_id}` in list `{target_list_id}`.\n\nThat target item does not exist in the target list.",
            fix: "In Scribblenot, `assigns:` means \"when this item is chosen, also choose that item in another list.\"\n\nFix the target item id, or remove the assignment if this side effect is no longer needed.",
        },
        MessageEntry {
            id: "runtime_assign_unknown_list",
            title: "Runtime Assigns Unknown List",
            description: "At runtime, choosing `{source_item_id}` in `{source_list_id}` still tries to auto-select `{target_item_id}` in list `{target_list_id}`.\n\nThat target list does not exist.",
            fix: "Fix the target list id in `assigns:`, or remove the assignment if this side effect is no longer needed.",
        },
        MessageEntry {
            id: "runtime_assign_unknown_item",
            title: "Runtime Assigns Unknown Item",
            description: "At runtime, choosing `{source_item_id}` in `{source_list_id}` still tries to auto-select `{target_item_id}` in list `{target_list_id}`.\n\nThat target item does not exist in the target list.",
            fix: "Fix the target item id in `assigns:`, or remove the assignment if this side effect is no longer needed.",
        },
        MessageEntry {
            id: "runtime_build_failed",
            title: "Runtime Build Failed",
            description: "{message}",
            fix: "Fix the reported hierarchy issue, then reload data.",
        },
        MessageEntry {
            id: "multiple_templates_across_files",
            title: "Multiple Template Roots",
            description: "Scribblenot found more than one `template:` block across the loaded hierarchy files.\n\nThe template is the single root of the hierarchy and defines the top-level navigation order.",
            fix: "Keep exactly one `template:` block across all hierarchy files.\n\nPick one root template and merge the other template's `contains:` entries into it.\n\nA minimal valid root looks like:\n```yaml\ntemplate:\n  contains:\n    - group: intake\n```",
        },
        MessageEntry {
            id: "template_count_invalid",
            title: "Template Count Problem",
            description: "Scribblenot requires exactly one `template:` block across the whole hierarchy, but found {found_count}.",
            fix: "Add one template if none exist, or remove extras until exactly one remains.\n\nA minimal valid root looks like:\n```yaml\ntemplate:\n  contains:\n    - group: intake\n```",
        },
        MessageEntry {
            id: "missing_template",
            title: "Missing Template",
            description: "The merged hierarchy has no `template:` block.\n\nScribblenot needs one template because it is the single root that defines top-level navigation order.",
            fix: "Add one `template:` block that contains the top-level group references.\n\nA minimal valid root looks like:\n```yaml\ntemplate:\n  contains:\n    - group: intake\n```\nThen put your sections and collections under groups referenced by that template.",
        },
        MessageEntry {
            id: "template_runtime_child_invalid",
            title: "Invalid Template Child",
            description: "The `template` block is the single root of the hierarchy.\n\nRight now, `template.contains` should list group references only, not {template_child_kind} `{template_child_id}` directly.",
            fix: "Put `{template_child_id}` inside a group, then reference that group from `template:`.\n\nA minimal valid shape looks like:\n```yaml\ntemplate:\n  contains:\n    - group: intake\n\ngroups:\n  - id: intake\n    contains:\n      - {template_child_kind}: {template_child_id}\n```",
        },
        MessageEntry {
            id: "yaml_parse_failed",
            title: "YAML Parse Error",
            description: "{message}",
            fix: "Fix the YAML syntax or unsupported key shown above, then reload data.",
        },
        MessageEntry {
            id: "yaml_unclosed_structure",
            title: "Unclosed YAML Value",
            description: "This YAML {structure_label} starts with `{opening_token}` but never closes with `{closing_token}`.",
            fix: "Add the missing closing `{closing_token}`, or rewrite the value so the opening `{opening_token}` is balanced.",
        },
        MessageEntry {
            id: "yaml_indented_top_level_key",
            title: "Indented Top-Level Key",
            description: "`{key}:` is a top-level block but has leading indentation, so YAML reads it as part of the block above.",
            fix: "Remove the leading spaces or tabs before `{key}:` so it starts at column 1 (no indentation).",
        },
        MessageEntry {
            id: "joiner_style_unknown_variant",
            title: "Unknown joiner_style Value",
            description: "`{provided}` is not a valid `joiner_style`. Valid values:\n- `comma_and` - joined with commas; last item preceded by \"and\"\n- `comma_and_the` - like comma_and but \"and the\" before last item\n- `comma_or` - joined with commas; last item preceded by \"or\"\n- `comma` - joined with commas only\n- `semicolon` - joined with semicolons\n- `slash` - joined with slashes\n- `newline` - each item on its own line",
            fix: "Replace `{provided}` with one of the valid values above.",
        },
        MessageEntry {
            id: "legacy_repeating_key",
            title: "Deprecated YAML Key",
            description: "{message}",
            fix: "Replace `repeating:` with `joiner_style:` in the reported file.",
        },
        MessageEntry {
            id: "unsupported_authored_key",
            title: "Unsupported Property: {key_name}",
            description: "{owner_label} uses unsupported key `{key_name}`.",
            fix: "Allowed keys here: {expected_keys}. Remove `{key_name}`, or rename it to the supported property you intended.",
        },
        MessageEntry {
            id: "missing_required_authored_key",
            title: "Missing Required Property: {key_name}",
            description: "{owner_label} is missing required key `{key_name}`.",
            fix: "Add `{key_name}` to {owner_label}. Required keys here: {required_keys}.",
        },
        MessageEntry {
            id: "invalid_authored_value_type",
            title: "Wrong Value Type: {key_name}",
            description: "{owner_label} expects {expected_type} for `{key_name}`, but this YAML value is {actual_type}.",
            fix: "Change `{key_name}` to the expected value shape here. Expected {expected_type}; found {actual_type}.",
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
    if report.kind_id() == "missing_child" {
        add_missing_child_example_params(&mut params);
    }
    add_derived_params(&mut params);
    params
}

fn add_derived_params(params: &mut HashMap<String, String>) {
    if let Some(placeholder_id) = params.get("placeholder_id").cloned() {
        params
            .entry("placeholder_token".to_string())
            .or_insert_with(|| format!("{{{placeholder_id}}}"));
    }
    if let Some(inline_map_token) = params
        .get("inline_map_token")
        .cloned()
        .or_else(|| {
            params
                .get("source_quoted_line")
                .and_then(|source| extract_inline_map_token(source))
        })
    {
        params
            .entry("inline_map_token".to_string())
            .or_insert_with(|| inline_map_token.clone());
        params.insert(
            "quoted_inline_map_token".to_string(),
            format!("\"{inline_map_token}\""),
        );
        if let Some(identifier) = params
            .get("inline_map_identifier")
            .cloned()
            .or_else(|| extract_inline_map_identifier(&inline_map_token))
        {
            params
                .entry("inline_map_identifier".to_string())
                .or_insert_with(|| identifier.clone());
            params.insert(
                "list_creation_example".to_string(),
                list_creation_example(&identifier),
            );
        }
    }
}

fn extract_inline_map_token(source_quoted_line: &str) -> Option<String> {
    let (_, value) = source_quoted_line.split_once(':')?;
    let value = value.trim();
    if value.starts_with('{') && value.ends_with('}') && !value.contains(':') {
        Some(value.to_string())
    } else {
        None
    }
}

fn extract_inline_map_identifier(inline_map_token: &str) -> Option<String> {
    let identifier = inline_map_token
        .strip_prefix('{')?
        .strip_suffix('}')?
        .trim();
    (!identifier.is_empty()).then(|| identifier.to_string())
}

fn list_creation_example(id: &str) -> String {
    format!(
        concat!(
            "lists:\n",
            "  - id: {id}\n",
            "    label: \"List Label\"\n",
            "    items:\n",
            "      - id: some_value\n",
            "        label: \"Some Value\"\n",
            "        output: \"Some Value\""
        ),
        id = id
    )
}

fn add_missing_child_example_params(params: &mut HashMap<String, String>) {
    let Some(referenced_kind) = params.get("referenced_kind").cloned() else {
        return;
    };
    let Some(referenced_id) = params.get("referenced_id").cloned() else {
        return;
    };

    params.insert(
        "creation_example".to_string(),
        missing_child_creation_example(&referenced_kind, &referenced_id),
    );
}

fn missing_child_creation_example(kind: &str, id: &str) -> String {
    match kind {
        "group" => format!(
            concat!(
                "groups:\n",
                "  - id: {id}\n",
                "    nav_label: \"GROUP\"\n",
                "    note_label: \"## Group Heading\"\n",
                "    contains:\n",
                "      - boilerplate: some_boilerplate_id\n",
                "      - section: some_section_id"
            ),
            id = id
        ),
        "section" => format!(
            concat!(
                "sections:\n",
                "  - id: {id}\n",
                "    label: \"Section Label\"\n",
                "    nav_label: \"SECTION\"\n",
                "    hotkey: z\n",
                "    show_field_labels: true\n",
                "    contains:\n",
                "      - field: some_field_id\n",
                "    note:\n",
                "      note_label: \"#### Section Heading\""
            ),
            id = id
        ),
        "collection" => format!(
            concat!(
                "collections:\n",
                "  - id: {id}\n",
                "    label: \"Collection Label\"\n",
                "    nav_label: \"COLLECT\"\n",
                "    note_label: \"#### Collection Heading\"\n",
                "    default_enabled: false\n",
                "    joiner_style: comma_and_the\n",
                "    contains:\n",
                "      - list: some_list_id\n",
                "    note:\n",
                "      note_label: \"#### Collection Heading\""
            ),
            id = id
        ),
        "field" => format!(
            concat!(
                "fields:\n",
                "  - id: {id}\n",
                "    label: \"Field Label\"\n",
                "    nav_label: \"FIELD\"\n",
                "    hotkey: f\n",
                "    format: \"{{some_child_id}}\"\n",
                "    preview: \"[Preview]\"\n",
                "    contains:\n",
                "      - list: some_list_id\n",
                "    joiner_style: comma_and_the\n",
                "    max_entries: 3\n",
                "    max_actives: 1"
            ),
            id = id
        ),
        "list" => format!(
            concat!(
                "lists:\n",
                "  - id: {id}\n",
                "    label: \"List Label\"\n",
                "    preview: \"[Preview]\"\n",
                "    sticky: false\n",
                "    default: default_item\n",
                "    modal_start: search\n",
                "    joiner_style: comma_and_the\n",
                "    max_entries: 3\n",
                "    items:\n",
                "      - id: default_item\n",
                "        label: \"Default Item\"\n",
                "        default_enabled: true\n",
                "        output: \"Default output\"\n",
                "        fields:\n",
                "          - some_field_id\n",
                "        assigns:\n",
                "          - list: some_other_list_id\n",
                "            item: some_other_item_id"
            ),
            id = id
        ),
        _ => format!("{kind}s:\n  - id: {id}", kind = kind, id = id),
    }
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
    fn render_missing_child_fix_includes_full_field_scaffold() {
        let messages = Messages::load();
        let report = ErrorReport::generic("missing_child", "raw")
            .with_param("owner_label", "section 'demo'")
            .with_param("referenced_kind", "field")
            .with_param("referenced_id", "consent_pects_field");

        let rendered = messages.render(&report);

        assert!(rendered.fix.contains("fields:"));
        assert!(rendered.fix.contains("- id: consent_pects_field"));
        assert!(rendered.fix.contains("label: \"Field Label\""));
        assert!(rendered.fix.contains("format: \"{some_child_id}\""));
        assert!(rendered.fix.contains("preview: \"[Preview]\""));
        assert!(rendered.fix.contains("joiner_style: comma_and_the"));
        assert!(rendered.fix.contains("max_entries: 3"));
        assert!(rendered.fix.contains("max_actives: 1"));
    }

    #[test]
    fn render_missing_child_fix_includes_list_item_properties() {
        let messages = Messages::load();
        let report = ErrorReport::generic("missing_child", "raw")
            .with_param("owner_label", "field 'demo'")
            .with_param("referenced_kind", "list")
            .with_param("referenced_id", "consent_regions");

        let rendered = messages.render(&report);

        assert!(rendered.fix.contains("lists:"));
        assert!(rendered.fix.contains("- id: consent_regions"));
        assert!(rendered.fix.contains("items:"));
        assert!(rendered.fix.contains("default_enabled: true"));
        assert!(rendered.fix.contains("fields:"));
        assert!(rendered.fix.contains("assigns:"));
        assert!(rendered.fix.contains("modal_start: search"));
    }

    #[test]
    fn render_unsupported_field_format_lists_as_teaching_moment() {
        let messages = Messages::load();
        let report = ErrorReport::generic("unsupported_authored_key", "raw")
            .with_param("owner_kind", "field")
            .with_param("owner_label", "field 'consent_pecs_field'")
            .with_param("key_name", "format_lists")
            .with_param("expected_keys", "`id`, `label`, `format`, `contains`");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Unsupported Field Property: format_lists");
        assert!(rendered
            .description
            .contains("field 'consent_pecs_field' uses unsupported key `format_lists`"));
        assert!(rendered.fix.contains("Remove `format_lists:` from this field"));
        assert!(rendered
            .fix
            .contains("resolves those extra placeholder lists automatically"));
    }

    #[test]
    fn render_unsupported_section_body_as_teaching_moment() {
        let messages = Messages::load();
        let report = ErrorReport::generic("unsupported_authored_key", "raw")
            .with_param("owner_kind", "section")
            .with_param("owner_label", "section 'subjective'")
            .with_param("key_name", "body")
            .with_param(
                "expected_keys",
                "`id`, `label`, `nav_label`, `hotkey`, `show_field_labels`, `contains`, `note`",
            );

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Unsupported Section Property: body");
        assert!(rendered.fix.contains("Remove `body:` from this section"));
        assert!(rendered
            .fix
            .contains("Scribblenot infers section behaviour from `contains:`"));
    }

    #[test]
    fn render_unsupported_item_branch_fields_as_teaching_moment() {
        let messages = Messages::load();
        let report = ErrorReport::generic("unsupported_authored_key", "raw")
            .with_param("owner_kind", "item")
            .with_param("owner_label", "item 'alpha' in list 'demo'")
            .with_param("key_name", "branch_fields")
            .with_param(
                "expected_keys",
                "`id`, `label`, `default_enabled`, `output`, `hotkey`, `fields`, `assigns`",
            );

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Unsupported Item Property: branch_fields");
        assert!(rendered.fix.contains("Replace `branch_fields:` with `fields:`"));
    }

    #[test]
    fn render_missing_required_authored_key() {
        let messages = Messages::load();
        let report = ErrorReport::generic("missing_required_authored_key", "raw")
            .with_param("owner_kind", "field")
            .with_param("owner_label", "field 'consent_glutes_field'")
            .with_param("key_name", "label")
            .with_param("required_keys", "`id`, `label`");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Missing Required Property: label");
        assert!(rendered
            .description
            .contains("field 'consent_glutes_field' is missing required key `label`"));
        assert!(rendered.fix.contains("Required keys here: `id`, `label`"));
    }

    #[test]
    fn render_field_label_inline_map_as_quote_hint_when_list_exists() {
        let messages = Messages::load();
        let report = ErrorReport::generic("invalid_authored_value_type", "raw")
            .with_source(Some(ErrorSource {
                file: PathBuf::from("data/intake_optional.yml"),
                line: 13,
                quoted_line: Some("label: {pt_pronouns}".to_string()),
            }))
            .with_param("owner_kind", "field")
            .with_param("owner_label", "field 'pt_info'")
            .with_param("owner_id", "pt_info")
            .with_param("key_name", "label")
            .with_param("actual_type", "map")
            .with_param("expected_type", "a string")
            .with_param("inline_map_token", "{pt_pronouns}")
            .with_param("inline_map_identifier", "pt_pronouns")
            .with_param("inline_map_list_exists", "true");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Field Label Needs Quotes");
        assert!(rendered.description.contains("{pt_pronouns} is YAML inline-map syntax"));
        assert!(rendered.fix.contains("a) If you meant the literal text"));
        assert!(rendered.fix.contains("label: \"{pt_pronouns}\""));
        assert!(rendered.fix.contains("b) If you meant to use the resolved list value instead"));
        assert!(rendered.fix.contains("format: \"{pt_pronouns}\""));
        assert!(rendered.fix.contains("- list: pt_pronouns"));
        assert!(!rendered.fix.contains("c) No list with id"));
    }

    #[test]
    fn render_field_label_inline_map_as_quote_hint_when_list_is_missing() {
        let messages = Messages::load();
        let report = ErrorReport::generic("invalid_authored_value_type", "raw")
            .with_source(Some(ErrorSource {
                file: PathBuf::from("data/demo.yml"),
                line: 7,
                quoted_line: Some("label: {missing_list}".to_string()),
            }))
            .with_param("owner_kind", "field")
            .with_param("owner_label", "field 'demo_field'")
            .with_param("owner_id", "demo_field")
            .with_param("key_name", "label")
            .with_param("actual_type", "map")
            .with_param("expected_type", "a string")
            .with_param("inline_map_token", "{missing_list}")
            .with_param("inline_map_identifier", "missing_list")
            .with_param("inline_map_list_exists", "false");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Field Label Needs Quotes");
        assert!(rendered.fix.contains("c) No list with id missing_list was found"));
        assert!(rendered.fix.contains("- id: missing_list"));
    }

    #[test]
    fn render_unclosed_yaml_structure() {
        let messages = Messages::load();
        let report = ErrorReport::generic("yaml_unclosed_structure", "raw")
            .with_param("structure_label", "quoted value")
            .with_param("opening_token", "\"")
            .with_param("closing_token", "\"");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Unclosed YAML Value");
        assert!(rendered
            .description
            .contains("starts with `\"` but never closes with `\"`"));
        assert!(rendered.fix.contains("Add the missing closing `\"`"));
    }

    #[test]
    fn render_looks_like_list_missing_items_teaches_inference() {
        let messages = Messages::load();
        let report = ErrorReport {
            kind: crate::diagnostics::report::ErrorKind::LooksLikeListMissingItems {
                id: "appointment_type".to_string(),
                registered_as: "field".to_string(),
                found_fingerprints: vec!["modal_start".to_string(), "sticky".to_string()],
            },
            message: "raw".to_string(),
            source: None,
            extra_params: Vec::new(),
        };

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "This Block Looks Like A List");
        assert!(rendered.description.contains("Scribblenot inferred"));
        assert!(rendered.description.contains("modal_start, sticky"));
        assert!(rendered.fix.contains("adding `items:`"));
        assert!(rendered.fix.contains("top-level `lists:`"));
    }

    #[test]
    fn render_missing_template_teaches_single_root() {
        let messages = Messages::load();
        let report = ErrorReport::generic("missing_template", "raw");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Missing Template");
        assert!(rendered.description.contains("single root"));
        assert!(rendered.fix.contains("template:"));
        assert!(rendered.fix.contains("- group: intake"));
    }

    #[test]
    fn render_template_runtime_child_invalid_shows_group_wrapper() {
        let messages = Messages::load();
        let report = ErrorReport::generic("template_runtime_child_invalid", "raw")
            .with_param("template_child_kind", "section")
            .with_param("template_child_id", "subjective_section");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Invalid Template Child");
        assert!(rendered.description.contains("group references only"));
        assert!(rendered.fix.contains("Put `subjective_section` inside a group"));
        assert!(rendered.fix.contains("- section: subjective_section"));
    }

    #[test]
    fn render_field_unknown_format_list_teaches_placeholder_contract() {
        let messages = Messages::load();
        let report = ErrorReport::generic("field_unknown_format_list", "raw")
            .with_param("owner_label", "field 'duration_field'")
            .with_param("owner_id", "duration_field")
            .with_param("placeholder_id", "year");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Unknown Format Placeholder");
        assert!(rendered.description.contains("placeholder `{year}`"));
        assert!(rendered.fix.contains("format: \"{year}\""));
        assert!(rendered.fix.contains("- list: year"));
    }

    #[test]
    fn render_field_double_brace_placeholder_teaches_single_brace_contract() {
        let messages = Messages::load();
        let report = ErrorReport::generic("field_double_brace_format_placeholder", "raw")
            .with_param("owner_label", "field 'pt_response_field'")
            .with_param("owner_id", "pt_response_field")
            .with_param("placeholder_id", "pt_tolerance")
            .with_param("actual_placeholder_token", "{{pt_tolerance}}");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Format Placeholder Uses Double Braces");
        assert!(rendered.description.contains("`{{pt_tolerance}}`"));
        assert!(rendered.description.contains("`{pt_tolerance}`"));
        assert!(rendered.fix.contains("format: \"{pt_tolerance}\""));
        assert!(rendered.fix.contains("`{{...}}` is treated as a typo"));
    }

    #[test]
    fn render_runtime_field_cycle_shows_cycle_path() {
        let messages = Messages::load();
        let report = ErrorReport::generic("runtime_field_cycle", "raw")
            .with_param("cycle_path", "pain_summary -> pain_region -> pain_detail -> pain_summary");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Field Cycle");
        assert!(rendered.description.contains("pain_summary -> pain_region"));
        assert!(rendered.fix.contains("Remove or change one `contains:` field reference"));
        assert!(rendered.fix.contains("format-driven value"));
    }

    #[test]
    fn render_assign_unknown_item_teaches_side_effect_flow() {
        let messages = Messages::load();
        let report = ErrorReport::generic("assign_unknown_item", "raw")
            .with_param("source_list_id", "side_list")
            .with_param("source_item_id", "left")
            .with_param("target_list_id", "laterality_words")
            .with_param("target_item_id", "lt");

        let rendered = messages.render(&report);

        assert_eq!(rendered.title, "Assigns Unknown Item");
        assert!(rendered.description.contains("When `left` is chosen in `side_list`"));
        assert!(rendered.description.contains("`lt` in list `laterality_words`"));
        assert!(rendered.fix.contains("when this item is chosen, also choose that item in another list"));
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
