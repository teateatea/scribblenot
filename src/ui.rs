use crate::app::{App, Focus, MapHintLevel, SectionState};
use crate::modal::ModalFocus;
use crate::sections::block_select::BlockSelectFocus;
use crate::sections::free_text::FreeTextMode;
use crate::sections::list_select::ListSelectMode;
use crate::Message;
use iced::widget::{
    button, column, container, row, scrollable, text, text_input, Scrollable, Stack,
};
use iced::{Background, Border, Color};
use iced::{Element, Length};

pub fn preview_scroll_id() -> scrollable::Id {
    scrollable::Id::new("note-preview")
}

pub fn map_scroll_id() -> scrollable::Id {
    scrollable::Id::new("map-pane")
}

pub fn wizard_scroll_id() -> scrollable::Id {
    scrollable::Id::new("wizard-pane")
}

#[derive(Debug, Clone, Copy)]
enum HintTarget {
    Group(usize),
    Section(usize),
    HeaderField,
}

/// Build the left pane: section map showing section labels.
fn map_pane(app: &App) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = Vec::new();
    let active_map_group = match app.focus {
        Focus::Map => match app.map_hint_level {
            MapHintLevel::Groups => None,
            MapHintLevel::Sections(group_idx) => Some(group_idx),
        },
        Focus::Wizard => Some(app.group_idx_for_section(app.current_idx)),
    };
    let map_labels = app.map_hint_labels(active_map_group);

    let mut flat_idx = 0usize;
    for (group_idx, group) in app.data.groups.iter().enumerate() {
        let group_hint = map_labels
            .groups
            .get(group_idx)
            .cloned()
            .unwrap_or_default();
        let group_hint = display_hint_label(app, &group_hint);
        let group_target = HintTarget::Group(group_idx);
        let group_hint_active = hint_is_active(app, group_target);
        let group_hint_color = hint_color(app, group_target);
        let group_color = if group_idx == app.group_idx_for_section(app.current_idx) {
            app.ui_theme.active
        } else {
            app.ui_theme.muted
        };
        items.push(hinted_line(
            "",
            group_hint,
            group.name.clone(),
            group_hint_color,
            group_hint_active,
            group_color,
            app,
        ));

        let section_labels = if active_map_group == Some(group_idx) {
            map_labels.sections.clone()
        } else {
            app.map_hint_labels(Some(group_idx)).sections
        };
        for (group_section_idx, sec) in group.sections.iter().enumerate() {
            let mut label = sec.map_label.clone();
            if app.section_is_completed(flat_idx) {
                label.push_str(" [done]");
            } else if app.section_is_skipped(flat_idx) {
                label.push_str(" [skip]");
            }

            let marker = if flat_idx == app.map_cursor && app.focus == Focus::Map {
                ">"
            } else if flat_idx == app.current_idx {
                "*"
            } else {
                " "
            };
            let label_color = if flat_idx == app.map_cursor && app.focus == Focus::Map {
                app.ui_theme.active
            } else if flat_idx == app.current_idx {
                app.ui_theme.selected
            } else {
                app.ui_theme.muted
            };
            let target = HintTarget::Section(flat_idx);
            let active = hint_is_active(app, target);
            let color = hint_color(app, target);
            let hint = section_labels
                .get(group_section_idx)
                .cloned()
                .unwrap_or_default();
            let hint = display_hint_label(app, &hint);
            items.push(hinted_line(
                marker,
                hint,
                label,
                color,
                active,
                label_color,
                app,
            ));
            flat_idx += 1;
        }
    }

    themed_scrollable(app, column(items).spacing(4))
        .id(map_scroll_id())
        .into()
}

fn display_hint_label(app: &App, hint: &str) -> String {
    if app.config.hint_labels_capitalized {
        hint.to_ascii_uppercase()
    } else {
        hint.to_string()
    }
}

fn hint_color(app: &App, target: HintTarget) -> Color {
    if hint_is_active(app, target) {
        app.ui_theme.hint
    } else {
        app.ui_theme.muted
    }
}

