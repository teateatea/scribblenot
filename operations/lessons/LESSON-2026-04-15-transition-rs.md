# Lesson: transition.rs

**Date:** 2026-04-15
**Project:** scribblenot-p2b
**Target:** src/transition.rs
**Files covered:**
- src/transition.rs
**Depth:** What & Why
**Branch:** modal-transition-p2-b

---

## Sub-sections

### FocusDirection

**What it is:** A label for which way the user navigated - Forward (higher-index unit) or Backward (lower-index unit).
**Why it exists:** The strip slides opposite to focus direction; this enum tells the rest of the code which way to push the animation.

### ModalTransitionEasing

**What it is:** A menu of animation curve options, plus an `apply()` function that shapes raw 0-to-1 progress into a curve.
**Why it exists:** Constant-speed animation looks mechanical. Easing lets motion accelerate/decelerate naturally. `apply()` is called each frame with elapsed time as a fraction and returns the shaped position value.

### UnitGeometry

**What it is:** A frozen record of one unit's physical dimensions - card widths, horizontal offsets, spacer width, stub kinds.
**Why it exists:** Without freezing, a window resize mid-animation could shift the departing unit's geometry and cause jitter. The snapshot insulates the animation from any live layout changes.

### UnitContentSnapshot

**What it is:** A frozen copy of the modal snapshots (field data) visible in a unit at transition start.
**Why it exists:** Only the departing unit uses this - its content is frozen so that user typing in the arriving unit (or any shared state change) can't visually update the card mid-animation. Also keeps content consistent with the frozen geometry, which was calculated from the same snapshot.

### ModalDepartureLayer

**What it is:** Mirror of ModalArrivalLayer for the outgoing unit. Same clock/easing fields, but adds frozen content and geometry since focus has left.
**Why it exists:** The departing unit must keep rendering while it animates out, but everything about it should be locked. Kept as a separate struct from ModalArrivalLayer so that Part 3 can give it independent timing (e.g. fade out faster than arrival slides in).
**Notes:** User observed that both layers currently fire at the same time with the same settings, so merging them would work today - the separation is a forward-looking bet on Part 3 independent timing.

### ModalTransitionLayer

**What it is:** An enum that bundles ModalArrivalLayer, ModalDepartureLayer, and slide_distance into one animation entry pushed onto the modal_transitions list.
**Why it exists:** The renderer pulls one entry and draws both layers from it, ensuring a consistent matched pair. It's an enum rather than a struct so Part 3 can add a second variant for queued transitions.

### unit_display_width

**What it is:** A utility that computes the total pixel width of a unit from its frozen geometry - stub widths (if any), modal card widths, and spacers.
**Why it exists:** Used by app.rs to calculate slide_distance before creating transition layers. Works from frozen geometry so it doesn't touch the live layout.

### Per-frame position math (ui/modal_unit.rs)

**What it is:** The shift formula inside `render_connected_transition` that converts `eased_progress` and `slide_distance` into a pixel offset for the strip each frame.
**Why it exists:** This is where all the frozen data finally becomes visible motion. Forward subtracts `slide * p` (strip slides left); backward negates the formula so the strip slides right. `row_width` and `viewport_width` are used to centre the departing unit at p=0.
**Notes:** During the lesson we spotted that an earlier version of this formula didn't branch on direction - another instance had already fixed it to use a match on `focus_direction`.
**Notes:** User asked why this is necessary if the departing unit has no focus. Key reason: the frozen geometry (modal_widths) was calculated from this content - if live content reflowed to a different size, geometry and rendering would disagree and misalign the animation.

### ModalArrivalLayer

**What it is:** The data for the incoming unit - owns the animation clock (started_at, duration_ms, easing) plus frozen geometry for positioning.
**Why it exists:** The renderer needs to ask "how far along is this animation?" each frame. This struct provides that via progress(), eased_progress(), and is_finished(). Content is deliberately live since focus is here.

---

## Summary

<!-- Added when the lesson is closed -->

---

## Changes & Improvements Noticed

<!-- Any bugs, improvements, or oddities noticed during the lesson -->
