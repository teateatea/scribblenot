// View layer: builds the iced element tree from app state.
// Modal unit rendering is in modal_unit.rs (submodule); everything else lives here.

mod modal_unit;
use modal_unit::*;

use crate::app::{App, Focus, MapHintLevel, SectionState};
use crate::modal_layout::{
    modal_height_for_viewport, modal_list_view_dimensions, ModalFocus, SimpleModalUnitLayout,
};
use crate::sections::collection::CollectionFocus;
use crate::sections::free_text::FreeTextMode;
use crate::sections::list_select::ListSelectMode;
use crate::Message;
use iced::widget::{
    button, column, container, mouse_area, rich_text, row, scrollable, span, text, text_input,
    Scrollable, Space, Stack,
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
            text(group.nav_label.clone())
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
    confirmed_values: &[crate::sections::header::HeaderFieldValue],
) -> String {
    crate::sections::multi_field::render_field_display(
        confirmed_values,
        field,
        &app.config.sticky_values,
    )
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
        let resolved_field = crate::sections::multi_field::resolve_field_values(
            confirmed_values,
            field,
            &app.config.sticky_values,
        );
        let base_color = if idx == state.field_index {
            if app.focus == Focus::Map {
                app.ui_theme.active_preview
            } else {
                app.ui_theme.active
            }
        } else {
            match resolved_field {
                crate::sections::multi_field::ResolvedMultiFieldValue::Complete(_) => {
                    app.ui_theme.selected
                }
                crate::sections::multi_field::ResolvedMultiFieldValue::Partial(_) => {
                    app.ui_theme.partial_preview
                }
                crate::sections::multi_field::ResolvedMultiFieldValue::Empty => app.ui_theme.muted,
            }
        };
        if let Some(limit) = field.max_entries {
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
                } else {
                    confirmed_values
                        .get(repeat_idx)
                        .map(|value| {
                            match crate::sections::multi_field::resolve_multifield_value(
                                value,
                                field,
                                &app.config.sticky_values,
                            ) {
                                crate::sections::multi_field::ResolvedMultiFieldValue::Complete(
                                    _,
                                ) => app.ui_theme.selected,
                                crate::sections::multi_field::ResolvedMultiFieldValue::Partial(
                                    _,
                                ) => app.ui_theme.partial_preview,
                                crate::sections::multi_field::ResolvedMultiFieldValue::Empty => {
                                    app.ui_theme.muted
                                }
                            }
                        })
                        .unwrap_or(app.ui_theme.muted)
                };
                let value = confirmed_values
                    .get(repeat_idx)
                    .map(|value| field_display_value(app, field, std::slice::from_ref(value)))
                    .unwrap_or_else(|| field_display_value(app, field, &[]));
                let field_label = confirmed_values
                    .get(repeat_idx)
                    .map(|value| {
                        crate::sections::multi_field::resolve_field_label(
                            value,
                            field,
                            &app.config.sticky_values,
                        )
                    })
                    .unwrap_or_else(|| {
                        crate::sections::multi_field::resolve_field_label(
                            &crate::sections::header::HeaderFieldValue::Text(String::new()),
                            field,
                            &app.config.sticky_values,
                        )
                    });
                let hint = field_hints
                    .get(hint_idx)
                    .map(|hint| display_hint_label(app, hint))
                    .unwrap_or_default();
                hint_idx += 1;
                items.push(hinted_line(
                    prefix,
                    hint,
                    format!("{} {}:\n{}", field_label, repeat_idx + 1, value),
                    field_hint_color,
                    field_hint_active,
                    color,
                    app,
                ));
            }
            continue;
        }

        let values = field_display_value(app, field, confirmed_values);
        let confirmed = confirmed_values.first().cloned().unwrap_or(
            crate::sections::header::HeaderFieldValue::Text(String::new()),
        );
        let field_label = crate::sections::multi_field::resolve_field_label(
            &confirmed,
            field,
            &app.config.sticky_values,
        );
        let prefix = if idx == state.field_index { ">" } else { " " };
        let field_hint = field_hints
            .get(hint_idx)
            .map(|hint| display_hint_label(app, hint))
            .unwrap_or_default();
        hint_idx += 1;
        items.push(hinted_line(
            prefix,
            field_hint,
            format!("{}:\n{}", field_label, values),
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

fn render_collection_state<'a>(
    app: &'a App,
    state: &'a crate::sections::collection::CollectionState,
) -> Element<'a, Message> {
    let mut items = section_header(app);

    if state.collections.is_empty() {
        items.push(
            text("[no collections loaded]")
                .color(app.ui_theme.error)
                .into(),
        );
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
            let base_color = if idx == state.collection_cursor {
                app.ui_theme.active
            } else if collection.active {
                app.ui_theme.selected
            } else {
                app.ui_theme.muted
            };
            let color = app
                .collection_text_flash_amount(&collection.id)
                .map(|amount| blend_color(base_color, app.ui_theme.text_color_flash, amount))
                .unwrap_or(base_color);
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
        CollectionFocus::Collections if !state.collections.is_empty() => {
            Some(state.collection_cursor)
        }
        CollectionFocus::Collections => None,
    };

    if let Some(collection_idx) = current_collection_idx {
        if let Some(collection) = state.collections.get(collection_idx) {
            items.push(
                text(format!("Items: {}", collection.label))
                    .color(app.ui_theme.modal)
                    .into(),
            );
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
                    text(format!("{prefix} {marker} {}", item.ui_label()))
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

fn apply_alpha(color: Color, alpha: f32) -> Color {
    let alpha = alpha.clamp(0.0, 1.0);
    Color {
        a: color.a * alpha,
        ..color
    }
}

fn italic_font(base: iced::Font) -> iced::Font {
    iced::Font {
        style: iced::font::Style::Italic,
        ..base
    }
}

fn preview_font(app: &App, italic: bool) -> iced::Font {
    if italic {
        italic_font(app.ui_theme.font_preview)
    } else {
        app.ui_theme.font_preview
    }
}

fn modal_composition_font(app: &App, italic: bool) -> iced::Font {
    if italic {
        italic_font(app.ui_theme.font_modal)
    } else {
        app.ui_theme.font_modal
    }
}

#[derive(Debug, Clone, Copy)]
struct HeaderPreviewLineMatch {
    section_idx: usize,
    field_idx: usize,
    manual_override: bool,
}

fn match_header_preview_line(app: &App, line: &str) -> Option<HeaderPreviewLineMatch> {
    for (section_idx, section) in app.sections.iter().enumerate() {
        let Some(SectionState::Header(state)) = app.section_states.get(section_idx) else {
            continue;
        };

        for (field_idx, field) in state.field_configs.iter().enumerate() {
            let values = state
                .repeated_values
                .get(field_idx)
                .map(|values| values.as_slice())
                .unwrap_or(&[]);

            if values.is_empty() {
                let empty_value = crate::sections::header::HeaderFieldValue::Text(String::new());
                let resolved = crate::sections::multi_field::resolve_multifield_value(
                    &empty_value,
                    field,
                    &app.config.sticky_values,
                );
                let Some(rendered) = resolved.export_value() else {
                    continue;
                };
                let candidate = crate::sections::multi_field::render_note_line(
                    section,
                    field,
                    &empty_value,
                    &app.config.sticky_values,
                )
                .unwrap_or_else(|| rendered.to_string());
                if line == candidate {
                    return Some(HeaderPreviewLineMatch {
                        section_idx,
                        field_idx,
                        manual_override: false,
                    });
                }
                continue;
            }

            for value in values {
                let resolved = crate::sections::multi_field::resolve_multifield_value(
                    value,
                    field,
                    &app.config.sticky_values,
                );
                let Some(rendered) = resolved.export_value() else {
                    continue;
                };
                let candidate = crate::sections::multi_field::render_note_line(
                    section,
                    field,
                    value,
                    &app.config.sticky_values,
                )
                .unwrap_or_else(|| rendered.to_string());
                if line == candidate {
                    return Some(HeaderPreviewLineMatch {
                        section_idx,
                        field_idx,
                        manual_override: value.is_manual_override(),
                    });
                }
            }
        }
    }

    None
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
    let manual_override =
        match_header_preview_line(app, &line).is_some_and(|line_match| line_match.manual_override);
    let color = preview_line_color(app, &line, group_idx);
    text(line)
        .font(preview_font(app, manual_override))
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

    if let Some(line_match) = match_header_preview_line(app, line) {
        let Some(SectionState::Header(state)) = app.section_states.get(line_match.section_idx)
        else {
            return app.ui_theme.text;
        };
        return header_field_preview_color(
            app,
            line_match.section_idx,
            state,
            line_match.field_idx,
            in_current_group,
        );
    }

    for (section_idx, section) in app.sections.iter().enumerate() {
        if crate::note::managed_heading_for_section(section).as_deref() == Some(line) {
            return section_preview_color(app, section_idx, in_current_group);
        }
    }

    if in_current_group {
        app.ui_theme.text
    } else {
        app.ui_theme.muted
    }
}

fn preview_group_for_line(app: &App, line: &str) -> Option<usize> {
    for (idx, group) in app.data.groups.iter().enumerate() {
        let heading = group.note.note_label.as_deref();
        if heading == Some(line) {
            return Some(idx);
        }
    }

    for (section_idx, section) in app.sections.iter().enumerate() {
        if crate::note::managed_heading_for_section(section).as_deref() == Some(line) {
            return Some(app.group_idx_for_section(section_idx));
        }
    }

    None
}

fn header_field_preview_color(
    app: &App,
    section_idx: usize,
    state: &crate::sections::header::HeaderState,
    field_idx: usize,
    in_current_group: bool,
) -> Color {
    let Some(field) = state.field_configs.get(field_idx) else {
        return app.ui_theme.text;
    };
    let resolved = crate::sections::multi_field::resolve_field_values(
        state
            .repeated_values
            .get(field_idx)
            .map(|values| values.as_slice())
            .unwrap_or(&[]),
        field,
        &app.config.sticky_values,
    );
    if section_idx == app.current_idx && field_idx == state.field_index {
        return if app.focus == Focus::Map {
            app.ui_theme.active_preview
        } else {
            app.ui_theme.active
        };
    }
    match resolved {
        crate::sections::multi_field::ResolvedMultiFieldValue::Complete(_) => {
            if in_current_group {
                app.ui_theme.sticky_default_preview
            } else {
                app.ui_theme.muted
            }
        }
        crate::sections::multi_field::ResolvedMultiFieldValue::Partial(_) => {
            if in_current_group {
                app.ui_theme.partial_preview
            } else {
                app.ui_theme.muted
            }
        }
        crate::sections::multi_field::ResolvedMultiFieldValue::Empty => app.ui_theme.muted,
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
        if app.modal_composition_editing {
            return "Modal: composition edit | Enter/Esc exits | Ctrl+R resets".to_string();
        }
        if let Some(state) = modal.collection_state.as_ref() {
            let focus = match state.focus {
                CollectionFocus::Collections => "collections",
                CollectionFocus::Items(_) => "items",
            };
            return format!(
                "Modal: {focus} | Enter toggles | Space/Right moves in | Left/Esc moves out | Shift+Enter confirms"
            );
        }
        let focus = match modal.focus {
            ModalFocus::SearchBar => "search",
            ModalFocus::List => "list",
        };
        return format!(
            "Modal: {focus} | {} matches | Enter selects | Ctrl+E edits entry | Esc closes",
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
                if let Some(limit) = field.max_entries {
                    parts.push(format!(
                        "Entries: {}/{}",
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
            parts.push("Mode: browsing".to_string());
            parts.push(format!("Selected: {}", state.selected_indices.len()));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalRenderMode {
    Interactive,
    Preview,
}

impl ModalRenderMode {
    fn is_preview(self) -> bool {
        matches!(self, Self::Preview)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalCardRole {
    Active,
    Inactive,
    Stub(crate::modal_layout::ModalStubKind),
}

const COLLECTION_STREAM_SPACING: f32 = 6.0;

fn modal_card_style(
    app: &App,
    mode: ModalRenderMode,
    role: ModalCardRole,
    focused: bool,
    alpha: f32,
) -> iced::widget::container::Style {
    let preview_background = match role {
        ModalCardRole::Active => app.ui_theme.modal_inactive_background,
        ModalCardRole::Inactive => app.ui_theme.modal_inactive_background,
        ModalCardRole::Stub(kind) => match kind {
            crate::modal_layout::ModalStubKind::NavLeft | crate::modal_layout::ModalStubKind::NavRight => {
                app.ui_theme.modal_nav_stub_background
            }
            crate::modal_layout::ModalStubKind::Exit => app.ui_theme.modal_exit_stub_background,
            crate::modal_layout::ModalStubKind::Confirm => app.ui_theme.modal_confirm_stub_background,
        },
    };
    let (mut background, mut text_color, mut border_color) = if mode.is_preview() {
        (
            preview_background,
            blend_color(app.ui_theme.modal_text, app.ui_theme.modal_muted_text, 0.6),
            blend_color(
                app.ui_theme.modal_input_border,
                app.ui_theme.modal_muted_text,
                0.45,
            ),
        )
    } else {
        (
            app.ui_theme.modal_active_background,
            app.ui_theme.text,
            if focused {
                app.ui_theme.text
            } else {
                app.ui_theme.modal_input_border
            },
        )
    };

    if alpha < 1.0 {
        background = apply_alpha(background, alpha);
        text_color = apply_alpha(text_color, alpha);
        border_color = apply_alpha(border_color, alpha);
    }

    background_style(background, text_color).border(Border {
        color: border_color,
        width: 1.0,
        radius: if mode.is_preview() { 5.0 } else { 6.0 }.into(),
    })
}

fn modal_card<'a>(
    app: &'a App,
    content: impl Into<Element<'a, Message>>,
    mode: ModalRenderMode,
    role: ModalCardRole,
    focused: bool,
    width: f32,
    height: f32,
    alpha: f32,
) -> Element<'a, Message> {
    let panel = container(content)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .style(move |_| modal_card_style(app, mode, role, focused, alpha));

    if mode.is_preview() {
        mouse_area(panel)
            .on_press(Message::ModalPanelPressed)
            .into()
    } else {
        mouse_area(panel)
            .on_press(Message::ModalPanelPressed)
            .into()
    }
}

fn preview_modal_search_strip<'a>(
    app: &'a App,
    query: String,
    alpha: f32,
) -> iced::widget::Container<'a, Message> {
    let mut panel_background = blend_color(
        app.ui_theme.modal_input_background,
        app.ui_theme.modal_panel_background,
        0.35,
    );
    let mut border_color = blend_color(
        app.ui_theme.modal_input_border,
        app.ui_theme.modal_muted_text,
        0.35,
    );
    let mut text_color = app.ui_theme.modal_muted_text;
    if alpha < 1.0 {
        panel_background = apply_alpha(panel_background, alpha);
        border_color = apply_alpha(border_color, alpha);
        text_color = apply_alpha(text_color, alpha);
    }
    let value = if query.is_empty() {
        "Search".to_string()
    } else {
        query
    };
    container(text(value).font(app.ui_theme.font_modal).color(text_color))
        .width(Length::Fill)
        .padding([6.0, 8.0])
        .style(move |_| {
            background_style(panel_background, app.ui_theme.modal_muted_text).border(Border {
                color: border_color,
                width: 1.0,
                radius: 2.0.into(),
            })
        })
}

fn preview_simple_modal_content<'a>(
    app: &'a App,
    snapshot: crate::modal_layout::ModalListViewSnapshot,
    alpha: f32,
) -> Element<'a, Message> {
    let title = snapshot.title.clone();
    let query = snapshot.query.clone();
    let mut items: Vec<Element<'a, Message>> = Vec::new();
    items.push(
        text(title)
            .font(app.ui_theme.font_modal)
            .color(apply_alpha(app.ui_theme.modal_hint_text, alpha))
            .into(),
    );
    items.push(preview_modal_search_strip(app, query, alpha).into());

    let end = (snapshot.list_scroll + app.modal_window_size().max(1)).min(snapshot.filtered.len());
    let modal_hints: Vec<String> = app
        .data
        .keybindings
        .hints
        .iter()
        .take(end.saturating_sub(snapshot.list_scroll))
        .cloned()
        .collect();
    for window_pos in snapshot.list_scroll..end {
        let Some(&entry_idx) = snapshot.filtered.get(window_pos) else {
            continue;
        };
        let Some(label) = snapshot.rows.get(entry_idx).cloned() else {
            continue;
        };
        let is_current = window_pos == snapshot.list_cursor;
        let hint = modal_hints
            .get(window_pos - snapshot.list_scroll)
            .map(|hint| display_hint_label(app, hint))
            .unwrap_or_default();
        let row_content = row![
            container(
                text(if is_current { ">" } else { " " })
                    .font(app.ui_theme.font_modal)
                    .color(apply_alpha(
                        if is_current {
                            app.ui_theme.modal_text
                        } else {
                            app.ui_theme.modal_muted_text
                        },
                        alpha,
                    )),
            )
            .align_left(Length::Fixed(14.0)),
            container(
                text(format!("{hint:<4}"))
                    .font(app.ui_theme.font_modal)
                    .color(apply_alpha(app.ui_theme.modal_muted_text, alpha)),
            )
            .align_left(Length::Fixed(24.0)),
            text(label).font(app.ui_theme.font_modal).color(apply_alpha(
                if is_current {
                    app.ui_theme.active_preview
                } else {
                    app.ui_theme.modal_muted_text
                },
                alpha,
            )),
        ]
        .spacing(0);
        items.push(container(row_content).width(Length::Fill).into());
    }

    container(column(items).spacing(4).padding(8))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn blended_modal_theme(app: &App, alpha: f32) -> crate::theme::AppTheme {
    if alpha >= 1.0 {
        return app.ui_theme.clone();
    }
    let mut theme = app.ui_theme.clone();
    let blend = |color| apply_alpha(color, alpha);
    theme.active = blend(theme.active);
    theme.modal = blend(theme.modal);
    theme.text = blend(theme.text);
    theme.modal_text = blend(theme.modal_text);
    theme.modal_selected_text = blend(theme.modal_selected_text);
    theme.modal_muted_text = blend(theme.modal_muted_text);
    theme.modal_hint_text = blend(theme.modal_hint_text);
    theme.modal_input_background = blend(theme.modal_input_background);
    theme.modal_input_text = blend(theme.modal_input_text);
    theme.modal_input_placeholder = blend(theme.modal_input_placeholder);
    theme.modal_input_border = blend(theme.modal_input_border);
    theme.modal_item_background = blend(theme.modal_item_background);
    theme.modal_item_hovered_background = blend(theme.modal_item_hovered_background);
    theme.scroll_rail = blend(theme.scroll_rail);
    theme.scroll_scroller = blend(theme.scroll_scroller);
    theme.scroll_rail_hovered = blend(theme.scroll_rail_hovered);
    theme.scroll_scroller_hovered = blend(theme.scroll_scroller_hovered);
    theme.scroll_rail_dragged = blend(theme.scroll_rail_dragged);
    theme.scroll_scroller_dragged = blend(theme.scroll_scroller_dragged);
    theme.scroll_gap = blend(theme.scroll_gap);
    theme
}

fn themed_scrollable_with_theme<'a>(
    app_theme: crate::theme::AppTheme,
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    let scrollbar = iced::widget::scrollable::Scrollbar::new()
        .width(app_theme.scroll_width)
        .scroller_width(app_theme.scroll_width)
        .spacing(app_theme.scroll_spacing);
    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(iced::widget::scrollable::Direction::Vertical(scrollbar))
        .style(move |_theme, status| scrollable_style(&app_theme, status))
}

fn active_simple_modal_content<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
    alpha: f32,
) -> Element<'a, Message> {
    let ui_theme = blended_modal_theme(app, alpha);
    let mut modal_items: Vec<Element<'a, Message>> = Vec::new();
    if let Some(part_label) = modal.current_part_label(&app.config.sticky_values) {
        modal_items.push(
            text(part_label)
                .font(ui_theme.font_modal)
                .color(ui_theme.modal_text)
                .into(),
        );
    }

    let app_theme = ui_theme.clone();
    modal_items.push(
        text_input("Search", &modal.query)
            .on_input(Message::ModalQueryChanged)
            .font(ui_theme.font_modal)
            .width(Length::Fill)
            .style(move |_theme, status| modal_input_style(&app_theme, status))
            .into(),
    );

    let end = (modal.list_scroll + modal.window_size).min(modal.filtered.len());
    let modal_hints: Vec<String> = app
        .data
        .keybindings
        .hints
        .iter()
        .take(end.saturating_sub(modal.list_scroll))
        .cloned()
        .collect();
    let mut list_items: Vec<Element<'a, Message>> = Vec::new();
    for window_pos in modal.list_scroll..end {
        if let Some(&entry_idx) = modal.filtered.get(window_pos) {
            let label = &modal.all_entries[entry_idx];
            let color = if window_pos == modal.list_cursor {
                ui_theme.active
            } else {
                ui_theme.modal_muted_text
            };
            let hint = modal_hints
                .get(window_pos - modal.list_scroll)
                .map(|hint| display_hint_label(app, hint))
                .unwrap_or_default();
            let hint_color = if matches!(modal.focus, ModalFocus::List) {
                ui_theme.modal_hint_text
            } else {
                ui_theme.modal_muted_text
            };
            let marker = if window_pos == modal.list_cursor {
                ">"
            } else {
                " "
            };
            let marker_color =
                if matches!(modal.focus, ModalFocus::List) && window_pos == modal.list_cursor {
                    ui_theme.active
                } else {
                    ui_theme.modal_muted_text
                };
            let button_label = row![
                container(
                    text(marker)
                        .font(ui_theme.font_modal)
                        .color(marker_color)
                )
                .align_left(Length::Fixed(14.0)),
                container(
                    text(format!("{hint:<4}"))
                        .font(ui_theme.font_modal)
                        .color(hint_color),
                )
                .align_left(Length::Fixed(24.0)),
                text(label).font(ui_theme.font_modal).color(color),
            ]
            .spacing(0);
            let app_theme = ui_theme.clone();
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
        themed_scrollable_with_theme(ui_theme.clone(), column(list_items))
            .height(Length::Fill)
            .into(),
    );

    container(
        column(modal_items)
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(4)
            .padding(8),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn entry_composition_panel<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
    modal_width: f32,
) -> Option<Element<'a, Message>> {
    let structured_spans =
        crate::app::compute_field_composition_spans(modal, &app.config.sticky_values);
    if structured_spans.is_empty() && modal.manual_override.is_none() {
        return None;
    }

    let styled_spans = structured_spans
        .clone()
        .into_iter()
        .map(|span_data| {
            let color = match span_data.kind {
                crate::app::FieldCompositionSpanKind::Literal => app.ui_theme.text,
                crate::app::FieldCompositionSpanKind::Confirmed => app.ui_theme.selected,
                crate::app::FieldCompositionSpanKind::Active => app.ui_theme.active,
                crate::app::FieldCompositionSpanKind::Preview => app.ui_theme.modal_muted_text,
            };
            span::<Message, iced::Font>(span_data.text).color(color)
        })
        .collect::<Vec<_>>();
    let subdued_source_spans = structured_spans
        .into_iter()
        .map(|span_data| {
            let base = match span_data.kind {
                crate::app::FieldCompositionSpanKind::Literal => app.ui_theme.text,
                crate::app::FieldCompositionSpanKind::Confirmed => app.ui_theme.selected,
                crate::app::FieldCompositionSpanKind::Active => app.ui_theme.active,
                crate::app::FieldCompositionSpanKind::Preview => app.ui_theme.modal_muted_text,
            };
            span::<Message, iced::Font>(span_data.text).color(blend_color(
                base,
                app.ui_theme.modal_muted_text,
                0.45,
            ))
        })
        .collect::<Vec<_>>();

    let panel_width = app
        .viewport_size
        .map(|size| (size.width * 0.9).clamp(modal_width, 980.0))
        .unwrap_or(820.0)
        .max(modal_width);
    let label_color = blend_color(app.ui_theme.modal_hint_text, app.ui_theme.text, 0.25);
    let panel_background = blend_color(
        app.ui_theme.modal_panel_background,
        app.ui_theme.modal_input_background,
        0.15,
    );
    let border_color = blend_color(app.ui_theme.modal_input_border, app.ui_theme.text, 0.25);
    let helper_text = if app.modal_composition_editing {
        "Manual edit mode | Enter or Esc exits | Ctrl+R resets"
    } else if modal.manual_override.is_some() {
        "Manual override active | Ctrl+E edits | Ctrl+R resets"
    } else {
        "Structured composition | Ctrl+E edits current field"
    };
    let helper_color = blend_color(app.ui_theme.modal_muted_text, app.ui_theme.text, 0.2);
    let input_theme = app.ui_theme.clone();

    let main_content: Element<'a, Message> = if app.modal_composition_editing {
        let current_text = modal.manual_override.as_deref().unwrap_or_default();
        text_input("Edit current field", current_text)
            .on_input(Message::ModalCompositionChanged)
            .font(modal_composition_font(app, true))
            .width(Length::Fill)
            .style(move |_theme, status| modal_input_style(&input_theme, status))
            .into()
    } else if let Some(override_text) = modal.manual_override.as_deref() {
        let display_text = if override_text.is_empty() {
            "[empty override]"
        } else {
            override_text
        };
        text(display_text)
            .font(modal_composition_font(app, true))
            .size(16)
            .width(Length::Fill)
            .color(blend_color(app.ui_theme.text, app.ui_theme.selected, 0.18))
            .into()
    } else {
        rich_text::<Message, iced::Theme, iced::Renderer>(styled_spans)
            .font(app.ui_theme.font_modal)
            .size(16)
            .width(Length::Fill)
            .into()
    };

    let mut panel_items: Vec<Element<'a, Message>> = vec![
        text("Entry Composition")
            .font(app.ui_theme.font_modal)
            .color(label_color)
            .into(),
        main_content,
        text(helper_text)
            .font(app.ui_theme.font_modal)
            .color(helper_color)
            .into(),
    ];

    if modal.manual_override.is_some() {
        panel_items.push(
            text("Structured source")
                .font(app.ui_theme.font_modal)
                .color(label_color)
                .into(),
        );
        panel_items.push(
            rich_text::<Message, iced::Theme, iced::Renderer>(subdued_source_spans)
                .font(app.ui_theme.font_modal)
                .size(16)
                .width(Length::Fill)
                .into(),
        );
    }

    Some(
        container(column(panel_items).spacing(8))
            .width(Length::Fixed(panel_width))
            .padding(12)
            .style(move |_| {
                background_style(panel_background, app.ui_theme.text).border(Border {
                    color: border_color,
                    width: 1.0,
                    radius: 6.0.into(),
                })
            })
            .into(),
    )
}


fn simple_modal_unit_root_width(layout: &SimpleModalUnitLayout) -> Option<f32> {
    layout
        .sequence
        .snapshots
        .get(layout.sequence.active_sequence_index)
        .map(|snapshot| modal_list_view_dimensions(snapshot).0)
}

fn collection_neighbor_previews_supported(app: &App) -> bool {
    app.viewport_size
        .map(|size| size.height >= 760.0)
        .unwrap_or(true)
}

/// Build the modal overlay when a search modal is active.
fn modal_overlay<'a>(app: &'a App, modal: &'a crate::modal::SearchModal) -> Element<'a, Message> {
    const COLLECTION_MODAL_MIN_HEIGHT: f32 = 220.0;
    const COLLECTION_MODAL_MAX_HEIGHT: f32 = 460.0;
    const COLLECTION_MODAL_CHROME_HEIGHT: f32 = 96.0;
    const COLLECTION_MODAL_ROW_HEIGHT: f32 = 28.0;

    let show_collection_preview =
        modal.is_collection_mode() && collection_modal_supports_preview(app);

    let simple_unit_layout: Option<&crate::modal_layout::SimpleModalUnitLayout> =
        if !modal.is_collection_mode() {
            app.modal_unit_layout.as_ref()
        } else {
            None
        };
    let (mut modal_width, fallback_height) =
        modal_dimensions_for_content(app, modal, show_collection_preview);
    if let Some(layout) = simple_unit_layout {
        if let Some(unit_width) = simple_modal_unit_root_width(layout) {
            modal_width = unit_width;
        }
    }
    let modal_height = if modal.is_collection_mode() {
        let collection_count = modal
            .collection_state
            .as_ref()
            .map(|state| state.collections.len())
            .unwrap_or(0);
        let item_count = modal
            .collection_state
            .as_ref()
            .map(|state| collection_preview_metrics(&state.collections).1)
            .unwrap_or(0);
        let content_rows = collection_count.max(item_count).max(1) as f32;
        let content_height = (COLLECTION_MODAL_CHROME_HEIGHT
            + content_rows * COLLECTION_MODAL_ROW_HEIGHT)
            .clamp(COLLECTION_MODAL_MIN_HEIGHT, COLLECTION_MODAL_MAX_HEIGHT);
        app.viewport_size
            .map(|size| content_height.min(size.height * crate::modal_layout::MODAL_HEIGHT_RATIO))
            .unwrap_or(fallback_height)
    } else {
        modal_height_for_viewport(app.viewport_size.map(|size| size.height), fallback_height)
    };
    let top_offset = modal_top_offset(app);
    let active_modal = if modal.is_collection_mode() {
        let content = if show_collection_preview {
            collection_modal_split_panes(app, modal)
        } else {
            let mut list_items: Vec<Element<'a, Message>> = Vec::new();
            render_collection_modal_items(app, modal, &mut list_items);
            collection_left_panel(
                app,
                modal,
                container(column(list_items).spacing(2)).height(Length::Fill),
            )
            .into()
        };
        modal_card(
            app,
            content,
            ModalRenderMode::Interactive,
            ModalCardRole::Active,
            true,
            modal_width,
            modal_height,
            1.0,
        )
    } else {
        Space::with_width(Length::Shrink).into()
    };

    let modal_stream: Element<'a, Message> = if !modal.is_collection_mode() {
        if let Some(current_layout) = app.modal_unit_layout.as_ref() {
            if let Some(current_unit) = current_layout.units.get(app.active_unit_index) {
                let mut layers = Stack::new();

                if let Some(crate::app::ModalTransitionLayer::ConnectedTransition {
                    arrival,
                    departure,
                    slide_distance,
                }) = app.modal_transitions.last()
                {
                    let p = arrival.eased_progress();
                    let slide = *slide_distance;
                    let connected_rendered = build_connected_transition_rendered_unit(
                        app.ui_theme.modal_stub_width,
                        current_layout,
                        arrival,
                        departure,
                        p,
                    );
                    layers = layers.push(render_connected_transition(
                        app,
                        &connected_rendered,
                        departure,
                        Some(modal),
                        modal_height,
                        p,
                        slide,
                        true,
                    ));
                } else {
                    // No active transition: render the current unit at rest.
                    let rendered_current = build_rendered_modal_unit(
                        app,
                        current_layout,
                        current_unit,
                        default_stub_mode(ModalUnitSide::Left),
                        default_stub_mode(ModalUnitSide::Right),
                    );
                    layers = layers.push(render_modal_unit(
                        app,
                        &rendered_current,
                        Some(modal),
                        modal_height,
                        0.0,
                        true,
                    ));
                }
                layers.width(Length::Fill).into()
            } else {
                modal_card(
                    app,
                    active_simple_modal_content(app, modal, 1.0),
                    ModalRenderMode::Interactive,
                    ModalCardRole::Active,
                    true,
                    modal_width,
                    modal_height,
                    1.0,
                )
            }
        } else {
            modal_card(
                app,
                active_simple_modal_content(app, modal, 1.0),
                ModalRenderMode::Interactive,
                ModalCardRole::Active,
                true,
                modal_width,
                modal_height,
                1.0,
            )
        }
    } else {
        active_modal
    };

    let composition_panel = if modal.is_collection_mode() {
        None
    } else {
        entry_composition_panel(app, modal, modal_width)
    };
    let base = main_layout(app);
    Stack::new()
        .push(base)
        .push(
            mouse_area(
                container(
                    column(
                        std::iter::once(Space::with_height(Length::Fixed(top_offset)).into())
                            .chain(
                                composition_panel
                                    .into_iter()
                                    .map(|panel| {
                                        container(panel)
                                            .width(Length::Fill)
                                            .center_x(Length::Fill)
                                            .into()
                                    })
                                    .chain(std::iter::once(
                                        container(modal_stream)
                                            .width(Length::Fill)
                                            .center_x(Length::Fill)
                                            .into(),
                                    )),
                            )
                            .collect::<Vec<Element<'a, Message>>>(),
                    )
                    .spacing(14),
                )
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .on_press(Message::ModalBackdropPressed),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_collection_modal_items<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
    list_items: &mut Vec<Element<'a, Message>>,
) {
    let Some(state) = modal.collection_state.as_ref() else {
        return;
    };
    let rows = state
        .collections
        .iter()
        .map(|collection| {
            let marker = if collection.active { "[x]" } else { "[ ]" };
            format!("{marker} {}", collection.label)
        })
        .collect::<Vec<_>>();
    let row_colors = state
        .collections
        .iter()
        .enumerate()
        .map(|(idx, collection)| {
            let base_color = if idx == state.collection_cursor {
                app.ui_theme.active
            } else if collection.active {
                app.ui_theme.modal_selected_text
            } else {
                app.ui_theme.modal_muted_text
            };
            app.collection_text_flash_amount(&collection.id)
                .map(|amount| blend_color(base_color, app.ui_theme.text_color_flash, amount))
                .unwrap_or(base_color)
        })
        .collect::<Vec<_>>();
    let cursor = state.collection_cursor;
    let pane_focused = matches!(state.focus, CollectionFocus::Collections);
    let range = wizard_window(app, cursor, rows.len());
    render_modal_rows(
        app,
        list_items,
        &rows,
        Some(&row_colors),
        cursor,
        range,
        pane_focused,
        crate::app::ModalPaneTarget::Left,
    );
}

fn collection_modal_split_panes<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
) -> Element<'a, Message> {
    let Some(_state) = modal.collection_state.as_ref() else {
        return Space::with_height(Length::Shrink).into();
    };
    let mut left_items = Vec::new();
    render_collection_modal_items(app, modal, &mut left_items);
    let right_panel = collection_modal_preview(app, modal);
    let left_panel = collection_left_panel(
        app,
        modal,
        container(column(left_items).spacing(2)).height(Length::Fill),
    );
    row![left_panel.width(Length::FillPortion(7)), right_panel]
        .spacing(4)
        .height(Length::Fill)
        .into()
}

fn collection_left_panel<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message> {
    let left_focused = modal
        .collection_state
        .as_ref()
        .is_some_and(|state| matches!(state.focus, CollectionFocus::Collections));
    let title = modal
        .current_part_label(&app.config.sticky_values)
        .unwrap_or_else(|| modal.field_name.clone());
    modal_subpanel(
        app,
        column![
            text(title)
                .font(app.ui_theme.font_modal)
                .color(app.ui_theme.modal_hint_text),
            content.into()
        ]
        .spacing(4)
        .height(Length::Fill),
        left_focused,
        crate::app::ModalPaneTarget::Left,
    )
}

fn render_modal_rows<'a>(
    app: &'a App,
    list_items: &mut Vec<Element<'a, Message>>,
    rows: &[String],
    row_colors: Option<&[Color]>,
    cursor: usize,
    range: std::ops::Range<usize>,
    pane_focused: bool,
    target: crate::app::ModalPaneTarget,
) {
    let modal_hints: Vec<String> = app
        .data
        .keybindings
        .hints
        .iter()
        .take(range.end.saturating_sub(range.start))
        .cloned()
        .collect();

    for window_pos in range.clone() {
        let Some(label) = rows.get(window_pos).cloned() else {
            continue;
        };
        let is_current = window_pos == cursor;
        let marker = if is_current { ">" } else { " " };
        let color = row_colors
            .and_then(|colors| colors.get(window_pos).copied())
            .unwrap_or_else(|| {
                if is_current {
                    app.ui_theme.active
                } else {
                    app.ui_theme.modal_muted_text
                }
            });
        let hint = modal_hints
            .get(window_pos - range.start)
            .map(|hint| display_hint_label(app, hint))
            .unwrap_or_default();
        let hint_color = if pane_focused {
            app.ui_theme.modal_hint_text
        } else {
            app.ui_theme.modal_muted_text
        };
        let marker_color = if is_current {
            if pane_focused {
                app.ui_theme.active
            } else {
                app.ui_theme.modal_text
            }
        } else {
            app.ui_theme.modal_muted_text
        };
        let button_label = row![
            container(
                text(marker)
                    .font(app.ui_theme.font_modal)
                    .color(marker_color)
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
            mouse_area(
                button(button_label)
                    .width(Length::Fill)
                    .on_press(Message::ModalRowPressed(target, window_pos))
                    .style(move |_theme, status| modal_item_button_style(&app_theme, status)),
            )
            .on_enter(Message::ModalRowHovered(target, window_pos))
            .interaction(iced::mouse::Interaction::Pointer)
            .into(),
        );
    }
}

fn preview_modal_subpanel<'a>(
    app: &'a App,
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message> {
    let panel_background = blend_color(
        app.ui_theme.modal_input_background,
        app.ui_theme.modal_panel_background,
        0.2,
    );
    let border_color = blend_color(
        app.ui_theme.modal_input_border,
        app.ui_theme.modal_muted_text,
        0.35,
    );
    let text_color = blend_color(app.ui_theme.modal_text, app.ui_theme.modal_muted_text, 0.55);
    container(container(content).height(Length::Fill).padding(6))
        .height(Length::Fill)
        .style(move |_| {
            background_style(panel_background, text_color).border(Border {
                color: border_color,
                width: 1.0,
                radius: 4.0.into(),
            })
        })
}

fn collection_preview_card_content<'a>(
    app: &'a App,
    snapshot: crate::modal::CollectionPreviewSnapshot,
    interactive: bool,
    pane_focused: bool,
) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = Vec::with_capacity(snapshot.rows.len() + 1);
    items.push(
        text(snapshot.title)
            .font(app.ui_theme.font_modal)
            .color(if interactive {
                app.ui_theme.modal_hint_text
            } else {
                app.ui_theme.modal_muted_text
            })
            .into(),
    );

    if interactive {
        let cursor = snapshot.item_cursor.unwrap_or(0);
        let range = wizard_window(app, cursor, snapshot.rows.len());
        render_modal_rows(
            app,
            &mut items,
            &snapshot.rows,
            None,
            cursor,
            range,
            pane_focused,
            crate::app::ModalPaneTarget::Right,
        );
    } else {
        for (idx, row_value) in snapshot.rows.into_iter().enumerate() {
            let color = if Some(idx) == snapshot.item_cursor {
                app.ui_theme.active_preview
            } else {
                app.ui_theme.modal_muted_text
            };
            items.push(
                text(row_value)
                    .font(app.ui_theme.font_modal)
                    .color(color)
                    .into(),
            );
        }
    }

    container(column(items).spacing(4))
        .height(Length::Fill)
        .into()
}

