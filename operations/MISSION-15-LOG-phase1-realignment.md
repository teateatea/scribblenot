# Mission Log: phase1-realignment

## Mission
- Slug: phase1-realignment
- Date: 2026-04-06
- Start-Time: 2026-04-06T12:30:37-04:00
- Source Plan: `operations/plans/PLAN-73-phase1-realignment.md`
- Mode: Pathfinder mission-team adaptation for standalone plan file
- Scope:
  - document-model completion
  - editable-note reconciliation
  - validation + tests
  - highest-value iced interaction fixes if milestone 1 lands cleanly

## Baseline

- `cargo build` passes on the current checkout
- `cargo test` passes on the current checkout
- `SCRIBBLENOT_HEADLESS=1 cargo run` passes on the current checkout
- Remaining gap is architectural: document anchors, reconciliation, validation refresh, and incomplete iced interaction

## Log

### 2026-04-06T12:30:37-04:00
- Mission started from `PLAN-73-phase1-realignment.md`
- Adaptation note: source Pathfinder protocol expects `MISSION-<N>-BRIEF.md` plus `TASKS.md` entries; this run uses the standalone plan as the mission brief and preserves the same execution discipline locally

### 2026-04-06T13:05:00-04:00
- Milestone reached: document-model completion checkpoint
- Implemented:
  - switched initial editable-note generation onto the managed editable-document renderer
  - required stable visible subheadings for shared sections (`header`, `subjective_section`, `tx_regions`, `objective_section`, `post_treatment`)
  - tightened document validation to require both canonical top-level headings and per-section heading-plus-marker anchors
  - kept structured-to-document reconciliation wired through `App::sync_section_into_editable_note`
  - confirmed direct editor edits recompute structure validity via `set_editable_note`
  - added app-level regression tests for section sync and manual-edit invalidation
- Verification:
  - `cargo test document -- --nocapture` passed
  - `cargo test` passed (`203 passed`)
  - direct headless binary launch passed via `.\target\debug\scribblenot.exe` with `SCRIBBLENOT_HEADLESS=1`
  - `cargo build` and `cargo run` currently hit a Windows file-lock on `target\debug\scribblenot.exe`; this is an environment/process issue, not a compile or test failure
- Scope note:
  - stopped at a stable checkpoint after milestone 1 and preserved the existing iced modal/editor work already present in the tree

### 2026-04-06T13:25:00-04:00
- Follow-up pass: closed the immediate user-visible gap around copy/export ergonomics
- Implemented:
  - wired `copy_requested` to the system clipboard in `src/main.rs` using `arboard`
  - end-of-note navigation now surfaces a status hint to press `c` to copy
  - removed dead Phase 2 app-state scaffolding (`show_window`, `clipboard_import`) instead of leaving misleading warnings
  - suppressed remaining test-only dead-code noise so `cargo check` is clean
- Verification:
  - `cargo check` passed with zero Rust warnings from project code
  - headless existing-binary launch still passed via `.\target\debug\scribblenot.exe` with `SCRIBBLENOT_HEADLESS=1`
  - full test suite still passed (`203 passed`)

### 2026-04-07
- User provided copied export from the live app
- Findings from copied output:
  - clipboard copied raw editable-document markers (`<!-- scribblenot:section ... -->`)
  - empty managed sections copied as empty clinical headings
  - preview placeholder lines such as `Communication: --` leaked into clipboard output
  - `tx_regions` block-select data included unrelated hierarchy lists from infection-control, objective findings, remedial exercises, and date header lists
- Implemented:
  - added `document::export_editable_document` to clean the editable document for clipboard output while keeping `editable_note` as the source of truth
  - clipboard copy now exports the cleaned document, not the raw editor buffer
  - `tx_regions` now declares `data_file: "tx_regions.yml"` in `data/sections.yml`
  - `AppData::load` now scopes `block_select` data from a section's `data_file` when present, overriding the broad merged hierarchy fallback
  - added regression tests for marker/empty-section/placeholder export cleanup and `tx_regions` data scoping
- Verification:
  - `cargo test export_editable_document_strips_markers_empty_sections_and_placeholders` passed
  - `cargo test app_data_load_scopes_tx_regions_block_select_to_tx_regions_file` passed
  - `cargo test` passed (`205 passed`)
  - `cargo check` passed

