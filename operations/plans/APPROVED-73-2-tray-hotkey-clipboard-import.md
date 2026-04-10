## Task

#73 Phase 2 - Tray, hotkeys, clipboard import, and packaging baseline

## Goal

Turn the iced desktop app into a tray-first workflow:

- start hidden
- show/hide from a global hotkey
- copy-and-close from a second hotkey
- optionally import a prior clinic note from the clipboard for temporary editing

This phase is implementation-ready.

## Scope

- tray icon
- show/hide hotkey
- copy-and-close hotkey
- clipboard import heuristic
- import banner flow
- temporary in-memory clinic-note editing session
- startup-time headless test
- release packaging baseline

## Current State (as of plan update)

Phase 1 is complete. The app compiles and runs under iced 0.13. Key facts relevant to this phase:

- Wrapper struct: `ScribbleApp { inner: app::App }` in `main.rs`
- `state.inner.editable_note: String` - the live editable text
- `state.inner.copy_requested: bool` - already used; clipboard write happens in `update()` in main.rs
- `state.inner.quit: bool` - triggers `iced::exit()` from `update()`
- `SCRIBBLENOT_HEADLESS=1` early-exit path already exists in `main.rs:175-181` but does not yet print `ready`
- `subscription()` fn in main.rs already wires `keyboard`, `resize`, and `tick` subscriptions
- No `show_window` field in App yet; no tray, hotkey, or import infrastructure exists

## Workflow Intent

The clipboard import feature exists to support this real user workflow:

1. copy an existing patient note from the clinic system
2. open Scribblenot
3. import that note into a temporary editing session
4. update or append to the note using Scribblenot
5. copy the updated plain-text note back into the clinic system

Privacy boundary:

- imported patient note content is session-only working text
- imported patient note content must not be auto-saved to disk
- product YAML and user preferences may persist normally, but patient note content must remain ephemeral unless a separate future feature explicitly changes that policy

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

Config is split by responsibility:

- product YAML data remains in the user-editable Scribblenot data directory
- per-user preferences such as hotkeys, theme, and window behavior belong in a real per-user config directory

Phase 2 must not treat the repo-local `data/config.yml` pattern as the long-term desktop-app config model.

## App State

Add `show_window: bool` to the `App` struct in `app.rs`. Initialize to `false` (tray-first launch).

Add a small amount of import-session state as needed for:

- pending clipboard import offer
- whether the current note originated from clipboard import
- import banner visibility and dismissal

## Tray Behavior

Create `assets/tray-icon.png` as a minimal placeholder icon.

Initialize tray support before the iced app event loop and keep the handle alive for the process lifetime.

Tray click behavior:

- if hidden (`show_window == false`), show the main window
- if visible (`show_window == true`), hide the main window
- when showing, run clipboard import heuristic

## Hotkey Behavior

Register:

- show/hide hotkey (`config.hotkey`)
- copy-and-close hotkey (`config.close_copy_hotkey`)

Show/hide behavior:

- toggle `state.inner.show_window`
- if showing, run clipboard import heuristic
- emit the iced window visibility command

Copy-and-close behavior:

- write the user-facing exported note text to clipboard
- set `state.inner.show_window = false`
- hide the window
- do not regenerate the note from `render_note()`

## Clipboard Import

Add `src/import.rs`.

Required helper:

- `try_parse_clipboard_note(text: &str, sections: &[SectionConfig]) -> Option<String>`

Heuristic:

- treat clipboard content as external plain note text, typically copied from the clinic system or from a prior Scribblenot export
- count lines that match known canonical headings or approved per-section anchors
- if at least two structural anchors are present, offer import
- otherwise ignore the clipboard and leave it untouched

UI behavior:

- show dismissible banner when a likely note is detected
- add `ImportYes` and `ImportNo` variants to the `Message` enum in `main.rs`
- `ImportYes` converts recognized plain note text into Scribblenot's managed editable-document format before assigning `state.inner.editable_note`
- `ImportNo` clears the banner only

Import must not silently overwrite the current note.

After import:

- run heading validation
- surface repair warning if anchors are invalid
- optional back-fill into structured state is allowed only where safe and explicit

Import conversion rules:

- clipboard import targets external plain note text with sufficiently similar exported structure
- imported text must not be assigned directly to `editable_note` unless it already contains the required managed section markers
- the conversion step rebuilds managed section markers around recognized sections before validation
- unmatched text should be preserved where feasible instead of being discarded silently
- if conversion cannot confidently rebuild the managed structure, the app must refuse structured import and leave the current note unchanged

Persistence rules:

- imported note text lives in memory only for the current app session
- clipboard import must not create patient-specific files
- clipboard import must not write patient note content into app config, theme files, product YAML, logs, crash notes, or test fixtures
- any future persisted draft or autosave feature is out of scope for this phase and requires explicit approval

## Main Wiring

Extend the `subscription()` fn in `main.rs` to batch two additional subscriptions:

- poll tray events
- poll global hotkey events

This phase does not include `rdev`.

## Startup Test

Add `tests/startup_time.rs`.

Also add `println!("ready")` to the `SCRIBBLENOT_HEADLESS` branch in `main.rs` (currently at line 175-181) before the early return.

Test behavior:

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

- cold launch shows tray icon only (window hidden)
- `Alt+Shift+N` shows the window
- `Alt+Shift+N` again hides it
- `Alt+Shift+C` copies current `editable_note` and hides the window
- clipboard import banner appears only for likely note text
- choosing not to import leaves clipboard content untouched
- imported text remains editable and is what gets copied back out
- copied text does not expose internal Scribblenot section markers
- importing a note previously copied from Scribblenot restores a valid managed editable document instead of raw pasted text
- importing a note copied from the clinic system allows the user to edit and append without saving patient text to disk
- closing and reopening the app does not restore prior imported patient note text

Automated checks:

- `tests/startup_time.rs`
- unit tests for `try_parse_clipboard_note`
- unit tests for import conversion on representative clinic-note text

## Exit Criteria

This phase is complete when:

- the app lives in the tray
- hotkeys control visibility and copy flow
- clipboard import is explicit and safe
- export uses the user-facing editable note content without exposing internal markers
- imported clinic-note text remains session-only
