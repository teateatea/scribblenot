## Task

#2 - Add Shift+Enter super-confirm keybinding to auto-complete remaining fields

## Context

The wizard currently requires the user to manually confirm each field via Enter. Users want a faster path: Shift+Enter should confirm the currently focused field using whatever visible value it already has (typed, sticky, or default). If the field is empty with no fallback, it is silently skipped. This is a single-field confirm-and-advance, not a cascade. The binding must be configurable in keybindings.yml.

The TDD tests that govern this sub-task already exist in src/app.rs (lines 1251-1279) and src/data.rs (lines 299-319). They require: a `match_binding_str` free function exported from app.rs, a `super_confirm` field on `KeyBindings`, and the default value `["shift+enter"]`.

## Approach

1. Add a `match_binding_str(binding: &str, key: &KeyEvent) -> bool` free function to src/app.rs that handles modifier-prefixed strings like `shift+enter`. Refactor `matches_key` to call it per-binding.
2. Add `super_confirm: Vec<String>` to the `KeyBindings` struct in src/data.rs with a serde default of `["shift+enter"]` and include it in `Default`.
3. Add `is_super_confirm(&self, key: &KeyEvent) -> bool` helper on `App` that delegates to `matches_key`.
4. Add `super_confirm: [shift+enter]` to data/keybindings.yml.

## Critical Files

- `src/app.rs` lines 155-177 (`matches_key`), lines 1246-1280 (existing tests)
- `src/data.rs` lines 144-190 (`KeyBindings` struct and `Default` impl), lines 295-320 (existing tests)
- `data/keybindings.yml` (add one line)

## Reuse

- `matches_key` (src/app.rs line 155) - already calls per-binding match logic; extract its inner match block into `match_binding_str` and call that from the loop.
- `KeyModifiers::SHIFT` from `crossterm::event` - already imported at line 11 of src/app.rs.
- Pattern of `#[serde(default = "...")]` with a named default fn - already used for `focus_left`, `focus_right`, `hints` in src/data.rs lines 155-170.

## Steps

1. In `src/data.rs`, add a default function and field for `super_confirm`:

```
- pub struct KeyBindings {
-     pub navigate_down: Vec<String>,
+ pub struct KeyBindings {
+     pub navigate_down: Vec<String>,
      // ... existing fields unchanged ...
      #[serde(default = "default_hints")]
      pub hints: Vec<String>,
+     #[serde(default = "default_super_confirm")]
+     pub super_confirm: Vec<String>,
  }

+ fn default_super_confirm() -> Vec<String> {
+     vec!["shift+enter".to_string()]
+ }
```

   Also add `super_confirm: default_super_confirm()` to the `Default` impl body after `hints`.

2. In `src/app.rs`, extract a `pub fn match_binding_str(binding: &str, key: &KeyEvent) -> bool` free function (outside `impl App`) that contains the current `match binding.as_str() { ... }` block, extended with a `shift+enter` arm:

```
+ pub fn match_binding_str(binding: &str, key: &KeyEvent) -> bool {
+     match binding {
+         "down" => key.code == KeyCode::Down,
+         "up" => key.code == KeyCode::Up,
+         "left" => key.code == KeyCode::Left,
+         "right" => key.code == KeyCode::Right,
+         "enter" => key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE,
+         "esc" => key.code == KeyCode::Esc,
+         "space" => key.code == KeyCode::Char(' '),
+         "backspace" => key.code == KeyCode::Backspace,
+         "shift+enter" => key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::SHIFT),
+         s if s.len() == 1 => {
+             let c = s.chars().next().unwrap();
+             key.code == KeyCode::Char(c)
+         }
+         _ => false,
+     }
+ }
```

   Update `matches_key` to call `match_binding_str` instead of inlining the match:

```
  fn matches_key(&self, key: &KeyEvent, action: &[String]) -> bool {
      for binding in action {
-         let matched = match binding.as_str() {
-             "down" => key.code == KeyCode::Down,
-             // ... (all arms) ...
-             _ => false,
-         };
+         let matched = match_binding_str(binding, key);
          if matched {
              return true;
          }
      }
      false
  }
```

   Note: the existing `"enter"` arm currently has no modifier guard. Adding `key.modifiers == KeyModifiers::NONE` ensures plain Enter and Shift+Enter are mutually exclusive.

3. In `src/app.rs`, add `is_super_confirm` inside `impl App` after `is_focus_right`:

```
+ fn is_super_confirm(&self, key: &KeyEvent) -> bool {
+     self.matches_key(key, &self.data.keybindings.super_confirm)
+ }
```

4. In `data/keybindings.yml`, append one line after the existing `hints:` line:

```
- hints: [q, w, f, p, 1, 2, 3, 4, 5, 6, 7, 8, 9]
+ hints: [q, w, f, p, 1, 2, 3, 4, 5, 6, 7, 8, 9]
+ super_confirm: [shift+enter]
```

## Verification

### Manual tests

- Run `cargo test` using the correct invocation - all five tests in the existing test blocks must pass:
  - `matches_key_shift_enter_binding_recognized` (src/app.rs)
  - `matches_key_shift_enter_does_not_match_plain_enter` (src/app.rs)
  - `keybindings_default_has_super_confirm_shift_enter` (src/data.rs)
  - `keybindings_super_confirm_serde_default` (src/data.rs)
  - `matches_key_super_confirm_binding_in_keybindings` (src/app.rs)
  ```
  CARGO="$USERPROFILE/.cargo/bin/cargo.exe" PATH="$PATH:$USERPROFILE/.cargo/bin:$USERPROFILE/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" cd "/c/scribble" "$CARGO" test
  ```
- Run `cargo build` using the correct invocation - zero warnings and zero errors:
  ```
  CARGO="$USERPROFILE/.cargo/bin/cargo.exe" PATH="$PATH:$USERPROFILE/.cargo/bin:$USERPROFILE/scoop/apps/mingw/15.2.0-rt_v13-rev1/bin" cd "/c/scribble" "$CARGO" build
  ```
- Open keybindings.yml and confirm `super_confirm: [shift+enter]` is present.

### Automated tests

The five unit tests already committed to src/app.rs and src/data.rs cover all required behaviors for this sub-task. No additional tests need to be written; the implementation is complete when `cargo test` passes without any of these tests being skipped or modified.

## Changelog

### Review - 2026-03-30
- #1: Fixed cargo test/build commands in Verification to use required PATH invocation (blocking - per mission log and reviewer brief)
- #2: Fixed "all three tests" wording to "all five tests" to match the five tests actually listed

## Prefect-1 Report

All cross-checks passed. One nit applied.

- **[nit] Critical Files line range for data.rs tests** (`M9-2-super-confirm-keybinding.md:21`): Range listed as `295-319` but the closing `}` of the `mod tests` block is at line 320. Fixed to `295-320`.

No blocking or minor issues found. Cargo commands use the correct PATH invocation. All five test names match the tests in the source. `match_binding_str` signature, `super_confirm` field, default function, and keybindings.yml diff all correctly reflect actual source state.

## Changelog

### Prefect-1 - 2026-03-30
- #1: Corrected data.rs test line range from 295-319 to 295-320 (nit)

## Progress

- Step 1: Added `super_confirm: Vec<String>` field with `#[serde(default = "default_super_confirm")]` to `KeyBindings` struct in src/data.rs, added `default_super_confirm()` fn, and added `super_confirm: default_super_confirm()` to `Default` impl
- Step 2: Added `pub fn match_binding_str` free function to src/app.rs with shift+enter arm; refactored `matches_key` to call it; "enter" arm now guards with `KeyModifiers::NONE`
- Step 3: Added `is_super_confirm` method to `impl App` in src/app.rs after `is_focus_right`
- Step 4: Added `super_confirm: [shift+enter]` line to data/keybindings.yml after the hints line
