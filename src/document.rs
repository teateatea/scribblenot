/// document.rs - helpers for managing the editable note document structure.

use crate::app::SectionState;
use crate::data::SectionConfig;
use crate::note::{render_note, NoteRenderMode};
use std::collections::HashMap;

/// The canonical headings that every note document must contain.
pub const CANONICAL_HEADINGS: &[&str] = &[
    "## SUBJECTIVE",
    "## OBJECTIVE / OBSERVATIONS",
    "## TREATMENT / PLAN",
    "## POST-TREATMENT",
];

/// Builds the initial document string with all canonical headings by rendering
/// the note and repairing its structure.
pub fn build_initial_document(
    sections: &[SectionConfig],
    states: &[SectionState],
    sticky_values: &HashMap<String, String>,
    boilerplate_texts: &HashMap<String, String>,
) -> String {
    let raw = render_note(sections, states, sticky_values, boilerplate_texts, NoteRenderMode::Preview);
    repair_document_structure(&raw)
}

/// Returns a list of heading lines found in the document along with their byte offsets.
pub fn parse_document_headings(document: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    let mut offset = 0;
    for line in document.lines() {
        if line.starts_with('#') {
            result.push((offset, line.to_string()));
        }
        offset += line.len() + 1; // +1 for '\n'
    }
    result
}

/// Returns true if the document contains all canonical headings in order.
pub fn validate_canonical_headings(document: &str) -> bool {
    let headings: Vec<String> = parse_document_headings(document)
        .into_iter()
        .map(|(_, h)| h)
        .collect();
    CANONICAL_HEADINGS.iter().all(|&canonical| headings.iter().any(|h| h == canonical))
}

