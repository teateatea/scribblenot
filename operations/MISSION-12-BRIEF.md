# Project Foundation

## Goals
Implement the canonical 6-level YAML data hierarchy (Template > Group > Section > Field > List > Item) as defined in DISCUSSION-yaml-data-hierarchy.md. This involves introducing new Rust data types for the hierarchy, building a directory-scanning YAML loader with cross-file ID resolution, migrating existing data files to the new structure, and adding validation for missing IDs, duplicate IDs, and circular references. The result is a consistent, author-friendly data format that all current and future scribblenot YMLs conform to.

## Requirements
- Define Rust data types for all six hierarchy levels (Template, Group, Section, Field, List, Item) with mandatory and optional fields matching the spec
- Implement a YAML loader that scans the data/ directory, merges all YAML files found, and resolves cross-file ID references without requiring a root config or specific file names
- Migrate existing data files (sections.yml, tx_regions.yml, and any other current data YMLs) to the new format, including renaming map_label to nav_label and conforming to the new structure
- Validate at load time: missing IDs, duplicate IDs, and circular references must all produce loud errors
- Runtime behavior must remain identical after migration - this is a data format and loading change only
- Boxes (UI layer) are out of scope and must not be designed for or assumed

## Task Priority Order
- #70 - Propose a plan to implement the canonical 6-level YAML data hierarchy from the discuss-idea session

## Explicit Non-Goals
- No changes to Boxes (the UI layer: map, wizard, etc.) - that is a separate concept and out of scope
- No changes to copy/paste behavior
- No file name requirements imposed on YAML files
- free_text is not a special section type - no special handling
- Multi-template switching is out of scope (only one Template is active at a time)
- repeat_limit at the Section or Group level is deferred
- Skip-level referencing (e.g. a Group referencing a Field directly) is not in scope
- date_prefix field migration is explicitly deferred
- No changes to keybindings or the editable preview field behavior

## Constraints
- ID resolution must be directory-scan based, not config-file based
- Structures may be defined inline or referenced by ID across files; both must be supported
- nav_label is the standard short-label field (map_label in existing files must be renamed)
- repeat_limit lives on Field only
- List items can be single-line shorthand (output-only strings) or multiline with full item attributes
- label on Item is optional and falls back to output in the UI
- List order in lists: controls presentation order; format string placeholder order is independent
- Plan must cover: (1) new Rust data types for the 6-level hierarchy, (2) YAML loader that scans data/ and resolves cross-file ID references, (3) migration of existing sections.yml/tx_regions.yml/etc to the new format, (4) validation (missing IDs, duplicate IDs, circular refs)

## Test Criteria
- All existing data files load without error after migration to the new format
- Cross-file ID references resolve correctly at load time
- Duplicate ID + type combinations produce a loud load-time error
- Missing referenced IDs produce a loud load-time error
- Circular references are detected and produce a loud load-time error
- Runtime note output is identical before and after the migration (no behavioral regressions)
- nav_label is used in place of map_label everywhere with no leftover references to map_label

## Coordination
- READY: false
- BEGIN: true
