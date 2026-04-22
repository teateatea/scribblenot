# Plan: Collection Preview Panel And Vertical Strip Behavior (#43)

**Date:** 2026-04-21
**Status:** Proposed from product discussion
**Scope:** Collection preview layout, motion, and focus behavior
**Depends on:** `operations/plans/PLAN-42-collection-modal-contract.md`
**Related roadmap items:** `#43`, `#42`, `#28`, `#36`

## Purpose

Redesign the collection preview/panel behavior so it supports the intended treatment workflow cleanly and becomes a reliable foundation for later non-simple modal work.

This plan is about how collection mode should look and move, not about redefining what collection actions mean. Action semantics come from `#42`.

## Plain-English Summary

The collection preview should behave like a vertical strip whose centered preview follows the focused collection.

Priority order:

1. give the focused collection enough height to show its full item list when possible
2. if it still needs scrolling, let it consume the full available preview height
3. only after that, use any leftover height for neighboring previews

Neighbors are supporting context, not co-equal panels.

They should:

- stay in authored order
- keep their own intrinsic heights
- run off-screen naturally when there is not enough room
- move vertically as one strip rather than fading or resizing in and out

## Product Goals

- make it easy to verify "this collection contains what I think it does"
- keep the focused collection visually dominant
- preserve stable, predictable motion when focus changes
- keep preview as a secondary surface rather than a second competing control plane

## Current Problem

Today the collection preview is too rough to act as a solid base for later modal work.

Known issues:

- preview height is effectively capped instead of being driven by focused content needs
- focused content can show only part of its list even when more screen space exists
- preview ownership and focus styling are not yet aligned with the intended interaction model
- the current implementation still depends on fixed gates and small-shape helpers that assume "current plus maybe one neighbor" rather than a real ordered strip

The result is a UI that is only barely serving the intended "double check at a glance" function.

## Canonical Layout Rules

### 1. Focused preview gets first claim on height

The focused collection preview should be sized before neighbors are considered.

Rules:

- if all focused items fit within the available preview height, show them all
- if they do not fit, let the focused preview take the full available preview height and become internally scrollable
- only leftover height, if any, may be used for neighbors

### 2. Vertical bounding-box rule

For style consistency with horizontal modal-unit math, the preview lane should reserve edge chrome similarly when space allows.

Starting rule:

- available preview height = `viewport_height - 2 * (stub_width + spacer_width)`

However, unlike the horizontal unit layout, this rule should be soft rather than absolute:

- if reserving that full edge space would make the focused preview unusably cramped, reduce or drop the reservation before shrinking the focused preview further

This keeps the visual language familiar without making the focused preview worse just to preserve decorative symmetry.

### 3. Neighbor allocation

Neighbors are optional context only.

Rules:

- neighbors keep their natural authored order
- leftover height is allocated outward from the focused preview
- neighbors may be partially visible and may run off-screen
- neighbors should not fade in/out as a primary sizing mechanism
- neighbors should not repeatedly resize while the focused collection changes

### 4. Strip behavior

The preview lane is a single vertical strip.

Rules:

- when focus moves down the collection list, the preview strip slides up
- when focus moves up the collection list, the preview strip slides down
- the goal is to keep the focused preview centered when possible
- all previews keep their own already-determined heights during motion

This mirrors the mental model of the horizontal modal strip without forcing collection mode to pretend it is the same layout primitive.

## Focus And Drill-In Behavior

### 1. Collection-list mode

When the collection list owns focus:

- the collection list gets active styling
- the active hint labels belong to the collection list
- the focused collection's preview is centered/prioritized in the preview lane

### 2. Item-list mode

When the item list owns focus:

- the item list becomes the active control surface
- the cursor highlight moves to the item list
- the collection-list side shows a non-active but still meaningful landing state
- active hint labels update to the item list's visible rows

The user's description of this state is important:

- active style moves
- the yellow cursor highlight hops over
- the departure cursor location becomes a non-active landing indicator

### 3. `Select` is a focus transfer, not a commit

`Select` should feel like moving between related panes, not like confirming anything.

The main visual effect is therefore:

- focus ownership changes
- styling changes
- hint ownership changes
- the preview lane remains attached to the same collection context unless focus movement itself changes the focused collection

## Interaction Rules This UI Must Respect

This UI plan must honor the `#42` contract:

- `Confirm` toggles the focused collection or item
- toggling an item in an inactive collection auto-activates that collection
- `Select` switches between collection list and item list
- `nav_left` backs out
- `nav_right` from the collection list enters the item/preview pane
- `nav_right` from the item/preview pane advances without using cursor fallback
- final confirm/super-confirm use only toggled state

