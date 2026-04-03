# Plan: YAML Data Hierarchy

## Task
#70 - Propose a plan to implement the canonical 6-level YAML data hierarchy from the discuss-idea session

## Context
The discuss-idea session produced `DISCUSSION-yaml-data-hierarchy.md`, defining a strict 6-level hierarchy: Template > Group > Section > Field > List > Item. The codebase currently has a working flat-block loader (`load_data_dir` in `src/data.rs`) that scans the data directory, merges blocks from all YAML files, validates uniqueness, detects cycles, and reconstructs the old runtime structs (`SectionGroup`, `SectionConfig`, `HeaderFieldConfig`). That reconstruction pass is not a pure data format swap - it is a runtime-model migration. Four blocking questions must be answered before implementation starts:

1. **Compatibility strategy** - do old structs (`SectionConfig`, `SectionGroup`, `HeaderFieldConfig`) stay as the runtime model, or get replaced by new hierarchy structs?
2. **date_prefix handling** - it is deferred in the spec but actively used in `note.rs:315` for `list_select` sections; dropping it breaks the "behavior identical" guarantee.
3. **nav_label consumer migration** - `map_label` appears in `SectionConfig`, `FlatBlock::Section`, the reconstruction pass, `ui.rs:208`, `note.rs:601`, and two test fixtures; rename scope must be explicit.
4. **ID uniqueness and resolution semantics** - the current duplicate check keys on `(type_tag, id)`; the id_map lookup keys on bare `id` only; these two rules are in conflict for cross-type resolution and must be reconciled.

This plan answers all four questions and provides a concrete, phased migration strategy.

## Key Decisions (answer the four questions)

**1. Compatibility strategy: thin shim, keep old runtime structs.**
The new hierarchy Rust structs (`HierarchyTemplate`, `HierarchyGroup`, `HierarchySection`, `HierarchyField`, `HierarchyList`, `HierarchyItem`) are introduced as YAML-layer types only. After loading and validation, a conversion function maps the new hierarchy into the existing `SectionGroup` / `SectionConfig` / `HeaderFieldConfig` runtime model. This keeps `app.rs`, `note.rs`, `ui.rs`, and all section state modules untouched in this mission. The old runtime structs are not renamed or deleted here.

**2. date_prefix handling: preserved on SectionConfig via loader carry-through.**
`date_prefix` is explicitly deferred in the spec but must not be dropped. The new `HierarchySection` struct carries an optional `date_prefix: Option<bool>` field so existing YAML authors can still set it. The shim conversion copies it into `SectionConfig.date_prefix` exactly as today. No behavioral change; the field is simply not part of the canonical spec and must be documented as a legacy carry-through.

**3. nav_label: rename in YAML files and the loader only; Rust runtime uses a compatibility alias.**
The YAML rename (`map_label` -> `nav_label`) happens in all data files. `FlatBlock::Section` gains `nav_label: Option<String>` alongside `map_label: Option<String>` (both accepted during load); the reconstruction pass reads `nav_label` first, falls back to `map_label` if absent, and writes the result into `SectionConfig.map_label`. The runtime field on `SectionConfig` keeps its name `map_label` for this mission; `ui.rs:208` and `note.rs:601` continue to compile without change. A follow-up task can complete the coordinated consumer rename once the YAML layer is settled.

**4. ID uniqueness and resolution: typed uniqueness, typed lookup.**
Uniqueness is scoped per `(type_tag, id)` - this is the current behavior and is correct. The `id_map` lookup, currently keyed on bare `id`, must be changed to key on `(type_tag, id)` as well. Cross-type resolution (a Section child referencing a Field by bare id) resolves by trying the expected type first; if no match, it is a load error. This eliminates the ambiguity where the same bare id could match a Section and a Field silently.

