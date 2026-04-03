**Task**: #46 - Neutralise block_select struct and key names so they aren't tied to treatment-region vocabulary

**Context**: The data-layer renames (data.rs, tx_regions.yml) from sub-tasks 1-3 are complete. `block_select.rs` still uses `RegionState`, `technique_selected`, `toggle_technique`, `BlockSelectFocus::Regions/Techniques`, and field/method names tied to region vocabulary. Tests in `tests_t46_st1_rename` (lines 226-334) fail at compile time because they reference the new neutral names (`BlockSelectGroup`, `item_selected`, etc.) that do not yet exist. External callers in `app.rs`, `ui.rs`, and `note.rs` also reference old method and field names and must be updated so the crate compiles.

**Approach**: Apply all renames in `block_select.rs` in one pass (struct, enum variants, fields, methods), then update every external caller that references the renamed symbols. Old test modules (`tests_st2_region_state_entries_field`, `tests_st3_default_selected`) reference old names and must be updated to match. No new abstractions are needed.

**Critical Files**:
- `src/sections/block_select.rs` lines 4-125 (production code), lines 162-222 (ST3 tests), lines 337-388 (ST2 tests)
- `src/app.rs` lines 244, 1052-1053, 1060, 1078, 1084, 1107-1108, 1116, 1135
- `src/ui.rs` lines 584-586, 600, 609-610, 634, 638, 641, 647
- `src/note.rs` lines 343, 345

**Reuse**: All existing logic is preserved; only identifiers change. No new functions or types required.

**Steps**:

1. In `src/sections/block_select.rs`, rename `RegionState` -> `BlockSelectGroup` everywhere in the file (struct definition, impl block, internal uses in tests). Also rename `technique_selected` -> `item_selected` and `toggle_technique` -> `toggle_item` on `BlockSelectGroup`, and update `has_selection` body accordingly.

```diff
-pub struct RegionState {
+pub struct BlockSelectGroup {
     pub label: String,
     pub header: String,
     pub entries: Vec<PartOption>,
-    pub technique_selected: Vec<bool>,
+    pub item_selected: Vec<bool>,
 }

-impl RegionState {
+impl BlockSelectGroup {
     pub fn from_config(cfg: &BlockSelectEntry) -> Self {
-        let technique_selected = cfg.entries.iter().map(|e| e.default_selected()).collect();
+        let item_selected = cfg.entries.iter().map(|e| e.default_selected()).collect();
         Self {
             label: cfg.label.clone(),
             header: cfg.header.clone(),
             entries: cfg.entries.clone(),
-            technique_selected,
+            item_selected,
         }
     }

     pub fn has_selection(&self) -> bool {
-        self.technique_selected.iter().any(|&s| s)
+        self.item_selected.iter().any(|&s| s)
     }

-    pub fn toggle_technique(&mut self, idx: usize) {
-        if let Some(val) = self.technique_selected.get_mut(idx) {
+    pub fn toggle_item(&mut self, idx: usize) {
+        if let Some(val) = self.item_selected.get_mut(idx) {
             *val = !*val;
         }
     }
 }
```

2. In `src/sections/block_select.rs`, rename the `BlockSelectFocus` enum variants: `Regions` -> `Groups`, `Techniques` -> `Items`.

```diff
 pub enum BlockSelectFocus {
-    Regions,
-    Techniques(usize),
+    Groups,
+    Items(usize),
 }
```

3. In `src/sections/block_select.rs`, rename fields on `BlockSelectState` and update the `new` constructor and all methods in its `impl` block.

```diff
 pub struct BlockSelectState {
-    pub regions: Vec<RegionState>,
-    pub region_cursor: usize,
-    pub technique_cursor: usize,
+    pub groups: Vec<BlockSelectGroup>,
+    pub group_cursor: usize,
+    pub item_cursor: usize,
     pub focus: BlockSelectFocus,
     pub skipped: bool,
     pub completed: bool,
 }
```

