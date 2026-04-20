## Task

#74 Full Cutover - Replace the section-centric hierarchy with typed `contains:` nodes and first-class collections

## Status

User Approved

This is intentionally a full cutover, not a compatibility layer. Old YAML shapes and transitional runtime assumptions should be removed rather than preserved.

## Reality Check - 2026-04-18

This task is partially implemented already.

What is already landed in the repo:

- typed `contains:` refs are live in `src/data.rs`
- `HierarchyCollection` exists and real data already uses `collections`
- loader validation already enforces typed containment rules and globally unique IDs across runtime-relevant kinds
- global merge still works across `data/*.yml`
- collections now resolve only the lists they name explicitly; the old ambient merged-list behavior is already gone
- `cargo test --quiet` currently passes (`139` tests on 2026-04-18)

What is still not cut over:

- `hierarchy_to_runtime()` still projects back into `SectionGroup` + `SectionConfig` for app use
- `RuntimeTemplate` / `RuntimeGroup` / `RuntimeNode` exist, but they are not yet the authoritative app-facing model
- `App` still treats `App.sections` as the real structure and navigates through flat section ordering plus `group_id`
- note rendering and editable-document sync still operate on flat section/group structures rather than traversing the authored runtime tree
- collections are still consumed through `section_type == "collection"` and `SectionState::Collection`, so they remain section-shaped in the app layer even though the loader distinguishes them structurally

Implementation handoff should treat schema/loader work as the starting point, not as future work.

## Why This Plan Exists

The current codebase already has a merged multi-file YAML loader and a typed ID registry, but the runtime still collapses back into the older model:

- groups and collections are distinguished structurally in the loader, but the app still flattens them back into section-centric runtime products
- `collection` is still consumed through a special `section_type` path in the app layer
- note layout and document sync still derive from flat group/section traversal rather than the authored runtime tree

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

### 1. `src/data.rs` has the new schema, but the app-facing runtime is still flattened

The loader is already on the new side of the cutover:

- `load_hierarchy_dir()` scans `data/`
- merges every `.yml` into one `HierarchyFile`
- validates typed refs and global IDs
- builds `RuntimeTemplate` / `RuntimeGroup` / `RuntimeNode`

The mismatch is that the same pass still emits:

- `Vec<SectionGroup>`
- `Vec<SectionConfig>`
- `collection_data`

Those flattened products are still what the app actually consumes.

### 2. Collections are structurally first-class, but still section-shaped at runtime use sites

Collection ownership is already explicit in the loader, but the app still consumes collections through:

- `SectionConfig { section_type: "collection", node_kind: RuntimeNodeKind::Collection, ... }`
- `SectionState::Collection`
- `data.collection_data.get(&cfg.id)`

So the schema distinction exists, but the interaction model still routes through the old section-centric path.

### 3. Note rendering is flatter than the authored tree

`src/note.rs` no longer depends on `note_render_slot`, which is good progress, but it still renders by walking:

- `groups: &[SectionGroup]`
- each group's `sections`
- `managed_heading_for_section(&SectionConfig)`

That means note order and managed-block ownership still come from the flattened projection instead of the authored runtime tree.

### 4. Editable-document sync is ID-stable, but still section-only

The marker format is already stable and ID-backed:

```markdown
<!-- scribblenot:section id=tx_mods:start -->
```

What is still missing is tree-aware ownership:

- anchor specs are generated from flat `SectionConfig` entries
- replacement targets sections only
- collections can only participate by masquerading as section-shaped nodes

### 5. App navigation still uses flat section indexes plus `group_id`

In [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs):

- `App.sections` is still the authoritative ordering
- `group_idx_for_section()` resolves by `section.group_id`
- map hints and preview sync still reason about flat section indexes

That is the main remaining structural blocker.

## Target End State

## Schema Model

Introduce typed structural references.

For this task, typed `contains` and node metadata are required. An authored `body` field is optional future refinement and should stay out of scope unless mode inference proves insufficient during implementation.

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

This is now a finish-the-cutover plan, not a start-from-scratch plan.

Earlier schema and loader phases are already complete enough to build on. The remaining work is to make the existing runtime tree authoritative across the app, note renderer, and editable-document sync.

Important scope adjustment for handoff:

- do not reopen the schema unless a concrete blocker appears
- do not fold roadmap item `22` (`body` enum) into this task unless implementation proves the inferred mode model is blocking the cutover
- keep the current marker syntax unless the document rewrite makes a generic rename clearly cheaper than preserving it

### Phase 0. Baseline capture

Current baseline on 2026-04-18:

- `cargo test --quiet` passes (`139` tests)

Before editing further, also record:

- `cargo check`

Purpose:

- separate new regressions from the already-stable partial implementation
- preserve a clear handoff baseline

### Phase 1. Make the runtime tree the real contract

Files:

- [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)

Changes:

1. Keep `HierarchyFile`, typed `contains`, and loader validation as the baseline.
2. Promote `RuntimeTemplate` / `RuntimeGroup` / `RuntimeNode` from loader byproduct to authoritative runtime structure.
3. Add explicit app-facing lookup/index data alongside that tree:
   - node lookup by stable `id`
   - authored-order navigation entries
   - parent/group ownership metadata
   - managed-document ownership metadata
4. Stop treating `Vec<SectionGroup>` and `Vec<SectionConfig>` as the primary runtime contract.
5. Keep explicit collection ownership exactly as it works today: a collection only sees the lists named in its own `contains`.

Critical result:

- after this phase, the runtime tree is the source of truth
- any flat navigation or rendering slices are derived views, not primary structure

### Phase 2. Rebuild app state and navigation over runtime nodes

Files:

- [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs)
- [src/ui/mod.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/ui/mod.rs)
- [src/sections/collection.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/sections/collection.rs)

Changes:

1. Replace `App.sections` as the authoritative structure with runtime-node navigation entries.
2. Keep the current flat wizard/map UX, but derive it from authored tree order.
3. Replace `group_id` lookups and `group_idx_for_section()` assumptions with navigation metadata from the runtime tree.
4. Keep leaf interaction state, but key it by stable node identity rather than by flat section position alone.
5. Stop routing collections through "section-shaped" app assumptions wherever possible.

Important constraint:

- the UI can remain visually flat in this task
- the structural source of truth must stop being flat

### Phase 3. Rebuild note rendering and editable-document sync from the runtime tree

Files:

- [src/note.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/note.rs)
- [src/document.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/document.rs)

Changes:

1. Replace flat group/section traversal with runtime-tree traversal.
2. Introduce helper(s) like:
   - `render_runtime_note(...)`
   - `render_group(...)`
   - `render_node(...)`
   - `render_managed_node_body(...)`
3. Keep stable managed markers keyed by node `id`.
4. Make visible headings and wrapper text derive from group/node metadata already carried by the runtime tree.
5. Let collections participate as first-class managed nodes rather than only as section-shaped leaves.

Important detail:

The current note layout has two distinct concerns:

- structural order
- wrapper text like `## SUBJECTIVE`, `#### TREATMENT REGIONS`, disclaimers, and separators

Those wrappers should move into explicit layout metadata rather than remain implicit in hard-coded slot names.

A practical first-pass model is:

- group-level metadata for top-level note heading and optional boilerplate/disclaimer block
- node-level metadata for visible subheading text and heading-search text

That is enough to reproduce the current clinical note shape without reintroducing hard-coded structural traversal.

### Phase 4. Cleanup and schema/runtime consolidation

Files:

- [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs)
- [src/app.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/app.rs)
- [data/sections.yml](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/data/sections.yml)

Changes:

1. Remove flattened compatibility helpers that are no longer needed once app/note/document use the runtime tree.
2. Remove section-shaped collection glue that only existed for the transitional app layer.
3. Clean up stale authoring comments or docs that still describe the already-removed older model.
4. Keep any additional schema cleanup tightly scoped to what the cutover actually needs; do not silently absorb roadmap item `22` unless required.

### Phase 5. Replace and expand tests

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
5. runtime tree preserves authored order
6. app navigation entries preserve authored order and group ownership
7. note rendering follows structural order from the runtime tree
8. editable document anchors still use stable IDs after tree-based traversal
9. collections render and sync as first-class runtime nodes
10. no code path still depends on flat `SectionGroup.sections` as the structural source of truth

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

- navigable collection nodes in the runtime tree are direct children of groups in this cutover
- field-level collection references used inside modal composition are already part of the current canonical field model and are not the same thing as top-level navigable collection nodes

Reason:

- this keeps authored navigation and document ownership simple without blocking the existing field-composition behavior
- the narrower rule materially reduces migration risk across navigation, note rendering, and document sync

