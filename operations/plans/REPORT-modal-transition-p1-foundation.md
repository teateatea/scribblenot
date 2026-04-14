# Implementation Report: Modal Transition Foundation - Phase 1

**Date:** 2026-04-14
**Commit:** `b407e45`
**Plan:** `PLAN-modal-transition-p1-foundation.md`
**Status:** Complete - all tests passing (123/123)

---

## Summary

Phase 1 establishes the foundational layer for the modal transition system redesign. No transition animation behavior changed. The visible changes are: `-` and `+` stub symbols now appear at sequence boundaries (replacing blank space), and all stub colors are now independently themeable per kind.

---

## What Changed

### 1. Theme knob renames (`src/theme.rs`, `src/app.rs`, `data/default-theme.yml`)

| Old | New |
|---|---|
| `modal_stream_transition_duration_ms` | `modal_transition_duration` |
| `modal_stream_transition_easing` | `modal_transition_easing` |
| `ModalStreamEasing` (enum) | `ModalTransitionEasing` |
| `parse_modal_stream_transition_easing()` | `parse_modal_transition_easing()` |

The enum and its variants live in `src/app.rs` and are referenced by `src/theme.rs`. All call sites in `start_modal_stream_transition()` updated. The theme test was renamed and its YAML fixture strings updated.

### 2. Six new stub color knobs (`src/theme.rs`, `data/default-theme.yml`)

Added to `AppTheme`, `ThemeFile`, `default()`, `from_file()`, and `default-theme.yml`:

```
modal_nav_stub_background / modal_nav_stub_text
modal_exit_stub_background / modal_exit_stub_text
modal_confirm_stub_background / modal_confirm_stub_text
```

All six default to `pane-blue` / `#E6E6E6`, matching the previous `modal_stub_background` behavior. The existing `modal_stub_background` field is kept for backwards compatibility.

### 3. Spacer and bounding-box helpers (`src/modal.rs`)

Two new public free functions:

```rust
pub fn effective_spacer_width(viewport_width: f32, modal_spacer_width: f32) -> f32
pub fn unit_bounding_box(viewport_width: f32, stub_width: f32, spacer_width: f32) -> f32
```

`build_simple_modal_unit_layout` now owns the viewport-cap math - it takes raw theme spacer width and calls `effective_spacer_width` itself. Previously callers passed a pre-capped value; now `App::simple_modal_unit_layout_for()` passes `self.ui_theme.modal_spacer_width` (raw) and `build_simple_modal_unit_layout` applies the cap.

`App::modal_spacer_width()` (used for rendering spacers between cards) was updated to call `effective_spacer_width` instead of inlining the formula.

Four new tests validate the helper contracts from the plan's validation section.

### 4. `ModalStubKind` enum (`src/modal.rs`)

```rust
pub enum ModalStubKind { NavLeft, NavRight, Exit, Confirm }
```

Includes a `symbol() -> char` method returning `< > - +` respectively.

Assignment rules - given a unit at snapshot range `[start, end]` within a sequence of `n` total snapshots:

| Side | Condition | Kind |
|---|---|---|
| Left | `start == 0` | Exit |
| Left | `start > 0` | NavLeft |
| Right | `end + 1 >= n` | Confirm |
| Right | `end + 1 < n` | NavRight |

### 5. Stub rendering overhaul (`src/ui.rs`)

**`ModalUnitStubMode`**: `Visible(char)` simplified to `Visible`. The char was redundant once kind carries the symbol. All construction sites updated; `default_stub_mode` now ignores the side parameter and returns `Visible`.

**`ModalUnitCardKind::Stub`**: Added `stub_kind: ModalStubKind` field alongside the existing `side` and `mode` fields.

**`ModalCardRole::Stub`**: Changed from a unit variant to `Stub(ModalStubKind)`. `modal_card_style` now selects background color per kind.

**`build_rendered_modal_unit`**: Stub conditions changed from `shows_stubs && start > 0` / `shows_stubs && end + 1 < len` to simply `shows_stubs`. Exit and Confirm stubs now appear at sequence boundaries (previously those edges had no stub even when `shows_stubs` was true). Kind is computed from unit position and stored in the card.

**`render_modal_unit`**: Stub rendering now uses `stub_kind.symbol()` for the displayed character and selects text color from the per-kind theme fields.

**Packed stream stubs** (dead code path): `LeftStub` and `RightStub` card types updated to `Stub(NavLeft)` and `Stub(NavRight)` respectively to satisfy the new `ModalCardRole` signature.

---

## Test coverage added

| Test | Location | Validates |
|---|---|---|
| `effective_spacer_width_caps_at_two_percent_of_viewport` | `modal_sizing_tests` | 1000px -> 20, 500px -> 10, 100px -> 2 |
| `effective_spacer_width_does_not_exceed_theme_value` | `modal_sizing_tests` | 3000px with 40 cap -> 40 |
| `unit_bounding_box_subtracts_both_stubs_and_spacers` | `modal_sizing_tests` | 1200 - 2*(120+20) = 920 |
| `unit_bounding_box_clamps_to_zero_on_narrow_viewport` | `modal_sizing_tests` | 50px narrow -> 0 |

Renamed: `modal_stream_transition_duration_defaults_and_overrides` -> `modal_transition_duration_defaults_and_overrides`

---

## Files modified

| File | Nature of change |
|---|---|
| `src/app.rs` | Rename enum + update theme field references |
| `src/theme.rs` | Rename fields, add 6 new fields, rename parser fn, update test |
| `src/ui.rs` | Stub kind integration, rendering overhaul |
| `src/modal.rs` | Two helper functions, ModalStubKind enum, build fn update, 4 tests |
| `data/default-theme.yml` | Rename 2 keys, add 6 new keys |

---

## Out of scope (deferred to Parts 2 and 3)

- Transition state, lifecycle, and animation changes
- Adaptive transition behavior
- Composition panel and collection-mode layout
- Removing `modal_stub_background` (kept for backwards compat)
