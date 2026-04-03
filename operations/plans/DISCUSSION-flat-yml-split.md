# Discussion: Flat YML Split

**Date:** 2026-04-01
**Status:** Discussion Complete

## Problem
The current `sections.yml` is a single deeply-nested file where groups contain sections, sections contain fields, and fields contain option lists inline. Adding a new field or expanding an options list requires scrolling through every layer of nesting to find the right place, and all concerns are entangled in one file.

## Goal
Restructure the data format so each logical concern lives in its own flat block, referenced by ID from its parent. Editing a field or an options list should be a targeted, isolated change — not a navigation challenge.

## Core Concept
Blocks (groups, sections, fields, option lists, boxes) are defined at the top level — either in their own dedicated file or together in one file. Parents reference children by flat ID list. The parser resolves references at load time and fails loudly if an ID is missing or mistyped.

## Context & Users
The primary user is the developer editing the data files. The new format should reduce friction when adding new fields or extending option lists, which will be frequent going forward. The app's runtime behavior should be identical to today — the change is purely in how data is authored and loaded.

## Constraints & Anti-goals
- The `id` / `label` / `output` shape for option objects must be preserved
- No silent failures — missing or mistyped IDs must produce a loud error at load time
- Splitting into multiple files is optional, not mandatory; everything can still live in one file
- No loss of existing functionality or features

## Key Decisions
- **Referencing syntax:** flat ID list — `fields: [field_date, field_start_time]` (not explicit `ref:` objects)
- **Options referencing:** a field references a named options list by ID — `options: minutes_list` — not individual option IDs
- **File split (proposed):** `boxes.yml`, `groups.yml`, `sections.yml`, `fields.yml`, plus options list files — each optional and combinable
- **Boxes:** the layout containers (map, wizard, preview) become first-class configurable objects, named "boxes" to allow non-column layouts in future
- **Inline still allowed:** a block can be defined inline within its parent if splitting it out isn't worth it
- **Loader strategy:** scan the `scribblenot/data` folder and load all yml files found there — no master import file needed
- **Type field required:** every block carries an explicit `type:` field (e.g. `type: section`, `type: field`, `type: option`). ID uniqueness is scoped per type — two blocks can share an ID as long as their types differ
- **Duplicate ID rule:** duplicate ID + type combinations are a loud error at load time
- **Cycle detection:** the loader actively detects and errors on circular references. The reference hierarchy (currently boxes -> groups -> sections -> fields -> options) is not hardcoded — any topology is valid, so cycle detection must be general-purpose

## Open Questions
None — all resolved during discussion.