```diff
 impl BlockSelectState {
     pub fn new(regions: Vec<BlockSelectEntry>) -> Self {
-        let region_states = regions.iter().map(RegionState::from_config).collect();
+        let region_states = regions.iter().map(BlockSelectGroup::from_config).collect();
         Self {
-            regions: region_states,
-            region_cursor: 0,
-            technique_cursor: 0,
-            focus: BlockSelectFocus::Regions,
+            groups: region_states,
+            group_cursor: 0,
+            item_cursor: 0,
+            focus: BlockSelectFocus::Groups,
             skipped: false,
             completed: false,
         }
     }

     pub fn navigate_up(&mut self) {
         match &self.focus {
-            BlockSelectFocus::Regions => {
-                if self.region_cursor > 0 {
-                    self.region_cursor -= 1;
+            BlockSelectFocus::Groups => {
+                if self.group_cursor > 0 {
+                    self.group_cursor -= 1;
                 }
             }
-            BlockSelectFocus::Techniques(_) => {
-                if self.technique_cursor > 0 {
-                    self.technique_cursor -= 1;
+            BlockSelectFocus::Items(_) => {
+                if self.item_cursor > 0 {
+                    self.item_cursor -= 1;
                 }
             }
         }
     }

     pub fn navigate_down(&mut self) {
         match &self.focus {
-            BlockSelectFocus::Regions => {
-                if !self.regions.is_empty() && self.region_cursor < self.regions.len() - 1 {
-                    self.region_cursor += 1;
+            BlockSelectFocus::Groups => {
+                if !self.groups.is_empty() && self.group_cursor < self.groups.len() - 1 {
+                    self.group_cursor += 1;
                 }
             }
-            BlockSelectFocus::Techniques(region_idx) => {
+            BlockSelectFocus::Items(region_idx) => {
                 let region_idx = *region_idx;
-                if let Some(region) = self.regions.get(region_idx) {
+                if let Some(region) = self.groups.get(region_idx) {
                     if !region.entries.is_empty()
-                        && self.technique_cursor < region.entries.len() - 1
+                        && self.item_cursor < region.entries.len() - 1
                     {
-                        self.technique_cursor += 1;
+                        self.item_cursor += 1;
                     }
                 }
             }
         }
     }

-    pub fn enter_region(&mut self) {
-        let idx = self.region_cursor;
-        self.focus = BlockSelectFocus::Techniques(idx);
-        self.technique_cursor = 0;
+    pub fn enter_group(&mut self) {
+        let idx = self.group_cursor;
+        self.focus = BlockSelectFocus::Items(idx);
+        self.item_cursor = 0;
     }

-    pub fn exit_techniques(&mut self) {
-        self.focus = BlockSelectFocus::Regions;
+    pub fn exit_items(&mut self) {
+        self.focus = BlockSelectFocus::Groups;
     }

-    pub fn toggle_technique(&mut self) {
-        if let BlockSelectFocus::Techniques(region_idx) = self.focus {
-            if let Some(region) = self.regions.get_mut(region_idx) {
-                region.toggle_technique(self.technique_cursor);
+    pub fn toggle_item(&mut self) {
+        if let BlockSelectFocus::Items(region_idx) = self.focus {
+            if let Some(region) = self.groups.get_mut(region_idx) {
+                region.toggle_item(self.item_cursor);
             }
         }
     }

-    pub fn in_techniques(&self) -> bool {
-        matches!(self.focus, BlockSelectFocus::Techniques(_))
+    pub fn in_items(&self) -> bool {
+        matches!(self.focus, BlockSelectFocus::Items(_))
     }

-    pub fn current_region_idx(&self) -> Option<usize> {
+    pub fn current_group_idx(&self) -> Option<usize> {
         match self.focus {
-            BlockSelectFocus::Techniques(i) => Some(i),
+            BlockSelectFocus::Items(i) => Some(i),
             _ => None,
         }
     }
 }
```

