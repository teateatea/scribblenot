use crate::app::App;
use crate::theme;
use crate::Message;
use iced::widget::{column, container, row, scrollable, text, text_input, Stack};
use iced::{Element, Length};

/// Build the left pane: section map showing section labels.
fn map_pane(app: &App) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = Vec::new();
    for (i, sec) in app.sections.iter().enumerate() {
        let label = text(&sec.map_label);
        let styled = if i == app.current_idx {
            label.color(theme::ACTIVE)
        } else {
            label.color(theme::MUTED)
        };
        items.push(styled.into());
    }
    column(items)
        .width(Length::Fill)
        .padding(4)
        .into()
}

/// Build the center pane: wizard area (placeholder for sub-task 4).
fn wizard_pane(_app: &App) -> Element<'_, Message> {
    column![text("Wizard")]
        .width(Length::Fill)
        .padding(4)
        .into()
}

/// Build the right pane: editable note with heading validity warning.
fn editor_pane(app: &App) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = Vec::new();

    if !app.note_headings_valid {
        items.push(
            text("Structure warning: required headings missing")
                .color(theme::ERROR)
                .into(),
        );
    }

    items.push(
        text_input("", &app.editable_note)
            .on_input(Message::EditableNoteChanged)
            .width(Length::Fill)
            .into(),
    );

    column(items)
        .width(Length::Fill)
        .padding(4)
        .into()
}

/// Build the status bar at the bottom.
fn status_bar(app: &App) -> Element<'_, Message> {
    let status_text = app
        .status
        .as_ref()
        .map(|s| s.text.as_str())
        .unwrap_or("");
    let color = if app.status.as_ref().map_or(false, |s| s.is_error) {
        theme::ERROR
    } else {
        theme::MUTED
    };
    text(status_text).color(color).into()
}

/// Build the three-pane main layout.
fn main_layout(app: &App) -> Element<'_, Message> {
    let left = container(map_pane(app)).width(Length::FillPortion(1));
    let center = container(wizard_pane(app)).width(Length::FillPortion(2));
    let right = container(editor_pane(app)).width(Length::FillPortion(3));

    let panes = row![left, center, right]
        .width(Length::Fill)
        .height(Length::Fill);

    column![panes, status_bar(app)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Build the modal overlay when a search modal is active.
fn modal_overlay<'a>(app: &'a App, modal: &'a crate::modal::SearchModal) -> Element<'a, Message> {
    let mut modal_items: Vec<Element<'a, Message>> = Vec::new();

    // Composite part label if present
    if let Some(part_label) = modal.current_part_label() {
        modal_items.push(text(part_label).color(theme::MODAL).into());
    }

    // Search query input
    modal_items.push(
        text(format!("Search: {}", &modal.query))
            .color(theme::MODAL)
            .into(),
    );

    // Filtered item list (visible window)
    let end = (modal.list_scroll + modal.window_size).min(modal.filtered.len());
    let mut list_items: Vec<Element<'a, Message>> = Vec::new();
    for window_pos in modal.list_scroll..end {
        if let Some(&entry_idx) = modal.filtered.get(window_pos) {
            let label = &modal.all_entries[entry_idx];
            let color = if window_pos == modal.list_cursor {
                theme::SELECTED
            } else {
                theme::MUTED
            };
            list_items.push(text(label).color(color).into());
        }
    }

    modal_items.push(
        scrollable(column(list_items))
            .height(Length::Fill)
            .into(),
    );

    let modal_panel = container(column(modal_items).width(Length::Fixed(400.0)).padding(8))
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    // Stack the modal on top of the main layout
    let base = main_layout(app);
    Stack::new()
        .push(base)
        .push(modal_panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Public entry point called from main.rs view().
pub fn view(app: &App) -> Element<'_, Message> {
    if let Some(ref modal) = app.modal {
        modal_overlay(app, modal)
    } else {
        main_layout(app)
    }
}
