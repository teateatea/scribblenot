# Plan: Modal Transition Redesign - Part 2: State Model and Transition Lifecycle

**Date:** 2026-04-14
**Status:** Proposed
**Part:** 2 of 3
**Prerequisite:** Part 1 (foundation) must be complete
**Prerequisite for:** Part 3 (adaptive transitions)

---

## Purpose

Replace the current implicit transition system with an explicit state model:
- App state holds named, typed fields for the active unit, prepared neighbors, and in-flight transitions
- Layout is precomputed on open, resize, and data refresh - not derived at render time
- Transitions are unit-index-based, not modal-snapshot comparisons
- The full strip (departing unit + transition stub + arriving unit) slides as one piece
- The transition stub stays full opacity throughout; its arrow direction is set at transition start
- Focus is immediately active in the arriving unit from the first frame of the transition
- Arriving and departing elements use whole-unit alpha (render-target fade) to prevent background pop-in
- Each in-flight layer carries its own frozen geometry and content, making it independent of layout rebuilds and live state changes

**Scope constraint:** This transition system applies only while the modal can produce a `SimpleModalSequence` (i.e. `supports_simple_list_teasers()` is true). If either the departing or arriving modal state is not simple-list-teaser compatible, do not start a transition. If the modal leaves simple mode during an in-flight transition, clear `modal_transitions` and render the active unit at rest on the next frame.

---

## New Data Structures

### Supporting types

```rust
/// Frozen geometry for one unit, captured at transition start.
/// Insulates in-flight layers from layout rebuilds (e.g. window resize).
pub struct UnitGeometry {
    pub unit_index: usize,
    pub modal_index_range: std::ops::Range<usize>, // which modals in the sequence belong to this unit
    pub shows_stubs: bool,
    pub leading_stub_kind: Option<ModalStubKind>,
    pub trailing_stub_kind: Option<ModalStubKind>,
    pub effective_spacer_width: f32,
    pub modal_widths: Vec<f32>,                    // width of each modal card in the unit, in order
    pub modal_x_offsets: Vec<f32>,                 // x position of each modal card relative to unit origin, in order
}

/// Frozen render inputs for one unit's modals, captured at transition start.
/// Prevents the departing unit's visuals from mutating if the user types during a transition.
/// Focus leaves this unit at transition start, so its render state should not change.
/// In Part 2, arrivals use live content via unit_index only while the modal remains in simple
/// mode. If simple mode is lost during an in-flight transition, settle immediately rather than
/// animating from partially invalid live state.
pub struct UnitContentSnapshot {
    pub modals: Vec<ModalRenderSnapshot>,
}

// ModalRenderSnapshot: capture whichever per-modal fields the renderer reads from live state,
// including: query text, filtered rows, cursor position, scroll position.
// Use the same field names as the live modal structs.
```

### Layer types

```rust
/// Direction focus moved to trigger a transition.
/// Strip slides in the opposite direction: Forward = strip moves left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Forward,   // focus moved to a higher-index unit (rightward)
    Backward,  // focus moved to a lower-index unit (leftward)
}

/// One unit currently sliding and fading in.
/// Uses live modal content via unit_index (focus is here; user input is correct).
/// Geometry is frozen so resize does not invalidate this layer.
#[derive(Debug, Clone)]
pub struct ModalArrivalLayer {
    pub unit_index: usize,
    pub geometry: UnitGeometry,
    pub focus_direction: FocusDirection,
    pub started_at: Instant,
    pub duration_ms: u64,
    pub easing: ModalTransitionEasing,
}

/// One unit currently sliding and fading out.
/// Content and geometry are both frozen at transition start.
#[derive(Debug, Clone)]
pub struct ModalDepartureLayer {
    pub content: UnitContentSnapshot,
    pub geometry: UnitGeometry,
    pub focus_direction: FocusDirection,
    pub started_at: Instant,
    pub duration_ms: u64,
    pub easing: ModalTransitionEasing,
}
```

