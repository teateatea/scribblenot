# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [x] **#45** Refactor data format to flat, type-tagged YML blocks with ID-based cross-references
  [D:80 C:75]
  Claude: Full refactor of the data loading pipeline. Currently sections.yml is a single deeply-nested file. The new format splits concerns across optional separate files (boxes.yml, groups.yml, sections.yml, fields.yml, plus freeform options files) all living in data/. Each block is flat and carries a `type:` field. Parents reference children by flat ID list (e.g. `fields: [field_date]`). Option lists are referenced by name (e.g. `options: minutes_list`). Loader scans data/ and merges all yml files found. ID uniqueness is scoped per type -- same ID is valid across different types. Missing IDs and duplicate ID+type combos are loud load-time errors. Circular references must be actively detected and errored. The id/label/output option shape is preserved. Runtime behavior must be identical -- this is a data authoring and loading change only. Full spec in operations/plans/DISCUSSION-flat-yml-split.md.
  Context: /discuss-idea session on reconfiguring sections.yml data format

- [x] **#44** Add /add-tasks as a forwarding alias to /add-task without duplicating the skill
  [D:15 C:60] Create a minimal /add-tasks skill entry that immediately delegates to /add-task so both trigger words work; the alias contains no logic of its own, avoiding a maintenance burden when /add-task changes.
  Joseph: The /add-task skill should also trigger on /add-tasks. It's too easy for me to add that s when I'm thining about adding several, and it might as well work correctly. Don't just copy the /add-task skill though, I don't want to have to maintain identical skills.
  Context: not specified

- [x] **#46** Neutralise block_select struct and key names so they aren't tied to treatment-region vocabulary
  [D:20 C:90]

- [ ] **#48** Generalize multi_field note rendering to support arbitrary sections beyond the appointment header
  [D:40 C:72]
  Claude: `format_header()` in `src/note.rs` renders multi_field output with hard-coded assumptions about date/time/appointment structure. To use multi_field for other sections (like tx_mods), the renderer must be generalized: given any completed multi_field section, output each confirmed field's value in sequence without assuming field semantics. The existing appointment header must continue to render identically -- either keep `format_header()` for the header section specifically (detected by section id) and add a generic renderer for others, or refactor `format_header()` to branch on section id. Changes are in `src/note.rs` and potentially `src/app.rs`. Prerequisite for #50.
  Context: tx_mods restructuring discussion

- [ ] **#49** Add repeat_limit: N to multi_field fields so a field can re-queue itself up to N times after confirmation
  [D:60 C:82]
  Claude: Add optional `repeat_limit: usize` to `HeaderFieldConfig` in `src/data.rs` (serde default = none = no repeat). In `HeaderState`, track a repetition counter per field slot. After confirming a field with repeat_limit set, re-present the same field at the current position and increment the counter; once the counter reaches repeat_limit, advance normally. The user can skip/back to stop repeating early. All confirmed values from repetitions are collected and included in the section's note output. The N cap prevents runaway repetition from held keys. Primary use case is the Modifications field in the planned tx_mods multi_field section. Touches HeaderFieldConfig, HeaderState (src/sections/header.rs), and note rendering. Prerequisite for #50.
  Joseph-Raw: I think for modifications, can we add a new field setting? "repeating: N" (or recommend a better name), where after that field is confirmed, it'll add itself as an available field again below, up to N times. So I can choose Modication: PREGNANCY, then Modification: HEAD PILLOW REQUIRED, then Modification: TIMELY, up to N times. (And it's up to N times and not repeating: true just to prevent accidentally adding thousands of entries if a button is held down too long lol. I'd probably put it at N = 10 or something most of the time.)
  Context: tx_mods restructuring discussion

- [ ] **#50** Convert tx_mods section to multi_field with 5 categorized fields, removing tx_mods.yml
  [D:40 C:88]
  Claude: Restructure tx_mods from list_select (one scrollable multi-select list) to multi_field (sequential single-select per category). Five fields: Pressure (LIGHT/MODERATE/REGULAR/FIRM/HEAVY/FULL), Challenge (VERY GENTLE/GENTLE/RELAXED/COMFORTABLE/STRONG/CHALLENGING), Mood (CALMING/RELAXING/CONVERSATIONAL/SOCIAL/RESTFUL/INTROSPECTIVE), Communication (CONCISE/STOIC x2/CONTROLLED/COMMUNICATIVE -- STOIC appears as two distinct entries for the two different notes), Modifications (PREGNANCY/POST-CONCUSSION/VERTIGO/LOW LIGHT/NO PRONE/SUPINE ONLY/HEAD PILLOW/HEAD PILLOW REQUIRED/LOWER TABLE/TALL/CLOTHED/RAYNAUDS/TIMELY with repeat_limit: 10). Pressure/Challenge/Mood/Communication are single-select. All options move inline to sections.yml field blocks; data/tx_mods.yml is deleted and the data_file reference removed from the section config. Note: RAYNAUD'S apostrophe needs handling in YAML. Depends on #48 and #49.
  Joseph-Raw: So, Tx Mods should probably actually be a multi_field. Can you split the list up into fields: Pressure, Challenge, Mood, Communication, and Modifications? Try to categorize them correctly, I'll adjust any that are confusing.
  Context: tx_mods restructuring discussion

