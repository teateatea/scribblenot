## Task

#50 - Convert tx_mods section to multi_field with 5 categorized fields, removing tx_mods.yml

## Context

The `tx_mods` section currently uses `section_type: list_select` backed by `data/tx_mods.yml`. The user wants it restructured as a `multi_field` section with five named fields (Pressure, Challenge, Mood, Communication, Modifications), each with its own inline option list. This eliminates the external data file and groups options into meaningful categories. The Modifications field must support `repeat_limit: 10` to allow multiple modification entries.

## Approach

Inline all 36 tx_mods.yml entries directly into `data/sections.yml` as five child Field blocks under a restructured `tx_mods` section. Add `repeat_limit` to `FlatBlock::Field` in flat_file.rs and thread it through the loader in data.rs. The external `data/tx_mods.yml` file is no longer needed and must be deleted (see Step 6). It is NOT in the `load_data_dir` skip list - leaving it in place would cause its `OptionsList` block to be parsed and added to the pool (harmless, but the mission requires deletion).

## Critical Files

- `data/sections.yml` - lines 75-80: the tx_mods section block to rewrite; also add five new Field blocks
- `src/flat_file.rs` - lines 25-32: `FlatBlock::Field` variant, needs `repeat_limit` field
- `src/data.rs` - lines 707-724: loader reconstruction pass that builds `HeaderFieldConfig` from Field blocks, currently hardcodes `repeat_limit: None`
- `src/flat_file.rs` - lines 88-95 and 139-147: two test functions (`flat_block_field_variant_has_id` at line 90, `flat_block_id_is_string` at line 142) both construct `Field` via struct literal and will need `repeat_limit: None` added

## Reuse

- Existing `FlatBlock::Field` deserialization via serde - just add the new optional field with `#[serde(default)]`
- Existing `HeaderFieldConfig.repeat_limit: Option<usize>` in data.rs line 90 - already present, just wire it up
- `FlatBlock::Field.options: Vec<String>` and `HeaderFieldConfig.options: Vec<String>` remain as-is; each option string serves as both the display label and the confirmed output value (identical to how `appointment_duration` uses `["30","45","60"]`). Do NOT introduce `{label, output}` dicts or change the type to `Vec<PartOption>` - the pre-written TDD tests at `src/data.rs:1812-1833` call `o.contains("PREGNANCY")` on `&String` and assume `Vec<String>` throughout.

## Steps

1. **Add `repeat_limit` to `FlatBlock::Field` in `src/flat_file.rs`**

   ```diff
    Field {
        id: String,
        #[serde(default)] children: Vec<String>,
        #[serde(default)] name: Option<String>,
        #[serde(default)] options: Vec<String>,
        #[serde(default)] composite: Option<CompositeConfig>,
        #[serde(default)] default: Option<String>,
   +    #[serde(default)] repeat_limit: Option<usize>,
    },
   ```

2. **Fix the `FlatBlock::Field` unit test struct literal** in `src/flat_file.rs` (both struct literal sites that construct `Field` directly must add `repeat_limit: None` - lines 90 and 142; the match arm `FlatBlock::Field { id, .. }` uses `..` and does not need updating):

   Each affected test line looks like:
   ```diff
   - FlatBlock::Field { id: "fld1".to_string(), children: vec![], name: None, options: vec![], composite: None, default: None }
   + FlatBlock::Field { id: "fld1".to_string(), children: vec![], name: None, options: vec![], composite: None, default: None, repeat_limit: None }
   ```
   Apply the same change to every `Field { ... }` literal in the test block.

3. **Thread `repeat_limit` through the loader in `src/data.rs`**

   In the reconstruction pass (around line 711-722), update the `Field` arm match to extract `repeat_limit` and pass it to `HeaderFieldConfig`:

   ```diff
    if let crate::flat_file::FlatBlock::Field {
        id: field_id, name: field_name,
   -    options, composite, default, ..
   +    options, composite, default, repeat_limit, ..
    } = &pool[fidx] {
        hfields.push(HeaderFieldConfig {
            id: field_id.clone(),
            name: field_name.clone().unwrap_or_default(),
            options: options.clone(),
            composite: composite.clone(),
            default: default.clone(),
   -        repeat_limit: None,
   +        repeat_limit: *repeat_limit,
        });
    }
   ```

