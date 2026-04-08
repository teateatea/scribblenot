## Task

#73 Phase 2 - Tray, hotkeys, clipboard import, and packaging baseline

## Goal

Turn the iced desktop app into a tray-first workflow:

- start hidden
- show/hide from a global hotkey
- copy-and-close from a second hotkey
- optionally import a prior note from the clipboard

This phase is implementation-ready.

## Scope

- tray icon
- show/hide hotkey
- copy-and-close hotkey
- clipboard import heuristic
- import banner flow
- startup-time headless test
- release packaging baseline

## Dependencies

Update [Cargo.toml](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/Cargo.toml):

```diff
+ tray-icon = "0.22"
+ global-hotkey = "0.7"
+ image = { version = "0.25", features = ["png"] }

+ [profile.release]
+ lto = true
+ codegen-units = 1
+ strip = true
```

`Config` changes belong in [config.rs](/C:/Users/solar/Documents/Claude%20Projects/scribblenot/src/config.rs):

```rust
#[serde(default = "default_show_hotkey")]
pub hotkey: String,
#[serde(default = "default_copy_hotkey")]
pub close_copy_hotkey: String,
```

Defaults:

```rust
fn default_show_hotkey() -> String { "Alt+Shift+N".to_string() }
fn default_copy_hotkey() -> String { "Alt+Shift+C".to_string() }
```

Config is loaded from the app data directory via `Config::load(&data_dir)`, not from a repo-local `data/config.yml`.

## Tray Behavior

Create `assets/tray-icon.png` as a minimal placeholder icon.

Initialize tray support before the iced app event loop and keep the handle alive for the process lifetime.

Tray click behavior:

- if hidden, show the main window
- if visible, hide the main window
- when showing, clear any future chord buffer state if that feature is ever enabled later

## Hotkey Behavior

Register:

- show/hide hotkey
- copy-and-close hotkey

Show/hide behavior:

- toggle `self.app.show_window`
- if showing, run clipboard import heuristic
- emit the iced window visibility command

Copy-and-close behavior:

- write `self.app.editable_note.clone()` to clipboard
- hide the window
- do not regenerate the note from `render_note()`

## Clipboard Import

Add `src/import.rs`.

Required helper:

- `try_parse_clipboard_note(text: &str, sections: &[SectionConfig]) -> Option<String>`

Heuristic:

- count lines that match known canonical headings or approved per-section anchors
- if at least two structural anchors are present, offer import
- otherwise ignore the clipboard and leave it untouched

UI behavior:

- show dismissible banner when a likely note is detected
- `ImportYes` loads text into `editable_note`
- `ImportNo` clears the banner only

Import must not silently overwrite the current note.

After import:

- run heading validation
- surface repair warning if anchors are invalid
- optional back-fill into structured state is allowed only where safe and explicit

## Main Wiring

Use the app wrapper and iced subscriptions introduced in Phase 1.

Subscription responsibilities in this phase:

- poll tray events
- poll global hotkey events

This phase does not include `rdev`.

## Startup Test

Add `tests/startup_time.rs`.

Behavior:

- spawn the release binary with `SCRIBBLENOT_HEADLESS=1`
- assert it prints `ready`
- assert it exits within 500ms on Windows

## Packaging Baseline

After `cargo build --release`, verify the release artifact is a single usable `.exe`.

Manual dependency verification can use:

```text
dumpbin /dependents target\release\scribblenot.exe
```

Expected DLLs are Windows system DLLs only.

Optional installer work stays out of this phase.

## Verification

Manual checks:

- cold launch shows tray icon only
- `Alt+Shift+N` shows the window
- `Alt+Shift+N` again hides it
- `Alt+Shift+C` copies current `editable_note` and hides the window
- clipboard import banner appears only for likely note text
- choosing not to import leaves clipboard content untouched
- imported text remains editable and is what gets copied back out

Automated checks:

- `tests/startup_time.rs`
- unit tests for `try_parse_clipboard_note`

## Exit Criteria

This phase is complete when:

- the app lives in the tray
- hotkeys control visibility and copy flow
- clipboard import is explicit and safe
- export uses `editable_note`
