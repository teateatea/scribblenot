## Task
#22 (Part 1 of 2) - Replace inferred `section_type: String` with a typed `SectionBodyType` enum across runtime dispatch sites

## Context
`SectionConfig` is the runtime config struct used by both authored sections and synthesized collection nodes. Today it stores a `section_type: String` populated by `infer_body_mode()` and by the hard-coded `"collection"` literal in `collection_to_config()`. Downstream code then dispatches on raw string matches such as `"multi_field"`, `"free_text"`, `"list_select"`, `"collection"`, and `"checklist"`.

That leaves two problems:
- runtime behavior still depends on string literals rather than an exhaustively matched type
- the inspector/debug UI string is coupled to those literals, so the migration needs a stable string representation rather than ad hoc `Debug` formatting

Part 1 is the runtime-typing pass only. No authored YAML changes land here. `Collection` remains a runtime-only synthesized variant for now; authored-section validation belongs in Part 2.

## Approach
Define a serde-aware `SectionBodyType` enum in `src/data.rs`, add a small `as_str()` helper so any string display remains stable, replace `SectionConfig.section_type` with `body: SectionBodyType`, and update all runtime dispatch sites to match on the enum. Then update every inline `SectionConfig` test fixture to use enum variants instead of string literals.

## Critical Files
- `src/data.rs` - add `SectionBodyType`, add `as_str()`, replace `SectionConfig.section_type -> body`, update `infer_body_mode`, `section_to_config`, `collection_to_config`, and `maybe_record_section_lists`
- `src/app.rs` - update `init_states` dispatch plus all inline `SectionConfig` test fixtures
- `src/note.rs` - update `render_section_body`, the test-only `states_for_real_data`, the direct `multi_field` comparison, and inline `SectionConfig` fixtures
- `src/document.rs` - update the test-only `states_for_real_data` helper and inline `SectionConfig` fixture
- `src/ui/mod.rs` - replace the debug inspector's raw `section_type` string read and update the inline `SectionConfig` fixture

## Reuse
- `RuntimeNodeKind` in `src/data.rs` already uses the correct derive/serde pattern for a snake_case enum
- `RuntimeNodeKind` continues to answer "section vs collection"; `SectionBodyType` answers "how this runtime node behaves once opened"

## Steps

### Step 1 - Add `SectionBodyType` and stable string conversion in `src/data.rs`
Insert the enum immediately before `SectionConfig`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionBodyType {
    MultiField,
    FreeText,
    ListSelect,
    Collection,
    Checklist,
}

