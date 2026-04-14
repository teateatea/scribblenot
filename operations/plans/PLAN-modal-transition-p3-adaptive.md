# Plan: Modal Transition Redesign - Part 3: Adaptive Transitions

**Date:** 2026-04-14
**Status:** Proposed
**Part:** 3 of 3
**Prerequisite:** Part 1 (foundation) and Part 2 (state model and lifecycle) must be complete
**Supersedes:** DEPRECATED-PLAN-modal-transition-p3-sprint.md

---

## Purpose

Add adaptive transition behavior: a self-tuning per-stub duration system that learns which unit boundaries are navigated frequently and adjusts their transition speed over time.

The core model:
- Each inter-unit stub has a stored duration, initialized to the theme max
- When the user triggers a new transition while one is already in flight, the in-flight stub's duration is reduced AND the new stub's duration is reduced, then the new transition is queued
- Transitions never overlap; the queue drains FIFO after each completion
- Learned durations persist to `transition_tuning.json` across sessions
- A `+` keybinding resets all stored durations to the theme max

There is no sprint state, no mode switch, and no overlapping layers. `modal_transitions` always contains at most one entry.

---

## New Data Structures

### QueuedTransition

```rust
pub struct QueuedTransition {
    pub arriving_unit_index: usize,
    pub focus_direction: FocusDirection,
}
```

Holds the destination for a pending transition. Duration is not stored here - it is resolved from `transition_tuning` at fire time, so any reductions that occur while a transition is waiting in the queue are reflected when it fires.

### Stub key

**Prerequisite:** stable modal boundary identifiers must be added to the frozen geometry layer (`UnitGeometry`) before Part 3 can implement this. Specifically, `UnitGeometry` should carry the `HierarchyList.id` of the boundary list on each side (last list ID of the unit on the left, first list ID of the unit on the right). Add these fields to `UnitGeometry` in Part 2's execution steps or at the start of Part 3 implementation.

A stub key uniquely identifies the relationship between two adjacent units. It is derived from the `HierarchyList.id` values of the two boundary lists (last list of the left unit, first list of the right unit), sorted lexicographically and joined with `|`:

```
key = min(left_list_id, right_list_id) + "|" + max(left_list_id, right_list_id)
```

This canonical form makes the key bidirectional: A->B and B->A produce the same key and share the same stored duration.

Use `HierarchyList.id` as the stable identifier - not a positional index (unit positions can shift on layout rebuild) and not `field_id` (one field can contain many list-to-list boundaries).

---

## AppState Changes

### Fields to add

```rust
// Pending transitions waiting to fire after the current in-flight transition completes.
// Drains FIFO. Grows only when a transition is triggered while another is in flight.
// Cleared on any full layout reset (open, data refresh, query changes that invalidate layout).
pub transition_queue: VecDeque<QueuedTransition>,

// Per-stub learned durations. Key: canonical stub key (sorted modal ID pair joined with |).
// Value: duration in ms. Absent keys resolve to theme.modal_transition_duration (the max).
// Loaded from transition_tuning.json at startup; saved on every modification.
pub transition_tuning: HashMap<String, u64>,
```

### Fields NOT added

No `sprint_mode` field. Adaptive tuning has no session-level mode switch.

---

## Theme Knob Additions (amends Part 1)

Add to `ThemeConfig`. These were not known at Part 1 planning time; add them alongside Part 1's theme changes or at Part 3 implementation if Part 1 is already done.

```rust
// Minimum duration a stub can reach through adaptive reduction. In ms.
// Default: 80
pub modal_transition_min_duration: u64,

// Amount to reduce a stub's stored duration each time it is trampled or queued. In ms.
// Default: 50
pub modal_transition_duration_decrement: u64,
```

`modal_transition_duration` (from Part 1) serves as both the maximum and the initial value for all stubs.

---

## Stub Duration Helpers

```rust
fn stub_key(modal_a_id: &str, modal_b_id: &str) -> String {
    if modal_a_id <= modal_b_id {
        format!("{}|{}", modal_a_id, modal_b_id)
    } else {
        format!("{}|{}", modal_b_id, modal_a_id)
    }
}

fn get_stub_duration(&self, key: &str) -> u64 {
    self.transition_tuning
        .get(key)
        .copied()
        .unwrap_or(self.theme.modal_transition_duration)
}

fn reduce_stub_duration(&mut self, key: &str) {
    let current = self.get_stub_duration(key);
    let min = self.theme.modal_transition_min_duration;
    let decrement = self.theme.modal_transition_duration_decrement;
    let new_val = current.saturating_sub(decrement).max(min);
    self.transition_tuning.insert(key.to_string(), new_val);
    self.save_transition_tuning();
}
```

