## Task
#70 - Implement canonical 6-level YAML data hierarchy

## Context
Sub-task 5 of task #70 requires verifying that the full migration is correct: all five groups load without panic, no `map_label:` keys survive in `data/*.yml`, note output is byte-for-byte identical to the pre-migration baseline, and all MISSION-13-TESTS.md criteria are met. Most criteria can be verified automatically via `cargo test` and grep; only the byte-for-byte note output check requires a live TUI session.

## Approach
Run all automated checks first (cargo test, cargo build, grep) to confirm 9 of the 11 MISSION-13-TESTS.md criteria automatically, then document the one manual criterion that requires a TUI session (criterion j). Criterion 11 (non-dependency on the list-select add-entry persistence issue) is satisfied structurally: none of the verification steps exercise the list-select add-entry path. All automated criteria are already covered by existing tests in `src/data.rs` (`hierarchy_runtime_tests`, `tx_regions_default_tests`, `real_data_dir_loads_as_hierarchy_format`) plus grep commands; no new tests need to be written.

## Critical Files
- `/c/scribble/src/data.rs` - contains all test modules: `hierarchy_runtime_tests` (lines 2607-2693), `tx_regions_default_tests` (lines 431-463), `real_data_dir_loads_as_hierarchy_format` (lines 1225-1242)
- `/c/scribble/data/` - all YAML data files to check for `map_label:` keys
- `/c/scribble/src/` - check for absence of `flat_file.rs` and `mod flat_file`

## Reuse
- Existing test `hierarchy_to_runtime_produces_groups_in_correct_order` (data.rs:2623) covers criterion (b)
- Existing test `hierarchy_to_runtime_objective_section_has_date_prefix_true` (data.rs:2670) covers criterion (c)
- Existing test `lower_back_prone_fascial_l4l5_starts_unselected` (data.rs:440) covers criterion (d)
- Existing test `hierarchy_to_runtime_produces_block_select_data_with_tx_regions_key` (data.rs:2645) covers criterion (e)
- Existing test `real_data_dir_loads_as_hierarchy_format` (data.rs:1225) covers criterion (h)

## Steps
1. Run `cargo test` and confirm all 179 tests pass (criterion a):
   ```
   PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" \
   /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml" 2>&1 | tail -5
   ```
   Expected: `test result: ok. N passed; 0 failed; 0 ignored`

2. Confirm `hierarchy_to_runtime_produces_groups_in_correct_order` passes (criterion b):
   ```
   cargo test --manifest-path "/c/scribble/Cargo.toml" hierarchy_to_runtime_produces_groups_in_correct_order 2>&1
   ```
   Expected: test passes, groups vec equals `["intake", "subjective", "treatment", "objective", "post_tx"]`

3. Confirm `hierarchy_to_runtime_objective_section_has_date_prefix_true` passes (criterion c):
   ```
   cargo test --manifest-path "/c/scribble/Cargo.toml" hierarchy_to_runtime_objective_section_has_date_prefix_true 2>&1
   ```
   Expected: test passes, `objective_section.date_prefix == Some(true)`

4. Confirm `lower_back_prone_fascial_l4l5_starts_unselected` passes (criterion d):
   ```
   cargo test --manifest-path "/c/scribble/Cargo.toml" lower_back_prone_fascial_l4l5_starts_unselected 2>&1
   ```
   Expected: test passes, `fascial_l4l5.default == Some(false)`

5. Confirm `hierarchy_to_runtime_produces_block_select_data_with_tx_regions_key` passes (criterion e):
   ```
   cargo test --manifest-path "/c/scribble/Cargo.toml" hierarchy_to_runtime_produces_block_select_data_with_tx_regions_key 2>&1
   ```
   Expected: test passes, `block_select_data["tx_regions"]` is non-empty

6. Verify `flat_file.rs` is absent from `src/` (criterion f):
   ```
   ls /c/scribble/src/flat_file.rs 2>&1
   grep "mod flat_file" /c/scribble/src/main.rs
   ```
   Expected: file not found; `mod flat_file` line absent from `main.rs`

7. Verify no `map_label:` keys in `data/*.yml` (criterion g):
   ```
   grep -r "map_label:" /c/scribble/data/
   ```
   Expected: no output (zero matches)

8. Confirm all data YAML files parse as HierarchyFile without error (criterion h):
   ```
   cargo test --manifest-path "/c/scribble/Cargo.toml" real_data_dir_loads_as_hierarchy_format 2>&1
   ```
   Expected: test passes

9. Run `cargo build` and confirm no warnings (criterion i):
   ```
   PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" \
   /c/Users/solar/.cargo/bin/cargo.exe build --manifest-path "/c/scribble/Cargo.toml" 2>&1
   ```
   Expected: `Finished` line, no `warning:` lines from new code

## Verification

### Manual tests
- Launch the TUI against the real data directory, navigate through all five groups (intake, subjective, treatment, objective, post_tx), and confirm no panic occurs.
- Fill out a representative note (one or more sections per group) and capture the rendered output. Compare it byte-for-byte against a pre-migration baseline note captured before the Task #70 changes were applied. They must match exactly (criterion j).

### Automated tests
All automated criteria are verified by the nine steps above using existing cargo tests plus grep. No new tests are needed. The full test suite (`cargo test`) provides regression coverage for all automated criteria simultaneously.

## Changelog

### Review - 2026-04-03
- #1: Corrected criterion count in Approach from "9 of the 10" to "9 of the 11" and added explicit acknowledgment that criterion 11 (non-dependency on list-select add-entry issue) is satisfied structurally
- #2: Corrected test count in Step 1 from "179+" to exact "179" (verified by running cargo test)
- #3: Removed spurious `-r` flag from single-file grep in Step 6

## Progress
- Step 1: cargo test - 179 passed, 0 failed, 0 ignored (criterion a verified)
- Step 2: hierarchy_to_runtime_produces_groups_in_correct_order - passed (criterion b verified)
- Step 3: hierarchy_to_runtime_objective_section_has_date_prefix_true - passed (criterion c verified)
- Step 4: lower_back_prone_fascial_l4l5_starts_unselected - passed (criterion d verified)
- Step 5: hierarchy_to_runtime_produces_block_select_data_with_tx_regions_key - passed (criterion e verified)
- Step 6: flat_file.rs absent, no mod flat_file in main.rs (criterion f verified)
- Step 7: grep map_label: in data/ returned zero matches (criterion g verified)
- Step 8: real_data_dir_loads_as_hierarchy_format - passed (criterion h verified)
- Step 9: cargo build succeeded; 1 pre-existing warning (default_selected unused) - not from new code (criterion i verified)

## Implementation
Complete - 2026-04-03
