## Task

#74 Full Cutover - Replace the section-centric hierarchy with typed `contains:` nodes and first-class collections

## Status

User Approved

This is intentionally a full cutover, not a compatibility layer. Old YAML shapes and transitional runtime assumptions should be removed rather than preserved.

## Why This Plan Exists

The current codebase already has a merged multi-file YAML loader and a typed ID registry, but the runtime still collapses back into the older model:

- groups contain only sections
- sections are the only real top-level runtime units
- `collection` is implemented as a special `section_type`
- `collection` and `block_select` still pull from the ambient merged `lists:` pool
- note layout is still driven by `note_render_slot` matches rather than the authored structure

That mismatch is now the blocker.

The user has finalized the new direction:

- use typed `contains:` references
- keep all YAML files merged into one global registry by kind and `id`
- do not allow unnamed ambient membership from that registry
- treat `collection` as a first-class node kind, separate from `section`
- use `id` for stable identity
- use `label` only for UI/display
- stop preserving backward compatibility with the older YAML structure

This plan turns those product decisions into a concrete implementation path against the current codebase.

## Product Decisions Locked In

### 1. Global registry stays

All `data/*.yml` files are still merged into one logical hierarchy registry.

This remains valid:

- a group in one YAML file can reference a section in another file
- a section in one YAML file can reference a collection or list in another file

What changes is resolution behavior:

- global lookup by typed ID: yes
- implicit parent membership just because a node exists at top level: no

### 2. `contains:` is the only structural composition mechanism

The target authoring model is explicit typed child references.

Example:

```yml
template:
  id: patient_standard_template
  contains:
    - group: intake_group
    - group: subjective_group
    - group: treatment_group
    - group: objective_group

groups:
  - id: treatment_group
    label: Treatment
    nav_label: TREATMENT
    contains:
      - section: tx_mods
      - collection: tx_regions

sections:
  - id: tx_mods
    label: Treatment Modifications & Preferences
    body: multi_field
    contains:
      - field: pressure_field
      - field: positioning_field

collections:
  - id: tx_regions
    label: Treatment Regions
    contains:
      - list: back_all_prone
      - list: glutes_prone
```

### 3. `collection` and `list` are different things

- `list` is a data primitive: ordered items, picker metadata, defaults, sticky rules, repeating behavior
- `collection` is a runtime/UI primitive: owns ordered child lists and its own toggle/activation behavior

`collection` is not a special section mode and should not continue to masquerade as one.

### 4. Identity must be stable and data-backed

- `id` is the durable identity key for runtime state, document markers, and cross-file references
- `label` is display text only

This applies to document synchronization too:

- managed block markers stay keyed to stable `id`
- visible headings can derive from `label` or layout metadata, but they are not identity

### 5. Rendering order comes from authored structure

The long-term replacement for `note_render_slot` is authored order plus authored layout metadata.

For this cutover:

- the runtime tree walks `template.contains`, then each parent `contains`
- note output order must follow that structure
- special formatting wrappers remain allowed, but they must be attached to nodes or layout metadata rather than hard-coded section IDs or slot strings

### 6. No backward compatibility with old YAML shapes

This cutover should remove:

- `template.groups`
- `group.sections`
- section-owned `section_type: collection`
- collection loading via ambient top-level `lists:`
- runtime dependence on old structural fields where the new typed `contains:` graph replaces them

## Scope

### In scope

- redesign hierarchy structs in [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)
- replace loader validation to understand typed child references
- add `HierarchyCollection` and a typed child-reference model
- replace section-centric runtime projection with a real authored tree
- replace ambient `collection_data` population
- replace `note_render_slot`-driven note traversal with structural traversal
- update document sync and editable note anchor generation to the new tree
- migrate current `data/*.yml` files to the new grammar
- update tests and docs for the new schema/runtime contract

### Out of scope

- preserving old YAML compatibility
- maintaining dual schema support
- a partial migration that leaves collections section-shaped internally
- unrelated UI redesign beyond what the structural cutover forces

## Current Fault Lines In The Code

### 1. `src/data.rs` already merges YAML, but runtime projection throws structure away

Current loader behavior is close to what we want:

- `load_hierarchy_dir()` scans `data/`
- merges every `.yml` into one `HierarchyFile`
- validates IDs and references

Current runtime projection is the mismatch:

