## Task

No matching entry in operations/TASKS.md (file does not exist yet). This plan covers the full desktop-app rewrite discussed in DISCUSSION-scribblenot-desktop-app.md.

## Context

Scribblenot currently runs inside a terminal via `cargo run`. For a clinician mid-workflow, opening a terminal is too much friction. The user needs a persistent background process that appears instantly on a hotkey, accepts shorthand input with chord shortcuts (including Shift+Enter), and disappears after copying the formatted note to the clipboard. The terminal backend (ratatui + crossterm) cannot support some required keyboard behaviors. Long-term, the binary must be distributable to colleagues.

## Approach

Replace the ratatui/crossterm terminal UI with an **egui** (via `eframe`) native-window GUI. Egui is chosen over iced for this project because: (1) egui has first-class Windows tray support via `tray-icon` + `winit` (both already used by eframe internally); (2) egui's immediate-mode rendering produces sub-10ms first-frame startup, satisfying the hard latency requirement; (3) egui handles Shift+Enter and arbitrary key combos natively in its `egui::Event` model; (4) iced's retained-mode architecture requires significantly more boilerplate and its tray story on Windows is immature. All application logic (YAML loading, section state machines, note rendering, clipboard) remains in Rust and is reused unchanged. The UI layer is the only thing replaced.

The rewrite is phased to remain shippable at each boundary:

- **Phase 1 (Framework swap):** Replace `main.rs` terminal bootstrap with an `eframe::run_native` call. Port the render loop from `ui.rs` to an `egui::App` impl. All existing section state machines (`app.rs`, `sections/`) are preserved as-is.
- **Phase 2 (Tray + hotkey):** Add `tray-icon` crate for the system tray icon and `global-hotkey` crate for the show/hide hotkey. Window starts hidden; hotkey raises it. Close-and-copy hotkey writes to clipboard and hides the window.
- **Phase 3 (Clipboard import):** At hotkey-open time, snapshot the clipboard. If the content matches a structured clinical note (headings present from the known section taxonomy), offer to pre-fill the editable note pane. Non-matching clipboard content is left untouched and never discarded silently.
- **Phase 4 (Global chords):** Implement an in-process rolling keypress buffer (6-10 chars, heap-only, never written to disk, never transmitted) using the `rdev` crate for low-level keyboard hooks. When a chord matches a section template key sequence, open the app and expand that section in one action. Buffer is cleared on window show.
- **Phase 5 (Packaging):** Build a single statically-linked `.exe` with no runtime dependencies. Provide a WiX/NSIS installer for colleague distribution.

**HIPAA / keylogger boundary clarification:** The rolling buffer stores only the last 6-10 keypresses in a `VecDeque<char>` allocated on the heap. It is never serialized, never written to disk, and never transmitted. No patient identifiers pass through it (clinical note text is typed inside the app window, not captured by the global hook). This architecture is consistent with the user's interpretation that no PHI is involved.

**Clipboard import edge-case resolution:** On hotkey open, the app reads the current clipboard value. It applies a heuristic: if the text contains two or more lines that begin with a known section heading (e.g. "Subjective:", "Objective:", "Treatment:"), it is treated as a prior note and offered for import. If the heuristic does not match, the clipboard value is preserved untouched and the note pane starts empty. The user is shown a one-line banner ("Import prior note? [Y/N]") so they always know a match was detected. No silent discard.

## Critical Files

- `src/main.rs` - terminal bootstrap to replace with eframe startup (lines 1-113)
- `src/ui.rs` - entire render layer to port from ratatui widgets to egui widgets
- `src/app.rs` - add `show_window: bool`, `clipboard_import: Option<String>` fields; replace all `crossterm::event::{KeyCode, KeyEvent, KeyModifiers}` imports and usages with egui-equivalent input types (see Step 3b); section state machine logic is otherwise unchanged
- `src/theme.rs` - ratatui color constants to be replaced/augmented with `egui::Color32` equivalents
- `Cargo.toml` - drop ratatui/crossterm, add eframe, tray-icon, global-hotkey, rdev
- `src/config.rs` - add `hotkey: String` and `chord_map: HashMap<String, String>` fields for configurable bindings
- `assets/tray-icon.png` - 16x16 PNG tray icon to be created in Step 5 (new file)

## Reuse

- `src/app.rs` `App` struct and section-state-machine logic - key handling logic is preserved but crossterm types must be replaced (see Step 3b)
- `src/note.rs` `render_note()` - unchanged; called identically on copy action
- `src/data.rs` `AppData::load()` / `find_data_dir()` - unchanged
- `src/sections/` all section state machines - unchanged
- `arboard` crate - already a dependency; clipboard read (for import) and write (for copy) both use existing `arboard::Clipboard`
- `src/config.rs` `Config::load()` / `Config::save()` - extended, not replaced

