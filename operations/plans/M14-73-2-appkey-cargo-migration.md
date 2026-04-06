## Task

#73 - Phase 1: AppKey enum, iced keyboard conversion, and Cargo.toml migration (sub-task 2)

## Context

The approved plan (APPROVED-73-1-document-model-and-iced-port.md) requires removing ratatui and crossterm in favor of iced. Nearly all key handling in app.rs currently accepts `crossterm::event::KeyEvent`. Removing crossterm breaks compilation of every key-handling function and the existing keybinding tests. This sub-task insulates the app logic from any specific input backend by introducing `AppKey` as the canonical key type, migrates all key-handling logic to it, and updates Cargo.toml so the project compiles under iced. It does not implement the iced UI or replace main.rs bootstrap - only the abstraction layer.

## Approach

1. Add `iced = { version = "0.13", features = ["tokio", "multi-window"] }` to Cargo.toml and remove ratatui and crossterm.
2. Define `AppKey` and `pub fn appkey_from_iced(...)` in app.rs, replacing the crossterm import.
3. Rewrite `match_binding_str` to accept `&AppKey` instead of `&KeyEvent`.
4. Update `matches_key` and all `is_*` helpers to accept `&AppKey`.
5. Update all `handle_*` functions to accept `AppKey` instead of `KeyEvent`.
6. Migrate the existing `#[cfg(test)] mod tests` in app.rs to construct `AppKey` values directly instead of crossterm `KeyEvent`.
7. The 8 tests in appkey_tests.rs (already committed) will compile and pass once AppKey and appkey_from_iced exist.
8. Keep main.rs compiling by removing the crossterm event loop (replace with a stub that panics or calls unimplemented!) - the full iced bootstrap is sub-task 4.

## Critical Files

- `C:/Users/solar/Documents/Claude Projects/scribblenot/Cargo.toml` - replace ratatui/crossterm with iced
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/app.rs` - line 12 (crossterm import), lines 88-105 (match_binding_str), lines 191-251 (is_* helpers), lines 400-523 (handle_key), lines 539+ (handle_header_key and remaining handle_* fns), lines 1402-1504 (existing tests)
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/main.rs` - remove crossterm/ratatui imports, stub out run_app
- `C:/Users/solar/Documents/Claude Projects/scribblenot/src/appkey_tests.rs` - already written; must compile after this sub-task

## Reuse

- `AppKey` enum definition is specified verbatim in APPROVED-73-1-document-model-and-iced-port.md - use it as-is, adding `Space` and `CtrlC` to match what appkey_tests.rs already tests for.
- `match_binding_str` binding strings ("down", "up", "shift+enter", etc.) are reused unchanged - only the argument type changes from `&KeyEvent` to `&AppKey`.
- All `is_*` helper names and `matches_key` remain; only their argument type changes.

## Steps

1. **Update Cargo.toml** - replace ratatui and crossterm with iced:

```diff
- ratatui = "0.29"
- crossterm = "0.28"
+ iced = { version = "0.13", features = ["tokio", "multi-window"] }
```

2. **Define AppKey and appkey_from_iced in app.rs** - replace the crossterm import at line 12 and add the enum plus conversion function before the `match_binding_str` fn:

```diff
- use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
+ use iced::keyboard::{key::Named, Key, Modifiers};
```

Add after the imports block:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppKey {
    Char(char),
    CtrlChar(char),
    CtrlC,
    Enter,
    ShiftEnter,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Backspace,
    Tab,
    Space,
}

