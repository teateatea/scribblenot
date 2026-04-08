# Runtime Field Cutover

## Task

Implement the canonical field model end-to-end. This is a full cutover from legacy composite/select runtime behavior to the canonical YAML hierarchy model:

- `Item` carries UI/search text and note-output text
- `List` owns picker metadata and item collections
- `Field` owns ordered list references plus optional formatting

This is not a compatibility mission. Old YAML field shapes are removed rather than preserved.

## Context

Mission 13 introduced the 6-level hierarchy and kept runtime behavior compatible with the old field model. That compatibility layer is now the blocker.

The current real-data failure is in [sections.yml](C:\Users\solar\Documents\Claude Projects\scribblenot\data\sections.yml): the new `date_field` uses canonical list items with `output` only, but `HierarchyItem` in [data.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\data.rs) still requires `id`.

This cutover should make the runtime match the canonical spec from `DISCUSSION-yaml-data-hierarchy.md`, so future section authoring happens against one clean model instead of a legacy-compatible hybrid.

## Decisions Locked In

### 1. Runtime field shape

Runtime fields store resolved lists directly.

Recommended shapes:

```rust
pub struct RuntimeItem {
    pub id: Option<String>,
    pub label: Option<String>,
    pub output: String,
}

impl RuntimeItem {
    pub fn ui_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.output)
    }
}

pub struct RuntimeFieldList {
    pub id: String,
    pub label: String,
    pub preview: Option<String>,
    pub sticky: bool,
    pub default: Option<String>,
    pub items: Vec<RuntimeItem>,
}

pub struct HeaderFieldConfig {
    pub id: String,
    pub name: String,
    pub format: Option<String>,
    pub lists: Vec<RuntimeFieldList>,
    pub repeat_limit: Option<usize>,
}
```

`SectionConfig` should carry these updated `HeaderFieldConfig` values directly. Legacy runtime fields like `options` and `composite` should be removed.

### 2. Item semantics

- `label` is for UI display and search/picking
- `output` is for preview text and copied/exported note text
- UI display falls back from `label` to `output`

### 3. Field semantics

- `lists:` order controls user interaction order
- `format` placeholders refer to list IDs
- `format` substitutions always use selected item `output`

### 4. Sticky/default/preview order

Per list, value resolution order is:

1. confirmed selection
2. sticky
3. default
4. preview

When opening a picker, initial selection order is:

1. sticky
2. default
3. first item

Sticky persistence is per `list_id`, not per field.

### 5. Format-less fields

- one-list field with no `format`: output the selected item `output`
- multi-list field with no `format`: load-time error

### 6. Unified field model

All selectable fields are just fields. There is no separate runtime concept for:

- simple select
- composite field

A one-list field is the simple case.

## Scope

### In scope

- [data.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\data.rs)
- [app.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\app.rs)
- [modal.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\modal.rs)
- [ui.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\ui.rs)
- [multi_field.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\sections\multi_field.rs)
- [note.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\note.rs)
- [sections.yml](C:\Users\solar\Documents\Claude Projects\scribblenot\data\sections.yml)
- any other `data/*.yml` files that still use removed field shapes

### Out of scope

- preserving backward compatibility for old YAML field shapes
- introducing a transitional runtime bridge back into composite structs
- unrelated UI redesign

## Critical Current Mismatches

### YAML layer mismatch

In [data.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\data.rs):

- `HierarchyItem` still requires `id` and `label`
- `HierarchyList` does not yet carry full canonical list metadata
- `HierarchyField` still uses legacy `field_type`, `options`, `list_id`, `composite`

### Runtime mismatch

In [app.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\app.rs), [modal.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\modal.rs), [ui.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\ui.rs), and [multi_field.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\sections\multi_field.rs):

- runtime still assumes composite parts
- modal progress still tracks composite-part advancement
- preview rendering still depends on composite structures

## Implementation Plan

### Phase 1: Canonicalize YAML structs in `data.rs`

1. Update `HierarchyItem`
- make `id` optional
- make `label` optional
- keep `output` required
- add helper methods for UI fallback if useful

2. Update `HierarchyList`
- include canonical metadata:
  - `id`
  - `label`
  - `preview`
  - `sticky`
  - `default`
  - `items`

3. Update `HierarchyField`
- remove:
  - `field_type`
  - `options`
  - `list_id`
  - `composite`
- add:
  - `format`
  - `lists`

4. Remove dead YAML/runtime legacy types from [data.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\data.rs)
- `PartOption`
- `CompositePart`
- `CompositeConfig`
- any related impl/test code that only exists for the removed field model