- [ ] **#51** Move hard-coded section metadata into sections.yml to enable config-driven section definitions
  [D:65 C:72]
  Claude: Currently, section-specific logic is scattered across source code: heading_search_text(), is_intake_section(), intake_heading(), and cfg.id == "..." comparisons in note.rs. This task centralises all that metadata into sections.yml with fields like heading_search_text, is_intake, heading_label, and render behavior flags. The goal is that adding a new section requires only a yml change, not source edits in multiple places. This is a meaningful refactor touching note.rs and likely the section config loading path, but it is well-scoped and the intent is clear. Requires identifying all scattered hardcoded section checks, designing the yml schema to cover them, updating the loader/structs to expose the new fields, and replacing all scattered code references. No new user-facing behaviour -- purely a configuration-driven architecture improvement.
  Context: Code audit of hard-coded one-off functions

- [ ] **#52** Extract hard-coded boilerplate strings from note.rs into editable YML data files *(52.1 implemented)* *(52.2 implemented)* *(52.3 implemented)*
  [D:62 C:72]
  Claude: Two hard-coded string literals in note.rs need to be moved to YML: (1) the treatment note boilerplate ("Regions and locations are bilateral...") and (2) the informed consent statement. Both are currently baked into functions in note.rs. The goal is to make these user-editable without requiring a source recompile. The strings should live in a YML file (likely alongside existing data YML files). Loader code will need to read these strings at runtime. No logic changes -- just lifting static strings out of Rust source into data files.
  Context: Code audit of hard-coded one-off functions

- [ ] **#53** Dispatch section type strings via extensible registry instead of hard-coded match arms (recommend /discuss-idea first)
  [D:75 C:45]
  Claude: Currently app.rs init_states() and data.rs load() both hard-code five section type string literals ("multi_field", "free_text", "list_select", "block_select", "checklist") in match arms. Unlike purely cosmetic or metadata tasks, these strings are load-bearing: they determine which rendering/state logic runs. Making them YML-extensible is a non-trivial architectural decision -- it requires deciding what "registerable section type" means (static enum, trait object, plugin map, etc.) before any plan can be written. A /discuss-idea session is explicitly recommended to resolve the design question first. High d_score because it touches the core dispatch spine of the app; moderate c_score because the problem is well-described but the solution space is deliberately left open.
  Context: Code audit of hard-coded one-off functions

- [ ] **#70** Propose a plan to implement the canonical 6-level YAML data hierarchy from the discuss-idea session
  [D:75 C:72]
  Claude: The /discuss-idea session produced DISCUSSION-yaml-data-hierarchy.md, which defines a canonical 6-level hierarchy: Template > Group > Section > Field > List > Item. Each level has mandatory and optional fields fully specified. The natural next step is /propose-plan referencing this discussion file. The plan will need to address: (1) schema/validation layer for the hierarchy, (2) migration of existing data YMLs to conform to the new structure (renaming fields like map_label to nav_label), (3) app-side ID resolution via directory scan rather than a root config, and (4) authoring new sections against the spec. Key constraints: no file name requirements, inline or cross-file ID referencing, free_text is not a special type, Boxes (UI layer) are out of scope. Open questions about repeat_limit at Section/Group level and skip-level referencing are deferred.
  Context: /discuss-idea session producing DISCUSSION-yaml-data-hierarchy.md

- [ ] **#71** Run /pathfinder-premission to set up mission infrastructure for the YAML hierarchy implementation
  [D:62 C:72]
  Claude: After /propose-plan produces a plan from DISCUSSION-yaml-data-hierarchy.md (task #70), run /pathfinder-premission referencing that discussion and resulting plan. The premission step sets up MISSION-BRIEF, MISSION-PERMISSIONS.json, and PROJECT-TESTS before handing off to pathfinder-mission-team for autonomous implementation. The discussion file is the primary input/reference for the premission session.
  Joseph-Raw: /pathfinder-premission DISCUSSION-yaml-data-hierarchy.md
  Context: /discuss-idea session producing DISCUSSION-yaml-data-hierarchy.md

---

## Code Quality

- [x] **#1** Remove dead code warning for unused `current_value` on `HeaderState`
  [D:10 C:55] Delete or use the `pub fn current_value()` method in `src/sections/header.rs` that triggers a dead_code warning on every `cargo build`/`cargo run`.
  Joseph: about that dead code clean up, I don't like that it pops up when I cargo run.
  Context: not specified

- [ ] **#54** Extract hard-coded layout strings from config.rs into a YML-backed enum
  [D:25 C:55]
  Claude: Two string literals ("default" and "swapped") are embedded directly in config.rs inside is_swapped() and set_swapped(). These should be replaced with an enum (or at minimum a YML-defined set of variants) so that adding new layout modes in the future does not require source changes. Low priority, small scope -- purely a future-proofing/maintainability concern with no current functional deficiency.
  Context: Code audit of hard-coded one-off functions
