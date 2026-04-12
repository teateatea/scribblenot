# Plan: Modal Unit And Transition Redesign

**Date:** 2026-04-11
**Status:** Proposed
**Supersedes:** transition and unit-behavior portions of `PLAN-modal-stream-v2-stubs-motion-composition.md`

## Purpose

Capture the next modal-stream redesign before implementation so the app can move from a good prototype to a stable, explicit unit/transition model.

This plan is intentionally narrower than the earlier V2 plan:

- keep the composition panel and override work as-is
- replace the current modal-unit transition behavior with an explicit prepared/active/departing/arriving model
- preserve immediate keyboard interaction on the arriving unit even while the animation is still running

## Plain-English Summary

The current system already does some useful things:

- it precomputes simple modal units in `src/modal.rs`
- it has theme knobs for spacer width, stub width, easing, and duration
- it can animate outgoing and incoming unit layers in `src/ui.rs`

But the current behavior is still organized around "old visible unit" and "new visible unit" snapshots rather than around explicit prepared units and a stable transition lifecycle.

The redesign should make the model easier to reason about:

- one active unit is on-screen and interactive
- one previous unit is prepared off-screen on the transition side opposite focus movement
- one next unit is prepared off-screen in the focus direction
- when focus crosses a unit boundary, it lands immediately in the prepared next unit
- that prepared next unit becomes the arriving unit and active unit at once
- the old active unit becomes the departing unit
- a fresh next unit is prepared immediately for the next possible move

This keeps advanced interaction fast while making render rules, stub behavior, and animation state more explicit.

## Verbatim User Reference

The following text is preserved verbatim so future implementation work can refer back to the original wording rather than only a summarized interpretation.

