## Task
#45 - Refactor data format to flat, type-tagged YML blocks with ID-based cross-references

## Context
Sub-task 1 defined `FlatBlock` and `FlatFile` in `src/flat_file.rs`. Sub-task 2 implements `pub fn load_data_dir(path: &Path) -> Result<AppData, String>` in `src/data.rs`, which is called by 9 failing tests at line 883. The tests write temp YAML files with a `children:` list field on blocks, so `FlatBlock` must first gain a `children: Vec<String>` field (defaulting empty). After the new loader passes all 9 tests, the old `SectionsFile` deserialization path inside `AppData::load` is removed.

## Approach
Add `children: Vec<String>` to every `FlatBlock` variant in `src/flat_file.rs`. Implement `load_data_dir` as a free function in `src/data.rs`: scan the given directory for `*.yml` files, skip `keybindings.yml`, deserialize each as `FlatFile`, merge all blocks into a single pool. Enforce duplicate `(type-tag, id)` pairs as loud errors using a `HashSet`. Resolve ID-reference children lists, erroring on any unknown ID. Detect cycles with a standard DFS visited/in-stack algorithm. Return a minimally-populated `AppData` (empty `groups`, `sections`, `list_data`, etc. - sufficient for the tests to receive `Ok`). Finally, remove the `SectionsFile` struct and the `SectionsFile` deserialization code from `AppData::load`, replacing it with a call to the new `load_data_dir` helper (or an equivalent inline scan) so existing callers are unaffected.

## Critical Files
- `src/flat_file.rs` - add `children: Vec<String>` with `#[serde(default)]` to each `FlatBlock` variant (lines 7-13)
- `src/data.rs` lines 103-106 - `SectionsFile` struct (to be removed)
- `src/data.rs` lines 212-277 - `AppData::load` (replace `SectionsFile` path, call new loader or inline scan)
- `src/data.rs` lines 883-1074 - 9 failing tests calling `load_data_dir`
- `src/data.rs` line 1081 - `find_data_dir` checks `sections.yml` existence (update sentinel after format change)

## Reuse
- `serde::Deserialize` `#[serde(default)]` pattern already used in `KeyBindings` fields in `src/data.rs` (lines 154-163) - apply same pattern to `children`
- `std::collections::HashMap` already imported at `src/data.rs` line 3 - reuse for duplicate detection (`HashSet` must be added)
- `serde_yaml::from_str` pattern already used throughout `AppData::load` - same call for `FlatFile`
- `fs::read_dir` from `std::fs` - use to scan the directory for `*.yml` files

## Steps

1. **Add `children` to `FlatBlock` in `src/flat_file.rs`.**

Each of the five variants gains `#[serde(default)] children: Vec<String>`. This is a non-breaking addition: existing YAML without `children:` deserializes to an empty vec.

```diff
-    Box { id: String },
-    Group { id: String },
-    Section { id: String },
-    Field { id: String },
-    OptionsList { id: String },
+    Box { id: String, #[serde(default)] children: Vec<String> },
+    Group { id: String, #[serde(default)] children: Vec<String> },
+    Section { id: String, #[serde(default)] children: Vec<String> },
+    Field { id: String, #[serde(default)] children: Vec<String> },
+    OptionsList { id: String, #[serde(default)] children: Vec<String> },
```

2. **Add `use std::collections::HashSet`, `use std::path::Path`, and `use crate::flat_file::FlatFile` imports to `src/data.rs`.**

`HashSet` is not yet imported. `Path` is needed for the function signature. `FlatFile` is needed so `serde_yaml::from_str::<FlatFile>` compiles -- `flat_file` is not currently imported in `data.rs`.

```diff
 use std::collections::HashMap;
+use std::collections::HashSet;
 use std::fs;
 use std::path::PathBuf;
+use std::path::Path;
+use crate::flat_file::FlatFile;
```

3. **Implement `load_data_dir` in `src/data.rs`** as a free function immediately before the `#[cfg(test)]` block (around line 440 region, after `resolve_hint`).

The function signature the tests call: `pub fn load_data_dir(path: &Path) -> Result<AppData, String>`

Implementation logic:
- Collect all `*.yml` entries from `path` via `fs::read_dir`; skip `keybindings.yml`
- For each file: read content, `serde_yaml::from_str::<FlatFile>(&content)` - error with filename on parse failure
- Push all blocks into a `Vec<FlatBlock>` pool
- Duplicate check: for each block, compute a key `(type_tag_string, id_string)`; insert into a `HashSet`; if already present, return `Err` mentioning the duplicate ID
- Build an `id -> index` lookup map keyed by just `id` (any type) for reference resolution
- Missing-ref check: for each block, for each string in `children`, verify it exists in the pool by ID; if not, return `Err` mentioning the missing ID
- Cycle detection: standard DFS with `visited: HashSet<usize>` and `in_stack: HashSet<usize>`; iterate over all nodes as potential roots; return `Err` on cycle
- On success: return `AppData { groups: vec![], sections: vec![], list_data: HashMap::new(), checklist_data: HashMap::new(), region_data: HashMap::new(), keybindings: KeyBindings::default(), data_dir: path.to_path_buf() }`