fn hint_is_active(app: &App, target: HintTarget) -> bool {
    if app.modal.is_some() || app.show_help {
        return false;
    }

    match app.focus {
        Focus::Map => match target {
            HintTarget::Group(_) => true,
            HintTarget::Section(flat_idx) => match app.map_hint_level {
                MapHintLevel::Groups => false,
                MapHintLevel::Sections(active_group) => {
                    app.group_idx_for_section(flat_idx) == active_group
                }
            },
            HintTarget::HeaderField => false,
        },
        Focus::Wizard => match target {
            HintTarget::Group(group_idx) => {
                wizard_map_hints_active(app)
                    && group_idx == app.group_idx_for_section(app.current_idx)
            }
            HintTarget::Section(flat_idx) => {
                wizard_map_hints_active(app) && flat_idx == app.current_idx
            }
            HintTarget::HeaderField => matches!(
                app.section_states.get(app.current_idx),
                Some(SectionState::Header(_))
            ),
        },
    }
}

fn wizard_map_hints_active(app: &App) -> bool {
    match app.section_states.get(app.current_idx) {
        Some(SectionState::Header(_)) => true,
        Some(SectionState::FreeText(state)) => !state.is_editing(),
        Some(SectionState::ListSelect(state)) => matches!(state.mode, ListSelectMode::Browsing),
        Some(SectionState::BlockSelect(state)) => !state.in_items(),
        Some(SectionState::Checklist(_)) => true,
        Some(SectionState::Pending) | None => false,
    }
}

fn hinted_line<'a>(
    marker: &'a str,
    hint: String,
    label: String,
    hint_color: Color,
    hint_active: bool,
    label_color: Color,
    app: &'a App,
) -> Element<'a, Message> {
    row![
        text(format!("{marker} ")).color(label_color),
        hint_label(app, hint, hint_color, hint_active),
        text(label).color(label_color),
    ]
    .spacing(2)
    .into()
}

fn hint_label<'a>(
    app: &'a App,
    hint: String,
    base_color: Color,
    hint_active: bool,
) -> Element<'a, Message> {
    if !hint_active || app.hint_buffer.is_empty() {
        return text(format!("{hint:<3}")).color(base_color).into();
    }

    let normalized_hint = normalize_hint_for_match(app, &hint);
    let normalized_buffer = normalize_hint_for_match(app, &app.hint_buffer);
    if !normalized_hint.starts_with(&normalized_buffer) {
        return text(format!("{hint:<3}")).color(app.ui_theme.muted).into();
    }

    let mut chars: Vec<Element<'a, Message>> = Vec::new();
    for (idx, ch) in hint.chars().enumerate() {
        let color = if idx < normalized_buffer.chars().count() {
            app.ui_theme.hint_prefix
        } else {
            base_color
        };
        chars.push(text(ch.to_string()).color(color).into());
    }
    while chars.len() < 3 {
        chars.push(text(" ").color(base_color).into());
    }

    row(chars).spacing(0).into()
}

fn normalize_hint_for_match(app: &App, value: &str) -> String {
    if app.config.hint_labels_case_sensitive {
        value.to_string()
    } else {
        value.to_ascii_lowercase()
    }
}

fn background_style(color: Color, text_color: Color) -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(color)
        .color(text_color)
}

fn pane_style(app: &App, active: bool) -> iced::widget::container::Style {
    let background = if active {
        app.ui_theme.pane_active_background
    } else {
        app.ui_theme.pane_inactive_background
    };
    let border = if active {
        app.ui_theme.pane_active_border
    } else {
        app.ui_theme.pane_inactive_border
    };

    background_style(background, app.ui_theme.text).border(Border {
        color: border,
        width: app.ui_theme.pane_border_width,
        radius: 0.0.into(),
    })
}

