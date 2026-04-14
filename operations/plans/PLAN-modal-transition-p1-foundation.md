# Plan: Modal Transition Redesign - Part 1: Foundation

**Date:** 2026-04-14
**Status:** Proposed
**Part:** 1 of 3
**Supersedes:** `PLAN-modal-unit-transition-redesign.md` (in combination with Parts 2 and 3)
**Prerequisite for:** Part 2 (state model and lifecycle), Part 3 (adaptive transitions)

---

## Purpose

Establish the foundational layer required before the transition system can be rebuilt:
- Fix the spacer width formula to use a viewport-percentage cap
- Rename existing transition theme knobs for consistency
- Add stub color theme knobs
- Migrate `data/default-theme.yml`
- Define three stub types (nav, exit, confirm) and wire them to per-type colors
- Export spacer and bounding-box helpers so render code and layout code share the same math

This phase produces no visible behavior change beyond stub symbols and colors becoming correct. Transition behavior is unchanged.

---

## Shared Vocabulary

These terms are used throughout all three parts. Use them consistently in code, comments, and docs.

| Term | Definition |
|---|---|
| modal unit / unit | a group of modals visible at the same time |
| focus | the modal with the cursor and yellow highlight |
| active unit | the visible unit that contains focus |
| prepared unit | a unit tracked in state, positioned off-screen at its arrival location; not visible, no hotkeys, but focus can land there immediately |
| transition | focus leaving the active unit and landing in a prepared unit |
| departing unit | the previously active unit, fading and sliding out |
| arriving unit | the prepared unit that just received focus, fading and sliding in |
| transition direction | opposite the focus movement (focus moves right = transition direction is left) |
| transition stub | the stub between departing and arriving units; stays full opacity throughout |
| spacer | gap between cards; `min(viewport_width * 0.02, modal_spacer_width)` |
| nav stub | `<` or `>` stub indicating an adjacent navigable unit |
| exit stub | `-` stub on the left of the first unit; "leave without adding to note" |
| confirm stub | `+` stub on the right of the last unit; "add to note / complete field" |
| unit bounding box | `viewport_width - 2 * (modal_stub_width + effective_spacer_width)` |
| shows_stubs | per-unit flag; false when the first modal exceeds the bounding box; stubs are omitted entirely |

---

## 1. Theme Knob Renames

### In `src/theme.rs`

| Old name | New name | Type | Notes |
|---|---|---|---|
| `modal_stream_transition_duration_ms` | `modal_transition_duration` | `u64` | value is in milliseconds |
| `modal_stream_transition_easing` | `modal_transition_easing` | `ModalTransitionEasing` | |

### In `src/app.rs`

| Old name | New name | Notes |
|---|---|---|
| `ModalStreamEasing` (enum) | `ModalTransitionEasing` | rename all variants, match arms, and imports |

Update all references to `ModalStreamEasing` / `ModalTransitionEasing` in `src/app.rs`, `src/ui.rs`, and any other file that uses them. The project must compile cleanly after renames before proceeding.

### In `data/default-theme.yml`

Rename:
```yaml
# Before
modal_stream_transition_duration_ms: 200
modal_stream_transition_easing: expo_out

# After
modal_transition_duration: 200
modal_transition_easing: expo_out
```

There is only one theme file (`data/default-theme.yml`). No other `.yml` files in `data/` are theme files.

---

## 2. New Theme Knobs

### Stub colors

Add to `AppTheme`:

```rust
pub modal_nav_stub_background: Color,     // Default: pane-blue (#141821), matches existing modal_stub_background
pub modal_nav_stub_text: Color,           // Default: #E6E6E6, matches modal_text
pub modal_exit_stub_background: Color,    // Default: pane-blue
pub modal_exit_stub_text: Color,          // Default: #E6E6E6
pub modal_confirm_stub_background: Color, // Default: pane-blue
pub modal_confirm_stub_text: Color,       // Default: #E6E6E6
```

The existing `modal_stub_background` field is kept for backwards compatibility during this transition. It is not removed in this part. If the new per-kind stub colors are absent in older theme data, they resolve to the previous `modal_stub_background` / `modal_text` behavior. Once Parts 2 and 3 are complete, decide whether `modal_stub_background` should remain as a deprecated alias or be removed in a later cleanup plan.

Add to `data/default-theme.yml`:
```yaml
modal_nav_stub_background: pane-blue
modal_nav_stub_text: "#E6E6E6"
modal_exit_stub_background: pane-blue
modal_exit_stub_text: "#E6E6E6"
modal_confirm_stub_background: pane-blue
modal_confirm_stub_text: "#E6E6E6"
```

---

