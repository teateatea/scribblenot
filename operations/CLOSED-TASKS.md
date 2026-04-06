# Closed Tasks

- [ ] **#73** Run /plan-review-team on DISCUSSION-scribblenot-desktop-app.md to produce a reviewed plan *(sub-task 1 implemented)* *(sub-task 2 implemented)* *(sub-task 3 implemented)* *(sub-task 4 implemented)*
  [D:65 C:80]
  Claude: Use the completed discussion file as input for /plan-review-team, which runs propose-plan then review-plan in a coordinated multi-agent pipeline. The plan will need to cover: tray app architecture, global hotkeys, chord shortcuts for section expansion, HIPAA constraints (no disk writes, no logging, clipboard-only output), cross-platform binary distribution, migration of existing terminal UI code, and near-instant startup. Natural next step after the discussion is complete and before /pathfinder-premission.
  Joseph-Raw: /plan-review-team DISCUSSION-scribblenot-desktop-app.md
  Context: not specified
- Completed: 2026-04-06T07:02:19

- [ ] **#51** Move hard-coded section metadata into sections.yml to enable config-driven section definitions
  [D:65 C:72]
  Claude: Added is_intake/heading_search_text/heading_label/note_render_slot to SectionConfig and FlatBlock::Section; populated all 14 section blocks in sections.yml; removed heading_anchor() section arms, is_intake_section(), intake_heading() from note.rs; replaced all 9 cfg.id == "..." render checks with cfg.note_render_slot lookups; eliminated known_ids shim. 180 tests pass, zero warnings.
- Completed: 2026-04-03T14:57:37

- [ ] **#50** Convert tx_mods section to multi_field with 5 categorized fields, removing tx_mods.yml
  [D:40 C:88]
  Claude: Restructured tx_mods from list_select to multi_field with 5 inline field children in sections.yml: pressure (6 options), challenge (6), mood (6), communication (6 incl. 2x STOIC entries for distinct patient notes), modifications (13 options, repeat_limit: 10). All options moved inline; data/tx_mods.yml deleted; no source references remain. FlatBlock::Field gained repeat_limit field; loader threads it through to HeaderFieldConfig. 172 tests pass, zero warnings.
- Completed: 2026-04-03T14:23:04

- [ ] **#48** Generalize multi_field note rendering to support arbitrary sections beyond the appointment header
  [D:40 C:72]
  Claude: Generalized multi_field rendering in note.rs. Split render_note into two passes: Pass 1 renders the appointment header by cfg.id=="header" using existing format functions (identical output); Pass 2/inline dispatch renders all other multi_field sections via render_multifield_section() at their correct position in the note. Added pub fn render_multifield_section dispatcher, #[derive(Clone)] to NoteRenderMode, and catch-all block for unknown multi_field ids after INFECTION CONTROL. tx_mods block now calls render_multifield_section when section_type is "multi_field". 160 tests pass, zero warnings.
  Context: tx_mods restructuring discussion
- Completed: 2026-04-03T02:18:58

- [ ] **#49** Add repeat_limit: N to multi_field fields so a field can re-queue itself up to N times after confirmation
  [D:60 C:82]
  Claude: Add optional `repeat_limit: usize` to `HeaderFieldConfig` in `src/data.rs` (serde default = none = no repeat). In `HeaderState`, track a repetition counter per field slot. After confirming a field with repeat_limit set, re-present the same field at the current position and increment the counter; once the counter reaches repeat_limit, advance normally. The user can skip/back to stop repeating early. All confirmed values from repetitions are collected and included in the section's note output. The N cap prevents runaway repetition from held keys. Primary use case is the Modifications field in the planned tx_mods multi_field section. Touches HeaderFieldConfig, HeaderState (src/sections/header.rs), and note rendering. Prerequisite for #50.
  Joseph-Raw: I think for modifications, can we add a new field setting? "repeating: N" (or recommend a better name), where after that field is confirmed, it'll add itself as an available field again below, up to N times. So I can choose Modication: PREGNANCY, then Modification: HEAD PILLOW REQUIRED, then Modification: TIMELY, up to N times. (And it's up to N times and not repeating: true just to prevent accidentally adding thousands of entries if a button is held down too long lol. I'd probably put it at N = 10 or something most of the time.)
  Context: tx_mods restructuring discussion
- Completed: 2026-04-03T01:19:15

- [ ] **#52** Extract hard-coded boilerplate strings from note.rs into editable YML data files *(implemented)*
  [D:62 C:72]
  Claude: Added FlatBlock::Boilerplate variant to flat_file.rs; created data/boilerplate.yml with treatment_plan_disclaimer and informed_consent blocks; added boilerplate_texts: HashMap<String,String> to AppData populated from loader; threaded &HashMap through render_note() and section_start_line() signatures; replaced hard-coded literals in note.rs with runtime lookups. 122 tests pass, zero warnings.
