# Mission 14 Regression Tests

## Task #46: Neutralise block_select struct and key names
- [ ] `cargo build` passes with zero errors and zero new warnings after the rename
- [ ] The block_select section loads and behaves identically to pre-mission - no visible change to the user
- [ ] `grep` finds zero occurrences of `RegionsFile`, `RegionConfig`, `TechniqueConfig`, `regions:`, and `techniques:` across all source and data files

## Task #47: Add per-technique default selection state to block_select
- [ ] Techniques marked `default: false` in tx_regions.yml begin the session unselected
- [ ] Techniques with no `default` field (or `default: true`) begin the session selected, preserving prior behaviour
- [ ] `cargo build` passes with zero errors and zero new warnings

## Task #52: Extract hard-coded boilerplate strings from note.rs
- [ ] Generated note output is character-for-character identical to pre-mission output
- [ ] The boilerplate and informed consent string literals are gone from Rust source; note.rs looks them up by ID from the data loader
- [ ] `cargo build` passes and the app starts without errors when the boilerplate YML file is present in data/

## Task #49: Add repeat_limit: N to multi_field fields
- [ ] After N confirmations on a repeat_limit field, the wizard advances normally even if the key is held (cap enforced)
- [ ] Repeat fields appear as additional optional boxes in the wizard - the user can skip re-entry at any point; repeat is not forced

## Task #48: Generalize multi_field note rendering
- [ ] The generated note for an appointment header session is identical to pre-mission output
- [ ] A multi_field section other than the appointment header produces note output without crashing
- [ ] `cargo build` passes with zero errors and zero new warnings

## Task #50: Convert tx_mods section to multi_field
- [ ] All five fields (Pressure, Challenge, Mood, Communication, Modifications) appear as distinct wizard steps with the correct options
- [ ] `data/tx_mods.yml` is deleted and no source or data file references it
- [ ] The generated tx_mods section of the note contains the selected values from all five fields

## Task #51: Move hard-coded section metadata into sections.yml
- [ ] `heading_search_text()`, `is_intake_section()`, `intake_heading()`, and all `cfg.id ==` string literal comparisons are gone from Rust source
- [ ] Generated note output is identical to pre-mission output for all section types
- [ ] `cargo build` passes with zero errors and zero new warnings

## Task #45: Refactor data format to flat, type-tagged YML blocks with ID-based cross-references
- [ ] `cargo run` succeeds and the app behaves exactly as before with the new data format
- [ ] Missing ID references, duplicate ID+type combinations, and circular references all produce loud errors at load time (no silent failures)
- [ ] `sections.yml` (and any split files in `data/`) uses the new flat type-tagged format with ID-based cross-references

## Task #23: Auto-generate multi-character hint permutations
- [ ] `hint_permutations:` is written to keybindings.yml after first run and is adjacency-ordered (adjacent pairs first, e.g. `qq, qw, wq, ww` before `qp`)
- [ ] Restarting the app without editing `hints:` does not rewrite `hint_permutations:`
- [ ] When the base hint pool is smaller than the number of sections needed, permutation hints are used automatically for the overflow sections (not the base hints pool repeated or left blank)

## Task #22: Multi-character hint sequences with progressive prefix filtering
- [ ] Typing the first character of a multi-char hint highlights the matching prefix in white and grays out non-matching hints
- [ ] Typing a character that matches no remaining hint resets all hints to active magenta state
- [ ] A character that is a proper prefix of an existing multi-char hint cannot be assigned as a standalone hint (enforced at config validation or assignment time)

## Task #21: Persistent group-jump hotkeys in map column
- [ ] Pressing Q/W/F (and other reserved group keys) from any position in the map column jumps to the first section of the correct group
- [ ] Group-reserved keys (Q, W, F, etc.) are never assigned as section hints
- [ ] `cargo build` passes with no warnings after implementation

## Task #2: Add Shift+Enter super-confirm keybinding
- [ ] Shift+Enter on a field with a visible value (typed, sticky, or default) confirms it and advances the wizard normally
- [ ] Shift+Enter on an empty field (no typed value, no sticky, no default) skips it silently without error
- [ ] `super_confirm:` field is present in keybindings.yml; rebinding it to a different key changes which key triggers super-confirm

## Task #70: Implement canonical 6-level YAML data hierarchy
- [ ] cargo test passes with all tests rewritten to hierarchy-format YAML and all new ST1/ST2 unit tests added
- [ ] load_hierarchy_dir against actual data/ directory returns Ok with groups in correct order (intake, subjective, treatment, objective, post_tx)
- [ ] SectionConfig from migrated objective_section has date_prefix == Some(true)
- [ ] tx_regions.yml parsed as HierarchyFile has back_lower_prone list with fascial_l4l5 item having default: Some(false)
- [ ] hierarchy_to_runtime produces block_select_data with key "tx_regions" containing at least one HierarchyList entry
- [ ] src/flat_file.rs is deleted and mod flat_file removed from src/main.rs
- [ ] No map_label: keys survive in any data/*.yml file (all renamed to nav_label:)
- [ ] All data YAML files (sections.yml, tx_regions.yml, boilerplate.yml, objective_findings.yml, remedial.yml, infection_control.yml) parse as HierarchyFile without error
- [ ] cargo build compiles cleanly with no warnings from new code