- `HierarchyTemplate` still uses `groups: Vec<String>`
- `HierarchyGroup` still uses `sections: Vec<String>`
- `HierarchySection` still mixes structure and runtime-only section behavior
- `hierarchy_to_runtime()` returns `Vec<SectionGroup>` plus `Vec<SectionConfig>`

That runtime shape hardcodes the older assumption that groups only contain sections and sections are the only real interactive units.

### 2. `collection` is still populated from the ambient merged list pool

In [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs), `hierarchy_to_runtime()` currently does this:

- clone all merged top-level lists into `top_lists`
- if a section has `section_type == "collection"`, insert `top_lists.clone()` into `collection_data`
- if a section has `section_type == "block_select"`, insert `top_lists.clone()` into `block_select_data`

That is exactly the behavior the user wants removed.

The new contract is:

- all lists may exist in one merged registry
- a collection only owns the lists named in its own `contains`

### 3. Note rendering still assumes slot-driven layout

In [src/note.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/note.rs), layout still depends on:

- `note_render_slot`
- hard-coded output groups like subjective, treatment, objective, post-treatment
- explicit branches for `tx_mods`, `tx_regions`, `remedial_section`, `infection_control_section`, etc.

That made sense when the authored structure could not express containment clearly. It is now the main source of data/runtime duplication.

### 4. Document sync is section-ID based, but not tree-aware yet

The document work in [src/document.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/document.rs) and [src/note.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/note.rs) already uses stable section IDs for managed markers:

```markdown
<!-- scribblenot:section id=tx_mods:start -->
```

That part is compatible with the new decision to use `id` as identity.

What needs to change is:

- headings and grouping should come from the new tree
- editable-document anchor generation should not depend on `note_render_slot`
- non-section nodes that own managed content may need their own anchor rules

### 5. App navigation and state still flatten through `Vec<SectionConfig>`

In [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs):

- `App.sections` is a flat `Vec<SectionConfig>`
- `SectionState` is the master runtime state enum
- map navigation is based on flat section ordering within `SectionGroup.sections`
- `CollectionState` is still nested under `SectionState::Collection`

That flat section list is the last major “everything is a section” assumption.

## Target End State

## Schema Model

Introduce typed structural references.

Recommended YAML-level structures:

```rust
pub struct HierarchyTemplate {
    pub id: Option<String>,
    pub contains: Vec<HierarchyChildRef>,
}

pub struct HierarchyGroup {
    pub id: String,
    pub label: Option<String>,
    pub nav_label: Option<String>,
    pub contains: Vec<HierarchyChildRef>,
}

pub struct HierarchySection {
    pub id: String,
    pub label: Option<String>,
    pub nav_label: Option<String>,
    pub body: Option<String>,
    pub contains: Vec<HierarchyChildRef>,
    pub note: Option<NoteNodeMeta>,
}

pub struct HierarchyCollection {
    pub id: String,
    pub label: Option<String>,
    pub nav_label: Option<String>,
    pub contains: Vec<HierarchyChildRef>,
    pub note: Option<NoteNodeMeta>,
}
```

Recommended typed child ref shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HierarchyChildRef {
    Group { group: String },
    Section { section: String },
    Collection { collection: String },
    Field { field: String },
    List { list: String },
}
```

If serde ergonomics make that enum awkward, a small tagged struct is also acceptable:

```rust
pub struct HierarchyChildRef {
    pub kind: HierarchyChildKind,
    pub id: String,
}
```

What matters is explicit typed containment, not a specific serde trick.

## Runtime Model

Introduce a runtime tree that matches authored structure instead of flattening back into the old section-only shape.

Recommended runtime direction:

```rust
pub struct RuntimeTemplate {
    pub id: String,
    pub children: Vec<RuntimeNode>,
}

pub enum RuntimeNode {
    Group(RuntimeGroup),
    Section(RuntimeSection),
    Collection(RuntimeCollection),
}

pub struct RuntimeGroup {
    pub id: String,
    pub label: String,
    pub nav_label: String,
    pub children: Vec<RuntimeNode>,
}

pub struct RuntimeSection {
    pub id: String,
    pub label: String,
    pub nav_label: String,
    pub body: RuntimeSectionBody,
    pub children: Vec<RuntimeSectionChild>,
    pub note: RuntimeNoteMeta,
}