pub fn appkey_from_iced(key: Key, modifiers: Modifiers) -> AppKey {
    match key {
        Key::Named(Named::Enter) => {
            if modifiers.contains(Modifiers::SHIFT) {
                AppKey::ShiftEnter
            } else {
                AppKey::Enter
            }
        }
        Key::Named(Named::Escape) => AppKey::Esc,
        Key::Named(Named::Backspace) => AppKey::Backspace,
        Key::Named(Named::Tab) => AppKey::Tab,
        Key::Named(Named::ArrowDown) => AppKey::Down,
        Key::Named(Named::ArrowUp) => AppKey::Up,
        Key::Named(Named::ArrowLeft) => AppKey::Left,
        Key::Named(Named::ArrowRight) => AppKey::Right,
        Key::Character(ref s) => {
            let c = s.chars().next().unwrap_or('\0');
            if modifiers.contains(Modifiers::CTRL) {
                if c == 'c' { AppKey::CtrlC } else { AppKey::CtrlChar(c) }
            } else if c == ' ' {
                AppKey::Space
            } else {
                AppKey::Char(c)
            }
        }
        _ => AppKey::Char('\0'),
    }
}
```

3. **Rewrite match_binding_str** to accept `&AppKey`:

```diff
- pub fn match_binding_str(binding: &str, key: &KeyEvent) -> bool {
-     match binding {
-         "down" => key.code == KeyCode::Down,
-         "up" => key.code == KeyCode::Up,
-         "left" => key.code == KeyCode::Left,
-         "right" => key.code == KeyCode::Right,
-         "enter" => key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE,
-         "esc" => key.code == KeyCode::Esc,
-         "space" => key.code == KeyCode::Char(' '),
-         "backspace" => key.code == KeyCode::Backspace,
-         "shift+enter" => key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::SHIFT),
-         s if s.len() == 1 => {
-             let c = s.chars().next().unwrap();
-             key.code == KeyCode::Char(c)
-         }
-         _ => false,
-     }
- }
+ pub fn match_binding_str(binding: &str, key: &AppKey) -> bool {
+     match binding {
+         "down" => matches!(key, AppKey::Down),
+         "up" => matches!(key, AppKey::Up),
+         "left" => matches!(key, AppKey::Left),
+         "right" => matches!(key, AppKey::Right),
+         "enter" => matches!(key, AppKey::Enter),
+         "esc" => matches!(key, AppKey::Esc),
+         "space" => matches!(key, AppKey::Space),
+         "backspace" => matches!(key, AppKey::Backspace),
+         "shift+enter" => matches!(key, AppKey::ShiftEnter),
+         s if s.len() == 1 => {
+             let c = s.chars().next().unwrap();
+             matches!(key, AppKey::Char(k) if *k == c)
+         }
+         _ => false,
+     }
+ }
```

4. **Update matches_key and all is_* helpers** - change argument types from `&KeyEvent` to `&AppKey`. Example diff for matches_key and two helpers (repeat for all 12 is_* fns):

```diff
- fn matches_key(&self, key: &KeyEvent, action: &[String]) -> bool {
+ fn matches_key(&self, key: &AppKey, action: &[String]) -> bool {
```

```diff
- fn is_navigate_down(&self, key: &KeyEvent) -> bool {
+ fn is_navigate_down(&self, key: &AppKey) -> bool {
```

Apply the same `&KeyEvent` -> `&AppKey` substitution to every `is_*` fn (is_navigate_up, is_select, is_confirm, is_add_entry, is_back, is_swap_panes, is_help, is_quit, is_copy_note, is_focus_left, is_focus_right, is_super_confirm).

5. **Update handle_key signature and body** - change argument from `KeyEvent` to `AppKey` and rewrite the two inline crossterm checks:

```diff
- pub fn handle_key(&mut self, key: KeyEvent) {
-     // Ctrl+C always quits
-     if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
-         self.quit = true;
-         return;
-     }
+ pub fn handle_key(&mut self, key: AppKey) {
+     // Ctrl+C always quits
+     if matches!(key, AppKey::CtrlC) {
+         self.quit = true;
+         return;
+     }
```

Also update the hint-key inline check that pattern-matches `KeyCode::Char(c)` to use `AppKey::Char(c)`:

```diff
-     if self.is_quit(&key) && self.focus != Focus::Map {
-         let is_hint_key = if let KeyCode::Char(c) = key.code {
+     if self.is_quit(&key) && self.focus != Focus::Map {
+         let is_hint_key = if let AppKey::Char(c) = key {
```

6. **Update handle_header_key, handle_free_text_key, handle_list_select_key, handle_block_select_key, handle_checklist_key, handle_map_key, handle_modal_key** - change signature from `fn handle_*(&mut self, key: KeyEvent)` to `fn handle_*(&mut self, key: AppKey)`. Within each fn, replace all `key.code == KeyCode::Xyz` pattern-matches with `AppKey::Xyz` matches. Replace `key.modifiers.contains(KeyModifiers::SHIFT)` checks with `matches!(key, AppKey::ShiftEnter)` where applicable. For character extraction (e.g. the hint key logic in handle_header_key that does `if let KeyCode::Char(c) = key.code`), change to `if let AppKey::Char(c) = key`.

   Additional inline replacements required:

   - **handle_header_key** (line 686): `if key.code == KeyCode::Enter` -> `if matches!(key, AppKey::Enter)`
   - **handle_map_key** (line 299): `if let KeyCode::Char(c) = key.code` -> `if let AppKey::Char(c) = key`
   - **handle_free_text_key** (line 888): `if key.code == KeyCode::Enter` -> `if matches!(key, AppKey::Enter)` (inside the is_editing block)
   - **handle_free_text_key** (line 894): `if key.code == KeyCode::Backspace` -> `if matches!(key, AppKey::Backspace)`
   - **handle_free_text_key** (line 900): `if let KeyCode::Char(c) = key.code` -> `if let AppKey::Char(c) = key`
   - **handle_list_select_key** (line 977): `if key.code == KeyCode::Enter` -> `if matches!(key, AppKey::Enter)` (inside the adding mode block)
   - **handle_list_select_key** (line 1011): `if key.code == KeyCode::Backspace` -> `if matches!(key, AppKey::Backspace)`
   - **handle_list_select_key** (line 1017): `if let KeyCode::Char(c) = key.code` -> `if let AppKey::Char(c) = key`
   - **try_navigate_to_map_via_hint** (line 1231): change signature from `fn try_navigate_to_map_via_hint(&mut self, key: &KeyEvent) -> bool` to `fn try_navigate_to_map_via_hint(&mut self, key: &AppKey) -> bool`, and replace `if let KeyCode::Char(c) = key.code` with `if let AppKey::Char(c) = *key` (or `if let AppKey::Char(c) = key` if matching on `&AppKey` directly)
   - **handle_modal_key** (line 727): standalone guard `if key.code == KeyCode::Esc` before the focus match must become `if matches!(key, AppKey::Esc)`
   - **handle_modal_key** (lines 755-824): the two-level `match focus { ... => match key.code { ... } }` must be restructured. The `key.code` field access does not exist on `AppKey`; instead match on `key` directly. Example for the SearchBar arm:

```rust
ModalFocus::SearchBar => match key {
    AppKey::Tab => { ... }
    AppKey::Enter => { ... }
    AppKey::Backspace => { ... }
    AppKey::Char(c) => { ... }
    _ => {}
},
ModalFocus::List => match key {
    AppKey::Backspace => { ... }
    AppKey::Space => { ... }
    AppKey::Enter => { ... }
    AppKey::Up => { ... }
    AppKey::Down => { ... }
    AppKey::Char(c) => { ... }
    _ => {}
},
```

   Because `AppKey` derives `Copy`, `key` can be matched multiple times without `.clone()` calls.

7. **Migrate existing tests in app.rs** (lines 1402-1504) - remove the crossterm import and rewrite KeyEvent construction to use AppKey directly:

```diff
- use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
+ use super::AppKey;
```

Replace each `KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)` with `AppKey::ShiftEnter`, and `KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)` with `AppKey::Enter`. Update `match_binding_str` call-sites to pass `&AppKey::ShiftEnter` etc. Update `app.handle_header_key(key)` to accept `AppKey` value directly.

8. **Stub out main.rs crossterm/ratatui usage** - the full iced bootstrap is sub-task 4. For now, keep main.rs compiling by removing the crossterm event loop body. Remove the crossterm and ratatui imports, remove `run_app`, and replace the `main` body with a minimal stub that builds the App and returns:

```rust
fn main() -> anyhow::Result<()> {
    let data_dir = data::find_data_dir();
    let app_data = data::AppData::load(data_dir.clone())?;
    let config = config::Config::load(&data_dir).unwrap_or_default();
    let _app = app::App::new(app_data, config, data_dir);
    // iced bootstrap added in sub-task 4
    unimplemented!("iced bootstrap not yet implemented")
}
```

9. **Verify the build compiles** - run `cargo test --lib` to confirm all 8 appkey_tests and the 5 migrated app.rs tests pass, and `cargo build` to confirm no compilation errors.

## Verification

### Manual tests

- Run `cargo build` and confirm it succeeds with no crossterm or ratatui errors.
- Run `cargo test` and confirm all 13 tests pass (8 in appkey_tests.rs + 5 in app.rs mod tests).

### Automated tests

- `cargo test` passes all tests in appkey_tests.rs: enter_without_modifiers_maps_to_enter, shift_enter_maps_to_shift_enter, escape_maps_to_esc, backspace_maps_to_backspace, arrow_keys_map_correctly, space_character_maps_to_space, regular_char_maps_to_char_variant, ctrl_c_maps_to_ctrl_c.
- `cargo test` passes all tests in app.rs mod tests: matches_key_shift_enter_binding_recognized, matches_key_shift_enter_does_not_match_plain_enter, matches_key_super_confirm_binding_in_keybindings, super_confirm_fills_default_and_advances, super_confirm_no_op_when_no_default.

## Changelog

### Review - 2026-04-06
- #1: Added `Copy` to `AppKey` derive list to prevent move-after-use errors in multi-branch key dispatch and nested match in handle_modal_key
- #2: Added `try_navigate_to_map_via_hint` to Step 6's scope - this private helper takes `&KeyEvent` and must also be migrated to `&AppKey`
- #3: Added explicit translation note for `handle_modal_key`'s two-level `match key.code` structure, which must be restructured to `match key` on `AppKey` directly
- #4: Called out `handle_header_key` line 686 inline `key.code == KeyCode::Enter` as a required change in Step 6
- #5: Called out `handle_map_key` line 299 inline `if let KeyCode::Char(c) = key.code` as a required change in Step 6
- #6: Fixed test count from 4 to 5 in Step 9 and Verification section (app.rs mod tests has 5 tests, confirmed by reading the file)

### Review - 2026-04-06 (Reviewer #2)
- #7: Called out `handle_modal_key` line 727 standalone `if key.code == KeyCode::Esc` guard (before the focus match) as a required inline replacement to `if matches!(key, AppKey::Esc)` - was absent from Step 6's explicit callout list

### Review - 2026-04-06 (Reviewer #3)
- #8: Added explicit callouts for six inline `key.code` checks in `handle_free_text_key` (lines 888, 894, 900) and `handle_list_select_key` (lines 977, 1011, 1017) - these are direct `KeyCode` accesses in the is_editing and adding-mode blocks that are not routed through `is_*` helpers, following the same pattern as prior callouts #4, #5, #7

## Progress
- Step 1: Updated Cargo.toml to replace ratatui/crossterm with iced 0.13
- Step 2: Defined AppKey enum and appkey_from_iced in app.rs, replaced crossterm import with iced keyboard imports
- Step 3: Rewrote match_binding_str to accept &AppKey with matches! macro
- Step 4: Updated matches_key and all 13 is_* helpers to accept &AppKey
- Step 5: Updated handle_key signature to AppKey with CtrlC and hint-key checks
- Step 6: Updated handle_map_key, handle_header_key, handle_modal_key, handle_free_text_key, handle_list_select_key, handle_block_select_key, handle_checklist_key, try_navigate_to_map_via_hint to use AppKey
- Step 7: Migrated tests in app.rs mod tests to use AppKey directly
- Step 8: Stubbed main.rs (removed crossterm/ratatui), also stubbed theme.rs and ui.rs (superficial mismatch: plan did not mention these but they referenced ratatui)
- Step 9: Verified build compiles and all 13 target tests pass (8 appkey + 5 app::tests)

## Implementation
Complete - 2026-04-06
