# Report: Task #33 Modal Active-Row Width Investigation

**Date:** 2026-04-20
**Branch:** `modal-correctness`
**Status:** In progress

## Goal

Investigate why the active modal row border renders narrower than the inactive modal rows
and the search-bar width in the active simple-list modal.

## User-visible symptom

The active row in the center modal still appears inset/narrower than the surrounding modal
content, even after earlier fixes. The user supplied a screenshot on 2026-04-20 showing the
active row in the `Appointment` field (`Month` modal) still not matching the expected width.

## Findings so far

1. The first attempt removed the plain `button(...)` wrapper around active rows and switched
   the active row interaction path to `mouse_area(...)`.
   Result: this did not resolve the visible width mismatch.

2. The remaining likely cause is not the border style itself, but the width negotiation chain
   inside the active modal's scrollable list:
   - `active_simple_modal_content(...)`
   - the `column(list_items)` passed into the themed `scrollable`
   - the interactive row wrapper around each active row

3. The preview rows do not suffer from this because they are rendered as plain containers
   without the active interaction wrapper and without the active scrollable layout path.

4. The screenshot still suggests the row border box is being sized from intrinsic content width
   somewhere inside the scrollable subtree, rather than from the modal panel width.

## Latest attempt on this branch

The current branch now adds one more explicit width-forcing pass:

- each active interactive row is wrapped in an outer `container(...).width(Length::Fill)`
- the scrollable content is wrapped in `container(column(...).width(Length::Fill)).width(Length::Fill)`

Intent:

- force the active row subtree to negotiate against the modal column width rather than the
  row's intrinsic content width
- eliminate one more place where the interactive wrapper might be shrinking the layout

## Scope note

This report documents investigation only. If the latest width-forcing pass still does not fix
the visual mismatch, task `#33` is a reasonable candidate to drop from `modal-correctness`
and revisit later with a deeper `iced` layout audit or a purpose-built visual regression seam.

## Files touched during investigation

- `src/ui/mod.rs`
- user screenshot reviewed: `Screenshot 2026-04-20 135009.png`
