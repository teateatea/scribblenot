## Context

Roadmap item 5 notes the unit suite is healthy but cross-state behaviors are where regressions slip through. Three specific gaps exist in the composition and editable-doc flow:

1. The existing override test (`modal_composition_override_persists_on_confirm_and_reopens_with_source_intact`, `src/app.rs:4863`) verifies `editable_note` but never calls `export_editable_document` — the export path is untested for manual overrides.
2. The Ctrl+R reset flow in composition editing mode has no end-to-end test verifying the editable note reverts to the modal-resolved value rather than the cleared override.
3. Free-text editable-note sync via the `handle_free_text_key` confirm path is untested. The only existing sync test (`sync_section_into_editable_note_updates_only_current_managed_block`, `src/app.rs:3564`) calls `sync_section_into_editable_note` directly, bypassing the key handler.

## Approach

Add four `#[test]` functions to the existing `mod composition_span_tests` block (`src/app.rs:3948`). Each test exercises one cross-state path through the real handler methods and asserts on `editable_note` and/or `export_editable_document`. Reuse existing in-module helpers; follow the established pattern of in-memory data construction and direct method calls.

## Critical Files

- `src/app.rs` — add tests inside `mod composition_span_tests`; currently ends at line 5215
- `src/document.rs` — `export_editable_document` (line 104); already imported in lines 5179, 5203

## Reuse

- `app_with_single_field(field)` — `src/app.rs:4368`
- `list_field(ModalStart::List)` — `src/app.rs:4200`; produces a "Region" field with "Shoulder"/"Hip" items
- `select_only_filtered_modal_match(app, query)` — `src/app.rs:3966`
- `crate::document::export_editable_document` — `src/document.rs:104`
- `App::open_header_modal`, `set_modal_composition_text`, `confirm_modal_value` — existing App methods accessible from the test module via `use super::*`
- `SectionState::FreeText`, `FreeTextState`, `FreeTextMode` — available via `crate::sections::free_text`

## Steps

1. Locate the closing `}` of `mod composition_span_tests` in `src/app.rs` (currently line 5215). Add the four tests before that closing brace.

2. Add `manual_override_text_survives_export`:
   ```rust
   #[test]
   fn manual_override_text_survives_export() {
       let mut app = app_with_single_field(list_field(ModalStart::List));
       app.open_header_modal();
       app.handle_key(AppKey::Enter); // descend into list so modal has a cursor value
       app.handle_key(AppKey::CtrlChar('e')); // enter composition editing
       app.set_modal_composition_text("custom override".to_string());
       app.handle_key(AppKey::Enter); // exit composition editing
       app.confirm_modal_value("Shoulder".to_string());

       let exported = crate::document::export_editable_document(&app.editable_note);
       assert!(
           app.editable_note.contains("Region: custom override"),
           "editable note should contain override text"
       );
       assert!(
           exported.contains("Region: custom override"),
           "exported note should also contain override text"
       );
   }
   ```

3. Add `composition_override_ctrl_r_reverts_editable_note`:
   ```rust
   #[test]
   fn composition_override_ctrl_r_reverts_editable_note() {
       let mut app = app_with_single_field(list_field(ModalStart::List));
       app.open_header_modal();
       app.handle_key(AppKey::Enter); // give modal a resolved cursor value
       app.handle_key(AppKey::CtrlChar('e')); // enter composition editing
       app.set_modal_composition_text("should not appear".to_string());
       app.handle_key(AppKey::CtrlChar('r')); // reset override, exit editing
       app.confirm_modal_value("Shoulder".to_string());

       assert!(
           app.editable_note.contains("Region: Shoulder"),
           "editable note should contain the modal-resolved value after reset"
       );
       assert!(
           !app.editable_note.contains("should not appear"),
           "editable note must not contain the cleared override text"
       );
   }
   ```