> Hi! I'd like to redo the modal units and transition system, because it's currently a good prototype but it has some foundational issues.
>
> First, some vocabulary definitions to make it easier to talk about this:
> - modal unit (or just unit): A group of modals visible at the same time
> - focus: Where the user's > cursor is, with yellow highlight.
> - active unit: The unit that's visible on-screen, contains the modal with focus.
> - prepared unit: A unit that's "off-screen", and thus inactive to the user. The focus will land in a modal here on transition. The intent with this is that focus *immediately* lands in this modal and unit, so that advanced users can start interacting with the modal (arrow keys, hintkeys, search, etc) and don't need the transition to fully complete the animation. Technically, "off-screen" actually just means invisible, with inactive hotkeys, but the prepared unit should actually be arranged one spacer lateral to the stub on its side. So it's prepared in the position it needs to be, and not actually fully off the screen.
> - transition: When focus leaves the on-screen unit, it should land in a modal in a prepared unit off-screen. This triggers a transition, where units will change positions/status, and we'll have some animations.
> - departing unit: When a transition begins, this is the unit with the modal that we just left, that will be transitioning to not visible.
> - arriving unit: When a transition begins, this is the unit with the modal that we've just focused on, that will be transitioning to visible.
> - transition direction: opposite to the focus movement. When we move the focus to the right (off the right stub), the transition direction is left.
>
> - previous unit: Just before the transition begins, this is the prepared unit off-screen in the transition direction (focus is moving away from this unit). It can be dropped when the transition starts, because it no longer needs to be an available landing spot for focus.
> - next unit: Just before the transition begins, this is the prepared unit off-screen in the focus direction. Focus will land in a modal here, then when the transition begins, this becomes the arriving unit, and a new next unit will be prepared.
> - spacers: a unit of empty space, separators between the modals in a unit. These should be 2% of the window width (or height, as appropriate), or modal_spacer_width: 40px (please add this as a theme knob), whichever is less.
> - stubs: currently implemented as indicators on the sides of modal units, to indicate there are more modals in this direction. Let's set these at modal_stub_width: 120px (add as theme knob please), and instead of having nav_labels, they just have < or > arrows (centered please)
> - transition stub: The stub on the focus side, between the arriving and departing units.
> - unit bounding box: The space available for full sized (non-stub) modals. This can be calculated as: [window width] - (2*([stub_width]+[spacer width])). The modals also need spacers between them, so something like [spacer width]*([number of modals]-1) also gets used up. Note that this automatically reserves space for stubs, so we can more focus on whether or not we can fit modals.
>
> To determine modal units:
> - The first modal always gets added to the unit. If this modal would exceed the bounding box, then stubs are omitted. If this modal would exceed the window size, it instead matches the window size (and scrollbars are used).
> - Then in order, check the next modal's width. If a modal does not exceed the bounding box, it gets added to the unit. If a modal does exceed the bounding box, don't add it and stop checking.
> When to create modal units:
> - When the window changes size, re-create the modal units (keeping the focused modal on-screen of course).
> - On program open, and on data refresh, we can actually pre-calculate these. There's no need to calculate these on the fly. If we don't already do some kind of data-change check, please add to the roadmap.md that we could probably use a high performance checksum (crc32?) to reduce redundant re-calculations, but that's outside the scope of this change for today.
>
> When a transition is triggered (focus departs the active unit and lands in the prepared unit's modal), these things happen:
> - The next unit is now the arriving unit (it also is now and will be the active unit, it's immediately interactable.)
> - The (previously) active unit becomes the departing unit.
> - A new next unit can be prepared, since we could be landing in it next.
> - The previous unit can be "unprepared," we won't be landing in it anymore.
> These things need to happen simultaneously because they're visible to the user:
> - The arriving unit begins fading in (both the text AND the backgrounds for the modals should fade in, we previously had an unusual bug where the backgrounds would pop in and cover text underneath awkwardly. The ENTIRE modal unit starts fading in, not just components of it.)
> - The departing unit begins fading out.
> - The arriving *and* departing unit begin sliding in the transition direction. Because the arriving unit should already be anchored to the stub on this side, these shouldn't look like two separate units moving. (Please use the ease style and duration determined by the theme.)
> Halfway through the transition duration, the transition stub's arrow should change directions (i.e > to <).

## Current Grounding

The codebase already contains part of the required foundation:

- `src/modal.rs` already builds `SimpleModalUnitLayout` with `ModalUnitRange`
- `src/theme.rs` already exposes `modal_spacer_width`, `modal_stub_width`, `modal_stream_transition_duration_ms`, and `modal_stream_transition_easing`
- `src/ui.rs` already renders full modal units, shared stubs, and animated departure/incoming layers
- `src/app.rs` already starts transitions when focus crosses into a different unit

Important note:

- the roadmap checksum follow-up requested by the user already exists as item `21` in `roadmap.md`, so this plan does not need to add a duplicate suggestion

## Canonical Vocabulary

Use these terms consistently in code, tests, and docs:

- `modal unit` or `unit`: a group of modals visible at the same time
- `focus`: the modal with the `>` cursor and yellow highlight
- `active unit`: the visible unit containing focus
- `prepared unit`: an invisible unit positioned one spacer beyond its stub on the side it would arrive from; it is not visible and does not own hotkeys, but focus can land there immediately to begin a transition
- `transition`: the event where focus leaves the active unit and lands in a prepared unit
- `departing unit`: the previously active unit, now fading/sliding out
- `arriving unit`: the prepared unit that just received focus and is fading/sliding in
- `transition direction`: opposite the focus movement
- `previous unit`: before transition start, the prepared unit on the transition-direction side
- `next unit`: before transition start, the prepared unit in the focus direction
- `spacers`: empty separators between cards; width is `min(2% of viewport axis, theme.modal_spacer_width)`
- `stubs`: fixed-width edge cards with centered `<` or `>` arrows; width is `theme.modal_stub_width`
- `transition stub`: the stub between departing and arriving units on the focus side
- `unit bounding box`: available full-card space after reserving both stubs and required spacers

## Product Rules

### Unit formation

The unit-packing rules should be:

1. Add the first modal to the unit unconditionally.
2. If that first modal exceeds the unit bounding box, omit stubs for that unit.
3. If that first modal exceeds the full window size, clamp it to the window size and rely on scrollbars.
4. Then check following modals in order.
5. Add a modal only if the next full width plus required spacer still fits inside the current unit bounding box.
6. Stop as soon as one modal does not fit.

Implications:

- unit formation is forward-only and deterministic
- units are precomputed, not discovered ad hoc during a transition
- the focused modal must remain in the active visible unit after resize/rebuild

### When to rebuild units

Recreate the unit layout:

- on program open
- on data refresh
- on window resize

Do not rebuild unit groupings opportunistically during every keypress.

### Prepared-unit behavior

Prepared units must satisfy all of these rules:

- they are not visible to the user
- they do not respond to hotkeys while merely prepared
- they are laid out in their eventual arrival position, one spacer beyond the side stub
- they become immediately interactive as soon as focus lands in them

This matters because the user wants advanced interactions to work before the animation finishes.

### Transition behavior

When focus leaves the active unit and lands in the prepared next unit:

- the old next unit becomes the arriving unit
- the old active unit becomes the departing unit
- a new next unit is prepared immediately
- the old previous unit is discarded immediately

The arriving unit is the active unit for interaction purposes from the first frame of the transition.

### Visual rules during transition

These changes must happen together:

- the arriving unit fades in as a whole rendered unit, including text and backgrounds
- the departing unit fades out as a whole rendered unit
- both units slide in the transition direction
- the two units should read visually as one continuous strip anchored around the transition stub, not as unrelated cards passing through each other
- halfway through the theme duration, the transition stub arrow flips direction

Important regression to prevent:

- no component-level background pop-in that obscures text underneath; alpha must apply to the full rendered unit layer, not only selected child surfaces

## Design Choice

Two implementation approaches are possible.

### Option A: keep extending the current transition layering

Pros:

- smaller patch set
- reuses most of the current render pipeline

Cons:

- terminology stays implicit
- prepared/previous/next state remains inferred instead of represented
- harder to prove immediate-focus behavior and stub-arrow timing rules
- easier for future fixes to regress into ad hoc special cases

### Option B: introduce an explicit modal-unit state model

Pros:

- matches the user’s vocabulary directly
- clean separation between layout derivation, transition lifecycle, and rendering
- easier to test prepared/arriving/departing semantics directly
- better base for later polish without hidden coupling

Cons:

- larger refactor
- some existing UI transition helpers will need to be replaced instead of merely adjusted

### Recommendation

Choose Option B.

This redesign is motivated by foundational issues, not just animation tuning. A patch-on-patch approach would likely preserve the same ambiguity that caused the current prototype to become hard to reason about.

## Recommended State Model

Represent modal-unit state explicitly in app state instead of deriving it indirectly at render time.

Suggested shape:

- `ModalUnitLayout`: precomputed immutable unit ranges for the current modal sequence and viewport
- `PreparedModalUnit`: unit index plus side and resolved render placement
- `ActiveModalUnit`: current visible/interactable unit index
- `ModalUnitTransition`: departing unit index, arriving unit index, direction, start time, duration, easing, and stub-flip progress point

Practical rules:

- only one active unit exists at a time
- zero or one previous prepared unit exists
- zero or one next prepared unit exists
- zero or one transition exists
- departures may still need a short-lived rendered-history layer if a new transition begins before the previous fade-out ends

## Code Direction

### `src/modal.rs`

Keep `build_simple_modal_unit_layout()` as the packing source of truth, but revise it to align exactly with the agreed rules:

- compute spacer width from viewport percentage capped by theme
- compute unit bounding box from viewport width, stub width, and spacer width
- preserve the existing oversized-first-modal behavior
- ensure unit grouping is deterministic and fully precomputed for the whole sequence

Likely additions:

- helper for effective spacer width based on viewport axis
- helper for unit bounding box calculation so render and test code do not drift

### `src/app.rs`

Move transition ownership here more explicitly.

Responsibilities:

- store the active unit index
- store prepared previous/next unit indices
- rebuild prepared neighbors whenever layout changes
- trigger transition when focus crosses from active into prepared next/previous
- make the arriving unit immediately active for input
- create fresh prepared neighbors after transition start
- discard no-longer-valid prepared state on resize or data refresh

Likely outcome:

- `same_modal_unit_window()` becomes either simpler or unnecessary
- `start_modal_stream_transition()` becomes a unit-level transition constructor instead of a modal-snapshot comparison hook

### `src/ui.rs`

Rendering should consume explicit unit state rather than infer it from snapshot deltas.

Responsibilities:

- render the active unit at rest
- render prepared units only through transition composition, not as visible interactive UI
- render departing and arriving layers with whole-unit alpha
- keep the arriving unit anchored one spacer beyond the transition stub before animation begins
- flip the shared transition stub arrow at 50% progress

Likely cleanups:

- replace the current shared-stub mode helper with a time-aware transition-stub helper
- keep stub cards arrow-only, centered, with no nav-label text
- ensure ghost/invisible stubs are layout tools only and never look visible

### `src/theme.rs`

Theme support is mostly already present.

Implementation work should:

- keep `modal_stub_width` at `120.0` default unless the user changes it later
- keep `modal_spacer_width` at `40.0` default unless the user changes it later
- continue to use theme-driven duration and easing

No new roadmap note is needed for checksum-based rebuild skipping because that suggestion already exists.

## Execution Phases

### Phase 1. Lock the unit math

Goal:

- make unit packing match the new vocabulary and exact fit rules

Tasks:

- centralize effective spacer-width calculation
- centralize unit bounding-box calculation
- verify first-modal oversized behavior
- verify units rebuild on open, refresh, and resize while keeping the focused modal on-screen

### Phase 2. Introduce explicit prepared-unit state

Goal:

- stop inferring previous/next/arriving/departing roles from render-time comparisons

Tasks:

- add app-level state for active, previous prepared, and next prepared units
- compute prepared neighbors from the precomputed layout
- invalidate and rebuild that state when layout changes

### Phase 3. Rebuild transition lifecycle

Goal:

- make focus landing and transition start consistent

Tasks:

- trigger transition when focus crosses the active-unit boundary
- immediately switch interactive ownership to the arriving unit
- discard old previous prepared state
- prepare a fresh next unit at transition start
- preserve short-lived departure rendering if transitions chain quickly

### Phase 4. Rework rendering around whole-unit layers

Goal:

- eliminate background-pop and make the transition read as one continuous strip

Tasks:

- render whole-unit alpha on arrival and departure
- anchor arriving layout one spacer beyond the transition stub
- slide both layers in transition direction
- flip transition stub arrow at half duration
- keep non-transition rendering simple and explicit

### Phase 5. Validate and document

Goal:

- lock in behavior so future refactors do not drift

Tasks:

- add focused tests for unit math and transition timing
- document final behavior in the completed plan or follow-up mission log

## Validation Plan

### Automated checks

Add or update tests for:

- effective spacer width uses `min(2% of viewport, modal_spacer_width)`
- unit bounding box reserves both stubs and needed spacers
- first modal always enters the unit even when oversized
- oversized first modal omits stubs
- units rebuild correctly on viewport changes while keeping focus visible
- transition start reassigns active unit immediately
- previous prepared unit is dropped and next prepared unit is regenerated
- transition stub arrow flips at 50% progress
- whole-unit fade does not require child-specific alpha hacks

### Manual checks

1. Resize the window across several breakpoints and confirm the focused modal stays visible while unit boundaries change.
2. Move focus across the right unit boundary and confirm the new modal accepts arrow keys, hint keys, and search input immediately.
3. Confirm the arriving and departing units look like one continuous strip moving left or right around the shared stub.
4. Confirm the shared stub arrow flips halfway through the transition rather than at the beginning or end.
5. Confirm a very wide first modal clamps to the window and uses scrollbars rather than breaking layout.
6. Confirm prepared units are not visibly present or hotkey-active before transition start.

## Risks To Watch

- coupling interactive focus too tightly to visible alpha instead of active-unit state
- keeping both old and new transition systems alive at once for too long
- letting render-time inference drift away from the precomputed layout model
- accidentally reintroducing label-bearing stubs instead of arrow-only stubs
- missing chained-transition behavior when a new move begins before the prior departure fully fades out

## Out Of Scope

This plan does not change:

- composition-panel behavior
- collection-mode modal layout
- checksum-based refresh skipping beyond acknowledging roadmap item `21`
- broader architecture outside modal-unit packing, transition ownership, and related theme/render usage

## Notes For Future Agents

- Treat this as a redesign, not a tuning pass.
- Prefer explicit unit-role state over render-time guessing.
- Preserve the user rule that interaction moves immediately to the arriving unit.
- Do not add a duplicate checksum suggestion to `roadmap.md`; the project already tracks that follow-up.