**5. data_file: kept as-is; no section-level lists migration.**
The data files (`objective_findings.yml`, `remedial.yml`, `tx_mods.yml`, `infection_control.yml`) stay as separate files and are migrated to `HierarchyFile` format, but sections continue to reference them via `data_file:` exactly as today. The runtime `data_file` dispatch loop in `AppData::load` stays intact, reading each file as `HierarchyFile` instead of `FlatFile`. `reload_list` and `append_list_entry` (which key on `section.data_file`) are unaffected structurally. No section-level `lists:` field is added to `HierarchySection` in this mission. This avoids breaking the custom-entry write path at `app.rs:966`.

`tx_regions.yml` is the sole exception: its data is moved inline into sections.yml as `HierarchyList` entries on the `tx_regions` section (see Decision 6 below), because `block_select` already works differently from `data_file`-keyed sections.

**6. tx_regions: inline lists on the section, no separate file section definition.**
`tx_regions.yml` is NOT migrated to a `HierarchyFile` with a `sections:` entry. That would create a duplicate Section ID since sections.yml already defines the `tx_regions` section. Instead, `tx_regions.yml` contains only `lists:` entries (the body regions as `HierarchyList` objects). The `tx_regions` section definition stays in sections.yml and gains an optional `lists:` field listing those list IDs. `HierarchySection` therefore does carry an optional `lists:` field, but it is only used for `block_select` sections - not for `list_select` or `checklist` sections, which continue to use `data_file:`.

**7. Items are not referenceable across files.**
Items only exist inline within Lists. `HierarchyFile` does NOT have a top-level `items:` field. `TypeTag::Item` is not added to the registry. This simplifies the loader and matches actual usage.

## Approach
Introduce the new hierarchy as a pure YAML-parsing layer sitting in front of the existing reconstruction pass. The existing `FlatFile` / `FlatBlock` plumbing is **replaced** by a new `HierarchyFile` that natively models the 6-level structure. A conversion function bridges from the new hierarchy structs into the existing runtime model. Existing data files are migrated to the new schema. Validation (missing IDs, duplicate ID+type, circular refs) is re-implemented against the new typed lookup. The public `AppData::load` entry point signature stays the same; callers of `AppData::load` see no change. The internal `load_data_dir` helper is replaced by `load_hierarchy_dir`.

The migration is split into four sub-tasks in the order a developer must work through them:
- ST1: Define Rust structs for all six hierarchy levels
- ST2: Build the directory-scanning loader with typed ID resolution and validation
- ST3: Migrate all existing data YAML files to the new schema
- ST4: Wire loader into `AppData::load`, verify end-to-end, remove dead code

## Critical Files

- `src/data.rs` - runtime structs (`SectionConfig`, `SectionGroup`, `HeaderFieldConfig`); `AppData::load`; `load_data_dir`; reconstruction pass (lines 694-760); duplicate/cycle checks (lines 629-691). New hierarchy structs and loader go here.
- `src/flat_file.rs` - `FlatFile`, `FlatBlock` enum (lines 1-47); all tests (lines 49-280). This module is replaced by the new hierarchy types; tests migrate to `src/data.rs`.
- `data/sections.yml` - defines all groups, sections, and fields; uses `map_label`, old block structure, `type:` tags, `data_file:` references. Must be migrated to new schema.
- `data/tx_regions.yml` - currently hard-skipped by loader (`load_data_dir:617`); uses `entries:` root key and `BlockSelectFile` struct. Migration: convert entries to `HierarchyList` objects in a `lists:`-only file; section definition and `lists:` ref stay in sections.yml.
- `data/boilerplate.yml` - currently uses FlatFile format (`blocks:` root key, `type: boilerplate` discriminant). Must be migrated in ST3 to `boilerplate: [{ id: X, text: Y }]` format before `flat_file.rs` is removed.
- `data/objective_findings.yml`, `data/remedial.yml`, `data/tx_mods.yml`, `data/infection_control.yml` - currently loaded by section_type dispatch in `AppData::load` (lines 228-265). Each migrates from `FlatFile` format to `HierarchyFile` `lists:` + `items:` format; `data_file:` refs on their sections remain unchanged.
- `src/sections/block_select.rs` - `BlockSelectGroup::from_config` takes `&BlockSelectEntry`; `BlockSelectState::new` at line 49 takes `Vec<BlockSelectEntry>`. Both must be updated to accept `&HierarchyList` / `Vec<HierarchyList>` when `block_select_data` is re-typed.
- `src/app.rs` - dispatch loop at lines 130-167 uses `cfg.data_file` to key into `list_data`, `block_select_data`, `checklist_data`; line 151 uses `cfg.data_file` for block_select lookup (must change to `cfg.id` after migration); line 966 uses `section.data_file` for `append_list_entry` (unchanged).
- `src/note.rs` - reads `cfg.date_prefix` (line 315); `map_label` appears in test helper `make_section` at line 608; no production changes required but must pass regression tests.
- `src/ui.rs` - reads `section.map_label` (line 208); no changes required.

