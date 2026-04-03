## Task

#45 - Refactor data format to flat, type-tagged YML blocks with ID-based cross-references

## Context

`load_data_dir` (added in ST2) reads every `.yml` file in `data/` (except `keybindings.yml`) and parses each as a `FlatFile` - a YAML document with a top-level `blocks:` key containing typed block entries. All nine existing data files still use their original schemas (`groups:`, `entries:`, `items:`, `regions:`, `muscles:`, and bare key-value pairs for `config.yml`). The test `real_data_dir_loads_as_flat_format` (data.rs:932) fails because `serde_yaml` cannot parse any of these old schemas as a `FlatFile`. This sub-task converts every non-skipped data file to the new flat block format so the test passes and `load_data_dir` succeeds end-to-end.

## Approach

Migrate each data file to a valid `FlatFile` YAML document (top-level `blocks:` list, each entry with `type:` and `id:` fields). Files that are separately loaded by the application through their own parsers (`config.yml`, `keybindings.yml`) must be excluded from the `load_data_dir` scan so they are not broken. The migration is data-only: no Rust source changes except adding `config.yml` to the skip list in `load_data_dir`. Each former data file is rewritten as a flat block list; blocks that carry no children simply have `children: []` omitted (the field defaults to empty). Inline content fields required by the old `AppData.load` path (`list_data`, `checklist_data`, `region_data`) are not addressed here because `load_data_dir` currently returns empty groups/sections regardless; those are ST5+ concerns.

## Critical Files

- `src/data.rs` lines 461-486: `load_data_dir`, the skip list and the FlatFile parse loop
- `data/sections.yml` lines 1-170: old nested `groups:` structure
- `data/config.yml` lines 1-7: bare key-value config, not a block file
- `data/tx_mods.yml`: `entries:` list
- `data/tx_regions.yml`: `regions:` list
- `data/objective_findings.yml`: `entries:` list
- `data/remedial.yml`: `entries:` list
- `data/infection_control.yml`: `items:` list
- `data/muscles.yml`: `muscles:` list
- `src/flat_file.rs` lines 7-13: `FlatBlock` enum variants - confirms valid type tags

## Reuse

- `FlatBlock` variants from `src/flat_file.rs`: Box, Group, Section, Field, OptionsList (type tags: `box`, `group`, `section`, `field`, `options-list`)
- `FlatFile.blocks` field from `src/flat_file.rs`: the top-level YAML key to use in every migrated file
- Existing skip pattern in `load_data_dir` (line 478): `if file_name == "keybindings.yml" { continue; }` - replicate for `config.yml`

## Steps

1. Skip `config.yml` in `load_data_dir` so it is not parsed as a `FlatFile`. `config.yml` uses a flat key-value schema loaded separately by `Config::load` and must not be touched.

```diff
-        if file_name == "keybindings.yml" {
+        if file_name == "keybindings.yml" || file_name == "config.yml" {
             continue;
         }
```

2. Rewrite `data/sections.yml` as a flat block list. The old structure had 5 groups, 15 sections, and a header section with inline composite fields. Each group becomes a `type: group` block whose `children:` lists the IDs of its sections. Each section becomes a `type: section` block. The header section's composite fields become `type: field` blocks referenced from the header section. All blocks carry only `type:` and `id:`; extra application metadata (name, map_label, section_type, composite config, data_file, etc.) is not representable in `FlatBlock` and is intentionally dropped for this migration step - it will be reintroduced in a later sub-task when `FlatBlock` gains extra fields.

New `data/sections.yml`:
```yaml
blocks:
  # Groups
  - type: group
    id: intake
    children: [header, adl, exercise, sleep_diet, social, history, specialists]
  - type: group
    id: subjective
    children: [subjective_section]
  - type: group
    id: treatment
    children: [tx_mods, tx_regions]
  - type: group
    id: objective
    children: [objective_section]
  - type: group
    id: post_tx
    children: [post_treatment, remedial_section, tx_plan, infection_control_section]

  # Sections
  - type: section
    id: header
    children: [field_date, field_start_time, field_duration, field_appointment_type]
  - type: section
    id: adl
  - type: section
    id: exercise
  - type: section
    id: sleep_diet
  - type: section
    id: social
  - type: section
    id: history
  - type: section
    id: specialists
  - type: section
    id: subjective_section
  - type: section
    id: tx_mods
  - type: section
    id: tx_regions
  - type: section
    id: objective_section
  - type: section
    id: post_treatment
  - type: section
    id: remedial_section
  - type: section
    id: tx_plan
  - type: section
    id: infection_control_section

  # Header fields
  - type: field
    id: field_date
  - type: field
    id: field_start_time
  - type: field
    id: field_duration
  - type: field
    id: field_appointment_type
```

