# Project Foundation

## Goals
This mission removes the remaining hard-coded section metadata, boilerplate strings, and treatment-vocabulary assumptions that are currently in scope for the existing section system. It also extends the multi_field section type with repeat support and generalized rendering, enabling tx_mods to be restructured as a multi_field. Within the existing section-type architecture, the result should be that changing supported section content and metadata can be done through YML edits without changing Rust source.

## Requirements
- All region/technique vocabulary must be removed from block_select struct and key names (neutralised to generic equivalents)
- Per-entry default selection state must be configurable in block_select data files
- Boilerplate strings (treatment note disclaimer, informed consent) must be extracted from note.rs into type-tagged YML blocks under data/, consistent with the #45 flat YML architecture
- multi_field rendering must be generalised to work for any section, not just the appointment header
- multi_field fields must support `repeat_limit: N` so a field can re-queue itself up to N times after confirmation
- tx_mods must be converted from list_select to multi_field with five categorised fields; tx_mods.yml must be deleted
- All true section metadata (heading_search_text, is_intake, heading_label, render flags) must move from Rust source into existing `type: section` blocks in sections.yml; do not introduce a new note-layout schema, top-level layout file, or new hierarchy layer in this mission

## Task Priority Order
- #46 - Neutralise block_select struct and key names
- #47 - Add per-technique default selection state to block_select
- #52 - Extract hard-coded boilerplate strings from note.rs
- #49 - Add repeat_limit: N to multi_field fields
- #48 - Generalize multi_field note rendering
- #50 - Convert tx_mods section to multi_field
- #51 - Move hard-coded section metadata into sections.yml

## Explicit Non-Goals
- No intentional note-output changes beyond the explicit `multi_field` repeat flow needed for `tx_mods`; preserve existing output text and ordering everywhere else
- No UI redesigns or unrelated workflow changes outside the explicit `multi_field` / `tx_mods` wizard behavior required by this mission
- Do not introduce a new block type for boilerplate strings; use `type: boilerplate` within the existing flat YML system
- Do not create a new section block in sections.yml for tx_mods metadata; restructure the existing block
- Do not address the section-type dispatch registry (#53) - that requires a separate /discuss-idea session
- Do not extract layout strings from config.rs (#54) - out of scope for this mission
- Do not claim support for brand-new section types defined only in YML; this mission improves the existing supported section types only

## Constraints
- The `id` / `label` / `output` shape for option objects must be preserved
- No silent failures - missing or mistyped IDs must produce a loud error at load time
- Boilerplate blocks must live in data/ alongside existing YML files; one shared file is acceptable
- For #51: conduct a full codebase audit of all cfg.id comparisons and section-specific branches before writing the plan; move only metadata-like decisions into existing section config blocks, and leave broader note-layout architecture changes out of scope
- For #51: if new per-section keys are needed, add them only as optional fields on existing flat `type: section` blocks and their runtime `SectionConfig`; do not add a separate metadata block type or a second config file
- For #52: boilerplate strings must be `type: boilerplate` blocks in the flat YML system, consistent with the #45 architecture
- RAYNAUD'S apostrophe must be handled correctly in YAML for tx_mods (#50)
- All changes must compile cleanly with no new dead_code warnings
- Duplicate ID + type combinations remain a loud load-time error
- For #50: define the five tx_mods categories explicitly in data and preserve this rendered category order: Pressure, Challenge, Mood, Communication, Modifications
- For #50: repeated Modifications entries append after the first four categories in the order confirmed by the user; skip/back exits the repeat loop without adding another entry
- For #50: preserving output means preserving the existing entry text verbatim for each migrated option, aside from the structural change that multiple Modifications lines may now be emitted

## Test Criteria
- `cargo build` produces zero warnings and zero errors after each task
- block_select data file (tx_regions.yml) loads and renders identically to pre-mission behaviour after #46 and #47
- Techniques marked `default: false` start unselected; all others start selected
- Boilerplate strings loaded from YML match the previously hard-coded strings exactly
- multi_field sections other than the appointment header render their confirmed fields in sequence without crashing
- Modifications field in tx_mods re-queues up to 10 times and stops at the cap; skip/back exits the repeat early
- tx_mods renders the first confirmed value from Pressure, Challenge, Mood, and Communication in that order, followed by zero or more Modifications entries in confirmation order
- Each migrated tx_mods option preserves its pre-mission output string exactly
- data/tx_mods.yml is deleted and no reference to it remains in source or data files
- All section metadata previously hard-coded in note.rs is driven entirely from sections.yml; removing a hard-coded check from source does not change app output

## Coordination
- READY: false
- BEGIN: true
