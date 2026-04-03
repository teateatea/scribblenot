## Task
#46 - Neutralise block_select struct and key names so they aren't tied to treatment-region vocabulary

## Context
Sub-task 3 of task #46 required renaming `region_data` to `block_select_data` in `src/app.rs` and renaming the top-level `regions:` key and per-region `techniques:` key to `entries:` in `data/tx_regions.yml`. The Test Writer confirmed these changes are fully implemented by ST1. This plan is verification-only: confirm all renames are in place and that `cargo test` passes.

## Approach
Read the current state of the two affected files, confirm each expected string is present (or absent), then run `cargo test` to confirm the data file loads and block_select renders identically. No code changes are required.

## Critical Files
- `src/app.rs` - already uses `block_select_data` (lines 151, 1431, 1463); no `region_data` references remain
- `src/data.rs` - defines `AppData.block_select_data: HashMap<String, Vec<BlockSelectEntry>>` (line 203); test `app_data_has_block_select_data_not_region_data` at line 438 guards against regression
- `data/tx_regions.yml` - top-level key is `entries:` (line 1); every per-region sub-list key is `entries:` (lines 5, 25, 39, etc.); no `regions:` or `techniques:` keys present

## Reuse
- Existing test suite (`cargo test`) covers compilation and data-load correctness
- `src/data.rs` test `app_data_has_block_select_data_not_region_data` specifically asserts the field rename is complete

## Steps
1. Confirm `src/app.rs` contains no occurrences of `region_data`, `RegionConfig`, or `TechniqueConfig` as live identifiers (comments in `data.rs` referencing these names as historical notes are acceptable).
2. Confirm `data/tx_regions.yml` top-level key is `entries:` and every nested technique list key is also `entries:`.
3. Run `cargo test` and verify all tests pass with no compilation errors.
4. If all checks pass, mark the sub-task verified. If any check fails, file a new task describing the discrepancy rather than modifying this plan.

## Verification
### Manual tests
- Open `src/app.rs` and search for `region_data` - no live usage should appear.
- Open `data/tx_regions.yml` and confirm the file starts with `entries:` and contains no `regions:` or `techniques:` keys.

### Automated tests
- `cargo test` - all tests must compile and pass; the sentinel test `app_data_has_block_select_data_not_region_data` in `src/data.rs` will fail to compile if the field rename is incomplete.

### Doc checks
`data/tx_regions.yml | contains | entries:`
`src/app.rs | contains | block_select_data`
`data/tx_regions.yml | missing | regions:`
`data/tx_regions.yml | missing | techniques:`

## Status: COMPLETED
