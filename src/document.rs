//! Helpers for managing the editable note document structure.

use crate::app::SectionState;
use crate::data::SectionConfig;
use crate::note::{managed_heading_for_section, render_editable_document};
use std::collections::HashMap;

/// The canonical headings that every note document must contain.
pub const CANONICAL_HEADINGS: &[&str] = &[
    "## SUBJECTIVE",
    "## OBJECTIVE / OBSERVATIONS",
    "## TREATMENT / PLAN",
    "## POST-TREATMENT",
];

#[derive(Debug, Clone)]
pub struct SectionAnchorSpec {
    pub section_id: String,
    pub heading_text: String,
    pub marker_start: String,
    pub marker_end: String,
}

/// Builds the initial editable document with stable managed markers.
pub fn build_initial_document(
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
) -> String {
    render_editable_document(sections, states, sticky_values, boilerplate_texts)
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

/// Returns a list of heading lines found in the document along with their byte offsets.
pub fn parse_document_headings(document: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    let mut offset = 0;
    for line in document.lines() {
        if line.starts_with('#') {
            result.push((offset, line.to_string()));
        }
        offset += line.len() + 1;
    }
    result
}

/// Returns true if the document contains all canonical headings.
pub fn validate_canonical_headings(document: &str) -> bool {
    let headings: Vec<String> = parse_document_headings(document)
        .into_iter()
        .map(|(_, h)| h)
        .collect();
    CANONICAL_HEADINGS
        .iter()
        .all(|&canonical| headings.iter().any(|h| h == canonical))
}

pub fn validate_section_anchors(document: &str, sections: &[SectionConfig]) -> Result<(), String> {
    let headings = parse_document_headings(document);

    for spec in editable_section_specs(sections) {
        let heading_matches: Vec<usize> = headings
            .iter()
            .filter_map(|(offset, heading)| (heading == &spec.heading_text).then_some(*offset))
            .collect();
        match heading_matches.len() {
            0 => {
                return Err(format!(
                    "Missing managed section heading '{}' for '{}'.",
                    spec.heading_text, spec.section_id
                ))
            }
            1 => {}
            _ => {
                return Err(format!(
                    "Duplicate managed section headings '{}' for '{}'.",
                    spec.heading_text, spec.section_id
                ))
            }
        }

        let start_matches: Vec<usize> = document
            .match_indices(&spec.marker_start)
            .map(|(idx, _)| idx)
            .collect();
        let end_matches: Vec<usize> = document
            .match_indices(&spec.marker_end)
            .map(|(idx, _)| idx)
            .collect();

        match start_matches.len() {
            0 => {
                return Err(format!(
                    "Missing managed section start marker for '{}'.",
                    spec.section_id
                ))
            }
            1 => {}
            _ => {
                return Err(format!(
                    "Duplicate managed section start markers for '{}'.",
                    spec.section_id
                ))
            }
        }

        match end_matches.len() {
            0 => {
                return Err(format!(
                    "Missing managed section end marker for '{}'.",
                    spec.section_id
                ))
            }
            1 => {}
            _ => {
                return Err(format!(
                    "Duplicate managed section end markers for '{}'.",
                    spec.section_id
                ))
            }
        }

        if start_matches[0] >= end_matches[0] {
            return Err(format!(
                "Managed section markers are out of order for '{}'.",
                spec.section_id
            ));
        }

        if heading_matches[0] >= start_matches[0] {
            return Err(format!(
                "Managed section heading '{}' must appear before its markers for '{}'.",
                spec.heading_text, spec.section_id
            ));
        }
    }

    Ok(())
}

pub fn validate_document_structure(
    document: &str,
    sections: &[SectionConfig],
) -> Result<(), String> {
    if !validate_canonical_headings(document) {
        return Err("Required top-level headings are missing or renamed.".to_string());
    }

    validate_section_anchors(document, sections)
}

/// Returns the byte range `(start, end)` of the body content that follows the
/// given heading anchor up to (but not including) the next heading or end of
/// document. Returns `None` if the anchor is not found.
#[cfg_attr(not(test), allow(dead_code))]
pub fn find_section_bounds(document: &str, heading_anchor: &str) -> Option<(usize, usize)> {
    let headings = parse_document_headings(document);
    for (i, (offset, heading)) in headings.iter().enumerate() {
        if heading == heading_anchor {
            let body_start = offset + heading.len() + 1;
            let body_end = if i + 1 < headings.len() {
                headings[i + 1].0
            } else {
                document.len()
            };
            return Some((body_start, body_end));
        }
    }
    None
}

#[allow(dead_code)]
pub fn find_managed_section_bounds(document: &str, section_id: &str) -> Option<(usize, usize)> {
    let start_marker = marker_start(section_id);
    let end_marker = marker_end(section_id);
    let start_idx = document.find(&start_marker)?;
    let body_start = start_idx + start_marker.len() + 1;
    let end_idx = document[body_start..].find(&end_marker)? + body_start;
    Some((body_start, end_idx.saturating_sub(1)))
}

/// Replaces the body content of the section identified by `heading_anchor`
/// with `new_body`, leaving all other sections unchanged.
#[cfg_attr(not(test), allow(dead_code))]
pub fn replace_section_body(document: &str, heading_anchor: &str, new_body: &str) -> String {
    match find_section_bounds(document, heading_anchor) {
        None => document.to_string(),
        Some((start, end)) => {
            let mut out = String::with_capacity(document.len());
            out.push_str(&document[..start]);
            out.push_str(new_body);
            out.push_str(&document[end..]);
            out
        }
    }
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
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("#### ")
            && lines
                .get(i + 1)
                .is_some_and(|next| is_marker_start_line(next))
        {
            let heading = line;
            let mut body: Vec<String> = Vec::new();
            i += 2;

            while i < lines.len() && !is_marker_end_line(lines[i]) {
                if should_export_line(lines[i]) {
                    body.push(lines[i].to_string());
                }
                i += 1;
            }

            if i < lines.len() && is_marker_end_line(lines[i]) {
                i += 1;
            }

            if body.iter().any(|line| !line.trim().is_empty()) {
                out.push(heading.to_string());
                out.extend(body);
            }
            continue;
        }

        if is_marker_start_line(line) || is_marker_end_line(line) || !should_export_line(line) {
            i += 1;
            continue;
        }

        out.push(line.to_string());
        i += 1;
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

/// Ensures all canonical headings are present in the document. If any are
/// missing they are appended, preserving existing content untouched.
#[cfg_attr(not(test), allow(dead_code))]
pub fn repair_document_structure(document: &str) -> String {
    let headings: Vec<String> = parse_document_headings(document)
        .into_iter()
        .map(|(_, h)| h)
        .collect();
    let mut out = document.to_string();
    for &canonical in CANONICAL_HEADINGS {
        if !headings.iter().any(|h| h == canonical) {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
            out.push_str(canonical);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_sections() -> Vec<SectionConfig> {
        vec![
            SectionConfig {
                id: "subjective_section".to_string(),
                name: "Subjective".to_string(),
                map_label: "Subjective".to_string(),
                section_type: "free_text".to_string(),
                data_file: None,
                date_prefix: None,
                options: vec![],
                composite: None,
                fields: None,
                is_intake: false,
                heading_search_text: Some("## SUBJECTIVE".to_string()),
                heading_label: None,
                note_render_slot: Some("subjective_section".to_string()),
            },
            SectionConfig {
                id: "tx_mods".to_string(),
                name: "Tx Mods".to_string(),
                map_label: "Tx Mods".to_string(),
                section_type: "multi_field".to_string(),
                data_file: None,
                date_prefix: None,
                options: vec![],
                composite: None,
                fields: None,
                is_intake: false,
                heading_search_text: Some("TREATMENT MODIFICATIONS".to_string()),
                heading_label: None,
                note_render_slot: Some("tx_mods".to_string()),
            },
        ]
    }

    fn doc_with_markers() -> String {
        format!(
            "## SUBJECTIVE\n#### SUBJECTIVE\n{}\nsubjective body\n{}\n## OBJECTIVE / OBSERVATIONS\ncontent\n## TREATMENT / PLAN\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}\ntx mods body\n{}\n## POST-TREATMENT\ncontent\n",
            marker_start("subjective_section"),
            marker_end("subjective_section"),
            marker_start("tx_mods"),
            marker_end("tx_mods"),
        )
    }

    #[test]
    fn find_section_bounds_returns_correct_byte_range() {
        let doc = "## SUBJECTIVE\nsome content here\n## OBJECTIVE / OBSERVATIONS\nother content\n";
        let bounds = find_section_bounds(doc, "## SUBJECTIVE");
        assert!(bounds.is_some());
        let (start, end) = bounds.unwrap();
        let body = &doc[start..end];
        assert!(body.contains("some content here"));
        assert!(!body.contains("other content"));
    }

    #[test]
    fn replace_section_body_replaces_only_target_section() {
        let doc =
            "## SUBJECTIVE\noriginal subjective\n## OBJECTIVE / OBSERVATIONS\noriginal objective\n";
        let updated = replace_section_body(doc, "## SUBJECTIVE", "new subjective content\n");
        assert!(updated.contains("new subjective content"));
        assert!(!updated.contains("original subjective"));
        assert!(updated.contains("original objective"));
    }

    #[test]
    fn validate_canonical_headings_returns_true_for_complete_document() {
        let doc = format!(
            "{}\ncontent\n{}\ncontent\n{}\ncontent\n{}\ncontent\n",
            CANONICAL_HEADINGS[0],
            CANONICAL_HEADINGS[1],
            CANONICAL_HEADINGS[2],
            CANONICAL_HEADINGS[3],
        );
        assert!(validate_canonical_headings(&doc));
    }

    #[test]
    fn repair_document_structure_restores_missing_heading() {
        let doc = format!(
            "{}\nsubjective notes\n{}\nobjective notes\n{}\nplan notes\n",
            CANONICAL_HEADINGS[0], CANONICAL_HEADINGS[1], CANONICAL_HEADINGS[2],
        );
        let repaired = repair_document_structure(&doc);
        assert!(repaired.contains(CANONICAL_HEADINGS[3]));
        assert!(repaired.contains("subjective notes"));
    }

    #[test]
    fn validate_section_anchors_accepts_one_marker_pair_per_section() {
        let doc = doc_with_markers();
        let sections = fake_sections();
        let result = validate_section_anchors(&doc, &sections);
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[test]
    fn validate_section_anchors_rejects_missing_start_marker() {
        let doc = doc_with_markers().replace(&marker_start("tx_mods"), "");
        let sections = fake_sections();
        let err = validate_section_anchors(&doc, &sections).expect_err("missing marker must fail");
        assert!(
            err.contains("Missing managed section start marker"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn replace_managed_section_body_replaces_only_managed_range() {
        let doc = format!(
            "## SUBJECTIVE\n#### SUBJECTIVE\n{}\nold body\n{}\nUser text stays here.\n## OBJECTIVE / OBSERVATIONS\ncontent\n## TREATMENT / PLAN\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}\ntx mods body\n{}\n## POST-TREATMENT\ncontent\n",
            marker_start("subjective_section"),
            marker_end("subjective_section"),
            marker_start("tx_mods"),
            marker_end("tx_mods"),
        );
        let updated = replace_managed_section_body(&doc, "subjective_section", "new body")
            .expect("managed replacement should succeed");

        assert!(updated.contains("new body"));
        assert!(!updated.contains("old body"));
        assert!(updated.contains("User text stays here."));
        assert!(updated.contains("tx mods body"));
    }

    #[test]
    fn export_editable_document_strips_markers_empty_sections_and_placeholders() {
        let doc = format!(
            "#### EMPTY\n{}\n\n{}\n#### TREATMENT MODIFICATIONS & PREFERENCES\n{}\nPressure: Regular\nCommunication: --\n{}\nManual note\n",
            marker_start("empty"),
            marker_end("empty"),
            marker_start("tx_mods"),
            marker_end("tx_mods"),
        );

        let exported = export_editable_document(&doc);

        assert!(!exported.contains("scribblenot:section"));
        assert!(!exported.contains("#### EMPTY"));
        assert!(!exported.contains("Communication: --"));
        assert!(exported.contains("#### TREATMENT MODIFICATIONS & PREFERENCES"));
        assert!(exported.contains("Pressure: Regular"));
        assert!(exported.contains("Manual note"));
    }
}