fn scrollable_style(
    app_theme: &crate::theme::AppTheme,
    status: iced::widget::scrollable::Status,
) -> iced::widget::scrollable::Style {
    let (rail_color, scroller_color) = match status {
        iced::widget::scrollable::Status::Active => {
            (app_theme.scroll_rail, app_theme.scroll_scroller)
        }
        iced::widget::scrollable::Status::Hovered { .. } => (
            app_theme.scroll_rail_hovered,
            app_theme.scroll_scroller_hovered,
        ),
        iced::widget::scrollable::Status::Dragged { .. } => (
            app_theme.scroll_rail_dragged,
            app_theme.scroll_scroller_dragged,
        ),
    };
    let rail = iced::widget::scrollable::Rail {
        background: Some(Background::Color(rail_color)),
        border: Border {
            color: rail_color,
            width: app_theme.scroll_border_width,
            radius: 2.0.into(),
        },
        scroller: iced::widget::scrollable::Scroller {
            color: scroller_color,
            border: Border {
                color: scroller_color,
                width: app_theme.scroll_border_width,
                radius: 2.0.into(),
            },
        },
    };

    iced::widget::scrollable::Style {
        container: iced::widget::container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: Some(Background::Color(app_theme.scroll_gap)),
    }
}

fn modal_item_button_style(
    app_theme: &crate::theme::AppTheme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let background = match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            app_theme.modal_item_hovered_background
        }
        iced::widget::button::Status::Active | iced::widget::button::Status::Disabled => {
            app_theme.modal_item_background
        }
    };

    iced::widget::button::Style {
        background: Some(Background::Color(background)),
        text_color: app_theme.modal_text,
        border: Border {
            color: background,
            width: 0.0,
            radius: 2.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

fn modal_input_style(
    app_theme: &crate::theme::AppTheme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let border_color = match status {
        iced::widget::text_input::Status::Focused => app_theme.modal_hint_text,
        iced::widget::text_input::Status::Hovered => app_theme.modal_text,
        iced::widget::text_input::Status::Active | iced::widget::text_input::Status::Disabled => {
            app_theme.modal_input_border
        }
    };

    iced::widget::text_input::Style {
        background: Background::Color(app_theme.modal_input_background),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 2.0.into(),
        },
        icon: app_theme.modal_muted_text,
        placeholder: app_theme.modal_input_placeholder,
        value: app_theme.modal_input_text,
        selection: app_theme.modal_item_hovered_background,
    }
}

fn themed_scrollable<'a>(
    app: &'a App,
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    let app_theme = app.ui_theme.clone();
    let scrollbar = iced::widget::scrollable::Scrollbar::new()
        .width(app.ui_theme.scroll_width)
        .scroller_width(app.ui_theme.scroll_width)
        .spacing(app.ui_theme.scroll_spacing);
    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(iced::widget::scrollable::Direction::Vertical(scrollbar))
        .style(move |_theme, status| scrollable_style(&app_theme, status))
}

fn header_field_hint_labels(app: &App) -> Vec<String> {
    let field_count = match app.section_states.get(app.current_idx) {
        Some(SectionState::Header(state)) => state.field_configs.len(),
        _ => 0,
    };
    app.wizard_hint_labels(field_count).fields
}

fn wizard_window(app: &App, cursor: usize, len: usize) -> std::ops::Range<usize> {
    if len == 0 {
        return 0..0;
    }
    let window_size = app.data.keybindings.hints.len().max(1);
    let start = if cursor >= window_size {
        cursor + 1 - window_size
    } else {
        0
    };
    let end = (start + window_size).min(len);
    start..end
}

fn composite_part_display_value(
    field_id: &str,
    part: &crate::data::CompositePart,
    app: &App,
) -> String {
    if part.sticky {
        let key = format!("{field_id}.{}", part.id);
        if let Some(value) = app.config.sticky_values.get(&key) {
            if !value.is_empty() {
                return value.clone();
            }
        }
    }

    if part.default.is_some() {
        if let Some(option) = part.options.get(part.default_cursor()) {
            return option.output().to_string();
        }
    }

    part.preview.clone().unwrap_or_default()
}