---

## Stub Key Storage on Layers

Add `stub_key: String` to `ModalArrivalLayer` (or directly on `ConnectedTransition`). This field is set when the transition is created and is used to look up the in-flight stub when a trample occurs.

---

## Modified Transition Trigger

Replace the Part 2 trigger with this version:

```
fn trigger_transition(new_list_idx, focus_direction):
    arriving_unit_index = unit containing new_list_idx

    // Compute key for the new transition's stub
    left_modal_id  = last modal id of the lower-index unit in this transition
    right_modal_id = first modal id of the higher-index unit in this transition
    new_key = stub_key(left_modal_id, right_modal_id)

    if self.modal_transitions is non-empty:
        // Something is in flight. Reduce in-flight stub AND new stub.
        in_flight_key = stub_key stored on the in-flight ConnectedTransition
        self.reduce_stub_duration(in_flight_key)
        self.reduce_stub_duration(new_key)
        // Queue the visual transition; fire it after the current animation completes.
        self.transition_queue.push_back(QueuedTransition {
            arriving_unit_index,
            focus_direction,
        })
        // Focus moves immediately: the arriving unit is live and interactive from this frame.
        // The in-flight animation is not interrupted; it completes before the queued
        // transition fires. The visual system may lag behind focus, but interaction does not.
        self.active_unit_index = arriving_unit_index
        recompute prev/next prepared units
        return

    // Nothing in flight. Fire immediately.
    self.fire_transition(arriving_unit_index, focus_direction, new_key)

fn fire_transition(arriving_unit_index, focus_direction, key):
    duration_ms = self.get_stub_duration(key)
    // Capture frozen geometry and content snapshot as in Part 2 steps 4-6.
    // Set stub_key: key on the new ConnectedTransition.
    // Use duration_ms instead of theme.modal_transition_duration.
    // Push ConnectedTransition to self.modal_transitions.
    self.active_unit_index = arriving_unit_index
    recompute prev/next prepared units
```

---

## Modified Pruning Loop

After removing a completed transition, drain the queue:

```
match entry:
    ConnectedTransition { arrival, .. }:
        if arrival.progress >= 1.0:
            remove entry
            // Drain queue
            if let Some(next) = self.transition_queue.pop_front() {
                // Derive the stub key for the next transition
                left_modal_id  = last modal id of the left unit
                right_modal_id = first modal id of next.arriving_unit_index unit
                key = stub_key(left_modal_id, right_modal_id)
                self.fire_transition(next.arriving_unit_index, next.focus_direction, key)
            }
```

---

## Exit / Confirm Stub During Transition

When the user navigates via the exit (`-`) or confirm (`+`) stub while a transition is in flight:

1. Retrieve `in_flight_key` from the `ConnectedTransition`.
2. Call `self.reduce_stub_duration(in_flight_key)`.
3. Clear `self.modal_transitions`.
4. Clear `self.transition_queue`.
5. Close the modal immediately (existing behavior).

The queue is discarded because the field session is ending. The in-flight stub still receives its reduction because the user was impatient.

---

## Persistence

### File location

`self.data_dir.join("transition_tuning.json")`

This uses the same `data_dir` root as the rest of the app's local mutable state.

### Format

```json
{
  "DateDay|DateMonth": 250,
  "DateMonth|DateYear": 180
}
```

Keys are canonical sorted pairs; values are durations in ms. The file is written in full on every `reduce_stub_duration` call. The file is small; no debounce or batching is needed.

### Load

At app startup, read `transition_tuning.json` if it exists. On failure or absence, start with an empty map (all stubs resolve to theme max on first use).

### Reset keybinding

`+` key: clear `self.transition_tuning`, write `{}` to `transition_tuning.json`. All stubs return to `theme.modal_transition_duration` on next use.

