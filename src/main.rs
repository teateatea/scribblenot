mod app;
#[cfg(test)]
mod appkey_tests;
mod config;
mod data;
mod document;
mod modal;
mod note;
mod sections;
mod theme;
mod ui;

use iced::keyboard;
use iced::time;
use iced::widget::scrollable;
use iced::{Element, Size, Subscription, Task};
use std::io::{stderr, stdout, IsTerminal};
use std::process::ExitCode;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    EditableNoteChanged(String),
    ModalQueryChanged(String),
    ModalCompositionChanged(String),
    ModalSelect(usize),
    ModalPanePressed(app::ModalPaneTarget),
    ModalRowPressed(app::ModalPaneTarget, usize),
    ModalRowHovered(app::ModalPaneTarget, usize),
    ModalBackdropPressed,
    ModalPanelPressed,
    WindowResized(Size),
    Tick,
}

pub struct ScribbleApp {
    inner: app::App,
}

impl ScribbleApp {
    fn new_from_env() -> (Self, Task<Message>) {
        let data_dir = data::find_data_dir();
        let app_data = data::AppData::load(data_dir.clone()).expect("failed to load app data");
        let config = config::Config::load(&data_dir).unwrap_or_default();
        let inner = app::App::new(app_data, config, data_dir);
        (Self { inner }, Task::none())
    }
}

fn update(state: &mut ScribbleApp, message: Message) -> Task<Message> {
    let mut should_scroll_preview = false;
    let mut should_scroll_active_pane = false;
    match message {
        Message::KeyPressed(key, mods) => {
            let app_key = app::appkey_from_iced(key, mods);
            state.inner.handle_key(app_key);
            should_scroll_preview = true;
            should_scroll_active_pane = true;
            if state.inner.quit {
                return iced::exit();
            }
        }
        Message::EditableNoteChanged(new_text) => {
            state.inner.set_editable_note(new_text);
        }
        Message::ModalQueryChanged(new_text) => {
            state.inner.set_modal_query(new_text);
        }
        Message::ModalCompositionChanged(new_text) => {
            state.inner.set_modal_composition_text(new_text);
            should_scroll_preview = true;
        }
        Message::ModalSelect(filtered_index) => {
            state.inner.select_modal_filtered_index(filtered_index);
            should_scroll_preview = true;
            should_scroll_active_pane = true;
        }
        Message::ModalPanePressed(target) => {
            state.inner.activate_modal_mouse_mode();
            state.inner.focus_modal_pane(target);
            should_scroll_preview = true;
            should_scroll_active_pane = true;
        }
        Message::ModalRowPressed(target, row_index) => {
            state.inner.activate_modal_mouse_mode();
            state.inner.press_modal_row(target, row_index);
            should_scroll_preview = true;
            should_scroll_active_pane = true;
        }
        Message::ModalRowHovered(target, row_index) => {
            state.inner.hover_modal_row(target, row_index);
            should_scroll_preview = true;
            should_scroll_active_pane = true;
        }
        Message::ModalBackdropPressed => {
            state.inner.dismiss_modal();
            should_scroll_preview = true;
            should_scroll_active_pane = true;
        }
        Message::ModalPanelPressed => {
            state.inner.activate_modal_mouse_mode();
        }
        Message::WindowResized(size) => {
            state.inner.set_viewport_size(size);
        }
        Message::Tick => {
            state.inner.tick();
        }
    }

    if state.inner.copy_requested {
        state.inner.copy_requested = false;
        let export_text = document::export_editable_document(&state.inner.editable_note);
        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(export_text)) {
            Ok(()) => {
                state.inner.status = Some(app::StatusMsg::success("Copied note to clipboard."));
                state.inner.copy_flash_until = Some(
                    Instant::now()
                        + Duration::from_millis(
                            state.inner.ui_theme.preview_copy_flash_duration_ms,
                        ),
                );
            }
            Err(err) => {
                state.inner.status = Some(app::StatusMsg::error(format!("Copy failed: {err}")));
            }
        }
    }

    let mut tasks = Vec::new();

    if should_scroll_preview {
        let line = state.inner.current_preview_scroll_line();
        tasks.push(scrollable::scroll_to(
            ui::preview_scroll_id(),
            scrollable::AbsoluteOffset {
                x: 0.0,
                y: f32::from(line) * 18.0,
            },
        ));
    }

    if should_scroll_active_pane
        && state.inner.modal.is_none()
        && matches!(state.inner.focus, app::Focus::Map)
    {
        let (id, line) = (ui::map_scroll_id(), state.inner.current_map_scroll_line());
        tasks.push(scrollable::scroll_to(
            id,
            scrollable::AbsoluteOffset {
                x: 0.0,
                y: f32::from(line) * 18.0,
            },
        ));
    }

    if !tasks.is_empty() {
        return Task::batch(tasks);
    }

    Task::none()
}