### 3. Whether `block_select` survives

Resolved status:

- `block_select` should remain removed from the active cutover path

Implementation consequence:

- do not reintroduce a parallel `block_select` path while finishing the runtime/app/document cutover

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

### 5. Broad schema cleanup can reintroduce ambiguity during the runtime cutover

Risk:

- if this task tries to solve every remaining schema question at once, implementation can stall or reintroduce avoidable breakage

Mitigation:

- keep this task focused on making the runtime tree authoritative
- leave authored `body` as roadmap item `22` unless a concrete blocker forces it into scope

## Implementation Checklist

Use this as the concrete implementation sequence. Each step should leave the branch compiling or very close to compiling before moving on.

### 1. Confirm the existing foundation and freeze scope

- record `cargo check`
- record `cargo test`
- treat typed `contains`, `HierarchyCollection`, and explicit collection-owned lists as already-landed baseline behavior
- do not widen this task into roadmap item `22` unless necessary

### 2. Make `RuntimeTemplate` authoritative in `src/data.rs`

- extend the runtime result with node lookup metadata and authored-order navigation entries
- keep the tree authoritative and flat views derived
- stop treating `SectionGroup` / `SectionConfig` as the long-term structural output

### 3. Rebuild `src/app.rs` and `src/ui/mod.rs` over runtime-node navigation

- replace `App.sections` as the structural source of truth
- move map/wizard navigation to derived runtime-node entries
- remove `group_id`-driven structural assumptions
- keep leaf interaction behavior intact while rekeying state to stable node identity

### 4. Rebuild `src/note.rs` and `src/document.rs` over runtime-tree traversal

- render groups and nodes from the authored tree
- generate anchor specs from runtime-node ownership
- keep marker IDs stable
- let collections render and sync as real runtime nodes

### 5. Remove transitional flattening glue

- delete helpers and fields that only existed to bridge back into the old section-centric model
- update authoring comments/docs so they describe the actual current schema/runtime contract

### 6. Expand regression coverage

- add runtime-tree ordering tests
- add navigation-order tests
- add tree-based note-render tests
- add editable-document sync tests for collection-owned and section-owned managed blocks
- add at least one real-data golden-note integration test

### 7. Validation pass

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
   - note output follows authored runtime-tree order, not legacy flattened ordering

Recommended manual checks on the real data set:

1. Open treatment regions and verify it shows only the lists explicitly named by the collection.
2. Toggle a collection on, customize item selection, toggle off, toggle on again, and verify overrides persist until reset.
3. Render the note and confirm treatment sections appear in authored order with the expected headings.
4. Edit the document manually outside a managed block, then update a structured node and verify only the managed block changes.
5. Confirm map navigation and preview sync still land on the correct group/node after the flat section list is removed as the structural source.

## Suggested Execution Order

1. Record the current green baseline.
2. Finish `src/data.rs` runtime outputs so the tree, lookup map, and navigation entries are authoritative.
3. Move `src/app.rs` and `src/ui/mod.rs` to derived runtime-node navigation.
4. Move `src/note.rs` and `src/document.rs` to tree-based traversal and ownership.
5. Remove transitional flattening glue and stale comments/docs.
6. Expand tests.
7. Run full validation.

This order starts from the code that is already landed and attacks the remaining structural bottleneck directly instead of replaying completed schema work.

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

Implementation should resume in [src/data.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/data.rs) by making the existing runtime tree authoritative for downstream callers: add node lookup/navigation metadata, then switch `App` to consume that instead of the flattened `SectionGroup` / `SectionConfig` projection.

## Changelog

### Review - 2026-04-18
- Added a reality-check section documenting that task 16 is partially implemented already.
- Replaced outdated fault-line notes that still described pre-`contains` / pre-collection-ownership behavior.
- Rewrote the implementation strategy, checklist, validation plan, and execution order around the remaining work: runtime/app/document cutover.
- Updated the immediate next step so implementation resumes from the current repo state instead of replaying completed schema work.

### Review - 2026-04-08
- #1: Added `TypeTag::Collection` bullet to checklist step 1 to make collection ID duplicate detection explicit
- #2: Replaced vague "any other data/*.yml" note in Phase 9 with an explicit list of files that do not need migration
- #3: Added step 6 to Phase 9 Changes to rewrite the authoring comment block in sections.yml
