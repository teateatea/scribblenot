## Task
#45 - Refactor data format to flat, type-tagged YML blocks with ID-based cross-references

## Context
`src/flat_file.rs` exists with failing tests that require `FlatBlock` (an enum with five variants: Box, Group, Section, Field, OptionsList) and `FlatFile` (a struct wrapping `Vec<FlatBlock>`). The tests verify struct construction, pattern matching, and `serde_yaml` deserialization using a `type:` discriminant field. This sub-task is purely additive: define those types in a new module so the tests compile and pass. No existing code is deleted or modified beyond wiring `flat_file` as a proper module.

## Approach
Define `FlatBlock` as a Rust enum with `#[serde(tag = "type", rename_all = "kebab-case")]` so that the YAML `type: options-list` field maps to the `OptionsList` variant. Each variant is a struct variant carrying `id: String`. Define `FlatFile` as a plain struct with `blocks: Vec<FlatBlock>`. Place all definitions at the top of `src/flat_file.rs`, above the existing `#[cfg(test)]` block. The module is already wired via `#[cfg(test)] mod flat_file;` in `main.rs`, so no change to `main.rs` is needed.

## Critical Files
- `src/flat_file.rs` - add `FlatBlock` enum and `FlatFile` struct above the existing test block (lines 1-2 are comments, blank line at 3, tests start at line 4)
- `Cargo.toml` - `serde` with `derive` feature and `serde_yaml` are already present (lines 13-14); no changes needed

## Reuse
- `serde::{Deserialize, Serialize}` pattern from `src/data.rs` (line 2) - same import style
- `serde_yaml` deserialization already used in `src/data.rs` - no new dependencies required

## Steps

1. Insert the `FlatBlock` enum and `FlatFile` struct into `src/flat_file.rs` at the top. The file currently has two comment lines (1-2), a blank line at 3, and `#[cfg(test)]` at line 4. The diff below replaces both comment lines with the first comment only, then adds the new type definitions.

```diff
--- a/src/flat_file.rs
+++ b/src/flat_file.rs
-// flat_file.rs — flat YAML data structures for scribblenot form definitions.
-// This module will hold FlatBlock and FlatFile once implemented.
+// flat_file.rs — flat YAML data structures for scribblenot form definitions.
+
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+#[serde(tag = "type", rename_all = "kebab-case")]
+pub enum FlatBlock {
+    Box { id: String },
+    Group { id: String },
+    Section { id: String },
+    Field { id: String },
+    OptionsList { id: String },
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct FlatFile {
+    pub blocks: Vec<FlatBlock>,
+}
```

2. Verify the tests compile and pass:
```
cargo test flat_file
```
All eight tests in the `flat_file::tests` module should pass.

## Verification

### Manual tests
None - this sub-task has no UI or runtime-visible behavior.

### Automated tests
Run `cargo test flat_file` from the project root. All eight tests must pass:
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

### Review – 2026-04-01
- #1: Updated test count from seven to eight in Steps step 2 and Verification section; added missing `flat_block_id_is_string` to the Automated tests list

### Review – 2026-04-01
- #1 (nit): Corrected Critical Files description to note two comment lines (1-2) not one; updated Step 1 narrative to match actual file structure (two comment lines, blank at 3, cfg(test) at 4)

## Progress
- Step 1: Inserted FlatBlock enum (5 variants) and FlatFile struct into src/flat_file.rs above existing test block
- Step 2: Ran cargo test flat_file - all 8 tests pass

## Implementation
Complete - 2026-04-01
