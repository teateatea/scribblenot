# Implementation Brief: Modal Stream UI

**Date:** 2026-04-10
**Status:** Implemented V1 prototype; V2 follow-up now fully implemented through `v0.3.8-alpha`

## Purpose

Capture the agreed direction for a new modal presentation model where the current modal remains the primary interaction surface, but nearby modal states become visible as non-interactive teaser cards.

This brief is planning only. It does not authorize broad UI changes beyond the scoped prototype described below.

## Plain-English Summary

The current modal flow works, but it hides context. The user moves through a sequence of modal states one at a time and can navigate left and right, yet only the active modal is visible.

The new direction is:

- keep the current centered active modal as the main interaction
- add always-visible neighboring modal cards as read-only teasers
- use horizontal teaser cards for ordinary modal progression
- use a vertical stream of collection preview cards for collection mode

The goal is to make modal navigation feel like a visible ribbon, stream, or river instead of a sequence the user must hold in memory.

## User Intent Captured

The user wants:

- the current behavior to remain the main interaction
- previous and next modal context to stay visible horizontally
- horizontal teasers to be intentionally narrow and allowed to cut off text
- horizontal teasers to be accurate representations of what the user will actually see next or previous
- a similar simultaneous-view pattern vertically for collection previews
- vertical collection preview cards to keep full width rather than being truncated
- an initial low-risk prototype, not a broad rewrite

## Recommendation

Treat teaser cards as real modal cards rendered in a constrained read-only mode, not as fake previews.

Why:

- it keeps teaser content accurate
- it reuses the same modal rendering rules as the active panel
- it avoids UI drift between active and preview states
- it reduces the chance that future modal logic changes break teaser correctness

## Scope

### In Scope

- one active center modal, unchanged in principle
- one previous and one next horizontal teaser card for simple sequential list progression
- one previous and one next vertical collection preview card around the current collection preview
- shared rendering primitives for active and teaser cards
- viewport-aware hiding rules for teaser cards
- visual distinction between active and teaser cards

### Out of Scope For V1

- nested field teaser support
- branch-flow teaser support
- horizontal teaser support for collection modals
- teaser card interaction, focus, hover, or click behavior
- keyboard navigation changes
- theme-system expansion unless required for minimal readability

## Constraints

- The active modal must remain the only interactive modal card.
- Teaser cards must not accept focus or imply clickability.
- Horizontal teaser cards should use the same title/search-strip structure as the real modal, but the search area must be inert.
- Vertical collection teaser cards should preserve full content width.
- Teasers should disappear cleanly on cramped viewports rather than forcing a crowded layout.
- Horizontal teaser previous/next state must be derived via non-mutating `SearchModal` inspection helpers, not from proactive snapshots or reconstructed replay.

## Existing Code Context

Current modal rendering is centralized in:

- `src/ui.rs`
  - `modal_overlay`
  - `collection_modal_split_panes`
  - `collection_modal_preview`
  - `modal_dimensions_for_content`

Current modal state/progression is centralized in:

- `src/modal.rs`
  - `SearchModal`
  - `FieldAdvance`
  - `advance_field`
  - `go_back_one_step`
  - `preview_collection`
  - `preview_field_value`

Current modal key handling is centralized in:

- `src/app.rs`
  - `handle_modal_key`
  - `handle_collection_modal_key`
  - `composite_go_back`
  - `update_collection_modal_preview`

This means teaser correctness should come from modal-state derivation, not from separate ad hoc UI text generation.

## Design Decisions

### 1. Keep the active modal unchanged as the primary UI

The current centered modal interaction is working and should remain the dominant surface.

The stream UI is additive:

- active center card remains interactive
- teaser cards provide context only

### 2. Vertical collection stream first

This is the safer first implementation because collection mode already has an explicit preview pane and explicit current selection.

Initial target:

- top teaser card: previous collection preview
- center card: current collection preview
- bottom teaser card: next collection preview

Only the center card is interactive.

The stream axis is collection adjacency, but the center preview must preserve current behavior.