fn field_display_value(
    app: &App,
    field: &crate::data::HeaderFieldConfig,
    confirmed_values: &[String],
) -> String {
    if !confirmed_values.is_empty() {
        return confirmed_values.join(", ");
    }

    if let Some(composite) = &field.composite {
        let mut display = composite.format.clone();
        let mut has_value = false;
        for part in &composite.parts {
            let value = composite_part_display_value(&field.id, part, app);
            if !value.is_empty() {
                has_value = true;
            }
            display = display.replace(&format!("{{{}}}", part.id), &value);
        }
        if has_value {
            return display;
        }
    } else {
        if let Some(value) = app.config.sticky_values.get(&field.id) {
            if !value.is_empty() {
                return value.clone();
            }
        }
        if let Some(default) = &field.default {
            if !default.is_empty() {
                return default.clone();
            }
        }
    }

    "[empty]".to_string()
}

fn section_header(app: &App) -> Vec<Element<'_, Message>> {
    let sec = &app.sections[app.current_idx];
    let group_idx = app.group_idx_for_section(app.current_idx);
    let group_name = app
        .data
        .groups
        .get(group_idx)
        .map(|g| g.name.as_str())
        .unwrap_or("Unknown");

    let mut items = vec![
        text(&sec.name).size(24).color(app.ui_theme.active).into(),
        text(format!("{} ({})", sec.id, group_name))
            .color(app.ui_theme.muted)
            .into(),
    ];

    if app.section_is_completed(app.current_idx) {
        items.push(
            text("Status: completed")
                .color(app.ui_theme.selected)
                .into(),
        );
    } else if app.section_is_skipped(app.current_idx) {
        items.push(text("Status: skipped").color(app.ui_theme.muted).into());
    }

    items
}

fn render_header_state<'a>(
    app: &'a App,
    sec: &'a crate::data::SectionConfig,
    state: &'a crate::sections::header::HeaderState,
) -> Element<'a, Message> {
    let mut items = section_header(app);

    items.push(text("Fields").color(app.ui_theme.modal).into());
    let field_hint_active = hint_is_active(app, HintTarget::HeaderField);
    let field_hint_color = hint_color(app, HintTarget::HeaderField);
    let field_range = wizard_window(app, state.field_index, state.field_configs.len());
    if field_range.start > 0 {
        items.push(text("...").color(app.ui_theme.muted).into());
    }
    let field_hints = header_field_hint_labels(app);
    for idx in field_range.clone() {
        let Some(field) = state.field_configs.get(idx) else {
            continue;
        };
        let values = state
            .repeated_values
            .get(idx)
            .map(|confirmed_values| field_display_value(app, field, confirmed_values))
            .unwrap_or_else(|| field_display_value(app, field, &[]));
        let prefix = if idx == state.field_index { ">" } else { " " };
        let color = if idx == state.field_index {
            app.ui_theme.active
        } else if state
            .repeated_values
            .get(idx)
            .map(|vals| !vals.is_empty())
            .unwrap_or(false)
        {
            app.ui_theme.selected
        } else {
            app.ui_theme.muted
        };
        let field_hint = field_hints
            .get(idx)
            .map(|hint| display_hint_label(app, hint))
            .unwrap_or_default();
        items.push(
            row![
                text(format!("{prefix} ")).color(color),
                hint_label(app, field_hint, field_hint_color, field_hint_active),
                text(format!("{}: {}", field.name, values)).color(color),
            ]
            .spacing(2)
            .into(),
        );
    }
    if field_range.end < state.field_configs.len() {
        items.push(text("...").color(app.ui_theme.muted).into());
    }

    if sec
        .fields
        .as_ref()
        .map(|fields| fields.is_empty())
        .unwrap_or(true)
    {
        items.push(
            text("No fields configured")
                .color(app.ui_theme.error)
                .into(),
        );
    }

    themed_scrollable(app, column(items).spacing(4))
        .id(wizard_scroll_id())
        .into()
}

