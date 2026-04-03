---
Status: Draft
---

# M11-52-1 — Add Boilerplate variant to FlatBlock enum

## Task

#52 — Extract hard-coded boilerplate strings from note.rs into editable YML data files

## Context

Task #52 calls for moving hard-coded boilerplate strings out of `note.rs` and into YAML data files. Before the data layer can reference boilerplate content, the `FlatBlock` enum must be able to represent it. Sub-task 52.1 adds the `Boilerplate` variant so the YAML loader can deserialize entries with `type: boilerplate`, `id`, and `text` fields. Four failing tests at lines 218-275 of `flat_file.rs` specify the exact shape required.

## Approach

Add a single new struct-like variant `Boilerplate { id: String, text: String }` to the existing `FlatBlock` enum. The enum already uses `#[serde(tag = "type", rename_all = "kebab-case")]`, so serde will automatically deserialize `type: boilerplate` to this variant. Both `id` and `text` are required (no `#[serde(default)]`), which satisfies the tests that assert deserialization fails when either field is absent.

## Critical Files

- `src/flat_file.rs` lines 8-38 — `FlatBlock` enum definition where the new variant is added
- `src/flat_file.rs` lines 215-275 — TDD tests that the new variant must satisfy

## Reuse

- Existing `#[serde(tag = "type", rename_all = "kebab-case")]` derive on `FlatBlock` handles all tag-based dispatch automatically; no custom `Deserialize` impl is needed.

## Steps

1. Open `src/flat_file.rs` and add the `Boilerplate` variant to `FlatBlock` after `OptionsList`:

```diff
     OptionsList {
         id: String,
         #[serde(default)] children: Vec<String>,
         #[serde(default)] entries: Vec<PartOption>,
     },
+    Boilerplate {
+        id: String,
+        text: String,
+    },
 }
```

No other files require changes for this sub-task.

## Verification

### Manual tests

- None required; the change is purely a data-type addition with no UI surface.

### Automated tests

Run `cargo test` from the project root. The four tests under `// --- Tests for FlatBlock::Boilerplate variant ---` must all pass:

- `boilerplate_variant_deserializes_from_yaml`
- `boilerplate_variant_id_and_text_are_correct`
- `boilerplate_missing_id_fails_deserialization`
- `boilerplate_missing_text_fails_deserialization`

All pre-existing tests in `flat_file.rs` must continue to pass (no regression).

## Changelog

### Review - 2026-04-02
- #1: Fixed "Two failing tests" to "Four failing tests" in Context section (lines 218-275 contain four test functions)

## Progress

- Step 1: Added `Boilerplate { id: String, text: String }` variant to `FlatBlock` enum after `OptionsList`
- Post-step: Updated three match expressions in `data.rs` (block_type_tag, block_id, block_children) to handle the new variant

## Implementation
Complete - 2026-04-02