pub struct RuntimeCollection {
    pub id: String,
    pub label: String,
    pub nav_label: String,
    pub lists: Vec<HierarchyList>,
    pub note: RuntimeNoteMeta,
}
```

The exact struct names may differ, but the end state must support:

- authored ordering
- typed parent/child relationships
- first-class collections
- section content that can own fields and lists

## Runtime State Model

Current `SectionState` is still a useful concept for interactive leaf content, but it should no longer be the whole runtime topology.

Recommended direction:

- keep per-leaf interactive state enums
- map them by node ID
- introduce a navigation index derived from the runtime tree

Possible shape:

```rust
pub enum NodeState {
    Section(SectionState),
    Collection(CollectionState),
}

pub struct NavigationEntry {
    pub node_id: String,
    pub group_id: String,
    pub depth: usize,
    pub nav_label: String,
}
```

This allows:

- tree-authored rendering
- a flat navigation slice for the current TUI
- future nested display without redoing state identity

## Note/Layout Metadata Model

`note_render_slot` should not survive this cutover as the main layout mechanism.

Replace it with explicit metadata attached to groups and nodes.

Recommended first-pass metadata:

```rust
pub struct NoteNodeMeta {
    pub heading: Option<String>,
    pub heading_search_text: Option<String>,
    pub top_level_heading: Option<String>,
    pub is_intake: bool,
}
```

Purpose:

- structural order comes from `contains`
- wrappers/headings are driven by data, not slot-name matches

This still allows the current note layout to be reproduced, but without scattering those rules through `note.rs`.

## Implementation Strategy

This is a one-pass cutover in outcome, but it should still be executed in disciplined phases so the codebase stays testable during the change.

### Phase 0. Baseline capture

Before editing, record the current state:

- `cargo check`
- `cargo test`
- note any failing tests already caused by in-progress collection work

Purpose:

- separate pre-existing breakage from cutover regressions
- give reviewers a clean before/after baseline

### Phase 1. Replace hierarchy structs with the new grammar

Files:

- [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)

Changes:

1. Add `collections` to `HierarchyFile`.
2. Replace:
   - `HierarchyTemplate.groups`
   - `HierarchyGroup.sections`
   - `HierarchySection.fields`
   - `HierarchySection.lists`
   with typed `contains`.
3. Add `HierarchyCollection`.
4. Remove old structural fields that no longer belong in the grammar.
5. Keep `Field`, `List`, and `Item` as reusable globally merged primitives.

Validation rules to add or update:

- `template.contains` may only contain `group` refs
- `group.contains` may only contain `section` or `collection` refs
- `section.contains` may only contain `field` or `list` refs
- `collection.contains` may only contain `list` refs
- every typed ref must resolve by kind and `id`
- duplicate `(kind, id)` pairs remain illegal
- duplicate `id` values across structural and runtime-relevant kinds also remain illegal

Locked decision:

- reject shared IDs across merged kinds during loader validation
- fail loudly with the conflicting kinds and the duplicated `id`

Reason:

- runtime state, navigation, node lookup, and document markers are all simpler and safer when `id` is globally unique
- this cutover is not the right time to carry `(kind, id)` identity everywhere just to preserve same-name reuse

### Phase 2. Rebuild `load_hierarchy_dir()` around typed containment validation

Files:

- [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)

Changes:

1. Extend merge logic to include `collections`.
2. Replace old cross-ref checks:
   - `template -> groups`
   - `group -> sections`
   - `section -> fields/lists`
   with a generic typed-ref validator.
3. Add parent-kind/child-kind grammar enforcement.
4. Update cycle detection if containment becomes more general than before.
5. Reject any shared `id` across merged structural/runtime kinds with a noisy error.

Expected behavior:

- unresolved typed refs fail loudly
- wrong-child-kind refs fail loudly
- YAML files can remain split arbitrarily because resolution is global

Important note:

The user explicitly wants file boundaries not to matter for lookup. The loader should continue to scan and merge every data file first, then validate the graph second.

### Phase 3. Replace `hierarchy_to_runtime()` with tree construction

Files:

- [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)

Changes:

1. Stop returning only:
   - `Vec<SectionGroup>`
   - `Vec<SectionConfig>`
   - `collection_data`
2. Introduce a real runtime hierarchy result, likely something like:
   - `RuntimeTemplate`
   - flat navigation index
   - per-node lookup maps as needed
3. Resolve all typed refs into concrete runtime children.
4. Remove ambient `top_lists.clone()` population for `collection_data` and `block_select_data`.
5. Resolve collection-owned lists only from the collection’s `contains`.
6. Do not support embedded collections in this cutover:
   - collections are authored directly under groups
   - sections do not own collection children in the runtime tree
   - any section -> collection containment should fail validation for now
7. Add a node lookup that records, for every runtime node:
   - node `id`
   - node kind
   - parent node `id` if any
   - whether the node owns its own managed document block or renders inside its parent block

Critical result:

After this phase there should be no code path where a collection sees an unrelated list unless it names that list explicitly.

Additional rule for this cutover:

- collections are first-class runtime nodes only as direct group children
- section-contained collections are explicitly out of scope until there is a real product use-case

### Phase 4. Decide how legacy section behaviors map into the new structure

Files:

- [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)
- [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs)
- [src/sections/*.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/sections)

This cutover still needs leaf behavior kinds for the current app:

- multi-field style input
- free text
- list select
- checklist
- collection toggle UI

There are two viable ways to encode that:

Option A: keep a typed section body enum

```rust
pub enum RuntimeSectionBody {
    MultiField { fields: Vec<HeaderFieldConfig> },
    FreeText,
    ListSelect { lists: Vec<HierarchyList>, date_prefix: bool },
    Checklist { lists: Vec<HierarchyList> },
}
```

Option B: infer section body behavior from its contained children

- field children => multi-field
- one or more list children => list-driven leaf mode
- empty contains => free-text

Recommendation:

Use Option A for this cutover.

Why:

- the current app already has meaningful behavior differences
- some section modes are UI/runtime concepts, not purely structural concepts
- trying to infer all behavior from child shape in the same cutover adds unnecessary risk

That means:

- remove `section_type: collection`
- keep a narrower non-structural behavior field for actual section body mode if needed
- collections become their own node kind rather than a section mode

If naming matters, rename `section_type` to something less structural, for example:

- `body`
- `mode`
- `input_mode`

Recommendation:

- use `body`

Example:

```yml
sections:
  - id: tx_mods
    label: Treatment Modifications & Preferences
    body: multi_field
    contains:
      - field: pressure_field
      - field: positioning_field
