# Plan: M9-21-2 Section Hint Exclusion

**Status:** Proposed

## Problem

Section hints are currently assigned by skipping only the active group's hint index (`g_idx`). With 3 groups (hints at indices 0, 1, 2) and group 1 active, sections receive hints at indices 0, 2, 3, 4... This lets sections claim hint slots reserved for other groups, creating collisions where a typed char could be ambiguous between a group hint and a section hint.

## Required Behavior

All `n_groups` hint indices (0..n_groups-1) are reserved for groups. Sections must start at index `n_groups`. With 3 groups: sections always get hints at indices 3, 4, 5, 6... regardless of which group is active.

## Changes

### 1. `section_hint_key_idx` in `src/app.rs` (line 372)

**Current logic** (lines 375-379):
```rust
for (g_idx, group) in self.data.groups.iter().enumerate() {
    for s_idx in 0..group.sections.len() {
        if fi == flat_idx {
            let nth = (0..hints.len()).filter(|&i| i != g_idx).nth(s_idx);
            return nth;
```

The filter skips only `g_idx`, so `s_idx=0` for group 0 gets hint 1, for group 1 gets hint 0, etc.

**New logic**: Compute a cumulative section index across all groups, then offset by `n_groups`.

```rust
pub fn section_hint_key_idx(&self, flat_idx: usize) -> Option<usize> {
    let hints = crate::data::combined_hints(&self.data.keybindings);
    let n_groups = self.data.groups.len();
    let mut fi = 0usize;
    for group in self.data.groups.iter() {
        for s_idx in 0..group.sections.len() {
            if fi == flat_idx {
                return hints.get(n_groups + s_idx).map(|_| n_groups + s_idx);
            }
            fi += 1;
        }
    }
    None
}
```

Wait - `s_idx` resets per group but we need the cumulative flat section index offset by `n_groups`. Use `flat_idx` directly:

```rust
pub fn section_hint_key_idx(&self, flat_idx: usize) -> Option<usize> {
    let hints = crate::data::combined_hints(&self.data.keybindings);
    let n_groups = self.data.groups.len();
    let hint_idx = n_groups + flat_idx;
    if hint_idx < hints.len() {
        Some(hint_idx)
    } else {
        None
    }
}
```

This is correct: `flat_idx` is already the 0-based index across all sections, so `n_groups + flat_idx` gives the correct hint slot.

### 2. `handle_map_key` Sections branch in `src/app.rs` (lines 316-353)

**Current logic** (lines 329-337): builds `section_hints` by filtering out `g_idx`, then slices to `group_len` and resolves `s_idx` against that list.

**New logic**: section hints start at index `n_groups`. The resolved `s_idx` within the group is `flat_idx - group_start`. Match typed char against hints at `n_groups + group_start .. n_groups + group_start + group_len`.

Replace lines 329-353 with:

```rust
let n_groups = self.data.groups.len();
let group_start: usize = self.data.groups.iter().take(g_idx).map(|g| g.sections.len()).sum();
let group_len = self.data.groups.get(g_idx).map(|g| g.sections.len()).unwrap_or(0);
// Section hints start at n_groups; sections in this group are at n_groups+group_start..
let section_refs: Vec<&str> = folded_refs
    .iter()
    .skip(n_groups + group_start)
    .take(group_len)
    .copied()
    .collect();
match crate::data::resolve_hint(&section_refs, &typed) {
    crate::data::HintResolveResult::Exact(s_idx) => {
        let flat_idx = group_start + s_idx;
        self.current_idx = flat_idx;
        self.map_cursor = flat_idx;
        self.focus = Focus::Wizard;
        self.map_hint_level = MapHintLevel::Groups;
        self.hint_buffer.clear();
    }
    crate::data::HintResolveResult::Partial(_) => {
        // Wait for more input
    }
    crate::data::HintResolveResult::NoMatch => {
        self.hint_buffer.clear();
    }
}
```

### 3. `render_section_map` in `src/ui.rs` (lines 149-153)

**Current logic**: builds `section_hints` per group by filtering out `g_idx`. Section `si` within the group gets `section_hints[si]`.

**New logic**: section hints start at `n_groups`. The flat section index for group `g_idx`, section `si` is `group_start + si`. So hint index is `n_groups + group_start + si`.

Replace lines 149-162 with:

```rust
// Section hints: all start at n_groups offset (no per-group exclusion needed)
let n_groups = app.data.groups.len();
let group_start: usize = app.data.groups.iter().take(g_idx).map(|g| g.sections.len()).sum();

for (si, section) in group.sections.iter().enumerate() {
    // ... existing is_current, is_map_cursor, etc. ...
    let section_hint_raw = hints.get(n_groups + group_start + si).copied().unwrap_or(" ");
```

The `section_hints` Vec construction (lines 150-153) is removed entirely; the hint is looked up directly by computed index.

## Files to Edit

- `src/app.rs`: `section_hint_key_idx` function, `handle_map_key` Sections branch
- `src/ui.rs`: `render_section_map` section hint assignment

## Build Command

```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe build --manifest-path "/c/scribble/Cargo.toml"
```

## Manual Tests

1. With 3 groups having 2 sections each:
   - Hints 0..2 = group hints (e.g. a, b, c)
   - Sections of group 0 get hints 3, 4 (not 1, 2)
   - Sections of group 1 get hints 5, 6 (not 0, 2)
   - Sections of group 2 get hints 7, 8 (not 0, 1)
2. In map mode: typing a group hint, then a section hint navigates to the correct section.
3. In wizard mode: section hint displayed matches the hint that triggers navigation.
4. No hint collision: typing a group hint char never accidentally triggers a section.