fn collection_modal_preview<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
) -> Element<'a, Message> {
    let Some(state) = modal.collection_state.as_ref() else {
        return Space::with_height(Length::Shrink).into();
    };
    let pane_focused = matches!(state.focus, CollectionFocus::Items(_));
    let Some(neighbors) = modal.collection_preview_neighbors() else {
        return Space::with_height(Length::Shrink).into();
    };

    let show_neighbors = collection_neighbor_previews_supported(app);
    let mut cards: Vec<Element<'a, Message>> = Vec::new();
    if show_neighbors {
        if let Some(previous) = neighbors.previous.as_ref() {
            cards.push(
                preview_modal_subpanel(
                    app,
                    collection_preview_card_content(app, previous.clone(), false, false),
                )
                .height(Length::FillPortion(1))
                .into(),
            );
        }
    }
    cards.push(
        modal_subpanel(
            app,
            collection_preview_card_content(app, neighbors.current.clone(), true, pane_focused),
            pane_focused,
            crate::app::ModalPaneTarget::Right,
        )
        .height(Length::FillPortion(if show_neighbors { 2 } else { 1 }))
        .into(),
    );
    if show_neighbors {
        if let Some(next) = neighbors.next.as_ref() {
            cards.push(
                preview_modal_subpanel(
                    app,
                    collection_preview_card_content(app, next.clone(), false, false),
                )
                .height(Length::FillPortion(1))
                .into(),
            );
        }
    }

    container(
        column(cards)
            .spacing(COLLECTION_STREAM_SPACING)
            .height(Length::Fill),
    )
    .width(Length::FillPortion(5))
    .height(Length::Fill)
    .into()
}