### 2026-04-07
- User provided a second copied export from the live app after the clipboard cleanup
- Findings from live use:
  - objective/remedial list-select sections were easy to skip because Enter with no selected options marked the section skipped
  - the right pane was still a single-line text input bound to the whole note, so it did not function as a visible multiline preview
- Implemented:
  - list-select Enter behavior now selects the highlighted option first when the section has entries but no selection
  - Enter still confirms and advances once at least one list-select option is selected
  - empty list-select sections retain the previous skip-on-confirm behavior
  - the right pane now renders a scrollable multiline note preview instead of a single-line note input
  - updated list-select helper text to describe the new Enter behavior
  - added an app regression test for first-Enter-selects, second-Enter-confirms behavior
- Verification:
  - `cargo test list_select_enter_selects_current_before_confirming_section` passed
  - `cargo test` passed (`206 passed`)
  - `cargo check` passed
- Scope note:
  - `cargo fmt --check` is not clean repo-wide due formatting drift in files beyond this focused patch; no repo-wide formatting sweep was performed

### 2026-04-07
- User reported additional live-preview and treatment-region friction:
  - the preview pane showed internal `<!-- scribblenot:section ... -->` markers
  - the preview stayed/reset at the top rather than tracking the current section
  - treatment-region block-select items started preselected, causing all regions to appear in the note unless manually removed
- Implemented:
  - preview pane now renders the cleaned export view instead of the raw editable-document buffer
  - preview scrollable now has a stable iced id and update actions request a scroll to the current section
  - block-select items now start unselected when `default:` is omitted
  - explicit `default: true` still preselects a block-select item when product data calls for it
  - added a regression test covering omitted block-select defaults as unselected
- Verification:
  - `cargo test block_select -- --nocapture` passed
  - `cargo check` passed
  - `cargo test` passed (`207 passed`)

### 2026-04-07
- Resumed Phase 1 realignment from the existing mission log and re-baselined before editing
- Findings:
  - `cargo check` and `cargo test` still passed at resume (`207 passed`)
  - preview scrolling still used the active wizard section when iced asked for a scroll target, even while the map cursor was highlighting a different section
  - scroll targeting also used the old raw preview renderer rather than the cleaned preview text shown in the right pane
- Implemented:
  - preview scroll targets now come from `document::export_editable_document(&editable_note)`, matching the cleaned preview the user sees
  - map-focused preview scrolling now follows `map_cursor` through `note_scroll`; wizard-focused scrolling still follows `current_idx`
  - hint-driven transitions into map focus now refresh `note_scroll`
  - added a regression test proving the preview scroll target follows the highlighted map row in the cleaned preview
  - marked old renderer scroll helpers as test-only/dead-code-tolerant so `cargo check` stays clean after the new preview-scroll path
- Verification:
  - `cargo test preview_scroll_tracks_map_cursor_in_clean_preview` passed
  - `cargo check` passed
  - `cargo test` passed (`208 passed`)
  - `SCRIBBLENOT_HEADLESS=1 cargo run` passed

### 2026-04-06T12:36:00-04:00
- Began milestone 1: document-model completion
- Confirmed implementation strategy: add marker-backed editable document sections, validate marker pairs plus canonical headings, and sync structured commits into marker-owned ranges

### 2026-04-06T13:05:00-04:00
- Implemented marker-backed editable document generation in `src/note.rs` and switched `build_initial_document` to use it
- Restored and expanded `src/document.rs` with section marker specs, structure validation, and managed-range replacement helpers
- Wired structured section commits in `src/app.rs` to sync back into `editable_note`
- Wired direct editor edits to recompute structure validity immediately via `set_editable_note`
- Upgraded the editor warning to show the current structure problem instead of a stale generic message
- Added two app tests for editable-note reconciliation and warning refresh

### 2026-04-06T13:13:00-04:00
- Implemented a contained milestone-2 improvement: modal search is now a real iced `text_input` with clickable result rows
- Verification:
  - `cargo test` passed: 203 tests
  - `cargo build --tests` passed
  - `cargo run` headless re-check was blocked by a Windows file lock on `target/debug/scribblenot.exe` from an already-running `scribblenot` process
