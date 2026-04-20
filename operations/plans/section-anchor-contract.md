## Task

#73 Phase 1 Recovery - Realign the editable document model and iced port with the approved outcome (step 1: anchor contract)

## Context

Roadmap item #15 flags that the tray app needs safe per-section replacement inside `editable_note`, but the current anchor helpers have two silent correctness gaps:

1. `managed_heading_for_section` returns `Some("")` when `note_label: ""` in YAML. `SectionAnchorSpec` stores that as `heading_text: String`, and `validate_section_anchors` checks `document.contains("")` which is trivially true for any document. Sections with an empty note label are effectively unvalidated.

2. There is no ordering check. Even for sections with a real heading, the validator only confirms the heading text appears somewhere in the document — it does not confirm the heading precedes the start marker, which is the guarantee required for safe section-scoped replacement.

These gaps mean the anchor contract is implicitly weaker than the codebase assumes and weaker than PLAN-73-phase1-realignment.md requires.

## Approach

Make `SectionAnchorSpec.heading_text` `Option<String>`, treat `Some("")` as `None` in spec construction, update validation to enforce marker ordering and heading-before-marker ordering, and add a module-level doc comment in `document.rs` that states the contract explicitly. No changes to `note.rs` rendering or `app.rs` sync paths — this plan is limited to making the contract correct and documented.

## Critical Files

- `src/document.rs` — all changes land here (lines 6-76 for struct and validation, module level for doc)
- `src/note.rs:87-91` — `managed_heading_for_section` read-only; confirms it can return `Some("")` when `note_label` is an empty string

## Reuse

- `marker_start` / `marker_end` (document.rs:32-38) — unchanged, reused in ordering check
- `managed_heading_for_section` (note.rs:87-91) — call result treated as `None` when empty

## Steps

1. **Change `SectionAnchorSpec.heading_text` to `Option<String>`**

```diff
 pub struct SectionAnchorSpec {
     pub section_id: String,
-    pub heading_text: String,
+    pub heading_text: Option<String>,
     pub marker_start: String,
     pub marker_end: String,
 }
```

2. **Update `editable_section_specs` to treat empty heading as `None`**

Replace the `filter_map` (which required a non-None heading) with `map` so all sections are always included. Filter the empty-string case:

```diff
 pub fn editable_section_specs(sections: &[SectionConfig]) -> Vec<SectionAnchorSpec> {
     sections
         .iter()
-        .filter_map(|cfg| {
-            Some(SectionAnchorSpec {
-                section_id: cfg.id.clone(),
-                heading_text: managed_heading_for_section(cfg)?,
-                marker_start: marker_start(&cfg.id),
-                marker_end: marker_end(&cfg.id),
-            })
-        })
+        .map(|cfg| {
+            let heading = managed_heading_for_section(cfg).filter(|h| !h.is_empty());
+            SectionAnchorSpec {
+                section_id: cfg.id.clone(),
+                heading_text: heading,
+                marker_start: marker_start(&cfg.id),
+                marker_end: marker_end(&cfg.id),
+            }
+        })
         .collect()
 }
```

3. **Update `validate_section_anchors` to check ordering and skip absent headings**

```diff
 pub fn validate_section_anchors(document: &str, sections: &[SectionConfig]) -> Result<(), String> {
     for spec in editable_section_specs(sections) {
-        if !document.contains(&spec.heading_text) {
-            return Err(format!(
-                "Missing managed section heading '{}' for '{}'.",
-                spec.heading_text, spec.section_id
-            ));
-        }
         if !document.contains(&spec.marker_start) {
             return Err(format!(
                 "Missing managed section start marker for '{}'.",
@@ -68,6 +62,18 @@
                 spec.section_id
             ));
         }
+        if let Some(ref heading) = spec.heading_text {
+            if !document.contains(heading.as_str()) {
+                return Err(format!(
+                    "Missing managed section heading '{}' for '{}'.",
+                    heading, spec.section_id
+                ));
+            }
+            let heading_pos = document.find(heading.as_str()).unwrap();
+            let marker_pos = document.find(&spec.marker_start).unwrap();
+            if heading_pos > marker_pos {
+                return Err(format!(
+                    "Section heading for '{}' appears after its start marker.",
+                    spec.section_id
+                ));
+            }
+        }
     }
     Ok(())
 }
```

4. **Add module-level doc comment stating the anchor contract**

Insert above the `use` statements at the top of `document.rs`:

```diff
+//! Editable document helpers and anchor contract.
+//!
+//! # Anchor contract
+//!
+//! Each runtime-editable section is represented in the document by:
+//!
+//! 1. An optional visible heading (e.g. `#### SUBJECTIVE`). When present it must
+//!    appear in the document *before* the section's start marker.
+//! 2. A start marker: `<!-- scribblenot:section id=<id>:start -->`
+//! 3. A machine-managed body region between the markers.
+//! 4. An end marker:   `<!-- scribblenot:section id=<id>:end -->`
+//!
+//! Replacement (`replace_managed_section_body`) rewrites only the body between
+//! the markers. Text outside the markers — including user free edits — is
+//! preserved. If either marker is absent or out of order the document is
+//! considered invalid and targeted replacement must be blocked.
+//!
+//! A section with an empty `note_label` has no heading anchor; its markers are
+//! the sole stable replacement boundary.
 use crate::app::SectionState;
```

5. **Update the existing test and add two new cases**

In `document.rs` tests, the existing `initial_document_from_real_data_validates_against_current_structure` test covers the happy path. Add:

- A test for a headingless section: construct a minimal document with only markers (no `####` line) and confirm `validate_section_anchors` passes for a config with empty `note_label`.
- A test for out-of-order anchor: place the start marker before the heading text and confirm validation returns an `Err`.

## Verification

### Manual tests

- Run `cargo build` — should compile cleanly with no warnings about the `heading_text` field type change.
- Check `editable_section_specs` call sites (currently only inside `validate_section_anchors` and the test) — confirm none break on the `Option<String>` change.

### Automated tests

- `cargo test` — all existing document tests must pass.
- New headingless-section test: build a document with `<!-- scribblenot:section id=foo:start -->` / body / `<!-- scribblenot:section id=foo:end -->` and no `#### FOO` line; create a `SectionConfig` with `note_label: Some("")`; assert `validate_section_anchors` returns `Ok(())`.
- New ordering test: build a document where the start marker appears on a line before the heading; assert `validate_section_anchors` returns `Err` containing "appears after its start marker".
