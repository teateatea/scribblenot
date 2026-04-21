# Plan: Modal Stub Option 2 - Semantic Stub Authority

**Date:** 2026-04-17
**Status:** Proposed (reviewed 2026-04-20)
**Scope:** Broader redesign
**Related Issue:** modal stub meaning is currently inferred from preview-sequence availability instead of real modal-flow semantics

## Purpose

Redesign stub classification so modal stub meaning comes from the real field engine rather than from whatever the preview system happens to be able to synthesize.

This is the more durable option if the modal system is expected to become substantially more complex over time.

## Plain-English Summary

Right now, the modal stream asks a visual question:

- "Can I build another preview snapshot to the right?"

If yes, the user sees `>`.
If no, the user sees `+`.

That is convenient, but it is the wrong authority.

The user does not care whether the preview layer can manufacture another snapshot. The user cares what will actually happen if they advance from here.

This option changes the source of truth:

- stubs should reflect real modal behavior
- teaser cards are a visual aid, not the authority on meaning

So instead of inferring stub type from preview-sequence length, the system should ask semantic questions such as:

- can the user advance to another modal from here?
- would advancing from here complete the field?
- would moving back from here leave the field?

Then the renderer uses those answers directly.

## Current Grounding In Code

Some of the "Option 1" groundwork already exists in the current tree:

- `src/modal.rs` now has repeat-joiner forward-preview coverage, including tests proving a repeat list can preview a real downstream list when that downstream list exists
- that means the original `obmuscle_field` mismatch is no longer the best sole justification for this plan

The remaining architectural problem is still real:

- at-rest stub choice in `src/ui/modal_unit.rs` is still derived from preview-sequence boundaries (`unit.start == 0`, `unit.end + 1 >= total_snapshots`)
- transition geometry in `src/transition.rs` freezes the same boundary-derived stub kinds into `UnitGeometry`
- so the system already has a semantic-looking vocabulary (`ModalStubKind`), but not a semantic authority

That is the gap this plan should close.

## Recommendation

Use this option if modal complexity is expected to keep growing through:

- more repeat-list patterns
- nested fields
- branch flows
- richer collection behavior
- future modal types that are not always previewable

Why:

- it reduces coupling between render shortcuts and real behavior
- it prevents the same class of bug from recurring in new modal forms
- it gives the modal system a clearer long-term contract

## Current Problem

The current architecture couples two things that should not be identical:

1. visual preview availability
2. semantic navigation meaning

That coupling was visible in the original `obmuscle_field` case, and it still exists anywhere preview generation is partial or intentionally conservative:

- the real modal flow can continue
- the preview layer may choose not to synthesize the downstream state
- the renderer then risks treating the edge as terminal and showing `+`

The bug is not only "preview generation is incomplete."

The deeper issue is that preview generation currently decides stub semantics.

## Design Goal

Establish an explicit semantic layer for modal edge actions.

The renderer should know:

- whether the left edge means exit or navigation
- whether the right edge means navigation or confirm

without having to infer those meanings from preview-sequence shape.

## Canonical Rule

Stub kind should be derived from real modal behavior, not from teaser availability.

That means:

- `NavLeft`: there is a meaningful previous modal state the user can navigate toward
- `Exit`: moving left from this semantic boundary leaves the field without confirming
- `NavRight`: confirming / advancing from the current modal can move to another modal state
- `Confirm`: confirming / advancing from the current modal would complete the field

This rule should hold even when no teaser card is available or practical to render.

## Proposed Architecture

### 1. Add a semantic edge model

Introduce a small semantic layer that can answer edge-action questions for the current modal state.

Possible shape:

```rust
pub struct ModalEdgeSemantics {
    pub left: ModalStubKind,
    pub right: ModalStubKind,
}
```

Prefer reusing `ModalStubKind` unless the implementation proves a broader vocabulary is required. The codebase already uses that enum as the user-facing meaning layer, so introducing a second near-duplicate enum would add churn without much payoff.

The key requirement is not "new enum names." It is that these values are derived from behavior, not from preview shape.

### 2. Derive semantics from the real modal engine

Add helpers in `src/modal.rs` that evaluate the current `SearchModal` state using the same real rules used by confirmation and backtracking.

Examples:

- `can_go_back_semantically()`
- `advance_outcome_for_current_state(...)`
- `edge_semantics(...)`

For the right edge specifically, the helper should determine whether the current modal state would:

- stay on the same list
- advance to another list
- complete the field

The stub should only be `Confirm` when the semantic outcome is completion.

Important implementation constraint:

- do not derive edge semantics from `SimpleModalSequence`
- do not duplicate a second handwritten copy of advance rules just for the renderer
- prefer one non-mutating helper path that simulates the same engine decisions the live modal uses

### 3. Separate teaser generation from stub meaning

Keep teaser generation for visual context, but do not let it decide edge meaning.

That means the system may validly render:

- a `NavRight` stub even if no right teaser card is shown

if real behavior still advances to another modal state.

This is the important architectural shift in this option.

### 4. Update render code and frozen transition geometry to consume semantic edge actions

In `src/ui/modal_unit.rs`, stop deriving stub type only from:

- first sequence unit
- last sequence unit
- preview sequence length

Instead, combine:

- unit position information
- semantic edge action information

The render layer should still use preview sequence data for card content and layout, but not as the sole authority for stub kind.

Also update `src/transition.rs::UnitGeometry::from_layout`. Today it freezes leading/trailing stub kinds from unit boundaries before animation starts. If only the still renderer changes, transition strips will continue to carry stale preview-derived semantics.