4. **Rewrite the `tx_mods` section block in `data/sections.yml`**

   Replace lines 76-80:
   ```diff
   - - type: section
   -   id: tx_mods
   -   name: "Treatment Modifications & Preferences"
   -   map_label: "Tx Mods"
   -   section_type: list_select
   -   data_file: "tx_mods.yml"
   + - type: section
   +   id: tx_mods
   +   name: "Treatment Modifications & Preferences"
   +   map_label: "Tx Mods"
   +   section_type: multi_field
   +   children: [pressure, challenge, mood, communication, modifications]
   ```

   Note: the field block IDs are unprefixed (`pressure`, `challenge`, `mood`, `communication`, `modifications`). The pre-written TDD test at `src/data.rs:1797` requires `f.id == "modifications"` (no `tx_mods_` prefix), and the children list must reference the same IDs used in the field block definitions in Step 5.

5. **Add five Field blocks to `data/sections.yml`** at the bottom of the file, after the existing `# Header fields` section (i.e., after the final `appointment_type` field block, line ~208), under a new `# Tx Mods fields` comment. This follows the established convention of grouping field blocks together at the bottom of sections.yml, separate from section blocks. Use the categorized options below. Each option is a plain YAML string that serves as both the selectable label and the note output value. All option strings must be verbatim from the `output:` values in `data/tx_mods.yml` (i.e., the full `- PREFIX: description` form). Use double-quoted YAML strings for any value containing an apostrophe (e.g., `RAYNAUD'S`). Do NOT use `{label, output}` dict notation - `FlatBlock::Field.options` is `Vec<String>`.

   **pressure** (Pressure field - 6 options):
   ```yaml
   - type: field
     id: pressure
     name: "Pressure"
     options:
       - "- LIGHT PRESSURE: Pt prefers much less than Mr. Gormley's usual working pressure."
       - "- MODERATE PRESSURE: Pt prefers slightly less than Mr. Gormley's usual working pressure."
       - "- REGULAR PRESSURE: Pt is comfortable with Mr. Gormley's usual working pressure."
       - "- FIRM PRESSURE: Pt prefers slightly more than Mr. Gormley's usual working pressure."
       - "- HEAVY PRESSURE: Pt prefers much more than Mr. Gormley's usual pressure."
       - "- FULL PRESSURE: Pt prefers the maximum pressure that Mr. Gormley can apply safely. Consider lowering the massage table."
   ```

   **challenge** (Challenge field - 6 options):
   ```yaml
   - type: field
     id: challenge
     name: "Challenge"
     options:
       - "- VERY GENTLE TREATMENT: Pt prefers a gentle treatment: Avoid all challenging techniques."
       - "- GENTLE TREATMENT: Pt prefers a gentle treatment: Avoid challenging techniques."
       - "- RELAXED TREATMENT: Pt prefers a relaxed treatment, but may tolerate some challenge."
       - "- COMFORTABLE TREATMENT: Pt prefers a challenge only when necessary for treatment."
       - "- STRONG TREATMENT: Pt prefers a strong treatment, with some challenge expected."
       - "- CHALLENGING TREATMENT: Pt prefers a challenging treatment, with pressure that approaches discomfort."
   ```

   **mood** (Mood field - 4 options):
   ```yaml
   - type: field
     id: mood
     name: "Mood"
     options:
       - "- CALMING: Pt responds well to slow-paced techniques with limited conversation.\n- - Give time to breathe and feel muscles relax, without distraction!"
       - "- RELAXING: Pt responds well to limited or casual conversation."
       - "- CONVERSATIONAL: Pt responds well to a social atmosphere during treatment.\n- - Remember to interrupt conversation for check ins; allow time for quiet."
       - "- SOCIAL: Pt responds well to ongoing sociable conversation during treatment.\n- - If necessary, interrupt conversation to check in!"
   ```

   **communication** (Communication field - 7 options, including both STOIC entries):
   ```yaml
   - type: field
     id: communication
     name: "Communication"
     options:
       - "- CONCISE: Pt responds well to clear, direct language, with limited conversation."
       - "- INTROSPECTIVE: Pt responds well to limited conversation.\n- - Give space for slowing down without distraction."
       - "- STOIC: Pt may suppress their responses to pain or discomfort. Check in as needed."
       - "- STOIC: Pt responds well to frequent verbal check ins.\n- - Pt may suppress their responses to pain or discomfort."
       - "- CONTROLLED: Pt prefers a very specific amount of pressure.\n- - Check in frequently, even with each muscle change."
       - "- COMMUNICATIVE: Pt speaks up about their needs or discomfort. Check in regularly anyways!"
       - "- RESTFUL: Pt prefers resting during appointment. Limit conversation, focus on recovery."
   ```

   **modifications** (Modifications field - 13 options, repeat_limit: 10):
   ```yaml
   - type: field
     id: modifications
     name: "Modifications"
     repeat_limit: 10
     options:
       - "- PREGNANCY: Patient is treated in sidelying whenever possible, and pillowed under the head, in front of the chest, and between the knees.\n- - Patient may be treated in supine for up to 5 minutes.\n- - Lower table at the beginning and end of treatment."
       - "- POST-CONCUSSION: Patient is treated supine only, and pillowed under head and knees.\n- - Avoid using prone position.\n- - Avoid the neck.\n- - Dim lighting when possible.\n- - No music.\n- - No air filter fan."
       - "- VERTIGO: Encourage taking time to turn over slowly.\n- - Allow extra time to settle after changing positions before continuing treatment.\n- - Encourage pausing at each step when sitting up from the massage table."
       - "- LOW LIGHT: Pt prefers as lights as dimmed as possible."
       - "- NO PRONE: Do not put Pt in prone position."
       - "- SUPINE ONLY: Treat Pt only in supine position."
       - "- HEAD PILLOW: Pt accepts pillow for head while supine."
       - "- HEAD PILLOW REQUIRED: Pillow must be immediately available when supine."
       - "- LOWER TABLE: Lower massage table before & after treatment."
       - "- TALL: Recommend using height extender on massage table."
       - "- CLOTHED: Patient prefers to be clothed, using no oil during treatment."
       - "- RAYNAUD'S: Pt may be sensitive to cold."
       - "- TIMELY: Patient prefers finishing treatment strictly on schedule."
   ```

   Note: The `RAYNAUD'S` option string contains an apostrophe. In YAML, double-quoted strings handle apostrophes without any escaping (the apostrophe is a plain character inside `"..."`). Single-quoted YAML strings require the apostrophe to be doubled (`''`), which is error-prone. Use double-quotes as shown above: `"- RAYNAUD'S: Pt may be sensitive to cold."`.

