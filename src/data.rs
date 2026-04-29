use crate::diagnostics::{ErrorReport, ErrorSource};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Pure data shape types live in `crate::data_model` after the data.rs split
// (slice 1). Hint-label helpers live in `crate::data_hints` after slice 2.
// Source-index types and YAML anchor helpers live in `crate::data_source`
// after slice 3. Runtime conversion helpers now live in `crate::data_runtime`
// after slice 5. Validation/reporting helpers now live in
// `crate::data_validate` after slice 6. Re-exporting from these sibling
// modules keeps `crate::data::TypeName` style callers and the remaining data.rs
// facade code working unchanged. The globs also bring these items into local
// scope so cross-slice helper calls can still use bare names.
pub use crate::data_hints::*;
pub use crate::data_load::*;
pub use crate::data_model::*;
pub use crate::data_runtime::*;
pub use crate::data_source::*;
pub use crate::data_validate::*;

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

#[derive(Clone)]
pub(crate) struct ValidationOwner {
    pub(crate) label: String,
    pub(crate) kind: &'static str,
    pub(crate) id: Option<String>,
    pub(crate) source_id: Option<String>,
}

impl ValidationOwner {
    pub(crate) fn new(kind: &'static str, id: Option<&str>) -> Self {
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

    pub(crate) fn source(&self, index: &SourceIndex) -> Option<ErrorSource> {
        self.source_id
            .as_deref()
            .and_then(|id| index.source_for(id))
    }
}

pub(crate) fn report(
    kind_id: &'static str,
    message: impl Into<String>,
    source: Option<ErrorSource>,
) -> ErrorReport {
    ErrorReport::generic(kind_id, message).with_source(source)
}

pub(crate) fn format_placeholder_report(
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

pub(crate) fn double_brace_format_placeholder_report(
    field_id: &str,
    placeholder_id: &str,
    message: impl Into<String>,
    source: Option<ErrorSource>,
) -> ErrorReport {
    let actual_placeholder_token = format!("{{{{{placeholder_id}}}}}");
    format_placeholder_report(
        "field_double_brace_format_placeholder",
        field_id,
        placeholder_id,
        message,
        source,
    )
    .with_param("actual_placeholder_token", actual_placeholder_token)
}

pub(crate) fn assign_rule_report(
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

pub(crate) fn child_reference_report(
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

pub(crate) fn child_reference_with_actual_kind(
    mut report: ErrorReport,
    actual_kind: TypeTag,
) -> ErrorReport {
    let actual_kind = kind_label(actual_kind).to_string();
    report.extra_params.retain(|(key, _)| key != "actual_kind");
    report.with_param("actual_kind", actual_kind)
}

pub(crate) fn child_reference_with_allowed_kinds(
    mut report: ErrorReport,
    allowed_kinds: &[TypeTag],
) -> ErrorReport {
    report
        .extra_params
        .retain(|(key, _)| key != "allowed_kinds");
    report.with_param("allowed_kinds", expected_kind_labels(allowed_kinds))
}

pub(crate) fn child_reference_with_found_source(
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

pub(crate) fn child_reference_with_reference_source(
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

pub(crate) fn reference_source_for_child(
    owner: &ValidationOwner,
    child: &HierarchyChildRef,
    index: &SourceIndex,
) -> Option<ErrorSource> {
    owner
        .id
        .as_deref()
        .and_then(|owner_id| index.source_for_child_ref(owner_id, child))
}

pub(crate) fn looks_like_double_brace_placeholder(format: &str, placeholder_id: &str) -> bool {
    let Some(normalized_id) = placeholder_id.strip_prefix('{') else {
        return false;
    };
    format.contains(&format!("{{{{{normalized_id}}}}}"))
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

pub fn find_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::ErrorKind;
    use std::collections::HashSet;
    use std::path::Path;
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
    fn field_double_brace_format_placeholder_gets_bespoke_diagnostic() {
        let (file, index) = parse_with_index(concat!(
            "template:\n  contains:\n    - group: g\n",
            "groups:\n  - id: g\n    contains:\n      - section: s\n",
            "sections:\n  - id: s\n    contains:\n      - field: f\n",
            "fields:\n",
            "  - id: f\n",
            "    label: Demo\n",
            "    format: \"{{pt_tolerance}}\"\n",
            "    contains:\n",
            "      - list: pt_tolerance\n",
            "lists:\n",
            "  - id: pt_tolerance\n",
            "    items:\n",
            "      - output: ok\n",
        ));
        let err = validate_with_index(&file, &index).expect_err("double-brace placeholder must fail");

        assert_eq!(err.kind_id(), "field_double_brace_format_placeholder");
        assert!(err.contains("double-brace placeholder '{{pt_tolerance}}'"));
        assert!(err.contains("single braces like '{pt_tolerance}'"));
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