## 3. Spacer and Bounding-Box Helpers

### Problem with current code

Spacer-width capping already exists at some call sites (notably `App::modal_spacer_width()`). The goal of Part 1 is to centralize the spacer and bounding-box math in shared helpers so layout code and render code cannot drift apart in future changes.

### Required formula

```
effective_spacer_width = min(viewport_width * 0.02, modal_spacer_width)
```

### Implementation

Add to `src/modal.rs` as public free functions:

```rust
/// Returns the spacer width for a given viewport axis dimension.
/// Caps at 2% of the viewport to prevent oversized gaps on wide displays.
pub fn effective_spacer_width(viewport_width: f32, modal_spacer_width: f32) -> f32 {
    (viewport_width * 0.02).min(modal_spacer_width)
}

/// Returns the bounding box available for full-width modals within a unit.
/// Reserves space for one stub and one spacer on each side.
/// Clamped to zero so extremely narrow viewports never produce a negative content limit.
pub fn unit_bounding_box(viewport_width: f32, stub_width: f32, spacer_width: f32) -> f32 {
    (viewport_width - 2.0 * (stub_width + spacer_width)).max(0.0)
}
```

Both functions accept primitive arguments so they can be called from render code without needing to borrow a full theme reference.

### API boundary

`build_simple_modal_unit_layout(...)` is the owner of layout-space math. It accepts the raw theme spacer width (not already-resolved) and calls `effective_spacer_width(...)` itself. Callers (`SearchModal::simple_modal_unit_layout(...)` and `App::simple_modal_unit_layout_for()`) must be updated to pass the raw theme spacer width instead of the pre-capped value. This eliminates the risk of double-applying the cap.

### Call-site updates

- `App::modal_spacer_width()`: replace the inline formula with a call to `effective_spacer_width(...)`.
- `App::simple_modal_unit_layout_for()` / `SearchModal::simple_modal_unit_layout(...)`: pass raw theme spacer width; do not pre-cap.
- `build_simple_modal_unit_layout(...)`: compute its own effective spacer from the viewport width and raw theme spacer width using the helper.
- Any UI-side bounding-box or spacer math in `src/ui.rs`: replace with calls to the helpers.

---

## 4. Stub Types

### Enum

Add to `src/modal.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalStubKind {
    NavLeft,    // "<" - navigate to previous unit
    NavRight,   // ">" - navigate to next unit
    Exit,       // "-" - leave field without adding to note
    Confirm,    // "+" - add to note / complete field
}
```

### Assignment rules

Given a layout with `n` units, for unit at index `i` with `shows_stubs: true`:

| Side | Condition | Kind |
|---|---|---|
| Left | `i == 0` | `Exit` |
| Left | `i > 0` | `NavLeft` |
| Right | `i == n - 1` | `Confirm` |
| Right | `i < n - 1` | `NavRight` |

For a unit with `shows_stubs: false`: no stubs are rendered. No stub width is reserved.

Single-unit sequences: left stub is `Exit`, right stub is `Confirm`.

### Rendering

Update stub rendering in `src/ui.rs`:

| Kind | Symbol | Background | Text color |
|---|---|---|---|
| `NavLeft` | `<` (centered) | `modal_nav_stub_background` | `modal_nav_stub_text` |
| `NavRight` | `>` (centered) | `modal_nav_stub_background` | `modal_nav_stub_text` |
| `Exit` | `-` (centered) | `modal_exit_stub_background` | `modal_exit_stub_text` |
| `Confirm` | `+` (centered) | `modal_confirm_stub_background` | `modal_confirm_stub_text` |

All four kinds use `modal_stub_width` for their width. The symbol must be centered both horizontally and vertically within the stub card.

---

## 5. Execution Steps

Work in this order. Compile after each step to catch errors early.

1. **`src/theme.rs`**
   - Rename `modal_stream_transition_duration_ms` to `modal_transition_duration`
   - Rename `modal_stream_transition_easing` to `modal_transition_easing`
   - Rename matching fields in `ThemeFile` (the deserialized YAML struct)
   - Rename any easing parser or helper that references the old field names
   - Update existing theme tests that assert the old YAML key names
   - Add six stub color fields with defaults as specified above

   **`src/app.rs`**
   - Rename `ModalStreamEasing` to `ModalTransitionEasing` (all variants, match arms, and imports)

2. **Fix compile errors** from renames in `src/app.rs`, `src/ui.rs`, and any other files

3. **`data/default-theme.yml`**
   - Rename `modal_stream_transition_duration_ms` to `modal_transition_duration`
   - Rename `modal_stream_transition_easing` to `modal_transition_easing`
   - Add six stub color entries