## Reuse

- Existing DFS cycle detection logic (`dfs` function, `src/data.rs` lines 664-686) - re-implement with the typed `(type_tag, id)` key map but the same algorithm.
- Existing `AppData::reload_list` and `AppData::append_list_entry` (lines 299-324) - keep unchanged structurally; update only the file-parse call from `ListFile` to `HierarchyFile`.
- Existing `PartOption` enum and its `label()` / `output()` methods - used when converting `HierarchyItem` to `PartOption` for the block_select renderer.
- Existing `CompositeConfig` and `CompositePart` - carried forward into `HierarchyField` for multi-part fields; no changes needed.

## Steps

### ST1: Define Rust structs for all six hierarchy levels

**1.1** In `src/data.rs`, after the existing `CompositeConfig` definition, add the six new hierarchy structs. All derive `Debug, Clone, Serialize, Deserialize`. Note: `HierarchyItem` has no `id` field at the top level of `HierarchyFile` - items only exist inline within lists.

```rust
// New: 6-level hierarchy structs (YAML-layer only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyItem {
    pub output: String,
    pub id: Option<String>,
    pub label: Option<String>,
    pub default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyList {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub items: Vec<HierarchyItemOrRef>,
    pub sticky: Option<bool>,
    pub default: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HierarchyItemOrRef {
    Inline(HierarchyItem),
    Shorthand(String),   // output-only shorthand
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyField {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub lists: Vec<HierarchyListOrRef>,
    pub format: Option<String>,
    pub repeat_limit: Option<usize>,
    // Legacy multi-part composite support
    pub composite: Option<CompositeConfig>,
    pub options: Option<Vec<String>>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HierarchyListOrRef {
    Inline(HierarchyList),
    Ref(String),   // bare id reference
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchySection {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub fields: Vec<HierarchyFieldOrRef>,
    // block_select sections only: inline or ref'd lists (tx_regions uses this)
    #[serde(default)]
    pub lists: Vec<HierarchyListOrRef>,
    pub nav_label: Option<String>,
    pub map_label: Option<String>,   // legacy alias, accepted during load
    pub header: Option<String>,
    pub footer: Option<String>,
    pub section_type: Option<String>,
    pub data_file: Option<String>,   // kept for list_select/checklist sections
    pub date_prefix: Option<bool>,   // legacy carry-through; not in canonical spec
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HierarchyFieldOrRef {
    Inline(HierarchyField),
    Ref(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyGroup {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub sections: Vec<HierarchySectionOrRef>,
    pub nav_label: Option<String>,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub num: Option<usize>,   // legacy carry-through: maps to SectionGroup.num; not in canonical spec
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HierarchySectionOrRef {
    Inline(HierarchySection),
    Ref(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyTemplate {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub groups: Vec<HierarchyGroupOrRef>,
    pub nav_label: Option<String>,
    pub header: Option<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HierarchyGroupOrRef {
    Inline(HierarchyGroup),
    Ref(String),
}

/// Top-level file container: one file may define any mix of hierarchy nodes.
/// Items are NOT top-level: they only exist inline within Lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyFile {
    #[serde(default)]
    pub templates: Vec<HierarchyTemplate>,
    #[serde(default)]
    pub groups: Vec<HierarchyGroup>,
    #[serde(default)]
    pub sections: Vec<HierarchySection>,
    #[serde(default)]
    pub fields: Vec<HierarchyField>,
    #[serde(default)]
    pub lists: Vec<HierarchyList>,
    #[serde(default)]
    pub boilerplate: Vec<BoilerplateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoilerplateEntry {
    pub id: String,
    pub text: String,
}
```