fn render_free_text_state<'a>(
    app: &'a App,
    state: &'a crate::sections::free_text::FreeTextState,
) -> Element<'a, Message> {
    let mut items = section_header(app);

    if state.is_editing() {
        items.push(text("Editing buffer").color(app.ui_theme.modal).into());
        items.push(text(&state.edit_buf).color(app.ui_theme.active).into());
    }

    items.push(text("Entries").color(app.ui_theme.modal).into());
    if state.entries.is_empty() {
        items.push(text("[no entries yet]").color(app.ui_theme.muted).into());
    } else {
        let range = wizard_window(app, state.cursor, state.entries.len());
        if range.start > 0 {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
        for idx in range.clone() {
            let Some(entry) = state.entries.get(idx) else {
                continue;
            };
            let prefix = if idx == state.cursor { ">" } else { " " };
            let color = if idx == state.cursor {
                app.ui_theme.active
            } else {
                app.ui_theme.muted
            };
            items.push(text(format!("{prefix} {entry}")).color(color).into());
        }
        if range.end < state.entries.len() {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
    }

    themed_scrollable(app, column(items).spacing(4))
        .id(wizard_scroll_id())
        .into()
}

fn render_list_select_state<'a>(
    app: &'a App,
    state: &'a crate::sections::list_select::ListSelectState,
) -> Element<'a, Message> {
    let mut items = section_header(app);

    match state.mode {
        ListSelectMode::AddingLabel => {
            items.push(text("New label").color(app.ui_theme.modal).into());
            items.push(text(&state.add_label_buf).color(app.ui_theme.active).into());
        }
        ListSelectMode::AddingOutput => {
            items.push(text("New label").color(app.ui_theme.selected).into());
            items.push(text(&state.add_label_buf).into());
            items.push(text("New output").color(app.ui_theme.modal).into());
            items.push(
                text(&state.add_output_buf)
                    .color(app.ui_theme.active)
                    .into(),
            );
        }
        ListSelectMode::Browsing => {}
    }

    items.push(text("Options").color(app.ui_theme.modal).into());
    if state.entries.is_empty() {
        items.push(text("[no options loaded]").color(app.ui_theme.error).into());
    } else {
        let range = wizard_window(app, state.cursor, state.entries.len());
        if range.start > 0 {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
        for idx in range.clone() {
            let Some(entry) = state.entries.get(idx) else {
                continue;
            };
            let marker = if state.is_selected(idx) { "[x]" } else { "[ ]" };
            let prefix = if idx == state.cursor { ">" } else { " " };
            let color = if idx == state.cursor {
                app.ui_theme.active
            } else if state.is_selected(idx) {
                app.ui_theme.selected
            } else {
                app.ui_theme.muted
            };
            items.push(
                text(format!(
                    "{prefix} {marker} {} -> {}",
                    entry.label, entry.output
                ))
                .color(color)
                .into(),
            );
        }
        if range.end < state.entries.len() {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
    }

    themed_scrollable(app, column(items).spacing(4))
        .id(wizard_scroll_id())
        .into()
}

fn render_block_select_state<'a>(
    app: &'a App,
    state: &'a crate::sections::block_select::BlockSelectState,
) -> Element<'a, Message> {
    let mut items = section_header(app);

    items.push(text("Groups").color(app.ui_theme.modal).into());
    if state.groups.is_empty() {
        items.push(text("[no groups loaded]").color(app.ui_theme.error).into());
    } else {
        let group_range = wizard_window(app, state.group_cursor, state.groups.len());
        if group_range.start > 0 {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
        for idx in group_range.clone() {
            let Some(group) = state.groups.get(idx) else {
                continue;
            };
            let selected_count = group
                .item_selected
                .iter()
                .filter(|&&selected| selected)
                .count();
            let prefix =
                if idx == state.group_cursor && matches!(state.focus, BlockSelectFocus::Groups) {
                    ">"
                } else {
                    " "
                };
            let color = if idx == state.group_cursor {
                app.ui_theme.active
            } else if selected_count > 0 {
                app.ui_theme.selected
            } else {
                app.ui_theme.muted
            };
            items.push(
                text(format!(
                    "{prefix} {} ({}/{})",
                    group.label,
                    selected_count,
                    group.entries.len()
                ))
                .color(color)
                .into(),
            );
        }
        if group_range.end < state.groups.len() {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
    }

    if let Some(group_idx) = state.current_group_idx().or({
        if state.groups.is_empty() {
            None
        } else {
            Some(state.group_cursor)
        }
    }) {
        if let Some(group) = state.groups.get(group_idx) {
            items.push(
                text(format!("Items: {}", group.header))
                    .color(app.ui_theme.modal)
                    .into(),
            );
            let item_range = wizard_window(app, state.item_cursor, group.entries.len());
            if item_range.start > 0 {
                items.push(text("...").color(app.ui_theme.muted).into());
            }
            for idx in item_range.clone() {
                let Some(item) = group.entries.get(idx) else {
                    continue;
                };
                let checked = group.item_selected.get(idx).copied().unwrap_or(false);
                let marker = if checked { "[x]" } else { "[ ]" };
                let prefix = if idx == state.item_cursor
                    && matches!(state.focus, BlockSelectFocus::Items(current) if current == group_idx)
                {
                    ">"
                } else {
                    " "
                };
                let color = if idx == state.item_cursor
                    && matches!(state.focus, BlockSelectFocus::Items(current) if current == group_idx)
                {
                    app.ui_theme.active
                } else if checked {
                    app.ui_theme.selected
                } else {
                    app.ui_theme.muted
                };
                items.push(
                    text(format!("{prefix} {marker} {}", item.label()))
                        .color(color)
                        .into(),
                );
            }
            if item_range.end < group.entries.len() {
                items.push(text("...").color(app.ui_theme.muted).into());
            }
        }
    }

    themed_scrollable(app, column(items).spacing(4))
        .id(wizard_scroll_id())
        .into()
}

fn render_checklist_state<'a>(
    app: &'a App,
    state: &'a crate::sections::checklist::ChecklistState,
) -> Element<'a, Message> {
    let mut items = section_header(app);

    items.push(text("Checklist").color(app.ui_theme.modal).into());
    if state.items.is_empty() {
        items.push(
            text("[no checklist items loaded]")
                .color(app.ui_theme.error)
                .into(),
        );
    } else {
        let range = wizard_window(app, state.cursor, state.items.len());
        if range.start > 0 {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
        for idx in range.clone() {
            let Some(item) = state.items.get(idx) else {
                continue;
            };
            let checked = state.checked.get(idx).copied().unwrap_or(false);
            let marker = if checked { "[x]" } else { "[ ]" };
            let prefix = if idx == state.cursor { ">" } else { " " };
            let color = if idx == state.cursor {
                app.ui_theme.active
            } else if checked {
                app.ui_theme.selected
            } else {
                app.ui_theme.muted
            };
            items.push(
                text(format!("{prefix} {marker} {item}"))
                    .color(color)
                    .into(),
            );
        }
        if range.end < state.items.len() {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
    }

    themed_scrollable(app, column(items).spacing(4))
        .id(wizard_scroll_id())
        .into()
}

/// Build the center pane: minimal real wizard area for the current section.
fn wizard_pane(app: &App) -> Element<'_, Message> {
    if app.sections.is_empty() || app.current_idx >= app.sections.len() {
        return column![text("No sections loaded").color(app.ui_theme.error)]
            .width(Length::Fill)
            .padding(4)
            .into();
    }

    let sec = &app.sections[app.current_idx];
    match &app.section_states[app.current_idx] {
        SectionState::Header(state) => render_header_state(app, sec, state),
        SectionState::FreeText(state) => render_free_text_state(app, state),
        SectionState::ListSelect(state) => render_list_select_state(app, state),
        SectionState::BlockSelect(state) => render_block_select_state(app, state),
        SectionState::Checklist(state) => render_checklist_state(app, state),
        SectionState::Pending => themed_scrollable(
            app,
            column(section_header(app))
                .push(text("Pending section state").color(app.ui_theme.muted))
                .spacing(4),
        )
        .id(wizard_scroll_id())
        .into(),
    }
}

