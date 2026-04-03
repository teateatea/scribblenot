**Task**: #46 - Neutralise block_select struct and key names so they aren't tied to treatment-region vocabulary

**Context**: `data.rs` still contains `TechniqueConfig`, `RegionConfig`, and `RegionsFile` which are domain-specific names tied to treatment-region vocabulary. `PartOption` already exists as the correct replacement for `TechniqueConfig`. The failing compile-time tests in `data.rs` (`rename_tests` module) document the required end state: `BlockSelectEntry`, `BlockSelectFile`, and `block_select_data` must exist. `app.rs` and `block_select.rs` reference the old names and must be updated to compile.

**Approach**: Rename the three types and one field in `data.rs` in-place, then update the two external files that import or reference them. Delete `TechniqueConfig` entirely since `PartOption` already covers the same role. Update `block_select.rs` to use the new names. Update `app.rs` field accesses.

**Critical Files**:
- `src/data.rs` lines 119-136, 204-212, 221-223, 259-260, 284-290, 684
- `src/sections/block_select.rs` lines 1-13, 50
- `src/app.rs` lines 151, 1431, 1463
- `src/note.rs` line 350
- `src/ui.rs` line 623

**Reuse**: `PartOption` (already defined in `data.rs` lines 11-39) replaces `TechniqueConfig` directly - no new type needed.

**Steps**:

1. In `src/data.rs`, delete the `TechniqueConfig` struct (lines 118-123) entirely.

```diff
-#[derive(Debug, Clone, Serialize, Deserialize)]
-pub struct TechniqueConfig {
-    pub id: String,
-    pub label: String,
-    pub output: String,
-}
-
 #[derive(Debug, Clone, Serialize, Deserialize)]
-pub struct RegionConfig {
+pub struct BlockSelectEntry {
     pub id: String,
     pub label: String,
     pub header: String,
-    pub techniques: Vec<TechniqueConfig>,
+    pub entries: Vec<PartOption>,
 }

 #[derive(Debug, Clone, Serialize, Deserialize)]
-struct RegionsFile {
-    regions: Vec<RegionConfig>,
+struct BlockSelectFile {
+    entries: Vec<BlockSelectEntry>,
 }
```

2. In `src/data.rs`, update `AppData` struct field and all references in `AppData::load` and the standalone `load_data_dir` function:

```diff
 pub struct AppData {
     pub groups: Vec<SectionGroup>,
     pub sections: Vec<SectionConfig>,
     pub list_data: HashMap<String, Vec<ListEntry>>,
     pub checklist_data: HashMap<String, Vec<String>>,
-    pub region_data: HashMap<String, Vec<RegionConfig>>,
+    pub block_select_data: HashMap<String, Vec<BlockSelectEntry>>,
     pub keybindings: KeyBindings,
     pub data_dir: PathBuf,
 }
```

```diff
-        let mut region_data: HashMap<String, Vec<RegionConfig>> = HashMap::new();
+        let mut block_select_data: HashMap<String, Vec<BlockSelectEntry>> = HashMap::new();
```

```diff
-                        "block_select" => {
-                            let file: RegionsFile = serde_yaml::from_str(&content)?;
-                            region_data.insert(data_file.clone(), file.regions);
-                        }
+                        "block_select" => {
+                            let file: BlockSelectFile = serde_yaml::from_str(&content)?;
+                            block_select_data.insert(data_file.clone(), file.entries);
+                        }
```

```diff
         Ok(Self {
             groups,
             sections,
             list_data,
             checklist_data,
-            region_data,
+            block_select_data,
             keybindings,
             data_dir,
         })
```

3. In `src/data.rs`, update the `load_data_dir` stub at line 684:

```diff
-            region_data: HashMap::new(),
+            block_select_data: HashMap::new(),
```

4. In `src/sections/block_select.rs`, update the import and all uses of the old names:

```diff
-use crate::data::{RegionConfig, TechniqueConfig};
+use crate::data::{BlockSelectEntry, PartOption};

 #[derive(Debug, Clone)]
 pub struct RegionState {
     pub label: String,
     pub header: String,
-    pub techniques: Vec<TechniqueConfig>,
+    pub techniques: Vec<PartOption>,
     pub technique_selected: Vec<bool>,
 }

 impl RegionState {
-    pub fn from_config(cfg: &RegionConfig) -> Self {
-        let technique_selected = vec![false; cfg.techniques.len()];
+    pub fn from_config(cfg: &BlockSelectEntry) -> Self {
+        let technique_selected = vec![false; cfg.entries.len()];
         Self {
             label: cfg.label.clone(),
             header: cfg.header.clone(),
-            techniques: cfg.techniques.clone(),
+            techniques: cfg.entries.clone(),
             technique_selected,
         }
     }
```

