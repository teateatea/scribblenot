# Plan: Modal Transition ClipTranslate Height Fix

**Date:** 2026-04-17
**Status:** Proposed
**Priority:** High
**Recommended direction:** Fix `ClipTranslate` first, then verify the overlay height contract

---

## Purpose

Investigate and fix the transition-only modal strip bug where:
- modal bottom edges and rounded lower corners disappear during animation
- stub labels (`>`, `<`, `+`, `-`) visibly drop downward during animation
- the same modals render correctly at rest before and after the transition

The current evidence points to a transition-only vertical layout/clipping mismatch rather than a bad unit-packing or horizontal slide-distance calculation.

---

## Current Diagnosis

The strongest suspect is the custom transition wrapper in `src/ui/modal_unit.rs`:

- at-rest strip rendering uses `render_modal_unit(...)`
- in-transition strip rendering uses `render_connected_transition(...)`
- only the transition path wraps the strip in `ClipTranslate`

Why this is the highest-signal target:

- `ClipTranslate` hard-sets its own height in `size()`
- `ClipTranslate` builds its child layout from a custom `Limits` value instead of the parent limits
- `ClipTranslate` clips drawing to its own bounds
- the visible bug is strictly transition-only

That makes `ClipTranslate` the main structural difference between the good state and the broken state.

---

## Files In Scope

- `src/ui/modal_unit.rs`
- `src/ui/mod.rs`
- targeted tests in `src/ui/mod.rs` test module, or a new focused test location if refactoring makes that cleaner
- optional lesson/report docs only if implementation discovers something worth preserving

Out of scope for this pass:

- changing modal-unit packing rules
- redesigning horizontal transition motion
- collection-mode modal layout
- broader modal architecture cleanup

---

## Hypothesis

The transition strip is likely receiving or assuming more vertical space than the overlay actually has after:

- top offset
- composition panel
- column spacing
- other overlay container constraints

At rest, the strip is rendered through normal iced containers and appears correct.
During animation, `ClipTranslate` likely bypasses part of the normal height negotiation and then clips the lower portion of the strip to its own fixed envelope.

This would explain all observed symptoms:

- top edge stays aligned
- bottom edge is clipped away
- vertically centered stub labels appear lower because the visible viewport into the card is wrong during animation

---

## Recommended Fix Order

### 1. Fix `ClipTranslate` to respect parent height constraints

Revise `ClipTranslate` so its layout contract matches the height actually granted by its parent instead of assuming its own fixed ideal height is always available.

Implementation intent:

- inspect the parent `layout::Limits` passed into `ClipTranslate::layout(...)`
- clamp the widget's effective height to the parent limits instead of unconditionally using `self.height`
- build the child layout using those same resolved limits so the child and clip envelope agree
- keep the x-translation behavior intact

Success condition:

- transition path and at-rest path share the same effective vertical envelope
- lower modal corners and bottom border remain visible during animation
- stub labels no longer drop during the transition

### 2. Re-check overlay height budgeting immediately after

If step 1 reduces but does not fully eliminate the bug, audit `modal_overlay(...)` in `src/ui/mod.rs`.

Specifically verify whether `modal_height` is being derived from full viewport height while the actual remaining vertical space is smaller because of:

- `modal_top_offset(...)`
- composition panel height
- overlay column spacing

If needed, compute the modal strip height from the remaining available vertical space rather than from raw viewport height alone.

Success condition:

- the modal stream never requests more height than the overlay can actually display

---

## Implementation Steps

### Phase 1. Make the transition wrapper height-aware

In `src/ui/modal_unit.rs`:

1. Review `ClipTranslate::size()` and `ClipTranslate::layout()`.
2. Stop ignoring the incoming parent `layout::Limits`.
3. Resolve an actual envelope size from the parent limits.
4. Use the resolved height consistently for:
   - the widget node size
   - the child layout limits
   - the draw-time clipped viewport envelope
5. Keep the current horizontal translation model unchanged.

Key rule:

The clip envelope height and the child layout height must come from the same resolved value.

### Phase 2. Verify overlay contract

In `src/ui/mod.rs`:

1. Trace how `modal_height` is computed in `modal_overlay(...)`.
2. Compare that value against the actual remaining vertical space after:
   - top spacer
   - composition panel
   - spacing between stacked overlay elements
3. If a mismatch remains, adjust the overlay so the transition and at-rest paths both receive a height that can truly fit.

Key rule:

The overlay should own the available height budget; the transition widget should not guess it.

### Phase 3. Add regression coverage

Add focused tests for the newly clarified contract.

Minimum test targets:

- `ClipTranslate` height resolution respects parent limits
- transition envelope height matches the child layout height
- modal overlay height budgeting does not exceed remaining vertical space when the composition panel is present

If direct widget-layout assertions are awkward in iced:

- extract a small pure helper for resolved transition envelope height
- test that helper directly

---

## Validation Plan

### Automated

Keep the existing horizontal-motion tests passing:

- `cargo test --quiet connected_transition_strip`
- `cargo test --quiet transition_dep_unit_is_centred_in_viewport_at_p0`
- `cargo test --quiet transition_arr_unit_is_centred_in_viewport_at_p1`
- `cargo test --quiet transition_slide_distance_is_symmetric_for_forward_and_backward`

Add new focused coverage for the vertical contract introduced by this fix.

### Manual

1. Open a simple multi-modal field flow with visible stubs.
2. Trigger forward and backward transitions repeatedly.
3. Confirm the bottom border and lower rounded corners stay visible throughout the animation.
4. Confirm stub labels remain vertically centered and do not dip during motion.
5. Repeat with the composition panel visible.
6. Repeat on a shorter window height where the bug was previously obvious.

---

## Risks

### Risk 1. Fixing `ClipTranslate` exposes a second overlay-budget mismatch

This is likely, not alarming. It means the transition widget was masking part of the issue.

Mitigation:

- keep the follow-up overlay audit in the same implementation pass

### Risk 2. Over-correcting height changes the resting appearance

The resting path currently looks right.

Mitigation:

- keep the fix scoped to the transition path first
- only touch overlay budgeting if the first fix proves insufficient

### Risk 3. Iced layout internals make direct widget tests awkward

Mitigation:

- extract pure sizing helpers instead of trying to test every widget detail through full layout machinery

---

## Recommendation

Proceed with the narrowest viable change first:

1. make `ClipTranslate` honor parent height constraints
2. validate visually
3. only then adjust overlay height budgeting if the issue is not fully resolved

This preserves the current transition geometry and keeps the fix targeted at the only major rendering-path difference that matches the bug.
