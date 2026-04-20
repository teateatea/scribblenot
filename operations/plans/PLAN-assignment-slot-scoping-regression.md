# Plan: Regression Coverage for Per-Slot Assignment Scoping

**Date:** 2026-04-19
**Status:** Proposed
**Supersedes:** PLAN-assignment-slot-scoping.md

## Purpose

Add regression tests for per-slot assignment scoping. The structural work from #31
is already implemented; this plan covers only what remains.

## Current State

The collision hazard described in PLAN-assignment-slot-scoping.md has been addressed
by the following code that already exists:

- `confirmed_value_assignments(value, cfg)` in `src/modal.rs:2343` - derives the
  assignments produced by a confirmed `HeaderFieldValue` without touching the global map
- `merged_slot_assigned_values(confirmed, cfg, global)` in `src/sections/multi_field.rs:64` -
  builds a slot-local lookup by cloning the global map and extending it with the confirmed
  value's own assignments (slot wins on collision)
- `render_note_line_for_confirmed_slot` and `render_multifield` in `src/note.rs` - note
  rendering iterates per slot and calls `merged_slot_assigned_values` for each entry
- `resolve_multifield_value_for_confirmed_slot` and `resolve_field_label_for_confirmed_slot`
  in `src/sections/multi_field.rs` - UI display and label resolution use the same pattern
- `src/ui/mod.rs:598-610` - the wizard renders each repeat slot individually via
  `std::slice::from_ref(value)`, so per-slot display is already correct
- `AssignmentSourceKey { section_idx, field_idx, value_idx }` and
  `assigned_contributions: BTreeMap` in `src/app.rs` - track per-slot contributions
  for in-flight modal helpers (this is intentional and should remain)

Modal reopen (the old plan's Step 4) is already correct: `SearchModal::new_field`
receives the current confirmed `HeaderFieldValue`, which for list fields contains a
`ListState` with stored `item_ids` and `values`. The cursor restores from those, and
any assignments re-derive from the confirmed value itself, not from the global cache.

## Remaining Work

### Primary: regression tests (old Step 5)

There are no tests that exercise assignment collision or per-slot isolation directly.
These tests should live in `src/note.rs` or `src/sections/multi_field.rs` alongside
existing render tests.

**Case A - same-field repeat collision**

Two repeat slots on one field both assign the same `format_list` target but with
different item selections. Assert that each slot's note line contains only its own
assigned value, not the other slot's.

**Case B - cross-field collision**

Two different fields in one section both assign the same `format_list` target.
Assert that each field's note line contains its own assigned value after both are
confirmed, even though the global `assigned_values` map can only hold one.

**Case C - edit one slot, assert neighbors unchanged**

Confirm slot 0, then confirm slot 1. Edit slot 0 (replace its value). Assert slot
1's rendered note output is unchanged.

**Case D - modal reopen restores provenance**

Open a modal for a field that uses `assigns`. Select item A (which assigns `list_id ->
"foo"`). Confirm. Reopen the same slot. Assert the modal cursor starts on item A, not
on whatever the global map's current value would suggest.

### Minor: status-color uses global (optional)

`src/ui/mod.rs:534` calls `resolve_field_values` with global `assigned_values` to
determine the overall field status color (Complete/Partial/Empty). This is display text
only - it does not affect note output or document sync. Fix only if it produces a
visible incorrect color in practice.

## Implementation Steps

1. Add Case A test: construct a `HeaderFieldConfig` with `max_entries: Some(2)` and a
   list that has an `assigns` target. Build two `HeaderFieldValue::ListState` entries
   with different item selections. Call `render_note_line_for_confirmed_slot` for each
   with an empty global `assigned_values`. Assert each renders its own assignment output.

2. Add Case B test: construct two `HeaderFieldConfig` entries that both assign the same
   `format_list` id with different items. Build one confirmed value per field. Populate
   `assigned_values` as the global map would (one value wins). Call
   `render_note_line_for_confirmed_slot` for each field's slot. Assert each renders its
   own assignment, overriding the global.

3. Add Case C test: use the App integration test harness (see existing tests in
   `src/app.rs`). Confirm slot 0, confirm slot 1, replace slot 0's value. Assert slot
   1's note output is identical before and after the slot 0 edit.

4. Add Case D test: open a modal for a field that has a list with `assigns`, advance
   to select an item, confirm, then call `open_modal_for_current_field` to reopen.
   Assert `modal.field_flow.list_cursor` corresponds to the previously selected item.

5. Run `cargo test` and confirm all four new tests pass alongside the existing suite.

## Validation

### Automated

Run `cargo test` - all existing tests must still pass and the four new cases must pass.

### Manual

1. Open a section with a time/style field that uses `assigns`. Confirm the first slot
   with one value. Confirm a second slot (repeat) with a different value. Read the note
   and editable document and verify each slot line shows its own assigned value.
2. Edit the first slot to a new value. Verify the second slot's note line is unchanged.
3. Reopen the first slot's modal and verify the cursor starts on the item last confirmed
   for that slot.

## Files Likely Touched

- `src/note.rs` - Cases A and B tests
- `src/sections/multi_field.rs` - Cases A and B unit tests (alternative location)
- `src/app.rs` - Cases C and D integration tests

No production code changes are expected. If a test fails and reveals a real bug, that
becomes a new scoped fix, not scope for this plan.
