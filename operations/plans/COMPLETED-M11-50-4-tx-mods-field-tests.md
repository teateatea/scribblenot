**Task**: #50 Convert tx_mods section to multi_field with 5 categorized fields, removing tx_mods.yml

**Context**: Sub-task 4 adds tests verifying the specific structural properties of the 5 tx_mods fields that ST50-1 tests did not cover: (1) STOIC appears exactly twice in communication options (two distinct notes for different patient types), (2) pressure/challenge/mood/communication are single-select (repeat_limit: None), and (3) the 5 field IDs are exactly correct in order. All three tests were written and pass against the already-complete implementation.

**Approach**: Three regression-guard tests appended to `tx_mods_multi_field_tests` module in src/data.rs (after line 1888). No implementation changes needed.

**Critical Files**:
- `src/data.rs` lines 1890-1962 (ST50-4-TEST-1, ST50-4-TEST-2, ST50-4-TEST-3)

**Steps**:

1. Write three tests in `src/data.rs` inside `tx_mods_multi_field_tests`:
   - `communication_has_exactly_two_stoic_entries`
   - `single_select_fields_have_no_repeat_limit`
   - `tx_mods_field_ids_are_correct`

2. Run `cargo test --manifest-path "C:/Users/solar/Documents/Claude Projects/scribblenot/Cargo.toml"` to confirm all tests pass.

3. Commit `src/data.rs` with message:
   `Implement task #50 sub-task 50.4: add field structure regression tests for tx_mods`

**Verification**:

### Automated tests
- `cargo test communication_has_exactly_two_stoic_entries` -- must pass
- `cargo test single_select_fields_have_no_repeat_limit` -- must pass
- `cargo test tx_mods_field_ids_are_correct` -- must pass
- `cargo test` (full suite) -- zero regressions

## Progress

ST50.4 complete. All 3 tests written and passing. Full suite: 172 passed, 0 failed.

- `communication_has_exactly_two_stoic_entries`: PASS
- `single_select_fields_have_no_repeat_limit`: PASS
- `tx_mods_field_ids_are_correct`: PASS