### Transition entry enum

```rust
/// A single animation entry. The three variants correspond to the three
/// distinct cases that can exist in the transition vec.
pub enum ModalTransitionLayer {
    /// Normal transition: dep and arr form one rigid strip with a shared transition stub.
    /// Both layers share the same x_offset, driven by the arrival's progress.
    ConnectedTransition {
        arrival: ModalArrivalLayer,
        departure: ModalDepartureLayer,
    },
}
```

`ModalArrivalLayer` and `ModalDepartureLayer` are kept as separate types because they have different data (live `unit_index` vs frozen `content`) and different pruning rules.

---

## AppState Changes

### Fields to add

```rust
// Precomputed layout for the current modal sequence and viewport.
// None before the first layout build or whenever the current modal state
// does not support simple-unit layout (e.g. query active, collection mode, etc.).
pub modal_unit_layout: Option<SimpleModalUnitLayout>,

// Index into modal_unit_layout.units identifying the currently active unit.
// Only meaningful when modal_unit_layout is Some(...).
pub active_unit_index: usize,

// Prepared neighbors of the active unit.
// None when no unit exists in that direction, or when modal_unit_layout is None.
pub prev_prepared_unit: Option<usize>,   // unit index
pub next_prepared_unit: Option<usize>,   // unit index

// In-flight animation entries, ordered oldest-to-newest (newest = last).
// Newest entry is always topmost in the render stack.
pub modal_transitions: Vec<ModalTransitionLayer>,

```

### Fields to remove

```rust
modal_stream_transition: Option<ModalStreamTransition>,
modal_stream_departures: Vec<ModalStreamDeparture>,
```

Remove the `ModalStreamTransition`, `ModalStreamDeparture`, and `ModalStreamCarry` structs entirely.

---

## Layout Rebuild

### Function signature

```rust
fn rebuild_modal_unit_layout(&mut self, viewport_size: Size)
```

### Behavior

Steps 1-4 run on every call. Steps 5-6 run on full reset only (open and data refresh), not on resize.

1. Call `build_simple_modal_unit_layout` with the current modal sequence and `viewport_size`. Store the result in `self.modal_unit_layout`.
2. Locate the unit whose range contains `self.modal.list_idx`. Set `self.active_unit_index` to that unit's index.
3. Set `self.prev_prepared_unit`:
   - `Some(active_unit_index - 1)` if `active_unit_index > 0`
   - `None` otherwise
4. Set `self.next_prepared_unit`:
   - `Some(active_unit_index + 1)` if `active_unit_index < n - 1` (where `n` = number of units)
   - `None` otherwise
5. *(Full reset only)* Clear `self.modal_transitions`.

### Resize and in-flight layers

Because each layer carries frozen geometry, resize only requires steps 1-4. In-flight layers in `modal_transitions` render from their own `layer.geometry` and are unaffected by the layout rebuild. There is no need to remap unit indexes or clear in-flight state on resize.

### Call sites

**Full reset** (all 6 steps): app initialization, data refresh, `open_header_modal()`.

**Geometry update** (steps 1-4 only): window resize. In-flight layers are preserved; their frozen geometry is unaffected.

**Query/filter/search edits:** if the modal is currently in transition, immediately settle to the active unit at rest (clear `modal_transitions`). Otherwise rebuild or clear layout depending on whether simple mode is still available.

**Field commits and modal sequence modifications:** rebuild layout (steps 1-4).

**Pure focus movement** (`list_idx` changing within the same unit): only update `active_unit_index`, no rebuild needed.

**Note on `same_modal_unit_window()`:** the replacement by unit-index comparison applies only in the simple-layout path. Non-simple states bypass transition creation entirely.

### Function rewiring checklist

All of the following must be addressed before implementation is complete:

