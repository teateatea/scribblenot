## Task
#46 - Update all call sites outside block_select.rs to match the renamed API

## Context
ST1 renamed the block_select API fields from massage-specific names (region_cursor, technique_cursor, technique_selected) to neutral names (group_cursor, item_cursor, item_selected). ST2 verifies that all call sites in app.rs, ui.rs, and note.rs already use the new names, and that the full test suite passes.

## Approach
Verification-only pass: grep each file for old names to confirm absence, grep for new names to confirm presence, then run cargo test to confirm no regressions.

## Critical Files
- `src/app.rs` - uses groups, group_cursor, item_cursor, item_selected, toggle_item, has_selection, in_items, exit_items, enter_group
- `src/ui.rs` - uses groups, group_cursor, item_cursor, item_selected, has_selection, in_items, current_group_idx
- `src/note.rs` - uses item_selected via region_state loop, render_block_select

## Reuse
No new code. Verification uses cargo test (existing test suite, 111 tests).

## Steps
1. Confirm old names are absent in app.rs, ui.rs, note.rs:
   - `grep -n "region_cursor\|technique_cursor\|technique_selected" src/app.rs src/ui.rs src/note.rs`
   - Expected: no output.
2. Confirm new names are present in app.rs, ui.rs, note.rs:
   - `grep -n "group_cursor\|item_cursor\|item_selected\|toggle_item\|has_selection" src/app.rs src/ui.rs src/note.rs`
   - Expected: hits in all three files using neutral names only.
3. Run the full test suite:
   - `cargo test`
   - Expected: all tests pass (baseline: 111 passed, 0 failed).

## Verification

### Manual tests
None - this sub-task is verification only.

### Automated tests
- `cargo test` - must report 0 failures. All 111 existing tests serve as the regression gate.

## Changelog

### Review – 2026-04-02
- #1 (nit): Expanded Critical Files descriptions to include all neutral-name symbols actually present in each file (in_items, exit_items, enter_group for app.rs; in_items, current_group_idx for ui.rs).
