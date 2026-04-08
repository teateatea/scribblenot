## Task

No exact TASKS.md match. Most related: #74 (Remove dead fields from hierarchy structs) — overlapping scope, but this plan reshapes structs for the new item-list-field YAML format rather than simply pruning unused fields.

## Context

Mission 13 implemented the 6-level YAML hierarchy (HierarchyFile -> HierarchyGroup -> HierarchySection -> HierarchyField -> HierarchyList -> HierarchyItem), but the structs were written to match the old composite-parts YAML format. The new authoring format uses flat lists of items referenced by ID from fields, with a `format:` string on the field for composite output. The structs need to be reshaped to accept this new YAML before sections.yml can be rewritten in the modern format.

The key mismatch:
- `HierarchyItem` requires `id` and `label` even when `output` alone is sufficient (e.g. `- output: "01"`)
- `HierarchyList` has no `preview` or `sticky` metadata
- `HierarchyField` has `field_type`, `options`, `list_id`, and `composite` — none of which exist in the new format; the new format uses `format` + `lists`
- `CompositeConfig` / `CompositePart` / `PartOption` become dead types once `HierarchyField.composite` is removed

## Approach

Update the three hierarchy structs in `src/data.rs` in-place, delete the now-dead legacy composite types, update the validation logic and `hierarchy_to_runtime` to use the new field names, and fix the affected tests. Runtime consumers (`modal.rs`, `app.rs`) that reference the deleted composite types will be left broken as known follow-on scope.

## Critical Files

- `src/data.rs`
  - `PartOption` enum + impl: lines 10-43
  - `CompositePart` struct + impl: lines 48-71
  - `CompositeConfig` struct: lines 74-77
  - `HeaderFieldConfig` struct: lines 80-89
  - `SectionConfig` struct: lines 100-120
  - `HierarchyItem` struct: lines 524-530
  - `HierarchyList` struct: lines 533-537
  - `HierarchyField` struct: lines 540-551
  - `part_option_default_tests` module: lines 402-429
  - `hierarchy_field_deserializes` test: lines 2234-2244
  - `hierarchy_field_deserializes_with_list_id_and_data_file` test: lines 2246-2256
  - `load_hierarchy_dir` list_id cross-ref block: lines 808-815
  - `hierarchy_to_runtime` HierarchyField mapping: lines 620-629

## Reuse

- Existing `#[serde(default)]` attribute pattern already used throughout the file — apply the same to new optional fields.
- Existing `load_hierarchy_dir` cycle-detection and cross-ref structure — extend the list-ref loop rather than replacing the whole validation block.

## Steps

1. **Update `HierarchyItem`** (lines 524-530): make `id` and `label` optional and add `effective_label()`.

```
- pub id: String,
- pub label: String,
+ #[serde(default)]
+ pub id: Option<String>,
+ #[serde(default)]
+ pub label: Option<String>,
```

Add `impl HierarchyItem` block immediately after the struct:

```
+ impl HierarchyItem {
+     pub fn effective_label(&self) -> &str {
+         if let Some(ref l) = self.label { return l; }
+         if let Some(ref o) = self.output { return o; }
+         if let Some(ref i) = self.id { return i; }
+         ""
+     }
+ }
```

2. **Update `HierarchyList`** (lines 533-537): add `preview` and `sticky`.

```
  pub id: String,
  pub label: Option<String>,
+ #[serde(default)]
+ pub preview: Option<String>,
+ #[serde(default)]
+ pub sticky: Option<bool>,
  pub items: Vec<HierarchyItem>,
```

3. **Update `HierarchyField`** (lines 540-551): replace old fields with `format` and `lists`.

```
  pub id: String,
  pub label: String,
- pub field_type: String,
- #[serde(default)]
- pub options: Vec<String>,
- pub list_id: Option<String>,
  pub data_file: Option<String>,
- pub composite: Option<CompositeConfig>,
+ pub format: Option<String>,
+ pub lists: Option<Vec<String>>,
  pub default: Option<String>,
  pub repeat_limit: Option<usize>,
```

