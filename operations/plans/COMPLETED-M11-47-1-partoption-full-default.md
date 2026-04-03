## Task

#47 - Add per-technique default selection state to block_select

## Context

`PartOption::Full` is the variant used when a block_select entry needs an `id` field. The UI needs a `default` boolean to know which options start pre-selected. The tests for this behaviour already exist in `src/data.rs` (lines 456-479, module `part_option_default_tests`) but the code does not compile because the `Full` variant is missing the `default` field and the `default_true` serde helper.

## Approach

Add `default: bool` to the `Full` struct variant, annotated with `#[serde(default = "default_true")]`, and add the module-level helper `fn default_true() -> bool { true }`. No other files need changes; all match sites use `..` and will continue to compile unchanged.

## Critical Files

- `src/data.rs` lines 10-16 (enum definition), and a new helper function to be inserted after line 39 (end of `impl PartOption`).

## Reuse

- Existing serde `#[serde(default)]` pattern already used on `sticky: bool` in `CompositePart` (line 50). The `default_true` helper uses the named-function form `#[serde(default = "default_true")]` to supply `true` instead of the type's `Default` impl (which would give `false`).

## Steps

1. Add `default: bool` to the `Full` variant and annotate it with the serde helper.

```diff
-    Full { id: String, label: String, output: String },
+    Full { id: String, label: String, output: String, #[serde(default = "default_true")] default: bool },
```

2. Insert the `default_true` helper function after the closing brace of `impl PartOption` (after line 39), before the `#[derive]` for `CompositePart`.

```diff
+fn default_true() -> bool { true }
+
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct CompositePart {
```

## Verification

### Manual tests

None - this is pure data-model logic with no UI surface for this sub-task.

### Automated tests

Run the existing unit tests in `src/data.rs`:

```
cargo test part_option_default_tests
```

Expected output: both `full_without_default_field_yields_default_true` and `full_with_default_false_yields_false` pass.

## Changelog

### Review - 2026-04-02
- #1: Corrected Reuse note - `sticky` uses `#[serde(default)]` not the named-function form; clarified that `default_true` is needed because `bool::default()` returns `false`, not `true`.