**1.2** Add unit tests that construct each struct and verify mandatory fields are present. At minimum: one test per struct verifying `id` / `label` / mandatory child vec fields exist and a round-trip serde_yaml test for `HierarchyFile`.

### ST2: Build the directory-scanning loader with typed ID resolution and validation

**2.1** Add a `load_hierarchy_dir(path: &Path) -> Result<HierarchyFile, String>` function in `src/data.rs`. It must:
- Read all `*.yml` files in the directory, skipping `keybindings.yml` and `config.yml`. Do NOT hard-skip `tx_regions.yml` - the old workaround is removed here since `tx_regions.yml` is migrated in ST3.
- Parse each file as `HierarchyFile` via `serde_yaml::from_str`.
- Merge all top-level vecs into a single combined `HierarchyFile`.

**2.2** Build a typed ID registry: `HashMap<(TypeTag, String), NodeRef>` where `TypeTag` is an enum `{ Template, Group, Section, Field, List }`. Note: `TypeTag::Item` is NOT added - items are not registered because they are not referenceable across files. Insert every node from the merged file. If any `(TypeTag, id)` pair appears more than once, return `Err("duplicate: type={} id={}")`. Additionally, validate boilerplate IDs for uniqueness: iterate `hf.boilerplate` and check for duplicate `id` values before building the `HashMap<String, String>`. The current codebase already treats duplicate boilerplate IDs as an error (see `src/data.rs:1662`); this behavior must be preserved.

**2.3** Validate template cardinality: after merging all files, check `hf.templates.len()`. If 0, return `Err("no template defined")`. If >1, return `Err("multiple templates defined: expected exactly 1")`. The single template is the "active template" used by `hierarchy_to_runtime` in ST4.1.

**2.4** For cross-file `Ref(id)` resolution: when resolving a `HierarchyGroupOrRef::Ref(id)` inside a Template, look up `(TypeTag::Group, id)`. If not found, return `Err("unresolved group ref: {id}")`. Apply the same typed lookup at every level (Section refs look up `TypeTag::Section`, Field refs look up `TypeTag::Field`, List refs look up `TypeTag::List`). A bare-id ref is never ambiguous because each ref site has a known expected type.

**2.5** Re-implement cycle detection using the typed key map. The DFS algorithm is identical to the existing one in `src/data.rs` lines 664-686; only the key type changes.

**2.6** Add unit tests for: duplicate ID detection (including duplicate boilerplate IDs), missing ref detection, circular ref detection, template cardinality (0 and >1), and successful multi-file merge.

### ST3: Migrate existing data YAML files

**3.1** Migrate `data/sections.yml` from the flat block format to the new hierarchy schema:
- Replace `type: group` blocks with `groups:` top-level list entries using `id:`, `label:` (was `name:`), `sections:` (was `children:`).
- Replace `type: section` blocks with `sections:` entries using `id:`, `label:` (was `name:`), `nav_label:` (was `map_label:`), `fields:` (was `children:`).
- Replace `type: field` blocks with `fields:` entries using `id:`, `label:` (was `name:`).
- Replace `type: options-list` blocks with `lists:` entries using `id:`, `label:`, `items:`.
- Keep `section_type:`, `data_file:`, `date_prefix:` on sections that still use them; they are legacy carry-through fields.
- For the `tx_regions` section (section_type: "block_select"), add a `lists:` field with ref IDs pointing to the lists defined in `tx_regions.yml`. Do NOT add a `data_file:` field to this section.
- Keep one `templates:` entry that wires all groups in order, making the active template explicit.

