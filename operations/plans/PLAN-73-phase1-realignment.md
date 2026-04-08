## Task

#73 Phase 1 Recovery - Realign the editable document model and iced port with the approved outcome

## Status

Draft implementation plan.

This file exists because Mission 14 was marked complete, but the current implementation still falls short of the approved Phase 1 contract in [APPROVED-73-1-document-model-and-iced-port.md](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/operations/plans/APPROVED-73-1-document-model-and-iced-port.md).

## Why This Plan Exists

The current codebase has meaningful Phase 1 scaffolding:

- `iced` is wired in
- `AppKey` migration exists
- `editable_note` exists
- `src/document.rs` exists
- the app currently builds, tests, and starts in headless mode on the real data set

But those pieces are not yet connected in the way the approved plan required.

As of 2026-04-06, the current baseline is:

- `cargo build` passes
- `cargo test` passes
- `SCRIBBLENOT_HEADLESS=1 cargo run` passes

So the remaining gap is not "make it start." The real gap is "make the editable document architecture actually work as the user-visible source of truth."

## Problem Summary

The approved Phase 1 contract said:

- the editable markdown note is the user-visible source of truth
- structured actions may update only their own anchored section inside that document
- top-level headings are not enough; safe replacement requires stable per-section anchors
- manual edits inside section bodies must be preserved
- broken anchors must surface as a repair problem, not cause silent overwrite

The current implementation does not meet that contract yet:

- `editable_note` is initialized once, but structured actions do not reconcile changes back into it
- `find_section_bounds` and `replace_section_body` in `src/document.rs` are unused
- document validation only checks canonical `##` headings, not per-section anchors
- manual editor changes do not recompute `note_headings_valid`
- the iced wizard and modal are still largely placeholder/text-status UI instead of the intended interactive replacement
- `show_window` and `clipboard_import` are present as state but not wired into behavior

## Goals

Bring the current Phase 1 implementation into line with the approved plan by completing five missing behaviors:

1. define and validate stable per-section editable-document anchors
2. make structured actions update only their own section inside `editable_note`
3. preserve manual text edits outside the machine-managed replacement region
4. recompute document validity after direct editor changes
5. raise the iced UI from shell/scaffolding to a usable structured-input replacement

## Non-Goals

Do not expand scope into later work:

- no tray integration in this plan
- no global hotkeys in this plan
- no clipboard import flow in this plan
- no always-on chord capture in this plan
- no bidirectional parser from edited markdown back into all structured state unless explicitly needed for a narrow Phase 1 behavior

## Key Decisions

### 1. Treat the approved Phase 1 document as the source of truth

This recovery plan does not redefine the phase. It only turns the approved outcome into concrete execution steps against the current codebase.

### 2. Keep two anchor levels

The approved plan was correct: top-level canonical headings are necessary but insufficient.

The document model must support:

- top-level anchors: the canonical `##` headings
- section-level anchors: one stable anchor per independently updatable runtime section

### 3. Separate machine-managed content from user-edited content inside a section

Whole-section overwrite is not safe if the user can edit the note freely.

For any section that structured actions can update after startup, the document format needs a stable machine-owned block inside that section anchor. Structured updates rewrite only that machine-owned block. User text outside that block remains untouched.

### 4. Prefer data-backed anchor metadata over new hard-coded matches

`data/sections.yml` already carries the metadata that should drive most of this:

- `heading_label`
- `heading_search_text`
- `note_render_slot`
- `is_intake`

Use that metadata as the basis for section anchor identity whenever possible instead of adding more hand-maintained `cfg.id` branching.

## Proposed Document Contract

Each runtime-editable section must render with:

1. a stable visible section anchor
2. a machine-managed body region inside that section
3. optional user-editable text before or after that machine-managed region

Recommended shape:

```markdown
#### TREATMENT MODIFICATIONS & PREFERENCES
<!-- scribblenot:section id=tx_mods:start -->
Pressure: Light
Challenge: Moderate
<!-- scribblenot:section id=tx_mods:end -->

User free edits stay here.
```

Notes:

- the visible heading remains part of the user document
- the HTML-comment markers provide a stable replacement boundary without cluttering the displayed note
- replacement uses the marker pair, not blind "replace everything until next heading"
- if markers are missing, duplicated, or mismatched, the section is invalid for targeted overwrite

This is stricter than the current `document.rs` helpers, but it is the smallest design that satisfies the approved "safe targeted replacement plus preserve manual edits" requirement.

## Critical Files

- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/document.rs`
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/app.rs`
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/main.rs`
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/note.rs`
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/ui.rs`
- `C:/Users/solar/Documents/Claude Projects/scribblenot/data/sections.yml`

## Implementation Plan

### Step 1. Re-baseline Mission 14 acceptance against the current checkout

Before changing code, create a short implementation note or task checklist that records the current baseline:

- `cargo build` passes
- `cargo test` passes
- `SCRIBBLENOT_HEADLESS=1 cargo run` passes

Purpose:

- avoid re-solving the already-fixed startup issue
- focus the recovery on the remaining contract gaps
- prevent future reviewers from mixing stale review evidence with current code state

### Step 2. Expand `document.rs` from heading utilities into a real section-anchor module

The current helpers are too shallow. They only understand top-level headings and whole-body replacement.

Add explicit document helpers for section-managed replacement:

- `parse_document_headings(...)`
- `validate_canonical_headings(...)`
- `validate_section_anchors(...)`
- `find_managed_section_bounds(...)`
- `replace_managed_section_body(...)`
- `repair_document_structure(...)`

Recommended supporting types:

```rust
pub struct SectionAnchorSpec {
    pub section_id: String,
    pub heading_text: String,
    pub marker_start: String,
    pub marker_end: String,
}
```

Validation rules:

- canonical `##` headings must all exist
- each editable section must have exactly one matching visible heading
- each editable section must have exactly one start marker and one end marker
- start marker must appear after the section heading
- end marker must appear after the start marker
- duplicate section markers or duplicated visible anchors invalidate the document

Repair behavior:

- restore missing canonical headings
- restore missing section anchors and empty managed blocks when feasible
- preserve unrelated user text outside damaged sections
- if safe repair is not possible, prefer "invalid, user must repair" over silent mutation

### Step 3. Make section-anchor generation data-driven

Create a helper that derives editable anchor specs from loaded `SectionConfig` values.

Use existing metadata in this order:

- `heading_label` when a section already has a stable rendered heading
- known top-level-heading sections via `note_render_slot`
- `cfg.name` fallback only where no explicit metadata exists and the rendered heading is already stable

Do not scatter new ad hoc heading literals through `app.rs` or `ui.rs`.

Expected result:

- one authoritative mapping from runtime section to visible heading text
- document validation and targeted replacement use the same mapping as rendering

### Step 4. Add section-level render helpers in `note.rs`

`render_note(...)` should remain for full initial document generation, but it is not the right primitive for structured section reconciliation.

Add section-focused rendering helpers such as:

- `render_section_body(...)`
- `render_section_heading(...)`
- `build_initial_document_with_anchors(...)`

Requirements:

- section-level rendering must reuse the same formatting logic as the full note
- output for a single section must match the content that full-note rendering would place inside that section's managed block
- no second formatting system should emerge for "document sync"

### Step 5. Define a single structured-to-document reconciliation path in `app.rs`

Add one app method responsible for syncing structured state into `editable_note`:

```rust
fn sync_current_section_into_editable_note(&mut self)
```

Responsibilities:

- derive the current section's anchor spec
- validate the current document before mutation
- render the current section's managed body
- replace only the managed body between markers
- recompute `note_headings_valid` and section-anchor validity after replacement
- set a clear status message if sync is blocked by invalid document structure

Call this method from the points where a section's structured state becomes committed or visibly updated:

- section confirm actions
- header field confirmations
- free-text entry confirmations
- list-select completion
- block-select completion
- checklist completion

Do not silently regenerate the whole note on each action.

### Step 6. Recompute document validity on direct editor changes

Update the `EditableNoteChanged` path in `src/main.rs`.

Current behavior:

- only updates `state.inner.editable_note`

Required behavior:

- update `editable_note`
- recompute canonical-heading validity
- recompute section-anchor validity
- update the app warning state immediately

This is the minimum needed to satisfy the "broken anchors become a repair problem" requirement.

### Step 7. Surface document validity clearly in the UI

Upgrade the editor warning in `src/ui.rs`.

Instead of a single stale boolean warning, show:

- no warning when the document is valid
- a clear invalid-structure warning when canonical headings are damaged
- a clear section-anchor warning when targeted overwrite is blocked

The warning text should explain the consequence plainly:

- structured edits are temporarily blocked until the document is repaired

If a repair action is implemented in this phase, expose it in the UI. If not, make the warning explicit and non-misleading.

### Step 8. Replace placeholder modal rendering with real iced interaction