Helper for getting type-tag and id from any variant:

```rust
fn block_type_tag(b: &crate::flat_file::FlatBlock) -> &'static str {
    use crate::flat_file::FlatBlock::*;
    match b { Box {..} => "box", Group {..} => "group", Section {..} => "section",
              Field {..} => "field", OptionsList {..} => "options-list" }
}

fn block_id(b: &crate::flat_file::FlatBlock) -> &str {
    use crate::flat_file::FlatBlock::*;
    match b { Box { id, .. } | Group { id, .. } | Section { id, .. }
            | Field { id, .. } | OptionsList { id, .. } => id.as_str() }
}

fn block_children(b: &crate::flat_file::FlatBlock) -> &[String] {
    use crate::flat_file::FlatBlock::*;
    match b { Box { children, .. } | Group { children, .. } | Section { children, .. }
            | Field { children, .. } | OptionsList { children, .. } => children.as_slice() }
}
```

4. **Remove `SectionsFile` and update `AppData::load`.**

Delete `struct SectionsFile` (lines 103-106). In `AppData::load`, replace the `SectionsFile` deserialization lines (214-222) with a call to `load_data_dir(&data_dir)` and map its error:

```diff
-        let sections_path = data_dir.join("sections.yml");
-        let sections_content = fs::read_to_string(&sections_path)?;
-        let sections_file: SectionsFile = serde_yaml::from_str(&sections_content)?;
-
-        let groups = sections_file.groups.clone();
-        let sections: Vec<SectionConfig> = groups
-            .iter()
-            .flat_map(|g| g.sections.iter().cloned())
-            .collect();
+        let base = load_data_dir(&data_dir)
+            .map_err(|e| anyhow::anyhow!(e))?;
+        let groups = base.groups;
+        let sections = base.sections;
```

The remainder of `AppData::load` (list/checklist/region data loading and keybindings) is unchanged.

5. **Update `find_data_dir` sentinel** (line 1081) to not require `sections.yml` - use any yml file or the directory's existence:

```diff
-    if cwd_data.join("sections.yml").exists() {
+    if cwd_data.exists() && cwd_data.is_dir() {
```

Apply the same change to the exe-parent branch (line 1089).

6. **Verify all 9 tests pass:**

```
cargo test load_data_dir
```

Also run the full suite to confirm no regressions:

```
cargo test
```

## Verification

### Manual tests
None - this sub-task has no UI or runtime-visible behavior. The app must still start with the existing `data/` directory once sub-task 3 (YAML rewrite) is complete; that is outside scope here.

### Automated tests
Run `cargo test load_data_dir` - all 9 tests in the `load_data_dir` block must pass:
- `load_data_dir_returns_app_data_for_valid_directory`
- `load_data_dir_merges_blocks_from_multiple_yml_files`
- `load_data_dir_errors_on_duplicate_id_and_type`
- `load_data_dir_errors_on_duplicate_id_and_type_across_files`
- `load_data_dir_allows_same_id_different_type`
- `load_data_dir_errors_on_missing_child_id_reference`
- `load_data_dir_errors_on_direct_cycle`
- `load_data_dir_errors_on_indirect_cycle`
- `load_data_dir_accepts_acyclic_tree`

Run `cargo test` to confirm all previously-passing tests still pass (no regressions from removing `SectionsFile`).

## Changelog

### Review - 2026-04-01
- #1 (blocking): Step 2 - added `use crate::flat_file::FlatFile;` to the import diff; `flat_file` was not imported in `data.rs` and `from_str::<FlatFile>` would not compile without it
- #2 (minor): Step 5 - corrected exe-parent branch line reference from 1088 to 1089 (actual line in source)

### Review - 2026-04-01
- #3 (minor): Step 3 - corrected placement hint from "after `combined_hints`" to "after `resolve_hint`"; no function named `combined_hints` exists in `data.rs`, the last function before `#[cfg(test)]` is `resolve_hint` at line 431

## Progress
- Step 1: Added `#[serde(default)] children: Vec<String>` to all five FlatBlock variants in src/flat_file.rs
- Step 2: Added HashSet, Path, and FlatFile imports to src/data.rs
- Step 3: Implemented load_data_dir free function with helper functions in src/data.rs
- Step 4: Removed SectionsFile struct and replaced deserialization in AppData::load with load_data_dir call
- Step 5: Updated find_data_dir sentinel to check directory existence instead of sections.yml presence
- Step 6: All 61 tests pass (9 load_data_dir tests + 52 existing tests, 0 regressions)