## Separation From `#36`

Keep this design separate from chained conditional sub-fields in `#36`.

Why:

- collections keep a persistent preview relationship beside the active list
- `#36` intentionally wants a call-stack replacement flow that slides the parent strip away

Both may use vertical motion, but they should not share one product model by accident.

## Expected Implementation Areas

- `src/ui/mod.rs`
- `src/modal.rs`
- `src/app.rs`
- possibly a new collection-preview layout helper if the current UI module becomes too crowded

## Suggested Implementation Phases

### Phase 1: Focused-preview sizing model

- replace the fixed-height collection preview assumptions
- relax or replace the current fixed `COLLECTION_MODAL_MAX_HEIGHT` ceiling so focused preview height can be derived from the new vertical bounding-box rule rather than a hard 460px cap
- compute focused preview height first
- add the soft vertical edge-reservation rule
- wire the focus-transfer state machine early enough that later strip work is built on the real collection-list versus item-list ownership model, not on placeholder assumptions

### Phase 2: Vertical strip data model

- replace the current binary neighbor gate (`collection_neighbor_previews_supported()`) with strip-driven sizing and visibility rules
- replace the current fixed prev/current/next preview shape with an ordered strip model that can represent all authored collections plus the focused index
- represent focused and neighboring previews as one ordered vertical strip
- preserve intrinsic preview heights
- allow off-screen overflow rather than trying to fully pack every preview

### Phase 3: Motion and centering

- slide the strip vertically as focus changes
- center the focused preview when possible
- avoid neighbor fade/resize churn during normal navigation

### Phase 4: Focus-transfer styling

- make `Select` visibly transfer active ownership between collection list and item list
- update cursor, landing-state, and hint-label styling accordingly

### Phase 5: Validation

- verify focused preview prioritization
- verify off-screen neighbor behavior
- verify vertical strip centering/motion
- verify item-toggle auto-activation of inactive collections
- verify reopen/restore behavior still lands in the right state

## Manual Verification

1. Open a treatment field with collections and confirm the focused preview gets enough height to show all items when possible.
2. Confirm that a long focused preview consumes the full available preview height before neighbors steal space.
3. Move through the collection list and verify the preview strip slides vertically while preserving authored order.
4. Confirm partially visible neighbors run off-screen naturally rather than fading or resizing.
5. Press `Select` and verify active styling, cursor ownership, and hint ownership move between the collection list and item list cleanly.
6. Toggle an item inside an inactive collection and verify the parent collection auto-activates, including FIFO eviction when `max_actives` requires it.

## Risks

- Iced layout may make vertical strip clipping/centering awkward in the same way the early horizontal strip work was awkward
- mixing preview-lane sizing and focus-transfer styling in one pass could obscure regressions
- if this implementation quietly drifts into generic non-simple modal semantics, it will overreach the intended scope

## Implementation Notes From Current Code

- `src/ui/mod.rs` currently clamps collection modal height through `COLLECTION_MODAL_MIN_HEIGHT` / `COLLECTION_MODAL_MAX_HEIGHT`; Phase 1 should treat that as implementation debt to remove or redefine rather than as a stable product rule.
- `src/ui/mod.rs::collection_neighbor_previews_supported()` is the current soft neighbor gate. The new strip model should replace it instead of layering more heuristics on top.
- `src/modal.rs::CollectionPreviewNeighbors` is currently a three-slot shape (`previous/current/next`). A real vertical strip likely needs an ordered preview list with the focused index tracked separately.
- `src/app.rs::handle_collection_modal_key()` currently routes pane transfer through nav-left/nav-right and `Space`. If `Select` is the intended side-switch contract, implementers should confirm the real keybinding route early and move the state-machine ownership into the collection handler before polishing visuals.

## Fallback If Iced Fights Centering

Preferred implementation:

- a manually positioned/clipped vertical strip whose offset keeps the focused preview centered when possible

Fallback:

- a `Scrollable` strip container with programmatic scroll positioning used to center or near-center the focused preview

The fallback should be chosen only if manual padding/offset control proves too fragile, but naming it now avoids redesigning under pressure mid-implementation.

## Exit Criteria

This plan is complete when:

- the focused collection preview always has first priority for height
- neighbors are contextual and can run off-screen
- preview order matches collection order
- preview motion is vertical strip motion, not fade-based churn
- `Select` clearly transfers focus ownership between the two sides
- the collection preview is solid enough to serve as the prerequisite foundation for later modal-semantic work