4. Add `free_text_confirm_via_key_handler_syncs_editable_note`:
   ```rust
   #[test]
   fn free_text_confirm_via_key_handler_syncs_editable_note() {
       use crate::sections::free_text::{FreeTextMode, FreeTextState};

       let section = SectionConfig {
           id: "notes_section".to_string(),
           name: "Notes".to_string(),
           map_label: "Notes".to_string(),
           section_type: "free_text".to_string(),
           show_field_labels: true,
           data_file: None,
           fields: None,
           lists: Vec::new(),
           note_label: None,
           group_id: "subjective".to_string(),
           node_kind: RuntimeNodeKind::Section,
       };
       let group = SectionGroup {
           id: "subjective".to_string(),
           num: None,
           nav_label: "Subjective".to_string(),
           sections: vec![section.clone()],
           note: GroupNoteMeta::default(),
       };
       let data = AppData {
           template: RuntimeTemplate {
               id: "test".to_string(),
               children: vec![RuntimeGroup {
                   id: "subjective".to_string(),
                   nav_label: "Subjective".to_string(),
                   note: GroupNoteMeta::default(),
                   children: vec![RuntimeNode::Section(section.clone())],
               }],
           },
           groups: vec![group],
           sections: vec![section],
           list_data: HashMap::new(),
           checklist_data: HashMap::new(),
           collection_data: HashMap::new(),
           boilerplate_texts: HashMap::new(),
           keybindings: KeyBindings::default(),
       };

       let mut app = App::new(data, Config::default(), PathBuf::new());
       app.section_states[0] = SectionState::FreeText(FreeTextState {
           entries: vec!["patient reported pain in left shoulder".to_string()],
           cursor: 0,
           mode: FreeTextMode::Browsing,
           edit_buf: String::new(),
           skipped: false,
           completed: false,
       });
       app.handle_free_text_key(AppKey::Enter); // browsing + has entries: complete + sync + advance

       assert!(
           app.editable_note.contains("patient reported pain in left shoulder"),
           "editable note should reflect the committed free-text entry"
       );
       assert!(app.note_headings_valid, "document structure should remain valid after sync");
   }
   ```

5. Add `completing_two_free_text_sections_keeps_both_in_editable_note`:
   ```rust
   #[test]
   fn completing_two_free_text_sections_keeps_both_in_editable_note() {
       use crate::sections::free_text::{FreeTextMode, FreeTextState};

       let make_section = |id: &str, name: &str, group_id: &str| SectionConfig {
           id: id.to_string(),
           name: name.to_string(),
           map_label: name.to_string(),
           section_type: "free_text".to_string(),
           show_field_labels: true,
           data_file: None,
           fields: None,
           lists: Vec::new(),
           note_label: None,
           group_id: group_id.to_string(),
           node_kind: RuntimeNodeKind::Section,
       };
       let sec_a = make_section("section_a", "SectionA", "group_a");
       let sec_b = make_section("section_b", "SectionB", "group_b");
       let data = AppData {
           template: RuntimeTemplate {
               id: "test".to_string(),
               children: vec![
                   RuntimeGroup {
                       id: "group_a".to_string(),
                       nav_label: "Group A".to_string(),
                       note: GroupNoteMeta::default(),
                       children: vec![RuntimeNode::Section(sec_a.clone())],
                   },
                   RuntimeGroup {
                       id: "group_b".to_string(),
                       nav_label: "Group B".to_string(),
                       note: GroupNoteMeta::default(),
                       children: vec![RuntimeNode::Section(sec_b.clone())],
                   },
               ],
           },
           groups: vec![
               SectionGroup {
                   id: "group_a".to_string(),
                   num: None,
                   nav_label: "Group A".to_string(),
                   sections: vec![sec_a.clone()],
                   note: GroupNoteMeta::default(),
               },
               SectionGroup {
                   id: "group_b".to_string(),
                   num: None,
                   nav_label: "Group B".to_string(),
                   sections: vec![sec_b.clone()],
                   note: GroupNoteMeta::default(),
               },
           ],
           sections: vec![sec_a, sec_b],
           list_data: HashMap::new(),
           checklist_data: HashMap::new(),
           collection_data: HashMap::new(),
           boilerplate_texts: HashMap::new(),
           keybindings: KeyBindings::default(),
       };

       let mut app = App::new(data, Config::default(), PathBuf::new());

       app.section_states[0] = SectionState::FreeText(FreeTextState {
           entries: vec!["section a content".to_string()],
           cursor: 0,
           mode: FreeTextMode::Browsing,
           edit_buf: String::new(),
           skipped: false,
           completed: false,
       });
       app.handle_free_text_key(AppKey::Enter); // complete section 0, advance to 1

       app.section_states[1] = SectionState::FreeText(FreeTextState {
           entries: vec!["section b content".to_string()],
           cursor: 0,
           mode: FreeTextMode::Browsing,
           edit_buf: String::new(),
           skipped: false,
           completed: false,
       });
       app.handle_free_text_key(AppKey::Enter); // complete section 1

       assert!(
           app.editable_note.contains("section a content"),
           "section A output should remain in editable note after section B syncs"
       );
       assert!(
           app.editable_note.contains("section b content"),
           "section B output should appear in editable note"
       );
       assert!(app.note_headings_valid, "document structure should remain valid after both syncs");
   }
   ```

6. Run `cargo test --lib composition_span_tests` and verify all 4 new tests pass alongside the existing tests in the module.

## Verification

### Manual tests

None — this change adds only test code.

### Automated tests

- `cargo test --lib composition_span_tests` — runs the full `mod composition_span_tests` block; all 4 new tests plus the existing ~20 tests should pass
- `cargo test --lib` — full library test suite should remain green with no regressions
