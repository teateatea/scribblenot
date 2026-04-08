use crate::app::{App, Focus, MapHintLevel, SectionState};
use crate::modal::{modal_height_for_viewport, ModalFocus};
use crate::sections::block_select::BlockSelectFocus;
use crate::sections::collection::CollectionFocus;
use crate::sections::free_text::FreeTextMode;
use crate::sections::list_select::ListSelectMode;
use crate::Message;
use iced::widget::{
    button, column, container, rich_text, row, scrollable, span, text, text_input, Scrollable,
    Space, Stack,
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
    Section,
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
        let group_color = app.ui_theme.text;
        items.push(
            text(group.name.clone())
                .font(app.ui_theme.font_pane)
                .color(group_color)
                .into(),
        );

        let section_labels = if active_map_group == Some(group_idx) {
            map_labels.sections.clone()
        } else {
            app.map_hint_labels(Some(group_idx)).sections
        };
        for (group_section_idx, sec) in group.sections.iter().enumerate() {
            let mut label = sec.map_label.clone();
            if app.section_is_skipped(flat_idx) {
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
            } else if flat_idx == app.current_idx && app.focus == Focus::Wizard {
                app.ui_theme.active_preview
            } else if flat_idx == app.current_idx {
                app.ui_theme.selected
            } else {
                app.ui_theme.muted
            };
            let target = HintTarget::Section;
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
            HintTarget::Section => true,
            HintTarget::HeaderField => false,
        },
        Focus::Wizard => match target {
            HintTarget::Section => wizard_map_hints_active(app),
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
        Some(SectionState::Collection(state)) => !state.in_items(),
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
        marker_cell(marker, label_color, app.ui_theme.font_pane),
        hint_label(app, hint, hint_color, hint_active),
        text(label).font(app.ui_theme.font_pane).color(label_color),
    ]
    .spacing(0)
    .into()
}

fn marker_cell(marker: &str, color: Color, font: iced::Font) -> Element<'_, Message> {
    container(text(marker.to_string()).color(color).font(font))
        .width(Length::Fixed(14.0))
        .align_left(Length::Fixed(14.0))
        .into()
}

fn hint_label<'a>(
    app: &'a App,
    hint: String,
    base_color: Color,
    hint_active: bool,
) -> Element<'a, Message> {
    if !hint_active || app.hint_buffer.is_empty() {
        return fixed_hint_label(
            text(format!("{hint:<4}"))
                .font(app.ui_theme.font_pane)
                .color(base_color),
        );
    }

    let normalized_hint = normalize_hint_for_match(app, &hint);
    let normalized_buffer = normalize_hint_for_match(app, &app.hint_buffer);
    if !normalized_hint.starts_with(&normalized_buffer) {
        return fixed_hint_label(
            text(format!("{hint:<4}"))
                .font(app.ui_theme.font_pane)
                .color(app.ui_theme.muted),
        );
    }

    let mut chars: Vec<Element<'a, Message>> = Vec::new();
    for (idx, ch) in hint.chars().enumerate() {
        let color = if idx < normalized_buffer.chars().count() {
            app.ui_theme.hint_prefix
        } else {
            base_color
        };
        chars.push(
            text(ch.to_string())
                .font(app.ui_theme.font_pane)
                .color(color)
                .into(),
        );
    }
    while chars.len() < 4 {
        chars.push(
            text(" ")
                .font(app.ui_theme.font_pane)
                .color(base_color)
                .into(),
        );
    }

    fixed_hint_label(row(chars).spacing(0))
}

fn fixed_hint_label<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content).align_left(Length::Fixed(24.0)).into()
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
        Some(SectionState::Header(state)) => state.visible_row_count(),
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

