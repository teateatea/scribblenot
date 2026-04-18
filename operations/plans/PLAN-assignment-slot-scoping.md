# Plan: Per-Slot Assignment Scoping For Multi-Field Format Lists

**Date:** 2026-04-18
**Status:** Proposed

## Purpose

Replace the current global `assigned_values: HashMap<String, String>` rendering model with slot-local assignment provenance so item `assigns` remain correct when:

- multiple fields assign the same target list
- repeating fields produce multiple confirmed entries
- one slot is edited without rewriting unrelated slots

The narrow fix clears stale state on modal back-out, but it does not solve the larger structural problem: confirmed assignments are still flattened into one global map, so later selections can silently overwrite earlier slot-specific values.

## Current Problem

Today the runtime keeps two different ideas of assignment state:

1. modal flow and nested/list state still know which item IDs were chosen
2. `App` flattens confirmed contributions into one app-wide `list_id -> output` map

That flattening leaks across boundaries:

- repeated entries in the same field can fight over the same assigned target list
- unrelated fields can overwrite each other's assigned target list outputs
- note rendering and field-label rendering cannot tell which assigned value belongs to which confirmed slot

## Recommendation

Make assignment provenance part of each confirmed slot's render context instead of maintaining one global assigned-value cache.

The safest version is:

- preserve the semantic source needed for assignment resolution with the confirmed value or an equivalent slot-owned sidecar
- derive assigned fallback values per slot at render time
- keep app-level mutable caches only as modal-session helpers, not as the source of truth for confirmed rendering

## Proposed Model

### Option A: Preserve structured source with confirmed values

Store enough structured state in confirmed values to re-derive assignments later.

Examples:

- keep `HeaderFieldValue::ListState` as the confirmed source for multi-list fields
- or introduce a small confirmed wrapper that stores:
  - display/export text
  - selected item IDs
  - repeat item IDs when relevant

Pros:

- no separate sync problem between displayed value and assignment provenance
- renderers can derive slot-local assignments directly from the confirmed value

Cons:

- larger change to existing assumptions that completed values collapse to `Text`

### Option B: Slot-owned assignment sidecar

Keep current confirmed display values, but store assignment contributions alongside each repeated slot.

Example shape:

```rust
struct ConfirmedSlotMeta {
    assigned_values: HashMap<String, String>,
}
```

Pros:

- smaller migration than making completed values fully structured again

Cons:

- still requires every mutating path to keep value text and sidecar state in sync

### Recommendation

Prefer Option A if the goal is a durable model. It keeps the render path honest about where assigned values came from and avoids another sidecar-sync trap.

If the change needs to stay smaller, Option B is acceptable, but it should still make slot ownership explicit and remove the global flattened cache from note rendering.

## Implementation Steps

1. Define the slot-owned source of truth.
Choose whether confirmed assignment provenance lives inside `HeaderFieldValue` or in a parallel per-slot structure attached to `HeaderState`.

2. Thread slot-local assignment context through render helpers.
Update `resolve_multifield_value`, `resolve_field_label`, note rendering, document rendering, and modal preview helpers so each confirmed value is rendered with its own assignment scope instead of the app-wide flattened map.

3. Remove global confirmed rendering dependence on `App::assigned_values`.
Keep any modal-session merged lookup only for in-flight preview/navigation, not for confirmed note output.

4. Rebuild modal reopen/edit paths around the new slot model.
Ensure reopening a confirmed field restores the same assignment provenance for that slot without needing an app-global cache.

5. Add regression coverage for collisions.
Cover:
- two repeated entries assigning the same format list with different outputs
- two different fields assigning the same target list
- edit-one-slot-without-changing-other-slots
- modal reopen/backspace/back-out behavior with preserved slot provenance

6. Clean up obsolete state.
Delete or shrink `assigned_contributions`, `AssignmentSourceKey`, and any render-time flattening helpers that become redundant.

## Validation

### Automated

Add tests proving:

- repeated entries render their own assigned format-list outputs independently
- cross-field collisions no longer overwrite earlier confirmed slots
- reopening a confirmed field preserves the slot's assignment provenance
- deleting one slot removes only that slot's assigned outputs

### Manual

Verify with real data that:

1. appointment time style fields still render correctly
2. editing one entry in a repeating field does not rewrite neighboring entries
3. note preview, editable document sync, and modal composition all agree on the same slot-specific output

## Expected Outcome

After this change, item `assigns` will behave like part of the confirmed field slot that produced them, not like a shared app-global override. That removes the current overwrite hazard and makes repeating multi-field data safe to expand.