## Steps

1. **Benchmark startup (informational, no code change):** Run `cargo build --release` and time the current binary cold start on Windows to establish baseline. Document in a comment in `main.rs`.

2. **Add eframe and remove ratatui/crossterm in `Cargo.toml`:**
   ```
   - ratatui = "0.29"
   - crossterm = "0.28"
   + eframe = { version = "0.31", features = ["default_fonts", "persistence"] }
   ```

2b. **Update `src/theme.rs`.** Replace all `ratatui::style::{Color, Modifier, Style}` types with `egui::Color32` and `egui::Stroke`/`egui::TextFormat` equivalents (ratatui is removed in Step 2, so the existing ratatui-typed constants and style-helper functions will not compile; they must be replaced, not added alongside). Keep the same semantic color names and palette values. **This step must be completed before Steps 3 and 4 — `theme.rs` must compile before the eframe startup and UI port can build.**

3. **Replace `src/main.rs` terminal bootstrap with eframe startup.** Keep `App::new(...)` construction identical. Replace the `enable_raw_mode` / `EnterAlternateScreen` / `run_app` block with `eframe::run_native("Scribblenot", options, Box::new(|_cc| Ok(Box::new(app)))).map_err(|e| anyhow::anyhow!("{e}"))?` (in eframe 0.31, `run_native` returns `eframe::Result<()>` which is not `anyhow::Error`; the `.map_err` converts it). Implement `eframe::App` for `App` with `fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)` delegating to the new UI module.

3b. **Replace crossterm key types in `src/app.rs` and add new App fields.** Removing crossterm in Step 2 breaks all of `app.rs`'s key-handling code, which imports `crossterm::event::{KeyCode, KeyEvent, KeyModifiers}` and uses them throughout `handle_key`, `match_binding_str`, and all `is_*` helper methods. Define a thin `AppKeyEvent` struct (or re-use a suitable egui type) that carries the same logical fields (`code: AppKeyCode`, `modifiers: AppModifiers`) so the existing match arms in `handle_key` can be updated with minimal structural change. The section state machines in `src/sections/` do not import crossterm and are unaffected. Also add two new fields to the `App` struct: `pub show_window: bool` (initialized to `false` in `App::new`) and `pub clipboard_import: Option<String>` (initialized to `None` in `App::new`) — these are required by Steps 6 and 7. This step is a prerequisite for Step 4 (the egui `update()` loop must be able to call `app.handle_key(...)` with the new type).

4. **Port `src/ui.rs` to egui widgets.** The three-column layout (Map | Wizard | Preview) maps to `egui::SidePanel` (map) + `egui::CentralPanel` (wizard) + `egui::SidePanel` (preview). Section-specific widgets (free text: `egui::TextEdit`; list select: `egui::SelectableLabel`; block select: checkboxes + group list; checklist: `egui::Checkbox`; header: labeled text inputs) replace the ratatui equivalents. Shift+Enter is handled by checking `ctx.input(|i| i.key_pressed(Key::Enter) && i.modifiers.shift)`.

5. **Add tray icon support.** Add `tray-icon = "0.20"` to `Cargo.toml`. Create `assets/tray-icon.png` (16x16 PNG; a minimal placeholder is sufficient for initial builds). In `main.rs`, create a `TrayIcon` with the icon bundled via `include_bytes!("../assets/tray-icon.png")` before the eframe run loop. Handle tray left-click to toggle window visibility. Window starts hidden (`NativeOptions { visible: false, .. }`).

6. **Add global hotkey for show/hide.** Add `global-hotkey = "0.6"` to `Cargo.toml`. Register a configurable hotkey (default: `Alt+Shift+N`, readable from `config.yml` `hotkey` field). **Do not attempt to mutate `App` from a hotkey callback outside eframe's loop — `App` is moved into eframe via `Box::new(app)` and is inaccessible from outside.** Instead, in `App::update()`, poll `GlobalHotKeyEvent::receiver()` each frame (non-blocking `try_recv`) and toggle `self.show_window` when a matching event arrives; then call `ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.show_window))`. (`eframe::Frame` does not expose a `set_visible` method in eframe 0.31; viewport commands are the correct API.) Store the registered `GlobalHotKeyManager` in a field on `App` or keep it alive in a local variable before `eframe::run_native` (it must not be dropped). Add close-and-copy hotkey (default: `Alt+Shift+C`) that calls the existing clipboard-write path then sets `self.show_window = false`.