fn field_display_value(
    app: &App,
    field: &crate::data::HeaderFieldConfig,
    confirmed_values: &[String],
) -> String {
    if !confirmed_values.is_empty() {
        return confirmed_values.join(", ");
    }

    let mut values = Vec::new();
    for list in &field.lists {
        if list.sticky {
            if let Some(value) = app.config.sticky_values.get(&list.id) {
                if !value.is_empty() {
                    values.push((list.id.clone(), value.clone()));
                    continue;
                }
            }
        }
        if let Some(default) = &list.default {
            if let Some(item) = list.items.iter().find(|item| {
                item.id == *default
                    || item.label == *default
                    || item.output.as_deref() == Some(default.as_str())
            }) {
                values.push((
                    list.id.clone(),
                    item.output.clone().unwrap_or_else(|| item.label.clone()),
                ));
                continue;
            }
        }
        if let Some(preview) = &list.preview {
            values.push((list.id.clone(), preview.clone()));
        }
    }

    if values.is_empty() {
        return "[empty]".to_string();
    }

    if let Some(format) = &field.format {
        let mut display = format.clone();
        for (list_id, value) in values {
            display = display.replace(&format!("{{{}}}", list_id), &value);
        }
        for list in &field.format_lists {
            let placeholder = format!("{{{}}}", list.id);
            if !display.contains(&placeholder) {
                continue;
            }
            let value = if list.sticky {
                app.config
                    .sticky_values
                    .get(&list.id)
                    .filter(|value| !value.is_empty())
                    .cloned()
            } else {
                None
            }
            .or_else(|| {
                list.default.as_ref().and_then(|default| {
                    list.items
                        .iter()
                        .find(|item| {
                            item.id == *default
                                || item.label == *default
                                || item.output.as_deref() == Some(default.as_str())
                        })
                        .map(|item| item.output.clone().unwrap_or_else(|| item.label.clone()))
                })
            })
            .unwrap_or_default();
            display = display.replace(&placeholder, &value);
        }
        return display;
    }

    values
        .first()
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| "[empty]".to_string())
}

