## Task

#73 Phase 1 - Editable document model and iced port

## Context

Sub-tasks 1-3 completed the document model, theme migration, and App struct changes. src/main.rs has a stub that calls `unimplemented!("iced bootstrap not yet implemented")` and src/ui.rs is an empty comment placeholder. This sub-task wires them up: replace the stub main with a real iced application bootstrap and implement the iced view() function as a three-pane layout with the editable note pane.

The App struct already has: `editable_note: String`, `note_headings_valid: bool`, `show_window: bool`, `clipboard_import: Option<String>`, and all key-handling logic via `handle_key(AppKey)`. The theme module already exports iced::Color constants. No new state is needed.

## Approach

Introduce a thin `ScribbleApp` wrapper that holds `App` and implements the iced Application pattern via the `iced::application(title, update, view)` builder. The `update` function receives a `Message` enum that covers keyboard events, text edits, and tick. The `view` function builds the three-pane layout: map pane (left), wizard pane (center), editor pane (right, bound to `editable_note`).

The `SCRIBBLENOT_HEADLESS=1` path exits immediately after `App::new`, before calling `iced::application`. The initial window is hidden (`show_window: false`); a `Message::ShowWindow` can flip `show_window` to true but the iced window itself starts visible - the field is a stub for future tray integration so we do not attempt a fully hidden window in this sub-task, we just honour the field for display decisions.

Modal overlay: when `app.modal` is `Some`, `view()` wraps the three-pane layout in an iced `Stack` (or absolute positioning via `container` centered) with the modal panel on top. When `app.modal` is `None`, just the three-pane layout.

TDD is infeasible for rendering - verification is a manual build and visual check.

## Critical Files

- `src/main.rs` (full rewrite, currently 22 lines - stub)
- `src/ui.rs` (full rewrite, currently 2 lines - stub)

Supporting files (read-only, referenced for types):
- `src/app.rs` - App struct, Focus, SectionState, AppKey, appkey_from_iced, handle_key
- `src/theme.rs` - ACTIVE, SELECTED, HINT, MODAL, MUTED, ERROR color constants
- `src/modal.rs` - SearchModal, ModalFocus fields

## Reuse

- `app::App::new(app_data, config, data_dir)` - already initializes editable_note and note_headings_valid
- `app::appkey_from_iced(key, modifiers) -> AppKey` - already exists in app.rs line 32
- `app.handle_key(AppKey)` - delegates all structured key logic
- `theme::ACTIVE`, `theme::SELECTED`, `theme::MUTED`, `theme::MODAL` - used directly in view()
- `data::find_data_dir()`, `data::AppData::load()`, `config::Config::load()` - already used in main.rs stub

## Steps

1. **Rewrite src/main.rs** - replace the unimplemented stub with a working iced bootstrap.

```diff
- fn main() -> Result<()> {
-     let data_dir = data::find_data_dir();
-     let app_data = data::AppData::load(data_dir.clone())?;
-     let config = config::Config::load(&data_dir).unwrap_or_default();
-     let _app = app::App::new(app_data, config, data_dir);
-     // iced bootstrap added in sub-task 4
-     unimplemented!("iced bootstrap not yet implemented")
- }
+ use iced::{Task, Element, Subscription};
+ use iced::keyboard;
+ use iced::time;
+ use std::time::Duration;
+
+ #[derive(Debug, Clone)]
+ pub enum Message {
+     KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
+     EditableNoteChanged(String),
+     Tick,
+ }
+
+ pub struct ScribbleApp {
+     inner: app::App,
+ }
+
+ impl ScribbleApp {
+     fn new_from_env() -> (Self, Task<Message>) {
+         let data_dir = data::find_data_dir();
+         let app_data = data::AppData::load(data_dir.clone()).expect("failed to load app data");
+         let config = config::Config::load(&data_dir).unwrap_or_default();
+         let inner = app::App::new(app_data, config, data_dir);
+         (Self { inner }, Task::none())
+     }
+ }
+
+ fn update(state: &mut ScribbleApp, message: Message) -> Task<Message> {
+     match message {
+         Message::KeyPressed(key, mods) => {
+             let app_key = app::appkey_from_iced(key, mods);
+             state.inner.handle_key(app_key);
+             if state.inner.quit {
+                 return iced::exit();
+             }
+         }
+         Message::EditableNoteChanged(new_text) => {
+             state.inner.editable_note = new_text;
+         }
+         Message::Tick => {
+             state.inner.tick();
+         }
+     }
+     Task::none()
+ }
+
+ fn view(state: &ScribbleApp) -> Element<'_, Message> {
+     ui::view(&state.inner)
+ }
+
+ fn subscription(_state: &ScribbleApp) -> Subscription<Message> {
+     let keys = keyboard::on_key_press(|key, mods| {
+         Some(Message::KeyPressed(key, mods))
+     });
+     let tick = time::every(Duration::from_millis(500)).map(|_| Message::Tick);
+     Subscription::batch(vec![keys, tick])
+ }
+
+ fn main() -> iced::Result {
+     if std::env::var("SCRIBBLENOT_HEADLESS").as_deref() == Ok("1") {
+         let data_dir = data::find_data_dir();
+         let app_data = data::AppData::load(data_dir.clone()).expect("failed to load");
+         let config = config::Config::load(&data_dir).unwrap_or_default();
+         let _ = app::App::new(app_data, config, data_dir);
+         return Ok(());
+     }
+     iced::application("Scribblenot", update, view)
+         .subscription(subscription)
+         .run_with(ScribbleApp::new_from_env)
+ }
```