6. **Delete `data/tx_mods.yml`**. The file is no longer referenced after Step 4 removes the `data_file: "tx_mods.yml"` line. Note: `tx_mods.yml` is NOT in the `load_data_dir` skip list (only `keybindings.yml`, `config.yml`, and `tx_regions.yml` are skipped). If the file is left in place, `load_data_dir` will parse it and add its `OptionsList` block to the pool — harmless but contrary to the mission brief requirement that "no reference to it remains in source or data files".

7. **Run `cargo test`** to confirm all tests pass, including the flat_file Field variant tests and any existing data loader tests.

## Verification

### Manual tests

- Launch the app and navigate to the TREATMENT group. The "Treatment Modifications & Preferences" section should now display five sub-fields (Pressure, Challenge, Mood, Communication, Modifications) instead of a flat list.
- Confirm the rendered order is: Pressure, then Challenge, then Mood, then Communication, then Modifications.
- Select a Pressure option and confirm the output string is written to the note verbatim (e.g., selecting the `- FIRM PRESSURE: Pt prefers slightly more than Mr. Gormley's usual working pressure.` entry produces that exact line in the note).
- Select both STOIC options in the Communication field and confirm they appear as distinct entries.
- Select `RAYNAUD'S` in the Modifications field and confirm it renders as `- RAYNAUD'S: Pt may be sensitive to cold.` with the apostrophe intact.
- On the Modifications field, confirm that it allows up to 10 repeat entries (repeat_limit: 10) before advancing.

### Automated tests

- `cargo test` - existing flat_file and data loader tests must all pass, including the four pre-written tests in `tx_mods_multi_field_tests` module
- Confirm `data/tx_mods.yml` no longer exists after Step 6
- ST50-1-TEST-3 (`tx_mods_modifications_field_has_repeat_limit_10`) exercises `repeat_limit` end-to-end via `load_data_dir` on the real data directory - no new test for `repeat_limit` needs to be written; confirm this test passes after implementation

## Changelog