```

That avoids reusing `type` or `section_type` for two different meanings.

Locked scope constraint:

- remove user-added custom-entry persistence from this cutover rather than preserving it through the schema rewrite
- `contains` defines membership and ordering only; this cutover does not define any write-back contract for mutable list sources

### Phase 5. Rebuild app state around runtime nodes instead of flat sections

Files:

- [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs)
- [src/ui.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/ui.rs)

Changes:

1. Replace `App.sections: Vec<SectionConfig>` as the authoritative structure.
2. Introduce:
   - runtime root/tree
   - flat navigation entries derived from the tree
   - state keyed by node ID
3. Keep the wizard/map UX flat for now if needed, but derive it from the tree rather than from `SectionGroup.sections`.
4. Make `group_idx_for_section()` and map hint logic work from navigation entries instead of the old group/section arrays.
5. Define navigation behavior for collections explicitly:
   - direct group-child collections appear in the same flat authored-order navigation slice as other runtime nodes
   - they keep their own node identity and state

Important constraint:

The TUI does not need a visually nested tree in this cutover. It only needs the correct authored order and node ownership semantics.

### Phase 6. Make `CollectionState` a first-class node state

Files:

- [src/sections/collection.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/sections/collection.rs)
- [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs)
- [src/ui.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/ui.rs)

Changes:

1. Stop instantiating collections through section config branches.
2. Initialize `CollectionState` from resolved collection-owned lists.
3. Preserve the current useful behavior:
   - collection activation
   - per-item remembered overrides
   - reset support
4. Key collection state by collection node ID.

Locked decision:

- retire `block_select` in this cutover
- migrate treatment regions fully to `collection`
- remove the separate `block_select` runtime path, loader path, UI path, and tests rather than carrying both concepts forward

Reason:

- the current real data no longer needs two parallel nested-list interaction models
- keeping both would undermine the stated goal of a full cutover

### Phase 7. Rebuild note rendering as structural traversal

Files:

- [src/note.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/note.rs)
- [src/document.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/document.rs)

Changes:

1. Replace slot-driven traversal with tree traversal.
2. Introduce helper(s) like:
   - `render_runtime_note(...)`
   - `render_group(...)`
   - `render_node(...)`
   - `render_managed_node_body(...)`
3. Keep stable managed markers keyed by node `id`.
4. Make visible headings derive from node metadata plus current note layout data.
5. Define collection rendering explicitly:
   - a collection authored directly under a group renders as its own managed node block
   - section-contained collections are not supported in this cutover

Important detail:

The current note layout has two distinct concerns:

- structural order
- wrapper text like `## SUBJECTIVE`, `#### TREATMENT REGIONS`, disclaimers, and separators

