# Plan: Modal Lifecycle Transitions for Open, Exit, and Confirm

**Date:** 2026-04-17
**Status:** Ready for implementation handoff
**Prerequisite:** Existing unit-to-unit modal transitions remain the base implementation

---

## Purpose

Extend the current modal unit transition system so it also animates the modal lifecycle boundary:

- opening a modal from the wizard
- exiting a modal back to the wizard via the `-` edge semantics
- confirming a modal back to the wizard via the `+` edge semantics

The goal is visual consistency with the existing unit transition language:

- opening uses rightward navigation semantics, so the arriving unit starts off to the right and travels left into place while fading in
- exiting uses leftward navigation semantics, so the current unit travels right and fades out
- confirming uses rightward navigation semantics, so the current unit travels left and fades out

This applies to all equivalent user actions, not only mouse clicks on visible stub cards.

---

## Clarified Motion Semantics

### Open

- Semantic direction: `Right`
- Visual strip motion: leftward
- Visible result: the first modal unit starts shifted to the right, then slides left into its resting position while fading from `0.0 -> 1.0`

This is an arrival-only transition. There is no visible departing modal unit.

### Exit

- Semantic direction: `Left`
- Visual strip motion: rightward
- Visible result: the current modal unit starts at rest, then slides right while fading from `1.0 -> 0.0`

This is a departure-only transition. After the animation completes, the wizard remains visible with no modal overlay.

### Confirm

- Semantic direction: `Right`
- Visual strip motion: leftward
- Visible result: the current modal unit starts at rest, then slides left while fading from `1.0 -> 0.0`

This is also a departure-only transition.

---

## Scope Rules

### Included

- Header-field modal opening from the wizard
- Exit-equivalent modal dismissal paths
- Confirm-equivalent modal completion paths
- Keyboard, mouse, and semantic action routes that mean the same thing

### Excluded for first pass

- Collection modals
- Non-simple modal states (`query` active, branch/nested flows, other non-simple-list-teaser states)
- Adaptive queue/tuning behavior from Part 3

If the modal is not in simple-unit mode, lifecycle entry/exit remains instantaneous.

---

## Behavioral Contract

Animate all equivalent semantic actions, not just stub clicks.

### Open-equivalent actions

- Wizard `nav_right` opening a modal-backed field
- Wizard `Enter` opening the same modal-backed field
- Any future helper that routes through `open_header_modal()`

### Exit-equivalent actions

- `Esc`
- Backdrop dismissal
- Back/navigation from the first modal list when that action leaves the field session
- Any explicit exit-stub click handling when present

### Confirm-equivalent actions

- Confirming the final modal step via `Enter`
- Confirming via `nav_right` when that action completes the field
- Super-confirm when it completes the field
- Any explicit confirm-stub click handling when present

The deciding factor is modal-session meaning, not which physical key or click path invoked it.

---

## State Model Changes

Extend `ModalTransitionLayer` with explicit lifecycle variants instead of inventing dummy modal units.

```rust
pub enum ModalTransitionLayer {
    ConnectedTransition {
        arrival: ModalArrivalLayer,
        departure: ModalDepartureLayer,
        slide_distance: f32,
    },
    ModalOpen {
        arrival: ModalArrivalLayer,
        slide_distance: f32,
    },
    ModalClose {
        departure: ModalDepartureLayer,
        slide_distance: f32,
    },
}
```

### Why not a ghost modal

The current connected-strip transition deliberately keeps the shared transition stub fully opaque. That is correct for unit-to-unit `<` / `>` transitions, but wrong for lifecycle open/exit/confirm:

- open has no real departing unit
- exit/confirm end the modal session entirely
- `-` and `+` should fade with the modal unit in these lifecycle cases

Using explicit lifecycle variants keeps those semantics clear and avoids fake modal content in state and rendering.

---

## Trigger Points

### Open trigger

Wire a new lifecycle transition helper into `open_header_modal()` in `src/app.rs`.

Execution order:

1. Build the new modal as today.
2. Rebuild simple layout with full reset semantics.
3. If simple-unit layout exists, create a `ModalOpen` entry for `active_unit_index`.
4. Render the modal overlay from the transition layer immediately.

### Exit trigger

Add a helper for semantic modal exit, then route all exit-equivalent paths through it instead of directly clearing `self.modal`.