**3.2** Migrate `data/tx_regions.yml` from the old `entries:` root format (`BlockSelectFile`) to a `lists:`-only `HierarchyFile`. The file must NOT define a `sections:` entry (that would create a duplicate ID for `tx_regions` which is already defined in sections.yml). Each `BlockSelectEntry` (fields: `id`, `label`, `header`, `entries: Vec<PartOption>`) maps as follows:
- The file contains only a `lists:` top-level key.
- Each `BlockSelectEntry` becomes one `HierarchyList`: `id:` from entry's `id`, `label:` from entry's `label`, `preview:` from entry's `header` string (verify against `src/sections/block_select.rs` that `preview` is the correct field for the region header).
- Each technique in `BlockSelectEntry.entries: Vec<PartOption>` becomes a `HierarchyItem` with `id:` (from `PartOption::Full.id` if present), `label:` (from `label()`), `output:` (from `output()`), and `default:` (from `default_selected()` if false, omit if true to let the renderer use its own default).
- The `tx_regions` section in sections.yml (step 3.1) gains a `lists:` field referencing these list IDs by bare string ref.

**3.3** Migrate `data/boilerplate.yml` from FlatFile format to `HierarchyFile` format. The current file has a `blocks:` root key with `type: boilerplate` discriminant on each entry. Convert to:

```yaml
boilerplate:
  - id: treatment_plan_disclaimer
    text: |
      Regions and locations are bilateral unless indicated otherwise.
      Patient is pillowed under ankles when prone, and under knees when supine.
  - id: informed_consent
    text: Patient has been informed of the risks and benefits of massage therapy, and has given informed consent to assessment and treatment.
```

This must be done before ST4.4 removes `flat_file.rs`, otherwise `load_hierarchy_dir` will fail to parse it.

**3.4** Migrate `data/objective_findings.yml`, `data/remedial.yml`, `data/tx_mods.yml`, `data/infection_control.yml` from the current `entries:` / OptionsList flat format to `lists:` + `items:` format. The `data_file:` references on their parent sections in sections.yml stay unchanged - these files continue to be loaded by the `data_file` dispatch loop, just parsed as `HierarchyFile` instead of `FlatFile`.

**3.5** Verify all migrated files parse without error using `serde_yaml::from_str::<HierarchyFile>()`.

### ST4: Wire loader into AppData::load and remove dead code

**4.1** Implement `hierarchy_to_runtime(hf: &HierarchyFile) -> (Vec<SectionGroup>, Vec<SectionConfig>, HashMap<String, String>, HashMap<String, Vec<HierarchyList>>)` before touching `AppData::load`. This shim must exist and compile before ST4.2 replaces the `load_data_dir` call, because both steps are part of a single atomic change to `AppData::load` that must not leave the crate in a broken intermediate state. The shim:
- Walks the active template's groups in order.
- For each group, resolves section refs, builds `SectionConfig` entries.
- Maps `HierarchyGroup.label` into `SectionGroup.name` and `HierarchySection.label` into `SectionConfig.name`.
- Copies `nav_label` (falling back to `map_label`, then to empty string if both absent, matching the existing `unwrap_or_default()` behavior) into `SectionConfig.map_label`.
- Copies `date_prefix` into `SectionConfig.date_prefix`.
- Copies `section_type` into `SectionConfig.section_type` using `.unwrap_or_default()` (matching the reconstruction pass at `data.rs:733`; `HierarchySection.section_type` is `Option<String>` while `SectionConfig.section_type` is a plain `String`). Copies `data_file` unchanged.
- Sets `SectionConfig.options` to `vec![]` and `SectionConfig.composite` to `None` (matching the reconstruction pass at `data.rs:736-737`; neither field is carried by `HierarchySection`).
- For each group, copies `HierarchyGroup.num` into `SectionGroup.num` (legacy carry-through field; groups that omit `num` default to `None`).
- For `multi_field` sections, resolves field refs and builds `Vec<HeaderFieldConfig>` from `HierarchyField`: maps `id`, `label -> name`, `options`, `composite`, `default`, and `repeat_limit` (all present on `HierarchyField`). Sets `repeat_limit` from `HierarchyField.repeat_limit` (matches the `HeaderFieldConfig` field of the same name; the old reconstruction pass hardcoded `None` here, so this is a forward-compatible improvement).
- Builds a `HashMap<String, String>` of boilerplate id -> text from `hf.boilerplate` and returns it as the third element.
- For `block_select` sections, resolves the section's `lists:` field (inline or ref) into `Vec<HierarchyList>` and accumulates them in a `HashMap<String, Vec<HierarchyList>>` keyed by section id (e.g. `"tx_regions"`). Returns this as the fourth element.