/// Build the right pane: editable note with heading validity warning.
fn editor_pane(app: &App) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = Vec::new();

    if let Some(ref warning) = app.note_structure_warning {
        items.push(
            text(format!("Structure warning: {}", warning))
                .color(app.ui_theme.error)
                .into(),
        );
    }

    items.push(text("Preview").color(app.ui_theme.modal).into());
    let preview_text = crate::document::export_editable_document(&app.editable_note);
    items.push(
        themed_scrollable(app, text(preview_text).size(14).width(Length::Fill))
            .id(preview_scroll_id())
            .height(Length::Fill)
            .into(),
    );

    column(items)
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(4)
        .into()
}

/// Build the status bar at the bottom.
fn status_bar(app: &App) -> Element<'_, Message> {
    let context = panel_status_text(app);
    let (status_text, color) = if let Some(status) = app.status.as_ref() {
        let color = if status.is_error {
            app.ui_theme.error
        } else {
            app.ui_theme.muted
        };
        if status.is_error || context.is_empty() {
            (status.text.clone(), color)
        } else {
            (format!("{} | {}", status.text, context), color)
        }
    } else {
        (context, app.ui_theme.muted)
    };
    text(status_text).color(color).into()
}

fn panel_status_text(app: &App) -> String {
    if let Some(modal) = app.modal.as_ref() {
        let focus = match modal.focus {
            ModalFocus::SearchBar => "search",
            ModalFocus::List => "list",
        };
        return format!(
            "Modal: {focus} | {} matches | Enter selects | Esc closes",
            modal.filtered.len()
        );
    }

    match app.focus {
        Focus::Map => map_status_text(app),
        Focus::Wizard => wizard_status_text(app),
    }
}

