use crate::app::SectionState;
use crate::data::{SectionConfig, SectionGroup};
use crate::note::{managed_heading_for_section, render_editable_document};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SectionAnchorSpec {
    pub section_id: String,
    pub heading_text: String,
    pub marker_start: String,
    pub marker_end: String,
}

pub fn build_initial_document(
    groups: &[SectionGroup],
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
) -> String {
    render_editable_document(groups, sections, states, sticky_values, boilerplate_texts)
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
        .filter_map(|cfg| {
            Some(SectionAnchorSpec {
                section_id: cfg.id.clone(),
                heading_text: managed_heading_for_section(cfg)?,
                marker_start: marker_start(&cfg.id),
                marker_end: marker_end(&cfg.id),
            })
        })
        .collect()
}

pub fn validate_section_anchors(document: &str, sections: &[SectionConfig]) -> Result<(), String> {
    for spec in editable_section_specs(sections) {
        if !document.contains(&spec.heading_text) {
            return Err(format!(
                "Missing managed section heading '{}' for '{}'.",
                spec.heading_text, spec.section_id
            ));
        }
        if !document.contains(&spec.marker_start) {
            return Err(format!(
                "Missing managed section start marker for '{}'.",
                spec.section_id
            ));
        }
        if !document.contains(&spec.marker_end) {
            return Err(format!(
                "Missing managed section end marker for '{}'.",
                spec.section_id
            ));
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
    use crate::data::AppData;
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
    fn initial_document_from_real_data_validates_against_current_structure() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let data = AppData::load(dir).expect("real data loads");
        let document = build_initial_document(
            &data.groups,
            &data.sections,
            &states_for_real_data(&data),
            &HashMap::new(),
            &data.boilerplate_texts,
        );

        validate_document_structure(&document, &data.sections)
            .expect("real editable document should validate");
    }
}