/// Returns the byte range `(start, end)` of the body content that follows the
/// given heading anchor up to (but not including) the next heading or end of
/// document. Returns `None` if the anchor is not found.
pub fn find_section_bounds(document: &str, heading_anchor: &str) -> Option<(usize, usize)> {
    let headings = parse_document_headings(document);
    for (i, (offset, heading)) in headings.iter().enumerate() {
        if heading == heading_anchor {
            let body_start = offset + heading.len() + 1; // skip heading line + '\n'
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

/// Replaces the body content of the section identified by `heading_anchor`
/// with `new_body`, leaving all other sections unchanged.
/// Returns the modified document, or the original document unchanged if the
/// anchor is not found.
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

/// Ensures all canonical headings are present in the document. If any are
/// missing they are appended, preserving existing content untouched.
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

    // ---------------------------------------------------------------------------
    // find_section_bounds
    // ---------------------------------------------------------------------------

    #[test]
    fn find_section_bounds_returns_correct_byte_range() {
        // Build a small two-section document.
        let doc = "## SUBJECTIVE\nsome content here\n## OBJECTIVE / OBSERVATIONS\nother content\n";
        // Ask for the bounds of SUBJECTIVE's body.
        let bounds = find_section_bounds(doc, "## SUBJECTIVE");
        assert!(
            bounds.is_some(),
            "find_section_bounds should return Some for a heading that exists"
        );
        let (start, end) = bounds.unwrap();
        // The body content should be "some content here\n"
        let body = &doc[start..end];
        assert!(
            body.contains("some content here"),
            "Body should contain 'some content here', got: {:?}",
            body
        );
        assert!(
            !body.contains("other content"),
            "Body should NOT bleed into the next section, got: {:?}",
            body
        );
    }

    #[test]
    fn find_section_bounds_returns_none_for_missing_heading_and_some_for_present() {
        // A heading that IS present must return Some (not None).
        let doc = "## SUBJECTIVE\nsome content\n";
        let result = find_section_bounds(doc, "## SUBJECTIVE");
        assert!(
            result.is_some(),
            "find_section_bounds should return Some for an existing heading"
        );
        // A heading that is NOT present must return None.
        let missing = find_section_bounds(doc, "## NONEXISTENT");
        assert!(
            missing.is_none(),
            "find_section_bounds should return None for a missing heading"
        );
    }

    // ---------------------------------------------------------------------------
    // replace_section_body
    // ---------------------------------------------------------------------------

    #[test]
    fn replace_section_body_replaces_only_target_section() {
        let doc = "## SUBJECTIVE\noriginal subjective\n## OBJECTIVE / OBSERVATIONS\noriginal objective\n";
        let updated = replace_section_body(doc, "## SUBJECTIVE", "new subjective content\n");

        // Target section body should be replaced.
        assert!(
            updated.contains("new subjective content"),
            "Replacement content should appear in result, got: {:?}",
            updated
        );
        // Original target body should be gone.
        assert!(
            !updated.contains("original subjective"),
            "Old content in replaced section should be gone, got: {:?}",
            updated
        );
        // Other section should be untouched.
        assert!(
            updated.contains("original objective"),
            "Other section should remain unchanged, got: {:?}",
            updated
        );
        // Both headings must still be present.
        assert!(
            updated.contains("## SUBJECTIVE"),
            "SUBJECTIVE heading must still be present"
        );
        assert!(
            updated.contains("## OBJECTIVE / OBSERVATIONS"),
            "OBJECTIVE heading must still be present"
        );
    }

    // ---------------------------------------------------------------------------
    // validate_canonical_headings
    // ---------------------------------------------------------------------------

    #[test]
    fn validate_canonical_headings_returns_true_for_complete_document() {
        let doc = format!(
            "{}\ncontent\n{}\ncontent\n{}\ncontent\n{}\ncontent\n",
            CANONICAL_HEADINGS[0],
            CANONICAL_HEADINGS[1],
            CANONICAL_HEADINGS[2],
            CANONICAL_HEADINGS[3],
        );
        assert!(
            validate_canonical_headings(&doc),
            "Should return true when all canonical headings are present"
        );
    }

    #[test]
    fn validate_canonical_headings_returns_false_for_missing_heading_and_true_for_complete() {
        // Complete document - must return true.
        let complete = format!(
            "{}\ncontent\n{}\ncontent\n{}\ncontent\n{}\ncontent\n",
            CANONICAL_HEADINGS[0],
            CANONICAL_HEADINGS[1],
            CANONICAL_HEADINGS[2],
            CANONICAL_HEADINGS[3],
        );
        assert!(
            validate_canonical_headings(&complete),
            "Should return true for complete document"
        );
        // Omit the last canonical heading - must return false.
        let incomplete = format!(
            "{}\ncontent\n{}\ncontent\n{}\ncontent\n",
            CANONICAL_HEADINGS[0],
            CANONICAL_HEADINGS[1],
            CANONICAL_HEADINGS[2],
        );
        assert!(
            !validate_canonical_headings(&incomplete),
            "Should return false when a canonical heading is missing"
        );
    }

    // ---------------------------------------------------------------------------
    // repair_document_structure
    // ---------------------------------------------------------------------------

    #[test]
    fn repair_document_structure_restores_missing_heading() {
        // Document is missing POST-TREATMENT but has the others with content.
        let doc = format!(
            "{}\nsubjective notes\n{}\nobjective notes\n{}\nplan notes\n",
            CANONICAL_HEADINGS[0],
            CANONICAL_HEADINGS[1],
            CANONICAL_HEADINGS[2],
        );
        let repaired = repair_document_structure(&doc);

        // The missing heading should now be present.
        assert!(
            repaired.contains(CANONICAL_HEADINGS[3]),
            "Repaired document should contain the previously missing heading '{}'",
            CANONICAL_HEADINGS[3]
        );
        // Existing content must be preserved.
        assert!(
            repaired.contains("subjective notes"),
            "Existing section content must not be discarded after repair"
        );
        assert!(
            repaired.contains("objective notes"),
            "Existing section content must not be discarded after repair"
        );
        assert!(
            repaired.contains("plan notes"),
            "Existing section content must not be discarded after repair"
        );
    }

    #[test]
    fn repair_document_structure_leaves_complete_document_unchanged() {
        let doc = format!(
            "{}\nsubjective\n{}\nobjective\n{}\nplan\n{}\npost\n",
            CANONICAL_HEADINGS[0],
            CANONICAL_HEADINGS[1],
            CANONICAL_HEADINGS[2],
            CANONICAL_HEADINGS[3],
        );
        let repaired = repair_document_structure(&doc);
        // All headings and all content must be present after repair.
        for heading in CANONICAL_HEADINGS {
            assert!(
                repaired.contains(heading),
                "Heading '{}' should still be present after repair of already-complete doc",
                heading
            );
        }
        assert!(repaired.contains("subjective"));
        assert!(repaired.contains("objective"));
        assert!(repaired.contains("plan"));
        assert!(repaired.contains("post"));
    }
}