impl SectionBodyType {
    pub const fn as_str(self) -> &'static str {
        match self {
            SectionBodyType::MultiField => "multi_field",
            SectionBodyType::FreeText => "free_text",
            SectionBodyType::ListSelect => "list_select",
            SectionBodyType::Collection => "collection",
            SectionBodyType::Checklist => "checklist",
        }
    }
}
```

Use `as_str()` anywhere the UI or debug helpers still need the old lowercase identifiers. Do not switch those call sites to `Debug` formatting because `MultiField`/`FreeText` would silently change displayed output.

### Step 2 - Replace `section_type: String` in `SectionConfig`
Change the runtime field:

```diff
-    #[serde(rename = "type")]
-    pub section_type: String,
+    pub body: SectionBodyType,
```

`SectionConfig` is no longer trying to preserve any old serialized `type` key. The authored schema moves to `body:` in Part 2.

### Step 3 - Update `infer_body_mode`
Change the helper to return `SectionBodyType` instead of `String`:

```diff
-fn infer_body_mode(fields: &[HeaderFieldConfig], lists: &[HierarchyList]) -> String {
+fn infer_body_mode(fields: &[HeaderFieldConfig], lists: &[HierarchyList]) -> SectionBodyType {
     if !fields.is_empty() {
-        "multi_field".to_string()
+        SectionBodyType::MultiField
     } else if lists.is_empty() {
-        "free_text".to_string()
+        SectionBodyType::FreeText
     } else {
-        "list_select".to_string()
+        SectionBodyType::ListSelect
     }
 }
```

### Step 4 - Update runtime config construction in `src/data.rs`
Change both constructors:

```diff
-        section_type: infer_body_mode(&field_configs, &attached_lists),
+        body: infer_body_mode(&field_configs, &attached_lists),
```

```diff
-        section_type: "collection".to_string(),
+        body: SectionBodyType::Collection,
```

### Step 5 - Update `maybe_record_section_lists`
Replace string dispatch with enum dispatch:

```diff
-    match section.section_type.as_str() {
-        "list_select" => {
+    match section.body {
+        SectionBodyType::ListSelect => {
             list_data.insert(section.id.clone(), list_entries_from_lists(&section.lists));
         }
-        "checklist" => {
+        SectionBodyType::Checklist => {
             checklist_data.insert(
                 section.id.clone(),
                 checklist_items_from_lists(&section.lists),
             );
         }
         _ => {}
     }
```

### Step 6 - Update `init_states` in `src/app.rs`
Make the match exhaustive on `cfg.body` and remove the old catch-all branch:

```diff
-        .map(|cfg| match cfg.section_type.as_str() {
-            "multi_field" => {
+        .map(|cfg| match cfg.body {
+            SectionBodyType::MultiField => {
                 ...
             }
-            "free_text" => SectionState::FreeText(FreeTextState::new()),
-            "list_select" => {
+            SectionBodyType::FreeText => SectionState::FreeText(FreeTextState::new()),
+            SectionBodyType::ListSelect => {
                 ...
             }
-            "collection" => {
+            SectionBodyType::Collection => {
                 ...
             }
-            "checklist" => {
+            SectionBodyType::Checklist => {
                 ...
             }
-            _ => SectionState::Pending,
         })
```

Import `SectionBodyType` explicitly into `src/app.rs`.

### Step 7 - Update `src/note.rs`
There are three distinct sites:

1. `render_section_body` should match on `(cfg.body, state)`, not on string slices:

```diff
-    match (cfg.section_type.as_str(), state) {
-        ("multi_field", SectionState::Header(header)) => {
+    match (cfg.body, state) {
+        (SectionBodyType::MultiField, SectionState::Header(header)) => {
             render_multifield(cfg, header, assigned_values, sticky_values)
         }
-        ("free_text", SectionState::FreeText(text)) => text.entries.join("\n"),
-        ("list_select", SectionState::ListSelect(list)) => {
+        (SectionBodyType::FreeText, SectionState::FreeText(text)) => text.entries.join("\n"),
+        (SectionBodyType::ListSelect, SectionState::ListSelect(list)) => {
             ...
         }
-        ("checklist", SectionState::Checklist(checklist)) => ...
-        ("collection", SectionState::Collection(collection)) => ...
+        (SectionBodyType::Checklist, SectionState::Checklist(checklist)) => ...
+        (SectionBodyType::Collection, SectionState::Collection(collection)) => ...
         _ => ...
     }
```

2. The test-only `states_for_real_data` helper must match on `section.body`.
3. The direct comparison near the multi-field note helper becomes:

```diff
-        if section.section_type != "multi_field" {
+        if section.body != SectionBodyType::MultiField {
```

### Step 8 - Update `src/document.rs`
The test helper there duplicates state initialization logic. Convert its `match section.section_type.as_str()` block to `match section.body`, import `SectionBodyType`, and remove the obsolete wildcard fallback.

### Step 9 - Keep the inspector/debug UI text stable
In `src/ui/mod.rs`, replace:

```diff
-    parts.push(format!("Type: {}", sec.section_type));
+    parts.push(format!("Type: {}", sec.body.as_str()));
```

That preserves the existing lower-case snake_case display instead of switching to Rust enum debug names.

### Step 10 - Update all remaining inline `SectionConfig` fixtures
After the runtime code changes, fix every remaining `SectionConfig { ... section_type: "...".to_string(), ... }` literal. Use:
- `SectionBodyType::MultiField`
- `SectionBodyType::FreeText`
- `SectionBodyType::ListSelect`
- `SectionBodyType::Collection`
- `SectionBodyType::Checklist`

Use:

```powershell
rg -n "section_type" src
```

The search should return zero hits when the migration is complete.

## Verification

### Automated
```powershell
cargo test
cargo run -- --validate-data
```

### Smoke
```powershell
$env:SCRIBBLENOT_HEADLESS="1"; cargo run
```

The app should still load the real data set without any runtime dispatch regressions.
