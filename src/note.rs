use crate::app::SectionState;
use crate::data::{RuntimeTemplate, SectionConfig};
use crate::sections::header::HeaderFieldValue;
use crate::sections::multi_field::{render_note_line, render_note_line_for_confirmed_slot};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone)]
pub enum NoteRenderMode {
    Preview,
    Export,
}

#[allow(dead_code)]
pub fn section_start_line(
    template: &RuntimeTemplate,
    sections: &[SectionConfig],
    states: &[SectionState],
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
    section_id: &str,
) -> u16 {
    let note = render_note(
        template,
        sections,
        states,
        assigned_values,
        sticky_values,
        boilerplate_texts,
        NoteRenderMode::Preview,
    );
    let anchor = sections
        .iter()
        .find(|section| section.id == section_id)
        .and_then(managed_heading_for_section)
        .unwrap_or_default();

    note.lines()
        .enumerate()
        .find_map(|(idx, line)| line.contains(&anchor).then_some(idx as u16))
        .unwrap_or(0)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn render_note(
    template: &RuntimeTemplate,
    sections: &[SectionConfig],
    states: &[SectionState],
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String {
    render_document(
        template,
        sections,
        states,
        assigned_values,
        sticky_values,
        boilerplate_texts,
        mode,
        false,
    )
}

pub fn render_editable_document(
    template: &RuntimeTemplate,
    sections: &[SectionConfig],
    states: &[SectionState],
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
) -> String {
    render_document(
        template,
        sections,
        states,
        assigned_values,
        sticky_values,
        boilerplate_texts,
        NoteRenderMode::Preview,
        true,
    )
}

pub fn managed_heading_for_section(cfg: &SectionConfig) -> Option<String> {
    cfg.note_label
        .clone()
        .or_else(|| Some(format!("#### {}", cfg.name.to_uppercase())))
}

pub fn render_editable_section_body(
    cfg: &SectionConfig,
    state: &SectionState,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String {
    render_section_body(cfg, state, assigned_values, sticky_values, mode)
}

fn render_document(
    template: &RuntimeTemplate,
    sections: &[SectionConfig],
    states: &[SectionState],
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
    mode: NoteRenderMode,
    editable: bool,
) -> String {
    let mut parts = Vec::new();
    let state_lookup = section_state_lookup(sections, states);

    for (group_idx, group) in template.children.iter().enumerate() {
        if group_idx > 0 {
            parts.push("\n\n\n_______________".to_string());
        }

        if let Some(heading) = group.note.note_label.as_deref() {
            parts.push(format!("\n\n{}", heading));
        }

        for boilerplate_id in &group.note.boilerplate_refs {
            if let Some(text) = boilerplate_texts.get(boilerplate_id) {
                if !text.trim().is_empty() {
                    parts.push(format!("\n{}", text));
                }
            }
        }

        for node in &group.children {
            let config = node.config();
            let Some((cfg, state)) = state_lookup.get(config.id.as_str()).copied() else {
                continue;
            };
            let body = render_section_body(cfg, state, assigned_values, sticky_values, mode.clone());
            if body.trim().is_empty() && is_skipped(state) {
                continue;
            }

            if editable {
                append_managed_section(&mut parts, cfg, body);
            } else if let Some(heading) = managed_heading_for_section(cfg) {
                parts.push(format!("\n\n{}", heading));
                if !body.trim().is_empty() {
                    parts.push(format!("\n{}", body));
                }
            }
        }
    }

    if !editable {
        parts.push("\n".to_string());
    }

    parts.join("")
}

fn section_state_lookup<'a>(
    sections: &'a [SectionConfig],
    states: &'a [SectionState],
) -> HashMap<&'a str, (&'a SectionConfig, &'a SectionState)> {
    sections
        .iter()
        .zip(states.iter())
        .map(|(section, state)| (section.id.as_str(), (section, state)))
        .collect()
}

fn append_managed_section(parts: &mut Vec<String>, cfg: &SectionConfig, body: String) {
    if let Some(heading) = managed_heading_for_section(cfg) {
        parts.push(format!("\n\n{}", heading));
    }
    parts.push(format!(
        "\n<!-- scribblenot:section id={}:start -->\n{}\n<!-- scribblenot:section id={}:end -->",
        cfg.id, body, cfg.id
    ));
}

fn render_section_body(
    cfg: &SectionConfig,
    state: &SectionState,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String {
    match (cfg.section_type.as_str(), state) {
        ("multi_field", SectionState::Header(header)) => {
            render_multifield(cfg, header, assigned_values, sticky_values)
        }
        ("free_text", SectionState::FreeText(text)) => text.entries.join("\n"),
        ("list_select", SectionState::ListSelect(list)) => {
            let items: Vec<String> = list
                .selected_indices
                .iter()
                .filter_map(|idx| list.entries.get(*idx))
                .map(|entry| entry.output.clone())
                .collect();
            items.join("\n")
        }
        ("checklist", SectionState::Checklist(checklist)) => checklist
            .items
            .iter()
            .zip(checklist.checked.iter())
            .filter_map(|(item, checked)| checked.then_some(item.clone()))
            .collect::<Vec<_>>()
            .join("\n"),
        ("collection", SectionState::Collection(collection)) => render_collection(collection),
        _ => match mode {
            NoteRenderMode::Preview | NoteRenderMode::Export => String::new(),
        },
    }
}

fn render_multifield(
    cfg: &SectionConfig,
    state: &crate::sections::header::HeaderState,
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
) -> String {
    let mut lines = Vec::new();
    for (field_idx, field_cfg) in state.field_configs.iter().enumerate() {
        let values = state
            .repeated_values
            .get(field_idx)
            .map(|values| values.as_slice())
            .unwrap_or(&[]);

        for value in values {
            if let Some(rendered) = render_note_line_for_confirmed_slot(
                cfg,
                field_cfg,
                value,
                assigned_values,
                sticky_values,
            ) {
                lines.push(rendered);
            }
        }

        if values.is_empty() {
            let empty_value = HeaderFieldValue::Text(String::new());
            if let Some(rendered) =
                render_note_line(cfg, field_cfg, &empty_value, assigned_values, sticky_values)
            {
                lines.push(rendered);
            }
        }
    }
    lines.join("\n")
}

fn render_collection(state: &crate::sections::collection::CollectionState) -> String {
    crate::modal::format_collection_field_value(&state.collections, false)
}

fn is_skipped(state: &SectionState) -> bool {
    match state {
        SectionState::Pending => false,
        SectionState::Header(state) => state.completed && state.field_configs.is_empty(),
        SectionState::FreeText(state) => state.skipped,
        SectionState::ListSelect(state) => state.skipped,
        SectionState::Collection(state) => state.skipped,
        SectionState::Checklist(state) => state.skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{
        AppData, GroupNoteMeta, HeaderFieldConfig, HierarchyItem, HierarchyList, ItemAssignment,
        ModalStart, ResolvedCollectionConfig, RuntimeGroup, RuntimeNode, RuntimeNodeKind,
        RuntimeTemplate, SectionConfig,
    };
    use crate::sections::collection::CollectionState;
    use crate::sections::free_text::FreeTextState;
    use crate::sections::header::{HeaderFieldValue, HeaderState, ListFieldValue};
    use crate::sections::list_select::ListSelectState;
    use std::fs;
    use std::path::PathBuf;

    fn states_for_real_data(data: &AppData) -> Vec<SectionState> {
        data.sections
            .iter()
            .map(|section| match section.section_type.as_str() {
                "multi_field" => SectionState::Header(HeaderState::new(
                    section.fields.clone().unwrap_or_default(),
                )),
                "free_text" => SectionState::FreeText(FreeTextState::new()),
                "list_select" => SectionState::ListSelect(ListSelectState::new(
                    data.list_data.get(&section.id).cloned().unwrap_or_default(),
                )),
                "checklist" => {
                    SectionState::Checklist(crate::sections::checklist::ChecklistState::new(
                        data.checklist_data
                            .get(&section.id)
                            .cloned()
                            .unwrap_or_default(),
                    ))
                }
                "collection" => SectionState::Collection(CollectionState::new(
                    data.collection_data
                        .get(&section.id)
                        .cloned()
                        .unwrap_or_default(),
                )),
                _ => SectionState::Pending,
            })
            .collect()
    }

    fn set_header_field_text(
        states: &mut [SectionState],
        sections: &[SectionConfig],
        section_id: &str,
        field_index: usize,
        text: &str,
    ) {
        let Some(section_index) = sections.iter().position(|section| section.id == section_id)
        else {
            panic!("missing section '{section_id}'");
        };
        let SectionState::Header(state) = &mut states[section_index] else {
            panic!("section '{section_id}' should create header state");
        };
        state.repeated_values[field_index] = vec![crate::sections::header::HeaderFieldValue::Text(
            text.to_string(),
        )];
    }

    fn set_header_field_explicit_empty(
        states: &mut [SectionState],
        sections: &[SectionConfig],
        section_id: &str,
        field_index: usize,
    ) {
        let Some(section_index) = sections.iter().position(|section| section.id == section_id)
        else {
            panic!("missing section '{section_id}'");
        };
        let SectionState::Header(state) = &mut states[section_index] else {
            panic!("section '{section_id}' should create header state");
        };
        state.repeated_values[field_index] =
            vec![crate::sections::header::HeaderFieldValue::ExplicitEmpty];
    }

    fn item(id: &str, label: &str, output: &str) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: Some(output.to_string()),
            fields: None,
            branch_fields: Vec::new(),
            assigns: Vec::new(),
        }
    }

    fn item_with_assignment(
        id: &str,
        label: &str,
        output: &str,
        list_id: &str,
        item_id: &str,
        assigned_output: &str,
    ) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: Some(output.to_string()),
            fields: None,
            branch_fields: Vec::new(),
            assigns: vec![ItemAssignment {
                list_id: list_id.to_string(),
                item_id: item_id.to_string(),
                output: assigned_output.to_string(),
            }],
        }
    }

    fn assigned_time_field(id: &str, name: &str, max_entries: Option<usize>) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: id.to_string(),
            name: name.to_string(),
            format: Some("{start_hour}:{start_minute}{am_pm}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "start_hour".to_string(),
                    label: Some("Start Hour".to_string()),
                    preview: Some("hh".to_string()),
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        item_with_assignment("hour_9", "9", "9", "am_pm", "am_item", "AM"),
                        item_with_assignment("hour_12", "12", "12", "am_pm", "pm_item", "PM"),
                    ],
                },
                HierarchyList {
                    id: "start_minute".to_string(),
                    label: Some("Start Minute".to_string()),
                    preview: Some("mm".to_string()),
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("minute_00", "00", "00"), item("minute_45", "45", "45")],
                },
            ],
            collections: Vec::new(),
            format_lists: vec![HierarchyList {
                id: "am_pm".to_string(),
                label: Some("AM/PM".to_string()),
                preview: Some("XM".to_string()),
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![item("am_item", "AM", "AM"), item("pm_item", "PM", "PM")],
            }],
            joiner_style: None,
            max_entries,
            max_actives: None,
        }
    }

    fn list_state(
        values: &[&str],
        item_ids: &[&str],
        list_idx: usize,
    ) -> HeaderFieldValue {
        HeaderFieldValue::ListState(ListFieldValue {
            values: values.iter().map(|value| (*value).to_string()).collect(),
            item_ids: item_ids.iter().map(|id| (*id).to_string()).collect(),
            list_idx,
            repeat_values: Vec::new(),
            repeat_item_ids: Vec::new(),
        })
    }

    #[test]
    fn real_data_render_uses_group_authored_heading_order() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let states = states_for_real_data(&data);

        let note = render_note(
            &data.template,
            &data.sections,
            &states,
            &HashMap::new(),
            &HashMap::new(),
            &data.boilerplate_texts,
            NoteRenderMode::Preview,
        );

        let expected_headings: Vec<&str> = data
            .groups
            .iter()
            .filter_map(|group| group.note.note_label.as_deref())
            .collect();
        let group_heading_positions: Vec<usize> = expected_headings
            .iter()
            .filter_map(|heading| note.find(heading))
            .collect();

        assert_eq!(
            group_heading_positions.len(),
            expected_headings.len(),
            "every authored runtime group heading should appear in the rendered note"
        );
        assert!(
            group_heading_positions
                .windows(2)
                .all(|pair| pair[0] < pair[1]),
            "group headings should render in authored group order"
        );
    }

    #[test]
    fn render_note_uses_runtime_template_order_not_sections_slice_order() {
        let first = SectionConfig {
            id: "first".to_string(),
            name: "First".to_string(),
            map_label: "FIRST".to_string(),
            section_type: "free_text".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: None,
            lists: Vec::new(),
            note_label: Some("#### FIRST".to_string()),
            group_id: "group_a".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let second = SectionConfig {
            id: "second".to_string(),
            name: "Second".to_string(),
            map_label: "SECOND".to_string(),
            section_type: "free_text".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: None,
            lists: Vec::new(),
            note_label: Some("#### SECOND".to_string()),
            group_id: "group_a".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let template = RuntimeTemplate {
            id: "test".to_string(),
            children: vec![RuntimeGroup {
                id: "group_a".to_string(),
                nav_label: "GROUP A".to_string(),
                note: GroupNoteMeta {
                    note_label: Some("## GROUP A".to_string()),
                    boilerplate_refs: Vec::new(),
                },
                children: vec![
                    RuntimeNode::Section(first.clone()),
                    RuntimeNode::Section(second.clone()),
                ],
            }],
        };
        let sections = vec![second.clone(), first.clone()];
        let states = vec![
            SectionState::FreeText(FreeTextState {
                entries: vec!["second body".to_string()],
                edit_buf: String::new(),
                mode: crate::sections::free_text::FreeTextMode::Browsing,
                cursor: 0,
                completed: true,
                skipped: false,
            }),
            SectionState::FreeText(FreeTextState {
                entries: vec!["first body".to_string()],
                edit_buf: String::new(),
                mode: crate::sections::free_text::FreeTextMode::Browsing,
                cursor: 0,
                completed: true,
                skipped: false,
            }),
        ];

        let note = render_note(
            &template,
            &sections,
            &states,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            NoteRenderMode::Preview,
        );

        let first_heading = note.find("#### FIRST").expect("first heading should render");
        let second_heading = note.find("#### SECOND").expect("second heading should render");
        assert!(first_heading < second_heading);
        assert!(note.contains("first body"));
        assert!(note.contains("second body"));
    }

    #[test]
    fn real_data_render_places_group_headings_before_authored_section_headings() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let states = states_for_real_data(&data);

        let note = render_editable_document(
            &data.template,
            &data.sections,
            &states,
            &HashMap::new(),
            &HashMap::new(),
            &data.boilerplate_texts,
        );

        for section in &data.sections {
            let Some(section_heading) = managed_heading_for_section(section) else {
                continue;
            };
            let heading_pos = note
                .find(&section_heading)
                .expect("managed section heading should be present");
            let marker_pos = note
                .find(&crate::document::marker_start(&section.id))
                .expect("managed section start marker should be present");
            assert!(
                heading_pos < marker_pos,
                "managed section heading should render before its start marker"
            );
        }
    }

    #[test]
    fn real_data_render_skips_group_heading_when_note_label_is_omitted() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let states = states_for_real_data(&data);

        let note = render_note(
            &data.template,
            &data.sections,
            &states,
            &HashMap::new(),
            &HashMap::new(),
            &data.boilerplate_texts,
            NoteRenderMode::Preview,
        );

        assert!(
            !note.lines().any(|line| line.trim() == "INTAKE"),
            "groups without note_label should not render a top-level note heading"
        );
    }

    #[test]
    fn real_data_render_uses_live_field_outputs() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let mut states = states_for_real_data(&data);
        let mut expected_outputs = Vec::new();

        for (index, section) in data.sections.iter().enumerate() {
            if section.section_type != "multi_field" {
                continue;
            }
            let SectionState::Header(header_state) = &mut states[index] else {
                panic!("multi_field section should create header state");
            };
            for field_index in 0..header_state.repeated_values.len() {
                let value = format!("TEST-VALUE-{}-{}", section.id, field_index);
                header_state.repeated_values[field_index] =
                    vec![crate::sections::header::HeaderFieldValue::Text(
                        value.clone(),
                    )];
                expected_outputs.push(value);
            }
        }

        assert!(
            !expected_outputs.is_empty(),
            "real data should seed at least one multi_field output assertion"
        );

        let note = render_note(
            &data.template,
            &data.sections,
            &states,
            &HashMap::new(),
            &HashMap::new(),
            &data.boilerplate_texts,
            NoteRenderMode::Preview,
        );

        for value in expected_outputs {
            assert!(
                note.contains(&value),
                "rendered note should include seeded field output"
            );
        }
    }

    #[test]
    fn representative_note_matches_golden_file() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let mut states = states_for_real_data(&data);

        set_header_field_explicit_empty(&mut states, &data.sections, "appointment_section", 0);
        set_header_field_text(
            &mut states,
            &data.sections,
            "appointment_section",
            1,
            "# Apr 12, 2026 at 1:30PM (60 min)",
        );
        set_header_field_text(
            &mut states,
            &data.sections,
            "appointment_section",
            2,
            "2026-04-12: Pt requested a Treatment massage, focusing on the Head, Neck, and Shoulders, the Low Back, and the Left Knee.",
        );
        set_header_field_text(
            &mut states,
            &data.sections,
            "subjective_section",
            0,
            "2026-04-12: BL Head, Neck, and Shoulders: Pt describes ongoing minor discomfort, tightness (without pain)",
        );
        set_header_field_text(
            &mut states,
            &data.sections,
            "treatment_section",
            0,
            "#### ALL - UPPER MIDDLE & LOW BACK\n- General Swedish Techniques\n- Specific Compressions:\n- - Trapezius (Upper Fiber)\n- - Levator Scapula\n- - Teres Major & Minor\n- - Quadratus Lumborum\n- Stretch (Serratus Anterior)\n- Broad Compressions (Triceps Brachii)",
        );
        set_header_field_explicit_empty(&mut states, &data.sections, "treatment_section", 1);
        set_header_field_text(
            &mut states,
            &data.sections,
            "treatment_section",
            2,
            "#### POSTERIOR LEGS & FEET (Prone)\n- Broad Compressions\n- Ulnar Kneading\n- - Biceps Femoris\n- - Semitendinosus\n- Knuckle Kneading\n- Fingertip Kneading",
        );
        set_header_field_text(
            &mut states,
            &data.sections,
            "objective_section",
            0,
            "2026-04-12: BL Trapezius (Upper Fibers): Increased Resting Muscle Tension",
        );

        let rendered = render_note(
            &data.template,
            &data.sections,
            &states,
            &HashMap::new(),
            &HashMap::new(),
            &data.boilerplate_texts,
            NoteRenderMode::Preview,
        );
        let expected =
            fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("golden_note.md"))
                .expect("golden note fixture should load");

        assert_eq!(
            normalize_newlines(&rendered).trim(),
            normalize_newlines(&expected).trim()
        );
        assert!(rendered.contains("#### ALL - UPPER MIDDLE & LOW BACK"));
        assert!(rendered.contains("- Broad Compressions (Triceps Brachii)"));
        assert!(!rendered.contains("- Muscle Stripping (Erector Spinae)"));
        assert!(!rendered.lines().any(|line| line.trim() == "INTAKE"));
    }

    fn normalize_newlines(text: &str) -> String {
        text.replace("\r\n", "\n")
    }

    #[test]
    fn collection_only_multifield_without_format_renders_without_field_label() {
        let cfg = SectionConfig {
            id: "prone_treatment".to_string(),
            name: "Treatment - Prone".to_string(),
            map_label: "PRONE".to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: Some(vec![HeaderFieldConfig {
                id: "back".to_string(),
                name: "BACK".to_string(),
                format: None,
                preview: None,
                fields: Vec::new(),
                lists: Vec::new(),
                collections: vec![ResolvedCollectionConfig {
                    id: "all_back".to_string(),
                    label: "ALL - UPPER MIDDLE & LOW BACK".to_string(),
                    note_label: Some("#### ALL - UPPER MIDDLE & LOW BACK".to_string()),
                    default_enabled: true,
                    joiner_style: None,
                    lists: vec![crate::data::HierarchyList {
                        id: "back_all_prone".to_string(),
                        label: Some("UPPER, MIDDLE & LOWER BACK (Prone)".to_string()),
                        preview: None,
                        sticky: false,
                        default: None,
                        modal_start: crate::data::ModalStart::List,
                        joiner_style: None,
                        max_entries: None,
                        items: vec![crate::data::HierarchyItem {
                            id: "swedish".to_string(),
                            label: Some("General Swedish Techniques".to_string()),
                            default_enabled: true,
                            output: Some("- General Swedish Techniques".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        }],
                    }],
                }],
                format_lists: Vec::new(),
                joiner_style: None,
                max_entries: None,
                max_actives: None,
            }]),
            lists: Vec::new(),
            note_label: None,
            group_id: "treatment".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let state = HeaderState::new(cfg.fields.clone().unwrap_or_default());

        let rendered = render_multifield(&cfg, &state, &HashMap::new(), &HashMap::new());

        assert!(rendered.starts_with("#### ALL - UPPER MIDDLE & LOW BACK"));
        assert!(!rendered.starts_with("BACK:"));
    }

    #[test]
    fn multifield_section_can_hide_labels_without_section_id_special_case() {
        let cfg = SectionConfig {
            id: "custom_header".to_string(),
            name: "Custom Header".to_string(),
            map_label: "CUSTOM".to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: false,
            data_file: None,
            fields: Some(vec![HeaderFieldConfig {
                id: "summary".to_string(),
                name: "Summary".to_string(),
                format: None,
                preview: None,
                fields: Vec::new(),
                lists: Vec::new(),
                collections: Vec::new(),
                format_lists: Vec::new(),
                joiner_style: None,
                max_entries: None,
                max_actives: None,
            }]),
            lists: Vec::new(),
            note_label: None,
            group_id: "intake".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let mut state = HeaderState::new(cfg.fields.clone().unwrap_or_default());
        state.repeated_values[0] = vec![crate::sections::header::HeaderFieldValue::Text(
            "Standalone summary".to_string(),
        )];

        let rendered = render_multifield(&cfg, &state, &HashMap::new(), &HashMap::new());

        assert_eq!(rendered, "Standalone summary");
    }

    #[test]
    fn repeating_slots_keep_their_own_assigned_format_list_outputs() {
        let cfg = SectionConfig {
            id: "appointments".to_string(),
            name: "Appointments".to_string(),
            map_label: "APPTS".to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: Some(vec![assigned_time_field("appointment", "Appointment", Some(2))]),
            lists: Vec::new(),
            note_label: None,
            group_id: "intake".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let mut state = HeaderState::new(cfg.fields.clone().unwrap_or_default());
        state.repeated_values[0] = vec![
            list_state(&["9", "00"], &["hour_9", "minute_00"], 2),
            list_state(&["12", "45"], &["hour_12", "minute_45"], 2),
        ];
        let assigned_values = HashMap::from([("am_pm".to_string(), "PM".to_string())]);

        let rendered = render_multifield(&cfg, &state, &assigned_values, &HashMap::new());

        assert!(rendered.contains("Appointment: 9:00AM"));
        assert!(rendered.contains("Appointment: 12:45PM"));
        assert!(!rendered.contains("Appointment: 9:00PM"));
    }

    #[test]
    fn different_fields_do_not_overwrite_each_others_assigned_format_lists() {
        let cfg = SectionConfig {
            id: "schedule".to_string(),
            name: "Schedule".to_string(),
            map_label: "SCHEDULE".to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: Some(vec![
                assigned_time_field("first_visit", "First Visit", None),
                assigned_time_field("follow_up", "Follow-up", None),
            ]),
            lists: Vec::new(),
            note_label: None,
            group_id: "intake".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let mut state = HeaderState::new(cfg.fields.clone().unwrap_or_default());
        state.repeated_values[0] = vec![list_state(&["9", "00"], &["hour_9", "minute_00"], 2)];
        state.repeated_values[1] =
            vec![list_state(&["12", "45"], &["hour_12", "minute_45"], 2)];
        let assigned_values = HashMap::from([("am_pm".to_string(), "PM".to_string())]);

        let rendered = render_multifield(&cfg, &state, &assigned_values, &HashMap::new());

        assert!(rendered.contains("First Visit: 9:00AM"));
        assert!(rendered.contains("Follow-up: 12:45PM"));
        assert!(!rendered.contains("First Visit: 9:00PM"));
    }
}