5. Update runtime config structs in [data.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\data.rs)
- `HeaderFieldConfig` should carry `format` and resolved lists
- remove `options` and `composite`
- update `SectionConfig` accordingly

### Phase 2: Update loader validation and runtime conversion

6. Update cross-reference validation in `load_hierarchy_dir`
- validate every `HierarchyField.lists` reference
- error clearly on missing list IDs

7. Update cycle/reference graph construction
- replace any `list_id`-specific logic with iteration over `lists`

8. Update `hierarchy_to_runtime`
- resolve `HierarchyField.lists: Vec<String>` into concrete runtime lists
- clone/copy canonical list metadata into runtime field lists
- ensure list order is preserved exactly as authored

9. Enforce format-less field rule at load time
- one-list field with no `format` is allowed
- multi-list field with no `format` is an error

### Phase 3: Replace composite runtime flow

10. Refactor [modal.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\modal.rs)
- remove `CompositeModal`
- track current list index instead of composite-part index
- modal entries should display item UI labels

11. Refactor [app.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\app.rs)
- remove composite-specific modal creation and navigation
- advance through a field by list order
- save sticky values keyed by `list_id`
- preserve repeat-limit behavior for fields

12. Refactor [ui.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\ui.rs)
- replace composite preview rendering with list-based formatting
- render unresolved values using list `preview`
- preserve visual distinction between auto-filled and confirmed values

13. Refactor [multi_field.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\sections\multi_field.rs)
- resolve field values from resolved lists
- format using selected item `output`
- keep partial/complete/empty state behavior where still useful

### Phase 4: Update real YAML data to spec

14. Update [sections.yml](C:\Users\solar\Documents\Claude Projects\scribblenot\data\sections.yml)
- keep the canonical `date_field`
- remove the old duplicate `date` composite block
- migrate any remaining fields still using removed field keys

15. Audit other `data/*.yml`
- remove remaining old field shapes if any exist
- ensure new authoring follows the canonical discussion spec

### Phase 5: Rewrite tests to the new model

16. Update unit tests in [data.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\data.rs)
- rewrite hierarchy field/item/list tests to canonical YAML
- add regression for output-only list items
- add regression for field `lists + format`
- remove legacy composite-struct tests

17. Update runtime tests in:
- [app.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\app.rs)
- [note.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\note.rs)
- [multi_field.rs](C:\Users\solar\Documents\Claude Projects\scribblenot\src\sections\multi_field.rs)

Replace composite-era fixtures/assertions with:
- one-list field behavior
- multi-list formatted behavior
- sticky/default/preview ordering
- UI label vs output distinction

## Expected Breakpoints

These are the likely compiler/test failures during implementation:

- missing-field parse failures in real-data tests until `HierarchyItem` and `HierarchyField` are updated
- compile failures in `app.rs`, `modal.rs`, `ui.rs`, and `multi_field.rs` immediately after composite types are removed
- note/export test failures until output formatting is switched to list-based resolution
- sticky/default behavior regressions if the state model is still keyed by field instead of list

## Verification

### Automated

- `cargo test`
- targeted checks that the real `data/` directory loads under the new schema

Required assertions:

- real [sections.yml](C:\Users\solar\Documents\Claude Projects\scribblenot\data\sections.yml) parses successfully
- the canonical `date_field` works
- `tx_mods` still behaves correctly after conversion to the unified field model
- one-list field without `format` works
- multi-list field without `format` fails loudly at load time
- UI label and note output semantics are distinct and correct
- sticky values are reused per list ID

### Manual

- open the header field flow and confirm the date field steps through list pickers in authored order
- confirm auto-filled values appear as preview placeholders/defaults before confirmation
- confirm confirmed values change to the “confirmed” styling
- confirm copied/exported note text uses item `output`, not item `label`

## Success Criteria

The task is complete only when all of the following are true:

- old YAML field shapes are removed from real data
- runtime no longer depends on composite field types
- `HeaderFieldConfig` uses resolved lists directly
- the app works with canonical field/list/item semantics end-to-end
- `cargo test` passes
- future data authoring can use the canonical field model without needing legacy exceptions

## Recommendation For Review

Reviewers should evaluate this mission as a structural cutover, not a serializer tweak.

The main questions are:

- does the runtime model now match the canonical YAML spec directly?
- are label/output semantics handled consistently across UI and note export?
- is sticky/default/preview behavior coherent and test-covered?
- has legacy composite logic been fully removed rather than half-preserved?