### Review – 2026-04-03
- #1 (blocking): Replaced `{label, output}` dict approach in Step 5 YAML with plain output strings (`Vec<String>`); `FlatBlock::Field.options` and `HeaderFieldConfig.options` are both `Vec<String>` and the pre-written TDD tests call `o.contains("PREGNANCY")` on `&String` - changing to `Vec<PartOption>` would break these tests. Updated Reuse section to document this constraint explicitly.
- #2 (blocking): Changed all five field block IDs from `tx_mods_*` prefix to unprefixed (`pressure`, `challenge`, `mood`, `communication`, `modifications`); the pre-written test at `src/data.rs:1797` looks up `f.id == "modifications"` (no prefix). Updated Step 4 children list and all Step 5 field block IDs accordingly.
- #3 (minor): Fixed Step 5 Modifications field header from "12 options" to "13 options" (PREGNANCY, POST-CONCUSSION, VERTIGO, LOW LIGHT, NO PRONE, SUPINE ONLY, HEAD PILLOW, HEAD PILLOW REQUIRED, LOWER TABLE, TALL, CLOTHED, RAYNAUD'S, TIMELY = 13 entries).
- #4 (minor): Fixed Step 2 "all four occurrences" to "both struct literal sites" - only lines 90 and 142 construct `FlatBlock::Field` via struct literal; match arms use `..` and are not affected.
- #5 (minor): Added Step 6 to delete `data/tx_mods.yml`; corrected Approach which incorrectly claimed the file was in the `load_data_dir` skip list (only `keybindings.yml`, `config.yml`, and `tx_regions.yml` are skipped).
- #6 (nit): Fixed Step 4 diff block fencing - moved the Note outside the closing ` ``` ` so it renders as prose, not code.
- #7 (nit): Clarified RAYNAUD'S YAML quoting note - single-quoted strings require apostrophe doubling (`''`), making them error-prone; double-quoted strings are the correct and safe choice.

### Review – 2026-04-03
- #8 (nit): Corrected entry count in Approach from "35 tx_mods.yml entries" to "36" (6+6+4+7+13=36 verified against tx_mods.yml).
- #9 (nit): Updated Step 5 placement instruction to follow the established sections.yml convention - field blocks belong at the bottom after the existing `# Header fields` section, under a new `# Tx Mods fields` comment, not interleaved with section blocks.

### Review – 2026-04-03
- N3 (nit): Corrected Critical Files line reference for `data/sections.yml` from "lines 76-80" to "lines 75-80" — the tx_mods block opens at line 75 (`- type: section`).

### Prefect-1 – 2026-04-03
- N1 (nit): Updated Critical Files entry for `src/flat_file.rs` to list both struct-literal test sites (lines 88-95 and 139-147); line 142 was omitted.
- N2 (nit): Replaced "Add a unit test...repeat_limit: 5" instruction in Automated tests with a confirmation note pointing at the already-present ST50-1-TEST-3.

## Prefect-2 Report

### Nit issues

- **N3** (nit) `M11-50-1-tx-mods-multi-field.md:15` Critical Files entry for `data/sections.yml` states "lines 76-80" but the tx_mods block opens at line 75 (`- type: section`); the range should be 75-80. The diff in Step 4 correctly shows all six lines being replaced, so this is a cosmetic inaccuracy in the line hint only.

  ```diff
  # M11-50-1-tx-mods-multi-field.md:15
  - - `data/sections.yml` - lines 76-80: the tx_mods section block to rewrite; also add five new Field blocks
  + - `data/sections.yml` - lines 75-80: the tx_mods section block to rewrite; also add five new Field blocks
  ```

## Prefect-1 Report

### Nit issues (auto-fixed)

- **N1** (nit) `M11-50-1-tx-mods-multi-field.md` Critical Files section: line 142 (`flat_block_id_is_string` test) was not listed; Critical Files only referenced "lines 88-95" but Step 2 correctly identifies both sites (lines 90 and 142). Updated Critical Files entry to cover both line ranges (88-95 and 139-147) so the implementer doesn't need to discover line 142 separately. No logic change.

- **N2** (nit) `M11-50-1-tx-mods-multi-field.md` Automated tests section: instructed implementer to "Add a unit test...repeat_limit: 5, loads it via `load_data_dir`" - but pre-written ST50-1-TEST-3 (`tx_mods_modifications_field_has_repeat_limit_10`) already covers this end-to-end. Replaced the "Add" instruction with a confirmation note pointing at the existing test.

## Implementation
Complete - 2026-04-03

## Progress
- Step 1: Added `repeat_limit: Option<usize>` with `#[serde(default)]` to `FlatBlock::Field` in flat_file.rs
- Step 2: Added `repeat_limit: None` to both Field struct literals in tests (lines 90 and 142)
- Step 3: Threaded `repeat_limit` through data.rs loader - extracted from FlatBlock::Field and passed as `*repeat_limit` to HeaderFieldConfig
- Step 4: Rewrote tx_mods section in sections.yml from list_select to multi_field with 5 children
- Step 5: Added 5 Field blocks (pressure, challenge, mood, communication, modifications) at bottom of sections.yml under `# Tx Mods fields` comment
- Step 6: Deleted data/tx_mods.yml
- Step 7: All 164 tests pass including 4 tx_mods_multi_field_tests