#[derive(Debug, Clone, Copy)]
enum ModalSize {
    Small,
    Medium,
    Large,
    ExtraWide,
}

impl ModalSize {
    fn dimensions(self) -> (f32, f32) {
        match self {
            Self::Small => (320.0, 220.0),
            Self::Medium => (520.0, 360.0),
            Self::Large => (760.0, 520.0),
            Self::ExtraWide => (980.0, 560.0),
        }
    }
}

fn modal_size_for_labels(
    labels: &[String],
    is_collection_mode: bool,
    show_collection_preview: bool,
) -> ModalSize {
    if show_collection_preview {
        return ModalSize::ExtraWide;
    }
    let max_label_len = labels
        .iter()
        .map(|label| label.chars().count())
        .max()
        .unwrap_or(0);
    let max_label_len = if is_collection_mode {
        max_label_len.saturating_add(12)
    } else {
        max_label_len
    };
    if max_label_len <= 34 {
        ModalSize::Small
    } else if max_label_len <= 90 {
        ModalSize::Medium
    } else {
        ModalSize::Large
    }
}

fn collection_modal_supports_preview(app: &App) -> bool {
    app.viewport_size
        .map(|size| size.width >= 900.0)
        .unwrap_or(true)
}

fn collection_preview_metrics(
    collections: &[crate::sections::collection::CollectionEntry],
) -> (usize, usize) {
    let mut max_chars = "No collection selected".chars().count();
    let mut max_rows = 1usize;

    for collection in collections {
        let (title, lines) = crate::modal::authored_collection_preview(collection);
        max_chars = max_chars.max(title.chars().count());
        max_rows = max_rows.max(lines.len() + 1);
        for line in lines {
            max_chars = max_chars.max(line.chars().count());
        }
    }

    (max_chars, max_rows)
}