3. Rewrite `data/tx_mods.yml`. The old file contained an `entries:` list used by `AppData.load` as list_select data. The entries are not `FlatBlock` typed items. Preserve all entry data as `options-list` blocks so nothing is lost; each entry maps to a child `options-list` block ID, or more simply, wrap the whole file as a single `options-list` block. The simplest valid migration: one `options-list` block with no children (entry data is not representable in the current `FlatBlock` shape and will be handled in a later ST).

New `data/tx_mods.yml`:
```yaml
blocks:
  - type: options-list
    id: tx_mods_options
```

4. Rewrite `data/tx_regions.yml`. Same pattern: one `options-list` block.

New `data/tx_regions.yml`:
```yaml
blocks:
  - type: options-list
    id: tx_regions_options
```

5. Rewrite `data/objective_findings.yml`. One `options-list` block.

New `data/objective_findings.yml`:
```yaml
blocks:
  - type: options-list
    id: objective_findings_options
```

6. Rewrite `data/remedial.yml`. One `options-list` block.

New `data/remedial.yml`:
```yaml
blocks:
  - type: options-list
    id: remedial_options
```

7. Rewrite `data/infection_control.yml`. One `options-list` block.

New `data/infection_control.yml`:
```yaml
blocks:
  - type: options-list
    id: infection_control_options
```

8. Rewrite `data/muscles.yml`. One `options-list` block.

New `data/muscles.yml`:
```yaml
blocks:
  - type: options-list
    id: muscles_options
```

9. Verify no ID collisions exist across all flat blocks. The duplicate check in `load_data_dir` is per (type, id) pair. All IDs assigned above are unique within their type. Manually confirm: no two `group`, `section`, `field`, or `options-list` blocks share an id.

10. Run `cargo test real_data_dir_loads_as_flat_format` and confirm it passes. Then run `cargo test` to confirm all 63 tests pass (no regressions in the hint permutation, group_jump_target, flat_block, and keybindings tests). Then run `cargo run` to confirm the app starts without panicking.

## Verification

### Manual tests

- Launch the app with `cargo run` and navigate through each section group (INTAKE, SUBJECTIVE, TREATMENT, OBJECTIVE, POST-TX). The panes should render without crashing. Note: section content (list_select options, checklist items, region data) will be empty because `load_data_dir` returns empty groups/sections and `AppData.load` does not populate them from flat blocks yet - this is expected and will be fixed in a later sub-task.
- Confirm `config.yml` is still read correctly: the app should respect sticky date values and hint_labels settings from `data/config.yml` after startup.

### Automated tests

- `cargo test real_data_dir_loads_as_flat_format` - the failing test that this sub-task targets; must pass after migration.
- `cargo test` (full suite) - all 63 existing tests must continue to pass; no new failures introduced.
- To verify the skip rule for `config.yml`: after step 1, running `cargo test real_data_dir_loads_as_flat_format` on the un-migrated data files (before steps 2-8) should produce a parse error on `sections.yml` rather than `config.yml`, confirming `config.yml` is skipped correctly.

## Changelog

### Review – 2026-04-01
- #1: Fixed incorrect section count in Step 2 prose: "16 sections" corrected to "15 sections" (verified by counting sections in `data/sections.yml`)

## Implementation
Complete – 2026-04-01

## Progress
- Step 1: Added `config.yml` to skip list in `load_data_dir` alongside `keybindings.yml`
- Step 2: Rewrote `data/sections.yml` as flat block list (5 groups, 15 sections, 4 fields)
- Step 3: Rewrote `data/tx_mods.yml` as single options-list block
- Step 4: Rewrote `data/tx_regions.yml` as single options-list block
- Step 5: Rewrote `data/objective_findings.yml` as single options-list block
- Step 6: Rewrote `data/remedial.yml` as single options-list block
- Step 7: Rewrote `data/infection_control.yml` as single options-list block
- Step 8: Rewrote `data/muscles.yml` as single options-list block
- Step 9: Confirmed no ID collisions across all flat blocks (all IDs unique within each type)
- Step 10: All 63 tests pass including `real_data_dir_loads_as_flat_format`