4. In `src/sections/block_select.rs`, update `tests_st3_default_selected` module to use new names. Replace all `RegionState` references with `BlockSelectGroup`, `technique_selected` with `item_selected`, and `state.regions` with `state.groups`.

```diff
-        let state = RegionState::from_config(&entry);
-        assert_eq!(state.technique_selected.len(), 3);
-        assert!(state.technique_selected[0], "entry 0 with default=true should start selected");
-        assert!(state.technique_selected[1], "entry 1 with default=true should start selected");
-        assert!(state.technique_selected[2], "entry 2 with default=true should start selected");
+        let state = BlockSelectGroup::from_config(&entry);
+        assert_eq!(state.item_selected.len(), 3);
+        assert!(state.item_selected[0], "entry 0 with default=true should start selected");
+        assert!(state.item_selected[1], "entry 1 with default=true should start selected");
+        assert!(state.item_selected[2], "entry 2 with default=true should start selected");
```

(Apply the same rename pattern — `RegionState` -> `BlockSelectGroup`, `technique_selected` -> `item_selected` — to: the second `from_config` block in `region_state_one_default_false_starts_unselected` at lines 197-201, and to the `state.regions[0].technique_selected` index assertions in `block_select_state_new_propagates_defaults` at lines 220-221, replacing `state.regions` with `state.groups`.)

5. In `src/sections/block_select.rs`, update `tests_st2_region_state_entries_field` module to use new names. Replace `RegionState::from_config` with `BlockSelectGroup::from_config`, `enter_region` with `enter_group`, `technique_cursor` with `item_cursor`, `state.regions` with `state.groups`, and update the two stale comments at lines 350 and 362 to reference `BlockSelectGroup`.

```diff
-    // ST2-TEST-1: RegionState must expose an `entries` field of type Vec<PartOption>
+    // ST2-TEST-1: BlockSelectGroup must expose an `entries` field of type Vec<PartOption>
```

```diff
-    // ST2-TEST-2: `techniques` field must NOT exist on RegionState.
+    // ST2-TEST-2: `techniques` field must NOT exist on BlockSelectGroup.
```

```diff
-        let state = RegionState::from_config(&entry);
+        let state = BlockSelectGroup::from_config(&entry);
```

```diff
-        state.enter_region();
+        state.enter_group();
         state.navigate_down();
-        assert_eq!(state.technique_cursor, 1);
+        assert_eq!(state.item_cursor, 1);
```

```diff
-        assert_eq!(state.regions.len(), 1);
-        assert_eq!(state.regions[0].entries.len(), 2);
+        assert_eq!(state.groups.len(), 1);
+        assert_eq!(state.groups[0].entries.len(), 2);
```

6. In `src/app.rs`, rename all references to the old method and field names on `BlockSelectState`:
   - `s.in_techniques()` -> `s.in_items()` (lines 244, 1052-1053)
   - local variable `in_techniques` -> `in_items` (lines 1052, 1057)
   - `s.exit_techniques()` -> `s.exit_items()` (lines 1060, 1084)
   - `s.toggle_technique()` -> `s.toggle_item()` (line 1078)
   - `s.regions.is_empty()` -> `s.groups.is_empty()` (line 1107)
   - `s.enter_region()` -> `s.enter_group()` (line 1108)
   - `s.regions.iter()` -> `s.groups.iter()` (lines 1116, 1135)

7. In `src/ui.rs`, rename all references:
   - `state.in_techniques()` -> `state.in_items()` (line 584)
   - `state.current_region_idx()` -> `state.current_group_idx()` (line 585)
   - `state.regions.get(region_idx)` -> `state.groups.get(region_idx)` (line 586)
   - `format!(" {} - Techniques ", region.label)` -> `format!(" {} - Items ", region.label)` (line 587)
   - `state.technique_cursor` -> `state.item_cursor` (lines 600, 610)
   - `region.technique_selected.get(i)` -> `region.item_selected.get(i)` (line 609)
   - `state.regions.len()` -> `state.groups.len()` (line 634)
   - `state.region_cursor` -> `state.group_cursor` (lines 638, 647)
   - `state.regions.iter()` -> `state.groups.iter()` (line 641)