- when focus is in `CollectionFocus::Collections`, the center card is derived from `collection_cursor`
- when focus is in `CollectionFocus::Items(collection_idx)`, the center card is derived from that focused collection index, matching the current `preview_collection()` behavior
- the neighboring teaser cards are the previous and next collection previews around that same center collection index
- this stream does not switch to neighboring items by `item_cursor`

### 3. Horizontal teasers use real modal-card rendering in read-only mode

The horizontal teaser should look like a real modal card compressed into a narrow frame.

It should include:

- title
- search strip chrome
- visible row list

It should not include:

- active caret
- hover behavior
- click behavior
- focus border that competes with the active modal

### 4. Horizontal V1 supports only simple sequential list progression

The first prototype should explicitly stop at simple list progression because nested and branching modal flows increase the state-derivation risk substantially.

Supported:

- previous list within a straightforward list sequence
- next list within a straightforward list sequence

Not yet supported:

- nested subfields
- branch stack transitions
- collection modal flow

### 5. Horizontal previous/next state derived via backward-inspectable SearchModal

Rather than capturing state snapshots proactively on each navigation, previous and next list states for horizontal teasers should be derived by making `SearchModal` inspectable in a read-only way.

This means adding non-mutating helpers on `SearchModal` that report whether the current modal is in a supported simple sequential list state, and if so, what the previous or next list view would be without mutating any state.

Why:

- going back one step is already a common user action; `go_back_one_step` already encodes the logic for what "previous" means
- a snapshot approach would duplicate state and create a second source of truth
- the real navigation boundary already lives on `SearchModal`, not `FieldModal`, because branch, nested, and collection state also determine whether a teaser is valid
- a non-mutating inspect API is a clean read-only contract that fits the teaser use case

Accessors needed:

- `peek_prev_list_view()` on `SearchModal` - returns a fully derived preview of the previous simple-list modal view without mutation, or `None` at the root or in unsupported flows
- `peek_next_list_view()` on `SearchModal` - returns a fully derived preview of the next simple-list modal view without mutation, or `None` if unsupported or at the final step

Both should return `None` for unsupported flows rather than approximating.

### 6. Width rules should be visual, not literal character counts

The user initially estimated about 40 characters for horizontal teaser width. That is a useful intuition, but the implementation should use card width constraints rather than a fixed character count.

Starting point:

- horizontal teaser width: roughly `280px` to `340px`
- tune by real use after rendering

This keeps layout decisions resilient to fonts, hint columns, and spacing.

## Proposed Architecture

### Step 1. Extract reusable modal-card rendering

Refactor `src/ui.rs` so the active overlay and teaser cards share the same rendering primitives.

Target extraction points:

- modal card shell/chrome
- simple-list card body
- collection preview card body
- read-only vs interactive render mode

Suggested shape:

- `ModalRenderMode::Interactive`
- `ModalRenderMode::Preview`

The preview mode should:

- suppress interactive handlers
- mute styling
- allow clipped/truncated content
- render inert search-strip chrome

### Step 2. Add collection preview snapshot helpers

Collection previews are already explicit enough that snapshot generation may be simple.

Add a helper that can derive:

- current collection preview card
- previous collection preview card if one exists
- next collection preview card if one exists

This should be derived from `collection_cursor` in `CollectionState`, indexing into the `collections` vec for the entry at `cursor - 1` and `cursor + 1`. `preview_collection()` only returns the current entry; a new helper (e.g., `collection_preview_neighbors()`) should return the full `(prev, current, next)` triple. This does not require new state - only direct vec access.

### Step 3. Add simple modal snapshot helpers for horizontal teasers

For ordinary modal flow, introduce a lightweight read-only snapshot representation for teaser rendering.

Suggested snapshot contents:

- modal title or part label
- search query string to display
- fully derived visible row window for the list
- highlighted row index
- visible hint labels for that row window
- whether previous/next snapshot exists

The snapshot should be derived from real modal progression rules and the real visible-window rules for simple list flow only.

The first prototype can intentionally avoid perfect support for complex flows by returning no teaser snapshot in unsupported cases.

