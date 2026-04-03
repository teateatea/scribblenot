## Task

#47 - Add per-technique default selection state to block_select

## Context

ST1 added `default: bool` (serde default = true) to `PartOption::Full`. ST2 renamed the entries field. ST3 (this sub-task) must wire that `default` field into runtime state so `RegionState::from_config` initializes `technique_selected` from each entry's default value instead of hardcoding `false`. Failing tests at `src/sections/block_select.rs:128` (module `tests_st3_default_selected`) specify the expected behavior: a `default_selected() -> bool` method on `PartOption`, and `from_config` using it to populate `technique_selected`.

## Approach

Add `default_selected() -> bool` to the existing `PartOption` impl block in `src/data.rs`. Then update the single line in `RegionState::from_config` in `src/sections/block_select.rs` that builds `technique_selected` from a `vec![false; ...]` to instead map over `cfg.entries` and call `.default_selected()` on each entry.

## Critical Files

- `src/data.rs` lines 18-39 (PartOption impl block) - add `default_selected()` method
- `src/sections/block_select.rs` line 13 (`from_config`) - change how `technique_selected` is built

## Reuse

- `PartOption::Full { default, .. }` field already present from ST1 (`src/data.rs` line 14)
- `cfg.entries` already available in `from_config` (`src/sections/block_select.rs` line 13)

## Steps

1. Add `default_selected()` to the `PartOption` impl block in `src/data.rs`, after the existing `option_id()` method (around line 38):

```
-    pub fn option_id(&self) -> Option<&str> {
-        match self {
-            Self::Full { id, .. } => Some(id.as_str()),
-            _ => None,
-        }
-    }
-}
+    pub fn option_id(&self) -> Option<&str> {
+        match self {
+            Self::Full { id, .. } => Some(id.as_str()),
+            _ => None,
+        }
+    }
+    pub fn default_selected(&self) -> bool {
+        match self {
+            Self::Full { default, .. } => *default,
+            _ => true,
+        }
+    }
+}
```

2. Update `RegionState::from_config` in `src/sections/block_select.rs` to derive `technique_selected` from entry defaults:

```
-        let technique_selected = vec![false; cfg.entries.len()];
+        let technique_selected = cfg.entries.iter().map(|e| e.default_selected()).collect();
```

3. Run the failing tests to confirm they now pass:

```
cargo test tests_st3_default_selected
```

## Verification

### Manual tests

None required - behavior is fully covered by the automated test suite in this sub-task.

### Automated tests

Run the existing test module that was already written to specify this behavior:

```
cargo test tests_st3_default_selected
```

Expected: all six tests pass (`part_option_default_selected_full_true`, `part_option_default_selected_full_false`, `part_option_default_selected_simple`, `region_state_all_default_true_starts_all_selected`, `region_state_one_default_false_starts_unselected`, `block_select_state_new_propagates_defaults`).

Also run the full test suite to confirm no regressions:

```
cargo test
```

## Progress
- Step 1: Added `default_selected()` method to `PartOption` impl block in `src/data.rs`
- Step 2: Updated `RegionState::from_config` in `src/sections/block_select.rs` to use `cfg.entries.iter().map(|e| e.default_selected()).collect()`
- Step 3: Ran `cargo test tests_st3_default_selected` - all 6 tests pass; full suite 98/98 pass

## Implementation
Complete - 2026-04-02

## Changelog

### Review - 2026-04-02
- #1: Fixed test count in Verification section from "four" to "six" (six test names were listed)
