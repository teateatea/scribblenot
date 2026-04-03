## Task

Add `hint_buffer: String` to `App` and wire up the prefix-matching state machine in `handle_map_key`. Single-char hints that resolve exactly on first keypress are unaffected. Multi-char hints accumulate in the buffer until `resolve_hint` returns `Exact`. `try_navigate_to_map_via_hint` and `handle_header_key` also need the same buffer treatment so hints there match full strings.

## Context

`resolve_hint` (data.rs lines 418-425) already handles the three-way result: `Exact(usize)`, `Partial(Vec<usize>)`, `NoMatch`. `combined_hints` (data.rs lines 389-393) gives the full ordered hint slice. The map uses a two-level hint scheme (`MapHintLevel::Groups` / `MapHintLevel::Sections(g_idx)`); currently `handle_map_key` (app.rs lines 274-326) matches a single typed char directly against each hint string. `try_navigate_to_map_via_hint` (lines 1143-1175) and `handle_header_key` (lines 490-542) both also do single-char comparisons. None of them handle multi-char hints today.

`App` struct is defined at lines 62-79. `App::new` initialises all fields at lines 101-123.

## Approach

1. Add `pub hint_buffer: String` to `App` (init to `String::new()`).
2. Rewrite the hint-key branch of `handle_map_key` to:
   - Extract the typed char, apply case folding.
   - Append to `self.hint_buffer`.
   - Call `resolve_hint` against the context-appropriate hint list.
   - `Exact(idx)` -> navigate as before, then `self.hint_buffer.clear()`.
   - `Partial(_)` -> do nothing (hold buffer, wait for next key).
   - `NoMatch` -> `self.hint_buffer.clear()` (reset silently).
3. Clear `hint_buffer` in all state-change paths: navigation (down/up/confirm/back in `handle_map_key`), Esc (`is_back`), and focus-switch returns to Wizard in `handle_key`.
4. Update `try_navigate_to_map_via_hint` to use the buffer similarly (append char, call `resolve_hint`, Exact triggers navigation + clear, Partial holds, NoMatch clears and returns false).
5. Update `handle_header_key` hint comparison identically.
6. In `handle_modal_key` (lines 731-737) the hint dispatch is already single-char by design (hint positions in the visible window are single-char indices), so no change needed there.

### Why `hint_buffer` on `App` (not local)

The buffer must survive across successive `handle_key` calls (one per keypress). Local variables would reset every call.

### Case sensitivity

Apply the same `config.hint_labels_case_sensitive` folding on the appended char before building the buffer string. All hints in `combined_hints` are stored as-is; the buffer must be folded to the same case as the comparison hint strings (lowercase when not case-sensitive).

### Clearing policy

Clear on: any non-Char key in the hint path, Esc, confirm, back, navigate-up/down, mode change (focus switch). For `try_navigate_to_map_via_hint` return false + clear on NoMatch; return true + clear on Exact; return false without clearing on Partial (waiting for more input).

## Critical Files

- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/app.rs` - all changes
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/data.rs` - read-only reference (`resolve_hint`, `combined_hints`, `HintResolveResult`)

## Reuse

`resolve_hint` and `combined_hints` from data.rs are used directly; no new functions needed.

## Steps

### Step 1 - Add `hint_buffer` field to `App` struct

In `app.rs` lines 62-79, add one field after `modal`:

```rust
pub struct App {
    // ... existing fields ...
    pub modal: Option<SearchModal>,
    pub hint_buffer: String,
}
```

### Step 2 - Initialise `hint_buffer` in `App::new`

In the `Self { ... }` block (lines 105-122), add:

```rust
hint_buffer: String::new(),
```

### Step 3 - Clear `hint_buffer` in `handle_map_key` navigation branches

In `handle_map_key`, every branch that returns early after navigation should clear the buffer. Add `self.hint_buffer.clear();` at the top of:
- `is_navigate_down` branch
- `is_navigate_up` branch
- `is_confirm` branch
- `is_back` branch

These are lines 243-271. The clear goes before the `return`.

### Step 4 - Replace single-char hint matching in `handle_map_key` with buffer logic

Replace lines 273-326 (the `if let KeyCode::Char(c) = key.code { ... }` block) with:

```rust
// Hint key navigation
if let KeyCode::Char(c) = key.code {
    let case_sensitive = self.config.hint_labels_case_sensitive;
    let ch_str: String = if case_sensitive {
        c.to_string()
    } else {
        c.to_ascii_lowercase().to_string()
    };
    self.hint_buffer.push_str(&ch_str);
    let typed = self.hint_buffer.clone();

    let hints = crate::data::combined_hints(&self.data.keybindings);
    // Build a case-folded hint list matching the typed string's case
    let folded_hints: Vec<String> = hints.iter().map(|h| {
        if case_sensitive { h.to_string() } else { h.to_ascii_lowercase().to_string() }
    }).collect();
    let folded_refs: Vec<&str> = folded_hints.iter().map(String::as_str).collect();

    let hint_level = self.map_hint_level.clone();
    match hint_level {
        MapHintLevel::Groups => {
            let n_groups = self.data.groups.len();
            // Only resolve against the first n_groups hints
            let group_refs: Vec<&str> = folded_refs.iter().take(n_groups).copied().collect();
            match crate::data::resolve_hint(&group_refs, &typed) {
                crate::data::HintResolveResult::Exact(g_idx) => {
                    self.map_hint_level = MapHintLevel::Sections(g_idx);
                    self.hint_buffer.clear();
                }
                crate::data::HintResolveResult::Partial(_) => {
                    // Wait for more input
                }
                crate::data::HintResolveResult::NoMatch => {
                    self.hint_buffer.clear();
                }
            }
        }
        MapHintLevel::Sections(g_idx) => {
            // Check parent group hint first (toggling back to Groups)
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

            // Section hints: all hints except position g_idx
            let section_hints: Vec<&str> = folded_refs.iter().enumerate()
                .filter(|(i, _)| *i != g_idx)
                .map(|(_, h)| *h)
                .collect();
            let group_start: usize = self.data.groups.iter().take(g_idx).map(|g| g.sections.len()).sum();
            let group_len = self.data.groups.get(g_idx).map(|g| g.sections.len()).unwrap_or(0);
            // Slice to only section-count hints
            let section_refs: Vec<&str> = section_hints.iter().take(group_len).copied().collect();
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
        }
    }
}
```

**Design note on `Sections` parent-hint check:** The parent hint is a full string (e.g. `"q"`). When typed equals it exactly that is the toggle-back action. When typed is a prefix of it but shorter, hold. When it doesn't match at all, fall through to section-hint resolution. This preserves the existing two-level toggle behavior.

### Step 5 - Update `try_navigate_to_map_via_hint` to use buffer

Replace the body of `try_navigate_to_map_via_hint` (lines 1143-1175):

```rust
fn try_navigate_to_map_via_hint(&mut self, key: &KeyEvent) -> bool {
    if let KeyCode::Char(c) = key.code {
        let hints = crate::data::combined_hints(&self.data.keybindings);
        let case_sensitive = self.config.hint_labels_case_sensitive;
        let ch_str: String = if case_sensitive { c.to_string() } else { c.to_ascii_lowercase().to_string() };
        self.hint_buffer.push_str(&ch_str);
        let typed = self.hint_buffer.clone();

        let folded_hints: Vec<String> = hints.iter().map(|h| {
            if case_sensitive { h.to_string() } else { h.to_ascii_lowercase().to_string() }
        }).collect();
        let folded_refs: Vec<&str> = folded_hints.iter().map(String::as_str).collect();

        let g_idx = self.group_idx_for_section(self.current_idx);

        // Check group hint
        let group_hint = folded_refs.get(g_idx).copied().unwrap_or("");
        match crate::data::resolve_hint(&[group_hint], &typed) {
            crate::data::HintResolveResult::Exact(_) => {
                self.focus = Focus::Map;
                self.map_cursor = self.current_idx;
                self.map_hint_level = MapHintLevel::Groups;
                self.hint_buffer.clear();
                return true;
            }
            crate::data::HintResolveResult::Partial(_) => return false, // hold buffer
            crate::data::HintResolveResult::NoMatch => {}
        }

        // Check section hint
        if let Some(shi) = self.section_hint_key_idx(self.current_idx) {
            let section_hint = folded_refs.get(shi).copied().unwrap_or("");
            match crate::data::resolve_hint(&[section_hint], &typed) {
                crate::data::HintResolveResult::Exact(_) => {
                    self.focus = Focus::Map;
                    self.map_cursor = self.current_idx;
                    self.map_hint_level = MapHintLevel::Sections(g_idx);
                    self.hint_buffer.clear();
                    return true;
                }
                crate::data::HintResolveResult::Partial(_) => return false, // hold buffer
                crate::data::HintResolveResult::NoMatch => {}
            }
        }

        // No hint matched
        self.hint_buffer.clear();
    }
    false
}
```

### Step 6 - Update `handle_header_key` hint comparison to use buffer

Replace the `if let KeyCode::Char(c) = key.code { ... }` hint block in `handle_header_key` (lines 492-542) with buffer-aware logic:

```rust
if let KeyCode::Char(c) = key.code {
    let hints = crate::data::combined_hints(&self.data.keybindings);
    let case_sensitive = self.config.hint_labels_case_sensitive;
    let ch_str: String = if case_sensitive { c.to_string() } else { c.to_ascii_lowercase().to_string() };
    self.hint_buffer.push_str(&ch_str);
    let typed = self.hint_buffer.clone();

    let folded_hints: Vec<String> = hints.iter().map(|h| {
        if case_sensitive { h.to_string() } else { h.to_ascii_lowercase().to_string() }
    }).collect();
    let folded_refs: Vec<&str> = folded_hints.iter().map(String::as_str).collect();

    let g_idx = self.group_idx_for_section(self.current_idx);

    // Group hint
    let group_hint = folded_refs.get(g_idx).copied().unwrap_or("");
    match crate::data::resolve_hint(&[group_hint], &typed) {
        crate::data::HintResolveResult::Exact(_) => {
            self.focus = Focus::Map;
            self.map_cursor = self.current_idx;
            self.map_hint_level = MapHintLevel::Groups;
            self.hint_buffer.clear();
            return;
        }
        crate::data::HintResolveResult::Partial(_) => return, // hold buffer
        crate::data::HintResolveResult::NoMatch => {}
    }

    // Section hint
    if let Some(shi) = self.section_hint_key_idx(self.current_idx) {
        let section_hint = folded_refs.get(shi).copied().unwrap_or("");
        match crate::data::resolve_hint(&[section_hint], &typed) {
            crate::data::HintResolveResult::Exact(_) => {
                self.focus = Focus::Map;
                self.map_cursor = self.current_idx;
                self.map_hint_level = MapHintLevel::Sections(g_idx);
                self.hint_buffer.clear();
                return;
            }
            crate::data::HintResolveResult::Partial(_) => return, // hold buffer
            crate::data::HintResolveResult::NoMatch => {}
        }

        // Field hints: exclude section hint index and group hint index
        let field_hint_indices: Vec<usize> = (0..hints.len())
            .filter(|&i| i != shi && i != g_idx)
            .collect();
        let idx = self.current_idx;
        let n_fields = match self.section_states.get(idx) {
            Some(SectionState::Header(s)) => s.field_configs.len(),
            _ => 0,
        };
        // Build the candidate list for field hints and resolve
        let field_refs: Vec<&str> = field_hint_indices.iter()
            .take(n_fields)
            .filter_map(|&hi| folded_refs.get(hi).copied())
            .collect();
        match crate::data::resolve_hint(&field_refs, &typed) {
            crate::data::HintResolveResult::Exact(f_idx) => {
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.field_index = f_idx;
                    s.completed = false;
                }
                self.hint_buffer.clear();
                self.open_header_modal();
                return;
            }
            crate::data::HintResolveResult::Partial(_) => return, // hold buffer
            crate::data::HintResolveResult::NoMatch => {
                self.hint_buffer.clear();
            }
        }
    }
}
```

### Step 7 - Clear `hint_buffer` in `handle_key` focus-switch paths

In `handle_key`, the `is_focus_left` and `is_focus_right` branches that return to Wizard (lines 418-441) should clear the buffer:

```rust
// When switching back to Wizard from Map, clear any partial hint
self.hint_buffer.clear();
self.current_idx = self.map_cursor;
self.focus = Focus::Wizard;
self.map_hint_level = MapHintLevel::Groups;
return;
```

Also clear in the focus-left/right branches that enter Map mode (lines 411-416, 428-432) since the user is now navigating by a different modality. Insert `self.hint_buffer.clear();` before `return` in those branches.

### Step 8 - Build and verify

```bash
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe build --manifest-path "/c/scribble/Cargo.toml"
```

Must compile with no errors. Run `cargo test` to confirm all existing data.rs tests still pass.

## Verification

### Manual tests

1. **Single-char hints still work (Groups level):** With default hints `[1,2,...,9]`, pressing `1` in Map focus at Groups level immediately drills into group 0. `hint_buffer` ends up cleared.

2. **Single-char hints still work (Sections level):** After drilling into a group via its single-char hint, pressing a section hint char navigates immediately to that section and returns to Wizard.

3. **Multi-char hint - Partial then Exact:** Configure 2 groups with hints `qq` and `qw`. Press `q` - UI stays at Groups (Partial, buffer = `"q"`). Press `q` again - navigates to group 0 (Exact, buffer cleared).

4. **Multi-char hint - NoMatch reset:** With hints `qq`/`qw`, press `z` - NoMatch, buffer cleared. Subsequent `q` starts fresh.

5. **Buffer cleared on Esc:** While buffer is `"q"` (Partial), press Esc - focus returns to Wizard, buffer cleared.

6. **Buffer cleared on navigate-down/up:** While buffer is `"q"`, press down-arrow - cursor moves, buffer cleared.

7. **`try_navigate_to_map_via_hint` Partial hold:** In a non-header section, section hint is `"qq"`. Press `q` - does not navigate to map, returns false. Press `q` again - navigates to map, buffer cleared.

8. **`handle_header_key` field hint - multi-char:** Field hint is `"ww"`. Press `w` - Partial, stays in header. Press `w` again - opens modal for that field, buffer cleared.

9. **Buffer cleared on section change:** Navigate to next section via confirm. Verify buffer is `""`.

### Automated tests

None - TDD infeasible for UI key-event state machine without a test harness. Existing data.rs unit tests for `resolve_hint` / `filter_hints_by_prefix` cover the logic layer and must continue to pass.

## Changelog

### Plan - 2026-03-30
- Initial plan