4. **`src/modal.rs`**
   - Add `effective_spacer_width` free function
   - Add `unit_bounding_box` free function
   - Add `ModalStubKind` enum
   - Update `build_simple_modal_unit_layout` to call `effective_spacer_width`

5. **`src/ui.rs`**
   - Update stub rendering to use `ModalStubKind`
   - Use per-kind background and text colors
   - Replace any independent spacer/bounding-box calculations with calls to the helpers

6. **Compile and run** - verify no regressions

---

## 6. Out of Scope

- Changes to transition state, lifecycle, or animation (Part 2)
- Adaptive transition behavior (Part 3)
- Composition panel, collection-mode layout, checksum-based refresh skipping

---

## 7. Validation

### Automated

- `effective_spacer_width(1000.0, 40.0)` returns `20.0` (2% of 1000 is less than 40)
- `effective_spacer_width(500.0, 40.0)` returns `10.0` (2% of 500 is less than 40)
- `effective_spacer_width(100.0, 40.0)` returns `2.0` (2% of 100)
- `unit_bounding_box(1200.0, 120.0, 20.0)` returns `1200.0 - 2 * (120.0 + 20.0)` = `920.0`

Run `cargo test` after each execution step that touches `src/theme.rs` or `data/default-theme.yml`. This catches stale theme key assertions (e.g. tests still asserting old YAML key names) immediately rather than at final compile.

### Manual

1. Open the app. The leftmost visible stub shows `-` and the rightmost shows `+`. All nav stubs between units show `<` or `>`.
2. All stub symbols are centered horizontally and vertically within their cards.
3. Set `modal_nav_stub_background` to a visually distinct color in `default-theme.yml` and confirm only nav stubs change; exit and confirm stubs are unaffected.
4. Set `modal_exit_stub_background` and `modal_confirm_stub_background` to distinct colors and confirm each applies correctly.
5. Resize the window to narrow width. Confirm spacers do not exceed 2% of the viewport width (visually, gaps between modals should shrink on narrow windows).
6. Transition behavior (animation, timing, direction) is unchanged from before this part.

---

## Resolved Design Decisions

1. **`unit_bounding_box` nonnegative clamp:** incorporated `.max(0.0)` directly into the helper definition in section 3. Matches existing behavior in `build_simple_modal_unit_layout` and prevents negative content limits on narrow viewports.

---

## Superseded Review Notes (2026-04-14)

The two earlier addenda that were previously here have been superseded by the main plan text and the `Codex Review Addendum (2026-04-14)` below.

Those earlier notes covered:
- correcting the enum/file wording around `ModalStreamEasing`
- centralizing spacer math and documenting the fallback behavior for `modal_stub_background`
- removing stale sprint-specific references and updating the Part 3 prerequisite wording

Those issues have now been incorporated into the main body of this plan, so the older "not yet" approval note is no longer current.

---

## Codex Review Addendum (2026-04-14)

### Findings

1. The spacer-helper ownership is still slightly ambiguous in the current plan. Today the call chain is:
   - `App::modal_spacer_width()`
   - `App::simple_modal_unit_layout_for()`
   - `SearchModal::simple_modal_unit_layout(...)`
   - `build_simple_modal_unit_layout(...)`

   Right now `build_simple_modal_unit_layout(...)` receives an already-resolved spacer width. Part 1 says that function should compute its own effective spacer, but the plan does not explicitly say which signatures change. Without that clarification, implementation can easily end up double-applying the cap or keeping duplicate math.

2. The theme-rename section covers the public field names, but it does not explicitly call out the full loader/test surface in `src/theme.rs`. The implementation also has to rename:
   - the deserialized YAML fields in `ThemeFile`
   - the easing parser helper naming
   - the existing theme tests that assert the old key names

### Recommended Changes

1. Add an explicit API-change note in section 3:
   - either `build_simple_modal_unit_layout(...)` now takes raw theme spacer width and computes the effective spacer itself
   - or callers continue passing the effective spacer and the helper is only used at call sites
   - Recommendation: make `build_simple_modal_unit_layout(...)` the owner of layout-space math, and update `SearchModal::simple_modal_unit_layout(...)` / `App::simple_modal_unit_layout_for()` to pass raw theme spacer width

2. Expand execution steps 1-4 to explicitly include:
   - `ThemeFile` rename updates
   - easing parser/helper rename updates
   - `src/theme.rs` test updates for the renamed YAML keys

3. In validation, add one compile-and-test checkpoint, not just compile:
   - `cargo test`
   - this catches stale theme key assertions immediately

### Approval Status

Approved after the doc is amended with the API-boundary and theme-test details above. I do not see a remaining design blocker in Part 1.
