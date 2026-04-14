# Plan: Modal Transition Redesign - Part 3: Sprint Mode

**Date:** 2026-04-14
**Status:** Proposed
**Part:** 3 of 3
**Prerequisite:** Part 1 (foundation) and Part 2 (state model and lifecycle) must be complete

---

## Purpose

Add sprint mode: a fast-transition state that activates when the user outpaces the animation, and stays active for the rest of the field session. Sprint mode handles:

- A shorter, configurable duration and easing for all subsequent transitions
- A clean layering and z-order model so overlapping transitions stack correctly
- A "sprint break" visual for the first sprint transition, where the new unit arrives independently rather than as a connected strip with the previous one
- Correct deferred-departure behavior: a unit that was mid-arrival when sprint triggered must complete its arrival before it begins departing

---

## Sprint Trigger

Sprint mode activates when a new transition is triggered **while at least one arrival is still in flight** (progress < 1.0).

```rust
// In the transition trigger logic, before creating new layers:
let has_in_flight_arrival = self.modal_transitions.iter().any(|entry| match entry {
    ModalTransitionLayer::ConnectedTransition { arrival, .. } => {
        arrival.progress(Instant::now()) < 1.0
    }
    ModalTransitionLayer::SprintBreakArrival(arrival) => {
        arrival.progress(Instant::now()) < 1.0
    }
    ModalTransitionLayer::DeferredDeparture(_) => false,
});

if has_in_flight_arrival {
    self.sprint_mode = true;
}
```

The `progress < 1.0` check avoids a false positive from a completed arrival that has not yet been pruned by the current frame's pruning pass.

Once active, `sprint_mode` remains `true` until the field session ends. It is never turned off mid-session.

### Session end reset

When the user navigates via the exit stub (`-`) or confirm stub (`+`), the field session ends. At that point:
- `self.sprint_mode = false`
- `self.modal_transitions.clear()`

This covers the normal field-close paths. Any other path that closes or resets the field should also reset sprint mode.

`rebuild_modal_unit_layout` resets `sprint_mode` to `false` on full reset (open and data refresh). Window resize uses the geometry-only update path (Part 2) which preserves `sprint_mode` and in-flight layers; resizing does not end a field session.

---

## Sprint Duration and Easing

When creating a `ModalArrivalLayer` or `ModalDepartureLayer` during sprint mode, use:

```rust
duration_ms: resolved_sprint_duration_ms(&self.theme),  // from Part 1
easing: self.theme.modal_transition_sprint_easing,
```

Where `resolved_sprint_duration_ms` returns `theme.modal_transition_sprint_duration.unwrap_or(theme.modal_transition_duration / 2)`.

---

## First Sprint Transition: The Sprint Break

The first sprint transition is the one that **triggers** sprint mode. It is visually different from all subsequent transitions because the arriving unit (C) cannot be part of a connected strip with the in-progress arrival (B).

### What "connected strip" means

In a normal transition (Part 2), the departing and arriving units form one rigid strip sharing a transition stub. This works because only one transition is in flight at a time.

When sprint triggers, unit B is still arriving at an arbitrary position. Anchoring C to B's current trailing stub would produce an unpredictable starting position for C.

### Sprint break behavior

When the sprint trigger fires (sprint mode just became true):

1. **Unit C arrives independently.** Push a `ModalTransitionLayer::SprintBreakArrival(arrival)` to `self.modal_transitions` where `arrival` is a `ModalArrivalLayer` created at sprint duration and easing.
2. **C's strip** consists only of C's own content: `[C_leading_stub | spacer | C_modals | spacer | C_trailing_stub]` (for `shows_stubs: true`). There is no shared transition stub with B.
3. **C's leading stub kind** follows Part 1's assignment rules: `Exit` if C is the first unit, `NavLeft` otherwise.
4. **C's leading stub fades in** with the rest of C's arriving group (`alpha = p`). It is not a transition stub, so it does not render at full opacity.
5. **B's ongoing arrival** (`ConnectedTransition` entry below C in the vec) continues at its original pace. It will be converted to a `DeferredDeparture` once its arrival completes (see below).
6. **A's ongoing departure** continues at its original pace. It is part of B's `ConnectedTransition` entry and is not affected by sprint mode.

