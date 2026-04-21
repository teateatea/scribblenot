## Task
#22 (Part 2 of 2) - Add explicit authored `body:` to hierarchy sections, validate it, and remove inference fallback

## Context
After Part 1, runtime dispatch uses `SectionBodyType`, but authored section behavior is still inferred from child shape. That keeps the loader ambiguous in exactly the place the roadmap item is trying to harden.

There is also an important semantic boundary to preserve:
- `sections:` author authored section bodies
- `collections:` author collection nodes
- `SectionConfig` is shared at runtime, so `SectionBodyType::Collection` still exists, but it should remain a runtime-only synthesized variant for `collection_to_config()`

If Part 2 only adds `body:` and removes `infer_body_mode()` without validation, the loader would still accept nonsensical combinations like `body: free_text` with children or `body: collection` under `sections:`. The validation work is therefore part of the task, not a follow-on cleanup.

## Approach
Land the authored schema in two phases:
1. Add `body: Option<SectionBodyType>` to `HierarchySection`, use it when present, and validate any authored body against the section's child shape.
2. Migrate all real authored sections and all inline YAML test fixtures to include `body:`, then make `body` required and delete `infer_body_mode()`.

The validation contract for authored sections should be:
- `multi_field`: one or more field refs, no direct list refs
- `free_text`: no children
- `list_select`: one or more list refs, no field refs
- `checklist`: one or more list refs, no field refs
- `collection`: invalid under `sections:`; collections must still be authored under `collections:`

## Critical Files
- `src/data.rs` - add `HierarchySection.body`, update `section_to_config`, add authored-body validation in `validate_merged_hierarchy`, remove `infer_body_mode`, and update inline YAML tests that deserialize `sections:`
- `data/sections.yml` - add `body:` to `appointment_section` and update the authoring reference comment from inferred shape rules to explicit body rules
- `data/subjective.yml` - add `body:` to `subjective_section`
- `data/objective.yml` - add `body:` to `objective_section`
- `data/treatment.yml` - add `body:` to `treatment_modifications`, `treatment_section_prone`, and `treatment_section_supine`

## Reuse
- `validate_merged_hierarchy()` is already the central place for authored schema validation and fix-hint errors
- the real-data validation entry point already exists as `cargo run -- --validate-data`
- `collection_to_config()` should keep synthesizing `SectionBodyType::Collection`; no authored `collections:` schema change is needed

## Steps

### Step 1 - Add optional authored `body` to `HierarchySection`
In `src/data.rs`, add:

```diff
 pub struct HierarchySection {
     pub id: String,
     #[serde(default)]
     pub label: Option<String>,
     #[serde(default)]
     pub nav_label: Option<String>,
+    #[serde(default)]
+    pub body: Option<SectionBodyType>,
     #[serde(default = "default_show_field_labels")]
     pub show_field_labels: bool,
     #[serde(default)]
     pub contains: Vec<HierarchyChildRef>,
     #[serde(default)]
     pub note: NoteNodeMeta,
 }
```

Place it before `show_field_labels` so the authored schema reads naturally near the other section metadata.

### Step 2 - Use authored `body` when present, otherwise fall back temporarily
In `section_to_config`:

```diff
-        body: infer_body_mode(&field_configs, &attached_lists),
+        body: section
+            .body
+            .unwrap_or_else(|| infer_body_mode(&field_configs, &attached_lists)),
```

This keeps the tree loading while the authored data and tests are being migrated.

### Step 3 - Add authored-body validation to `validate_merged_hierarchy`
Add a helper invoked from the existing `for section in &file.sections` loop. It should inspect the section's `contains` refs and reject invalid authored combinations with actionable fix hints.

Validation rules in the transitional phase:
- if `section.body` is `None`, allow it temporarily so pre-migration data still validates
- if `section.body` is `Some(SectionBodyType::MultiField)`, require at least one `field:` child and reject any direct `list:` child
- if `section.body` is `Some(SectionBodyType::FreeText)`, require `contains: []`
- if `section.body` is `Some(SectionBodyType::ListSelect | SectionBodyType::Checklist)`, require at least one `list:` child and reject any `field:` child
- if `section.body` is `Some(SectionBodyType::Collection)`, reject it with a fix hint telling the author to move that node under `collections:`

This is the key plan addition. It prevents the typed schema from still silently accepting invalid authored combinations.

### Step 4 - Add focused validation tests before the full migration
In `src/data.rs` test coverage, add or update fixtures so there is explicit coverage for:
- `body: multi_field` deserializing on `HierarchySection`
- `body: free_text` rejecting non-empty `contains`
- `body: list_select` rejecting field children
- `body: collection` rejecting authored sections

These tests should target `validate_merged_hierarchy()` so the failure mode is pinned before real-data migration begins.

### Step 5 - Migrate the real authored section YAML
Add explicit bodies to the six currently authored sections:

`data/sections.yml`
```diff
   - id: appointment_section
+    body: multi_field
     label: Appointment
```

`data/subjective.yml`
```diff
   - id: subjective_section
+    body: multi_field
     label: "Subjective"
```

`data/objective.yml`
```diff
   - id: objective_section
+    body: multi_field
     label: "Objective / Observations"
```

`data/treatment.yml`
```diff
   - id: treatment_modifications
+    body: multi_field
     label: "Treatment - Prone"

   - id: treatment_section_prone
+    body: multi_field
     label: "Treatment - Prone"

   - id: treatment_section_supine
+    body: multi_field
     label: "Treatment - Supine"
```

No authored collection nodes need `body:`. They remain under `collections:` and still synthesize their runtime body via `collection_to_config()`.

### Step 6 - Update the authoring reference comment in `data/sections.yml`
Replace the current inference-oriented guidance with explicit rules:
- `body: multi_field` means the section contains field refs
- `body: free_text` means the section has no children
- `body: list_select` means the section contains list refs
- `body: checklist` means the section contains list refs but renders checklist behavior
- `body: collection` is not valid in `sections:`; use `collections:` instead

Also update the "Contains rules" comment so it says section child kinds are still `field` or `list`, but valid combinations now depend on `body`.

### Step 7 - Update inline YAML fixtures in `src/data.rs`
Once `body:` is being enforced for real data, the inline YAML fixtures in `src/data.rs` also need to be migrated. This is not optional. Many parser/validation/runtime tests currently author sections like:

```yaml
sections:
  - id: s
    contains: []
```

Every such fixture must gain the correct `body:`:
- `contains: []` => `body: free_text`
- `contains: [field: ...]` => `body: multi_field`
- `contains: [list: ...]` => `body: list_select` unless the test is specifically exercising checklist behavior

Do this before making the field required, otherwise a large, noisy compile/test failure wave will hide real regressions.

### Step 8 - Make `body` required and delete inference
After all authored data and test fixtures are migrated:

1. Change `HierarchySection.body` from `Option<SectionBodyType>` to `SectionBodyType`
2. Remove `#[serde(default)]` from that field
3. Change `section_to_config` to use `body: section.body`
4. Delete `infer_body_mode()`

At that point the loader no longer infers runtime section behavior from child shape.

### Step 9 - Make authored-body validation unconditional
Now that `body` is required, tighten the Step 3 validation so it runs for every section instead of only `Some(...)` bodies. The same rules stay in place; only the optional transitional branch disappears.

### Step 10 - Final verification
Run:

```powershell
cargo test
cargo run -- --validate-data
$env:SCRIBBLENOT_HEADLESS="1"; cargo run
```

The expected result is:
- all tests pass
- the real data directory validates with explicit authored bodies
- the app can still initialize against the migrated hierarchy