fn section_header(app: &App) -> Vec<Element<'_, Message>> {
    let sec = &app.sections[app.current_idx];

    let mut items = vec![text(&sec.name)
        .font(app.ui_theme.font_heading)
        .size(24)
        .color(if app.focus == Focus::Map {
            app.ui_theme.active_preview
        } else {
            app.ui_theme.active
        })
        .into()];

    if app.section_is_skipped(app.current_idx) {
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

    let field_hint_active = hint_is_active(app, HintTarget::HeaderField);
    let field_hint_color = hint_color(app, HintTarget::HeaderField);
    let field_range = wizard_window(app, state.field_index, state.field_configs.len());
    if field_range.start > 0 {
        items.push(text("...").color(app.ui_theme.muted).into());
    }
    let field_hints = header_field_hint_labels(app);
    let mut hint_idx: usize = (0..field_range.start)
        .map(|field_idx| state.visible_row_count_for_field(field_idx))
        .sum();
    for idx in field_range.clone() {
        let Some(field) = state.field_configs.get(idx) else {
            continue;
        };
        let confirmed_values = state
            .repeated_values
            .get(idx)
            .map(|values| values.as_slice())
            .unwrap_or(&[]);
        let base_color = if idx == state.field_index {
            if app.focus == Focus::Map {
                app.ui_theme.active_preview
            } else {
                app.ui_theme.active
            }
        } else if !confirmed_values.is_empty() {
            app.ui_theme.selected
        } else {
            app.ui_theme.muted
        };
        if let Some(limit) = field.repeat_limit {
            let active_value_index = state
                .repeat_counts
                .get(idx)
                .copied()
                .unwrap_or(0)
                .min(limit.saturating_sub(1));
            let mut visible_count = state.visible_value_count(idx);
            if idx == state.field_index && active_value_index >= visible_count {
                visible_count = active_value_index + 1;
            }
            visible_count = visible_count.max(1).min(limit.max(1));

            for repeat_idx in 0..visible_count {
                let is_active = idx == state.field_index && repeat_idx == active_value_index;
                let prefix = if is_active { ">" } else { " " };
                let color = if is_active {
                    base_color
                } else if repeat_idx < confirmed_values.len() {
                    app.ui_theme.selected
                } else {
                    app.ui_theme.muted
                };
                let value = confirmed_values
                    .get(repeat_idx)
                    .map(|value| field_display_value(app, field, std::slice::from_ref(value)))
                    .unwrap_or_else(|| field_display_value(app, field, &[]));
                let hint = field_hints
                    .get(hint_idx)
                    .map(|hint| display_hint_label(app, hint))
                    .unwrap_or_default();
                hint_idx += 1;
                items.push(hinted_line(
                    prefix,
                    hint,
                    format!("{} {}: {}", field.name, repeat_idx + 1, value),
                    field_hint_color,
                    field_hint_active,
                    color,
                    app,
                ));
            }
            continue;
        }

        let values = field_display_value(app, field, confirmed_values);
        let prefix = if idx == state.field_index { ">" } else { " " };
        let field_hint = field_hints
            .get(hint_idx)
            .map(|hint| display_hint_label(app, hint))
            .unwrap_or_default();
        hint_idx += 1;
        items.push(hinted_line(
            prefix,
            field_hint,
            format!("{}: {}", field.name, values),
            field_hint_color,
            field_hint_active,
            base_color,
            app,
        ));
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
                    text(format!("{prefix} {marker} {}", item.label))
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

fn render_collection_state<'a>(
    app: &'a App,
    state: &'a crate::sections::collection::CollectionState,
) -> Element<'a, Message> {
    let mut items = section_header(app);

    if state.collections.is_empty() {
        items.push(text("[no collections loaded]").color(app.ui_theme.error).into());
    } else {
        let collection_range = wizard_window(app, state.collection_cursor, state.collections.len());
        if collection_range.start > 0 {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
        for idx in collection_range.clone() {
            let Some(collection) = state.collections.get(idx) else {
                continue;
            };
            let enabled_count = collection.enabled_count();
            let marker = if collection.active { "[x]" } else { "[ ]" };
            let prefix = if idx == state.collection_cursor
                && matches!(state.focus, CollectionFocus::Collections)
            {
                ">"
            } else {
                " "
            };
            let color = if idx == state.collection_cursor {
                app.ui_theme.active
            } else if collection.active {
                app.ui_theme.selected
            } else {
                app.ui_theme.muted
            };
            items.push(
                text(format!(
                    "{prefix} {marker} {} ({}/{})",
                    collection.label,
                    enabled_count,
                    collection.items.len()
                ))
                .color(color)
                .into(),
            );
        }
        if collection_range.end < state.collections.len() {
            items.push(text("...").color(app.ui_theme.muted).into());
        }
    }

    let current_collection_idx = match state.focus {
        CollectionFocus::Items(idx) => Some(idx),
        CollectionFocus::Collections if !state.collections.is_empty() => Some(state.collection_cursor),
        CollectionFocus::Collections => None,
    };

    if let Some(collection_idx) = current_collection_idx {
        if let Some(collection) = state.collections.get(collection_idx) {
            items.push(text(format!("Items: {}", collection.label)).color(app.ui_theme.modal).into());
            let item_range = wizard_window(app, state.item_cursor, collection.items.len());
            if item_range.start > 0 {
                items.push(text("...").color(app.ui_theme.muted).into());
            }
            for idx in item_range.clone() {
                let Some(item) = collection.items.get(idx) else {
                    continue;
                };
                let enabled = collection.item_enabled.get(idx).copied().unwrap_or(false);
                let marker = if enabled { "[x]" } else { "[ ]" };
                let prefix = if idx == state.item_cursor
                    && matches!(state.focus, CollectionFocus::Items(current) if current == collection_idx)
                {
                    ">"
                } else {
                    " "
                };
                let color = if idx == state.item_cursor
                    && matches!(state.focus, CollectionFocus::Items(current) if current == collection_idx)
                {
                    app.ui_theme.active
                } else if enabled {
                    app.ui_theme.selected
                } else {
                    app.ui_theme.muted
                };
                items.push(
                    text(format!("{prefix} {marker} {}", item.label))
                        .color(color)
                        .into(),
                );
            }
            if item_range.end < collection.items.len() {
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
        SectionState::Collection(state) => render_collection_state(app, state),
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

    items.push(
        text("Preview")
            .font(app.ui_theme.font_heading)
            .size(24)
            .color(app.ui_theme.active)
            .into(),
    );
    let preview_content = preview_lines(app);
    items.push(
        themed_scrollable(
            app,
            container(
                column(preview_content)
                    .spacing(0)
                    .push(Space::with_height(Length::Fixed(1200.0))),
            )
            .padding([0.0, app.ui_theme.pane_buffer_width])
            .width(Length::Fill),
        )
        .id(preview_scroll_id())
        .height(Length::Fill)
        .into(),
    );

    let background = preview_background(app);
    let text_color = app.ui_theme.text;
    container(
        column(items)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(4),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| background_style(background, text_color))
    .into()
}

fn preview_background(app: &App) -> Color {
    let base = app.ui_theme.pane_inactive_background;
    let Some(until) = app.copy_flash_until else {
        return base;
    };
    let duration_ms = app.ui_theme.preview_copy_flash_duration_ms.max(1);
    let remaining_ms = until
        .saturating_duration_since(std::time::Instant::now())
        .as_millis()
        .min(u128::from(duration_ms)) as f32;
    let t = remaining_ms / duration_ms as f32;
    let eased = t * t * (3.0 - 2.0 * t);
    blend_color(base, app.ui_theme.preview_copy_flash_background, eased)
}

fn blend_color(base: Color, flash: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: base.r + (flash.r - base.r) * amount,
        g: base.g + (flash.g - base.g) * amount,
        b: base.b + (flash.b - base.b) * amount,
        a: base.a + (flash.a - base.a) * amount,
    }
}

fn preview_lines(app: &App) -> Vec<Element<'_, Message>> {
    let preview_text = crate::document::export_editable_document(&app.editable_note);
    let mut group_idx: Option<usize> = None;
    preview_text
        .lines()
        .map(|line| {
            if let Some(idx) = preview_group_for_line(app, line) {
                group_idx = Some(idx);
            }
            preview_line(app, line.to_string(), group_idx)
        })
        .collect()
}

fn preview_line(app: &App, line: String, group_idx: Option<usize>) -> Element<'_, Message> {
    if let Some(spans) = appointment_header_line_spans(app, &line, group_idx) {
        return rich_text::<Message, iced::Theme, iced::Renderer>(spans)
            .font(app.ui_theme.font_preview)
            .size(14)
            .width(Length::Fill)
            .into();
    }

    let color = preview_line_color(app, &line, group_idx);
    text(line)
        .font(app.ui_theme.font_preview)
        .size(14)
        .width(Length::Fill)
        .color(color)
        .into()
}

fn preview_line_color(app: &App, line: &str, group_idx: Option<usize>) -> Color {
    let in_current_group = group_idx
        .map(|group_idx| group_idx == app.group_idx_for_section(app.current_idx))
        .unwrap_or(true);

    if line.starts_with("## ") {
        return if in_current_group {
            app.ui_theme.modal
        } else {
            app.ui_theme.muted
        };
    }

    for (section_idx, section) in app.sections.iter().enumerate() {
        if crate::note::managed_heading_for_section(section).as_deref() == Some(line) {
            return section_preview_color(app, section_idx, in_current_group);
        }

        let Some(SectionState::Header(state)) = app.section_states.get(section_idx) else {
            continue;
        };
        for (field_idx, field) in state.field_configs.iter().enumerate() {
            let prefix = format!("{}: ", field.name);
            if line.starts_with(&prefix) {
                return header_field_preview_color(
                    app,
                    section_idx,
                    state,
                    field_idx,
                    in_current_group,
                );
            }
        }
    }

    if in_current_group {
        app.ui_theme.text
    } else {
        app.ui_theme.muted
    }
}

fn preview_group_for_line(app: &App, line: &str) -> Option<usize> {
    match line {
        "## SUBJECTIVE" => {
            return app
                .data
                .groups
                .iter()
                .position(|group| group.id == "subjective")
        }
        "## TREATMENT / PLAN" => {
            return app
                .data
                .groups
                .iter()
                .position(|group| group.id == "treatment")
        }
        "## OBJECTIVE / OBSERVATIONS" => {
            return app
                .data
                .groups
                .iter()
                .position(|group| group.id == "objective")
        }
        "## POST-TREATMENT" => {
            return app
                .data
                .groups
                .iter()
                .position(|group| group.id == "post_tx")
        }
        _ => {}
    }

    for (section_idx, section) in app.sections.iter().enumerate() {
        if crate::note::managed_heading_for_section(section).as_deref() == Some(line) {
            return Some(app.group_idx_for_section(section_idx));
        }
    }

    None
}

fn appointment_header_line_spans<'a>(
    app: &'a App,
    line: &'a str,
    group_idx: Option<usize>,
) -> Option<Vec<iced::widget::text::Span<'static, Message, iced::Font>>> {
    let header_idx = app
        .sections
        .iter()
        .position(|section| section.note_render_slot.as_deref() == Some("header"))?;
    let SectionState::Header(state) = app.section_states.get(header_idx)? else {
        return None;
    };

    let date = rendered_header_field_value(app, state, "date").map(format_header_date_for_preview);
    let start =
        rendered_header_field_value(app, state, "start_time").map(format_header_time_for_preview);
    let duration = rendered_header_field_value(app, state, "appointment_duration");
    let appointment_type = rendered_header_field_value(app, state, "appointment_type");
    let in_current_group = group_idx
        .map(|group_idx| group_idx == app.group_idx_for_section(app.current_idx))
        .unwrap_or(true);

    if appointment_type.as_deref() == Some(line) {
        let color =
            header_field_color_by_id(app, header_idx, state, "appointment_type", in_current_group);
        return Some(vec![
            span::<Message, iced::Font>(line.to_string()).color(color)
        ]);
    }

    let (Some(date), Some(start), Some(duration)) = (date, start, duration) else {
        return None;
    };
    let expected = format!("{date} at {start} ({duration} min)");
    if line != expected {
        return None;
    }

    Some(vec![
        span::<Message, iced::Font>(date).color(header_field_color_by_id(
            app,
            header_idx,
            state,
            "date",
            in_current_group,
        )),
        span::<Message, iced::Font>(" at ").color(if in_current_group {
            app.ui_theme.text
        } else {
            app.ui_theme.muted
        }),
        span::<Message, iced::Font>(start).color(header_field_color_by_id(
            app,
            header_idx,
            state,
            "start_time",
            in_current_group,
        )),
        span::<Message, iced::Font>(" (").color(if in_current_group {
            app.ui_theme.text
        } else {
            app.ui_theme.muted
        }),
        span::<Message, iced::Font>(duration).color(header_field_color_by_id(
            app,
            header_idx,
            state,
            "appointment_duration",
            in_current_group,
        )),
        span::<Message, iced::Font>(" min)").color(if in_current_group {
            app.ui_theme.text
        } else {
            app.ui_theme.muted
        }),
    ])
}

