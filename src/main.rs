mod app;
mod config;
mod data;
mod document;
mod modal;
mod note;
mod sections;
mod theme;
mod ui;
#[cfg(test)]
mod appkey_tests;

use iced::{Task, Element, Subscription};
use iced::keyboard;
use iced::time;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    EditableNoteChanged(String),
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
    match message {
        Message::KeyPressed(key, mods) => {
            let app_key = app::appkey_from_iced(key, mods);
            state.inner.handle_key(app_key);
            if state.inner.quit {
                return iced::exit();
            }
        }
        Message::EditableNoteChanged(new_text) => {
            state.inner.editable_note = new_text;
        }
        Message::Tick => {
            state.inner.tick();
        }
    }
    Task::none()
}

fn view(state: &ScribbleApp) -> Element<'_, Message> {
    ui::view(&state.inner)
}

fn subscription(_state: &ScribbleApp) -> Subscription<Message> {
    let keys = keyboard::on_key_press(|key, mods| {
        Some(Message::KeyPressed(key, mods))
    });
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
