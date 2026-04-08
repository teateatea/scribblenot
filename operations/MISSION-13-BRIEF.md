# Project Foundation

## Goals
Implement the canonical 6-level YAML data hierarchy (Template > Group > Section > Field > List > Item) in the scribblenot codebase. This replaces the flat-block loader and FlatFile/FlatBlock types with a new HierarchyFile parsing layer and typed ID registry, while preserving byte-for-byte identical runtime behavior. All existing data YAML files are migrated to the new schema; the public AppData::load signature is unchanged.

In plain English: change how the app reads and validates note-definition YAML, but do not change what the user sees or what note text gets produced.

## Requirements
- Introduce six Rust hierarchy structs (HierarchyTemplate, HierarchyGroup, HierarchySection, HierarchyField, HierarchyList, HierarchyItem) as YAML-layer types in src/data.rs
- Implement load_hierarchy_dir with typed (TypeTag, id) registry, duplicate detection, cycle detection, and template cardinality validation (exactly one template required)
- Migrate all data YAML files: sections.yml, tx_regions.yml, boilerplate.yml, objective_findings.yml, remedial.yml, infection_control.yml to HierarchyFile format (tx_mods.yml was already deleted by task #50)
- Implement hierarchy_to_runtime shim that maps hierarchy structs into existing SectionGroup / SectionConfig / HeaderFieldConfig runtime model
- Wire load_hierarchy_dir into AppData::load, remove load_data_dir and flat_file.rs, update block_select.rs to accept HierarchyList
- Rewrite all ~20 existing load_data_dir tests to use load_hierarchy_dir with hierarchy-format YAML fixtures
- All cargo tests pass; manual note output is byte-for-byte identical to pre-migration baseline

## Recommended Implementation Sequence
1. Add the hierarchy-layer structs and parser/validator first, without changing AppData::load yet.
2. Build and compile the hierarchy_to_runtime shim next, keeping the existing runtime structs untouched.
3. Migrate the data/*.yml files only after the new loader can parse and validate representative fixtures.
4. Swap AppData::load over to load_hierarchy_dir only when the shim and migrated data compile together.
5. Remove flat_file.rs and the old test fixtures last, after the real data directory and test suite are green.

This order matters because it keeps the risky pieces isolated: parser correctness first, runtime compatibility second, file migration third, cleanup last.

## Task Priority Order
- #70 - Implement canonical 6-level YAML data hierarchy

## Explicit Non-Goals
- No changes to Boxes (UI layer: map, wizard)
- No multi-template switching support; only one active template
- No migration of date_prefix to a canonical spec field; it stays as a legacy carry-through field
- No repeat_limit at Section or Group level
- No skip-level referencing (e.g. Group directly referencing a Field)
- No coordinated rename of SectionConfig.map_label to SectionConfig.nav_label in runtime Rust structs or consumers (ui.rs, note.rs); deferred to follow-up
- No top-level items: key in HierarchyFile; items are not referenceable across files
- No section-level lists: for list_select or checklist sections; data_file: stays for those types
- No changes to copy/paste behavior
- No fix for the separate list-select add-entry persistence mismatch already tracked in roadmap item #2
- free_text remains a section_type value, not a structural concept

## Constraints
- The old runtime structs (SectionGroup, SectionConfig, HeaderFieldConfig) must not be renamed or deleted; the thin-shim compatibility strategy is mandatory per Key Decision 1
- date_prefix must not be dropped; HierarchySection carries it as Option<bool> and the shim copies it into SectionConfig.date_prefix (Key Decision 2)
- nav_label rename applies to YAML files only; SectionConfig.map_label retains its name in Rust for this mission; the loader accepts both nav_label and map_label, preferring nav_label (Key Decision 3)
- ID uniqueness is scoped per (TypeTag, id); the id_map lookup must also key on (TypeTag, id) to resolve the current bare-id conflict (Key Decision 4)
- tx_regions.yml must NOT define a sections: entry; section definition stays in sections.yml to avoid duplicate Section ID (Key Decision 6)
- data_file: dispatch stays for list_select and checklist sections; block_select dispatch is removed and replaced by hierarchy_to_runtime output keyed by section id (Key Decision 5)
- boilerplate.yml must be migrated to HierarchyFile format before flat_file.rs is removed (ST3.3 before ST4.4)
- hierarchy_to_runtime shim must exist and compile before the load_data_dir call is replaced in AppData::load (ST4.1 before ST4.2)
- Boilerplate IDs must be validated for uniqueness; existing error behavior at data.rs:1662 must be preserved
- Exactly one template must be defined across all merged YAML files; 0 or >1 is a loud load-time error
- The public AppData::load signature must not change

## Test Criteria
- cargo test passes with all existing tests rewritten to hierarchy-format YAML and all new unit/integration tests added in ST1 and ST2
- Integration test: load_hierarchy_dir against the actual data/ directory returns Ok with groups in expected order (intake, subjective, treatment, objective, post_tx)
- Regression test: SectionConfig built from migrated objective_section has date_prefix == Some(true)
- Regression test: tx_regions.yml parsed as HierarchyFile has back_lower_prone list with fascial_l4l5 item having default: Some(false)
- Integration test: hierarchy_to_runtime on loaded data/ produces block_select_data with key "tx_regions" containing at least one HierarchyList entry
- Manual verification: all five groups load without panic, note output is byte-for-byte identical to pre-migration baseline, no map_label: keys survive in data/ YAML files

## Exit Conditions
- Mission is implementation-ready only when the loader design, migration scope, and validation targets are internally consistent and do not depend on unrelated known issues.
- The main success bar is compatibility: same runtime structure, same rendered note text, stricter load-time validation.
- If any implementation step would force a broader runtime-model rewrite, stop and split that work into a follow-up instead of silently expanding mission scope.

## Coordination
- READY: true
- BEGIN: true
