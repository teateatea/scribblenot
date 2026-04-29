# Plan: `src/data.rs` Hotspot Split

**Date:** 2026-04-24
**Status:** Proposed
**Goal:** Reduce merge pressure in `src/data.rs` without changing public behavior.

## Plain-English Summary
`src/data.rs` is doing too many jobs at once.

That matters for two reasons:

- almost any schema, validation, diagnostic, or runtime-tree branch ends up touching it
- once two branches both edit it, the merge gets harder than the actual feature work

The safest fix is not one giant rewrite. The safest fix is a staged split where `src/data.rs` stays as a thin public facade and the internal responsibilities move into explicitly named helper files one slice at a time.

## Decision: Naming Shape

### Option 1: Convert to `src/data/mod.rs`
Use a directory-style module layout:

- `src/data/mod.rs`
- `src/data/model.rs`
- `src/data/load.rs`
- `src/data/source.rs`

**Pros**
- standard Rust shape
- no special module-path handling

**Cons**
- adds another `mod.rs` tab
- easier to open the wrong facade file during parallel work
- slightly larger rename on the first split

### Option 2: Keep `src/data.rs` and add unique sibling files
Use explicit filenames:

- `src/data.rs`
- `src/data_model.rs`
- `src/data_load.rs`
- `src/data_source.rs`
- `src/data_validate.rs`
- `src/data_runtime.rs`
- `src/data_hints.rs`

**Pros**
- filenames stay obvious in tabs, ripgrep, and merge views
- smaller first refactor
- keeps `crate::data::*` stable for callers

**Cons**
- slightly less conventional
- needs disciplined module wiring

## Recommendation
Choose **Option 2**.

This repo already has several directory modules, and the user concern is valid: too many `mod.rs` facades are easy to mix up. The safer near-term move is explicit filenames plus a stable `src/data.rs` facade.

## Target Shape

### Keep
- `src/data.rs`

### Add
- `src/data_model.rs`
- `src/data_load.rs`
- `src/data_source.rs`
- `src/data_validate.rs`
- `src/data_runtime.rs`
- `src/data_hints.rs`

## Ownership Boundaries

### `src/data.rs`
Owns:

- public exports
- `AppData` entrypoints
- top-level orchestration glue between loading, validation, and runtime build

Should not grow new parsing, validation, or hint logic once the split starts.

### `src/data_model.rs`
Owns:

- authored config structs
- runtime structs/enums
- simple impls on those types
- serde defaults and lightweight helpers tied directly to the types

Examples from current `src/data.rs`:

- `HeaderFieldConfig`
- `SectionConfig`
- `HierarchyList`
- `HierarchyItem`
- `HierarchyTemplate`
- `RuntimeTemplate`
- `SectionBodyMode`
- `JoinerStyle`

### `src/data_hints.rs`
Owns:

- hint-label generation
- hint normalization
- permutation helpers
- authored hotkey assignment helpers that are mostly independent of hierarchy validation

Examples from current `src/data.rs`:

- `resolve_hint`
- `assign_hint_labels`
- `generate_hint_permutations`
- `ensure_hint_permutations`
- `combined_hints`

### `src/data_source.rs`
Owns:

- source-index types
- YAML document/source anchor mapping
- line/entry location helpers
- authored-context lookup helpers

Examples from current `src/data.rs`:

- `SourceIndex`
- `SourceNode`
- `build_source_index`
- `top_level_block_range`
- `find_mapping_anchor`
- `collect_top_level_entry_anchors`
- `collect_child_ref_anchors`

### `src/data_load.rs`
Owns:

- directory scanning
- YAML document splitting
- parse-to-merged-hierarchy flow
- top-level file merge rules

Examples from current `src/data.rs`:

- `read_hierarchy_dir`
- `load_hierarchy_dir`
- `parse_hierarchy_file_documents`
- `split_yaml_documents`
- `yaml_doc_error`
- `authored_yaml_doc_error`

### `src/data_runtime.rs`
Owns:

- hierarchy-to-runtime transformation
- flattening and navigation helpers
- child resolution and body-mode inference

Examples from current `src/data.rs`:

- `hierarchy_to_runtime`
- `section_to_config`
- `collection_to_config`
- `resolve_collection`
- `resolve_field`
- `infer_body_mode`
- `flat_sections_from_template`
- `runtime_navigation`

### `src/data_validate.rs`
Owns:

- hierarchy validation
- wrong-kind / missing-ref / duplicate-id reporting
- author-facing fix hints
- strict-key and authored-shape validation helpers