fn map_status_text(app: &App) -> String {
    let mode = match app.map_hint_level {
        MapHintLevel::Groups => "groups",
        MapHintLevel::Sections(_) => "sections",
    };
    let mut parts = vec![format!("Map: {mode}")];
    if !app.hint_buffer.is_empty() {
        parts.push(format!("Hint: {}", app.hint_buffer));
    }
    parts.push("Enter previews | Esc returns".to_string());
    parts.join(" | ")
}

fn wizard_status_text(app: &App) -> String {
    let Some(sec) = app.sections.get(app.current_idx) else {
        return "Wizard: no section".to_string();
    };
    let mut parts = vec![format!("Wizard: {}", sec.name)];
    parts.push(format!("Type: {}", sec.section_type));

    if app.section_is_completed(app.current_idx) {
        parts.push("completed".to_string());
    } else if app.section_is_skipped(app.current_idx) {
        parts.push("skipped".to_string());
    }

    match app.section_states.get(app.current_idx) {
        Some(SectionState::Header(state)) => {
            if let Some(field) = state.field_configs.get(state.field_index) {
                parts.push(format!("Field: {}", field.name));
                if let Some(limit) = field.repeat_limit {
                    parts.push(format!(
                        "Repeat: {}/{}",
                        state.repeat_counts[state.field_index], limit
                    ));
                }
            }
            parts.push("Enter opens choices".to_string());
        }
        Some(SectionState::FreeText(state)) => {
            let mode = match state.mode {
                FreeTextMode::Browsing => "browsing",
                FreeTextMode::Editing => "editing",
            };
            parts.push(format!("Mode: {mode}"));
            parts.push(format!("Entries: {}", state.entries.len()));
        }
        Some(SectionState::ListSelect(state)) => {
            let mode = match state.mode {
                ListSelectMode::Browsing => "browsing",
                ListSelectMode::AddingLabel => "adding label",
                ListSelectMode::AddingOutput => "adding output",
            };
            parts.push(format!("Mode: {mode}"));
            parts.push(format!("Selected: {}", state.selected_indices.len()));
        }
        Some(SectionState::BlockSelect(state)) => {
            let mode = match state.focus {
                BlockSelectFocus::Groups => "groups",
                BlockSelectFocus::Items(_) => "items",
            };
            parts.push(format!("Mode: {mode}"));
            parts.push("Enter opens group | Space toggles".to_string());
        }
        Some(SectionState::Checklist(state)) => {
            let checked = state.checked.iter().filter(|&&checked| checked).count();
            parts.push(format!("Checked: {checked}"));
            parts.push("Space toggles | Enter confirms".to_string());
        }
        Some(SectionState::Pending) | None => {}
    }

    if !app.hint_buffer.is_empty() {
        parts.push(format!("Hint: {}", app.hint_buffer));
    }

    parts.join(" | ")
}

