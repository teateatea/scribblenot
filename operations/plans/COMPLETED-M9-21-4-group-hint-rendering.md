# Plan: M9-21-4 — Group Hint Rendering: Always HINT Color When Map Focused

**Status:** Draft
**File:** `src/ui.rs` — `render_section_map`

---

## Problem

In `render_section_map`, the color of each group's hint label is computed at lines 111-123 of `src/ui.rs`. The current logic is:

```rust
let group_hint_color = if app.modal.is_some() {
    theme::MUTED
} else if !map_focused {
    if g_idx == current_group { theme::HINT } else { theme::MUTED }
} else {
    match &app.map_hint_level {
        MapHintLevel::Groups => theme::HINT,
        MapHintLevel::Sections(active_g) => {
            if *active_g == g_idx { theme::HINT } else { theme::MUTED }
        }
    }
};
```

When `map_hint_level` is `Sections(active_g)`, only the group whose index matches `active_g` is rendered in `HINT` color. All other group hints are `MUTED`. However, after ST 21.3, group hints are universally active: the input handler (lines 297-315 of `app.rs`) fires a group-jump at **any** `map_hint_level`, meaning group hints are always available when the map is focused. The dimming of non-active-group hints is therefore misleading.

The `hint_buffer` filtering path (lines 131-144) also only calls `hint_spans` when `group_hint_color == theme::HINT`, so dimmed groups never get partial-match highlighting. This is correct in principle, but since all groups should be lit, all should participate in `hint_spans`.

---

## Goal

When the map is focused, all group hint labels must render at `theme::HINT` regardless of `map_hint_level`. The `Sections` arm of the match must no longer dim non-active groups.

---

## Changes

### 1. `src/ui.rs` — `render_section_map`, group_hint_color block (lines 117-122)

**Before:**
```rust
        } else {
            match &app.map_hint_level {
                MapHintLevel::Groups => theme::HINT,
                MapHintLevel::Sections(active_g) => {
                    if *active_g == g_idx { theme::HINT } else { theme::MUTED }
                }
            }
        };
```

**After:**
```rust
        } else {
            // Group hints are always available when map is focused (universal group-jump).
            theme::HINT
        };
```

The entire `match &app.map_hint_level` block is replaced with a single `theme::HINT`. The `active_g` discrimination is removed.

No other changes are needed: the `group_hint_spans` branch at line 131 already uses `hint_spans` when `group_hint_color == theme::HINT` and `hint_buffer` is non-empty, so all group hints will automatically participate in partial-match highlighting once they are all `HINT` color.

---

## What Is NOT Changed

- Section hint color logic (lines 162-173) is unaffected — sections remain dim at `Groups` level and only the active group's sections light up at `Sections` level.
- Wizard mode (non-map-focused) behavior is unaffected.
- Modal behavior is unaffected.
- `app.rs` input handling is unaffected.

---

## Build Command

```bash
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" \
  /c/Users/solar/.cargo/bin/cargo.exe build \
  --manifest-path "/c/scribble/Cargo.toml"
```

---

## Manual Tests

1. Open map (focus Map). Verify ALL group hint labels appear in HINT color regardless of which group is current.
2. Type the first character of a multi-char group hint. Verify ALL group hints show partial-match highlighting (typed prefix in HINT, remainder in a dimmed/different style) — not just the active group.
3. Complete a group jump (type a full group hint). Verify the map cursor jumps to that group, `map_hint_level` transitions to `Sections(g_idx)`, and ALL group hints remain in HINT color.
4. With `map_hint_level` at `Sections`, type a different group's hint. Verify the jump fires (universal group-jump) and group hints remain HINT throughout.
5. Press Back to return to `Groups` level. Verify group hints still HINT.
6. Blur map (focus Wizard). Verify group hint behavior returns to wizard-mode logic (only current group's hint is HINT, others MUTED).