fn modal_dimensions_for_content(
    app: &App,
    modal: &crate::modal::SearchModal,
    show_collection_preview: bool,
) -> (f32, f32) {
    if show_collection_preview {
        let left_chars = modal
            .all_entries
            .iter()
            .map(|label| label.chars().count())
            .max()
            .unwrap_or(0) as f32;
        let right_chars = modal
            .collection_state
            .as_ref()
            .map(|state| collection_preview_metrics(&state.collections).0)
            .unwrap_or("No collection selected".chars().count()) as f32;
        let left_width = (left_chars * 8.0 + 92.0).clamp(340.0, 520.0);
        let right_width = (right_chars * 7.2 + 48.0).clamp(260.0, 420.0);
        let width = left_width + right_width + 34.0;
        let capped = app
            .viewport_size
            .map(|size| width.min((size.width - 40.0).max(720.0)))
            .unwrap_or(width);
        return (capped, 320.0);
    }

    let modal_size = modal_size_for_labels(&modal.all_entries, modal.is_collection_mode(), false);
    modal_size.dimensions()
}

fn modal_subpanel<'a>(
    app: &'a App,
    content: impl Into<Element<'a, Message>>,
    focused: bool,
    target: crate::app::ModalPaneTarget,
) -> iced::widget::Container<'a, Message> {
    let panel_background = app.ui_theme.modal_input_background;
    let border_color = if focused {
        app.ui_theme.active
    } else {
        app.ui_theme.modal_input_border
    };
    let text_color = app.ui_theme.modal_text;
    container(
        mouse_area(container(content).height(Length::Fill).padding(6))
            .on_press(Message::ModalPanePressed(target))
            .interaction(iced::mouse::Interaction::Pointer),
    )
    .height(Length::Fill)
    .style(move |_| {
        background_style(panel_background, text_color).border(Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        })
    })
}