```diff
 impl BlockSelectState {
-    pub fn new(regions: Vec<RegionConfig>) -> Self {
+    pub fn new(regions: Vec<BlockSelectEntry>) -> Self {
```

5. In `src/app.rs`, update `region_data` field accesses to `block_select_data`:

```diff
-                        .and_then(|f| data.region_data.get(f))
+                        .and_then(|f| data.block_select_data.get(f))
```

```diff
-            region_data: Default::default(), keybindings: KeyBindings::default(),
+            block_select_data: Default::default(), keybindings: KeyBindings::default(),
```
(Apply this replacement to both occurrences at lines 1431 and 1463.)

6. In `src/note.rs`, update the field access at line 350 — `t` is now `&PartOption` (an enum), which has no `.output` field, only an `.output()` method:

```diff
-                .map(|t| t.output.clone())
+                .map(|t| t.output().to_string())
```

7. In `src/ui.rs`, update the field access at line 623 — `tech` is now `&PartOption` (an enum), which has no `.label` field, only a `.label()` method:

```diff
-                        format!("{} {} {}", prefix, check, tech.label),
+                        format!("{} {} {}", prefix, check, tech.label()),
```

8. Update `data/tx_regions.yml` to match the new field names used by `BlockSelectFile` and `BlockSelectEntry`:
   - Rename the top-level key `regions:` to `entries:`
   - Rename each nested key `techniques:` to `entries:`

   This is required because serde deserializes by field name. After renaming the struct fields from `regions`/`techniques` to `entries`, the existing YAML will fail to parse at runtime (the top-level `regions:` key and per-entry `techniques:` keys will be ignored and the struct fields will be empty, or serde will return an error). The rename tests only check compilation, so this runtime breakage would not surface from `cargo test rename_tests` alone.

9. Run `cargo build` to confirm the file compiles with no errors. The `rename_tests` module tests should also pass.

**Verification**:

### Manual tests
- Run `cargo build` from the project root and confirm zero errors.
- Run `cargo test rename_tests` and confirm all three tests pass: `block_select_entry_exists_with_entries_field`, `block_select_file_exists_with_entries_field`, `app_data_has_block_select_data_not_region_data`.

### Automated tests
- `cargo test` (full suite) should show no regressions. The three compile-time tests in `rename_tests` are the primary automated coverage for this rename.

## Prefect-1 Report

### Blocking

**B1** - `data/tx_regions.yml` not updated for renamed serde fields (`M11-46-1-rename-block-select-types.md`, Step 6 added)

The plan renames `RegionsFile.regions` -> `BlockSelectFile.entries` and `RegionConfig.techniques` -> `BlockSelectEntry.entries`. Because serde deserializes by field name, the existing `tx_regions.yml` (which uses `regions:` at the top level and `techniques:` within each region) will fail to produce populated data at runtime after these struct renames. The compile tests do not exercise YAML loading, so `cargo test rename_tests` and `cargo build` would both pass while the application silently loads empty block_select data (or errors). Step 6 (YAML data update) was added to address this.

## Prefect-2 Report

### Blocking

**B2** - `note.rs:350` accesses `t.output` as a struct field, but after the rename `t` is `&PartOption` (an enum). `PartOption` has no `.output` field - only an `.output()` method. The plan does not update `note.rs` and `cargo build` will fail.

`src/note.rs:350`:
```
- .map(|t| t.output.clone())
+ .map(|t| t.output().to_string())
```

The plan must add a step to update `src/note.rs` line 350.

**B3** - `ui.rs:623` accesses `tech.label` as a struct field, but after the rename `tech` is `&PartOption` (an enum). `PartOption` has no `.label` field - only a `.label()` method. The plan does not update `ui.rs` and `cargo build` will fail.

`src/ui.rs:623`:
```
- format!("{} {} {}", prefix, check, tech.label),
+ format!("{} {} {}", prefix, check, tech.label()),
```

The plan must add a step to update `src/ui.rs` line 623.

## Changelog

### Review - 2026-04-02
- B1: Added Step 6 to update `data/tx_regions.yml` - rename top-level key `regions:` to `entries:` and each nested `techniques:` to `entries:` to match the renamed serde struct fields. Renumbered original Step 6 to Step 7.

### Review - 2026-04-02
- B2: Added `src/note.rs` line 350 to Critical Files and added Step 6 to fix `.output` field access to `.output()` method call on `PartOption` enum.
- B3: Added `src/ui.rs` line 623 to Critical Files and added Step 7 to fix `.label` field access to `.label()` method call on `PartOption` enum. Renumbered YAML update to Step 8 and cargo build to Step 9.
