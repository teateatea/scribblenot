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

use anyhow::Result;

fn main() -> Result<()> {
    let data_dir = data::find_data_dir();
    let app_data = data::AppData::load(data_dir.clone())?;
    let config = config::Config::load(&data_dir).unwrap_or_default();
    let _app = app::App::new(app_data, config, data_dir);
    // iced bootstrap added in sub-task 4
    unimplemented!("iced bootstrap not yet implemented")
}