8. In `src/note.rs`, rename the field iteration on `BlockSelectState` and `BlockSelectGroup`:
   - `for region_state in &s.regions` -> `for region_state in &s.groups` (line 343)
   - `region_state.technique_selected` -> `region_state.item_selected` (line 345)

9. Run `cargo test` to confirm all tests in `tests_t46_st1_rename`, `tests_st2_region_state_entries_field`, and `tests_st3_default_selected` pass with zero compilation errors.

**Verification**:

### Manual tests
- Run `cargo build` from the project root and confirm zero errors.

### Automated tests
- `cargo test` - the `tests_t46_st1_rename` module (12 tests, lines 226-334) is the primary coverage. `tests_st2_region_state_entries_field` and `tests_st3_default_selected` must also continue to pass.
- Specific: `cargo test tests_t46_st1_rename` should show 12 passing tests.

## Prefect-1 Report

**Round**: 1
**Verdict**: Nit fixed, no blocking or minor issues.

**Cross-check summary**:
- All old names in `block_select.rs` production code (lines 4-125) are covered by Steps 1-3 diffs, which apply cleanly against the actual source.
- All old names in `tests_st3_default_selected` (lines 162-222) are covered by Step 4.
- All old names in `tests_st2_region_state_entries_field` (lines 337-388) are covered by Step 5.
- All old call sites in `app.rs` (lines 244, 1052-1053, 1057, 1060, 1078, 1084, 1107-1108, 1116, 1135) are covered by Step 6.
- All old call sites in `ui.rs` (lines 584-586, 600, 609-610, 634, 638, 641, 647) are covered by Step 7.
- All old field references in `note.rs` (lines 343, 345) are covered by Step 8.
- No missed call sites found via full-codebase grep.

**Nit fixed**:
- Step 4 diff replaced `...` placeholders in `assert!` calls with the actual string literal messages from the source file, and expanded the prose note to include specific line numbers for the second and third rename locations within the ST3 test module.

## Prefect-2 Report

**Round**: 1
**Verdict**: Two nits found; no blocking or minor issues.

**1. (nit) ui.rs line 587 - string literal still uses old vocabulary**

The plan renames the `BlockSelectFocus::Techniques` variant to `Items` and all related symbols, but the UI title string at `src/ui.rs:587` is not updated:

```
src/ui.rs:587
-            let title = format!(" {} - Techniques ", region.label);
+            let title = format!(" {} - Items ", region.label);
```

After the rename the enum variant is `Items` but the displayed panel header still says "Techniques", creating a vocabulary mismatch in the running UI. Step 7 should include this line.

**2. (nit) block_select.rs - stale comments in tests_st2_region_state_entries_field**

Comments at lines 350 and 362 reference the old struct name and field name that are being removed:

- Line 350: `// ST2-TEST-1: RegionState must expose an \`entries\` field` -- should reference `BlockSelectGroup`
- Line 362: `// ST2-TEST-2: \`techniques\` field must NOT exist on RegionState.` -- should reference `BlockSelectGroup`

These don't affect compilation but are misleading after the rename. Step 5 should include comment updates for these two lines.

## Changelog

### Review - 2026-04-02
- #1 (nit): Step 4 diff - replaced `...` placeholders with actual assert message strings and added line-number references to the prose note for the remaining ST3 rename locations.

### Review - 2026-04-02
- Nit 1: Step 7 - added `format!(" {} - Items ", region.label)` rename for `src/ui.rs:587` string literal (was still "Techniques").
- Nit 2: Step 5 - added two comment-update diffs for `block_select.rs` lines 350 and 362, updating references from `RegionState` to `BlockSelectGroup`.