4. **Delete legacy types** (lines 10-77 and 402-429): remove `PartOption` enum + impl, `CompositePart` struct + impl, `CompositeConfig` struct, and `part_option_default_tests` module entirely.

5. **Update `HeaderFieldConfig`** (lines 80-89): remove `options` and `composite`, add `format` and `lists`.

```
  pub id: String,
  pub name: String,
- #[serde(default)]
- pub options: Vec<String>,
- pub composite: Option<CompositeConfig>,
+ pub format: Option<String>,
+ pub lists: Option<Vec<String>>,
  pub default: Option<String>,
  pub repeat_limit: Option<usize>,
```

6. **Update `SectionConfig`** (lines 100-120): same removals and additions.

```
- pub options: Vec<String>,
- pub composite: Option<CompositeConfig>,
+ pub format: Option<String>,
+ pub lists: Option<Vec<String>>,
```

7. **Update `load_hierarchy_dir` cross-ref validation** (lines 808-815): change `list_id` single-ref check to iterate `lists`.

```
- // Field -> list_id ref
- for f in merged.fields.as_deref().unwrap_or(&[]) {
-     if let Some(ref lid) = f.list_id {
-         if !list_ids.contains(lid.as_str()) {
-             return Err(format!("unresolved list_id ref '{}' in field '{}'", lid, f.id));
-         }
-     }
- }
+ // Field -> lists refs
+ for f in merged.fields.as_deref().unwrap_or(&[]) {
+     for lid in f.lists.as_deref().unwrap_or(&[]) {
+         if !list_ids.contains(lid.as_str()) {
+             return Err(format!("unresolved list ref '{}' in field '{}'", lid, f.id));
+         }
+     }
+ }
```

Apply the same change to the cycle-detection adjacency builder (lines ~845-847) which also walks `f.list_id`.

8. **Update `hierarchy_to_runtime`** (lines 620-629): update the struct-literal that maps `HierarchyField` fields to `HeaderFieldConfig`.

```
- options: hf.options.clone(),
- composite: hf.composite.clone(),
+ format: hf.format.clone(),
+ lists: hf.lists.clone(),
```

Also update any inline `HeaderFieldConfig { ... }` or `SectionConfig { ... }` literals in tests within `data.rs` (lines ~1169, ~1262, ~1428, ~1463) to replace `options: vec![], composite: None` with `format: None, lists: None`.

9. **Update tests** (`hierarchy_struct_tests`, lines 2230-2256):
   - `hierarchy_field_deserializes`: remove `field_type: select` from the YAML string, remove `assert_eq!(field.field_type, "select")` and `assert_eq!(field.options, ...)`, add `lists: [alpha_list]` to YAML and assert `field.lists == Some(vec!["alpha_list".to_string()])`.
   - Rename `hierarchy_field_deserializes_with_list_id_and_data_file` to `hierarchy_field_deserializes_with_lists`, update YAML from `field_type: list\nlist_id: my_list` to `lists:\n  - my_list`, assert `field.lists == Some(vec!["my_list".to_string()])`.
   - Scan for any other inline YAML strings in tests containing `field_type:` or `composite:` and update them.

## Verification

### Manual tests

- Run `cargo build` — expected: compile errors only in `modal.rs` and `app.rs` (missing `CompositeConfig`, `CompositeModal`, `new_composite`); no errors in `data.rs` itself.
- Confirm the sections.yml experimental `date_field` block (with `lists: [...]` and `format:`) now deserializes without error by adding a temporary `dbg!` print or checking `cargo test` output.

### Automated tests

- `cargo test` in `src/data.rs` — all `hierarchy_struct_tests` and `hierarchy_loader_tests` should pass after the test updates in Step 9.
- The existing `lower_back_prone_fascial_l4l5` loader regression test exercises `HierarchyItem` deserialization from a real data file; it should continue to pass with `id`/`label` now optional.
