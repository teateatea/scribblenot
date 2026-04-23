## Task
#24 - Thread source spans and raw node snapshots through parse/merge to enable location-aware, fingerprint-routed validation errors

## Implement before #44
This plan should land before `#44` (the error modal). `#44` depends on an `ErrorReport` type carrying source location and specific `ErrorKind` variants. Both come from this work. Once this plan is complete, `#44`'s two-pass parse approach is no longer needed and can be removed from that plan.

## 2026-04-23 Scope Note
This task was split out of branch A (`inference-contract-hardening`) into a dedicated exploratory worktree so the source-span strategy can be evaluated without blocking the narrower schema-contract tasks.

The recommendation for that exploratory worktree is to prefer a parser-level span pass first ("option 1"), not the simpler branch-A-friendly source-index heuristic. The reason is not that the heuristic is bad; it is that `#24` is the foundation for `#44`, and the stronger parser-level path is the better long-term base if it proves maintainable.

That said, this area is fragile. The current loader in `src/data.rs` is built around straightforward typed deserialization, and the repo is on `serde_yaml 0.9`, which is deprecated. So the exploratory worktree should treat parser-event code as a deliberate architectural experiment: stronger foundation if it works cleanly, but higher implementation risk.

## Recommended Direction For The Separate Worktree

### Option 1 - Parser-level span pass
Use a low-level YAML event/token walk ahead of typed deserialization to capture exact source marks for top-level authored blocks, and ideally for the keys that define their kind. This is the preferred direction for the dedicated `#24` worktree.

Why prefer it there:
- it is the strongest foundation for future diagnostics
- it can support richer source reporting than just top-level line numbers
- it avoids project-specific heuristics becoming the permanent provenance model

Why it was not kept on branch A:
- it widens the loader change substantially
- it leans on parser-shaped code in a sensitive area
- it is easier to destabilize than the narrower contract-hardening work on `#45`, `#50`, `#51`, and `#64`

### Important correction to the earlier plan
The earlier Step 2 assumed `serde_yaml::Value` would carry enough public source-location metadata to recover per-node lines after deserialization. That assumption is likely too optimistic in this repo's current `serde_yaml` setup. Parse errors expose locations, but ordinary deserialized `Value`s do not appear to provide a clean public API for "this mapping started on line N".

So if the goal is a true long-term provenance layer, the exploratory worktree should plan on capturing positions during parsing, not after plain `Value` deserialization.

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

### Step 2 - Parser-level span pass in `parse_hierarchy_file_documents`
Change the function to accept the file path and perform a parser-level pre-pass that captures source marks before normal typed deserialization. The implementation details can vary, but the key requirement is: collect source locations during parsing rather than assuming they are recoverable later from plain `serde_yaml::Value`.

Conceptually:

```rust
// pseudocode
for doc in yaml_documents(content) {
    let parsed_doc = parser_span_pass(doc)?;
    // walk events/nodes while marks are still available
    // find top-level block arrays such as lists / sections / fields
    // for each mapping entry, capture:
    // - the mapping start mark or `id:` key mark
    // - the raw sub-tree needed for fingerprint inspection
    // store SourceNode { file, line, raw } into a local index
    //
    // then run the existing typed deserialization path for the same document
}
```

The line of an entry's mapping start or `id:` key is still the best anchor because it points the author to the actual block definition. If the parser-layer implementation makes quoted-line capture cheap, record that during this same pass as well.

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
