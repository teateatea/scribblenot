## Task

#45 - Refactor data format to flat, type-tagged YML blocks with ID-based cross-references

## Context

Sub-task 3 of task #45 requires unit tests for the new `load_data_dir` loader covering five cases: (a) happy-path round-trip with a multi-file fixture, (b) missing ID reference produces error, (c) duplicate ID+type produces error, (d) circular reference produces error, and (e) hybrid inline+cross-file ID-reference block resolves correctly.

Sub-task 2 wrote 9 of these tests (cases a-d plus several positive-path variants). This sub-task added the 10th test covering case (e). All 10 tests were verified passing before this plan was written.

## Approach

No new code is needed. The 10 tests are already implemented in the `data::tests` module and pass under `cargo test`. The plan documents their locations and the coverage each provides.

## Critical Files

- `src/data.rs` lines 1007-1235 - all 10 `load_data_dir_*` tests live in the `data::tests` module

## Reuse

- `make_test_dir` / `cleanup_test_dir` / `write_yml` - helpers at lines 1022-1039 used by all 10 tests to create and tear down temp directories

## Steps

1. Verify the 10 tests exist and pass.
   - Run: `cargo test load_data_dir`
   - Expected output: `10 passed; 0 failed`

2. Confirm coverage mapping:
   - (a) happy-path multi-file: `load_data_dir_returns_app_data_for_valid_directory` (line 1043) and `load_data_dir_merges_blocks_from_multiple_yml_files` (line 1061)
   - (b) missing ID reference: `load_data_dir_errors_on_missing_child_id_reference` (line 1126)
   - (c) duplicate ID+type: `load_data_dir_errors_on_duplicate_id_and_type` (line 1072) and `load_data_dir_errors_on_duplicate_id_and_type_across_files` (line 1094); plus positive-path `load_data_dir_allows_same_id_different_type` (line 1108)
   - (d) circular reference: `load_data_dir_errors_on_direct_cycle` (line 1148), `load_data_dir_errors_on_indirect_cycle` (line 1165), and `load_data_dir_accepts_acyclic_tree` (line 1182)
   - (e) hybrid inline+cross-file: `load_data_dir_hybrid_inline_and_cross_file_id_reference_resolves_correctly` (line 1211)

3. No gaps found. No additional tests needed.

## Verification

### Manual tests

None - all verification is automated.

### Automated tests

Run `cargo test load_data_dir` from the project root. All 10 tests must report `ok`.

- Status: PASSING (10/10 at time of plan creation)

## Progress

- Step 1: Ran `cargo test load_data_dir` -- 10 passed; 0 failed
- Step 2: Coverage mapping confirmed -- all 10 tests present at documented line numbers
- Step 3: No gaps found, no additional tests needed

## Implementation
Complete -- 2026-04-01