- `set_viewport_size()` - call `rebuild_modal_unit_layout` (steps 1-4 only); preserve in-flight layers
- `set_modal_query()` - settle-and-clear if in transition; otherwise rebuild or clear layout based on simple-mode availability
- `open_header_modal()` - full reset: layout build, transition clear
- `reload_data()` - full reset: layout build, transition clear
- Every existing call site of `start_modal_stream_transition(...)` - migrate to the new transition trigger path

---

## Transition Trigger

### When a transition fires

Focus crosses a unit boundary when `advance_field` (or equivalent) returns a result where the new `list_idx` belongs to a different unit than `self.active_unit_index`.

### Steps

1. Determine `focus_direction`:
   - `Forward` if the new `list_idx` is greater than the old one
   - `Backward` if the new `list_idx` is less than the old one
2. Record `departing_unit_index = self.active_unit_index`.
3. Find `arriving_unit_index` by locating which unit contains the new `self.modal.list_idx`.
4. Capture frozen data:
   ```
   departure_geometry = UnitGeometry::from_layout(&self.modal_unit_layout, departing_unit_index)
   departure_content  = UnitContentSnapshot::from_modal_state(&self.modal, departing_unit_index)
   arrival_geometry   = UnitGeometry::from_layout(&self.modal_unit_layout, arriving_unit_index)
   ```
5. Create a `ModalDepartureLayer`:
   ```
   content: departure_content
   geometry: departure_geometry
   focus_direction: focus_direction
   started_at: Instant::now()
   duration_ms: self.theme.modal_transition_duration
   easing: self.theme.modal_transition_easing
   ```
6. Create a `ModalArrivalLayer`:
   ```
   unit_index: arriving_unit_index
   geometry: arrival_geometry
   focus_direction: focus_direction
   started_at: Instant::now()
   duration_ms: self.theme.modal_transition_duration
   easing: self.theme.modal_transition_easing
   ```
7. Push `ModalTransitionLayer::ConnectedTransition { arrival, departure }` to `self.modal_transitions`.
8. Set `self.active_unit_index = arriving_unit_index`.
9. Recompute `prev_prepared_unit` and `next_prepared_unit` from the new `active_unit_index`.

Focus (`self.modal`) is already at the new modal before these steps run. Interaction with the arriving unit is live from this frame.

### Removing old helpers

- `same_modal_unit_window()`: remove entirely. Unit-index comparison replaces it.
- `start_modal_stream_transition()`: remove entirely. The steps above replace it.

---

## Per-Frame Animation Pruning

Run at the start of each frame before rendering. Iterate from newest to oldest to avoid index shifting when removing entries, or collect indexes to remove and apply in reverse.

```
for each entry in self.modal_transitions:

    match entry:
        ConnectedTransition { arrival, .. }:
            if arrival.progress >= 1.0:
                // Arrival complete. Remove entry; active unit renders at rest.
                // Part 3 queue drain hook: fire next queued transition here if present.
                remove entry
            // departure.progress >= 1.0 is handled implicitly:
            // both layers share the same started_at and duration_ms,
            // so they complete together.
```

Progress computation:
```
elapsed = now - layer.started_at
raw_progress = elapsed.as_millis() as f32 / layer.duration_ms as f32
eased_progress = apply_easing(layer.easing, raw_progress.clamp(0.0, 1.0))
```

In Part 2, only `ConnectedTransition` is used. Part 3 inserts the queue drain hook at the pruning completion point.

---

## Strip Geometry

The departing and arriving units slide as a single rigid strip. The `x_offset` is identical for both components of a `ConnectedTransition`.

### Geometry source

All positions, widths, `shows_stubs`, and stub kinds come from the frozen geometry on the layer (`arrival.geometry`, `departure.geometry`), not from `self.modal_unit_layout`. This is what makes resize-safe rendering possible.

### Transition stub

The transition stub is the departing unit's trailing stub on the focus side:

- `FocusDirection::Forward`: the departing unit's `NavRight` stub (right side) is the transition stub.
- `FocusDirection::Backward`: the departing unit's `NavLeft` stub (left side) is the transition stub.

The transition stub renders once, at full opacity (`alpha = 1.0`), throughout the transition.

**Arrow direction:** set immediately at transition start. Never changes during the animation.
- `FocusDirection::Forward` (strip moves left): transition stub shows `<`
- `FocusDirection::Backward` (strip moves right): transition stub shows `>`

### Slide direction and distance

Use `arrival.geometry` to determine `shows_stubs` and `effective_spacer_width`.

For `FocusDirection::Forward` (strip moves left, offset is negative):

```
slide_distance = if arrival.geometry.shows_stubs {
    viewport_width - modal_stub_width
} else {
    viewport_width + arrival.geometry.effective_spacer_width
}

x_offset(p) = -(slide_distance * p)
```

For `FocusDirection::Backward` (strip moves right, offset is positive):

```
x_offset(p) = +(slide_distance * p)
```

where `p` is the eased progress (0.0 to 1.0).

### Mixed shows_stubs cases

When `departure.geometry.shows_stubs != arrival.geometry.shows_stubs`:
- `dep.shows_stubs: true, arr.shows_stubs: false`: transition stub slides off-screen at full opacity with the strip. `slide_distance = viewport_width + arrival.geometry.effective_spacer_width`.
- `dep.shows_stubs: false, arr.shows_stubs: true`: no transition stub. Arriving unit's leading stub slides in with the group at `alpha = p`. `slide_distance = viewport_width + arrival.geometry.effective_spacer_width`.

Default when either unit has `shows_stubs: false`: `slide_distance = viewport_width + effective_spacer_width`.

### Prepared unit starting position

At rest (no transition), prepared neighbors have their first modal at `x = viewport_width + effective_spacer_width` (next) or mirrored on the left (prev).

---

## Rendering

### Two-pass render over `modal_transitions`

Iterate `modal_transitions` from index 0 (oldest) to last (newest) for each pass.

**Pass 1 - departure components (bottom of stack):**
- `ConnectedTransition { departure, .. }`: render departure with x_offset and group alpha.

**Pass 2 - arrival components (top of stack):**
- `ConnectedTransition { arrival, .. }`: render arrival with x_offset and group alpha.

If `modal_transitions` is empty after pruning: render the active unit at rest with no alpha effect.

### Alpha rules

For a departure at eased progress `p`:
- Departing modals and departing far stub: render as a group at `alpha = 1.0 - p`
- Transition stub (if `departure.geometry.shows_stubs: true`): `alpha = 1.0`

For a `ConnectedTransition` arrival at eased progress `p`:
- Arriving modals and arriving far stub: render as a group at `alpha = p`
- Transition stub: already rendered by the departure component at `alpha = 1.0`; do not render again

**Whole-unit group fade:** apply alpha to the composited group, never per-child, to prevent background surfaces from popping in front of text. Two implementation options - choose during Part 2 based on what `iced` supports:
- **Preferred** (if a clear `iced` mechanism is identified): composite the departing modals + far stub into an offscreen render-target buffer and render at `alpha = 1.0 - p`; same for arriving group at `alpha = p`.
- **Fallback:** use the best achievable group fade with current `iced` widgets; track true whole-unit compositing as a follow-up improvement item.

### x_offset source

For a `ConnectedTransition`: compute `x_offset` from `arrival.progress()`. Apply the same offset to both departure and arrival render passes. (Both layers share the same `started_at` and `duration_ms`, so progress is always equal.)

---

## Code to Remove

Remove the following after implementing the new system. The project must compile cleanly without them.