Those wrappers should move into explicit layout metadata rather than remain implicit in hard-coded slot names.

A practical first-pass model is:

- group-level metadata for top-level note heading and optional boilerplate/disclaimer block
- node-level metadata for visible subheading text and heading-search text

That is enough to reproduce the current clinical note shape without keeping `note_render_slot`.

### Phase 8. Update editable-document anchor generation and sync

Files:

- [src/document.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/document.rs)
- [src/note.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/note.rs)
- [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs)

Changes:

1. Generate managed anchors from runtime nodes, not only sections.
2. Continue using stable IDs for markers.
3. Ensure visible headings come from the new data-backed layout metadata.
4. Make targeted section/node sync operate on the new node identity map.
5. Lock the ownership rule for managed content:
   - every node that owns a managed block gets exactly one marker pair keyed by its own `id`
   - for this cutover, collections that own managed blocks are direct group children
   - section-contained collections are out of scope and should not be representable in valid data

Recommendation:

- keep the marker format itself unchanged if possible

Example:

```markdown
<!-- scribblenot:section id=tx_regions:start -->
```

Reason:

- the existing marker format is already ID-based and stable
- there is no need to churn document syntax if the ownership model is what is changing

If non-section nodes begin to own managed content, either:

- keep `section id=` as a historical marker label while the IDs stay stable, or
- rename to a more generic `node id=` everywhere in one deliberate change

Recommendation:

- switch to `node id=` during this cutover only if document helpers are already being rewritten heavily
- otherwise keep the marker string stable and treat it as an implementation detail

### Phase 9. Migrate all `data/*.yml` files to the new schema

Files:

- [data/sections.yml](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/data/sections.yml)
- [data/treatment.yml](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/data/treatment.yml)
- note: `boilerplate.yml`, `objective_findings.yml`, `remedial.yml`, `infection_control.yml`, `objective.yml`, `subjective.yml` are flat list-source files with no structural keys and do not need migration

Changes:

1. Rewrite the real data set to use:
   - `contains`
   - `collections`
   - section `body` where needed
2. Remove the transitional comments that describe the current collection workaround.
3. Encode current note/headings metadata in the new fields rather than in `note_render_slot`.
4. Ensure `tx_regions` is defined as a real collection node.
5. Remove custom-entry persistence assumptions from the migrated data contract.
6. Rewrite the authoring reference comment block at the top of `sections.yml` to reflect the new schema.

Recommended direction for current data:

- keep one root template
- keep current group ordering
- move treatment-region authored content fully under a `collections:` block
- keep lists globally defined and reusable by ID
- do not preserve mutable custom-entry write targets in this cutover

### Phase 10. Replace and expand tests

Files:

- [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)
- [src/note.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/note.rs)
- [src/document.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/document.rs)
- [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs)

Add or update tests for:

1. typed child ref deserialization
2. multi-file global merge with cross-file typed refs
3. containment grammar violations
4. unresolved typed refs
5. collection resolution only loads named lists
6. real data directory loads successfully under the new schema
7. runtime tree preserves authored order
8. note rendering follows structural order
9. editable document anchors still use stable IDs
10. collection state uses only its owned lists
11. section-contained collections are rejected by validation for this cutover
12. no custom-entry persistence path remains
13. no `block_select` runtime path remains

Strongly recommended:

- add at least one golden-note style integration test using the real data set or a focused fixture

This will catch future regressions in:

- authored order
- missing wrappers
- heading identity drift
- document marker placement

## Detailed Design Decisions Still Needed In Code

These are implementation decisions, not product-direction questions. They should be resolved during implementation and documented in the final code/comments/tests.

### 1. Exact typed-ref serde shape

Two options:

1. untagged enum like `- section: tx_mods`
2. tagged struct like `- kind: section; id: tx_mods`

