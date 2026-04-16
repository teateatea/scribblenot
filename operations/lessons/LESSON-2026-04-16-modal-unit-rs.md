# Lesson: modal_unit.rs

**Date:** 2026-04-16
**Project:** scribblenot-p2b
**Target:** src/ui/modal_unit.rs
**Files covered:**
- src/ui/modal_unit.rs
- src/ui/mod.rs (context)
**Depth:** What & Why
**Branch:** fix/modal-transition-backward-direction

---

## Sub-sections

### The card taxonomy

**What it is:** The complete set of types that describe every possible card in a modal strip.
**Why it exists:** Before the strip can be painted, each card needs a label - is it active, a dim preview, or a stub button? These four types (ModalUnitStubMode, ModalUnitSide, ModalUnitCardKind, ModalUnitCardData) provide that vocabulary.

### RenderedModalUnit

**What it is:** The output of the assembly step - a left-to-right ordered list of card descriptions for the whole strip.
**Why it exists:** The painting functions need a single, complete picture of the strip before they start producing UI widgets. This struct is that picture, plus a total_width helper that sums card widths and inter-card gaps.

### build_rendered_modal_unit

**What it is:** The function that assembles a still (at-rest) modal strip into a RenderedModalUnit.
**Why it exists:** Something has to decide stub labels based on position (Exit vs NavLeft, Confirm vs NavRight), mark which card is Active vs Preview, and collect everything into an ordered list. This function does that for the normal, non-animating case.

### modal_unit_runway_layout

**What it is:** A math function that computes left padding, right padding, and outer container width to position the card strip in the viewport.
**Why it exists:** The strip is often wider than the viewport and needs to slide sideways during transitions. An oversized outer container (the "runway") gives it room to move without being clipped; left_pad and right_pad shift its position within that container.
**Notes:** User asked for a second pass with realistic numbers (viewport 800, strip 1200, shift 100). Key insight: base_offset goes negative when the strip is wider than the viewport, and runway absorbs both the overhang and the shift distance.

### render_modal_unit

**What it is:** The painting function for the still strip - converts a RenderedModalUnit into actual iced UI widgets.
**Why it exists:** Card descriptions (ModalUnitCardData) are just data; something has to turn them into visible elements. This function does that for the non-animating case, using the runway math to center and position the result.

### render_connected_transition

**What it is:** The painting function for the animated transition strip - identical card loop to render_modal_unit, but with a live shift computed from progress p.
**Why it exists:** During a transition the strip must slide across the viewport frame by frame. The shift formula centers the departure unit at p=0 and slides the whole strip so the arrival unit is centered at p=1. clip(true) ensures nothing outside the viewport boundary is visible.
**Notes:** User clarified that "slide * p pulls it toward the arrival unit" was imprecise - the whole strip moves, departure exits one side, arrival enters from the other.

### push_strip_stub

**What it is:** A small helper that pushes a single stub card onto a card list.
**Why it exists:** The transition builder needs to insert stub cards in several places; this avoids repeating the same struct construction each time, and adds alpha support for fading stubs during animation.

---

## Summary

<!-- Added when the lesson is closed -->

---

## Changes & Improvements Noticed

<!-- Any bugs, improvements, or oddities noticed during the lesson -->
