# Plan: M9-21-3 Universal Group Jump from Any Hint Level

## Status: Draft

## Goal
In `handle_map_key`, add a universal group-jump check that runs **before** the `match self.map_hint_level` block. When the typed hint buffer matches any group hint (regardless of current `map_hint_level`), immediately jump `map_cursor` to the first section of that group, set `map_hint_level = Sections(g_idx)`, and clear `hint_buffer`. This replaces the old "parent hint toggles back to Groups" behavior in the `Sections` branch, and removes the now-redundant group-hint resolution from the `Groups` branch.

## Context

### Current behavior (after ST 21.2)
`handle_map_key` (lines 280-354 of `src/app.rs`) does the following on `Char` key press:

1. Builds `folded_refs` from `combined_hints`.
2. Matches on `self.map_hint_level`:
   - **Groups branch** (lines 299-315): resolves against the first `n_groups` hints. On `Exact(g_idx)`, sets `map_hint_level = Sections(g_idx)` and clears buffer. Does NOT navigate `map_cursor`.
   - **Sections branch** (lines 316-352): first checks whether `typed == parent_hint` (the group hint for the current group) and if so toggles back to `MapHintLevel::Groups`. Then resolves against the section hints for the current group, and on `Exact(s_idx)` navigates to that section.

### Target behavior
- Before the `match` block, run a universal group-jump check against the first `n_groups` folded hints.
  - `Exact(g_idx)`: jump `map_cursor` to `group_jump_target(&self.data.groups, g_idx)`, set `map_hint_level = MapHintLevel::Sections(g_idx)`, clear `hint_buffer`, return early.
  - `Partial(_)`: hold the buffer (return early without proceeding).
  - `NoMatch`: fall through to the per-level `match` block.
- Because group navigation is fully handled universally:
  - The `Groups` branch can be simplified to a no-op / just clear on NoMatch (the `Exact` case is now unreachable since the universal check returns early).
  - The `Sections` branch: remove the "parent hint toggles back to Groups" logic (lines 318-327). The parent group hint now triggers the universal check above instead, jumping to the group's first section rather than returning to Groups mode.

### Functions used
- `crate::data::combined_hints(&self.data.keybindings)` - returns full ordered hint list
- `crate::data::resolve_hint(hints, typed)` - returns `Exact(i)`, `Partial(v)`, or `NoMatch`
- `crate::data::group_jump_target(&self.data.groups, g_idx)` - returns flat index of first section in group g_idx

## Steps

### Step 1: Add universal group-jump block before `match hint_level`
In `src/app.rs`, inside `handle_map_key`, after the `let hint_level = ...` line (line 297) and before `match hint_level {`, insert the universal group-jump block:

```rust
// Universal group-jump: check typed against group hints regardless of current level
let n_groups = self.data.groups.len();
let group_refs: Vec<&str> = folded_refs.iter().take(n_groups).copied().collect();
match crate::data::resolve_hint(&group_refs, &typed) {
    crate::data::HintResolveResult::Exact(g_idx) => {
        let target = crate::data::group_jump_target(&self.data.groups, g_idx);
        self.map_cursor = target;
        self.map_hint_level = MapHintLevel::Sections(g_idx);
        self.hint_buffer.clear();
        self.update_note_scroll();
        return;
    }
    crate::data::HintResolveResult::Partial(_) => {
        // Buffer held - wait for more input
        return;
    }
    crate::data::HintResolveResult::NoMatch => {
        // Fall through to per-level resolution
    }
}
```

### Step 2: Simplify the Groups branch
The `Groups` branch (lines 299-315) can now be reduced. Since all `Exact` group matches are handled by the universal check above (which returns early), this branch is only reached on `NoMatch` from the universal check. The `Partial` case from the universal check also returns early. So the entire `Groups` branch body can be replaced with a simple `{}` (empty arm, or just clear on NoMatch):

```rust
MapHintLevel::Groups => {
    // Group-exact and group-partial are handled by the universal check above.
    // If we reach here, typed has no match among group hints.
    self.hint_buffer.clear();
}
```

### Step 3: Remove parent-hint toggle from Sections branch
In the `Sections` branch (lines 316-327), delete the "parent hint toggles back to Groups" block:

```rust
// DELETE these lines:
let parent_hint = folded_refs.get(g_idx).copied().unwrap_or("");
if typed == parent_hint {
    self.map_hint_level = MapHintLevel::Groups;
    self.hint_buffer.clear();
    return;
}
if parent_hint.starts_with(typed.as_str()) && typed.len() < parent_hint.len() {
    // Partial match toward parent hint - hold buffer
    return;
}
```

The parent group hint now fires the universal group-jump check in Step 1, so these lines are redundant and incorrect (they would toggle to Groups instead of jumping).

After deletion, the `Sections` branch retains only the section-hint resolution logic (lines 329-351):

```rust
MapHintLevel::Sections(g_idx) => {
    let n_groups = self.data.groups.len();
    let group_start: usize = self.data.groups.iter().take(g_idx).map(|g| g.sections.len()).sum();
    let group_len = self.data.groups.get(g_idx).map(|g| g.sections.len()).unwrap_or(0);
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
        crate::data::HintResolveResult::Partial(_) => {}
        crate::data::HintResolveResult::NoMatch => {
            self.hint_buffer.clear();
        }
    }
}
```

Note: the `n_groups` binding in this branch is now a duplicate of the one computed for the universal check above. Hoist it to avoid recomputation (or leave it; the compiler will deduplicate). The cleaner approach is to compute `n_groups` once before the universal check and reuse it in both places.

### Step 4: Hoist `n_groups` binding (cleanup)
Move `let n_groups = self.data.groups.len();` to just before the universal group-jump block (before the `group_refs` slice), and remove the redundant `let n_groups = ...` inside the `Sections` branch.

### Step 5: Build and verify
```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe build --manifest-path "/c/scribble/Cargo.toml"
```
Expected: zero errors, zero new warnings.

## Manual Tests

1. **Group jump from Groups mode**: Open the map. Press a group hint key. Confirm `map_cursor` jumps to the first section of that group and hint level switches to `Sections(g_idx)`.

2. **Group jump from Sections mode**: While in `Sections` mode for group 1, press a different group's hint key. Confirm `map_cursor` jumps to the first section of the new group and hint level switches to `Sections(new_g_idx)`.

3. **Group jump to same group from Sections mode**: While in `Sections(0)`, press group 0's hint key. Confirm `map_cursor` jumps to the first section of group 0 (flat index 0), hint level stays `Sections(0)`.

4. **Partial group hint holds buffer**: If a group hint is multi-character (e.g. `"qq"`), typing the first character alone holds the buffer (Partial) without navigating. Typing the second character completes the jump.

5. **Section hints still work**: In `Sections(g_idx)` mode, press a section hint for a section in that group. Confirm navigation to that section and focus returns to Wizard.

6. **NoMatch clears buffer**: Typing a character that matches no group hint and no section hint in the current level clears `hint_buffer` cleanly.

7. **Up/down arrow navigation unaffected**: Arrow keys still navigate sections and update `map_hint_level` to `Sections(current_group)` as before.
