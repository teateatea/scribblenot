# Plan: Collection Modal Interaction Contract (#42)

**Date:** 2026-04-21
**Status:** Implemented in code on 2026-04-22
**Scope:** Product contract and interaction rules
**Related roadmap items:** `#42`, `#43`, `#28`, `#36`

## Purpose

Define what collection modals mean before more modal-edge semantics or preview/layout machinery are built on top of them.

The goal is to stop treating collection mode as an awkward special case and instead give it a stable product contract that later rendering and animation work can follow.

## Plain-English Summary

Collections are meant to behave like configurable checkbox or radio-button menus inside the modal workflow.

In the current product, that mostly means treatment areas and the technique sets nested under those areas:

- choose one or more treatment areas
- optionally dive into an area's technique list
- toggle the specific techniques actually used
- move on without the cursor pretending to mean intent

This plan intentionally keeps collection mode separate from chained conditional sub-fields in `#36`.

- collection mode is a menu-plus-preview interaction with optional drill-in
- chained sub-fields are a call-stack modal flow that replaces the parent strip while composing

Those are different interaction families and should not be merged prematurely.

## Current Product Grounding

Real authored collection usage is currently concentrated in `data/treatment.yml`, referenced by the treatment group in `data/sections.yml`.

Today collections already support:

- multi-select activation
- `max_actives`, including `max_actives: 1` radio-style behavior
- FIFO eviction of the oldest active collection when the limit is exceeded
- nested item toggles inside a focused collection

The missing part is not raw state capability. The missing part is a clear contract for:

- which key means what
- when a collection counts as intentionally active
- what confirm and advance should use as truth
- what reopening should restore

## Canonical Product Rules

### 1. What a collection is

A collection is a selectable menu entry that may also own a more specific inner item list.

Examples:

- a treatment area collection behaves like a checkbox or radio option
- the area's technique list behaves like the inner options for that chosen area

The system should support:

- many active collections when `max_actives` is unset or greater than `1`
- radio-style behavior when `max_actives: 1`

### 2. Confirm toggles the focused thing

`Confirm` acts on the currently focused row, not on preview content and not on cursor implication outside the focused list.

Rules:

- on the collection list, `Confirm` toggles the focused collection on or off
- on the item list, `Confirm` toggles the focused item on or off

### 3. Toggling an item implies intent to use that collection

If the user toggles an item inside a collection that is currently inactive, that collection should automatically become active first.

Why:

- item-level editing is stronger intent than mere cursor placement
- if the user is modifying a collection's inner list, they are almost certainly trying to use that collection

If auto-activation would exceed `max_actives`, normal collection eviction rules should still apply.

### 4. `Select` switches sides

`Select` is the side-switch key, not the toggle key.

This means the resolved `Select` keybinding, not a hardcoded physical key. If the default binding happens to be Space today, that is a configuration detail rather than the product rule.

Rules:

- on the collection list, `Select` dives into the focused collection's item list
- on the item list, `Select` returns to the collection list

This should feel analogous to how other modal modes let the user move into a more specific editing surface without turning simple navigation into commitment.

### 5. Directional navigation

Directional meaning should be:

- `nav_up` / `nav_down`: move within the currently focused list
- `nav_left` from item list: return to the collection list
- `nav_left` from collection list: go to the previous modal, or return to the wizard if this is the first modal
- `nav_right` from collection list: move into the item/preview pane for the focused collection
- `nav_right` from item list: advance to the next modal without using cursor location as implicit confirmation

If the current modal is the last modal in the field:

- `nav_right` from the item list should confirm the field using the actual toggled collection/item state only

No cursor-fallback behavior should exist for collections.

### 6. Super-confirm

`Super-Confirm` should commit the current toggled state and advance immediately.

Rules:

- if another modal follows, advance to it
- if this is the last modal, confirm and close the field

Like normal terminal confirm, super-confirm uses only the toggled state, never cursor fallback.

### 7. Reopen behavior

Reopening a collection field should restore the full prior state as closely as possible.

That includes:

- which collections were active
- which inner items were enabled
- which side had focus
- which collection row was focused
- which item row was focused

The intended feel is "right where I left it."

### 8. Preview role

The collection preview is not the primary control surface.

Its job is:

- passive verification at a glance
- live reflection of the currently focused collection
- visual context for nearby collections when space allows

Direct manipulation remains owned by the collection list and item list. Preview behavior, sizing, motion, and strip layout are follow-on design work in `#43`.

## Explicit Non-Goals

This contract does not define:

- generic non-simple modal unit semantics for all modal types
- the final shared semantic authority layer for modal edge stubs (`#28`)
- chained sub-field behavior from `#36`
- third-level "collection of collection of collection" behavior

If deeper chained follow-up input is needed later, that should remain a `#36`-style call-stack modal concern unless a real collection use case proves otherwise.

## Code Areas Likely To Consume This Contract

- `src/sections/collection.rs`
- `src/modal.rs`
- `src/app.rs`
- `src/ui/mod.rs`

## Implementation Status In Current Code

- `src/sections/collection.rs::toggle_current_item()` auto-activates an inactive parent collection before toggling the item, and still honors `max_actives` plus FIFO eviction.
- `src/app.rs::handle_collection_modal_key()` routes side-switching through the resolved `Select` keybinding, also lets `nav_right` enter the item/preview pane from the collection list, uses `nav_right` from the item pane for advance/commit, and delegates `nav_left` on the collection list into the normal modal back/exit path instead of silently no-oping.
- `CollectionFieldValue` persists activation order, focused collection, focused item, and side ownership so reopening can restore the prior collection-modal state closely.
- Focused regressions now cover configured `Select` routing, collection-list `nav_left` exit, collection-list `nav_right` entry into the item/preview pane, cursor-free `nav_right` confirmation from the item pane, auto-activation, FIFO eviction, and reopen restoration.

## Validation Snapshot

- `cargo test --quiet collection_`
- Contract-specific regressions in `src/app.rs`, `src/modal.rs`, and `src/sections/collection.rs`

## Validation Expectations

When this contract is implemented and enforced, validation should cover:

1. `Confirm` toggles collection rows and item rows distinctly.
2. Toggling an item in an inactive collection auto-activates that collection.
3. Auto-activation respects `max_actives` and FIFO eviction.
4. `Select` switches between collection list and item list without toggling.
5. `nav_right` enters the item/preview pane from the collection list, then advances or terminal-confirms from the item pane using toggled state only.
6. Reopening restores toggles plus focus position.

## Recommendation To Follow-On Work

Use this plan as the product source of truth for collection modal meaning.

Then implement `#43` as the UI/layout realization of this contract before resuming `#28`.
