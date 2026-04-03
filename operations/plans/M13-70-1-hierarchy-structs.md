## Task

#70 - Implement canonical 6-level YAML data hierarchy

## Context

The existing loader uses FlatFile/FlatBlock types and a flat reference pool. MISSION-13 replaces this with a typed 6-level hierarchy (Template > Group > Section > Field > List > Item). Sub-task 1 establishes the Rust struct layer in src/data.rs so later sub-tasks can build the loader, shim, and migration on top of it. No runtime behavior changes in this sub-task.

## Approach

Add seven new structs (HierarchyItem, HierarchyList, HierarchyField, HierarchySection, HierarchyGroup, HierarchyTemplate, HierarchyFile) and one supporting struct (BoilerplateEntry) directly in src/data.rs, after the existing runtime structs. Add a TypeTag enum for (TypeTag, id) scoped uniqueness. Derive Serialize and Deserialize on all new types. The TDD test module hierarchy_struct_tests already exists in data.rs and defines the exact struct contract; the structs must satisfy those 10 tests. Key design points: HierarchyItem has required id+label and optional output/default/note; HierarchyGroup.sections and HierarchyTemplate.groups hold Vec<String> ID references (not embedded structs); HierarchyFile.template is singular and optional; nav_label and map_label are both required String fields on HierarchySection (not an alias pair, and map_label is not optional).

## Critical Files

- `src/data.rs` - all new types go here; insert after line 608 (after `block_children` fn ends, before `load_data_dir`)

## Reuse

- Existing `Serialize, Deserialize` import at line 2 - already in scope
- Existing `serde_yaml` via `serde_yaml::from_str` - used by the pre-existing unit tests
- `default_true()` at line 47 - NOT used by the new structs (HierarchyItem.default is Option<bool>, not bool)

## Steps

1. Add the TypeTag enum and all hierarchy structs after line 608 in src/data.rs, before the `load_data_dir` fn. Insert exactly this block:

```
+ /// Identifies the structural level of a hierarchy node for scoped (TypeTag, id) uniqueness.
+ #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
+ #[serde(rename_all = "snake_case")]
+ pub enum TypeTag {
+     Template,
+     Group,
+     Section,
+     Field,
+     List,
+     Item,
+     Boilerplate,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct HierarchyItem {
+     pub id: String,
+     pub label: String,
+     pub default: Option<bool>,
+     pub output: Option<String>,
+     pub note: Option<String>,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct HierarchyList {
+     pub id: String,
+     pub label: Option<String>,
+     pub items: Vec<HierarchyItem>,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct HierarchyField {
+     pub id: String,
+     pub label: String,
+     pub field_type: String,
+     #[serde(default)]
+     pub options: Vec<String>,
+     pub list_id: Option<String>,
+     pub data_file: Option<String>,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct HierarchySection {
+     pub id: String,
+     pub nav_label: String,
+     pub map_label: String,
+     pub section_type: String,
+     pub fields: Option<Vec<HierarchyField>>,
+     pub lists: Option<Vec<HierarchyList>>,
+     pub date_prefix: Option<bool>,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct HierarchyGroup {
+     pub id: String,
+     pub nav_label: String,
+     pub sections: Vec<String>,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct HierarchyTemplate {
+     pub id: Option<String>,
+     pub groups: Vec<String>,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct BoilerplateEntry {
+     pub id: String,
+     pub text: String,
+ }
+
+ #[derive(Debug, Clone, Serialize, Deserialize)]
+ pub struct HierarchyFile {
+     pub template: Option<HierarchyTemplate>,
+     pub groups: Option<Vec<HierarchyGroup>>,
+     pub sections: Option<Vec<HierarchySection>>,
+     pub fields: Option<Vec<HierarchyField>>,
+     pub lists: Option<Vec<HierarchyList>>,
+     pub items: Option<Vec<HierarchyItem>>,
+     #[serde(default)]
+     pub boilerplate: Vec<BoilerplateEntry>,
+ }
```

Notes on this struct design (derived from the existing ST70-1 tests in data.rs):
- `HierarchyItem.id` and `.label` are required `String` (not `Option`); `output`, `default`, `note` are optional.
- `HierarchyList.label` is `Option<String>`; no `sticky`, `default`, or `preview` fields at this layer.
- `HierarchyField` uses `field_type: String` and `options: Vec<String>` (inline list or empty), not embedded `HierarchyList` values.
- `HierarchySection` carries both `nav_label: String` (required) and `map_label: String` (required) as separate fields; `fields` and `lists` are `Option<Vec<...>>`.
- `HierarchyGroup.sections` and `HierarchyTemplate.groups` hold `Vec<String>` ID references, not embedded structs.
- `HierarchyFile.template` is singular and optional; `groups`/`sections`/`fields`/`lists`/`items` are all `Option<Vec<...>>`. The `items` top-level field exists for YAML round-trip but items are not referenceable across files.
- `#[serde(rename_all = "snake_case")]` is omitted on the hierarchy structs because all field names are already snake_case and no variant remapping is needed. The TypeTag enum retains it for its variant names.