Recommendation:

- use the short YAML form the user already chose: `- section: tx_mods`

Reason:

- it matches the agreed authoring ergonomics
- this is the product-facing schema

### 2. How collections are scoped in this cutover

Locked decision:

- collections are supported only as direct children of groups in this cutover
- sections cannot contain collections yet

Reason:

- this preserves the intended product direction without introducing a premature ownership model for nested managed content
- the narrower rule materially reduces migration risk across navigation, note rendering, and document sync

### 3. Whether `block_select` survives

Recommendation:

- remove `block_select` in this cutover

Reason:

- two parallel nested-list interaction models will increase maintenance cost immediately
- this cutover is the right moment to collapse onto one canonical model

Implementation consequence:

- delete the dedicated loader/runtime/UI/test branches for `block_select` instead of translating them forward under a new schema

### 4. How custom-list persistence is handled after the schema cutover

Locked decision:

- remove custom-entry persistence from this cutover

Reason:

- it is not a stabilized product requirement yet
- preserving a mutable write-path through a structural rewrite would add complexity without enough value
- a future persistence design can be introduced deliberately once the typed hierarchy architecture settles

### 5. How note wrapper metadata is stored

Two options:

1. attach wrapper metadata to groups and nodes
2. create a separate top-level note-layout config

Recommendation:

- attach it to groups and nodes for this cutover

Reason:

- lower implementation overhead
- fewer disconnected config concepts
- enough flexibility for the current single-template app

If note layout grows substantially later, a separate layout config can still be introduced.

## Risks

### 1. This cutover touches every major seam at once

Affected areas:

- YAML schema
- loader
- runtime projection
- app state
- navigation
- UI rendering
- note rendering
- editable document sync
- tests

Mitigation:

- keep the branch isolated
- keep the implementation sequence disciplined
- validate after each phase, even inside one branch

### 2. Tree correctness can regress silently without golden tests

Risk:

- authored order, headings, or wrappers can shift while code still compiles

Mitigation:

- add one or more note-output assertions, not only unit tests around structs

### 3. State identity can drift if any path keeps using labels

Risk:

- document markers, navigation identity, or reload behavior can accidentally key off `label`

Mitigation:

- assert in code review and tests that state lookups and markers always use `id`

### 4. Shared IDs would make the new runtime ambiguous

Risk:

- a reused `id` across kinds would collide in runtime state, node lookup, navigation, or managed document sync

Mitigation:

- reject duplicate `id` values across merged structural/runtime kinds during loader validation
- make the error name the conflicting kinds and duplicated `id`

### 5. Attempting too much inference from structure could reintroduce ambiguity

Risk:

- if section runtime behavior is inferred too aggressively from child shapes, author intent becomes unclear and errors become harder to diagnose

Mitigation:

- keep explicit `body` mode for sections while using `contains` only for ownership/order

## Implementation Checklist

Use this as the concrete implementation sequence. Each step should leave the branch compiling or very close to compiling before moving on.

### 1. `src/data.rs` schema and loader contract

- replace `HierarchyTemplate.groups` with typed `contains`
- replace `HierarchyGroup.sections` with typed `contains`
- replace section-owned structural `fields`/`lists` assumptions with `contains`
- add `HierarchyCollection`
- add `Collection` to `TypeTag` and include collection ID duplicate detection in `load_hierarchy_dir`
- reject shared IDs across merged kinds, not only duplicate `(kind, id)` pairs
- add note/layout metadata structs needed by the new schema
- update `HierarchyFile` merge logic to include `collections`
- replace old reference validation with typed containment validation
- enforce parent-kind/child-kind grammar rules
- keep global merge first, validation second
- keep ID-based duplicate detection explicit and well-tested

### 2. `src/data.rs` runtime projection

- replace the old `RuntimeHierarchy` tuple with a runtime tree result plus navigation/index helpers
- build runtime nodes from typed refs in authored order
- add node lookup metadata:
  node id, kind, parent id, document-owner rule
- make collections first-class runtime nodes as direct group children
- remove ambient top-level list injection for both `collection` and `block_select`
- delete `block_select` runtime projection paths

### 3. `src/app.rs` runtime state and navigation