Examples from current `src/data.rs`:

- `validate_data_dir`
- `validate_merged_hierarchy`
- `validate_children`
- `validate_child_exists`
- `route_wrong_kind_error`
- `unsupported_authored_key_report`
- `missing_required_authored_key_report`

## Extraction Order
The order matters. The goal is to move the least-coupled code first and leave the most cross-connected validation code for later.

### Slice 0: Facade setup only
Create the new helper files and wire them into `src/data.rs` without moving much logic yet.

Done when:

- the crate still compiles
- `src/data.rs` can re-export from helper modules
- there is no behavior change

Suggested ownership sentence:

`This branch owns internal data-module scaffolding only. It should not change loader behavior, validation behavior, or public APIs.`

### Slice 1: Move pure data model types
Move the structs/enums/default helpers that mostly define data shape.

Why first:

- lowest behavioral risk
- creates shared vocabulary for every later extraction

Move first:

- config/runtime types
- serde enums and defaults
- lightweight inherent impls tied directly to those types

Do not move yet:

- validation functions
- source index logic
- parse/report helpers

### Slice 2: Move hint generation
Move the bottom-of-file hint logic into `src/data_hints.rs`.

Why second:

- largely isolated
- easy to validate
- immediately removes a noticeable chunk from the hotspot

Watch for:

- imports of `KeyBindings`
- any hidden dependency on validation helpers

### Slice 3: Move source-index and anchor helpers
Move source mapping and location helpers into `src/data_source.rs`.

Why third:

- still fairly cohesive
- becomes a clean dependency for both loading and validation

Watch for:

- private helper visibility
- circular references between parse code and authored-context helpers

### Slice 4: Move YAML load/merge flow
Move directory scanning and multi-file merge logic into `src/data_load.rs`.

Why fourth:

- by now, model and source helpers already exist
- loader code can depend on them cleanly

Keep in `src/data.rs` if simpler:

- `AppData::load()` as a thin facade
- tiny orchestration wrappers that call load + validate + runtime build

### Slice 5: Move runtime conversion
Move hierarchy resolution and runtime-build helpers into `src/data_runtime.rs`.

Why fifth:

- this is a large chunk, but conceptually coherent once model types are already moved
- many feature branches want runtime changes without loader/parser edits

Watch for:

- any validation logic mixed into runtime conversion
- helper functions that report authored-source errors and may belong in validation instead

### Slice 6: Move validation and author-facing reporting
Move the remaining validation/reporting logic into `src/data_validate.rs`.

Why last:

- this is the most interconnected slice
- it touches model, source, load assumptions, and runtime assumptions

Success condition:

- `src/data.rs` reads as a facade, not a second giant implementation file

## Branch Plan

### Branch A: `refactor/data-split-scaffold`
Scope:

- facade setup
- helper file creation
- first re-exports

Target files:

- `src/data.rs`
- new empty or near-empty helper files

### Branch B: `refactor/data-model-and-hints`
Scope:

- move model types
- move hint helpers

Reason:

- these are the safest extractions and provide immediate payoff

### Branch C: `refactor/data-source-and-load`
Scope:

- move source-index helpers
- move YAML load/merge flow

Reason:

- these pieces already belong together operationally

### Branch D: `refactor/data-runtime`
Scope:

- move runtime conversion and flatten/navigation helpers

### Branch E: `refactor/data-validate`
Scope:

- move validation and fix-hint/reporting logic
- final facade cleanup

## Validation Plan

### After every slice
- run `cargo check --quiet`

### After slices 2, 4, 5, and 6
- run `cargo test --quiet`

### Manual checks at the end
- run `cargo run -- --validate-data`
- open the app and verify it still loads the existing authored data set without a new startup error

## Safety Rules

- no public behavior changes during the split
- no schema changes during the split
- no opportunistic cleanup in unrelated modules
- if a helper seems to belong in two files, keep it where current behavior is clearest and leave a follow-up note instead of forcing a perfect split

## Merge Strategy

- merge each slice as soon as it is coherent
- do not stack feature work on top of the refactor branches
- if a real feature branch needs `src/data.rs` mid-split, merge the smallest stable slice first rather than keeping the refactor open

## Done Condition
This plan is complete when:

- `src/data.rs` is mostly facade/orchestration code
- the new helper files have clear ownership
- new data-related branches can often touch `data_runtime`, `data_validate`, or `data_load` without all colliding in one giant file