fn view(state: &ScribbleApp) -> Element<'_, Message> {
    ui::view(&state.inner)
}

fn subscription(state: &ScribbleApp) -> Subscription<Message> {
    let keys = keyboard::on_key_press(|key, mods| Some(Message::KeyPressed(key, mods)));
    let resize = iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size));
    let tick_interval = if state.inner.copy_flash_until.is_some()
        || state.inner.has_active_text_flash()
        || state.inner.has_active_modal_transition()
    {
        Duration::from_millis(33)
    } else {
        Duration::from_millis(500)
    };
    let tick = time::every(tick_interval).map(|_| Message::Tick);
    Subscription::batch(vec![keys, resize, tick])
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!(
                "{}",
                format_error_message(&err.to_string(), stderr().is_terminal())
            );
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "--validate-data" || arg == "--validate" {
            let summary =
                data::validate_data_dir(&data::find_data_dir()).map_err(anyhow::Error::msg)?;
            let keybindings = if summary.keybindings_present {
                "keybindings checked"
            } else {
                "no keybindings.yml present"
            };
            let color_enabled = stdout().is_terminal();
            let heading = colorize("Validation OK:", ANSI_GREEN, true, color_enabled);
            let rest = colorize(
                &format!(
                    " {} hierarchy files, {} groups, {} sections, {} collections, {} fields, {} lists, {} boilerplate entries, {}.",
                    summary.hierarchy_file_count,
                    summary.group_count,
                    summary.section_count,
                    summary.collection_count,
                    summary.field_count,
                    summary.list_count,
                    summary.boilerplate_count,
                    keybindings
                ),
                ANSI_GREEN,
                false,
                color_enabled,
            );
            println!("{heading}{rest}");
            return Ok(());
        }
        return Err(anyhow::anyhow!(
            "unknown argument '{arg}'; supported options: --validate, --validate-data"
        ));
    }

    if std::env::var("SCRIBBLENOT_HEADLESS").as_deref() == Ok("1") {
        let data_dir = data::find_data_dir();
        let app_data = data::AppData::load(data_dir.clone()).expect("failed to load");
        let config = config::Config::load(&data_dir).unwrap_or_default();
        let _ = app::App::new(app_data, config, data_dir);
        return Ok(());
    }

    iced::application("Scribblenot", update, view)
        .subscription(subscription)
        .run_with(ScribbleApp::new_from_env)?;
    Ok(())
}

const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_RESET: &str = "\x1b[0m";

fn colorize(text: &str, color: &str, bold: bool, enabled: bool) -> String {
    if enabled {
        let weight = if bold { ANSI_BOLD } else { "" };
        format!("{weight}{color}{text}{ANSI_RESET}")
    } else {
        text.to_string()
    }
}

fn format_error_message(message: &str, color_enabled: bool) -> String {
    let prefix = colorize("scribblenot:", ANSI_YELLOW, true, color_enabled);
    match message.split_once(" Fix:") {
        Some((issue, fix)) => {
            let issue = colorize(issue, ANSI_YELLOW, false, color_enabled);
            let fix_heading = colorize("Fix:", ANSI_GREEN, true, color_enabled);
            let fix_body = colorize(fix, ANSI_GREEN, false, color_enabled);
            let fix = format!("{fix_heading}{fix_body}");
            format!("{prefix} {issue} {fix}")
        }
        None => {
            let issue = colorize(message, ANSI_YELLOW, false, color_enabled);
            format!("{prefix} {issue}")
        }
    }
}
