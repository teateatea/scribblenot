# Discussion: YAML Data Hierarchy Spec

**Date:** 2026-04-02
**Status:** Discussion Complete

## Problem
The existing data YMLs use inconsistent field names, container names, and nesting conventions across files. New sections need to be authored soon, and there is no canonical structure to write against. This is the right moment to standardize before the bulk of the data work begins.

## Goal
Define a clean, consistent hierarchy of named structures that all scribblenot YMLs conform to, enabling both new authoring and eventual migration of existing files.

## Core Concept
Six nested structures form a strict hierarchy: **Template > Group > Section > Field > List > Item**.

Each level contains the next. Structures can be defined inline or referenced by ID across files. The app resolves IDs by scanning the data directory - no root config or file name requirements.

### Structure Specs

**Item** - the smallest selectable unit; its output goes directly into the note.
- `output` - mandatory
- `id` - optional, for internal referencing
- `label` - optional, shown in UI; falls back to `output` if absent

**List** - a collection of Items presented to the user as a picker.
- `id`, `label`, `items` - mandatory
- `sticky` - optional, saves last chosen item for next session
- `default` - optional, used when no sticky saved (resolves by ID > label > output)
- `preview` - optional, shown in UI when no sticky or default
- `items` can be single-line `["a","b","c"]` (output-only) or multiline with full item attributes

**Field** - a UI box that opens a modal, stepping through its Lists in order.
- `id`, `label`, `lists` - mandatory
- `format` - optional, template string (e.g. `"{list1}: Patient prefers {list2}"`); placeholders map to list IDs; list order in `lists:` is presentation order, independent of format order
- `repeat_limit` - optional, default 0; if >0, confirming the field adds a duplicate UI box (up to limit)

**Section** - a container of Fields; produces a block of output in the note.
- `id`, `label`, `fields` - mandatory
- `nav_label` - optional, shortened label for navigation/sidebar
- `header` - optional, appears in the note before fields, only when something is confirmed
- `footer` - optional, appears in the note after fields, only when something is confirmed

**Group** - a container of Sections; same shape as Section.
- `id`, `label`, `sections` - mandatory
- `nav_label`, `header`, `footer` - optional, same semantics as Section

**Template** - top-level document; same shape as Group.
- `id`, `label`, `groups` - mandatory
- `nav_label`, `header`, `footer` - optional, same semantics as Group
- Only one Template is active at a time (multi-template switching is out of scope)

## Context & Users
The primary author is Tea, writing YMLs to populate scribblenot note sections. The consumer is the scribblenot app. Different Templates can represent different note formats (e.g. standard vs MVA), but only one is active per session for now.

## Constraints & Anti-goals
- No file name requirements for YAML files; app scans the data directory and resolves IDs from all files found
- Structures can be defined inline or referenced by ID across files
- free_text sections are not a special type; an editable preview field covers that use case (keybinding TBD, out of scope here)
- Boxes (the UI layer: map, wizard, etc.) are a separate concept and out of scope for this spec
- No changes to current copy/paste behavior
- `date_prefix` field (from existing sections.yml) is explicitly deferred

## Key Decisions
- `label` on Item is optional and falls back to `output` in the UI
- List order in `lists:` controls presentation order to the user, not format string order
- `nav_label` adopted as the standard short-label field (replacing `map_label`)
- `repeat_limit` lives on Field only (not Section or Group) for now
- ID resolution is directory-scan based, not config-file based

## Open Questions
- Should `repeat_limit` ever be supported at the Section or Group level?
- Should a higher structure be able to skip a level and reference a lower one directly (e.g. a Group referencing a Field)? Current assumption: no.