fn rendered_header_field_value(
    app: &App,
    state: &crate::sections::header::HeaderState,
    field_id: &str,
) -> Option<String> {
    let (field_idx, field) = state
        .field_configs
        .iter()
        .enumerate()
        .find(|(_, field)| field.id == field_id)?;
    let confirmed = state
        .repeated_values
        .get(field_idx)
        .and_then(|values| values.first())
        .map(|value| value.as_str())
        .unwrap_or("");
    let value = crate::sections::multi_field::resolve_multifield_value(
        confirmed,
        field,
        &app.config.sticky_values,
    );
    value.export_value().map(str::to_string)
}

fn format_header_date_for_preview(date: String) -> String {
    chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map(|d| d.format("%a %b %-d, %Y").to_string())
        .unwrap_or(date)
}

fn format_header_time_for_preview(time: String) -> String {
    chrono::NaiveTime::parse_from_str(&time, "%H:%M")
        .or_else(|_| chrono::NaiveTime::parse_from_str(&time, "%I:%M%P"))
        .map(|t| t.format("%-I:%M%P").to_string())
        .unwrap_or(time)
}

fn header_field_color_by_id(
    app: &App,
    section_idx: usize,
    state: &crate::sections::header::HeaderState,
    field_id: &str,
    in_current_group: bool,
) -> Color {
    state
        .field_configs
        .iter()
        .position(|field| field.id == field_id)
        .map(|field_idx| {
            header_field_preview_color(app, section_idx, state, field_idx, in_current_group)
        })
        .unwrap_or(app.ui_theme.text)
}

