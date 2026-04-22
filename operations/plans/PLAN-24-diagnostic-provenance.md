## Task
#24 - Thread source spans and raw node snapshots through parse/merge to enable location-aware, fingerprint-routed validation errors

## Implement before #44
This plan should land before `#44` (the error modal). `#44` depends on an `ErrorReport` type carrying source location and specific `ErrorKind` variants. Both come from this work. Once this plan is complete, `#44`'s two-pass parse approach is no longer needed and can be removed from that plan.

## Context
Current validation errors explain what is wrong and suggest a fix, but two things are missing:

**1. Source location.** After typed deserialization, all file/line context is gone. An error like "duplicate id 'foo' across list and field" cannot tell you which file or line to look at.

**2. Diagnostic depth on wrong-kind errors.** A wrong-kind error fires when a `contains:` ref points at an id that is registered as the wrong type. But this often masks a simpler authoring mistake: the author didn't misidentify the thing, they accidentally deleted a key that defines what kind it is. For example, a list block missing its `items:` key would still be present in the YAML, but serde would silently deserialize it as a field (or whichever kind it most resembles), and the wrong-kind error would report `expected list, found field` — giving no hint that the block just needs `items:` restored.

If we retain a raw `serde_yaml::Value` snapshot of each parsed node alongside the typed struct, we can inspect it for kind-exclusive fields (fingerprints) when a wrong-kind error fires. This lets us route to a much more specific error: "this block has `modal_start:` and `sticky:` which are list-only fields — it looks like a list that lost its `items:` key."

## Kind Fingerprints

These are fields that appear on exactly one hierarchy kind. Their presence in a raw YAML node is strong evidence of intended type. Verified against the current struct definitions:

| Field | Kind |
|---|---|
| `items` | List only |
| `modal_start` | List only |
| `sticky` | List only |
| `default_enabled` | Collection only |
| `show_field_labels` | Section only |
| `boilerplate_refs` | Group only |
| `note_label` (direct, not nested under `note:`) | Group only |
| `format` | Field only |
| `max_actives` | Field only |

Fields that appear on multiple kinds (`label`, `nav_label`, `preview`, `joiner_style`, `max_entries`, `contains`) are not fingerprints and are ignored by the diagnostic.

## What Gets Stored

For each top-level node parsed from any `.yml` file, alongside the typed struct we store a `SourceNode`:

```rust
pub struct SourceNode {
    pub file: PathBuf,
    pub line: usize,       // 1-indexed line of the node's `id:` key in the authored file
    pub raw: serde_yaml::Value, // the full raw mapping for this node
}
```

A `HashMap<String, SourceNode>` (keyed by node id) is built during `parse_hierarchy_file_documents` and merged alongside the typed structs into a `SourceIndex`. This index is threaded through `validate_merged_hierarchy` and `hierarchy_to_runtime` so every error site can look up source info.

## New Error Kinds

Replace the current bare `String` error returns with a typed `DataError` enum (or extend the existing approach toward `ErrorReport`). The fingerprint check adds three new variants that would otherwise all be reported as generic wrong-kind errors:

- `LooksLikeListMissingItems { id, file, line, quoted_line, found_fingerprints }` — registered as non-list, but has list-exclusive fields
- `LooksLikeCollectionMissingKey { id, file, line, quoted_line, found_fingerprints }` — registered as non-collection, but has collection-exclusive fields
- `LooksLikeSectionOrGroupMissingKey { id, file, line, quoted_line, found_fingerprints }` — registered as field/list, but has section- or group-exclusive fields

The `found_fingerprints` field carries the actual field names found (e.g. `["modal_start", "sticky"]`) so the authored message in `messages/` can include them.

## Critical Files

- `src/data.rs` — add `SourceNode`, `SourceIndex`; update `parse_hierarchy_file_documents` to do a Value pass first; build the index during merge; thread index through `validate_merged_hierarchy` and `hierarchy_to_runtime`; add `fingerprint_kind` helper; update `validate_child_exists` to call fingerprint check and route to specific error kinds
- `src/error_report.rs` (new, or defined inline) — `ErrorReport`, `ErrorSource`, and the `DataError` / `ErrorKind` types shared with `#44`

## Steps

### Step 1 - Define `SourceNode` and `SourceIndex`
Add to `src/data.rs`:

```rust
pub struct SourceNode {
    pub file: PathBuf,
    pub line: usize,
    pub raw: serde_yaml::Value,
}

pub struct SourceIndex {
    pub nodes: HashMap<String, SourceNode>,
}
```

`SourceIndex` is what gets threaded through validation. It is not part of `AppData` and does not persist beyond the load call.

### Step 2 - Two-pass parse in `parse_hierarchy_file_documents`
Change the function to accept the file path and deserialize to `serde_yaml::Value` first. Walk the Value to extract spans:

```rust
// pseudocode
for doc in serde_yaml::Deserializer::from_str(content) {
    let value = serde_yaml::Value::deserialize(doc)?;
    // walk value.get("lists"), value.get("sections"), etc.
    // for each entry, read its "id" key and the Mark (line) on that mapping
    // store SourceNode { file, line, raw: entry.clone() } into a local index
    // then proceed with typed deserialization of value as before
}
```

`serde_yaml::Value` carries location info through its internal representation. The line of an entry's mapping start is the most useful anchor — it points to the `- id: foo` line in the authored YAML.

### Step 3 - Merge `SourceIndex` alongside `HierarchyFile` in `read_hierarchy_dir`
Return `(HierarchyFile, SourceIndex, usize)` from `read_hierarchy_dir` and from `load_hierarchy_dir`. Thread the index into all callers.

### Step 4 - Thread `SourceIndex` into `validate_merged_hierarchy`
Update the signature to accept `&SourceIndex`. Every existing error site can now optionally look up source info:

```rust
fn validate_merged_hierarchy(file: &HierarchyFile, index: &SourceIndex) -> Result<(), DataError>
```

For the immediate wrong-kind and missing-ref errors, include file/line from the index in the returned `DataError` when available.

### Step 5 - Add `fingerprint_kind` helper
```rust
fn fingerprint_kind(raw: &serde_yaml::Value) -> Vec<(&'static str, TypeTag)> {
    // returns list of (field_name, TypeTag) for any fingerprint fields found
    let mut found = Vec::new();
    if raw.get("items").is_some()        { found.push(("items", TypeTag::List)); }
    if raw.get("modal_start").is_some()  { found.push(("modal_start", TypeTag::List)); }
    if raw.get("sticky").is_some()       { found.push(("sticky", TypeTag::List)); }
    if raw.get("default_enabled").is_some() { found.push(("default_enabled", TypeTag::Collection)); }
    if raw.get("show_field_labels").is_some() { found.push(("show_field_labels", TypeTag::Section)); }
    if raw.get("boilerplate_refs").is_some() { found.push(("boilerplate_refs", TypeTag::Group)); }
    if raw.get("note_label").is_some()   { found.push(("note_label", TypeTag::Group)); }
    if raw.get("format").is_some()       { found.push(("format", TypeTag::Field)); }
    if raw.get("max_actives").is_some()  { found.push(("max_actives", TypeTag::Field)); }
    found
}
```

If all fingerprints agree on a single `TypeTag`, that is the inferred intended kind.

### Step 6 - Update `validate_child_exists` to use fingerprint check
When a wrong-kind error is about to fire (the target id exists but as the wrong type), look up the target in `SourceIndex` and run `fingerprint_kind` on its raw snapshot. If fingerprints agree on an intended kind different from the registered kind, return a specific `DataError` variant instead of the generic wrong-kind error.

The specific error message should name the fingerprint fields found and suggest the missing key. Example output for a list missing `items:`:

> 'my_treatments' is referenced as a list, but is registered as a field.
> Its YAML has `modal_start:` and `sticky:` which are list-only fields.
> It looks like a list that is missing its `items:` key.
> Fix: add `items:` to 'my_treatments' or check that the block is under `lists:` in your data file.

### Step 7 - Define `DataError` / `ErrorReport` types
These are the shared types that `#44` will consume. Minimal first version:

```rust
pub struct ErrorReport {
    pub kind_id: &'static str,
    pub params: HashMap<&'static str, String>,
    pub source: Option<ErrorSource>,
}

pub struct ErrorSource {
    pub file: PathBuf,
    pub line: usize,
    pub quoted_line: String,
}
```

Update `load_hierarchy_dir` and `validate_merged_hierarchy` to return `ErrorReport`. `AppData::load` and `reload_data` in `app.rs` should propagate this through.

### Step 8 - Update `hierarchy_to_runtime` errors
The runtime build errors (`"unknown group '{id}'"`, `"group cannot contain {:?}"`, etc.) currently do not have source info. Thread the index in and attach source location to these errors as well.

### Step 9 - Tests
- Unit test: `fingerprint_kind` returns the correct tags for YAML with known fingerprint fields
- Unit test: `validate_child_exists` routes to the specific `LooksLikeListMissingItems` variant when the target has list-only fields
- Unit test: a node with no fingerprint fields still gets the generic wrong-kind error
- Unit test: `ErrorReport.source.line` matches the actual line number in a known test fixture

### Step 10 - Final verification
```powershell
cargo test
cargo run -- --validate-data
```

Manually introduce a list block with `modal_start:` but no `items:`, reference it from a section's `contains:`, and confirm the error output names the fingerprint fields and suggests the fix.