fn modal_top_offset(app: &App) -> f32 {
    let _ = app;
    // Leave some air above the composition panel, but keep more of the spare height
    // below the modal stack so it does not feel glued to the bottom edge.
    44.0
}

/// Public entry point called from main.rs view().
pub fn view(app: &App) -> Element<'_, Message> {
    if let Some(ref modal) = app.modal {
        modal_overlay(app, modal)
    } else {
        main_layout(app)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_connected_transition_rendered_unit, collection_preview_metrics,
        modal_unit_runway_layout, transition_unit_display_width, ModalUnitCardKind, ModalUnitSide,
    };
    use crate::app::{
        FocusDirection, ModalArrivalLayer, ModalDepartureLayer, ModalTransitionEasing,
        UnitContentSnapshot, UnitGeometry,
    };
    use crate::data::{HierarchyItem, HierarchyList, ModalStart, ResolvedCollectionConfig};
    use crate::modal_layout::{ModalFocus, ModalListViewSnapshot, ModalStubKind, SimpleModalSequence, SimpleModalUnitLayout};
    use crate::sections::collection::CollectionEntry;
    use std::time::Instant;

    fn item(id: &str, label: &str) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: false,
            output: Some(label.to_string()),
            fields: None,
            branch_fields: Vec::new(),
        }
    }

    fn collection(id: &str, label: &str, items: Vec<HierarchyItem>) -> CollectionEntry {
        CollectionEntry::from_config(&ResolvedCollectionConfig {
            id: id.to_string(),
            label: label.to_string(),
            note_label: None,
            default_enabled: false,
            joiner_style: None,
            lists: vec![HierarchyList {
                id: format!("{id}_list"),
                label: Some(label.to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items,
            }],
        })
    }

    fn snapshot(title: &str) -> ModalListViewSnapshot {
        ModalListViewSnapshot {
            title: title.to_string(),
            query: String::new(),
            rows: vec![format!("{title} row")],
            filtered: vec![0],
            list_cursor: 0,
            list_scroll: 0,
            focus: ModalFocus::List,
        }
    }

    fn connected_transition_fixture(
        direction: FocusDirection,
    ) -> (SimpleModalUnitLayout, ModalArrivalLayer, ModalDepartureLayer) {
        let (dep_leading_stub, dep_trailing_stub, arr_leading_stub, arr_trailing_stub) =
            match direction {
                FocusDirection::Forward => (
                    crate::modal_layout::ModalStubKind::Exit,
                    crate::modal_layout::ModalStubKind::NavRight,
                    crate::modal_layout::ModalStubKind::NavLeft,
                    crate::modal_layout::ModalStubKind::Confirm,
                ),
                FocusDirection::Backward => (
                    crate::modal_layout::ModalStubKind::NavLeft,
                    crate::modal_layout::ModalStubKind::Confirm,
                    crate::modal_layout::ModalStubKind::Exit,
                    crate::modal_layout::ModalStubKind::NavRight,
                ),
            };
        let sequence = SimpleModalSequence {
            snapshots: vec![
                snapshot("dep-a"),
                snapshot("dep-b"),
                snapshot("arr-a"),
                snapshot("arr-b"),
            ],
            active_sequence_index: 2,
        };
        let layout = SimpleModalUnitLayout {
            sequence,
            units: vec![],
        };
        let departure = ModalDepartureLayer {
            content: UnitContentSnapshot {
                modals: vec![snapshot("dep-a"), snapshot("dep-b")],
            },
            geometry: UnitGeometry {
                unit_index: 0,
                modal_index_range: 0..2,
                shows_stubs: true,
                leading_stub_kind: Some(dep_leading_stub),
                trailing_stub_kind: Some(dep_trailing_stub),
                effective_spacer_width: 24.0,
                modal_widths: vec![300.0, 340.0],
                modal_x_offsets: vec![0.0, 324.0],
                first_list_id: "dep-a".to_string(),
                last_list_id: "dep-b".to_string(),
            },
            focus_direction: direction,
            started_at: Instant::now(),
            duration_ms: 220,
            easing: ModalTransitionEasing::Linear,
        };
        let arrival = ModalArrivalLayer {
            unit_index: 1,
            geometry: UnitGeometry {
                unit_index: 1,
                modal_index_range: 2..4,
                shows_stubs: true,
                leading_stub_kind: Some(arr_leading_stub),
                trailing_stub_kind: Some(arr_trailing_stub),
                effective_spacer_width: 24.0,
                modal_widths: vec![320.0, 360.0],
                modal_x_offsets: vec![0.0, 344.0],
                first_list_id: "arr-a".to_string(),
                last_list_id: "arr-b".to_string(),
            },
            focus_direction: direction,
            started_at: Instant::now(),
            duration_ms: 220,
            easing: ModalTransitionEasing::Linear,
        };
        (layout, arrival, departure)
    }

    #[test]
    fn collection_preview_metrics_use_largest_preview_across_all_collections() {
        let collections = vec![
            collection("short", "Short", vec![item("a", "A")]),
            collection(
                "long",
                "Long",
                vec![
                    item("b", "A much longer preview row"),
                    item("c", "C"),
                    item("d", "D"),
                ],
            ),
        ];

        let (max_chars, max_rows) = collection_preview_metrics(&collections);

        assert_eq!(max_chars, "[ ] A much longer preview row".chars().count());
        assert_eq!(max_rows, 4);
    }

    #[test]
    fn collection_preview_metrics_keep_empty_state_fallbacks() {
        let (max_chars, max_rows) = collection_preview_metrics(&[]);

        assert_eq!(max_chars, "No collection selected".chars().count());
        assert_eq!(max_rows, 1);
    }

    #[test]
    fn modal_unit_runway_layout_keeps_forward_departure_moving_past_left_edge() {
        let viewport_width = 1200.0;
        let row_width = 780.0;
        let shift = -684.0;

        let (outer_width, left_pad, _) = modal_unit_runway_layout(viewport_width, row_width, shift);
        let actual_left = (viewport_width - outer_width) * 0.5 + left_pad;

        assert_eq!(actual_left, -474.0);
    }

    #[test]
    fn modal_unit_runway_layout_keeps_backward_departure_moving_past_right_edge() {
        let viewport_width = 1200.0;
        let row_width = 468.0;
        let shift = 684.0;

        let (outer_width, left_pad, _) = modal_unit_runway_layout(viewport_width, row_width, shift);
        let actual_left = (viewport_width - outer_width) * 0.5 + left_pad;
        let actual_right = actual_left + row_width;

        assert_eq!(actual_right, 1518.0);
    }

    #[test]
    fn connected_transition_strip_has_single_forward_transition_stub() {
        let (layout, arrival, departure) = connected_transition_fixture(FocusDirection::Forward);
        let rendered =
            build_connected_transition_rendered_unit(120.0, &layout, &arrival, &departure, 0.25);

        let mut nav_right_count = 0;
        let mut exit_count = 0;
        let mut confirm_count = 0;
        let mut active_count = 0;

        for card in rendered.cards {
            match card.kind {
                ModalUnitCardKind::Stub {
                    side, stub_kind, ..
                } => match stub_kind {
                    crate::modal_layout::ModalStubKind::NavRight => {
                        nav_right_count += 1;
                        assert_eq!(side, ModalUnitSide::Right);
                        assert_eq!(card.alpha, 1.0);
                    }
                    crate::modal_layout::ModalStubKind::Exit => {
                        exit_count += 1;
                        assert_eq!(card.alpha, 0.75);
                    }
                    crate::modal_layout::ModalStubKind::Confirm => {
                        confirm_count += 1;
                        assert_eq!(card.alpha, 0.25);
                    }
                    _ => {}
                },
                ModalUnitCardKind::Active { .. } => {
                    active_count += 1;
                    assert_eq!(card.width, 320.0);
                    assert_eq!(card.alpha, 0.25);
                }
                ModalUnitCardKind::Preview { .. } => {}
            }
        }

        assert_eq!(nav_right_count, 1);
        assert_eq!(exit_count, 1);
        assert_eq!(confirm_count, 1);
        assert_eq!(active_count, 1);
    }

    #[test]
    fn connected_transition_strip_has_single_backward_transition_stub() {
        let (layout, arrival, departure) = connected_transition_fixture(FocusDirection::Backward);
        let rendered =
            build_connected_transition_rendered_unit(120.0, &layout, &arrival, &departure, 0.6);

        let nav_left_cards = rendered
            .cards
            .iter()
            .filter(|card| {
                matches!(
                    card.kind,
                    ModalUnitCardKind::Stub {
                        stub_kind: crate::modal_layout::ModalStubKind::NavLeft,
                        ..
                    }
                )
            })
            .collect::<Vec<_>>();
        let active_cards = rendered
            .cards
            .iter()
            .filter(|card| matches!(card.kind, ModalUnitCardKind::Active { .. }))
            .collect::<Vec<_>>();

        assert_eq!(nav_left_cards.len(), 1);
        assert_eq!(nav_left_cards[0].alpha, 1.0);
        assert_eq!(active_cards.len(), 1);
        assert_eq!(active_cards[0].width, 320.0);
        assert_eq!(active_cards[0].alpha, 0.6);
    }

    // Helpers shared by the geometry tests below.
    fn strip_inner_left(viewport: f32, row_width: f32, shift: f32) -> f32 {
        let (outer_width, left_pad, _) = modal_unit_runway_layout(viewport, row_width, shift);
        (viewport - outer_width) * 0.5 + left_pad
    }

    #[test]
    fn transition_dep_unit_is_centred_in_viewport_at_p0() {
        // With p = 0 the dep unit's centre must sit at viewport_centre.
        let viewport = 1200.0;
        let stub = 120.0;
        let (_, arrival, departure) = connected_transition_fixture(FocusDirection::Forward);
        let spacer = departure.geometry.effective_spacer_width;

        let dep_w = transition_unit_display_width(&departure.geometry, stub);
        let arr_w = transition_unit_display_width(&arrival.geometry, stub);

        // Combined forward strip width: dep + arr - shared stub.
        let row_w = dep_w + arr_w - stub;

        let shift_p0 = (row_w - dep_w) / 2.0;
        let _ = spacer; // used for slide_distance derivation only
        let inner_left = strip_inner_left(viewport, row_w, shift_p0);
        let dep_centre = inner_left + dep_w / 2.0;

        assert!(
            (dep_centre - viewport / 2.0).abs() < 0.5,
            "dep centre {dep_centre} should be at viewport centre {}",
            viewport / 2.0
        );
    }

    #[test]
    fn transition_arr_unit_is_centred_in_viewport_at_p1() {
        // With p = 1 the arr unit's centre must sit at viewport_centre.
        let viewport = 1200.0;
        let stub = 120.0;
        let (_, arrival, departure) = connected_transition_fixture(FocusDirection::Forward);

        let dep_w = transition_unit_display_width(&departure.geometry, stub);
        let arr_w = transition_unit_display_width(&arrival.geometry, stub);
        let row_w = dep_w + arr_w - stub;

        let slide = (dep_w + arr_w) / 2.0 - stub;
        let shift_p1 = (row_w - dep_w) / 2.0 - slide;

        let inner_left = strip_inner_left(viewport, row_w, shift_p1);
        // arr section starts at (dep_w - stub) within the strip.
        let arr_centre = inner_left + (dep_w - stub) + arr_w / 2.0;

        assert!(
            (arr_centre - viewport / 2.0).abs() < 0.5,
            "arr centre {arr_centre} should be at viewport centre {}",
            viewport / 2.0
        );
    }

    #[test]
    fn transition_slide_distance_is_symmetric_for_forward_and_backward() {
        // Forward and backward use the same geometry fixture with the stubs
        // swapped.  The slide_distance should be identical in both cases.
        let stub = 120.0;
        let (_, arr_fwd, dep_fwd) = connected_transition_fixture(FocusDirection::Forward);
        let (_, arr_bwd, dep_bwd) = connected_transition_fixture(FocusDirection::Backward);

        let dep_w_fwd = transition_unit_display_width(&dep_fwd.geometry, stub);
        let arr_w_fwd = transition_unit_display_width(&arr_fwd.geometry, stub);
        let slide_fwd = (dep_w_fwd + arr_w_fwd) / 2.0 - stub;

        let dep_w_bwd = transition_unit_display_width(&dep_bwd.geometry, stub);
        let arr_w_bwd = transition_unit_display_width(&arr_bwd.geometry, stub);
        let slide_bwd = (dep_w_bwd + arr_w_bwd) / 2.0 - stub;

        assert_eq!(slide_fwd, slide_bwd, "forward and backward slide distances must match");
    }

    #[test]
    fn connected_transition_forward_strip_puts_dep_before_arr() {
        // The Active card belongs to arr.  In a forward strip it must appear to
        // the right of the transition stub (NavRight), not to the left.
        let (layout, arrival, departure) = connected_transition_fixture(FocusDirection::Forward);
        let rendered =
            build_connected_transition_rendered_unit(120.0, &layout, &arrival, &departure, 0.5);

        let transition_pos = rendered.cards.iter().position(|c| {
            matches!(
                c.kind,
                ModalUnitCardKind::Stub { stub_kind: ModalStubKind::NavRight, .. }
            )
        });
        let active_pos = rendered
            .cards
            .iter()
            .position(|c| matches!(c.kind, ModalUnitCardKind::Active { .. }));

        let (t, a) = (transition_pos.unwrap(), active_pos.unwrap());
        assert!(
            a > t,
            "Forward: Active (arr) card must come after transition stub; active={a} transition={t}"
        );
    }

    #[test]
    fn connected_transition_backward_strip_puts_arr_before_dep() {
        // The Active card belongs to arr.  In a backward strip it must appear to
        // the left of the transition stub (NavLeft), not to the right.
        let (layout, arrival, departure) = connected_transition_fixture(FocusDirection::Backward);
        let rendered =
            build_connected_transition_rendered_unit(120.0, &layout, &arrival, &departure, 0.5);

        let transition_pos = rendered.cards.iter().position(|c| {
            matches!(
                c.kind,
                ModalUnitCardKind::Stub { stub_kind: ModalStubKind::NavLeft, .. }
            )
        });
        let active_pos = rendered
            .cards
            .iter()
            .position(|c| matches!(c.kind, ModalUnitCardKind::Active { .. }));

        let (t, a) = (transition_pos.unwrap(), active_pos.unwrap());
        assert!(
            a < t,
            "Backward: Active (arr) card must come before transition stub; active={a} transition={t}"
        );
    }

    #[test]
    fn transition_dep_unit_is_centred_in_viewport_at_p0_backward() {
        // With p = 0 the dep unit's centre must sit at viewport_centre for a
        // backward transition.  In the backward strip dep occupies the right side,
        // so the shift formula is negated relative to the forward case.
        let viewport = 1200.0;
        let stub = 120.0;
        let (_, arrival, departure) = connected_transition_fixture(FocusDirection::Backward);

        let dep_w = transition_unit_display_width(&departure.geometry, stub);
        let arr_w = transition_unit_display_width(&arrival.geometry, stub);
        // Total strip width is the same as forward (same content, different order).
        let row_w = dep_w + arr_w - stub;

        // Backward shift formula at p = 0.
        let shift_p0 = -(row_w - dep_w) / 2.0;
        let inner_left = strip_inner_left(viewport, row_w, shift_p0);

        // In the backward strip [arr | transition | dep], dep starts at row_w - dep_w.
        let dep_centre = inner_left + (row_w - dep_w) + dep_w / 2.0;

        assert!(
            (dep_centre - viewport / 2.0).abs() < 0.5,
            "dep centre {dep_centre} should be at viewport centre {}",
            viewport / 2.0
        );
    }

    #[test]
    fn transition_arr_unit_is_centred_in_viewport_at_p1_backward() {
        // With p = 1 the arr unit's centre must sit at viewport_centre for a
        // backward transition.  Arr occupies the left side of the backward strip.
        let viewport = 1200.0;
        let stub = 120.0;
        let (_, arrival, departure) = connected_transition_fixture(FocusDirection::Backward);

        let dep_w = transition_unit_display_width(&departure.geometry, stub);
        let arr_w = transition_unit_display_width(&arrival.geometry, stub);
        let row_w = dep_w + arr_w - stub;

        let slide = (dep_w + arr_w) / 2.0 - stub;
        // Backward shift formula at p = 1.
        let shift_p1 = -(row_w - dep_w) / 2.0 + slide;
        let inner_left = strip_inner_left(viewport, row_w, shift_p1);

        // In the backward strip arr starts at the left edge; arr centre = arr_w / 2.
        let arr_centre = inner_left + arr_w / 2.0;

        assert!(
            (arr_centre - viewport / 2.0).abs() < 0.5,
            "arr centre {arr_centre} should be at viewport centre {}",
            viewport / 2.0
        );
    }
}
