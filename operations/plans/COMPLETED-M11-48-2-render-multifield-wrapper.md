## Task

#48 - Generalize multi_field note rendering to support arbitrary sections beyond the appointment header

## Context

Sub-task 1 introduced two-pass rendering in `render_note` so every non-header `multi_field` section is rendered generically. Sub-task 2 exposes that same dispatch logic as a standalone public free function `render_multifield_section`. Five TDD tests already exist in `src/note.rs` (lines 1338-1438) and fail at compile time because the function does not yet exist. The function must be public so tests (and future callers) can use it directly without going through `render_note`.

## Approach

Add a single `pub fn render_multifield_section` in `src/note.rs` immediately after `format_header_generic_export` (after line 551). The function accepts `cfg: &SectionConfig` (available for future use but not needed for dispatch), `hs: &HeaderState`, `sticky_values: &HashMap<String, String>`, and `mode: NoteRenderMode`. For Preview it calls `format_header_generic_preview` and wraps the result in `Some`. For Export it delegates directly to `format_header_generic_export` (which already returns `Option<String>`). No new abstractions are needed.

## Critical Files

- `src/note.rs` line 551 (insertion point: after closing brace of `format_header_generic_export`)
- `src/note.rs` lines 494-551 (`format_header_generic_preview` and `format_header_generic_export` - called by the new wrapper)
- `src/note.rs` lines 1338-1438 (five TDD tests that must compile and pass)

## Reuse

- `format_header_generic_preview(hs, sticky_values)` - line 494, called for `NoteRenderMode::Preview`
- `format_header_generic_export(hs, sticky_values)` - line 524, called for `NoteRenderMode::Export`
- `crate::sections::header::HeaderState` - already imported in test module via `use super::*`
- `SectionConfig` - already imported at top of `src/note.rs` via `use crate::data::{SectionConfig, SectionGroup}`

## Steps

1. In `src/note.rs`, insert the following function after line 551 (the closing brace of `format_header_generic_export`), before `fn format_header_date`:

```diff
+/// Dispatch a non-header multi_field section to the appropriate renderer.
+/// Preview always returns Some (empty fields show "--" placeholders).
+/// Export returns None when all fields are empty.
+pub fn render_multifield_section(
+    _cfg: &SectionConfig,
+    hs: &crate::sections::header::HeaderState,
+    sticky_values: &HashMap<String, String>,
+    mode: NoteRenderMode,
+) -> Option<String> {
+    match mode {
+        NoteRenderMode::Preview => Some(format_header_generic_preview(hs, sticky_values)),
+        NoteRenderMode::Export => format_header_generic_export(hs, sticky_values),
+    }
+}
+
 fn format_header_date(date: &str) -> String {
```

2. Run `cargo test` to confirm all five ST48-2 tests pass and no existing tests regress.

## Verification

### Manual tests

- No UI-visible change for this sub-task; the function is only called by tests and future callers. Confirm the app still builds and opens normally.

### Automated tests

- `cargo test` - the five pre-written tests at lines 1386-1438 in `src/note.rs` cover all required behaviors:
  - `render_multifield_section_preview_two_fields_joined_by_newline`: two filled fields, Preview returns `Some("alpha: hello\nbeta: world")`
  - `render_multifield_section_preview_empty_field_shows_placeholder`: empty field, Preview returns `Some` containing `"beta: --"`
  - `render_multifield_section_export_values_only_joined_by_newline`: two filled fields, Export returns `Some("hello\nworld")`
  - `render_multifield_section_export_none_when_all_empty`: both fields empty, Export returns `None`
  - `render_multifield_section_export_partial_returns_some`: one filled field, Export returns `Some("only_this")`

## Progress

- Step 1: Inserted `pub fn render_multifield_section` after `format_header_generic_export` in src/note.rs
- Step 2: Ran `cargo test` - all 152 tests pass including 5 ST48-2 TDD tests

## Implementation
Complete - 2026-04-03