### 5. Keep preview helpers as preview helpers

The preview system can remain conservative.

For example:

- a branch-heavy state may choose not to render a forward teaser
- a repeat-list state may choose not to render a full next card

But that should not force the right stub to become `Confirm` if the real modal flow is not terminal.

## Implementation Phases

### Phase 1: Semantic helper design

- define vocabulary and data shape
- identify the real modal outcomes needed by the UI
- document how repeat, branch, nested, collection, and filtered-search states map to edge semantics

### Phase 2: Modal-engine helpers

- implement outcome helpers in `src/modal.rs`
- ensure helpers do not mutate live modal state
- use cloned / simulated state where necessary

### Phase 3: Still-render integration

- feed semantic edge data into `build_rendered_modal_unit`
- replace the current `unit.start` / `unit.end` stub inference
- keep teaser layout behavior unchanged unless needed

### Phase 4: Transition integration

- thread the same semantic edge data into `UnitGeometry::from_layout`
- keep transition-strip ownership rules unchanged
- verify still and animated strips agree on edge meaning

### Phase 5: Test hardening

- cover repeat-list downstream advancement
- cover true terminal states
- cover branch transitions
- cover nested modal progression
- cover states where teaser cards are absent but nav semantics still exist
- cover filtered/search-active states explicitly so "no teaser because the user is searching" does not accidentally imply terminal flow

## Expected Code Areas

- `src/modal.rs`
- `src/ui/modal_unit.rs`
- `src/transition.rs`
- tests in `src/ui/mod.rs`
- `src/modal_layout.rs` only if carrying semantics through layout structs becomes the cleanest seam

`src/app.rs` should stay out of this unless a concrete ownership problem appears. The semantic questions belong closer to `SearchModal` than to the top-level app event loop.

## Product Rules By Modal Type

### Simple sequential list

- if another list follows, right stub is `NavRight`
- if no list follows, right stub is `Confirm`

### Repeat-joiner list

- if finishing the repeat can advance to another list, right stub is `NavRight`
- if finishing the repeat would complete the field, right stub is `Confirm`
- staying on the same repeat list does not by itself make the right edge a confirm boundary

### Branch state

- if the selected item leads into branch fields, the right edge remains navigational
- branch preview absence must not imply semantic completion

### Nested state

- stubs must reflect nested field progression, not only leaf preview visibility

### Collection state

- collection navigation semantics should be defined explicitly rather than inferred indirectly from collection preview layout

## Testing Plan

Add or update tests for:

1. Semantic helper unit tests in `src/modal.rs`:
- repeat-joiner with downstream list yields right `NavRight`
- true terminal repeat yields right `Confirm`
- branch flow remains navigational before branch completion
- nested flow remains navigational before leaf completion

2. Still-render tests:
- rendered right stub matches semantic helper output rather than preview count alone

3. Transition-geometry tests in `src/transition.rs` or `src/ui/mod.rs`:
- frozen leading/trailing stub kinds match the same semantic helper output
- forward and backward transitions preserve the expected shared transition stub

4. Preview-absence / no-teaser tests:
- semantics remain correct when preview helpers return fewer or no teaser cards
- searching/filtering does not silently flip a navigational edge into `Confirm`

## Manual Verification

1. Verify `obmuscle_field` shows a right nav stub on `muscle`.
2. Verify advancing from `muscle` still lands in `place`.
3. Verify a truly terminal final modal shows green `+`.
4. Verify branch and nested modal flows do not regress.
5. Verify cramped or non-simple states can still show semantically correct stubs even without full teaser coverage.

## Risks

- Larger implementation surface than the focused fix
- Requires careful vocabulary and ownership decisions
- May expose other existing mismatches once semantic truth is made explicit
- Could pull more modal types into scope than initially intended
- If semantic simulation drifts from the live confirm/backtrack code, the UI could gain a different class of mismatch than the one this plan is trying to remove

## Tradeoffs

### Pros

- Better long-term architecture
- Stub meaning matches real behavior
- Less likely to regress as modal complexity grows
- Makes future debugging simpler because semantic authority is explicit

### Cons

- More code changes
- More tests required
- Higher short-term risk than the focused fix
- May lead naturally into a wider cleanup once the semantic layer exists

## Recommendation Relative To Option 1

If the immediate priority is a safe bug fix, do Option 1 first.

That said, the current tree has already absorbed a meaningful part of Option 1: repeat-joiner downstream previews are now covered in `src/modal.rs`. So the staging value of "do more preview-only work first" is lower than it was when this plan was first written.

If the priority is long-term clarity and the modal system is expected to keep evolving, this option is the stronger foundation.

A reasonable staged approach would be:

1. implement Option 1 to remove the user-facing mismatch quickly
2. later implement this option as the architectural cleanup

Or, if you want to invest now and avoid layering more heuristics on top of preview logic, implement this option directly.

## Coordination Note With #33

Keep task `#33` separate in implementation even though both tasks live in the same modal/UI lane.

- `#28` is about semantic authority and correctness
- `#33` is about active-row width/layout behavior

They can share review context, but they should not become one mixed patch. Semantic seam changes are easier to validate when visual width churn is not happening in the same diff.

## Exit Criteria

This option is complete when:

- stub kind is no longer solely inferred from preview sequence length
- repeat, branch, nested, and other modal types have explicit semantic edge rules
- `obmuscle_field` and similar cases render correct stub meaning
- teaser absence no longer causes semantic misclassification
- automated tests cover the new semantic contract
