# Plan: Modal Stream V2 - Stubs, Motion, Chunking, Composition Panel

**Date:** 2026-04-10
**Status:** In Progress
**Supersedes:** follow-up scope decisions after `IMPLEMENTATION-BRIEF-modal-stream-ui.md`

## Purpose

Preserve the user feedback and follow-up decisions for the next modal-stream iteration so future instances do not have to reconstruct them from chat.

This document is a design-and-sequencing plan. It captures product direction, packing priorities, and recommended implementation order for:

- horizontal continuation stubs
- animated modal-stream motion
- future chunked/unit modal paging
- a top-of-window entry composition panel

## Implementation Status

Implemented through `v0.3.7-alpha`:

- Phase 1: horizontal stub packing
- Phase 2: animated horizontal stream motion with easing support
- Phase 3: display-only top composition panel
- Phase 4: field-level manual override editing with preview/composition styling

Remaining:

- Phase 5: chunked/unit modal paging

## Plain-English Summary

V1 proved that adjacent modal context is useful, but it still wastes scarce window space on smaller screens and the stream feels static when focus moves.

The next direction is:

- keep the active modal as the only interactive modal
- use smaller stub cards to show that the stream continues when full teaser cards do not fit
- add animated horizontal movement so the stream visibly slides when focus changes
- later, support chunked/unit paging so wide screens can show more forward modal context at once
- add a separate composition panel near the top of the window that shows the entry being built with richer visual meaning than the current field preview

## Relationship To V1

V1 remains the correct foundation:

- teaser state is derived from real `SearchModal` progression rules
- the active modal remains the dominant surface
- unsupported flows fail closed instead of approximating

V2 should build on that foundation rather than replacing it.

## User Intent Captured

The user wants:

- continuation in both directions to remain visible even when there is not room for full teaser cards
- smaller placeholder-style modal cards that show only the modal label and otherwise stay empty
- horizontal movement to be animated so users can understand where focus moved
- easing functions to be easy to tune, with `expo_out` and `expo_in_out` available for experimentation
- future support for chunked/unit modal windows so repeating composites can show more than previous/current/next when screen width allows
- a top composition panel that shows the full entry being assembled before it lands in the preview note
- richer per-span semantics in that composition panel so literal format text, confirmed field values, preview/default text, and manual edits are visually distinct
- eventual manual editing in the composition panel without marking the entire entry as "contaminated" when only one field span was edited

## Key Product Decisions

### 1. Stub cards have higher priority than non-active full teaser cards

Continuation matters. When space is tight, the UI should prefer showing that the stream continues rather than consuming that width on a lower-priority full teaser.

Example of what the user does **not** want:

- `[stub][past][active][NO SPACE]`

That case should instead collapse the past full teaser:

- `[stub][active][next]`
- or, if needed, `[active][next][stub]`

depending on what actually fits.

### 2. The first future full teaser has higher priority than the previous full teaser

Looking ahead matters more than re-reading a confirmed previous modal.

Priority order for horizontal simple-list flow:

1. active modal
2. continuation stubs
3. first future full teaser
4. one previous full teaser
5. additional future full teasers

This is intentionally not symmetrical.

### 3. A continuation stub is preferable to an implied dead-end

If the right side has more hidden modals, the stream should show that explicitly.

Preferred:

- `[stub][past][active][next][stub]`

Less desirable:

- `[stub][past][active][next][next][NO SPACE]`

The stub is valuable because it says "the stream continues further" instead of falsely implying the visible next teaser is the end.

### 4. Previous full teaser count should stay low

The user explicitly prefers forward context over backward context.

For horizontal simple-list flow:

- show at most one previous full teaser
- older previous states should collapse into a left stub immediately

### 5. Stub cards are placeholder-style modal cards, not mini previews

Stub cards should:

- show only the modal label
- otherwise remain empty
- use the same card shell family as preview cards
- match the active modal height in horizontal mode

Initial width rule:

- content-fit by label, clamped to roughly `96px..150px`

Future vertical stub rule:

- 1 to 2 text lines tall
- width-matched to the composition they are attached to

### 6. Motion and chunking solve different problems

This distinction should be preserved for future instances:

- motion changes **how** the stream changes
- chunking changes **what** is simultaneously visible

Recommendation:

- implement motion first
- implement chunked/unit paging after the animated stream feels correct

### 7. Use easing as a tuning surface, not a hard-coded one-off

The user specifically wants `simple-easing` available so animation curves can be tuned by feel.

Preferred first curve:

- `expo_in_out` for whole-stream movement

Useful alternate:

- `expo_out` for comparison

The implementation should keep several easing options available instead of baking in only one.

### 8. The top entry panel is a composition panel first, an editor second

The composition panel should begin as a display surface that exposes how the final entry is being assembled.

It should:

- sit above the modal stream near the top of the window
- show the full entry being built from current modal state
- visually distinguish literal format text from field-owned text
- later become the place where the user can jump in for manual edits

Recommendation:

- ship it display-only first
- add editing only after the override model is defined clearly

### 9. Manual edits should contaminate spans or fields, not whole entries

This is an important product rule and should not be lost.

If the user manually edits one field-owned span:

- that span or field becomes detached from automatic modal updates
- unrelated spans remain structured and safe
- the entire entry should **not** be marked as non-standard

Literal format text should be treated separately from placeholder-owned field spans.

Implication:

- editing literal format text should not inherently break modal ownership of structured field spans
- editing inside a field-owned span should mark that span or field as manually overridden until reset

## Recommended Phasing

### Phase 1. Horizontal stub packing

Goal:

- improve small-screen usefulness without redesigning modal progression

Scope:

- add horizontal stub cards for simple-list modal streams
- preserve the active modal as the only interactive card
- show continuation explicitly in both directions when relevant
- apply the priority order documented above

Recommended packing policy:

1. Place the active modal.
2. Reserve continuation stubs where hidden stream continuation exists or may need to be represented.
3. Upgrade the first future stub to a full teaser if it fits.
4. Upgrade one previous stub to a full teaser if it fits.
5. Add more future full teasers only when the right-continuation stub can still be preserved if more future modals remain.

Implementation note:

- derive any additional visible future teasers from real `SearchModal` state progression, not from ad hoc UI-only guesses

### Phase 2. Animated stream transitions

Goal:

- make horizontal movement legible

Scope:

- animate focus changes between modal states in the horizontal stream
- shift the entire visible stream left or right
- fade exiting/entering cards at the stream edge
- preserve current keyboard behavior

Recommended implementation shape:

- add transient stream-transition state on `App`
- capture `from` and `to` visible stream layouts
- render both during transition
- interpolate x-offset and opacity

Why this should come before chunking:

- it improves orientation without changing modal grouping policy
- it exercises the presentation seam already created by V1 snapshot helpers

### Phase 3. Top composition panel

Goal:

- expose the entry being assembled before it reaches the preview note

Display-only first pass:

- render the current entry near the top of the window
- use richer span coloring than current field preview
- keep it non-editable initially

Suggested visual semantics:

- literal format text: white
- confirmed field text: green
- default/preview/fallback text: muted
- manually edited span: italic plus distinct color

This panel should become the canonical "what will be inserted" surface.

### Phase 4. Composition-panel editing and override model

Goal:

- allow the user to jump into the entry and manually edit before committing

Required rule before implementation:

- define override ownership explicitly at span/field level

Recommended model:

- literal spans remain literal
- placeholder-owned spans remain structured until manually edited
- manual edit of a structured span detaches only that span or field
- detached spans can be reset back to structured ownership

Preview-note consequence:

- entries with any manual edit should render as visually non-standard
- but only the edited span or field should show as overridden in the composition panel

### Phase 5. Chunked/unit modal paging

Goal:

- use wide screens better
- support repeating composites and other multi-step flows with more simultaneous forward context

Definition:

- a chunk/unit is a visible run of modals that stay on screen together until focus moves off the group boundary

Desired behavior:

- on wide screens, prepare as many forward modals as fit within a target width budget
- on narrow screens, fall back to the simpler stub-and-slide model
- when moving within a chunk, focus advances across visible modals without repaging the whole view
- when moving off a chunk boundary, the old chunk slides out and the next chunk slides in

Important sequencing note:

- do not implement chunking before the animated single-stream behavior feels correct

## Technical Direction

### Snapshot architecture

Preserve the current rule:

- teaser and stub correctness must come from inspectable modal-state derivation, not duplicated UI logic

Likely additions:

- sequence helpers for multiple future simple-list snapshots
- lightweight stub-label derivation for hidden previous/next modal states

### Animation architecture

The app already has a tick/subscription loop, so motion can be layered onto current rendering without introducing a second runtime model.

Recommended transition state fields:

- direction
- started_at
- duration_ms
- easing_kind
- previous visible stream layout
- next visible stream layout

### Composition-panel data source

Prefer reusing the existing preview/span derivation already used for header modal preview state.

That existing logic should become the first data source for the display-only composition panel before any new editing state is introduced.

## Validation Plan

### Manual checks for stub packing

1. Open a simple sequential modal flow on a narrow viewport and confirm the active modal remains visible while low-priority full teasers collapse into stubs.
2. Confirm the first future full teaser is preserved before a previous full teaser when width is constrained.
3. Confirm the stream prefers showing a right-continuation stub over implying that the visible rightmost teaser is the final modal.
4. Confirm only one previous full teaser can appear; older previous states collapse into a left stub.

### Manual checks for motion

1. Move from modal 2 to modal 3 and confirm the stream visibly slides left so modal 3 becomes centered.
2. Confirm the leftmost outgoing card fades/de-emphasizes while the incoming right-side card fades in.
3. Confirm keyboard behavior and focus ownership remain unchanged during and after the transition.

### Manual checks for composition panel

1. Open a formatted multi-list field and confirm the top panel shows the full assembled entry.
2. Confirm literal format text, confirmed values, and fallback/default text use distinct visual semantics.
3. In the later editable phase, manually alter a single field span and confirm only that span/field becomes overridden.

## Recommended Execution Order

1. Implement horizontal stub packing and continuation rules.
2. Implement animated horizontal stream transitions using tunable easing.
3. Add a display-only top composition panel driven by existing preview/span derivation.
4. Define and implement span/field override ownership for manual editing.
5. Explore chunked/unit modal paging after the animated stream feels stable in practice.

## Notes For Future Agents

- Do not let the distinction between motion and chunking collapse into one vague "improve the stream" task.
- Preserve the priority order exactly unless the user changes it explicitly.
- Stubs are not fake mini-previews; they are continuation indicators.
- The top entry panel is meant to become an editing surface later, but only after override ownership is designed.
- Avoid whole-entry contamination rules when the user has explicitly asked for span/field-level contamination instead.