**4.2** Replace the `load_data_dir` call inside `AppData::load` with `load_hierarchy_dir`, then immediately call `hierarchy_to_runtime` on the result. `AppData::load` currently destructures `base.groups`, `base.sections`, and `base.boilerplate_texts` from the old `AppData` return value; after this change those values come from the tuple returned by `hierarchy_to_runtime`. The `block_select_data` value also comes from the tuple (fourth element). The existing `list_data`, `checklist_data`, and `block_select_data` declarations at lines 226-228 (`let mut list_data`, `let mut checklist_data`, `let mut block_select_data`) become: `list_data` and `checklist_data` are still populated by the dispatch loop below; `block_select_data` is initialized from the shim tuple instead and declared without `mut`.

**4.3** Update the existing per-section `data_file` dispatch loop (lines 230-271):
- For `list_select`: parse the data file as `HierarchyFile` instead of `FlatFile`. Iterate `hf.lists`, flatten their `items` into `Vec<ListEntry>` (converting `HierarchyItemOrRef::Inline(item)` to `ListEntry { label: item.label.unwrap_or(item.output.clone()), output: item.output }`, and `HierarchyItemOrRef::Shorthand(s)` to `ListEntry { label: s.clone(), output: s }`).
- For `checklist`: parse as `HierarchyFile`, iterate `hf.lists`, collect item labels into `Vec<String>` using the same shorthand/inline conversion as above.
- For `block_select`: **remove this branch entirely**. `block_select_data` is now populated from `hierarchy_to_runtime` (ST4.1), not from a `data_file` dispatch. The `tx_regions` section has no `data_file:` field after ST3, so this branch would never fire anyway.
- Update `app.rs:151` (the `block_select` state construction): it currently does `cfg.data_file.as_ref().and_then(|f| data.block_select_data.get(f))`. After re-typing, change to `data.block_select_data.get(&cfg.id)` since `block_select_data` is now keyed by section id.

**4.4** Update `src/sections/block_select.rs` in the same commit as ST4.3 (these changes must land together to avoid compile errors):
- Re-type `AppData.block_select_data` from `HashMap<String, Vec<BlockSelectEntry>>` to `HashMap<String, Vec<HierarchyList>>`.
- Update `BlockSelectGroup::from_config` to accept `&HierarchyList` instead of `&BlockSelectEntry`: map `list.label` to `BlockSelectGroup.label`, `list.preview.unwrap_or_default()` to `BlockSelectGroup.header`, and convert `list.items` (each `HierarchyItemOrRef`) to `Vec<PartOption>` for `BlockSelectGroup.entries`.
- Update `BlockSelectState::new` at line 49 to accept `Vec<HierarchyList>` instead of `Vec<BlockSelectEntry>`. The body call expression `regions.iter().map(BlockSelectGroup::from_config).collect()` is textually unchanged; only the parameter type and the resolved `from_config` signature change. The call site at `app.rs:154` changes accordingly.
- Do not remove `BlockSelectEntry` or `BlockSelectFile` until all consumers are updated in this same commit.