Likely entry points include:

- `dismiss_modal()`
- `Message::ModalBackdropPressed`
- first-list back-out paths in `composite_go_back()`
- any other path that currently closes the modal session without confirmation

Execution order:

1. Capture frozen departure geometry/content from the current active unit.
2. If simple-unit layout exists, push `ModalClose` with left/back semantics.
3. Preserve enough overlay state for the departure animation to render after `self.modal` becomes `None`.
4. When the animation completes, clear the retained overlay transition state.

### Confirm trigger

Add a helper for semantic modal completion, then route all field-complete paths through it.

Likely entry points include:

- `confirm_modal_value()` when `FieldAdvance::Complete`
- `super_confirm_modal_field()` when it completes

Execution order:

1. Finish the field mutation and section-state updates as today.
2. Capture frozen departure geometry/content before the modal session is fully discarded.
3. Push `ModalClose` with right/confirm semantics.
4. Clear live modal state while retaining the one-shot departure layer for rendering.

---

## Rendering Changes

The overlay renderer in `src/ui/mod.rs` currently assumes the overlay only exists while `app.modal.is_some()`.

That must change to:

- render the modal overlay when `app.modal.is_some()`
- or when a lifecycle transition layer still needs to paint after live modal state has been cleared

### Render responsibilities by variant

#### `ConnectedTransition`

- keep the current shared-strip behavior
- keep transition stub alpha at `1.0`
- preserve existing unit-to-unit semantics

#### `ModalOpen`

- render only the arriving unit
- start from positive x offset (`shift right`)
- animate to `0`
- fade whole-unit alpha from `0.0 -> 1.0`
- keep the active unit interactive from frame 1, matching current arrival behavior

#### `ModalClose`

- render only the departing unit
- start from `0`
- animate to signed offset based on lifecycle semantics:
  - exit: positive x offset
  - confirm: negative x offset
- fade whole-unit alpha from `1.0 -> 0.0`
- render as non-interactive preview content because the live modal session is already over

---

## Stub Alpha Rules

Lifecycle transitions use unit-owned stub fading, not shared-stub ownership.

### Unit-to-unit nav

- `<` / `>` transition stub remains fully opaque
- existing connected-strip logic remains unchanged

### Lifecycle open/exit/confirm

- `-` and `+` belong to the entering/leaving unit
- they fade with that unit's whole-strip alpha
- there is no special always-opaque shared stub in these variants

This is the key rule that prevents lifecycle transitions from accidentally looking like normal inter-unit navigation.

---

## Overlay Retention Requirement

Departure-only animations need a small amount of retained overlay state after `self.modal` becomes `None`.

Recommended approach:

- keep live modal data ownership unchanged
- let `ModalTransitionLayer::ModalClose` carry everything required to render the departing strip on its own
- make the overlay entry condition depend on transition presence as well as live modal presence

Avoid reusing `self.modal` as a half-cleared zombie object just to keep the animation alive.

---

## Implementation Readiness Review (2026-04-18)

### What was missing

The original draft was close, but it still left a few implementation-critical seams implicit:

1. Top-level overlay gating:
   - `src/ui/mod.rs::view()` currently returns `modal_overlay(app, modal)` only when `app.modal.is_some()`
   - close-only lifecycle transitions cannot render unless the top-level view path also checks `app.modal_transitions`

2. Hard-close vs animated-close responsibilities:
   - `close_modal()` and `dismiss_modal()` currently settle transition state immediately
   - confirm/exit animation must not be routed through those helpers unchanged or the new transition will be cleared before it can render

3. Close-only overlay contents:
   - once the modal session has ended, there is no live modal to drive the composition panel or interactive active card
   - the handoff needs to say explicitly that close-only rendering is departure-strip only

4. Exact rewiring targets:
   - the earlier draft named the concepts, but not every concrete seam that has to change in the current codebase

### Clarified implementation rules

#### 1. Keep `close_modal()` as the hard-stop helper

Do not repurpose `close_modal()` into the animated lifecycle path.

Keep it as the immediate cleanup path for cases that really should settle everything at once:

- data refresh or invalidation
- forced teardown
- any future "abort all modal UI now" path

For task 30, add separate semantic lifecycle helpers that preserve just enough state for animation:

