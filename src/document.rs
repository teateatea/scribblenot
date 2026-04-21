//! Editable document helpers and anchor contract.
//!
//! # Anchor contract
//!
//! Each runtime-editable section is represented in the document by:
//!
//! 1. An optional visible heading (for example `#### SUBJECTIVE`). When present,
//!    it must appear before the section's start marker.
//! 2. A start marker: `<!-- scribblenot:section id=<id>:start -->`
//! 3. A machine-managed body region between the markers.
//! 4. An end marker: `<!-- scribblenot:section id=<id>:end -->`
//!
//! Replacement rewrites only the body between the markers. Text outside the
//! markers remains untouched. If either marker is missing or out of order, the
//! document is invalid and targeted replacement must be blocked.
//!
//! A section with an empty `note_label` has no heading anchor; its markers are
//! the stable replacement boundary.

use crate::app::SectionState;
use crate::data::{RuntimeTemplate, SectionConfig};
use crate::note::{managed_heading_for_section, render_editable_document};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SectionAnchorSpec {
    pub section_id: String,
    pub heading_text: Option<String>,
    pub marker_start: String,
    pub marker_end: String,
}

pub fn build_initial_document(
    template: &RuntimeTemplate,
    sections: &[SectionConfig],
    states: &[SectionState],
    assigned_values: &HashMap<String, String>,
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
) -> String {
    render_editable_document(
        template,
        sections,
        states,
        assigned_values,
        sticky_values,
        boilerplate_texts,
    )
}

pub fn marker_start(section_id: &str) -> String {
    format!("<!-- scribblenot:section id={section_id}:start -->")
}

pub fn marker_end(section_id: &str) -> String {
    format!("<!-- scribblenot:section id={section_id}:end -->")
}

pub fn editable_section_specs(sections: &[SectionConfig]) -> Vec<SectionAnchorSpec> {
    sections
        .iter()
        .map(|cfg| {
            let heading = managed_heading_for_section(cfg).filter(|heading| !heading.is_empty());
            SectionAnchorSpec {
                section_id: cfg.id.clone(),
                heading_text: heading,
                marker_start: marker_start(&cfg.id),
                marker_end: marker_end(&cfg.id),
            }
        })
        .collect()
}

pub fn validate_section_anchors(document: &str, sections: &[SectionConfig]) -> Result<(), String> {
    for spec in editable_section_specs(sections) {
        let start_pos = document.find(&spec.marker_start).ok_or_else(|| {
            format!(
                "Missing managed section start marker for '{}'.",
                spec.section_id
            )
        })?;
        let end_pos = document.find(&spec.marker_end).ok_or_else(|| {
            format!(
                "Missing managed section end marker for '{}'.",
                spec.section_id
            )
        })?;
        if end_pos <= start_pos {
            return Err(format!(
                "Managed section markers for '{}' are out of order.",
                spec.section_id
            ));
        }
        if let Some(ref heading) = spec.heading_text {
            let heading_pos = document.find(heading).ok_or_else(|| {
                format!(
                    "Missing managed section heading '{}' for '{}'.",
                    heading, spec.section_id
                )
            })?;
            if heading_pos > start_pos {
                return Err(format!(
                    "Section heading for '{}' appears after its start marker.",
                    spec.section_id
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_document_structure(
    document: &str,
    sections: &[SectionConfig],
) -> Result<(), String> {
    validate_section_anchors(document, sections)
}

pub fn replace_managed_section_body(
    document: &str,
    section_id: &str,
    new_body: &str,
) -> Option<String> {
    let start_marker = marker_start(section_id);
    let end_marker = marker_end(section_id);
    let start_idx = document.find(&start_marker)?;
    let body_start = start_idx + start_marker.len() + 1;
    let end_idx = document[body_start..].find(&end_marker)? + body_start;

    let mut out = String::with_capacity(document.len() + new_body.len());
    out.push_str(&document[..body_start]);
    out.push_str(new_body);
    out.push('\n');
    out.push_str(&document[end_idx..]);
    Some(out)
}

pub fn export_editable_document(document: &str) -> String {
    let lines: Vec<&str> = document.lines().collect();
    let mut out = Vec::new();
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = lines[idx];
        if is_marker_start_line(line) || is_marker_end_line(line) {
            idx += 1;
            continue;
        }

        if line.starts_with("#### ")
            && lines
                .get(idx + 1)
                .is_some_and(|next| is_marker_start_line(next))
        {
            let heading = line.to_string();
            idx += 2;
            let mut body = Vec::new();
            while idx < lines.len() && !is_marker_end_line(lines[idx]) {
                if should_export_line(lines[idx]) {
                    body.push(lines[idx].to_string());
                }
                idx += 1;
            }
            if idx < lines.len() && is_marker_end_line(lines[idx]) {
                idx += 1;
            }
            if body.iter().any(|line| !line.trim().is_empty()) {
                out.push(heading);
                out.extend(body);
            }
            continue;
        }

        if should_export_line(line) {
            out.push(line.to_string());
        }
        idx += 1;
    }

    compact_blank_lines(&out).trim().to_string()
}

fn is_marker_start_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("<!-- scribblenot:section id=") && trimmed.ends_with(":start -->")
}

fn is_marker_end_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("<!-- scribblenot:section id=") && trimmed.ends_with(":end -->")
}

fn should_export_line(line: &str) -> bool {
    let trimmed = line.trim();
    !(trimmed == "--" || trimmed.ends_with(": --"))
}

fn compact_blank_lines(lines: &[String]) -> String {
    let mut out = String::new();
    let mut blank_count = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SectionState;
    use crate::data::{AppData, RuntimeNodeKind};
    use crate::sections::collection::CollectionState;
    use crate::sections::free_text::FreeTextState;
    use crate::sections::header::HeaderState;
    use crate::sections::list_select::ListSelectState;
    use std::path::PathBuf;

    fn test_section(id: &str, note_label: Option<&str>) -> SectionConfig {
        SectionConfig {
            id: id.to_string(),
            name: "Demo".to_string(),
            map_label: "Demo".to_string(),
            section_type: "free_text".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: None,
            lists: Vec::new(),
            note_label: note_label.map(str::to_string),
            group_id: String::new(),
            node_kind: RuntimeNodeKind::Section,
        }
    }

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

    #[test]
    fn initial_document_from_real_data_validates_against_current_structure() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let document = build_initial_document(
            &data.template,
            &data.sections,
            &states_for_real_data(&data),
            &HashMap::new(),
            &HashMap::new(),
            &data.boilerplate_texts,
        );

        validate_document_structure(&document, &data.sections)
            .expect("real editable document should validate");
    }

    #[test]
    fn headingless_section_validates_with_markers_only() {
        let section = test_section("foo", Some(""));
        let document = format!(
            "{}\nbody\n{}",
            marker_start(&section.id),
            marker_end(&section.id)
        );

        validate_section_anchors(&document, &[section])
            .expect("empty note_label sections should validate without heading text");
    }

    #[test]
    fn heading_after_start_marker_fails_validation() {
        let section = test_section("foo", Some("#### FOO"));
        let document = format!(
            "{}\nmanaged body\n{}\n#### FOO",
            marker_start(&section.id),
            marker_end(&section.id)
        );

        let err = validate_section_anchors(&document, &[section])
            .expect_err("heading after marker should fail validation");
        assert!(err.contains("appears after its start marker"));
    }
}
