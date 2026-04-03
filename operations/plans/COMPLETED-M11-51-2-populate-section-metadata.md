**Task**: #51 Move hard-coded section metadata into sections.yml

**Context**: ST51.1 added four metadata fields to SectionConfig. ST51.2 populates those
fields across the remaining 13 section blocks in data/sections.yml so the four ST51-2
tests (currently failing) all pass. No source changes needed -- only YAML edits.
tx_mods is already complete and is skipped.

**Approach**: Add missing metadata fields to 13 section blocks in sections.yml. No source
changes needed. Serde defaults handle absent fields with false/None.

**Critical Files**:
- data/sections.yml (populate 13 section blocks)
- src/data.rs lines 2075+ (ST51-2 tests -- must pass after)

**Steps**:

All new lines use 4-space indentation (matching existing sections.yml style).

1. **header** -- after `section_type: multi_field`, add:
   `note_render_slot: "header"`

2. **adl** -- already has is_intake and heading_label. After `heading_label: "#### ACTIVITIES OF DAILY LIVING"`, add:
   `heading_search_text: "ACTIVITIES OF DAILY LIVING"`

3. **exercise** -- after `section_type: free_text`, add:
   `is_intake: true`
   `heading_label: "#### EXERCISE HABITS"`
   `heading_search_text: "EXERCISE HABITS"`

4. **sleep_diet** -- after `section_type: free_text`, add:
   `is_intake: true`
   `heading_label: "#### SLEEP & DIET"`
   `heading_search_text: "SLEEP & DIET"`

5. **social** -- after `section_type: free_text`, add:
   `is_intake: true`
   `heading_label: "#### SOCIAL & STRESS"`
   `heading_search_text: "SOCIAL & STRESS"`

6. **history** -- after `section_type: free_text`, add:
   `is_intake: true`
   `heading_label: "#### HISTORY & PREVIOUS DIAGNOSES"`
   `heading_search_text: "HISTORY & PREVIOUS DIAGNOSES"`

7. **specialists** -- after `section_type: free_text`, add:
   `is_intake: true`
   `heading_label: "#### SPECIALISTS & TREATMENT"`
   `heading_search_text: "SPECIALISTS & TREATMENT"`

8. **subjective_section** -- after `section_type: free_text`, add:
   `heading_search_text: "## SUBJECTIVE"`
   `note_render_slot: "subjective_section"`

9. **tx_mods** -- already complete, no change.

10. **tx_regions** -- after `data_file: "tx_regions.yml"`, add:
    `heading_search_text: "TREATMENT / PLAN"`
    `note_render_slot: "tx_regions"`

11. **objective_section** -- after `date_prefix: true`, add:
    `heading_search_text: "## OBJECTIVE / OBSERVATIONS"`
    `note_render_slot: "objective_section"`

12. **post_treatment** -- after `section_type: free_text`, add:
    `heading_search_text: "## POST-TREATMENT"`
    `note_render_slot: "post_treatment"`

13. **remedial_section** -- after `date_prefix: true`, add:
    `heading_search_text: "REMEDIAL EXERCISES"`
    `note_render_slot: "remedial_section"`

14. **tx_plan** -- after `section_type: free_text`, add:
    `heading_search_text: "TREATMENT PLAN"`
    `note_render_slot: "tx_plan"`

15. **infection_control_section** -- after `data_file: "infection_control.yml"`, add:
    `heading_search_text: "INFECTION CONTROL"`
    `note_render_slot: "infection_control_section"`

16. Run 4 named ST51-2 tests individually to confirm each passes.

17. Run `cargo test --manifest-path "C:/Users/solar/Documents/Claude Projects/scribblenot/Cargo.toml"` -- zero regressions.

18. Commit `data/sections.yml` and `src/data.rs` (ST51-2 tests) with message:
    `Implement task #51 sub-task 51.2: populate section metadata in sections.yml`

**Verification**:

### Automated tests
- `cargo test all_intake_sections_have_is_intake_true` -- currently fails, must pass after
- `cargo test all_intake_sections_have_heading_label` -- currently fails, must pass after
- `cargo test all_sections_with_search_text_are_set` -- currently fails, must pass after
- `cargo test remaining_sections_have_note_render_slot` -- currently fails, must pass after
- `cargo test` (full suite) -- zero regressions

## Progress

All 15 metadata edits applied to data/sections.yml (items 1-15 in plan steps).

Test results (2026-04-03):
- all_intake_sections_have_is_intake_true: PASS
- all_intake_sections_have_heading_label: PASS
- all_sections_with_search_text_are_set: PASS
- remaining_sections_have_note_render_slot: PASS
- Full suite: 180/180 passed, 0 failed

Commit: 74935c4
