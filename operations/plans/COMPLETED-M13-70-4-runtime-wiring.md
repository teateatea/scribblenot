## Task

#70 - Implement canonical 6-level YAML data hierarchy (sub-task 4: runtime wiring and cleanup)

## Context

Sub-tasks 1-3 introduced hierarchy structs, load_hierarchy_dir, and migrated all data/*.yml files to HierarchyFile format. Sub-task 4 is the final wiring step: expose hierarchy_to_runtime as a public function, change AppData.block_select_data to hold Vec<HierarchyList>, update BlockSelectState to accept HierarchyList, wire AppData::load to use the new loader, rewrite the ~20 load_data_dir tests so they call load_hierarchy_dir with hierarchy-format fixtures, and delete flat_file.rs and its mod declaration. All changes land in one commit.

## Approach

Add hierarchy_to_runtime as a standalone pub function in data.rs (extracted from the existing private hierarchy_to_app_data). Change AppData.block_select_data from HashMap<String, Vec<BlockSelectEntry>> to HashMap<String, Vec<HierarchyList>>. Update BlockSelectGroup::from_config and BlockSelectState::new in block_select.rs to accept HierarchyList. Update the block_select dispatch arm in app.rs to key on cfg.id instead of cfg.data_file. Rewrite AppData::load to call load_hierarchy_dir then hierarchy_to_runtime. Rewrite all load_data_dir tests to use load_hierarchy_dir with hierarchy-format YAML fixtures. Delete flat_file.rs and remove its mod declaration from main.rs.

## Critical Files

- `/c/scribble/src/data.rs` - add hierarchy_to_runtime (lines 896-964 region), change AppData.block_select_data field type (line 221), rewrite load_data_dir tests (lines 1637-1932, boilerplate_load_tests lines 1958-2064, tx_mods tests lines 2111-2350, section_metadata tests lines 2351-2559), remove load_data_dir function (lines 703-895), remove `use crate::flat_file::FlatFile;` import (line 8)
- `/c/scribble/src/sections/block_select.rs` - update from_config and new to accept HierarchyList instead of BlockSelectEntry (lines 1-61)
- `/c/scribble/src/app.rs` - update block_select dispatch to key on cfg.id (lines 147-155)
- `/c/scribble/src/main.rs` - remove `mod flat_file;` declaration (line 9)
- `/c/scribble/src/flat_file.rs` - delete entire file

## Reuse

- `hierarchy_to_app_data` (data.rs line 896): the group/section/boilerplate conversion logic is already written; hierarchy_to_runtime extracts that logic and also collects block_select_data from block_select sections
- `HierarchyList` struct (data.rs line 633): already defined; used directly as the Vec<HierarchyList> element type
- `load_hierarchy_dir` (data.rs line 966): already implemented; AppData::load calls it, then hierarchy_to_runtime
- `make_hier_test_dir` / `cleanup_hier_test_dir` / `write_hier_yml` helpers (data.rs load_hierarchy_dir test module): reuse in rewritten load_data_dir test bodies

## Steps

1. **Add hierarchy_to_runtime in data.rs.**

   Add a new public function after load_hierarchy_dir. Signature:

   ```
   pub fn hierarchy_to_runtime(hf: HierarchyFile) -> Result<(Vec<SectionGroup>, Vec<SectionConfig>, HashMap<String, String>, HashMap<String, Vec<HierarchyList>>), String>
   ```

   Implementation: replicate the group/section/field mapping from hierarchy_to_app_data, collect boilerplate_texts (HashMap<String, String>), and collect block_select_data from the merged top-level `HierarchyFile.lists` field. Specifically: after processing all sections, find every section whose `section_type == "block_select"` and insert `hf.lists.clone().unwrap_or_default()` into `block_select_data` keyed by that section's id. (The real data places block_select lists in `tx_regions.yml` as top-level `lists:` entries, not inline inside the section definition; `HierarchySection.lists` will be `None` for block_select sections in the current schema.) Return all four values.

   The existing test stubs in `hierarchy_runtime_tests` unpack only `(groups, block_select_data)` using a 2-tuple pattern. Update each test to unpack `(groups, _, _, block_select_data)` to match the 4-tuple signature. The three call sites to update are:

   ```
   // data.rs line ~3004 (ST70-4-TEST-1)
   - let (groups, _block_select_data) = hierarchy_to_runtime(hf)
   + let (groups, _, _, _block_select_data) = hierarchy_to_runtime(hf)

   // data.rs line ~3026 (ST70-4-TEST-2)
   - let (_groups, block_select_data) = hierarchy_to_runtime(hf)
   + let (_groups, _, _, block_select_data) = hierarchy_to_runtime(hf)

   // data.rs line ~3051 (ST70-4-TEST-3)
   - let (groups, _block_select_data) = hierarchy_to_runtime(hf)
   + let (groups, _, _, _block_select_data) = hierarchy_to_runtime(hf)
   ```

2. **Change AppData.block_select_data field type.**

   In data.rs, change:
   ```
   - pub block_select_data: HashMap<String, Vec<BlockSelectEntry>>,
   + pub block_select_data: HashMap<String, Vec<HierarchyList>>,
   ```

3. **Rewrite AppData::load to use load_hierarchy_dir + hierarchy_to_runtime.**

   Replace the body of AppData::load. Remove the call to load_data_dir and the flat_file-based list/checklist/block_select loading loops. New body:
   - Call `load_hierarchy_dir(&data_dir)?`
   - Call `hierarchy_to_runtime(hf)?` to get (groups, sections, boilerplate_texts, block_select_data)
   - For list_select and checklist sections that still use data_file: these are the only remaining data_file users. Since flat_file.rs is being deleted, load them using serde_yaml directly. The actual data files (`objective_findings.yml`, `remedial.yml`, `infection_control.yml`) are already in HierarchyFile format with a top-level `lists:` key (migrated in ST3). Parse each file as `HierarchyFile` using `serde_yaml::from_str::<HierarchyFile>(&content)`, then extract entries from `hf.lists.unwrap_or_default()`. For list_select, map each `HierarchyItem` in each list's `items` to `ListEntry { label: item.label.clone(), output: item.output.clone().unwrap_or_else(|| item.label.clone()) }` and insert the collected entries into `list_data` keyed by `data_file`. For checklist, collect only `item.label` strings and insert into `checklist_data`. Do not use `ListFile`, `FlatFile`, or `FlatBlock` anywhere in the new body. Note: `reload_list` and `append_list_entry` are pre-existing non-goal items (tracked as roadmap #2) and are not updated by this task.
   - Load keybindings.yml as before.
   - Construct AppData with the new block_select_data.

4. **Update block_select.rs: accept HierarchyList instead of BlockSelectEntry.**

   Change BlockSelectGroup::from_config to take &HierarchyList:
   ```
   - pub fn from_config(cfg: &BlockSelectEntry) -> Self {
   -     let item_selected = cfg.entries.iter().map(|e| e.default_selected()).collect();
   -     Self {
   -         label: cfg.label.clone(),
   -         header: cfg.header.clone(),
   -         entries: cfg.entries.clone(),
   -         item_selected,
   -     }
   + pub fn from_config(cfg: &HierarchyList) -> Self {
   +     let item_selected = cfg.items.iter().map(|e| e.default.unwrap_or(true)).collect();
   +     Self {
   +         label: cfg.label.clone().unwrap_or_default(),
   +         header: cfg.label.clone().unwrap_or_default(),
   +         entries: cfg.items.iter().map(|item| PartOption::Full {
   +             id: item.id.clone(),
   +             label: item.label.clone(),
   +             output: item.output.clone().unwrap_or_else(|| item.label.clone()),
   +             default: item.default.unwrap_or(true),
   +         }).collect(),
   +         item_selected,
   +     }
   ```

   Change BlockSelectState::new to take Vec<HierarchyList>:
   ```
   - pub fn new(regions: Vec<BlockSelectEntry>) -> Self {
   + pub fn new(regions: Vec<HierarchyList>) -> Self {
   ```

   Update imports at top of block_select.rs:
   ```
   - use crate::data::{BlockSelectEntry, PartOption};
   + use crate::data::{HierarchyList, PartOption};
   ```

   **Rewrite block_select.rs tests that use BlockSelectEntry.** The three test modules
   `tests_st3_default_selected`, `tests_t46_st1_rename`, and `tests_st2_region_state_entries_field`
   (lines 127-388) use `BlockSelectEntry` directly and will fail to compile once `BlockSelectEntry`
   is removed. Rewrite these tests to use `HierarchyList` / `HierarchyItem` fixtures instead.
   Each test that previously called `BlockSelectGroup::from_config(&BlockSelectEntry{...})`
   must call `BlockSelectGroup::from_config(&HierarchyList{...})`. Tests that call
   `BlockSelectState::new(vec![BlockSelectEntry{...}])` must call
   `BlockSelectState::new(vec![HierarchyList{...}])`. Preserve the behavioural assertions;
   only the fixture construction changes.

5. **Update block_select dispatch in app.rs.**

   Change the block_select arm in App::init_states to key on cfg.id instead of cfg.data_file:
   ```
   - "block_select" => {
   -     let regions = cfg
   -         .data_file
   -         .as_ref()
   -         .and_then(|f| data.block_select_data.get(f))
   -         .cloned()
   -         .unwrap_or_default();
   -     SectionState::BlockSelect(BlockSelectState::new(regions))
   - }
   + "block_select" => {
   +     let regions = data.block_select_data.get(&cfg.id)
   +         .cloned()
   +         .unwrap_or_default();
   +     SectionState::BlockSelect(BlockSelectState::new(regions))
   + }
   ```

6. **Rewrite all ~20 load_data_dir tests to use load_hierarchy_dir with hierarchy-format YAML fixtures.**

   Each test in the following modules must be rewritten:
   - `mod load_data_dir_tests` (lines 1637-1932): ~13 tests including valid_single, multi_file, dupe, missing_child, cycle, acyclic_tree, reconstruction pass, hybrid cross-file
   - `mod boilerplate_load_tests` (lines 1958-2064): 4 tests
   - `mod tx_mods_multi_field_tests` (lines 2111-2350): 8 tests that call `load_data_dir(&dir)` via the `load()` helper
   - `mod section_metadata_fields_tests` and `mod section_metadata_complete_tests` (lines 2351-2559): 8 tests that call `load_data_dir(&dir)` via the `load()` helper

   For the structural tests (valid_single, multi_file, dupe, etc.) that use temp dirs: rewrite fixture content from flat-block YAML to hierarchy-format YAML. For example:

   ```
   # Old flat format:
   blocks:
     - type: box
       id: root_box
     - type: section
       id: sec_a
     - type: field
       id: fld_a

   # New hierarchy format (minimal valid file with one template):
   template:
     groups: [grp_a]
   groups:
     - id: grp_a
       nav_label: Group A
       sections: [sec_a]
   sections:
     - id: sec_a
       nav_label: Section A
       section_type: free_text
   ```

   For tests that load the real data directory: change `load_data_dir(&dir)` to `load_hierarchy_dir(&dir)` then call `hierarchy_to_runtime(hf)` to get AppData-equivalent fields, and call `AppData::load(dir)` directly where the test needs the full AppData struct (for section_metadata, tx_mods tests etc.).

   The duplicate-id and missing-child tests must be rewritten with hierarchy-format fixtures that trigger the equivalent errors in load_hierarchy_dir (duplicate group id, duplicate section id, section referenced in template group but not defined, etc.).

   **Cycle tests cannot be translated.** The two cycle tests (`load_data_dir_errors_on_direct_cycle`, `load_data_dir_errors_on_indirect_cycle`) rely on flat-block `children:` back-references that have no equivalent in the hierarchy schema. `HierarchyGroup.sections` is a Vec<String> of section IDs, and `HierarchySection` has no `groups` or `sections` field, so a group->section->group loop is structurally impossible. The `HierarchyField.list_id` path also cannot form a cycle because `HierarchyList` carries no `list_id`. Delete both cycle tests rather than attempting to rewrite them. Add a short comment in their place explaining that cycle detection code is retained for future schema evolution but cannot be exercised with the current schema.

   The "allows same id different type" test maps to load_hierarchy_dir's per-type uniqueness: same id for a group and a section is allowed; write a fixture that has the same id string in both `groups:` and `sections:`.

7. **Remove load_data_dir and now-dead types from data.rs.**

   Delete the entire `pub fn load_data_dir` function (lines 703-895) and the helper functions it alone uses (block_type_tag, block_id, block_children, dfs inner function).

   Delete the private `hierarchy_to_app_data` function (lines 896-964); its logic was extracted into `hierarchy_to_runtime` and it is no longer called.

   Delete the `BlockSelectEntry` struct (line 138-143) and the `BlockSelectFile` struct (line 145-148); both become dead code once `AppData::load` no longer parses `block_select` sections via `data_file` and block_select.rs tests are rewritten.

   Delete the `mod rename_tests` test module (lines 426-470); all three tests in that module (`block_select_entry_exists_with_entries_field`, `block_select_file_exists_with_entries_field`, `app_data_has_block_select_data_not_region_data`) reference removed types or the old field type and will fail to compile. The rename they verified is now historical.

   Remove the `use crate::flat_file::FlatFile;` import at line 8.

   Remove references to `crate::flat_file` inside AppData::load (already replaced in step 3).

8. **Delete flat_file.rs and remove mod declaration.**

   - Delete `/c/scribble/src/flat_file.rs`
   - In `/c/scribble/src/main.rs` line 9, remove `mod flat_file;`

9. **Verify compilation and tests.**

   Run `cargo build` first to catch type errors. Then run `cargo test` to confirm all tests pass. Fix any remaining references to `BlockSelectEntry` or `FlatBlock`/`FlatFile` types.

## Verification

### Manual tests

- Run the application against the real data directory and confirm all five groups load without panic.
- Navigate to the tx_regions block_select screen and confirm region options appear as expected.
- Complete a note and verify the rendered output is byte-for-byte identical to the pre-migration baseline.

### Automated tests

- `cargo test` must pass with zero failures. All ~20 rewritten load_data_dir tests, plus the 3 new hierarchy_runtime_tests, must be green.
- The integration tests in hierarchy_runtime_tests cover: group order (intake, subjective, treatment, objective, post_tx), block_select_data key presence ("tx_regions"), and date_prefix propagation (objective_section = Some(true)).
- Regression: tx_mods tests and section_metadata tests pass through AppData::load using load_hierarchy_dir, confirming the shim preserves all metadata fields.

## Changelog

### Review - 2026-04-03
- #1 (blocking): Step 4 - added explicit instruction to rewrite the three block_select.rs test modules (tests_st3_default_selected, tests_t46_st1_rename, tests_st2_region_state_entries_field) that use BlockSelectEntry and will fail to compile after the signature change.
- #2 (blocking): Step 6 - clarified that the two cycle tests (load_data_dir_errors_on_direct_cycle, load_data_dir_errors_on_indirect_cycle) cannot be translated to hierarchy format because the hierarchy schema cannot express a cycle; added instruction to delete them with an explanatory comment.
- #3 (minor): Step 7 - expanded cleanup list to include: delete hierarchy_to_app_data (now dead code), delete BlockSelectEntry and BlockSelectFile structs, delete mod rename_tests (three tests reference removed types and will fail to compile).
- #4 (nit): Step 3 - removed self-contradictory mid-bullet note about FlatFile; consolidated into a single clear instruction to use ListFile for list_select/checklist deserialization. Removed redundant trailing Note paragraph that repeated the same content.

### Review - 2026-04-03 (R2)
- #5 (blocking): Step 1 - corrected the block_select_data collection strategy. The plan said to collect from HierarchySection.lists, but the tx_regions section in sections.yml has no inline lists field; its lists live as top-level lists: entries in tx_regions.yml (merged into HierarchyFile.lists). Updated Step 1 to instruct collecting HierarchyFile.lists for each block_select section, not HierarchySection.lists.
- #6 (minor): Step 1 - added explicit diff showing the three hierarchy_runtime_tests call sites (lines ~3004, ~3026, ~3051) that must be updated from 2-tuple to 4-tuple destructuring to match the new hierarchy_to_runtime return type.

## Progress

- Step 1: Added hierarchy_to_runtime pub fn in data.rs, updated 3 test call sites from 2-tuple to 4-tuple
- Step 2: Changed AppData.block_select_data field type from HashMap<String, Vec<BlockSelectEntry>> to HashMap<String, Vec<HierarchyList>>
- Step 3: Rewrote AppData::load to use load_hierarchy_dir + hierarchy_to_runtime, parse list_select/checklist via HierarchyFile
- Step 4: Updated block_select.rs: from_config takes &HierarchyList, new takes Vec<HierarchyList>, rewrote all 3 test modules
- Step 5: Updated block_select dispatch in app.rs to key on cfg.id instead of cfg.data_file
- Step 6: Rewrote all load_data_dir tests to hierarchy format, deleted cycle tests (structurally impossible), updated real_data_dir tests
- Step 7: Removed load_data_dir, hierarchy_to_app_data, BlockSelectEntry, BlockSelectFile, rename_tests, block_type_tag/block_id/block_children helpers, flat_file import
- Step 8: Deleted flat_file.rs, removed mod flat_file from main.rs
- Step 9: All 179 tests pass, also fixed note.rs test that referenced load_data_dir

## Implementation
Complete - 2026-04-03

### Review - 2026-04-03 (R3)
- #7 (blocking): Step 3 - corrected list_select/checklist deserialization. The plan instructed using `ListFile { entries: Vec<ListEntry> }`, but the actual data files (objective_findings.yml, remedial.yml, infection_control.yml) were migrated in ST3 to HierarchyFile format with a top-level `lists:` key. Updated Step 3 to parse each file as `HierarchyFile` and extract `ListEntry` / label strings from `hf.lists[*].items`. Also noted that `reload_list` and `append_list_entry` are pre-existing non-goal items (roadmap #2) and are not updated by this task.