- replace `App.sections` as the authoritative source with runtime tree plus navigation entries
- key interactive state by runtime node id
- instantiate `CollectionState` from real collection nodes, not section-type branches
- remove `block_select` state wiring
- make flat navigation derive from authored tree order
- define direct group-child collections as ordinary flat navigation steps for this UI pass
- update targeted sync helpers to work from node identity rather than flat section assumptions

### 4. `src/ui.rs` and `src/sections/*` interaction layer

- remove `block_select` UI/render paths
- keep `CollectionState` behavior intact: activation, remembered overrides, reset
- update map rendering and hints to consume navigation entries instead of `SectionGroup.sections`

### 5. `src/note.rs` structural rendering

- replace `note_render_slot` traversal with runtime tree traversal
- move heading/wrapper decisions onto explicit node/group metadata
- define one renderer rule for collections as direct group-child managed nodes
- keep stable marker IDs
- remove hard-coded special cases that were only compensating for slot-driven layout

### 6. `src/document.rs` editable note structure

- generate anchor specs from runtime nodes
- preserve stable marker format unless the rewrite makes a generic `node id=` migration clearly cleaner
- implement the document-owner rule consistently:
  every owner node gets one marker pair, non-owners render inside nearest owner
- keep canonical top-level headings validation working with the new metadata path

### 7. `data/*.yml` migration

- migrate template, groups, sections, and treatment data to typed `contains`
- define treatment regions as a real `collections:` entry
- remove `section_type: collection`
- remove `note_render_slot`
- keep authored order matching the intended clinical note order

### 8. Test replacement and expansion

- replace loader tests that assume `template.groups` / `group.sections`
- add typed-ref deserialization and grammar violation tests
- add runtime tree ordering tests
- add tests proving collections only load named lists
- add tests proving section-contained collections are rejected by validation
- add tests proving no custom-entry persistence path remains
- add tests proving no `block_select` path remains
- add at least one golden-note integration test

### 9. Validation pass

- run `cargo check`
- run `cargo test`
- verify real data directory load
- verify note rendering order and wrappers on real data
- verify editable document markers and targeted replacement
- manually confirm collection toggles and remembered overrides still behave correctly
- manually confirm treatment regions show only explicitly named lists

## Validation Plan

Minimum validation for the cutover:

1. `cargo check`
2. `cargo test`
3. real data directory load test passes
4. at least one golden-note render test passes
5. manual smoke check in headless or normal app run confirms:
   - current group order is preserved
   - treatment regions show only named lists
   - collections still toggle and remember item overrides
   - editable note markers remain stable

Recommended manual checks on the real data set:

1. Open treatment regions and verify it shows only the lists explicitly named by the collection.
2. Toggle a collection on, customize item selection, toggle off, toggle on again, and verify overrides persist until reset.
3. Render the note and confirm treatment sections appear in authored order with the expected headings.
4. Edit the document manually outside a managed block, then update a structured node and verify only the managed block changes.

## Suggested Execution Order

1. Replace hierarchy structs and loader validation.
2. Add `HierarchyCollection` and typed `contains`.
3. Build the runtime tree and remove ambient list loading.
4. Rebuild app state/navigation over runtime nodes.
5. Rewire collection state as a first-class node.
6. Rebuild note rendering as structural traversal.
7. Rewire editable document anchors/sync.
8. Migrate real data files.
9. Update tests.
10. Run full validation.

This order keeps the highest-leverage structural changes first and avoids repeatedly adapting the data files to half-finished runtime assumptions.

## Reviewer Checklist

Reviewers should specifically verify:

- no code path still grants ambient collection membership from merged top-level lists
- no code path still treats `collection` as a `section_type`
- `contains:` resolution is typed and explicit everywhere
- runtime identity is always keyed by `id`, never by `label`
- shared IDs across merged kinds fail loudly in loader validation
- note traversal follows authored structure rather than `note_render_slot`
- editable document markers remain stable and targeted replacement still works
- real data files no longer depend on compatibility shims

## Immediate Next Step

If this plan is approved, implementation should begin in [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs) by replacing the hierarchy structs and validation model first. Everything else depends on that new structural contract being explicit.

## Changelog

### Review - 2026-04-08
- #1: Added `TypeTag::Collection` bullet to checklist step 1 to make collection ID duplicate detection explicit
- #2: Replaced vague "any other data/*.yml" note in Phase 9 with an explicit list of files that do not need migration
- #3: Added step 6 to Phase 9 Changes to rewrite the authoring comment block in sections.yml
