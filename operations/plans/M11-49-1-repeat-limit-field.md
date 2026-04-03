**Task**: #49 — Add repeat_limit field to HeaderFieldConfig

**Context**: Three pre-written tests in `src/data.rs` (lines 1681-1722, module `header_field_repeat_limit_tests`) assert that `HeaderFieldConfig` has a `repeat_limit: Option<usize>` field that deserializes from YAML. The struct currently has no such field, so the tests fail to compile. This plan adds the field structurally with no logic changes.

**Approach**: Add `repeat_limit: Option<usize>` with `#[serde(default)]` to `HeaderFieldConfig`. All sites that manually construct `HeaderFieldConfig` via struct literals must also be updated to set `repeat_limit: None` so the struct literals remain exhaustive. There are 13 such sites across `src/data.rs`, `src/app.rs`, `src/note.rs`, and `src/sections/multi_field.rs`.

**Critical Files**:
- `src/data.rs` lines 82-89 — `HeaderFieldConfig` struct definition
- `src/data.rs` lines 713-719 — manual struct construction inside `load_from_flat_file`
- `src/app.rs` lines 1419, 1420, 1453 — struct literals in tests
- `src/note.rs` lines 480, 501, 508, 534, 541, 548, 555 — struct literals in test helpers/functions
- `src/sections/multi_field.rs` lines 132, 142 — struct literals in test helpers

**Reuse**: `#[serde(default)]` pattern already used on other fields in this file (e.g. `options: Vec<String>` uses `#[serde(default)]` explicitly). Note: serde treats `Option` fields as `None` when absent without `#[serde(default)]`, but using the attribute explicitly matches the style of this struct. Use the same attribute.

**Steps**:

1. Add `repeat_limit` field to `HeaderFieldConfig` struct in `src/data.rs`:

```diff
 pub struct HeaderFieldConfig {
     pub id: String,
     pub name: String,
     #[serde(default)]
     pub options: Vec<String>,
     pub composite: Option<CompositeConfig>,
     pub default: Option<String>,
+    #[serde(default)]
+    pub repeat_limit: Option<usize>,
 }
```

2. Update the manual `HeaderFieldConfig` construction at line 713 to initialize the new field:

```diff
                             hfields.push(HeaderFieldConfig {
                                 id: field_id.clone(),
                                 name: field_name.clone().unwrap_or_default(),
                                 options: options.clone(),
                                 composite: composite.clone(),
                                 default: default.clone(),
+                                repeat_limit: None,
                             });
```

3. Add `repeat_limit: None` to all remaining struct literal construction sites (Rust struct literals are exhaustive; every site must include the new field):

   - `src/app.rs` line 1419: add `repeat_limit: None` before the closing `}`
   - `src/app.rs` line 1420: add `repeat_limit: None` before the closing `}`
   - `src/app.rs` line 1453: add `repeat_limit: None` before the closing `}`
   - `src/note.rs` line 480 (`.map` closure): add `repeat_limit: None` before the closing `}`
   - `src/note.rs` line 501 (`dur_cfg`): add `repeat_limit: None` before the closing `}`
   - `src/note.rs` line 508 (`date_cfg`): add `repeat_limit: None` before the closing `}`
   - `src/note.rs` line 534 (`date_cfg`): add `repeat_limit: None` before the closing `}`
   - `src/note.rs` line 541 (`time_cfg`): add `repeat_limit: None` before the closing `}`
   - `src/note.rs` line 548 (`dur_cfg`): add `repeat_limit: None` before the closing `}`
   - `src/note.rs` line 555 (`appt_cfg`): add `repeat_limit: None` before the closing `}`
   - `src/sections/multi_field.rs` line 132 (`simple_field` helper): add `repeat_limit: None` before the closing `}`
   - `src/sections/multi_field.rs` line 142 (`composite_field_date` helper): add `repeat_limit: None` before the closing `}`

**Verification**:

### Manual tests
- None required; this is a purely structural change with no UI surface.

### Automated tests
- Run `cargo test header_field_repeat_limit_tests` — all three tests (`repeat_limit_some_when_present`, `repeat_limit_none_when_absent`, `repeat_limit_some_zero_when_zero`) must pass.
- Run `cargo build` to confirm the rest of the codebase compiles without error after the struct change.

## Prefect-1 Report

### Blocking issues

**#1 — Missing struct literal update sites** (`src/app.rs`, `src/note.rs`, `src/sections/multi_field.rs`)
The plan identified only one `HeaderFieldConfig { ... }` construction site (`src/data.rs:713`) but there are 12 additional struct literal sites across three other files. Rust struct literals are exhaustive — all sites must include every field or the build fails. Without this fix, Step 2 alone would leave the codebase failing `cargo build`.

Fixed by:
- Updating the Approach paragraph to state there are 13 construction sites across four files.
- Expanding Critical Files to list the additional files and line ranges.
- Adding Step 3 with a bullet for each of the 12 remaining sites.

## Changelog

### Review – 2026-04-02
- #1 (nit): Clarified Reuse note — replaced imprecise "uses derive default implicitly" with accurate description of serde's `Option` handling and why explicit `#[serde(default)]` is still preferred for style consistency.

### Prefect-1 – 2026-04-03
- #1 (blocking): Added Step 3 and expanded Critical Files / Approach to cover the 12 additional `HeaderFieldConfig` struct literal sites in `src/app.rs`, `src/note.rs`, and `src/sections/multi_field.rs` that the plan omitted.

## Progress
- Step 1: Added `repeat_limit: Option<usize>` with `#[serde(default)]` to `HeaderFieldConfig` struct in `src/data.rs`
- Step 2: Updated manual `HeaderFieldConfig` construction in `src/data.rs` `load_from_flat_file` to include `repeat_limit: None`
- Step 3: Added `repeat_limit: None` to all 12 remaining struct literal sites across `src/app.rs`, `src/note.rs`, and `src/sections/multi_field.rs`

## Implementation
Complete – 2026-04-03