Use the same gated pattern as `/` (refresh theme) and `\` (refresh data): check `AppKey::Char('+')` only when `self.modal.is_none() && !self.text_entry_active()`. Inside a modal, `+` continues to behave as normal text/hint input. Outside a modal, `+` triggers the reset.

---

## Execution Steps

1. Add `modal_transition_min_duration` and `modal_transition_duration_decrement` to `ThemeConfig` (amend Part 1 if not yet implemented)
2. Add `stub_key: String` to `ModalArrivalLayer` (or `ConnectedTransition`); set it in the transition trigger
3. Add `transition_queue: VecDeque<QueuedTransition>` and `transition_tuning: HashMap<String, u64>` to `AppState`
4. Implement `stub_key()`, `get_stub_duration()`, `reduce_stub_duration()`, and `save_transition_tuning()` helpers
5. Load `transition_tuning.json` at startup into `self.transition_tuning`
6. Modify the transition trigger to detect in-flight transitions, reduce both stubs, and enqueue
7. Extract `fire_transition()` from the existing trigger logic; use `get_stub_duration()` for `duration_ms`
8. Extend the pruning loop to drain the queue after completion
9. Handle exit/confirm stub during transition: reduce in-flight stub, clear transitions and queue, close modal
10. Add `+` keybinding for reset
11. Compile and verify all tests pass

---

## Out of Scope

- Per-stub maximum (all stubs share `theme.modal_transition_duration` as max)
- Duration decay or auto-reset over time
- Settings UI for viewing or editing per-stub durations
- Any visual indication of adaptive state

---

## Validation

### Automated

- `stub_key("A", "B") == stub_key("B", "A")` (bidirectionality)
- `get_stub_duration` returns `theme.modal_transition_duration` for unknown keys
- `reduce_stub_duration` decrements by `modal_transition_duration_decrement`, floors at `modal_transition_min_duration`
- Triggering while in flight: reduces in-flight stub AND new stub, adds to queue, does not create a second entry in `modal_transitions`
- Triggering when nothing is in flight: fires immediately, queue unmodified
- Pruning completion: pops one entry from queue and fires it
- Exit/confirm during transition: reduces in-flight stub, clears both `modal_transitions` and `transition_queue`
- Already-queued stubs are not re-reduced by subsequent triggers (Interpretation A: each stub reduced exactly once per rush event it participates in)

### Manual

1. **No stacking:** Navigate slowly, waiting for each transition to complete. Durations unchanged; all stubs stay at initial max speed.
2. **First trample:** Trigger a new transition during an in-flight one. Both stubs (in-flight and new) are reduced. The queued transition fires immediately after the first completes.
3. **Convergence:** Repeatedly navigate the same unit boundary with stacking. Duration converges toward `modal_transition_min_duration` over multiple sessions.
4. **Non-trampled stubs:** Stubs never trampled stay at max duration. Frequently-used boundaries become fast; infrequently-used ones stay slow.
5. **Bidirectional:** Navigate forward and backward through the same boundary. Both directions use the same stored duration.
6. **Exit during transition:** Press exit (`-`) mid-transition. Modal closes immediately. In-flight stub gets its reduction; next navigation through that stub is faster.
7. **Reset:** Press `+`. All transitions return to initial max speed immediately.
8. **Persistence:** Trample several stubs, close the app, reopen. Learned durations are preserved.

---

## Resolved Design Decisions

1. **Interpretation A for reductions:** when a trigger arrives while something is in flight, exactly two stubs are reduced: the in-flight one and the new one being queued. Stubs already waiting in the queue are not re-reduced by subsequent triggers. Each stub is reduced once per rush event it is directly involved in.

2. **Duration resolved at fire time:** `QueuedTransition` stores no duration. The duration is looked up from `transition_tuning` when the entry is popped and fired. This means a stub whose stored duration was reduced while it was waiting in the queue will fire at the lower value.

3. **No in-flight duration change:** the in-flight transition runs at the duration it was created with. Reducing `stored_duration` affects future transitions only, never the current one.

4. **`transition_tuning` in AppState:** loaded at startup, written on each modification. Writes are infrequent and the file is small; no batching needed.

5. **`+` reset keybinding:** `+` is the label on the confirm stub but is not a bound key. The `+` key is available for reset.

6. **Queued focus model (Option A - intentional):** when a transition is queued, focus and `active_unit_index` move immediately to the arriving unit. The arriving unit is interactive from frame 1. The visual animation for the in-flight transition completes before the queued one fires; the user may be interacting with a unit that is not yet visible in the animation. This is by design - advanced users can navigate rapidly and work ahead of the visuals unimpeded. Most users will naturally slow down when the relevant content is not yet on screen.

7. **Queue invalidation:** `transition_queue` is cleared on any full layout reset - this includes open (`open_header_modal`), data refresh, and any query/filter change that invalidates the layout. Resize does not clear the queue (it only updates geometry, not unit identity).

8. **Stable boundary IDs:** stub keys require stable per-modal identifiers on the frozen geometry. These must be added to `UnitGeometry` (fields carrying the boundary modal IDs on each side of a unit) before Part 3 implementation begins.

9. **Persistence path:** `self.data_dir.join("transition_tuning.json")`, consistent with the app's existing local mutable state root.

10. **`+` gating:** `+` reset follows the same pattern as `/` and `\` - only fires when `self.modal.is_none() && !self.text_entry_active()`. Inside a modal, `+` is ordinary text/hint input. `Shift+=` and numpad `+` both produce `AppKey::Char('+')` and behave identically.

---

## Review Addendum (2026-04-14)

### Findings

1. The queue model conflicts with the immediate-focus rule inherited from Part 2. This plan says queued transitions do not overlap, but it also says that when a new transition is queued during an in-flight one, focus and `active_unit_index` update immediately. That would make user input target a unit that is not yet visible.

2. `QueuedTransition` stores only `arriving_unit_index`. That index is not stable across resize, layout rebuilds, simple-mode loss, or any other event that can recompute `modal_unit_layout`. The plan does not currently say when the queue is cleared or revalidated.

3. The stub-key design does not yet line up with the current code shape. `SimpleModalSequence` / `ModalListViewSnapshot` do not currently carry a stable modal or list identifier, so the implementation cannot reliably derive canonical boundary keys from queued units or frozen transition data without adding explicit IDs first.

4. The persistence location is inconsistent with the current app. The running app currently loads and saves local mutable state via `data_dir` and `config.yml`; this plan introduces a new `{app_data_dir}/transition_tuning.json` path without tying it to the existing storage root.

5. The `+` reset key is not actually free in the current input model. Inside modal input paths, `AppKey::Char('+')` is just ordinary text / hint input, so binding reset to `+` would collide with normal typing.

### Recommended Changes

1. Resolve the queued-focus contract before implementation. Two workable options:
   - Option A: keep immediate interaction as the priority, and allow the visual system to create an immediate arrival representation for the queued step
   - Option B: keep the strict FIFO single-transition queue, but do not move focus until the queued transition actually begins
   - Recommendation: Option A fits the user requirement better; Option B is simpler but weakens the "interactive from frame 1" rule

2. Replace `QueuedTransition { arriving_unit_index, focus_direction }` with a queue entry that survives layout rebuilds, or explicitly clear the queue on any rebuild/invalidation event. As written, the queued target is too fragile.

3. Add an explicit stable boundary identifier to the simple modal data model before Part 3 depends on stub keys. The cleanest place is usually the simple-sequence snapshot / frozen geometry layer, not an ad hoc lookup at drain time.

4. Reconcile persistence with the app's existing storage model:
   - either store adaptive tuning alongside other user state in `config.yml`
   - or explicitly define it as `self.data_dir.join("transition_tuning.json")`

5. Replace the `+` reset binding with a non-text shortcut, or make it a configurable command instead of a printable character.

### Approval Status

Not yet. Part 3 still has a core interaction contradiction around queued focus, and it also needs stable queue identity, storage-path, and keybinding fixes before it is implementation-ready.

---

## Codex Review Addendum (2026-04-14)

### Findings

1. `QueuedTransition { arriving_unit_index, ... }` is not stable enough for this codebase. A pure window resize can repack `SimpleModalUnitLayout` into different unit boundaries, so the current resolved decision:
   - "Resize does not clear the queue (it only updates geometry, not unit identity)"

   is not correct. In this app, resize can change unit identity as well as geometry.

2. The stub-key section should name the actual stable identifier already present in the code: `HierarchyList.id`. Using a generic "modal ID" here is too vague and risks an incorrect implementation. `field_id` would be wrong for this purpose because one field can contain many list-to-list boundaries.

3. The core queue/focus contradiction is still present. The plan wants:
   - only one visual transition in flight
   - no overlapping layers
   - queued transitions drained later
   - focus and `active_unit_index` to move immediately when a later transition is queued

   Those rules do not fit together cleanly. They let interaction jump ahead of the visible strip, so the user can be editing a not-yet-visible unit while the UI is still animating an earlier one.

### Recommended Changes

1. Replace `QueuedTransition { arriving_unit_index, focus_direction }` with a queue entry that survives repacking:
   - store the stable boundary key
   - store stable target list identity (derived from `HierarchyList.id`) or enough data to resolve it again at fire time
   - if that is not adopted, clear the queue on resize as well as on other layout-invalidating events

2. Amend the Part 2 prerequisite and this Part 3 plan so boundary keys are explicitly derived from adjacent `HierarchyList.id` values, not modal titles or field IDs.

3. Pick one interaction contract before implementation:
   - strict single-transition FIFO with delayed focus movement
   - or immediate focus movement with a more complex visual model than this plan currently describes
   - Recommendation: if the "no overlapping layers" rule stays, delay focus until the queued transition actually starts

### Approval Status

Not yet. The queue identity and focus contract both still need to be resolved before implementation should begin.
