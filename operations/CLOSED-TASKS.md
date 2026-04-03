# Closed Tasks

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

