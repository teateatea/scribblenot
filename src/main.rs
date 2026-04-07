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
use iced::{Element, Subscription, Task};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    EditableNoteChanged(String),
    ModalQueryChanged(String),
    ModalSelect(usize),
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
        Message::ModalSelect(filtered_index) => {
            state.inner.select_modal_filtered_index(filtered_index);
            should_scroll_preview = true;
            should_scroll_active_pane = true;
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

fn subscription(_state: &ScribbleApp) -> Subscription<Message> {
    let keys = keyboard::on_key_press(|key, mods| Some(Message::KeyPressed(key, mods)));
    let tick = time::every(Duration::from_millis(500)).map(|_| Message::Tick);
    Subscription::batch(vec![keys, tick])
}

fn main() -> iced::Result {
    if std::env::var("SCRIBBLENOT_HEADLESS").as_deref() == Ok("1") {
        let data_dir = data::find_data_dir();
        let app_data = data::AppData::load(data_dir.clone()).expect("failed to load");
        let config = config::Config::load(&data_dir).unwrap_or_default();
        let _ = app::App::new(app_data, config, data_dir);
        return Ok(());
    }
    iced::application("Scribblenot", update, view)
        .subscription(subscription)
        .run_with(ScribbleApp::new_from_env)
}