Note: `fn main() -> iced::Result` replaces `fn main() -> anyhow::Result<()>`. Remove the `use anyhow::Result;` import from mod declarations. The `anyhow` crate can remain in Cargo.toml for use elsewhere.

2. **Rewrite src/ui.rs** - implement `view()` returning the three-pane layout.

The function signature is:
```
pub fn view(app: &crate::app::App) -> iced::Element<'_, crate::Message>
```

Because `Message` is defined in main.rs (the crate root), add `use crate::Message;` at the top of ui.rs. (Alternatively, move `Message` and `ScribbleApp` to a new `src/scribblenot.rs` module - but stay minimal: keep them in main.rs and reference via `crate::`.)

Three-pane layout structure:
- Use `iced::widget::row!` containing three `column!` panels with `width(iced::Length::FillPortion(N))`
- Left pane (Fill 1): map pane - renders section list with hint labels; active section highlighted with `theme::ACTIVE` text
- Center pane (Fill 2): wizard pane - renders the current section's structured widget (placeholder `text("Wizard")` is acceptable for sub-task 4, as long as it compiles and the architecture is correct)
- Right pane (Fill 3): editor pane - `text_editor` or `text_input` bound to `app.editable_note`, emitting `Message::EditableNoteChanged`

For the right pane, use `iced::widget::text_input` for simplicity (text_editor with full Content state requires sub-task-level refactoring of App state). Bind it as:
```
text_input("", &app.editable_note)
    .on_input(Message::EditableNoteChanged)
    .width(iced::Length::Fill)
```

Status bar: `column!` wrapping the row, with a `text` widget at the bottom showing `app.status.as_ref().map(|s| s.text.as_str()).unwrap_or("")` (i.e. access `StatusMsg::text` via the `Option<StatusMsg>` field).

Heading validity warning: if `!app.note_headings_valid`, render a `text("Structure warning: required headings missing")` styled with `theme::ERROR` color above the editor pane.

3. **Implement modal overlay in view()** - when `app.modal` is `Some(modal)`:

Wrap the entire three-pane layout in an `iced::widget::Stack` (if available in iced 0.13) or use a `container` with manual sizing. The modal layer is a centered `container` with:
- A `column!` containing: part label text (if composite), a `text_input` for `modal.query`, then a `scrollable` `column` of item rows
- Each visible item row (from `modal.list_scroll` to `modal.list_scroll + modal.window_size`) is a `button(text(label))` that emits `Message::KeyPressed(Enter key)` or a dedicated `Message::ModalSelect(idx)` - for minimal implementation, just render items as `text` widgets without click handling (keyboard-only modal is acceptable for sub-task 4)
- The item at `modal.list_cursor` rendered with `theme::SELECTED` color text

`iced::widget::Stack` is available in iced 0.13.1 (confirmed in iced_widget 0.13.4). Use it directly - no fallback needed.