- open helper: creates `ModalOpen`
- exit helper: creates `ModalClose` with backward/left semantics
- confirm helper: creates `ModalClose` with forward/right semantics

That keeps the existing "hard close" behavior available and avoids hidden coupling.

#### 2. Close-only overlay is render-only

When `self.modal` has already been cleared and a `ModalClose` layer is still active:

- render the modal overlay shell
- render only the frozen departing strip
- do not render the composition panel
- do not render interactive modal content
- treat the overlay as visual playback, not a live session

#### 3. Top-level view gating must change

`src/ui/mod.rs::view()` currently short-circuits to `main_layout(app)` when `app.modal.is_none()`.

For lifecycle close transitions, the top-level rule must become:

- show overlay if `app.modal.is_some()`
- or if `app.modal_transitions.last()` is a lifecycle close layer that is still active

This is separate from the inner `modal_overlay(...)` logic. Both levels need to be updated.

#### 4. Exact rewiring checklist

Implementation should explicitly touch these seams:

- `src/app.rs`
  - `open_header_modal()`
  - `dismiss_modal()`
  - `close_modal()`
  - `confirm_modal_value()`
  - `super_confirm_modal_field()`
  - `composite_go_back()`
  - `tick()`
  - `has_active_modal_transition()`
- `src/ui/mod.rs`
  - top-level `view()`
  - `modal_overlay(...)` entry assumptions
  - overlay composition for connected/open/close/rest states
- `src/ui/modal_unit.rs`
  - one-unit lifecycle builders/render helpers
- `src/transition.rs`
  - new `ModalTransitionLayer` variants
- `src/main.rs`
  - backdrop dismissal route only if needed after the new exit helper is introduced

### Readiness decision

After these clarifications, the task is ready to hand off.

The core design is stable; the remaining work is implementation, not product-definition discovery.

---

## Implementation Steps

1. Extend `ModalTransitionLayer` in `src/transition.rs` with `ModalOpen` and `ModalClose`.
2. Add helper constructors in `src/app.rs` for:
   - `fire_modal_open_transition_if_possible()`
   - `fire_modal_exit_transition_if_possible()`
   - `fire_modal_confirm_transition_if_possible()`
3. Refactor modal close/complete call sites so semantic exit and semantic confirm route through distinct helpers.
4. Update pruning in `App::tick()` so all transition variants expire cleanly.
5. Update `has_active_modal_transition()` and overlay gating so closing transitions still repaint after `self.modal` is cleared.
6. Extend `src/ui/modal_unit.rs` with one-unit lifecycle render builders instead of forcing those cases through the connected-strip builder.
7. Update `src/ui/mod.rs` top-level `view()` and overlay composition to render:
   - connected transition
   - open-only arrival
   - close-only departure
   - at-rest active unit
   - main layout with retained close-only overlay when `self.modal.is_none()`
8. Add targeted tests for direction, alpha, and overlay retention.

---

## Validation

### Automated

Add targeted tests for:

- open transition starts to the right and settles at rest
- exit transition moves right and fades out
- confirm transition moves left and fades out
- lifecycle stubs fade with the unit
- connected `<` / `>` transitions still keep the shared stub fully opaque
- overlay still renders during a closing transition after `self.modal` is cleared
- top-level `view()` still returns the overlay path while a close transition is active and `self.modal.is_none()`
- non-simple modal states fall back to instant open/close

### Manual

Verify:

1. Open a modal from the wizard with both `Enter` and `nav_right`; both should animate identically.
2. Leave a modal with `Esc`, backdrop click, and back-out-from-first-list; all exit-equivalent paths should animate rightward.
3. Complete a modal with `Enter`, `nav_right`, and super-confirm where applicable; all confirm-equivalent paths should animate leftward.
4. During exit and confirm, the `-` or `+` edge fades with the unit rather than staying full opacity.
5. During the close animation, no composition panel or interactive modal session remains visible.
6. Existing unit-to-unit modal paging still behaves exactly as before.

---

## Recommendation

Implement lifecycle transitions as explicit one-sided transition variants, not ghost modals.

That keeps the renderer honest about what is happening:

- open = arrival only
- exit = departure only with left/back semantics
- confirm = departure only with right/forward semantics

It also cleanly preserves the existing invariant that only true inter-unit transitions own a shared always-opaque transition stub.