7. **Implement clipboard import heuristic.** In `main.rs` (or a new `src/import.rs`), add `fn try_parse_clipboard_note(text: &str, sections: &[SectionConfig]) -> Option<HashMap<String, String>>`. Heuristic: split text on newlines; count lines that start with a known `cfg.heading_label` value (note: `SectionConfig.heading_label` is `Option<String>`, so skip entries where it is `None`); if count >= 2, return a map of heading -> content. In `App::update()`, when the show hotkey event is received (see Step 6), read the clipboard, run the heuristic, and set `self.clipboard_import` — this replaces any notion of a separate `App::show()` method called from outside eframe, since `App` is not accessible outside the eframe event loop. In `update()`, if `clipboard_import.is_some()`, render a banner: "Import prior note? [Y] yes [N] skip".

8. **Implement global chord detector.** Add `rdev = "0.5"` to `Cargo.toml`. Spawn a background thread in `main.rs` that calls `rdev::listen` with a callback that appends keypresses to a `Arc<Mutex<VecDeque<char>>>` (capacity 10, pop-front on overflow). Never write the buffer to disk. In the eframe update loop, drain the shared buffer each frame and check against the `chord_map` from config. On match, show the window and pre-select the matching section.

9. **Extend `src/config.rs`:**
   ```
   + pub hotkey: String,          // default: "Alt+Shift+N"
   + pub close_copy_hotkey: String, // default: "Alt+Shift+C"
   + pub chord_map: HashMap<String, String>, // e.g. {"arstob": "objective_section"}
   ```
   Add `#[serde(default)]` with sensible defaults so existing `config.yml` files load without error.

10. **Add a startup-time assertion in CI.** Before this test can work, add headless support to the binary: check for `SCRIBBLENOT_HEADLESS=1` in the environment at startup and, if set, print `"ready"` to stdout and exit immediately after loading data (before calling `eframe::run_native`). Then in a new integration test `tests/startup_time.rs`, set `SCRIBBLENOT_HEADLESS=1`, spawn the release binary, wait for `"ready"` on stdout, and assert elapsed < 500ms. (Windows-only test, `#[cfg(target_os = "windows")]`.)

11. **Build single-exe release.** In `Cargo.toml`, add:
    ```
    [profile.release]
    lto = true
    codegen-units = 1
    strip = true
    ```
    Note: `strip = true` is safe on Windows/MSVC for distribution but removes PDB debug info; keep a non-stripped build locally for debugging. Verify `cargo build --release` produces a single `.exe` with no DLL dependencies beyond the Windows system DLLs (`kernel32`, `user32`, `gdi32`).

## Verification

### Manual tests

- Cold-launch the release binary; the window must not appear (starts in tray only). Verify tray icon is visible in the system notification area.
- Press the configured show hotkey (`Alt+Shift+N`). The scribblenot window must appear within 200ms of the keypress (subjective assessment; no stopwatch needed).
- Type in the wizard pane; press Shift+Enter to advance a free-text section. Confirm Shift+Enter advances the section without inserting a literal newline.
- Copy a realistic patient note to the clipboard, then press the show hotkey. A banner "Import prior note?" must appear. Press Y to import; confirm the editable note pane is pre-filled. Press N; confirm the pane starts empty and the original clipboard content is unchanged.
- Complete a note and press the close-and-copy hotkey. Confirm: (a) the window hides, (b) pasting (`Ctrl+V`) in another app yields the fully formatted note, (c) no file was written to disk.
- Type a configured chord sequence (`arstob` or equivalent) in any application. Confirm scribblenot opens and navigates to the matching section.
- Resize the window; confirm the three-column layout reflows without clipping.
- Verify custom theme colors render correctly (change a color in `theme.rs`, rebuild, confirm the UI updates).
- Distribute the single `.exe` to a machine without Rust installed; confirm it runs.

### Automated tests

- `tests/startup_time.rs`: spawn release binary in headless mode (pass `--headless` flag or use `SCRIBBLENOT_HEADLESS=1` env var), assert process reaches ready state in < 500ms.
- Unit test for `try_parse_clipboard_note`: feed it a multi-section clinical note string and assert the returned map contains the expected section headings; feed it lorem ipsum and assert `None` is returned.
- Unit test for chord buffer: construct a `VecDeque<char>` of capacity 10, push 11 chars, assert length is 10 and the oldest char is dropped; assert a matching chord sequence is detected.
- Existing 180-test suite must continue to pass unchanged (all section logic is untouched).

## Prefect Report

### Pass 2 - 2026-04-03

