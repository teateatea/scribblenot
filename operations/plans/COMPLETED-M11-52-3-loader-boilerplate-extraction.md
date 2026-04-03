---
Task: #52 — Extract hard-coded boilerplate strings from note.rs into editable YML data files
Status: Draft
---

## Context

Sub-tasks 52.1 and 52.2 added the `FlatBlock::Boilerplate` variant and a `boilerplate.yml` data file. Sub-task 52.3 closes the loop by making `load_data_dir` recognise Boilerplate blocks in the flat YAML pool and surface them on `AppData` as a `HashMap<String, String>` keyed by id. Without this field, the rest of the app cannot retrieve boilerplate text at runtime. Four tests in `boilerplate_load_tests` currently fail to compile (field missing) or fail at runtime (logic missing).

## Approach

Add `boilerplate_texts: HashMap<String, String>` to `AppData`, then in `load_data_dir` iterate the pool after the existing duplicate/cycle checks, collect every `FlatBlock::Boilerplate` block into the map, and return an error if two blocks share the same id. Populate the field in the `Ok(AppData { ... })` return expression.

## Critical Files

- `src/data.rs`
  - Line 206-214: `AppData` struct definition
  - Line 598-757: `load_data_dir` function — pool collection, duplicate check, reconstruction pass, final `Ok(AppData { ... })`
  - Lines 1560-1666: `boilerplate_load_tests` module (four failing tests)

## Reuse

- `crate::flat_file::FlatBlock::Boilerplate { id, text }` — already defined in `src/flat_file.rs` line 38-41, no new types needed.
- `std::collections::HashMap` — already imported at line 3.
- Existing error-return pattern: `return Err(format!(...))` used throughout `load_data_dir`.

## Steps

1. Add the field to the `AppData` struct (line 213, after `block_select_data`):

```diff
     pub block_select_data: HashMap<String, Vec<BlockSelectEntry>>,
+    pub boilerplate_texts: HashMap<String, String>,
     pub keybindings: KeyBindings,
```

2. After the reconstruction pass in `load_data_dir` (after line 746, before the final `Ok`), add boilerplate extraction. Duplicate boilerplate ids are already caught by the general `(type_tag, id)` duplicate check at lines 626-636, so no second duplicate check is needed here:

```diff
+    // Boilerplate extraction: collect id -> text.
+    // Duplicate ids are already rejected by the general duplicate check above.
+    let mut boilerplate_texts: HashMap<String, String> = HashMap::new();
+    for block in &pool {
+        if let crate::flat_file::FlatBlock::Boilerplate { id, text } = block {
+            boilerplate_texts.insert(id.clone(), text.clone());
+        }
+    }
+
     Ok(AppData {
         groups,
         sections: all_sections,
         list_data: HashMap::new(),
         checklist_data: HashMap::new(),
         block_select_data: HashMap::new(),
+        boilerplate_texts,
         keybindings: KeyBindings::default(),
         data_dir: path.to_path_buf(),
     })
```

3. Fix all call sites that construct `AppData` directly by adding the new field. Grep for `AppData {` across all of `src/` (not just `src/data.rs`) before implementing — there are two additional constructors in `src/app.rs` (lines 1428 and 1460, inside test functions). Both must have `boilerplate_texts: Default::default()` added alongside the other `Default::default()` fields.

## Verification

### Manual tests

- Run `cargo build` and confirm it compiles with zero errors.
- Launch the app and open a note that displays boilerplate text; confirm the text appears correctly.

### Automated tests

- Run `cargo test boilerplate_load` to execute the four tests in `boilerplate_load_tests`:
  - `app_data_has_boilerplate_texts_field` — compile-time check, passes once the field is added.
  - `boilerplate_texts_contains_treatment_plan_disclaimer` — verifies a single Boilerplate block is loaded and its text matches expected content.
  - `boilerplate_texts_contains_informed_consent` — same for a second block.
  - `load_data_dir_errors_on_duplicate_boilerplate_id` — verifies the error path and that the error message contains the duplicate id.
- All four must pass with no regressions in the existing test suite (`cargo test`).

## Prefect-1 Report

### Nit

- **N1** (`M11-52-3-loader-boilerplate-extraction.md:40-52`): Step 2's extraction loop included a redundant `if boilerplate_texts.contains_key(id)` duplicate check. The general `(type_tag, id)` duplicate check at `data.rs:626-636` already catches duplicate boilerplate ids before this loop runs, making the branch dead code. Removed the dead branch; added a clarifying comment that duplicates are handled upstream. All four tests still pass because `load_data_dir_errors_on_duplicate_boilerplate_id` only asserts `err_msg.contains("duplicate_bp")`, which the existing general check's error message satisfies.

## Changelog

### Review - 2026-04-02
- #1 (blocking): Step 3 grep was scoped to `src/data.rs` only; expanded to `src/` and added explicit instruction to update the two `AppData` constructors in `src/app.rs` (lines 1428 and 1460) with `boilerplate_texts: Default::default()`.

### Prefect-1 - 2026-04-02
- N1: Removed dead duplicate-check branch from Step 2 extraction loop; added comment that upstream general check handles duplicates.

## Progress

- Step 1: Added `boilerplate_texts: HashMap<String, String>` field to `AppData` struct in data.rs
- Step 2: Added boilerplate extraction loop before `Ok(AppData {...})` and included `boilerplate_texts` in the return expression
- Step 3: Added `boilerplate_texts: Default::default()` to both `AppData` constructors in app.rs (lines 1428 and 1460), and `boilerplate_texts: base.boilerplate_texts` to AppData::load in data.rs (line 287)

## Implementation
Complete - 2026-04-02