| Symbol | Location | Reason |
|---|---|---|
| `ModalStreamTransition` | `src/app.rs` | replaced by `ModalArrivalLayer` / `ModalTransitionLayer` |
| `ModalStreamDeparture` | `src/app.rs` | replaced by `ModalDepartureLayer` / `ModalTransitionLayer` |
| `ModalStreamCarry` | `src/app.rs` | carry logic no longer needed |
| `same_modal_unit_window()` | `src/app.rs` | replaced by unit-index comparison |
| `start_modal_stream_transition()` | `src/app.rs` | replaced by new trigger logic |
| `suppress_duplicate_transition_stub()` | `src/ui.rs` | replaced by explicit stub ownership via `ModalTransitionLayer` |
| `transition_shared_stub_modes()` | `src/ui.rs` | replaced by `ModalStubKind` + direction |
| Old departure/arrival rendering paths | `src/ui.rs` | replaced by two-pass render over `modal_transitions` |

---

## Execution Steps

1. Add `UnitGeometry`, `UnitContentSnapshot`, `FocusDirection`, `ModalArrivalLayer`, `ModalDepartureLayer`, `ModalTransitionLayer` to `src/app.rs`
2. Add `modal_transitions`, `modal_unit_layout`, `active_unit_index`, `prev_prepared_unit`, `next_prepared_unit` to `AppState`; remove `modal_stream_transition` and `modal_stream_departures`
3. Remove `ModalStreamTransition`, `ModalStreamDeparture`, `ModalStreamCarry` structs
4. Implement `rebuild_modal_unit_layout` and wire to init, data refresh, and resize
5. Replace `start_modal_stream_transition` with the new transition trigger (steps 1-9 above); capture frozen geometry and content snapshot; remove `same_modal_unit_window`
6. Implement per-frame pruning loop for `ConnectedTransition`; add a comment marking where Part 3 inserts the queue drain hook
7. **Animation-loop plumbing** (`src/app.rs`, `src/main.rs`):
   - Replace `App::tick()` pruning logic with `modal_transitions` pruning
   - Replace `has_active_modal_stream_transition()` with a helper that checks `self.modal_transitions.is_empty()`
   - Update `clear_modal_stream_animations()` and every call site that settles or clears transitions
   - Update `main.rs` subscription logic to tick at animation cadence whenever `modal_transitions` is non-empty
8. Rewrite transition rendering in `src/ui.rs`:
   - Two-pass render over `modal_transitions` (departure components then arrival components)
   - Strip geometry from frozen layer geometry, not from `modal_unit_layout`
   - Slide distance and `x_offset` calculation
   - Whole-unit group alpha for departing and arriving groups
   - Transition stub: full opacity, arrow set at transition start, direction from `FocusDirection`
   - `shows_stubs: false` path (no transition stub, `slide_distance = viewport_width + spacer`)
9. Remove `suppress_duplicate_transition_stub`, `transition_shared_stub_modes`, and old rendering paths
10. Compile and verify all tests pass

---

## Out of Scope

- Composition panel, collection-mode layout, checksum-based refresh skipping

---

## Validation

### Automated

- `rebuild_modal_unit_layout` correctly identifies the active unit after resize when focus stays in the same modal
- Transition trigger creates exactly one `ConnectedTransition` entry in `modal_transitions` when focus crosses a unit boundary
- Transition trigger does not fire when focus moves within the same unit
- After the `ConnectedTransition` entry reaches `p >= 1.0`, it is pruned and the active unit renders at rest
- `prev_prepared_unit` and `next_prepared_unit` are correctly updated after each transition
- Resizing during a transition does not clear `modal_transitions`; layers render from frozen geometry

### Manual

1. Move focus right across a unit boundary. The entire strip (departing left stub + departing modals + transition stub + arriving modals + arriving right stub) slides left as one rigid piece. No element starts moving before another.
2. Move focus left across a unit boundary. The strip slides right symmetrically.
3. During a transition (before animation completes), press an arrow key or hint key. Focus responds immediately in the arriving unit. The animation continues to completion independently.
4. Transition stub arrow: for a rightward focus move, the transition stub shows `<` from the first frame. For a leftward move, it shows `>`. Never changes during the animation.
5. The departing far stub and arriving far stub fade to/from transparent. The transition stub stays at full opacity throughout.
6. Resize during a transition. The focused modal remains visible. The in-flight animation continues without a visual jump.
7. No background surface pops in front of text during fade-in or fade-out. The entire modal group fades as one.
8. First unit: exit stub (`-`) on left. Last unit: confirm stub (`+`) on right. After a forward transition to a non-last unit, the new right stub is nav (`>`).

