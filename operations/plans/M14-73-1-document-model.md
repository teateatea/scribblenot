## Task

#73 Phase 1 - Editable document model and iced port (sub-task 1: document.rs helpers + App fields)

## Context

The approved plan for task #73 requires introducing an editable document model before the iced UI port. Sub-task 1 covers only the document helper module and the four new App fields. The stub implementations in `src/document.rs` were written by the Test Writer and the tests are already defined but all currently fail. `src/app.rs` does not yet have `editable_note`, `note_headings_valid`, `show_window`, or `clipboard_import`.

## Approach

Implement the six helpers in `src/document.rs` using only the standard library and `CANONICAL_HEADINGS`. Add the four fields to the `App` struct in `src/app.rs` and initialize `editable_note` by calling `build_initial_document` (which wraps `note::render_note`) inside `App::new`. Keep all changes confined to these two files; do not touch the iced or ratatui port files.

## Critical Files

- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/document.rs` - all six helpers are stubs (lines 13-53); tests are at lines 55-248
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/app.rs` - `App` struct definition at lines 62-81; `App::new` at lines 103-127
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/note.rs` - `render_note` signature at line 71 and `NoteRenderMode` at line 8; these are reused by `build_initial_document`

## Reuse

- `note::render_note(sections, states, sticky_values, boilerplate_texts, NoteRenderMode::Preview)` - call this from `build_initial_document` to produce the initial editable markdown rather than duplicating rendering logic
- `CANONICAL_HEADINGS` constant already defined in `document.rs` - use in every helper that needs the heading list

## Steps

1. Implement `parse_document_headings` in `src/document.rs`.

   Scan the document line by line. A heading line starts with `#`. Collect `(byte_offset, line.to_string())` for every such line. Return the vec.

   ```rust
   pub fn parse_document_headings(document: &str) -> Vec<(usize, String)> {
       let mut result = Vec::new();
       let mut offset = 0;
       for line in document.lines() {
           if line.starts_with('#') {
               result.push((offset, line.to_string()));
           }
           offset += line.len() + 1; // +1 for '\n'
           // Note: does not handle '\r\n'; documents are Unix line endings
       }
       result
   }
   ```

2. Implement `validate_canonical_headings` in `src/document.rs`.

   Use `parse_document_headings` to collect all heading lines. Then verify that every entry in `CANONICAL_HEADINGS` appears at least once as a full-line match among the collected headings (exact string match against the heading text, not a substring match). Return `true` only when all four are found.

   ```rust
   pub fn validate_canonical_headings(document: &str) -> bool {
       let headings: Vec<String> = parse_document_headings(document)
           .into_iter()
           .map(|(_, h)| h)
           .collect();
       CANONICAL_HEADINGS.iter().all(|&canonical| headings.iter().any(|h| h == canonical))
   }
   ```

3. Implement `find_section_bounds` in `src/document.rs`.

   Walk the parsed headings. When the heading text matches `heading_anchor` exactly, note the byte offset of the character immediately after the newline that terminates the heading line. The end of the section is the byte offset of the next heading (or `document.len()` if there is none). Return `Some((body_start, body_end))`.

   ```rust
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
   ```

4. Implement `replace_section_body` in `src/document.rs`.

   Call `find_section_bounds`. If `None`, return the document unchanged. Otherwise splice the new body into the document using string slicing: prefix up to `body_start`, then `new_body`, then the suffix from `body_end`.

   ```rust
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
   ```

5. Implement `repair_document_structure` in `src/document.rs`.

   Start with the existing document string. For each canonical heading that is not already present (checked via `validate_canonical_headings` per heading), append a blank-line separator and the heading. Preserve all existing content.

   ```rust
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
   ```

6. Implement `build_initial_document` in `src/document.rs`.

   Change the signature to accept the parameters needed by `note::render_note`, then call it and run `repair_document_structure` on the result to guarantee all canonical headings are present.

   Add these `use` statements at the top of `src/document.rs` (file-level, before the first `pub` item):

   ```rust
   use crate::app::SectionState;
   use crate::data::SectionConfig;
   use crate::note::{render_note, NoteRenderMode};
   use std::collections::HashMap;
   ```

   Replace the stub `build_initial_document` function with:

   ```rust
   pub fn build_initial_document(
       sections: &[SectionConfig],
       states: &[SectionState],
       sticky_values: &HashMap<String, String>,
       boilerplate_texts: &HashMap<String, String>,
   ) -> String {
       let raw = render_note(sections, states, sticky_values, boilerplate_texts, NoteRenderMode::Preview);
       repair_document_structure(&raw)
   }
   ```

   Note: the existing stub had no parameters. Update the signature only in `document.rs`; callers will be wired in step 7.