The current modal view is mostly text output. That falls short of the approved port.

Minimum acceptable Phase 1 implementation:

- actual `text_input` bound to the modal query
- actual selectable result rows
- visible highlighted current row
- keyboard behavior still routed through existing `app.rs` logic

The goal is not visual polish. The goal is that the search/composite workflow is genuinely usable in iced rather than represented by debug-like text.

### Step 9. Raise the wizard pane from status inspector to usable structured controls

The current wizard pane is informative, but still feels like a state dump.

For each section type, implement an iced interaction surface that is visibly tied to the current state:

- `multi_field`: current field, values, and entry/confirm flow
- `free_text`: editable buffer and entry list
- `list_select`: selectable/toggleable entries plus add-entry flow
- `block_select`: group selection and item selection view
- `checklist`: toggleable checklist rows

Keyboard-first operation should remain supported, but the pane should no longer read like a debug viewer.

### Step 10. Decide Phase 1 treatment of `show_window` and `clipboard_import`

Two valid options:

Option A: wire both fields into minimal live behavior now.

Pros:

- removes dead fields
- aligns the state shape with intended desktop behavior

Cons:

- starts pulling in Phase 2 behavior

Option B: remove or gate them until Phase 2.

Pros:

- cleaner Phase 1 boundary
- avoids carrying dead scaffolding

Cons:

- minor rework later when Phase 2 starts

Recommendation: Option B.

Short explanation:

They are Phase 2 concerns in practice. Leaving them as unread dead fields makes the code look more complete than it is.

### Step 11. Add tests for the actual document contract

The existing `document.rs` tests are too weak because they only cover top-level heading replacement.

Add tests for:

- valid document with both canonical and section markers
- missing canonical heading invalidates the document
- missing section start marker invalidates the document
- duplicate section start or end markers invalidate the document
- `find_managed_section_bounds(...)` returns only the marker-owned body
- replacing one section body leaves other sections untouched
- replacing one section body preserves user text outside the marker block
- repair restores missing anchors without deleting unrelated content when feasible

### Step 12. Add app-level integration tests for editable-note reconciliation

Add tests that prove the architecture, not just the helpers:

- confirming a structured change updates only the current section in `editable_note`
- manual edits in another section remain unchanged after structured sync
- invalid document structure blocks targeted overwrite
- direct editor edits recompute validity immediately

This is the gap the current test suite missed.

### Step 13. Run a manual verification sweep against the approved Phase 1 exit criteria

Manual checks:

- app launches under iced without terminal setup
- editable note shows initial document with stable anchors
- manual edits in one section persist after structured edits in another section
- deleting a required section marker or heading raises an invalid-structure warning
- structured actions stop mutating the document when anchors are invalid
- search modal is usable with keyboard flow
- Shift+Enter still performs the intended structured action

### Step 14. Close Phase 1 with an explicit acceptance note

When the work is done, record:

- which approved Phase 1 requirements are now satisfied
- which were intentionally deferred
- exact verification date and commands used

Do not rely on a generic "mission complete" note without mapping back to the approved plan.

## Verification

### Automated

- `cargo build`
- `cargo test`
- targeted document tests for section markers and repair
- app-level tests for structured reconciliation into `editable_note`

### Manual

- launch the app normally and confirm the iced shell loads
- confirm the editor shows the initial anchored document
- type manual text into one section
- use structured input in another section
- confirm only the targeted machine-managed region updates
- intentionally damage one section marker
- confirm a warning appears and structured overwrite is blocked
- repair the marker and confirm updates work again

## Exit Criteria

Phase 1 is realigned only when all of the following are true:

- the app runs under iced
- `editable_note` is the active user-visible source of truth
- structured actions update only their own anchored managed section
- manual edits outside the managed region survive structured updates
- broken anchors surface as an explicit repair problem
- direct editor edits immediately refresh document-validity state
- the modal and wizard panes are usable iced interfaces rather than placeholder status renderers

## Risks To Watch

- duplicating note-formatting logic between full-note render and section render
- choosing unstable visible headings as anchors
- silently overwriting user edits by replacing whole section bodies
- mixing Phase 2 tray behavior into this recovery and inflating scope
- keeping tests too local to helpers instead of proving end-to-end reconciliation behavior

## Recommendation

Implement this recovery in two small internal milestones:

1. document-model completion
2. iced-interaction completion

Reason:

The document-model work is the correctness-critical part. Once the replacement contract is safe, the UI work can iterate without risking silent data loss or false completeness.
