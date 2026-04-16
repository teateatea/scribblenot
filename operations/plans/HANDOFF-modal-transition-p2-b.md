# Handoff: Modal Transition Part 2B

**Date:** 2026-04-15
**Branch:** `modal-transition-p2-b`
**Worktree:** `C:\Users\solar\Documents\Claude-Projects\scribblenot-p2b`
**Status:** Partial implementation only. Do not treat as finished.

---

## What Was Implemented

Work completed in [src/ui.rs](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/src/ui.rs:1909):

- added per-card `alpha` to `ModalUnitCardData`
- added `build_connected_transition_rendered_unit(...)`
- replaced split `dep_shift` / `arr_shift` with one shared `strip_shift`
- made transition-stub ownership explicit in the connected-strip builder
- removed transition-path reliance on ghost stubs
- added focused strip-composition tests

Added docs:

- [REPORT-modal-transition-p2-b.md](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/operations/plans/REPORT-modal-transition-p2-b.md:1)

---

## Validation Completed

Automated checks passed:

- `cargo check --quiet`
- `cargo check --quiet --tests`
- `cargo test --quiet connected_transition_strip`
- `cargo test --quiet`

Result: `117` tests passed.

---

## Manual Verification Result

Manual verification failed. The user ran `cargo run` from this worktree and reported that the core visual bugs are still present.

Observed failures:

1. Crossing the left stub:
   - the departing unit only slides until its left edge reaches the left window edge
   - then it stops sliding and only fades
   - the arriving unit waits in the wrong position, then snaps into place when the departure is gone

2. Left-stub crossing still appears much faster than right-stub crossing.

3. Crossing the right stub:
   - the arriving unit still appears to grow rather than truly slide

Conclusion:

- this branch improved strip ownership logic
- but it did **not** solve the root geometry problem

---

## Why It Is Still Broken

The main remaining problem is that transition movement is still being faked by runway padding math in:

- [src/ui.rs](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/src/ui.rs:2095) `modal_unit_runway_layout()`
- [src/ui.rs](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/src/ui.rs:2142) `render_modal_unit()`

So even though the transition now uses one connected strip and one shared shift value, the strip is still rendered through:

- dynamic left/right padding
- dynamic outer width
- centered row layout

That means the transition path is still **not** a true translated and clipped viewport envelope.

This likely explains all three remaining visual failures:

- edge-stop instead of off-screen slide
- width growth/shrink at the edge
- snap after fade

The left/right speed mismatch is also still likely influenced by `slide_distance` creation in:

- [src/app.rs](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/src/app.rs:2807)

The current formulas can still produce noticeably different travel distances between boundary cases.

---

## Next Correct Fix

The next implementation pass should **not** keep patching the runway approach.

It should do these things:

1. Remove transition-path dependence on `modal_unit_runway_layout()`.
2. Replace transition rendering with a real clipped viewport/envelope model.
3. Keep card widths fixed for the whole animation.
4. Move the connected strip by translation, not by changing padding/outer width.
5. Re-check `slide_distance` symmetry in `src/app.rs` after the new viewport model is in place.

In short:

- connected strip logic is useful and can be kept
- runway-layout movement is the real remaining blocker and must be replaced

---

## Safe Reuse vs Rework

Safe to keep:

- connected-strip card ownership in `build_connected_transition_rendered_unit(...)`
- per-card alpha model
- focused strip tests

Needs replacement or major rewrite:

- transition-path use of `render_modal_unit(...)` as currently written
- transition-path use of `modal_unit_runway_layout()`

---

## Current Branch State

Uncommitted files in this worktree:

- modified: [src/ui.rs](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/src/ui.rs:1909)
- new: [REPORT-modal-transition-p2-b.md](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/operations/plans/REPORT-modal-transition-p2-b.md:1)
- new: [HANDOFF-modal-transition-p2-b.md](C:/Users/solar/Documents/Claude-Projects/scribblenot-p2b/operations/plans/HANDOFF-modal-transition-p2-b.md:1)

No commit was created.