**1. [minor] Stale crate version pins across Steps 2, 5, 6, 8**
The plan pins `eframe = "0.31"` (Step 2), `tray-icon = "0.20"` (Step 5), and `global-hotkey = "0.6"` (Step 6). Current published versions are eframe 0.34.1, tray-icon 0.22.0, global-hotkey 0.7.0. The eframe gap (0.31 vs 0.34) spans three minor releases and includes breaking changes in `NativeOptions` and the `eframe::App` trait. Using pinned-but-outdated versions risks API mismatches that will surface as compile errors. `rdev = "0.5"` (Step 8) is current (0.5.3) and is fine.
- Steps 2, 5, 6: update version pins to latest stable (eframe 0.34, tray-icon 0.22, global-hotkey 0.7).
- Note: the Step 3 inline comment "in eframe 0.31, `run_native` returns `eframe::Result<()>`" is version-annotated to an outdated pin; if the version is updated the comment should drop the "0.31" qualifier since the same return type applies in current releases.

**2. [minor] Step 3b: `AppKeyEvent`/`AppKeyCode`/`AppModifiers` wrapper types are unspecified**
Step 3b instructs the implementer to "define a thin `AppKeyEvent` struct (or re-use a suitable egui type)" but never enumerates the required variants. Examining `app.rs:83-99` (`match_binding_str`) and `app.rs:387-424` (`handle_key`), the key codes required are at minimum: `Char(char)`, `Enter`, `Esc`, `Up`, `Down`, `Left`, `Right`, `Backspace`. The modifiers needed are `SHIFT`, `CONTROL`, and `NONE`. Without a concrete definition or an explicit instruction to map these from `egui::Key` and `egui::Modifiers`, an implementer is likely to produce an incompatible wrapper that breaks `match_binding_str`'s string-matching logic. The step should either specify the enum variants or explicitly instruct using `egui::Key` + `egui::Modifiers` directly and updating `match_binding_str` accordingly.

**3. [minor] Cargo.toml changes are fragmented across five steps with no consolidated view**
Steps 2, 5, 6, and 8 each append dependency changes to `Cargo.toml` (remove ratatui/crossterm; add eframe; add tray-icon; add global-hotkey; add rdev). Step 9 adds fields to `src/config.rs` but the `HashMap` import is already present. No single step shows the final intended `[dependencies]` block. This means the code will not compile between Steps 2 and 5 (eframe added but tray-icon/global-hotkey absent), creating an unstable intermediate state. Consider adding a note in Step 2 to add all new crate dependencies at once, or provide a final consolidated `[dependencies]` block in Step 11 as a verification target.

**4. [nit] Step 5 tray-icon construction: no code example for `TrayIconBuilder` API**
Step 5 says "create a `TrayIcon` with the icon bundled via `include_bytes!(...)`" but does not show the builder call. The `tray-icon` crate API changed between 0.18 and 0.22 (menu construction moved to `muda` crate; `TrayIconBuilder::new()` is stable but `with_menu` behavior changed). Since an implementer will be writing new code with no prior usage in the project, a minimal constructor example (even pseudocode) would prevent a category of compile errors. Low risk given the crate's docs, but worth noting.

## Changelog

### Review - 2026-04-03
- #1: Step 6 - replaced non-existent `frame.set_visible()` with correct eframe 0.31 API `ctx.send_viewport_cmd(egui::ViewportCommand::Visible(...))`
- #2: Step 7 - clarified that `SectionConfig.heading_label` is `Option<String>` so heuristic must skip `None` entries; added requirement to implement `App::show()` method explicitly
- #3: Step 11 - added missing headless mode implementation requirement (binary must check `SCRIBBLENOT_HEADLESS=1` and exit after data load) so the startup-time test can actually run
- #4: Step 5 - specified icon file path `assets/tray-icon.png` and added it to Critical Files
- #5: Step 12 - added note that `strip = true` removes PDB debug info on Windows/MSVC; keep non-stripped build locally

### Review - 2026-04-03
- #6: Steps 6+7 (blocking) - corrected threading model: `App` is moved into eframe and cannot be mutated from outside the event loop; hotkey events must be polled via `GlobalHotKeyEvent::receiver().try_recv()` inside `App::update()`, not via a direct method call from a callback; removed misleading `App::show()` description in Step 7 accordingly
- #7: Step 3 (minor) - added `.map_err(|e| anyhow::anyhow!("{e}"))?` to `eframe::run_native` call to convert `eframe::Result` to `anyhow::Result` as required by `main()`'s return type
- #8: Step 10 (minor) - clarified that ratatui types in `theme.rs` must be replaced (not added alongside) since ratatui is removed in Step 2; flagged Step 10 as a prerequisite for Steps 3-4

### Review - 2026-04-03
- #9: Critical Files + Reuse + new Step 3b (blocking) - `app.rs` imports and uses `crossterm::event::{KeyCode, KeyEvent, KeyModifiers}` throughout `handle_key`, `match_binding_str`, and all `is_*` helpers; removing crossterm in Step 2 breaks these; added Step 3b to replace crossterm key types with egui-equivalent types; corrected Critical Files and Reuse sections which falsely stated `app.rs` key logic could be kept unchanged