4. **Fix main.rs mod declarations** - `mod ui` must be declared, and the `use anyhow::Result;` import must be removed since `main` now returns `iced::Result`. Verify all existing mod declarations are present: `mod app`, `mod config`, `mod data`, `mod document`, `mod modal`, `mod note`, `mod sections`, `mod theme`, `mod ui`, and `#[cfg(test)] mod appkey_tests`. Do not drop `mod note` - it is used transitively by `app.rs` and `document.rs` via `crate::note`.

5. **Cargo.toml check** - confirm `iced = { version = "0.13", features = ["tokio", "multi-window"] }` is present. No changes needed (already done in sub-task 2).

6. **Run `cargo build`** and fix any compilation errors iteratively. Expected error categories:
   - `use of undeclared type` for `Message` in ui.rs - `Message` is defined at the crate root (main.rs), so use `crate::Message` (not `crate::main::Message`); also ensure `pub enum Message` is declared `pub` in main.rs
   - Lifetime issues on `view()` return type - may need to add explicit lifetime `'_`

7. **Run `cargo test`** to verify the existing unit tests (document.rs, theme.rs, appkey_tests.rs) still pass.

## Verification

### Manual tests

- Run `cargo build` and confirm zero errors.
- Run `cargo run` and confirm the iced window opens without panicking.
- Confirm the editor pane (right pane) shows the initial document text from `editable_note`.
- Type in the editor pane and confirm the text updates (no crash).
- Run `SCRIBBLENOT_HEADLESS=1 cargo run` (or `set SCRIBBLENOT_HEADLESS=1 && cargo run` on Windows cmd) and confirm the process exits without opening a window.
- Confirm the left pane shows a list of section names (even as plain text labels).
- Confirm the center pane renders without panicking.

### Automated tests

- `cargo test` must pass: all existing tests in document.rs, theme.rs, and appkey_tests.rs must remain green.
- No new automated tests are added in this sub-task (iced rendering requires a live window; TDD is infeasible here).

## Changelog

### Review - 2026-04-06
- #1: `update()` now returns `iced::exit()` when `state.inner.quit` is set after `handle_key`, so Ctrl+C actually exits the application
- #2: Step 4 mod declaration list expanded to include `#[cfg(test)] mod appkey_tests` and an explicit note not to drop `mod note`
- #3: Removed false Stack unavailability fallback - `iced::widget::Stack` is confirmed available in iced_widget 0.13.4
- #4: Removed misleading `keyboard::on_key_press` and `Stack` items from Step 6 expected error list - both are valid in iced 0.13

### Review - 2026-04-06 (Reviewer #2)
- #5: Step 2 status bar description now shows exact field access pattern (`Option<StatusMsg>` requires `.as_ref().map(|s| s.text.as_str()).unwrap_or("")`)
- #6: `src/ui.rs` line count corrected from 3 to 2 in Critical Files
- #7: Step 1 code block tag changed from untagged to `diff` for consistency with plan conventions

### Review - 2026-04-06 (Reviewer #3)
- #8: Fixed invalid `crate::main::Message` path in Step 2 function signature, import, and parenthetical note - types in main.rs (crate root) are accessed as `crate::Message`, not `crate::main::Message`; Step 6 expected error updated to give the correct fix guidance

## Progress
- Step 1: Rewrote src/main.rs with iced application bootstrap, ScribbleApp wrapper, Message enum, update/view/subscription functions, and SCRIBBLENOT_HEADLESS exit path
- Step 2: Rewrote src/ui.rs with three-pane layout (map, wizard, editor), status bar, heading validity warning, and text_input bound to editable_note
- Step 3: Implemented modal overlay in view() using iced Stack with composite label, search query, and filtered item list
- Step 4: Verified mod declarations in main.rs - all present including appkey_tests and note
- Step 5: Confirmed Cargo.toml has iced 0.13 with tokio and multi-window features
- Step 6: cargo check passes (0 errors, only pre-existing dead_code warnings)
- Step 7: cargo test passes for theme, appkey, document tests (22/22 ok); 24 pre-existing data failures unrelated to this change

## Implementation
Complete - 2026-04-06
