## Task
#23 ST3 - Combined hint pool for assignment (hints + hint_permutations)

## Context
Currently, hint assignment throughout app.rs and ui.rs draws exclusively from `keybindings.hints` (the single-char pool). With `hint_permutations` now generated (ST2), the app can assign 2-char hints to groups, sections, and fields when the single-char pool is exhausted. ST3 wires these two vecs into one ordered slice for every assignment site, without touching key-event buffering or UI highlight rendering (those are task #22).

## Approach
Add a free function `combined_hints(kb: &KeyBindings) -> Vec<&str>` in `src/data.rs` that returns a `Vec<&str>` containing all `hints` entries followed by all `hint_permutations` entries. Replace every location in `app.rs` and `ui.rs` that directly accesses `&self.data.keybindings.hints` (or clones it) for hint-assignment purposes with a call to `combined_hints`. The `window_size` used for `SearchModal` (line 619 in app.rs) should also use `combined_hints(...).len()` so the modal window can display more rows when permutations exist.

`section_hint_key_idx` at line 342 uses `hints.len()` to compute how many valid index positions exist - this must also use the combined length so indices into the combined pool are valid.

The quit-key guard at line 384 (`hints.iter().any(...)`) must also use the combined pool so 2-char prefixes are not treated as the quit key when they match a single hint char. However, since hint_permutations are multi-char strings and key events are single chars, extending that guard is a no-op for multi-char hints. It is still correct to extend it for completeness and to keep the pattern consistent.

No changes to key-event buffering, UI highlight rendering, or task #22 state machine are needed.

## Critical Files

- `src/data.rs` - KeyBindings struct (line 159-164), `ensure_hint_permutations` (line 375), tests module (line 387+)
- `src/app.rs` - hint assignment sites:
  - Line 275: `handle_map_key` - group/section hint matching
  - Line 343: `section_hint_key_idx` - computes hint index for a section
  - Line 384: quit-key guard
  - Line 493: `handle_header_key` - field hint assignment
  - Line 519-520: field_hint_indices calculation
  - Line 619: `window_size` for SearchModal
  - Line 640: `handle_modal_key` - modal list hint matching
  - Line 1145: `try_navigate_to_map_via_hint`
- `src/ui.rs` - hint display (render only, not assignment per se, but reads pool for display):
  - Line 67: `render_section_map` - group hints
  - Line 205: `render_section_widget` (field_hints)
  - Line 676: `render_modal` - window_size for modal height

## Reuse
- Pattern mirrors how `ensure_hint_permutations` already takes `&mut KeyBindings` - keep the same import path.
- The existing `hints` field retains its type (`Vec<String>`) - `combined_hints` borrows from it.
- Test module already lives in `src/data.rs` at line 387; new unit tests go in the same `#[cfg(test)] mod tests` block.

## Steps

### Step 1 - Add `combined_hints` free function to `src/data.rs`

After `ensure_hint_permutations` (around line 385), insert:

```rust
/// Returns a combined ordered slice of all hints followed by all hint_permutations.
/// Use this wherever hints are assigned to groups, sections, fields, or modal rows.
pub fn combined_hints(kb: &KeyBindings) -> Vec<&str> {
    kb.hints.iter().map(String::as_str)
        .chain(kb.hint_permutations.iter().map(String::as_str))
        .collect()
}
```

### Step 2 - Update `section_hint_key_idx` in `src/app.rs` (line 342)

Change:
```rust
let hints = &self.data.keybindings.hints;
```
to:
```rust
let hints = crate::data::combined_hints(&self.data.keybindings);
```
And update the length reference on line 348 - since `hints` is now `Vec<&str>`, `hints.len()` is already correct. No further change needed there.

### Step 3 - Update `handle_map_key` (line 275)

Change:
```rust
let hints = self.data.keybindings.hints.clone();
```
to:
```rust
let hints = crate::data::combined_hints(&self.data.keybindings);
```
All subsequent `.get(g_idx)` and `.iter().enumerate()` calls on `hints` work identically because `Vec<&str>` supports the same operations.

### Step 4 - Update quit-key guard (line 384)

Change:
```rust
self.data.keybindings.hints.iter().any(|h| h.to_ascii_lowercase().to_string() == c_str)
```
to:
```rust
crate::data::combined_hints(&self.data.keybindings).iter().any(|h| h.to_ascii_lowercase() == c_str)
```

### Step 5 - Update `handle_header_key` (line 493)

Change:
```rust
let hints = self.data.keybindings.hints.clone();
```
to:
```rust
let hints = crate::data::combined_hints(&self.data.keybindings);
```

### Step 6 - Update `window_size` for SearchModal (line 619)

Change:
```rust
let window_size = self.data.keybindings.hints.len();
```
to:
```rust
let window_size = crate::data::combined_hints(&self.data.keybindings).len();
```

### Step 7 - Update `handle_modal_key` (line 640)

Change:
```rust
let hints = self.data.keybindings.hints.clone();
```
to:
```rust
let hints = crate::data::combined_hints(&self.data.keybindings);
```
The downstream usage `hints.iter().position(|h| h == &c.to_string())` still works because `Vec<&str>` elements implement `PartialEq<String>` via deref coercion - but to be safe, change the comparison to:
```rust
hints.iter().position(|h| *h == c.to_string().as_str())
```

### Step 8 - Update `try_navigate_to_map_via_hint` (line 1145)

Change:
```rust
let hints = self.data.keybindings.hints.clone();
```
to:
```rust
let hints = crate::data::combined_hints(&self.data.keybindings);
```

### Step 9 - Update `src/ui.rs` render sites

**Line 67 (`render_section_map`):**
```rust
let hints = crate::data::combined_hints(&app.data.keybindings);
```

**Line 205 (`render_section_widget` field_hints):**
```rust
let hints = crate::data::combined_hints(&app.data.keybindings);
```
The `hints.get(i)` calls return `Option<&&str>`. The existing `map` closure calls `.clone()` which returns `&&str` instead of `String` after the type change — change `.clone()` to `.to_string()` in that closure to keep `field_hints: Vec<String>` compiling:
```rust
.map(|h| if cap { h.to_uppercase() } else { h.to_string() })
```

**Line 676 (`render_modal` window height):**
```rust
let hints = crate::data::combined_hints(&app.data.keybindings);
```

### Step 10 - ~~Write unit tests~~ ALREADY DONE (pre-written by Test Writer in Red phase)

All 4 tests already exist in `src/data.rs` at lines 563+. Do NOT re-insert them — duplicate function names will not compile.

## Verification

### Manual tests
1. Run the app with default keybindings (9 hints, 81 permutations after ensure_hint_permutations). Verify hint labels on groups/sections still show single-char hints as before (no regression).
2. Temporarily shrink the hints pool to 2 entries in keybindings.yml (e.g. `["a","b"]`) and add enough groups/sections to exceed 2. Verify that groups/sections beyond index 1 receive 2-char hint labels (aa, ab, ba, bb) instead of blank.
3. Open a modal list with many options - verify the window height grows to reflect the combined pool size.

### Automated tests
Test names:
- `combined_hints_returns_hints_then_permutations`
- `combined_hints_total_length`
- `combined_hints_empty_permutations`
- `combined_hints_order_hints_before_permutations`

Cargo command:
```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml"
```

Run tests targeting only the new function:
```
PATH="$PATH:/c/Users/solar/.cargo/bin:/c/Users/solar/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" /c/Users/solar/.cargo/bin/cargo.exe test --manifest-path "/c/scribble/Cargo.toml" combined_hints
```

## Changelog
### Plan - 2026-03-30
- Initial plan
- Prefect R1: Step 10 marked ALREADY DONE (4 tests pre-written at lines 563+); Step 9 line 205 updated to specify `.clone()` → `.to_string()` fix for `field_hints: Vec<String>` type compatibility.