### Why this breaks the train metaphor

During the sprint break, both B's trailing stub and C's leading stub may be briefly visible on the right side of the screen simultaneously. This is expected and acceptable. The user triggered this by moving faster than the animation. Subsequent transitions return to the connected-strip model only when the newest arrival has settled (see Subsequent Sprint Transitions below).

---

## Subsequent Sprint Transitions

After the sprint break, new transitions use sprint speed. Whether they use the connected-strip model depends on whether the newest arrival has settled.

**Determining which variant to create for each new transition:**

Inspect the newest arrival-bearing layer in `modal_transitions` (the last `ConnectedTransition` or `SprintBreakArrival` in the vec, ignoring `DeferredDeparture`):

```rust
let newest_arrival_in_flight = self.modal_transitions.iter().rev()
    .find_map(|entry| match entry {
        ModalTransitionLayer::ConnectedTransition { arrival, .. } => Some(arrival),
        ModalTransitionLayer::SprintBreakArrival(arrival) => Some(arrival),
        ModalTransitionLayer::DeferredDeparture(_) => None,
    })
    .map(|a| a.progress(Instant::now()) < 1.0)
    .unwrap_or(false);
```

- If `newest_arrival_in_flight` is **true**: the new unit cannot anchor to an arbitrary mid-animation position - push `SprintBreakArrival` (independent strip, own leading stub fades in at `alpha = p`).
- If `newest_arrival_in_flight` is **false**: the newest arrival has settled - push `ConnectedTransition` at sprint speed with a shared transition stub.

**Case A - newest arrival at rest:** `ConnectedTransition` at sprint speed. One rigid strip, shared transition stub. Train-car behavior, just faster.

**Case B - newest arrival still in flight:** `SprintBreakArrival`. The new unit arrives independently. Both leading stubs may be briefly visible simultaneously. This is expected.

There is no cap on the number of stacked transitions.

---

## Deferred Departure: Unit B After Sprint Break

Unit B was mid-arrival when sprint triggered. It must complete its arrival before departing.

### Detection and conversion

In the per-frame pruning loop (Part 2), fill in the stub arm for non-newest completed arrivals:

```
For each ConnectedTransition { arrival, departure } at index i
    where i < modal_transitions.len() - 1   // not the newest entry
    and arrival.progress >= 1.0:

    // Arrival is complete. Start a deferred departure at sprint speed.
    let new_departure = ModalDepartureLayer {
        content: departure.content,          // keep frozen content from original transition
        geometry: departure.geometry,        // keep frozen geometry from original transition
        focus_direction: departure.focus_direction,
        started_at: Instant::now(),          // departure begins now, not at original transition start
        duration_ms: resolved_sprint_duration_ms(&self.theme),
        easing: self.theme.modal_transition_sprint_easing,
    };

    replace entry at index i with ModalTransitionLayer::DeferredDeparture(new_departure)
```

The departure's `content` and `geometry` were frozen when the `ConnectedTransition` was created and remain valid. Only the timing resets to mark the moment B starts leaving.

### Departure behavior for B

When B departs (after C has arrived):
- C is the active unit with its left stub at `x = 0`
- B departs in the direction of the original transition, fading out and sliding off
- B's departing strip is `[B_left_stub | B_modals]` (or without stubs if `shows_stubs: false`)
- C's left stub is part of C, already at rest, and serves as the visual right boundary
- There is no separate transition stub for B's departure: B simply fades and slides out

The transition stub concept applies only to live transitions where dep and arr move together. B is already done arriving and C is at rest; B's departure is an independent fade-and-exit.

---

## Z-Order During Overlapping Transitions

The two-pass render from Part 2 (departure components in pass 1, arrival components in pass 2, both in push order) naturally handles all sprint cases:

During the sprint break:
- A is departing (pass 1, rendered first = bottom)
- B's `ConnectedTransition` arrival renders above A in pass 2
- C's `SprintBreakArrival` was pushed after B, so it renders last in pass 2 = top