---

## Resolved Design Decisions

1. **Rebuild triggers:** also fires after modal-state mutations affecting snapshot inputs (search text, field commits, sequence changes). Pure focus movement only needs `active_unit_index` updated.

2. **Layer render data:** arrivals use `unit_index` + live content (focus is here; user input is correct). Departures use a frozen content snapshot captured at transition start, so the departing unit's visuals cannot mutate after focus leaves. Both layer types carry frozen geometry.

3. **Resize safety:** each layer carries frozen geometry captured at transition start. Resize only updates the active-unit layout (steps 1-4). In-flight layers render from their own geometry and are unaffected. No remapping or clearing is needed.

4. **Layer pairing:** `ModalTransitionLayer` wraps a `ConnectedTransition` (the only variant). `ModalArrivalLayer` and `ModalDepartureLayer` are kept separate because they carry different data and have different pruning rules. Two render passes (departures then arrivals) keep departure visuals below arrival visuals without an explicit z-sort.

5. **Arrival pruning:** when the `ConnectedTransition` arrival reaches `progress >= 1.0`, the entry is removed and the active unit renders at rest. Part 3 fires the next queued transition from this point.

---

## Review Addendum (2026-04-14)

### Findings

1. The plan currently treats `modal_unit_layout` as if it always exists after the first build, but the current code can only build a simple modal sequence while `supports_simple_list_teasers()` is true. Query text, collection mode, branch/nested flows, and filtered list states all break that assumption.

2. The arrival-layer design says arrivals use live content via `unit_index`, while the rebuild rules also say layout should be rebuilt after query changes and other modal-state mutations. Those two statements conflict in the current architecture: a query change can invalidate the simple layout entirely while an arrival is still animating.

3. The plan requires whole-unit fades via an offscreen composited buffer / render target. That is a valid visual goal, but it is not yet tied to a concrete `iced` implementation strategy in this document. As written, it is likely to slow implementation because it reads as mandatory without specifying the mechanism.

4. The reset wiring is slightly inconsistent. The prose says full reset happens on open and data refresh, but the execution steps only name init, data refresh, and resize. The actual modal entry point is `open_header_modal()`, so that call site should be named explicitly.

### Recommended Changes

1. Narrow Part 2 scope explicitly:
   - this transition system applies only while the modal can produce a `SimpleModalSequence`
   - if either the departing or arriving modal state is not simple-list-teaser compatible, do not start a transition
   - if the modal leaves simple mode during an in-flight transition, clear `modal_transitions` and render the active unit at rest on the next frame

2. Update the `modal_unit_layout` field comment and rebuild contract:
   - `modal_unit_layout: Option<SimpleModalUnitLayout>` remains `None` whenever the current modal state does not support simple-unit layout
   - `active_unit_index`, `prev_prepared_unit`, and `next_prepared_unit` are only meaningful when `modal_unit_layout` is `Some(...)`

3. Replace the current arrival-content rule with an implementation-safe version:
   - in Part 2, arrivals are live only while the modal remains in simple mode
   - if that guarantee is lost, settle immediately instead of trying to keep animating from partially invalid live state

4. Adjust rebuild triggers:
   - resize: rebuild layout if possible, but preserve in-flight layers
   - query/filter/search edits: if the modal is in transition, immediately settle to the active unit at rest; otherwise rebuild or clear layout depending on whether simple mode is still available
   - open via `open_header_modal()`: full reset, including layout build, transition clear, and `sprint_mode = false`

