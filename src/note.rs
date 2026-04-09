use crate::app::SectionState;
use crate::data::{SectionConfig, SectionGroup};
use crate::sections::multi_field::resolve_multifield_value;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone)]
pub enum NoteRenderMode {
    Preview,
    Export,
}

#[allow(dead_code)]
pub fn section_start_line(
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    groups: &[SectionGroup],
    boilerplate_texts: &HashMap<String, String>,
    section_id: &str,
) -> u16 {
    let note = render_note(groups, sections, states, sticky_values, boilerplate_texts, NoteRenderMode::Preview);
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
    groups: &[SectionGroup],
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String {
    render_document(groups, sections, states, sticky_values, boilerplate_texts, mode, false)
}

pub fn render_editable_document(
    groups: &[SectionGroup],
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
) -> String {
    render_document(
        groups,
        sections,
        states,
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
    sticky_values: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String {
    render_section_body(cfg, state, sticky_values, mode)
}

fn render_document(
    groups: &[SectionGroup],
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
    mode: NoteRenderMode,
    editable: bool,
) -> String {
    let mut parts = Vec::new();

    for (group_idx, group) in groups.iter().enumerate() {
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

        for (cfg, state) in group_sections(group, sections, states) {
            let body = render_section_body(cfg, state, sticky_values, mode.clone());
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

fn group_sections<'a>(
    group: &'a SectionGroup,
    sections: &'a [SectionConfig],
    states: &'a [SectionState],
) -> Vec<(&'a SectionConfig, &'a SectionState)> {
    group.sections
        .iter()
        .filter_map(|group_section| {
            sections
                .iter()
                .zip(states.iter())
                .find(|(section, _)| section.id == group_section.id)
        })
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
    sticky_values: &HashMap<String, String>,
    mode: NoteRenderMode,
) -> String {
    match (cfg.section_type.as_str(), state) {
        ("multi_field", SectionState::Header(header)) => render_multifield(cfg, header, sticky_values),
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
            let resolved = resolve_multifield_value(value, field_cfg, sticky_values);
            if let Some(rendered) = resolved.export_value() {
                if multifield_renders_without_field_label(cfg, field_cfg) {
                    lines.push(rendered.to_string());
                } else {
                    let label =
                        crate::sections::multi_field::resolve_field_label(value, field_cfg, sticky_values);
                    lines.push(format!("{}: {}", label, rendered));
                }
            }
        }

        if values.is_empty() {
            let resolved = resolve_multifield_value(
                &crate::sections::header::HeaderFieldValue::Text(String::new()),
                field_cfg,
                sticky_values,
            );
            if let Some(rendered) = resolved.export_value() {
                if multifield_renders_without_field_label(cfg, field_cfg) {
                    lines.push(rendered.to_string());
                } else {
                    let label =
                        crate::sections::multi_field::resolve_field_label(
                            &crate::sections::header::HeaderFieldValue::Text(String::new()),
                            field_cfg,
                            sticky_values,
                        );
                    lines.push(format!("{}: {}", label, rendered));
                }
            }
        }
    }
    lines.join("\n")
}

fn multifield_renders_without_field_label(
    cfg: &SectionConfig,
    field_cfg: &crate::data::HeaderFieldConfig,
) -> bool {
    cfg.id == "appointment_section"
        || (!field_cfg.collections.is_empty() && field_cfg.lists.is_empty() && field_cfg.format.is_none())
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
    use crate::data::{AppData, HeaderFieldConfig, ResolvedCollectionConfig, RuntimeNodeKind, SectionConfig};
    use crate::sections::collection::CollectionState;
    use crate::sections::free_text::FreeTextState;
    use crate::sections::header::HeaderState;
    use crate::sections::list_select::ListSelectState;
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
                    data.list_data
                        .get(&section.id)
                        .cloned()
                        .unwrap_or_default(),
                )),
                "checklist" => SectionState::Checklist(
                    crate::sections::checklist::ChecklistState::new(
                        data.checklist_data
                            .get(&section.id)
                            .cloned()
                            .unwrap_or_default(),
                    ),
                ),
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

    #[test]
    fn real_data_render_uses_group_authored_heading_order() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let states = states_for_real_data(&data);

        let note = render_note(
            &data.groups,
            &data.sections,
            &states,
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
            group_heading_positions.windows(2).all(|pair| pair[0] < pair[1]),
            "group headings should render in authored group order"
        );
    }

    #[test]
    fn real_data_render_places_group_headings_before_authored_section_headings() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let states = states_for_real_data(&data);

        let note = render_editable_document(
            &data.groups,
            &data.sections,
            &states,
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
    fn real_data_render_uses_live_field_outputs() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let mut states = states_for_real_data(&data);
        let mut expected_outputs = Vec::new();

        for (index, section) in data.sections.iter().enumerate() {
            if section.section_type != "header" {
                continue;
            }
            let SectionState::Header(header_state) = &mut states[index] else {
                panic!("header section should create header state");
            };
            for field_index in 0..header_state.repeated_values.len() {
                let value = format!("TEST-VALUE-{}-{}", section.id, field_index);
                header_state.repeated_values[field_index] =
                    vec![crate::sections::header::HeaderFieldValue::Text(value.clone())];
                expected_outputs.push(value);
            }
        }

        let note = render_note(
            &data.groups,
            &data.sections,
            &states,
            &HashMap::new(),
            &data.boilerplate_texts,
            NoteRenderMode::Preview,
        );

        for value in expected_outputs {
            assert!(note.contains(&value), "rendered note should include seeded field output");
        }
    }

    #[test]
    fn collection_only_multifield_without_format_renders_without_field_label() {
        let cfg = SectionConfig {
            id: "prone_treatment".to_string(),
            name: "Treatment - Prone".to_string(),
            map_label: "PRONE".to_string(),
            section_type: "multi_field".to_string(),
            data_file: None,
            fields: Some(vec![HeaderFieldConfig {
                id: "back".to_string(),
                name: "BACK".to_string(),
                format: None,
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
                        }],
                    }],
                }],
                format_lists: Vec::new(),
                max_entries: None,
                max_actives: None,
            }]),
            lists: Vec::new(),
            note_label: None,
            group_id: "treatment".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let state = HeaderState::new(cfg.fields.clone().unwrap_or_default());

        let rendered = render_multifield(&cfg, &state, &HashMap::new());

        assert!(rendered.starts_with("#### ALL - UPPER MIDDLE & LOW BACK"));
        assert!(!rendered.starts_with("BACK:"));
    }
}
