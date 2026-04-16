# Implementation Report: Modal Transition Part 2B - Connected Strip Rendering

**Date:** 2026-04-15
**Branch:** `modal-transition-p2-b`
**Status:** Complete

---

## Summary

Part 2B finishes the visual half of the Part 2 transition rewrite.

The old Part 2 renderer still animated the departing and arriving units as two separately positioned layers. This branch replaces that with one connected in-flight strip:

- one shared horizontal offset
- explicit transition-stub ownership
- per-card alpha so the transition stub stays fully visible while the two sides fade independently

This keeps the existing Part 2 lifecycle/state model in `src/app.rs` and does not overlap Part 3 adaptive timing/queue work.

---

## What Changed

### `src/ui.rs`

- Removed transition-path use of ghosted stubs
- Added per-card alpha to `ModalUnitCardData`
- Added `build_connected_transition_rendered_unit(...)`
- Replaced the split `dep_shift` / `arr_shift` transition render path with one shared `strip_shift`
- Kept at-rest rendering on the existing `build_rendered_modal_unit(...)` path

### Tests

Added focused transition-composition tests for:

- single forward transition-stub ownership
- single backward transition-stub ownership

Existing runway-layout tests remain in place and still pass.

---

## Validation

Automated validation completed:

- `cargo check --quiet`
- `cargo check --quiet --tests`
- `cargo test --quiet connected_transition_strip`
- `cargo test --quiet`

Result: `117` tests passed.

---

## Scope Boundary

This branch does **not** implement Part 3 work:

- no adaptive duration tuning
- no queued transitions
- no persistence changes
- no keybinding changes

Part 3 should build on this renderer rather than re-solving shared-strip motion or transition-stub ownership.