5. Reframe the offscreen-buffer requirement as one of these two options:
   - preferred if a clear `iced` mechanism is identified: keep it in Part 2 and name the mechanism
   - otherwise: make Part 2 use the best achievable group fade with current widgets, then track true whole-unit compositing as a follow-up improvement item

6. Add a short implementation note that `same_modal_unit_window()` is being replaced by unit-index comparison only in the simple-layout path. Non-simple states should bypass transition creation entirely.

### Recommendation

Part 2 is not implementation-ready yet. It is close once the scope is narrowed to simple-layout states and the query-change / live-arrival contradiction is removed.

---

## Implementation Readiness Addendum (2026-04-14)

### Findings

1. Part 2 still contains `sprint_mode` leftovers from the deprecated sprint design:
   - Execution step 2 says to add `sprint_mode` to `AppState`
   - the prior addendum also references resetting `sprint_mode = false` on open

2. Those sprint references now conflict with the current Part 3, which explicitly says there is no sprint state or mode switch. If left in place, Part 2 will create state that the implementation should not actually have.

### Recommended Changes

1. Remove every `sprint_mode` reference from Part 2.

2. Where Part 2 currently points forward to Part 3, keep the hook generic:
   - pruning completion can fire the next queued adaptive transition
   - no Part 2 state should exist only to support the deprecated sprint design

3. Keep the earlier scope fixes from the first review addendum. Those are still required before implementation.

### Approval Status

Not yet. Part 2 still needs the earlier simple-layout scope fixes, and it also needs the stale sprint-state references removed before implementation should start.

---

## Codex Review Addendum (2026-04-14)

### Findings

1. The current plan removes the stream-transition structs, but it does not explicitly cover the animation-loop plumbing that makes those transitions render frame-by-frame today. The implementation will also have to replace or rewrite:
   - `App::tick()`
   - `App::has_active_modal_stream_transition()`
   - `App::clear_modal_stream_animations()`
   - the `main.rs` subscription cadence that currently depends on `has_active_modal_stream_transition()`

   If those call sites are missed, the new `modal_transitions` state can exist without driving redraws correctly.

2. The rebuild rules are conceptually clear, but the execution section still does not name the main current code entry points that must be rewired:
   - `set_viewport_size()`
   - `set_modal_query()`
   - `open_header_modal()`
   - `reload_data()`
   - the confirm/back paths that currently call `start_modal_stream_transition(previous_modal)`

   That makes the implementation plan easy to under-wire, especially in a refactor this broad.

3. `UnitGeometry` still says "include any other fields the renderer reads." That is workable as design prose, but it is too loose for an implementation handoff. The current renderer depends on enough layout detail that this should be named concretely before coding starts.

### Recommended Changes

1. Add a dedicated execution step for runtime plumbing:
   - replace `tick()` pruning logic with `modal_transitions` pruning
   - replace `has_active_modal_stream_transition()` with a helper that reflects the new state
   - update `clear_modal_stream_animations()` and every call site that settles/clears transitions
   - update `main.rs` subscription logic to tick at animation cadence whenever `modal_transitions` is non-empty

2. Add an explicit call-site checklist under layout rebuild / trigger wiring:
   - `set_viewport_size()` rebuilds geometry
   - `set_modal_query()` settles-or-clears as specified
   - `open_header_modal()` performs a full reset
   - `reload_data()` performs a full reset
   - every current `start_modal_stream_transition(...)` caller is migrated to the new trigger path

3. Tighten `UnitGeometry` so it names the minimum frozen render inputs up front:
   - unit index
   - modal index range
   - `shows_stubs`
   - stub kinds
   - effective spacer width
   - any per-card width/x-position data the current `RenderedModalUnit` path needs

### Approval Status

Not yet. Part 2 is close, but I would not start implementation until the animation-loop plumbing and exact call-site rewiring are written into the plan.
