# Lesson: modal_layout.rs - How Modal Units Are Prepared

**Date:** 2026-04-15
**Project:** scribblenot-p2b
**Target:** src/modal_layout.rs
**Files covered:**
- src/modal_layout.rs
**Depth:** What & Why
**Branch:** modal-transition-p2-b

---

## Sub-sections

### Data shapes (8-44)

**What it is:** The types that describe a single modal panel's state, a sequence of panels, and how panels are grouped into screen-fitting units.
**Why it exists:** The renderer and the layout algorithm need a shared vocabulary for "what is a panel" and "how are panels arranged" before any drawing or geometry work can happen.

### Unit grouping algorithm

**What it is:** The main packing function - walks through all panels left to right, greedily fitting as many as possible into each unit before starting a new one.
**Why it exists:** Panels need to be grouped into screen-fitting chunks before the renderer can position them. This is where all the size and spacing prep work from earlier sections gets used.
**Notes:** Two edge cases: unknown viewport width (falls back to one unit containing everything) and oversized first panel (stubs hidden, panel uses full viewport width).

### Height helpers

**What it is:** Two functions - one sets modal height to 80% of the viewport (floor 160px, fallback if viewport unknown), the other calculates how many list rows fit after subtracting the chrome area.
**Why it exists:** The renderer needs to know the modal's dimensions before drawing it. Vertical position lives elsewhere; this is only about size.
**Notes:** User confirmed this section does not handle vertical positioning - that lives in the rendering and transition code.

### Spacing helpers

**What it is:** Two small calculations - one caps the gap size between panels at 0.5% of the viewport, the other subtracts stub and spacer space from both sides to produce the usable content width.
**Why it exists:** The packing algorithm needs to know exactly how much horizontal room is available before it can decide which panels fit in a unit.

### Stub symbols

**What it is:** The four named action markers (`<`, `>`, `-`, `+`) shown at the edges of a unit, and the helper that maps each to its display character.
**Why it exists:** The renderer needs a typed way to ask "what symbol goes here" rather than hardcoding characters at the call site.
**Notes:** Stubs only appear when `shows_stubs: true` on the unit - panels too wide to leave room for stub cards bypass this entirely.

---

## Summary

`modal_layout.rs` is the file that prepares modal panels for display - it defines what a panel is, decides how big it should be, groups panels into screen-fitting units, and calculates modal height. It sits between raw app state and the renderer, giving both layers a shared set of types and measurements to work from. The six sections build on each other: the data shapes come first, then sizing logic, then the stub markers used at unit edges, then two spacing helpers that feed into the main packing algorithm, and finally height helpers for the vertical dimension. Vertical positioning is out of scope here - that lives in the rendering and transition code.

---

## Changes & Improvements Noticed

- The `effective_spacer_width_does_not_exceed_theme_value` test was testing with a 3000px viewport and 40px theme cap - with the new 0.5% multiplier that's 15px, well under the cap, so the test was actually verifying nothing about the cap. Updated to use a 200px viewport with a 3px cap so the cap behaviour is actually exercised.
- The hardcoded fallback `modal_spacer_width: 40.0` in `theme.rs:433` was not updated when `default-theme.yml` was changed to 3px. These two values now diverge - worth checking whether the fallback matters in practice.
