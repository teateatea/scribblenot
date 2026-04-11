# TASKS

## scribblenot

_Tasks for active development. Feature backlog lives in TODOS.md._

- [ ] **#53** Dispatch section type strings via extensible registry instead of hard-coded match arms (recommend /discuss-idea first)
  [D:75 C:45]
  Claude: Currently app.rs init_states() and data.rs load() both hard-code five section type string literals ("multi_field", "free_text", "list_select", "block_select", "checklist") in match arms. Unlike purely cosmetic or metadata tasks, these strings are load-bearing: they determine which rendering/state logic runs. Making them YML-extensible is a non-trivial architectural decision -- it requires deciding what "registerable section type" means (static enum, trait object, plugin map, etc.) before any plan can be written. A /discuss-idea session is explicitly recommended to resolve the design question first. High d_score because it touches the core dispatch spine of the app; moderate c_score because the problem is well-described but the solution space is deliberately left open.
  Context: Code audit of hard-coded one-off functions

- [ ] **#71** Run /pathfinder-premission to set up mission infrastructure for the YAML hierarchy implementation
  [D:62 C:72]
  Claude: After /propose-plan produces a plan from DISCUSSION-yaml-data-hierarchy.md (task #70), run /pathfinder-premission referencing that discussion and resulting plan. The premission step sets up MISSION-BRIEF, MISSION-PERMISSIONS.json, and PROJECT-TESTS before handing off to pathfinder-mission-team for autonomous implementation. The discussion file is the primary input/reference for the premission session.
  Joseph-Raw: /pathfinder-premission DISCUSSION-yaml-data-hierarchy.md
  Context: /discuss-idea session producing DISCUSSION-yaml-data-hierarchy.md

- [ ] **#72** Completed discussion document: scribblenot desktop app conversion
  [D:5 C:40]
  Claude: DISCUSSION-scribblenot-desktop-app.md exists at operations/plans/ and is marked "Discussion Complete" (dated 2026-04-03). It covers converting scribblenot from a terminal tool to a distributable tray desktop app with global hotkey invocation, chord shortcuts for section expansion, clipboard-only output (HIPAA: no disk writes), near-instant startup, and optional pre-population from highlighted text. This entry captures the discussion as a reference artifact before the planning phase. No action required -- the discussion is already complete.
  Joseph-Raw: DISCUSSION-scribblenot-desktop-app.md
  Context: not specified

- [ ] **#75** Evaluate splitting sticky values out of config.md into a separate file (e.g. sticky.md)
  [D:35 C:45]
  Claude: Currently, sticky values (persistent/remembered user preferences or state) are written to config.md alongside user-edited configuration. The proposal is to move these to a dedicated file -- suggested name sticky.md or similar. The motivation is UX: users opening config.md to edit settings don't care about sticky values, so mixing them creates noise. The task is framed as a review/discuss of pros and cons before committing to an implementation. Pros of splitting: cleaner config.md, separation of concerns, stickies can be cleared/reset without touching config, easier to reason about each file's purpose. Cons: another file to manage, tooling must know to read from two places, potential for drift or sync issues. This is a design/architecture discussion task that may lead to an implementation decision.
  Joseph-Raw: We currently write sticky values to config.md. Review/discuss pros/cons for having a non-config file to write to instead. User doesn't really care about sticky values when they're opening config, so this could be a sticky.md or  whatever better name.
  Context: not specified

- [ ] **#76** Refactor default-theme: rename custom_colours to color_names, add iced built-in colors, replace hex literals with named constants, handle # prefix flexibly
  [D:55 C:72]
  Claude: Four-part refactor of the default-theme module. (1) Rename the custom_colours field/map to color_names (or similar -- user is flexible on the exact name). (2) Populate color_names with all iced built-in color constants so they're available by name throughout the theme. (3) Find-replace pass across the file swapping raw hex string literals for the named references -- the file is currently hard to read with scattered hex codes. (4) Make custom color parsing robust to hex strings with or without a leading '#' character (normalize at parse time). Part 4 implies a small parsing/normalization step. No new files implied -- edits are within the existing default-theme file(s). The rename and population steps should be done first so the replacement pass has names to substitute.
  Joseph-Raw: In default-theme, rename custom_colours to just color_names or something like that, then add all those iced colours in there. Then, replace all the hex codes throughout that file with the names, it's not very legible as is. Also, for our custom colors, I'd like it to be able to handle hex codes with and without # smoothly.
  Context: not specified

---

## Code Quality

- [ ] **#74** Remove dead fields from hierarchy structs in src/data.rs
  [D:15 C:90]
  Claude: Several fields added to the hierarchy structs during Mission 13 are never consumed by hierarchy_to_runtime or any downstream runtime code and should be removed: HierarchyTemplate.id, HierarchyItem.note, HierarchyField.data_file, HierarchyField.list_id, HierarchySection.lists, HierarchyFile.items. Additionally HierarchyGroup.num and SectionGroup.num carry a value through the data pipeline but are never read in app.rs or ui.rs (the num: keys have already been removed from sections.yml). Removing these fields eliminates dead code, makes struct definitions match actual usage, reduces the YAML surface area for future authors, and may require updating a small number of test fixtures that reference these fields.
  Context: post-Mission 13 review of hierarchy struct usage

- [ ] **#54** Extract hard-coded layout strings from config.rs into a YML-backed enum
  [D:25 C:55]
  Claude: Two string literals ("default" and "swapped") are embedded directly in config.rs inside is_swapped() and set_swapped(). These should be replaced with an enum (or at minimum a YML-defined set of variants) so that adding new layout modes in the future does not require source changes. Low priority, small scope -- purely a future-proofing/maintainability concern with no current functional deficiency.
  Context: Code audit of hard-coded one-off functions