After B's arrival completes and converts to `DeferredDeparture`:
- B's departure entry renders in pass 1
- C's arrival (now the sole pass-2 element) renders on top

After A and B finish and are pruned:
- C is the only entry, renders at rest once pruned

---

## Stacking Beyond Two Overlapping Transitions

The user may trigger additional transitions before earlier ones complete. The same rules apply:

- Each new transition pushes `ConnectedTransition` if the newest arrival is already settled, or `SprintBreakArrival` if the newest arrival is still in flight - regardless of how many layers are already stacked
- Each completed non-newest arrival in a `ConnectedTransition` is converted to `DeferredDeparture` at sprint speed
- The newest arrival is always on top (last in vec = renders last in pass 2)
- Departures accumulate until they complete and are pruned

There is no cap on the number of stacked transitions.

---

## Summary of Sprint Mode Behavior Per Unit

| Unit | Condition | Arrival speed | Departure speed | Notes |
|---|---|---|---|---|
| A (original active) | Was active when sprint triggered | N/A | Normal (already in flight) | Continues at original pace |
| B (mid-arrival when sprint triggered) | Was arriving when sprint triggered | Normal (continues unchanged) | Sprint (deferred until B's arrival completes) | Converted to DeferredDeparture automatically |
| C (first sprint arrival) | Triggered sprint mode; B was still in flight | Sprint | Sprint (if next transition fires before C completes; otherwise no departure) | SprintBreakArrival, independent strip, own leading stub |
| D+ (sprint already active, newest arrival at rest) | Triggered while latest arrival has completed | Sprint | Sprint | ConnectedTransition with shared transition stub |
| D+ (sprint already active, newest arrival in flight) | Triggered while latest arrival is still animating | Sprint | Sprint | SprintBreakArrival, same rules as C above |

---

## Execution Steps

1. In the transition trigger logic in `src/app.rs`:
   - Before creating new layers, run the `has_in_flight_arrival` check and set `self.sprint_mode = true` if so
   - When `sprint_mode` is true, use sprint duration and easing for new arrival and departure layers
   - When this is the sprint trigger (sprint just became true): push `SprintBreakArrival` instead of `ConnectedTransition`
   - When sprint is already active: check whether the newest arrival is at rest (`newest_arrival_in_flight` check above); if at rest push `ConnectedTransition` at sprint speed with a shared transition stub; if still in flight push `SprintBreakArrival`

2. In the per-frame pruning loop in `src/app.rs`:
   - Fill in the non-newest completed arrival branch: convert `ConnectedTransition` to `DeferredDeparture` with reset timing and sprint speed

3. In the field-close paths in `src/app.rs`:
   - On exit stub navigation (`-`) and confirm stub navigation (`+`): set `sprint_mode = false` and call `self.modal_transitions.clear()`

4. In the rendering code in `src/ui.rs`:
   - For `SprintBreakArrival`: render C's leading stub as part of C's fading group (`alpha = p`); do not render it at full opacity
   - For `DeferredDeparture`: render the departure component only; apply its own `x_offset` and group alpha
   - Connected-strip arrivals and departures follow Part 2 rendering unchanged

5. Compile and verify the app cleanly.

---

## Out of Scope

- Composition panel, collection-mode layout, checksum-based refresh skipping
- Any cap on the number of overlapping transitions

---

## Validation

### Automated

- `sprint_mode` is false before any transition is triggered
- `sprint_mode` becomes true when a transition is triggered while an arrival with `progress < 1.0` is in flight
- `sprint_mode` remains true after subsequent transitions in the same session
- `sprint_mode` is false after `rebuild_modal_unit_layout` full reset
- A completed non-newest `ConnectedTransition` arrival generates exactly one `DeferredDeparture` at sprint speed
- Sprint duration resolves to `modal_transition_duration / 2` when `modal_transition_sprint_duration` is `None`

### Manual

1. **Normal transitions (no sprint):** Move focus across unit boundaries slowly, waiting for each animation to complete. Behavior is identical to Part 2. Sprint mode never activates.

2. **Sprint trigger:** During a transition animation, immediately move focus to the next unit. Sprint mode activates. The new unit (C) slides in from the right at a noticeably faster speed than B's ongoing arrival. C is independently animated: its own leading stub (kind determined by unit position) fades in with it.

3. **B's deferred departure:** After C arrives, B fades out and slides off at sprint speed. This happens automatically as soon as B's original-speed arrival completes.

4. **Z-order:** During the sprint break, C is fully visible on top of B. B is visible on top of A (if A is still fading). No unit obscures a later-arriving unit.

5. **Sprint continues:** After the sprint break, move focus to the next unit (D). D's transition is a connected strip at sprint speed. C and D share a transition stub.

6. **Interaction during sprint:** During any sprint transition, hint keys and search work on the arriving unit immediately. No need to wait for animation.

7. **Session reset:** Navigate via the confirm stub (`+`) or exit stub (`-`). Sprint mode is cleared. Open the field again and confirm transitions start at normal speed.

---

## Resolved Design Decisions

1. **Resize vs. sprint mode:** resize uses the geometry-only update path (Part 2); `sprint_mode` and in-flight layers are preserved across resize.

2. **Layer render data:** resolved in Part 2. Arrivals use `unit_index` + live content. Departures use frozen content and geometry. `DeferredDeparture` reuses the frozen content and geometry from its original `ConnectedTransition`; only timing resets.

3. **Sprint-break leading stub:** C's leading stub kind is resolved from Part 1's assignment rules (Exit if first unit, NavLeft otherwise). It fades in with C's group at `alpha = p`, not at full opacity, because it is not a transition stub.

4. **Sprint trigger precision:** uses `arrival.progress() < 1.0` check, not `!modal_transitions.is_empty()`, to avoid a false positive from a completed but not-yet-pruned arrival.

5. **Layer pairing for sprint cases:** the `ModalTransitionLayer` enum from Part 2 (`ConnectedTransition`, `SprintBreakArrival`, `DeferredDeparture`) makes all three sprint-mode cases explicit. No additional pairing model is needed in Part 3.

---

## Review Addendum (2026-04-14)

### Findings

1. The plan defines a sprint break only for the first overlapping sprint transition, then says later sprint transitions return to the connected-strip model. That only works if the current active arrival is already at rest.

2. The same plan also says the user may trigger additional transitions before earlier ones complete and that there is no cap on stacked transitions. With those rules, unit D can be triggered before C finishes arriving. In that case D has the same anchoring problem that forced the original sprint break.

3. Because of that gap, the current wording around "After the sprint break" and "C is the active unit and is at rest" is too optimistic. It describes one happy path, not the full state machine.

### Recommended Changes

1. Replace the first-sprint-only rule with a general rule:
   - whenever a new transition is triggered while the newest arrival is still in flight, create `SprintBreakArrival`
   - only create a `ConnectedTransition` when the source active unit is visually at rest

2. Rewrite the "After the sprint break" section to distinguish two cases:
   - case A: current active arrival has completed -> next transition is `ConnectedTransition`
   - case B: current active arrival is still in flight -> next transition is another `SprintBreakArrival`

3. Add an explicit definition for the check used by new sprint transitions:
   - inspect the newest arrival-bearing layer in `modal_transitions`
   - if its arrival progress is `< 1.0`, the next transition must be an independent arrival, not a connected strip

4. Update the stacking section so it matches the generalized rule:
   - "each new transition pushes either `ConnectedTransition` or `SprintBreakArrival` depending on whether the newest arrival is already settled"
   - this avoids the contradiction between unlimited overlap and first-sprint-only sprint breaks

5. In the execution steps, change:
   - "When sprint is already active (not the first sprint transition): push `ConnectedTransition`..."
   - to:
   - "When sprint is already active: push `ConnectedTransition` if the newest arrival is at rest; otherwise push `SprintBreakArrival`"

### Recommendation

Part 3 is not implementation-ready until the repeated-overlap rule is fixed. Once that is clarified, the rest of the sprint plan looks workable.
