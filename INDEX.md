<!-- indexed: 2026-03-25 13:35 -->
# Project Index

.claude/plans/ - COMPLETED-* plan files for each pathfinder task, one per implemented sub-task, named with mission prefix

.claude/PROJECT-TESTS.md - Per-task acceptance test checklists used by pathfinder to verify implementation; checked after each task completes

.claude/PROJECT_LOG.md - Project breadcrumb trail: version history, architecture decisions, rebuild instructions

.claude/settings.local.json - Project-local permission allow rules: cargo build/check, gstack, gh, skill invocations

.claude/TASKS.md - Active task backlog for scribblenot; scored by Difficulty and Clarity Confidence; feature backlog lives in TODOS.md

Cargo.lock - Rust dependency lockfile, pinning all transitive dependency versions

Cargo.toml - Package manifest; defines scribblenot binary, dependencies (ratatui, crossterm, serde_yaml, arboard, chrono, anyhow)

LICENSE - Project license file

TODOS.md - Feature backlog: returning-patient note continuation (P2/L) and section defaults (P2/S); superseded configurable-section-order item

data/config.yml - Runtime config state: pane_layout, sticky_values (date parts), hint label settings

data/infection_control.yml - Checklist items for standard infection control precautions section

data/keybindings.yml - User-configured key bindings overriding defaults (navigate, select, confirm, hints, etc.)

data/muscles.yml - Reference list of muscle label/output pairs used by the objective findings selector

data/objective_findings.yml - Pre-built list-select entries for objective/observation findings (muscle tension, ROM, etc.)

data/remedial.yml - Pre-built list-select entries for remedial exercises and self-care recommendations with full instruction text

data/sections.yml - Master section definition file: all groups (intake, subjective, treatment, objective, post-tx) with section types, field configs, composite formats, and data_file references

data/tx_mods.yml - Pre-built list-select entries for treatment modification preferences (pressure, atmosphere, positioning, special conditions)

data/tx_regions.yml - Block-select data: treatment regions (prone/supine body areas) with their available massage techniques and formatted output text

espanso/config/default.yml - Espanso global config: clipboard backend, pre-paste delay, undo_backspace disabled

espanso/match/1 General Note.yml - Espanso legacy match file for general note snippets

espanso/match/base.yml - Espanso base matches: date helpers (.DATE., .ISODATE.), consent form (.CONSENT.), appointment request builder (.REQUEST.), time/region pickers, exercise/sleep/social entry forms

espanso/match/muscles.yml - Espanso muscle-name snippet collection (large file, muscle-specific triggers)

espanso/match/OSE.yml - Espanso matches for OSE/SAVE scoring tables (strength assessment output with bolt-formatted values by finger number)

espanso/match/Patient Notes (jot notes for memory).md - Markdown jot notes / patient memory reference document

espanso/match/RMT 1 - Intake.yml - Espanso matches for RMT intake note section

espanso/match/RMT 1 - Intake Tidy.yml - Espanso matches for cleaned-up intake note snippets

espanso/match/RMT 2 - Subjective Tidy.yml - Espanso matches for tidy subjective section snippets

espanso/match/RMT 3 - Treatment.yml - Espanso matches for treatment section note snippets

espanso/match/RMT 4 - Subjective.yml - Espanso matches for subjective section note snippets

espanso/match/RMT 5 - Objective.yml - Espanso matches for objective findings note snippets

espanso/match/RMT 9 - Post Treatment.yml - Espanso matches for post-treatment section note snippets

espanso/match/RMT.yml - Espanso top-level RMT match file: note headings, appointment header (oienhead/arsthead), subjective builder, response-to-previous form

example-note.md - Sample completed clinical note showing the full rendered output format for all sections

operations/plans/PLAN-74-full-cutover-typed-contains.md - Full-cutover plan for replacing section-centric hierarchy assumptions with typed contains refs, first-class collections, structural note traversal, and explicit runtime ownership

pathfinder/MISSION-LOG-1-super-confirm-cleanup.md - Mission log for mission 1 (super-confirm-cleanup): task status, permission events, casualties

pathfinder/MISSION-LOG-2-pathfinder-skill-fixes.md - Mission log for mission 2 (pathfinder-skill-fixes): task status, permission events, casualties

pathfinder/MISSION-LOG-3-pathfinder-skill-polish.md - Mission log for mission 3 (pathfinder-skill-polish): task status, permission events, casualties

pathfinder/MISSION-LOG-4-tdd-warn-tracking.md - Mission log for mission 4 (tdd-warn-tracking): task status, permission events, casualties

pathfinder/MISSION-LOG-active.md - Rolling log of permission denials and needs-manifest events from the most recent mission; rotated to numbered log on completion

pathfinder/MISSION-PERMISSIONS.json - Approved action manifest for the current mission: allowed Read/Write/Edit/Bash patterns with rationale

pathfinder/MISSION-6-BRIEF.md - Mission goals, requirements, non-goals, constraints, and test criteria for mission 6 (skill-log-quality)

pathfinder/SUCCESSFUL-MISSION-LOG-5-pathfinder-skill-overhaul.md - Completed mission 5 log: all task outcomes, priority scores, attempt counts for the pathfinder-skill-overhaul mission

pathfinder/SUCCESSFUL-MISSION-LOG-5-precompact-hook-log.md - Pre-compact hook output captured during mission 5; records what was logged before context compaction

src/app.rs - Core application state (App struct, SectionState enum, Focus, MapHintLevel, StatusMsg); key event dispatch for all section types, modal handling, map navigation, config persistence
  also: App::new initialization, tick logic, state transitions between sections

src/config.rs - Config struct: pane_layout, sticky_values, hint label settings; load/save from data/config.yml

src/data.rs - All data model types (SectionConfig, SectionGroup, HeaderFieldConfig, CompositeConfig, PartOption, KeyBindings, AppData); YAML deserialization and AppData::load
  also: find_data_dir() resolution; list entry append/reload helpers

src/main.rs - Entry point: terminal setup (crossterm alternate screen, raw mode), event loop, clipboard copy on note completion

src/modal.rs - SearchModal and CompositeModal: filterable list picker with search bar, sticky cursor memory, multi-part composite field progression

src/note.rs - Note renderer: render_note() assembles all section states into the final markdown note string with correct headings and separators
  also: section_start_line() for scroll-to-section; header date/time formatters

src/sections/block_select.rs - Legacy nested toggle state used by older treatment-region flows
src/sections/collection.rs - CollectionState: top-level collection toggles with remembered inner item defaults and resets for treatment regions

src/sections/checklist.rs - ChecklistState: toggle-based checklist (defaults all checked) for infection control section

src/sections/free_text.rs - FreeTextState: date-prefixed free-text entry with browse/edit mode for narrative sections (ADL, subjective, etc.)

src/sections/header.rs - HeaderState: multi-field header entry (date, time, duration, appointment type) with composite field support and forward/back navigation

src/sections/list_select.rs - ListSelectState: scrollable list with multi-select and inline add-new-entry capability for tx_mods, objective, remedial sections

src/sections/mod.rs - Sections module declarations: header, free_text, list_select, block_select, collection, checklist

src/theme.rs - Semantic color palette constants (ACTIVE, SELECTED, HINT, MODAL, MUTED, ERROR, DISPLACED) and composed ratatui Style helper functions

src/ui.rs - Ratatui render function: three-pane layout (map / wizard / note preview), section-specific widget rendering, modal overlay, help overlay, status bar
