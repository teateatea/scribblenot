# Project Foundation

## Goals
Refactor the scribblenot data loading pipeline from a single deeply-nested `sections.yml` to a flat, type-tagged YML block format with ID-based cross-references. Each logical concern (boxes, groups, sections, fields, option lists) lives in its own flat block, referenced by ID from its parent. The loader scans the `data/` directory, merges all YML files found, resolves references at load time, and produces loud errors for missing IDs, duplicate ID+type pairs, and circular references. Runtime behavior is identical to today -- this is a data authoring and loading change only.

## Requirements
- Every block carries an explicit `type:` field; ID uniqueness is scoped per type
- Parents reference children by flat ID list (e.g. `fields: [field_date, field_start_time]`)
- Fields reference named option lists by ID (e.g. `options: minutes_list`), not individual option IDs
- Inline child definitions and ID-references are both supported in the same parent block (hybrid loader)
- Loader scans `scribblenot/data/` and merges all YML files found; no master import file required
- Missing IDs and duplicate ID+type combinations are loud load-time errors
- Circular references are actively detected and raised as errors at load time
- The `id` / `label` / `output` shape for option objects is preserved exactly
- The existing `sections.yml` is rewritten into the new flat format as part of this task
- No silent failures anywhere in the loading pipeline

## Task Priority Order
- #45 - Refactor data format to flat, type-tagged YML blocks with ID-based cross-references

## Explicit Non-Goals
- No changes to runtime behavior -- this is not a feature change
- No changes to the UI, rendering logic, or any non-loading code
- No mandatory file split -- authors may keep all blocks in a single file if preferred
- No hardcoding of the reference hierarchy (boxes -> groups -> sections -> fields -> options)
- No loss of existing functionality or features

## Constraints
- The `id` / `label` / `output` option shape must be preserved without alteration
- Splitting into multiple files is optional; everything may still live in one file
- Loader must handle both inline child definitions and ID-references in the same parent block (hybrid support required per pre-mission clarification)
- The new loader must be implemented AND `sections.yml` must be rewritten into the flat format as part of task #45 (per pre-mission clarification)
- Cycle detection must be general-purpose -- the reference hierarchy is not hardcoded
- Any topology of references is valid; only cycles are prohibited

## Test Criteria
- The app loads and runs with identical runtime behavior after the refactor
- All existing fields, groups, sections, boxes, and option lists are present and correctly resolved from the new flat format
- A YML file with a missing ID reference produces a loud load-time error and does not silently continue
- A YML file with a duplicate ID+type combination produces a loud load-time error
- A YML file with a circular reference produces a loud load-time error
- A parent block using inline child definitions and one using ID-references both load correctly in the same run
- All blocks in the rewritten `sections.yml` (or equivalent `data/` files) carry explicit `type:` fields

## Coordination
- READY: false
- BEGIN: true