- Completed: 2026-04-03T00:06:03

- [ ] **#46** Neutralise block_select struct and key names so they aren't tied to treatment-region vocabulary *(implemented)*
  [D:20 C:90]
  Claude: Renamed RegionConfig->BlockSelectEntry, RegionsFile->BlockSelectFile (TechniqueConfig deleted; entries reuse Vec<PartOption>), YAML keys regions:/techniques:->entries:. Runtime renames: RegionState->BlockSelectGroup, technique_selected->item_selected, toggle_technique->toggle_item, BlockSelectFocus::Regions/Techniques->Groups/Items, BlockSelectState fields regions->groups/region_cursor->group_cursor/technique_cursor->item_cursor, methods enter_region->enter_group/exit_techniques->exit_items/in_techniques->in_items/current_region_idx->current_group_idx. Updated app.rs, ui.rs, note.rs call sites. Zero warnings; 111 tests pass.
- Completed: 2026-04-02T21:02:22

- [ ] **#47** Add per-technique default selection state to block_select *(implemented)*
  [D:20 C:92]
  Claude: Add `default: bool` (serde default = true) to `TechniqueConfig` in `src/data.rs`. Update `RegionState::from_config` in `src/sections/block_select.rs` to initialize `technique_selected` from each technique's `default` field rather than hardcoding `false`. In tx_regions.yml, any technique with `default: false` will start unselected; all others (field omitted or `default: true`) start selected. Lets authors mark rarely-used techniques as off by default on a per-region basis without affecting others.
  Context: User noted they nearly always use the first three techniques in LOWER BACK (Prone) and only Fascial is typically off
- Completed: 2026-04-02T20:26:50

- [ ] **#23** Auto-generate multi-character hint permutations from base hint characters for overflow assignment
  [D:55 C:58] When the base hint pool is smaller than the number of hints needed, generate 2-char (and if needed, 3-char+) permutations using n^r logic. Permutations adjacency-ordered; stored in `hint_permutations:` field on KeyBindings; regenerated at load time when hints list changes.
- Completed: 2026-03-30T18:52:49

- [ ] **#22** Implement multi-character hint sequences with progressive prefix filtering
  [D:65 C:60] Added hint_buffer state machine; filter_hints_by_prefix + resolve_hint (Exact/Partial/NoMatch); progressive prefix highlighting in render_section_map and render_header_widget.
- Completed: 2026-03-30T18:52:49

- [ ] **#21** Add persistent group-jump hotkeys in map column (e.g. Q=Intake, W=Subjective, F=Treatment)
  [D:62 C:55] Universal group-jump fires at any map_hint_level; section hints skip first n_groups slots; all group hints always active (HINT color) when map focused.
- Completed: 2026-03-30T18:52:49

- [ ] **#2** Add Shift+Enter super-confirm keybinding to auto-complete remaining fields
  [D:70 C:55] Implement a Shift+Enter keybinding that, when pressed in any field or wizard modal, automatically confirms all remaining parts using already-confirmed values first, then sticky/default values -- skipping user interaction for fields that already have a valid answer.
  Joseph: Add Shift+Enter, for a "super confirm". Add an option for it in keybindings please. Super-confirm can be used on a field to automatically enter whatever is in the text box: Any entries that already got confirmed (green), then Sticky values and default values (grey). This should work in any field, but the example for Date would be a) Select Day: 24 to update the day, then Shift+Enter to auto-confirm the already correct Month and Year, or even b) if the Day is already a correct sticky, a Shift+Enter from the wizard directly skips all the modals and puts the sticky 2026-03-24.
  Context: not specified
- Completed: 2026-03-30T12:05:11


- [ ] **#70** Propose a plan to implement the canonical 6-level YAML data hierarchy from the discuss-idea session
  [D:75 C:72]
  Claude: The /discuss-idea session produced DISCUSSION-yaml-data-hierarchy.md, which defines a canonical 6-level hierarchy: Template > Group > Section > Field > List > Item. Each level has mandatory and optional fields fully specified. The natural next step is /propose-plan referencing this discussion file. The plan will need to address: (1) schema/validation layer for the hierarchy, (2) migration of existing data YMLs to conform to the new structure (renaming fields like map_label to nav_label), (3) app-side ID resolution via directory scan rather than a root config, and (4) authoring new sections against the spec. Key constraints: no file name requirements, inline or cross-file ID referencing, free_text is not a special type, Boxes (UI layer) are out of scope. Open questions about repeat_limit at Section/Group level and skip-level referencing are deferred.
  Context: /discuss-idea session producing DISCUSSION-yaml-data-hierarchy.md
- Completed: 2026-04-03T17:13:32

