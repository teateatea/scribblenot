mod app;
mod config;
mod data;
mod modal;
mod note;
mod sections;
mod theme;
mod ui;

use anyhow::Result;
use app::App;
use config::Config;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use data::{find_data_dir, AppData};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

fn main() -> Result<()> {
    let data_dir = find_data_dir();
    let app_data = AppData::load(data_dir.clone())?;
    let config = Config::load(&data_dir).unwrap_or_default();

    let mut app = App::new(app_data, config, data_dir.clone());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        app.tick();

        if app.quit {
            break;
        }

        if app.note_completed && !app.quit {
            // Try to copy note to clipboard
            let note_text = note::render_note(&app.sections, &app.section_states);
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    match clipboard.set_text(note_text.clone()) {
                        Ok(_) => {
                            app.status = Some(app::StatusMsg::success(
                                "Note copied to clipboard! Press [q] to quit.",
                            ));
                        }
                        Err(e) => {
                            app.status = Some(app::StatusMsg::error(format!(
                                "Clipboard error: {}. Press [q] to quit.",
                                e
                            )));
                        }
                    }
                }
                Err(e) => {
                    app.status = Some(app::StatusMsg::error(format!(
                        "Clipboard unavailable: {}. Press [q] to quit.",
                        e
                    )));
                }
            }
            // Reset so we don't keep re-copying
            app.note_completed = false;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }

    Ok(())
}