fn header_field_preview_color(
    app: &App,
    section_idx: usize,
    state: &crate::sections::header::HeaderState,
    field_idx: usize,
    in_current_group: bool,
) -> Color {
    if section_idx == app.current_idx && field_idx == state.field_index {
        return if app.focus == Focus::Map {
            app.ui_theme.active_preview
        } else {
            app.ui_theme.active
        };
    }
    if state
        .repeated_values
        .get(field_idx)
        .is_some_and(|values| !values.is_empty())
    {
        return if in_current_group {
            app.ui_theme.selected
        } else {
            app.ui_theme.confirmed_muted_preview
        };
    }
    let Some(field) = state.field_configs.get(field_idx) else {
        return app.ui_theme.text;
    };
    let resolved = crate::sections::multi_field::resolve_multifield_value(
        "",
        field,
        &app.config.sticky_values,
    );
    if resolved.export_value().is_some() {
        if in_current_group {
            app.ui_theme.sticky_default_preview
        } else {
            app.ui_theme.muted
        }
    } else {
        app.ui_theme.muted
    }
}

fn section_preview_color(app: &App, section_idx: usize, in_current_group: bool) -> Color {
    if section_idx == app.current_idx {
        if app.focus == Focus::Map {
            app.ui_theme.active_preview
        } else {
            app.ui_theme.active
        }
    } else if app.section_is_completed(section_idx) {
        if in_current_group {
            app.ui_theme.selected
        } else {
            app.ui_theme.confirmed_muted_preview
        }
    } else if app.section_is_skipped(section_idx) {
        app.ui_theme.muted
    } else if !in_current_group {
        app.ui_theme.muted
    } else {
        app.ui_theme.text
    }
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
    text(status_text)
        .font(app.ui_theme.font_status)
        .color(color)
        .into()
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

    if app.section_is_skipped(app.current_idx) {
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
        Some(SectionState::Collection(state)) => {
            let mode = match state.focus {
                CollectionFocus::Collections => "collections",
                CollectionFocus::Items(_) => "items",
            };
            parts.push(format!("Mode: {mode}"));
            parts.push("Space toggles | Enter opens | Backspace resets | Esc confirms".to_string());
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

    if let Some(part_label) = modal.current_part_label(&app.config.sticky_values) {
        modal_items.push(
            text(part_label)
                .font(app.ui_theme.font_modal)
                .color(app.ui_theme.modal_text)
                .into(),
        );
    }

    let app_theme = app.ui_theme.clone();
    modal_items.push(
        text_input("Search", &modal.query)
            .on_input(Message::ModalQueryChanged)
            .font(app.ui_theme.font_modal)
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
                app.ui_theme.active
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
            let marker = if window_pos == modal.list_cursor {
                ">"
            } else {
                " "
            };
            let marker_color =
                if matches!(modal.focus, ModalFocus::List) && window_pos == modal.list_cursor {
                    app.ui_theme.active
                } else {
                    app.ui_theme.modal_muted_text
                };
            let button_label = row![
                container(
                    text(marker)
                        .font(app.ui_theme.font_modal)
                        .color(marker_color),
                )
                .align_left(Length::Fixed(14.0)),
                container(
                    text(format!("{hint:<4}"))
                        .font(app.ui_theme.font_modal)
                        .color(hint_color),
                )
                .align_left(Length::Fixed(24.0)),
                text(label).font(app.ui_theme.font_modal).color(color),
            ]
            .spacing(0);
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

    let modal_size = modal_size_for_labels(&modal.all_entries);
    let (modal_width, fallback_height) = modal_size.dimensions();
    let modal_height =
        modal_height_for_viewport(app.viewport_size.map(|size| size.height), fallback_height);
    let top_offset = modal_top_offset(app);
    let modal_background = app.ui_theme.modal_panel_background;
    let text_color = app.ui_theme.text;
    let modal_panel = container(
        column(modal_items)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8),
    )
    .width(Length::Fixed(modal_width))
    .height(Length::Fixed(modal_height))
    .style(move |_| {
        background_style(modal_background, text_color).border(Border {
            color: text_color,
            width: 1.0,
            radius: 6.0.into(),
        })
    });

    let base = main_layout(app);
    Stack::new()
        .push(base)
        .push(
            container(column![
                Space::with_height(Length::Fixed(top_offset)),
                modal_panel
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[derive(Debug, Clone, Copy)]
enum ModalSize {
    Small,
    Medium,
    Large,
}

impl ModalSize {
    fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Small => (320.0, 220.0),
            Self::Medium => (520.0, 360.0),
            Self::Large => (760.0, 520.0),
        }
    }
}

fn modal_size_for_labels(labels: &[String]) -> ModalSize {
    let max_label_len = labels
        .iter()
        .map(|label| label.chars().count())
        .max()
        .unwrap_or(0);
    if max_label_len <= 34 {
        ModalSize::Small
    } else if max_label_len <= 90 {
        ModalSize::Medium
    } else {
        ModalSize::Large
    }
}

fn modal_top_offset(app: &App) -> f32 {
    let field_offset = match app.section_states.get(app.current_idx) {
        Some(SectionState::Header(state)) => {
            let range = wizard_window(app, state.field_index, state.field_configs.len());
            state.field_index.saturating_sub(range.start) as f32
        }
        _ => 0.0,
    };
    88.0 + field_offset * 24.0
}

/// Public entry point called from main.rs view().
pub fn view(app: &App) -> Element<'_, Message> {
    if let Some(ref modal) = app.modal {
        modal_overlay(app, modal)
    } else {
        main_layout(app)
    }
}