**4.5** Update `AppData::reload_list` and `AppData::append_list_entry` (lines 299-324). Currently both methods use `ListFile { entries: Vec<ListEntry> }` for parse/write. After ST3.4 migrates the underlying files to `HierarchyFile` list/item schema, these methods will fail to parse. Replace the `ListFile` parse with `HierarchyFile` parse and a corresponding serialization path (write). The `list_data` cache key and `ListEntry` type remain unchanged; the conversion from `HierarchyItem` to `ListEntry` matches the ST4.3 `list_select` dispatch logic.

For `append_list_entry` specifically: after reading the file as `HierarchyFile`, locate the first entry in `hf.lists` (each `list_select` data file contains exactly one list; if `hf.lists` is empty, treat it as a new list with a generated id matching the data_file stem). Push the new entry as `HierarchyItemOrRef::Inline(HierarchyItem { output: entry.output.clone(), id: None, label: Some(entry.label.clone()), default: None })`, then re-serialize the entire `HierarchyFile` back to disk. Do not attempt to write a `ListFile` format after this step - the file must stay in `HierarchyFile` format so `reload_list` can re-parse it.

**4.6** Remove `load_data_dir` and replace with `load_hierarchy_dir`. The ~20 existing `load_data_dir` unit tests in `src/data.rs` (lines ~1290-1669) all write temporary files in the old `FlatFile` `blocks:` format. These tests must be rewritten to use `load_hierarchy_dir` with `HierarchyFile`-format YAML. For each test: replace the flat-format YAML fixture with the equivalent hierarchy-format YAML, change the `load_data_dir` call to `load_hierarchy_dir`, and update assertions to match the new return type. Tests that verify duplicate-ID detection, cycle detection, and multi-file merge should map directly - only the YAML format and function name change; the validation semantics are preserved.

Remove `src/flat_file.rs` and its module declaration from `src/main.rs` (confirmed location: `src/main.rs:9`, `mod flat_file;`). Migrate any surviving tests from `flat_file.rs` that cover behavior still needed into `src/data.rs`. Additionally, remove or migrate the `lower_back_prone_fascial_l4l5_starts_unselected` test in `src/data.rs` (lines 500-523): that test directly parses `tx_regions.yml` as `BlockSelectFile`, which will no longer compile after ST3.2 migrates the file and `BlockSelectFile` is removed. Replace it with an equivalent test that parses `tx_regions.yml` as `HierarchyFile` and asserts that the `fascial_l4l5` item in the `back_lower_prone` list has `default: Some(false)`.

**4.7** Run `cargo test` - all existing tests must pass. Run the app manually and confirm note output is byte-for-byte identical to the pre-migration baseline for a complete session.

## Out of Scope (explicit exclusions)
- Boxes (UI layer: map, wizard) - no changes
- Multi-template switching - only one Template is active; switching is deferred
- `date_prefix` migration to a proper spec field - deferred; preserved as legacy carry-through
- `repeat_limit` at Section or Group level - deferred
- Skip-level referencing (Group directly referencing a Field) - not supported
- Changes to copy/paste behavior
- Coordinated rename of `SectionConfig.map_label` -> `SectionConfig.nav_label` in runtime Rust structs and consumers (`ui.rs`, `note.rs`) - deferred to a follow-up task
- `free_text` as a special type - it remains a `section_type` value, not a structural concept
- Section-level `lists:` for `list_select` or `checklist` sections - deferred; `data_file:` stays for those section types
- Top-level `items:` in `HierarchyFile` - items are not referenceable across files in this mission

## Verification

### Manual tests
- Launch the app after migration; navigate all five groups (Intake, Subjective, Treatment, Objective, Post-TX) and confirm sections load without panic.
- Open the Header section and complete all four fields; confirm the note preview shows the correct date/time/duration/type.
- Select items in tx_mods; confirm output appears in the note preview.
- Toggle treatment regions in tx_regions; confirm the TREATMENT / PLAN block renders identically to pre-migration.
- Select entries in objective_findings and remedial_section; confirm the date prefix appears on each line (date_prefix behavior preserved).
- Toggle items in infection_control; confirm checkboxes render correctly.
- Add a custom entry via the list_select custom-entry flow; confirm it persists on restart (tests `append_list_entry` write path).
- Confirm no `map_label:` keys survive in YAML files (grep `data/` for `map_label:`).