/// Build the three-pane main layout.
fn main_layout(app: &App) -> Element<'_, Message> {
    let buffer = app.ui_theme.pane_buffer_width;
    let map_pane_active = app.focus == Focus::Map;
    let map = container(map_pane(app))
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .padding(buffer)
        .style(move |_| pane_style(app, map_pane_active));
    let wizard_pane_active = app.focus == Focus::Wizard;
    let wizard = container(wizard_pane(app))
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .padding(buffer)
        .style(move |_| pane_style(app, wizard_pane_active));
    let editor = container(editor_pane(app))
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .padding(buffer)
        .style(move |_| pane_style(app, false));

    let panes = if app.pane_swapped {
        row![wizard, editor, map]
    } else {
        row![map, wizard, editor]
    }
    .width(Length::Fill)
    .height(Length::Fill);

    let status_background = app.ui_theme.status_background;
    let text_color = app.ui_theme.text;
    let status = container(status_bar(app))
        .width(Length::Fill)
        .padding([2.0, buffer])
        .style(move |_| background_style(status_background, text_color));

    let background = app.ui_theme.background;
    let text_color = app.ui_theme.text;
    container(column![panes, status])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| background_style(background, text_color))
        .into()
}

/// Build the modal overlay when a search modal is active.
fn modal_overlay<'a>(app: &'a App, modal: &'a crate::modal::SearchModal) -> Element<'a, Message> {
    let mut modal_items: Vec<Element<'a, Message>> = Vec::new();

    if let Some(part_label) = modal.current_part_label() {
        modal_items.push(text(part_label).color(app.ui_theme.modal_text).into());
    }

    let app_theme = app.ui_theme.clone();
    modal_items.push(
        text_input("Search", &modal.query)
            .on_input(Message::ModalQueryChanged)
            .width(Length::Fill)
            .style(move |_theme, status| modal_input_style(&app_theme, status))
            .into(),
    );

    let end = (modal.list_scroll + modal.window_size).min(modal.filtered.len());
    let mut list_items: Vec<Element<'a, Message>> = Vec::new();
    let modal_hints: Vec<String> = app
        .data
        .keybindings
        .hints
        .iter()
        .take(end.saturating_sub(modal.list_scroll))
        .cloned()
        .collect();
    for window_pos in modal.list_scroll..end {
        if let Some(&entry_idx) = modal.filtered.get(window_pos) {
            let label = &modal.all_entries[entry_idx];
            let color = if window_pos == modal.list_cursor {
                app.ui_theme.modal_selected_text
            } else {
                app.ui_theme.modal_muted_text
            };
            let hint = modal_hints
                .get(window_pos - modal.list_scroll)
                .map(|hint| display_hint_label(app, hint))
                .unwrap_or_default();
            let hint_color = if matches!(modal.focus, ModalFocus::List) {
                app.ui_theme.modal_hint_text
            } else {
                app.ui_theme.modal_muted_text
            };
            let button_label = row![
                text(format!("{hint:<3}")).color(hint_color),
                text(label).color(color),
            ]
            .spacing(6);
            let app_theme = app.ui_theme.clone();
            list_items.push(
                button(button_label)
                    .width(Length::Fill)
                    .on_press(Message::ModalSelect(window_pos))
                    .style(move |_theme, status| modal_item_button_style(&app_theme, status))
                    .into(),
            );
        }
    }

    modal_items.push(
        themed_scrollable(app, column(list_items))
            .height(Length::Fill)
            .into(),
    );

    let modal_background = app.ui_theme.modal_panel_background;
    let text_color = app.ui_theme.text;
    let modal_panel = container(column(modal_items).width(Length::Fixed(400.0)).padding(8))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_| background_style(modal_background, text_color));

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
