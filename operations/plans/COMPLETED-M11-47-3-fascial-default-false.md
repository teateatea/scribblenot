## Task

#47 - Add per-technique default selection state to block_select

## Context

The user nearly always uses the first three techniques in LOWER BACK (Prone) and only rarely uses the Fascial entry. A failing test (`lower_back_prone_fascial_l4l5_starts_unselected` at `src/data.rs:498`) asserts that the `fascial_l4l5` entry in the `back_lower_prone` region has `default: false`. Currently the entry has no `default` field, which means it is treated as selected by default, causing the test to fail.

No other regions contain a `fascial_l4l5` entry or any analogous fascial technique that the user has indicated should start unselected. The change is intentionally scoped to the one entry the user described.

## Approach

Add `default: false` to the `fascial_l4l5` entry inside the `back_lower_prone` region in `data/tx_regions.yml`. No other files need to change.

## Critical Files

- `data/tx_regions.yml` - lines 49-51 (the `fascial_l4l5` entry inside `back_lower_prone`)

## Reuse

No new utilities needed. The existing `default_selected()` method on `PartOption` (already used by the test) reads the `default` field from the parsed YAML.

## Steps

1. Open `data/tx_regions.yml` and locate the `fascial_l4l5` entry under `back_lower_prone` (lines 49-51). Add `default: false` as a new field after `output`.

```
-      - id: fascial_l4l5
-        label: "Fascial Techniques (L4-L5)"
-        output: "- Fascial Techniques (L4-L5)"
+      - id: fascial_l4l5
+        label: "Fascial Techniques (L4-L5)"
+        output: "- Fascial Techniques (L4-L5)"
+        default: false
```

2. Run `cargo test` and confirm `lower_back_prone_fascial_l4l5_starts_unselected` passes and no other tests regress.

## Verification

### Manual tests

- Launch the app, add the LOWER BACK (Prone) region to a session. Confirm the Fascial Techniques (L4-L5) checkbox starts unchecked while the other three entries (Swedish, Specific Compressions, Muscle Stripping) start checked.

### Automated tests

- `cargo test lower_back_prone_fascial_l4l5_starts_unselected` - must pass (this is the pre-written failing test that this change is designed to fix).
- `cargo test` (full suite) - must pass with no regressions.

## Changelog

### Review - 2026-04-02
- #1 (nit): Corrected test line reference from `src/data.rs:494` to `src/data.rs:498` (494 is a comment, 498 is the function).
- #2 (nit): Corrected Reuse section: `default_selected()` is on `PartOption`, not `FlatBlock`.