2. The test module `hierarchy_struct_tests` already exists in src/data.rs at line 2200 (written as TDD pre-conditions for this sub-task). Do NOT re-add it. The 10 tests already there define the contract:
   - TEST-1: HierarchyItem deserializes with required id and label; default/output/note are None
   - TEST-2: HierarchyItem deserializes with all optional fields present (default=Some(true), output, note)
   - TEST-3: HierarchyList deserializes with id and items Vec
   - TEST-4: HierarchyField deserializes with field_type and options Vec<String>; list_id/data_file are None
   - TEST-5: HierarchyField deserializes with list_id and data_file
   - TEST-6: HierarchySection deserializes with nav_label (required String), map_label (required String, separate field), section_type, fields Vec
   - TEST-7: HierarchyGroup deserializes with id, nav_label, and sections as Vec<String> IDs
   - TEST-8: HierarchyTemplate deserializes with groups as Vec<String> IDs; no id required
   - TEST-9: HierarchyFile (minimal): template field (singular, optional) parses; other fields are None
   - TEST-10: HierarchyFile (full): all optional collections present including items

   These tests currently fail because the structs do not exist. After Step 1 they must all pass.

3. Run `cargo test` and confirm all tests pass (all existing tests including the pre-existing hierarchy_struct_tests).

## Verification

### Manual tests

- None required for this sub-task. All validation is automated.

### Automated tests

- `cargo test` must pass with zero failures.
- The pre-existing `hierarchy_struct_tests` module (10 tests) must all pass after Step 1.
- Confirm `cargo test hierarchy_struct_tests` prints all 10 tests as ok.

## Changelog

### Review - 2026-04-03
- #1 (blocking): Replaced all struct definitions in Step 1 to match the TDD contract established by the pre-existing hierarchy_struct_tests in data.rs. Key corrections: HierarchyItem has required id+label and optional output/default/note (not required output with optional id/label); HierarchyField uses field_type+options+list_id+data_file (not lists+format+repeat_limit); HierarchySection has nav_label as required String and map_label as separate Option<String> field (not alias pair, and adds section_type); HierarchyGroup.sections and HierarchyTemplate.groups are Vec<String> ID references (not embedded structs); HierarchyFile.template is singular Option<HierarchyTemplate> with all collections as Option<Vec<...>> including items (not plural templates Vec with #[serde(default)]).
- #2 (blocking): Removed #[serde(rename_all = "snake_case")] from hierarchy structs (all field names already snake_case; alias pair design was also wrong); removed #[serde(alias = "map_label")] from nav_label.
- #3 (blocking): Step 2 rewritten to acknowledge the test module already exists at data.rs:2200 with 10 tests and must NOT be re-added.
- #4 (minor): Fixed insertion line number from 598 to 608 (block_children ends at line 608, not 598).
- #5 (minor): Updated Reuse section to remove incorrect default_true() reference; HierarchyItem.default is Option<bool>.
- #6 (nit): Verification section updated from "at least 8 tests" to "10 tests" to match the actual pre-existing test count.

### Review - 2026-04-03
- #7 (blocking): Changed `HierarchySection.map_label` from `Option<String>` to `String` (required). TEST-6 at data.rs:2292 asserts `assert_eq!(section.map_label, "SEC 1")` which compares directly to a `&str` - this only compiles if `map_label` is `String`, not `Option<String>`. Updated struct definition in Step 1, notes block, TEST-6 description, and Context/Approach descriptions accordingly.

## Progress
- Step 1: Added TypeTag enum and all 8 hierarchy structs (HierarchyItem, HierarchyList, HierarchyField, HierarchySection, HierarchyGroup, HierarchyTemplate, BoilerplateEntry, HierarchyFile) after block_children fn in src/data.rs
- Step 2: Test module hierarchy_struct_tests already exists at data.rs - confirmed no changes needed
- Step 3: cargo test passed - 190 tests ok, 0 failed (includes all 10 hierarchy_struct_tests)

## Implementation
Complete - 2026-04-03