State derivation approach: add `peek_prev_list_view()` and `peek_next_list_view()` non-mutating helpers on `SearchModal` (see Design Decision #5). These helpers drive snapshot construction without touching live modal state and should include the same derived row-window information the active modal uses. Both return `None` in branching, nested, collection, or endpoint cases.

### Step 4. Compose stream layouts around the existing active modal

Vertical collection layout:

- three stacked preview cards inside the right-side collection preview area, or a dedicated preview stack container if cleaner

Horizontal ordinary-modal layout:

- left teaser card
- center active modal card
- right teaser card

The center card remains visually dominant.

## UI Rules

### Active Card

- current sizing logic remains the baseline
- current focus styling remains
- full interaction remains

### Horizontal Teaser Cards

- narrow width
- same modal title/search-strip structure
- list content may clip or truncate
- dimmer border/background/text
- no focus ring
- no mouse interaction
- no keyboard interaction

### Vertical Collection Teaser Cards

- full width
- full preview text width
- stacked above/below current card
- dimmer than current card
- no interaction

### Viewport Rules

Hide teasers instead of over-compressing the active modal.

Suggested initial policy:

- hide horizontal teasers below a safe minimum width
- hide vertical neighboring preview cards below a safe minimum height
- if only one teaser can fit comfortably, prefer symmetry and hide both rather than showing only one side

Exact breakpoints should be tuned after visual testing.

## File-Level Work Plan

### `src/ui.rs`

- extract reusable modal card rendering primitives
- add render mode support for active vs preview cards
- implement vertical collection stream layout
- implement horizontal teaser layout for simple list modal flow
- add viewport guards and width/height rules

### `src/modal.rs`

- add snapshot helpers for collection preview neighbors
- add `SearchModal` snapshot helpers for simple-list previous/next modal states
- explicitly return no teaser snapshots for unsupported flows in V1

### `src/app.rs`

- likely minimal changes if teaser rendering stays derived from read-only modal state
- only add wiring if the UI needs small helper accessors from `App`

### `roadmap.md`

- keep a roadmap entry pointing to this work until implemented

## Risks

### 1. Preview drift

If teaser cards are built from copied UI logic instead of real modal-state derivation, they will become inaccurate over time.

Mitigation:

- derive teaser snapshots from the same modal progression model used by the active modal

### 2. Layout crowding

Horizontal teaser cards may make the modal overlay feel cramped on smaller screens.

Mitigation:

- add aggressive viewport guards
- keep the center modal visually dominant
- prefer hiding teasers over shrinking the active card too far

### 3. Focus ambiguity

If teaser cards look too similar to the active card, users may assume they are interactive.

Mitigation:

- use clear visual de-emphasis
- remove hover/click affordances
- keep only the center card with active focus styling

### 4. Prototype scope creep

Supporting nested and branch flows in the first pass could turn a manageable UI iteration into a modal-state refactor.

Mitigation:

- explicitly keep horizontal V1 to simple list progression

## Validation Plan

### Manual checks

1. Open a collection modal with preview support and confirm previous/current/next collection previews are visible vertically when viewport height allows.
2. Confirm only the center collection preview remains interactive.
3. Navigate up and down in collection mode and confirm the preview stream updates without focus confusion.
4. Open a simple sequential list modal and confirm previous/current/next cards render horizontally when viewport width allows.
5. Confirm horizontal teaser cards visually match the real modal structure but do not accept interaction.
6. Confirm teaser cards disappear cleanly on constrained viewport sizes.
7. Confirm the active modal’s existing keyboard behavior is unchanged.

### Explicit non-goals to verify

- nested modal flows may show no horizontal teaser cards in V1
- branch flows may show no horizontal teaser cards in V1
- collection modals do not need horizontal teaser cards in V1

## Acceptance Criteria For V1

- The active modal remains the primary and only interactive modal card.
- Collection mode can show previous/current/next preview cards vertically.
- Simple sequential list modals can show previous/current/next cards horizontally.
- Horizontal teaser cards use real modal-card rendering in read-only mode.
- Teaser cards hide on small viewports rather than degrading the active modal excessively.
- Unsupported modal flows fail gracefully by omitting teaser cards instead of rendering misleading previews.

## Post-Implementation Notes

### What shipped in V1

- `src/modal.rs` now exposes read-only snapshot helpers for:
  - simple-list teaser views via `peek_prev_list_view()` and `peek_next_list_view()`
  - collection preview triples via `collection_preview_neighbors()`
- `src/ui.rs` now renders:
  - horizontal previous/current/next modal cards for supported simple sequential flows
  - vertical previous/current/next collection preview cards inside the collection preview pane
- unsupported flows currently omit horizontal teaser cards instead of approximating
- automated tests cover the new snapshot helpers and full test suite passed during implementation

### What worked well in practice

- the read-only `SearchModal` snapshot approach was the right call; it kept teaser correctness tied to real modal state instead of duplicating presentation logic
- collection neighbors were straightforward once preview ownership was defined around the focused collection index rather than `item_cursor`
- viewport guardrails were worth doing early because the active card still needs to dominate visually

### Where the real implementation differed from the planning brief

- the active simple-list modal was kept on its existing interactive rendering path, while preview cards use a lighter shared card shell plus preview-specific content builders
- the horizontal teaser helpers currently fail closed not only for nested, branch, and collection flows, but also for filtered/search-active states; this was the safest V1 boundary because "next" becomes ambiguous once the user is mid-filter
- the vertical collection stream uses stacked cards inside the existing right preview pane rather than a broader modal-wide vertical stream layout

### Practical limits discovered during implementation

- preview rendering in `iced` is simpler and safer when preview card builders own their snapshot payloads instead of borrowing short-lived local state
- "next list" means the next actual modal state after confirming the current selection, not merely "the current list with another highlighted row"; this matters for tests and for future UX decisions
- the current visual distinction is good enough for a prototype, but spacing, contrast, and teaser prominence still need live tuning by feel rather than more planning

## Recommended Follow-Up Work

### Highest-value next steps

1. Run a manual visual pass and tune teaser breakpoints, widths, spacing, and de-emphasis from real usage.
2. Decide whether filtered/search-active states should eventually support horizontal teasers or intentionally remain teaser-free.
3. Decide whether preview cards should be made fully inert to mouse clicks instead of sharing the current panel press behavior.
4. If the user likes the pattern, consider extracting more shared modal card primitives so active and preview paths converge further without forcing a risky rewrite.

### Explicit questions for the next instance

- Is the current horizontal teaser width too wide, too narrow, or about right in the user’s most common viewport?
- Should collection preview neighbors remain hidden below the current height guard, or would a denser compact mode be preferable?
- Does the user want teaser support while typing in the search box, or is omitting teasers during filtering actually clearer?

## Recommended Execution Order

1. Extract shared modal-card rendering in `src/ui.rs`.
2. Implement vertical collection stream.
3. Add viewport guardrails.
4. Implement horizontal simple-list teaser snapshots.
5. Render horizontal teaser cards.
6. Tune width, spacing, clipping, and visual contrast after real use.

## Notes For Future Agents

- Do not start by inventing a second preview-only UI path.
- First make the active modal and teaser cards share rendering primitives.
- Preserve the current interaction model unless the user explicitly asks to redesign it.
- Prefer omitting a teaser in unsupported cases over showing an inaccurate teaser.
- The current V1 prototype already exists. Next work should start with visual tuning and scope decisions, not by redoing the snapshot architecture.
- The agreed V2 follow-up direction now lives in `operations/plans/PLAN-modal-stream-v2-stubs-motion-composition.md`. Read that before making further modal-stream changes so the stub-priority rules, motion-vs-chunking distinction, and composition-panel direction are not lost.

## V2 Completion Note

The follow-up work described in `PLAN-modal-stream-v2-stubs-motion-composition.md` is now implemented through `v0.3.8-alpha`.

Useful checkpoints:

- `v0.3.5-alpha` - animated stream transitions
- `v0.3.6-alpha` - top entry composition panel
- `v0.3.7-alpha` - field-level composition overrides
- `v0.3.8-alpha` - chunked/unit modal paging

If a future instance is trying to understand the current modal-stream system, the V2 plan is now the more important source of truth than this original V1 brief.
