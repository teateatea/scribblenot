**Task**: #52 - Extract hard-coded boilerplate strings from note.rs into editable YML data files

**Context**: Sub-task 52.3 added `boilerplate_texts: HashMap<String, String>` to `AppData` and populates it at load time. Sub-task 52.5 will replace hard-coded string literals in `render_note()` with lookups into that map. This sub-task threads the `boilerplate_texts` reference from the call sites in `main.rs`, `ui.rs`, and `app.rs` down into `render_note()` and `section_start_line()` in `note.rs`, so 52.5 can perform lookups without any further signature changes.

**Approach**: Add a `boilerplate_texts: &HashMap<String, String>` parameter to `render_note()` and `section_start_line()`. Update every call site to pass `&app.data.boilerplate_texts`. The parameter is unused inside `render_note()` until 52.5 touches it; Rust allows an unused parameter in a non-test function without a lint error (no `#[allow]` needed since it will be used immediately in 52.5). Do not change any string literals yet - that is strictly 52.5's scope.

**Critical Files**:
- `src/note.rs` lines 42-49 (`section_start_line` signature and its call to `render_note`), line 81-86 (`render_note` signature)
- `src/main.rs` lines 68 and 84 (two `render_note` calls in `run_app`)
- `src/ui.rs` line 909 (`render_note` call inside `render_note_pane`)
- `src/app.rs` line 384 (`section_start_line` call inside `update_note_scroll`)
- `src/note.rs` lines 617, 621, 639, 640, 658, 672, 674 (test calls to `section_start_line` and `render_note`)

**Reuse**: `app.data.boilerplate_texts` is already populated by `AppData::load`; no new data structures needed. The `App` struct already exposes `data: AppData` as a public field, so call sites can reach it as `&app.data.boilerplate_texts`.

**Steps**:

1. In `src/note.rs`, add `boilerplate_texts: &HashMap<String, String>` as the fifth parameter to `render_note`, and thread it through the internal call from `section_start_line`:

```diff
 pub fn render_note(
     sections: &[SectionConfig],
     states: &[SectionState],
     sticky_values: &HashMap<String, String>,
+    boilerplate_texts: &HashMap<String, String>,
     mode: NoteRenderMode,
 ) -> String {
```

2. In `src/note.rs`, add `boilerplate_texts: &HashMap<String, String>` as the sixth parameter to `section_start_line`, and pass it through to `render_note`:

```diff
 pub fn section_start_line(
     sections: &[SectionConfig],
     states: &[SectionState],
     sticky_values: &HashMap<String, String>,
     groups: &[SectionGroup],
+    boilerplate_texts: &HashMap<String, String>,
     section_id: &str,
 ) -> u16 {
-    let note = render_note(sections, states, sticky_values, NoteRenderMode::Preview);
+    let note = render_note(sections, states, sticky_values, boilerplate_texts, NoteRenderMode::Preview);
```

3. Update the two `render_note` calls in `src/main.rs` (lines 68 and 84) to pass `&app.data.boilerplate_texts`:

```diff
-let note_text = note::render_note(&app.sections, &app.section_states, &app.config.sticky_values, note::NoteRenderMode::Export);
+let note_text = note::render_note(&app.sections, &app.section_states, &app.config.sticky_values, &app.data.boilerplate_texts, note::NoteRenderMode::Export);
```

Apply this change to both the `note_completed` block (line 68) and the `copy_requested` block (line 84).

4. Update the `render_note` call in `src/ui.rs` (line 909) inside `render_note_pane`:

```diff
-let note_text = render_note(&app.sections, &app.section_states, &app.config.sticky_values, NoteRenderMode::Preview);
+let note_text = render_note(&app.sections, &app.section_states, &app.config.sticky_values, &app.data.boilerplate_texts, NoteRenderMode::Preview);
```

5. Update the `section_start_line` call in `src/app.rs` (line 384) inside `update_note_scroll`:

```diff
-self.note_scroll = crate::note::section_start_line(&self.sections, &self.section_states, &self.config.sticky_values, &self.data.groups, &section_id);
+self.note_scroll = crate::note::section_start_line(&self.sections, &self.section_states, &self.config.sticky_values, &self.data.groups, &self.data.boilerplate_texts, &section_id);
```

6. Update all test call sites in `src/note.rs` (inside the `#[cfg(test)]` block) to pass an empty `HashMap` as `boilerplate_texts`. Each test already constructs `let sticky = HashMap::new();` - add a `let bp = HashMap::new();` alongside it and thread it in:

For `section_start_line` test calls: pass `&bp` as the new fifth argument (before `section_id`).
For `render_note` test calls: pass `&bp` as the new fourth argument (before `NoteRenderMode::Preview`).

There are seven affected test call sites: lines 617, 621, 639, 640, 658, 672, 674.

7. Run `cargo build` to confirm the project compiles without errors:

```
cargo build
```

**Verification**:

### Manual tests
- Launch the application and confirm the note preview pane renders correctly (no blank note, no panic).
- Trigger a clipboard copy (Shift+Enter or manual copy key) and paste into a text editor; confirm the note text appears unchanged from pre-change behavior.
- Navigate between sections on the section map and confirm scrolling in the note preview pane still jumps to the correct heading.

### Automated tests
- `cargo test` must pass with zero failures. The existing tests in `note.rs` (`empty_tx_plan_falls_back_to_post_treatment_heading`, `non_empty_tx_plan_returns_own_heading_line`, etc.) exercise both `render_note` and `section_start_line` and will catch any signature mismatch.
- No new tests are needed for this sub-task since it introduces no behavioral change - the new parameter is passed through but not yet read.

## Changelog

### Review - 2026-04-02
- #1 (blocking): Added missing test call site at line 658 (`skipped_intake_section_returns_zero`) to Critical Files and step 6 - plan listed six sites but seven exist, omitting it would cause a compile error.

## Progress
- Step 1: Added `boilerplate_texts: &HashMap<String, String>` as fifth parameter to `render_note` in note.rs
- Step 2: Added `boilerplate_texts: &HashMap<String, String>` as sixth parameter to `section_start_line` and threaded it to `render_note` call
- Step 3: Updated both `render_note` calls in main.rs to pass `&app.data.boilerplate_texts`
- Step 4: Updated `render_note` call in ui.rs to pass `&app.data.boilerplate_texts`
- Step 5: Updated `section_start_line` call in app.rs to pass `&self.data.boilerplate_texts`
- Step 6: Updated all 7 test call sites in note.rs with `let bp = HashMap::new();` and threaded `&bp`
- Step 7: cargo build succeeds (1 expected unused variable warning); cargo test passes 119/119

## Implementation
Complete - 2026-04-02
