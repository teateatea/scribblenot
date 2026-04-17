# Plan: Modal Stub Option 2 - Semantic Stub Authority

**Date:** 2026-04-17
**Status:** Proposed
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

That coupling is visible in the `obmuscle_field` case:

- the real modal flow can continue from `muscle` to `place`
- but the preview builder does not generate downstream snapshots for repeat-joiner lists
- the renderer then treats the right edge as terminal and shows `+`

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
    pub left: ModalEdgeAction,
    pub right: ModalEdgeAction,
}

pub enum ModalEdgeAction {
    Exit,
    Navigate,
    Confirm,
    None,
}
```

The exact shape can vary, but the key is that this model describes behavior, not rendering.

### 2. Derive semantics from the real modal engine

Add helpers in `src/modal.rs` that evaluate the current `SearchModal` state using the same real rules used by confirmation and backtracking.

Examples:

- `can_go_back_semantically()`
- `advance_outcome_for_current_state(...)`
- `semantic_right_edge(...)`

For the right edge specifically, the helper should determine whether the current modal state would:

- stay on the same list
- advance to another list
- complete the field

The stub should only be `Confirm` when the semantic outcome is completion.

### 3. Separate teaser generation from stub meaning

Keep teaser generation for visual context, but do not let it decide edge meaning.

That means the system may validly render:

- a `NavRight` stub even if no right teaser card is shown

if real behavior still advances to another modal state.

This is the important architectural shift in this option.

### 4. Update render code to consume semantic edge actions

In `src/ui/modal_unit.rs`, stop deriving stub type only from:

- first sequence unit
- last sequence unit
- preview sequence length

Instead, combine:

- unit position information
- semantic edge action information

The render layer should still use preview sequence data for card content and layout, but not as the sole authority for stub kind.

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
- document how repeat, branch, nested, and collection modes map to edge semantics

### Phase 2: Modal-engine helpers

- implement outcome helpers in `src/modal.rs`
- ensure helpers do not mutate live modal state
- use cloned / simulated state where necessary

### Phase 3: UI integration

- thread semantic edge data into modal-unit rendering
- replace current stub-kind inference where appropriate
- keep teaser layout behavior unchanged unless needed

### Phase 4: Test hardening

- cover repeat-list downstream advancement
- cover true terminal states
- cover branch transitions
- cover nested modal progression
- cover states where teaser cards are absent but nav semantics still exist

## Expected Code Areas

- `src/modal.rs`
- `src/modal_layout.rs` if semantic data is threaded through layout structs
- `src/ui/modal_unit.rs`
- possibly `src/app.rs` if semantic helpers need app-owned context

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

1. Repeat-joiner with downstream list:
- semantic right edge is `NavRight`
- rendered right stub is not `Confirm`

2. Terminal repeat-joiner:
- semantic right edge is `Confirm`

3. Branch flow:
- semantic right edge is navigational before branch completion

4. Nested flow:
- semantic right edge reflects next required nested modal

5. Preview absence:
- semantic right edge remains correct even when no teaser card is rendered

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

If the priority is long-term clarity and the modal system is expected to keep evolving, this option is the stronger foundation.

A reasonable staged approach would be:

1. implement Option 1 to remove the user-facing mismatch quickly
2. later implement this option as the architectural cleanup

Or, if you want to invest now and avoid layering more heuristics on top of preview logic, implement this option directly.

## Exit Criteria

This option is complete when:

- stub kind is no longer solely inferred from preview sequence length
- repeat, branch, nested, and other modal types have explicit semantic edge rules
- `obmuscle_field` and similar cases render correct stub meaning
- teaser absence no longer causes semantic misclassification
- automated tests cover the new semantic contract