### Automated tests
- `cargo test` - all existing tests in `src/data.rs` (including tests migrated from `src/flat_file.rs` in ST4.6) and `src/sections/` must pass.
- New unit tests in `src/data.rs` (added in ST1 and ST2): struct construction, serde round-trip for `HierarchyFile`, duplicate ID error, missing ref error, circular ref error, multi-file merge.
- Integration test: call `load_hierarchy_dir` against the actual `data/` directory and assert `Ok`; assert the returned groups list matches the expected order (intake, subjective, treatment, objective, post_tx).
- Regression test for date_prefix: construct a `SectionConfig` from a migrated `objective_section` entry and assert `date_prefix == Some(true)`.
- Regression test for tx_regions migration: parse `tx_regions.yml` as `HierarchyFile`, assert the `back_lower_prone` list exists, assert the `fascial_l4l5` item has `default: Some(false)`.
- Integration test for block_select dispatch (Decision 6): call `hierarchy_to_runtime` on the loaded `HierarchyFile` from the actual `data/` directory and assert that `block_select_data` contains a key `"tx_regions"` (keyed by section id, not `data_file`) with at least one `HierarchyList` entry.

## Changelog

### Review - 2026-04-02
- #1 (minor): Fixed stale `note.rs:306` line reference in Context section to `note.rs:315` (matching the actual `date_prefix` usage; Critical Files section was already correct).
- #2 (nit): Clarified ST4.4 `BlockSelectState::new` body description - replaced identical before/after strings with an accurate note that the call expression is textually unchanged and only the type signature changes.
- #3 (nit): Corrected `note.rs:601` reference for `map_label` in test helper to `line 608` (the actual line of the `map_label` field assignment in `make_section`).

### Review – 2026-04-02
- #4 (minor): ST4.1 shim description omitted `label -> name` mappings for `SectionGroup.name`, `SectionConfig.name`, and `HeaderFieldConfig.name`; added explicit bullet covering all three.
- #5 (nit): ST4.1 function signature was stated as a 3-tuple then expanded mid-bullet to a 4-tuple; updated the signature in the step heading to the final 4-tuple so a developer implements the correct signature from the start.
- #6 (nit): Verification automated tests lacked a test for Decision 6 (block_select_data keyed by section id after hierarchy_to_runtime); added integration test asserting `block_select_data` contains key `"tx_regions"`.

### Review – 2026-04-02
- #7 (minor): ST4.1 stated "copies `section_type` unchanged" but `HierarchySection.section_type` is `Option<String>` and `SectionConfig.section_type` is `String`; corrected to use `.unwrap_or_default()` matching the reconstruction pass at `data.rs:733`.
- #8 (minor): ST4.1 shim omitted setting `SectionConfig.options = vec![]` and `SectionConfig.composite = None`; added explicit note matching the reconstruction pass at `data.rs:736-737`.
- #9 (minor): ST4.1 shim mentioned only `label -> name` for `HeaderFieldConfig` but omitted `options`, `composite`, `default`, and `repeat_limit` (all present on `HierarchyField`); updated bullet to cover all five fields including `repeat_limit` (which the old reconstruction pass hardcoded to `None`).

### Codex review fixes – 2026-04-02
- #10 (blocking): ST2.2 now explicitly validates boilerplate IDs for uniqueness, preserving the existing error behavior at `data.rs:1662`.
- #11 (blocking): ST4.6 now specifies that `load_data_dir` is removed and all ~20 existing tests are rewritten to use `load_hierarchy_dir` with hierarchy-format YAML.
- #12 (minor): ST2.3 added template cardinality validation (exactly 1 template required after merge; 0 or >1 is a loud error). ST2.6 updated to include template cardinality tests.
- #13 (minor): Approach section corrected: `AppData::load` signature stays the same; `load_data_dir` is replaced by `load_hierarchy_dir`.

## Status
User Approved