7. Add four new fields to `App` in `src/app.rs` and initialize them in `App::new`.

   In the `App` struct (around line 62):

   ```diff
    pub struct App {
        pub sections: Vec<SectionConfig>,
        ...
        pub hint_buffer: String,
   +    pub editable_note: String,
   +    pub note_headings_valid: bool,
   +    pub show_window: bool,
   +    pub clipboard_import: Option<String>,
    }
   ```

   In `App::new` (around line 107), add imports and initialization. Add `use crate::document::build_initial_document;` to the top of `app.rs`, then in the `Self { ... }` block:

   ```diff
    let pane_swapped = config.is_swapped();
   +let editable_note = build_initial_document(
   +    &sections,
   +    &section_states,
   +    &config.sticky_values,
   +    &data.boilerplate_texts,
   +);
   +let note_headings_valid = crate::document::validate_canonical_headings(&editable_note);
    Self {
        sections,
        section_states,
        ...
        hint_buffer: String::new(),
   +    editable_note,
   +    note_headings_valid,
   +    show_window: false,
   +    clipboard_import: None,
    }
   ```

8. Update `main.rs` module declaration if `document` is not already declared.

   Check `src/main.rs` line 4: `mod document;` is already present. No change needed.

9. Run `cargo test` to verify all tests in `document.rs` pass.

## Verification

### Manual tests

- Run `cargo build` and confirm the project compiles with no errors.
- Add a temporary `eprintln!("{}", app.editable_note)` in `main.rs` before the terminal setup, run the app, and confirm the editable note contains all four canonical headings (`## SUBJECTIVE`, `## OBJECTIVE / OBSERVATIONS`, `## TREATMENT / PLAN`, `## POST-TREATMENT`). Remove the debug line after checking.

### Automated tests

- Run `cargo test document` to execute all seven tests in `src/document.rs`:
  - `find_section_bounds_returns_correct_byte_range`
  - `find_section_bounds_returns_none_for_missing_heading_and_some_for_present`
  - `replace_section_body_replaces_only_target_section`
  - `validate_canonical_headings_returns_true_for_complete_document`
  - `validate_canonical_headings_returns_false_for_missing_heading_and_true_for_complete`
  - `repair_document_structure_restores_missing_heading`
  - `repair_document_structure_leaves_complete_document_unchanged`
- All tests must pass with zero failures before this sub-task is considered complete.

## Prefect-1 Report

### Issues Found

**Nit**
- N1 (`M14-73-1-document-model.md`, Step 6): The `use` statements and the `build_initial_document` function body were presented in a single fused code block, which could lead an implementer to place the `use` statements inside the function body rather than at file level. While Rust allows scoped `use` inside functions, file-level placement is conventional and clearer. Split the block into two: one for the `use` statements (with an explicit note that they go at the top of the file) and one for the function replacement.

### Cross-Check Results

- `document.rs` stubs (lines 13-53): confirmed present and match plan description.
- `app.rs` App struct (lines 62-81): confirmed; `hint_buffer: String` is the last field at line 80 - diff target is correct.
- `app.rs` App::new (lines 103-127): confirmed; `let pane_swapped = config.is_swapped();` at line 106 is the correct context line for the diff.
- `note::render_note` signature (line 71): confirmed; call in step 6 matches exactly.
- `config.sticky_values` on `Config`: verified in `config.rs:11`.
- `data.boilerplate_texts` on `AppData`: verified in `data.rs:207`.
- `mod document;` in `main.rs`: present at line 4 - step 8 no-op is correct.
- Test count (7): verified by counting `#[test]` functions in `document.rs`.

## Changelog

### Review - 2026-04-06
- #1: Corrected test count from "eight" to "seven" in Verification section (document.rs contains 7 tests, not 8)

### Review - 2026-04-06 (Prefect-1)
- N1: Split step 6 code block into two blocks - `use` statements with explicit file-level placement note, then separate function replacement block

## Progress
- Step 1: Implemented parse_document_headings - scans lines for '#' prefix, collects (byte_offset, line) tuples
- Step 2: Implemented validate_canonical_headings - checks all CANONICAL_HEADINGS appear in parsed headings
- Step 3: Implemented find_section_bounds - returns (body_start, body_end) byte range for a heading's body
- Step 4: Implemented replace_section_body - splices new body into document using find_section_bounds
- Step 5: Implemented repair_document_structure - appends missing canonical headings to document
- Step 6: Implemented build_initial_document with use imports - calls render_note then repair_document_structure
- Step 7: Added editable_note, note_headings_valid, show_window, clipboard_import fields to App struct and initialized in App::new
- Step 8: Verified mod document already declared in main.rs - no change needed
- Step 9: All 7 document tests pass

## Implementation
Complete - 2026-04-06
