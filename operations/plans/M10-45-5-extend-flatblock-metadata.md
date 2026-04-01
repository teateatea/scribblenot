## Task
#45 - Refactor data format to flat, type-tagged YML blocks with ID-based cross-references

## Context
Sub-task 1 (attempt 1) added the five `FlatBlock` variants with only `id` and `children` fields. Three tests added at line 120 of `src/flat_file.rs` now fail because the variants lack the metadata fields required to reconstruct the runtime structs (`SectionGroup`, `SectionConfig`, etc.) from flat blocks:

- `group_block_deserializes_name_and_num` - `Group` variant is missing `name: Option<String>` and `num: Option<usize>`
- `section_block_deserializes_name_map_label_section_type` - `Section` variant is missing `name: Option<String>`, `map_label: Option<String>`, and `section_type: Option<String>`
- `options_list_block_deserializes_entries` - `OptionsList` variant is missing `entries: Vec<PartOption>`

All new fields must use `#[serde(default)]` so existing YAML without them continues to parse.

## Approach
Extend only the three affected `FlatBlock` variants in `src/flat_file.rs` with the metadata fields the failing tests require. Add a `use crate::data::PartOption;` import so `PartOption` is in scope for the `OptionsList` variant. All new fields use `#[serde(default)]` to stay backward-compatible with YAML that omits them. No other variants, structs, or files are modified.

## Critical Files
- `src/flat_file.rs` lines 1-13 - the `FlatBlock` enum definition; three variants need new fields
- `src/data.rs` lines 11-16 - `PartOption` enum definition to reuse (already has `#[serde(untagged)]`)

## Reuse
- `PartOption` from `src/data.rs` (line 12) - already handles `Simple(String)` and `Labeled { label, output }` shapes; import with `use crate::data::PartOption;`
- `#[serde(default)]` pattern already used throughout `src/data.rs` (lines 47, 77, etc.)
- `Option<String>` / `Option<usize>` pattern from `SectionGroup` (lines 86-87) and `SectionConfig` (lines 92-103) in `src/data.rs`

## Steps

1. Add `use crate::data::PartOption;` import to `src/flat_file.rs` below the existing `use serde::{Deserialize, Serialize};` line.

```diff
 use serde::{Deserialize, Serialize};
+use crate::data::PartOption;
```

2. Extend the `Group` variant with `name` and `num` fields, both optional with `#[serde(default)]`.

```diff
-    Group { id: String, #[serde(default)] children: Vec<String> },
+    Group {
+        id: String,
+        #[serde(default)] children: Vec<String>,
+        #[serde(default)] name: Option<String>,
+        #[serde(default)] num: Option<usize>,
+    },
```

3. Extend the `Section` variant with `name`, `map_label`, and `section_type` fields, all optional with `#[serde(default)]`.

```diff
-    Section { id: String, #[serde(default)] children: Vec<String> },
+    Section {
+        id: String,
+        #[serde(default)] children: Vec<String>,
+        #[serde(default)] name: Option<String>,
+        #[serde(default)] map_label: Option<String>,
+        #[serde(default)] section_type: Option<String>,
+    },
```

4. Extend the `OptionsList` variant with an `entries` field defaulting to an empty vec.

```diff
-    OptionsList { id: String, #[serde(default)] children: Vec<String> },
+    OptionsList {
+        id: String,
+        #[serde(default)] children: Vec<String>,
+        #[serde(default)] entries: Vec<PartOption>,
+    },
```

5. Update the existing test struct-literal constructions in `src/flat_file.rs` that build the now-extended variants. `#[serde(default)]` applies only to serde deserialization; Rust struct literals require all fields to be specified explicitly. Four construction sites need the new fields added with their zero/None defaults:

- Line 43: `FlatBlock::Group { id: "grp1".to_string(), children: vec![] }` — add `name: None, num: None`
- Line 52: `FlatBlock::Section { id: "sec1".to_string(), children: vec![] }` — add `name: None, map_label: None, section_type: None`
- Line 70: `FlatBlock::OptionsList { id: "opt1".to_string(), children: vec![] }` — add `entries: vec![]`
- Line 83 (inside `flat_file_holds_list_of_blocks`): `FlatBlock::Section { id: "s1".to_string(), children: vec![] }` — add `name: None, map_label: None, section_type: None`

```diff
-        let block = FlatBlock::Group { id: "grp1".to_string(), children: vec![] };
+        let block = FlatBlock::Group { id: "grp1".to_string(), children: vec![], name: None, num: None };
```

```diff
-        let block = FlatBlock::Section { id: "sec1".to_string(), children: vec![] };
+        let block = FlatBlock::Section { id: "sec1".to_string(), children: vec![], name: None, map_label: None, section_type: None };
```

```diff
-        let block = FlatBlock::OptionsList { id: "opt1".to_string(), children: vec![] };
+        let block = FlatBlock::OptionsList { id: "opt1".to_string(), children: vec![], entries: vec![] };
```

```diff
-                FlatBlock::Section { id: "s1".to_string(), children: vec![] },
+                FlatBlock::Section { id: "s1".to_string(), children: vec![], name: None, map_label: None, section_type: None },
```

6. Run the full test suite to confirm all three previously-failing tests now pass and no existing tests regress:

```
cargo test flat_file
```

All eleven tests in `flat_file::tests` must pass.

## Verification

### Manual tests
None - this sub-task has no UI or runtime-visible behavior.

### Automated tests
Run `cargo test flat_file` from the project root. The three previously failing tests must now pass:
- `group_block_deserializes_name_and_num`
- `section_block_deserializes_name_map_label_section_type`
- `options_list_block_deserializes_entries`

The eight tests that already pass must continue to pass:
- `flat_block_box_variant_has_id`
- `flat_block_group_variant_has_id`
- `flat_block_section_variant_has_id`
- `flat_block_field_variant_has_id`
- `flat_block_options_list_variant_has_id`
- `flat_file_holds_list_of_blocks`
- `flat_file_deserializes_from_yaml`
- `flat_block_id_is_string`

Also run `cargo build` to confirm no new warnings are introduced.

## Changelog

### Review - 2026-04-01
- #1: Added step 5 to update existing test struct-literal constructions for Group, Section, and OptionsList variants; `#[serde(default)]` does not supply Rust field defaults for struct literals, so the four affected constructions would fail to compile without explicit values for the new fields.

### Review #2 - 2026-04-01
- #1: Corrected "Five construction sites" to "Four" in step 5 preamble - only 4 sites exist (Box and Field variants are not extended).
- #2: Corrected line reference for `flat_file_holds_list_of_blocks` Section construction from line 82 to line 83 (actual line in source).
- #3: Corrected `PartOption` line reference in Reuse section from line 11 to line 12 (line 11 is the `#[serde(untagged)]` attribute; the enum keyword is line 12).

## Progress
- Step 1: Added `use crate::data::PartOption;` import below `use serde::{Deserialize, Serialize};`
- Step 2: Extended `Group` variant with `name: Option<String>` and `num: Option<usize>` fields using `#[serde(default)]`
- Step 3: Extended `Section` variant with `name`, `map_label`, and `section_type` fields using `#[serde(default)]`
- Step 4: Extended `OptionsList` variant with `entries: Vec<PartOption>` field using `#[serde(default)]`
- Step 5: Updated four test struct literals to include new fields with None/vec![] defaults
- Step 6: All 11 flat_file tests pass (3 previously failing now pass, 8 existing still pass)

## Implementation
Complete - 2026-04-01
