// View layer: builds the iced element tree from app state.
// Modal unit rendering is in modal_unit.rs (submodule); everything else lives here.

mod modal_unit;
use modal_unit::*;

use crate::app::{App, ErrorModalFlashKind, Focus, MapHintLevel, SectionState};
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

    for (group_idx, group) in app.data.template.children.iter().enumerate() {
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
        let group_indices: Vec<usize> = app
            .navigation
            .iter()
            .enumerate()
            .filter_map(|(flat_idx, entry)| (entry.group_index == group_idx).then_some(flat_idx))
            .collect();
        for (group_section_idx, flat_idx) in group_indices.into_iter().enumerate() {
            let Some(sec) = app.config_for_index(flat_idx) else {
                continue;
            };
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
    forced_background: Option<Color>,
    forced_text_color: Option<Color>,
) -> iced::widget::button::Style {
    let background = match forced_background {
        Some(background) => background,
        None => match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                app_theme.modal_item_hovered_background
            }
            iced::widget::button::Status::Active | iced::widget::button::Status::Disabled => {
                app_theme.modal_item_background
            }
        },
    };

    iced::widget::button::Style {
        background: Some(Background::Color(background)),
        text_color: forced_text_color.unwrap_or(app_theme.modal_text),
        border: Border {
            color: background,
            width: 0.0,
            radius: 2.0.into(),
        },
        shadow: iced::Shadow::default(),
    }
}

fn modal_row_cursor_accent(
    app_theme: &crate::theme::AppTheme,
    is_current: bool,
    is_active_modal: bool,
) -> Option<Color> {
    if is_current {
        Some(if is_active_modal {
            app_theme.active
        } else {
            app_theme.active_preview
        })
    } else {
        None
    }
}

fn confirmed_row_border_color(
    app_theme: &crate::theme::AppTheme,
    is_confirmed: bool,
    cursor_accent: Option<Color>,
) -> Option<Color> {
    if !is_confirmed {
        None
    } else {
        Some(cursor_accent.unwrap_or(app_theme.selected))
    }
}

fn modal_row_color(
    cursor_accent: Option<Color>,
    confirmed_color: Option<Color>,
    fallback: Color,
) -> Color {
    cursor_accent.or(confirmed_color).unwrap_or(fallback)
}

fn modal_row_container_style(border_color: Option<Color>) -> iced::widget::container::Style {
    match border_color {
        Some(border_color) => iced::widget::container::Style::default().border(Border {
            color: border_color,
            width: 1.0,
            radius: 2.0.into(),
        }),
        None => iced::widget::container::Style::default(),
    }
}

fn plain_modal_row_button_style() -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: None,
        text_color: Color::TRANSPARENT,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
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
    app.wizard_hint_labels().fields
}

fn wizard_window(app: &App, cursor: usize, len: usize) -> std::ops::Range<usize> {
    modal_row_window(cursor, len, app.data.keybindings.hints.len().max(1))
}

fn modal_row_window(cursor: usize, len: usize, window_size: usize) -> std::ops::Range<usize> {
    if len == 0 {
        return 0..0;
    }
    let window_size = window_size.max(1);
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
    crate::sections::multi_field::render_field_display_for_confirmed_values(
        confirmed_values,
        field,
        &app.assigned_values,
        &app.config.sticky_values,
    )
}

fn section_header(app: &App) -> Vec<Element<'_, Message>> {
    let Some(sec) = app.config_for_index(app.current_idx) else {
        return vec![text("Missing section").color(app.ui_theme.error).into()];
    };

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
            &app.assigned_values,
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
                            match crate::sections::multi_field::resolve_multifield_value_for_confirmed_slot(
                                value,
                                field,
                                &app.assigned_values,
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
                        crate::sections::multi_field::resolve_field_label_for_confirmed_slot(
                            value,
                            field,
                            &app.assigned_values,
                            &app.config.sticky_values,
                        )
                    })
                    .unwrap_or_else(|| {
                        crate::sections::multi_field::resolve_field_label(
                            &crate::sections::header::HeaderFieldValue::Text(String::new()),
                            field,
                            &app.assigned_values,
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
        let field_label = crate::sections::multi_field::resolve_field_label_for_confirmed_slot(
            &confirmed,
            field,
            &app.assigned_values,
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
    if app.navigation.is_empty() || app.current_idx >= app.navigation.len() {
        return column![text("No sections loaded").color(app.ui_theme.error)]
            .width(Length::Fill)
            .padding(4)
            .into();
    }

    let Some(sec) = app.config_for_index(app.current_idx) else {
        return column![text("Missing runtime section").color(app.ui_theme.error)]
            .width(Length::Fill)
            .padding(4)
            .into();
    };
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
        style: iced::font::Style::Oblique,
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
    for (section_idx, entry) in app.navigation.iter().enumerate() {
        let Some(section) = app.config_for_node_id(&entry.node_id) else {
            continue;
        };
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
                    &app.assigned_values,
                    &app.config.sticky_values,
                );
                let Some(rendered) = resolved.export_value() else {
                    continue;
                };
                let candidate = crate::sections::multi_field::render_note_line(
                    section,
                    field,
                    &empty_value,
                    &app.assigned_values,
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
                let resolved =
                    crate::sections::multi_field::resolve_multifield_value_for_confirmed_slot(
                        value,
                        field,
                        &app.assigned_values,
                        &app.config.sticky_values,
                    );
                let Some(rendered) = resolved.export_value() else {
                    continue;
                };
                let candidate = crate::sections::multi_field::render_note_line_for_confirmed_slot(
                    section,
                    field,
                    value,
                    &app.assigned_values,
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

    for (section_idx, entry) in app.navigation.iter().enumerate() {
        let Some(section) = app.config_for_node_id(&entry.node_id) else {
            continue;
        };
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
    for (idx, group) in app.data.template.children.iter().enumerate() {
        let heading = group.note.note_label.as_deref();
        if heading == Some(line) {
            return Some(idx);
        }
    }

    for (section_idx, entry) in app.navigation.iter().enumerate() {
        let Some(section) = app.config_for_node_id(&entry.node_id) else {
            continue;
        };
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
        &app.assigned_values,
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
                "Modal: {focus} | Confirm toggles | Select switches sides | Left/Esc moves out | Right enters or commits | Super-confirm commits"
            );
        }
        let focus = match modal.focus {
            ModalFocus::SearchBar => "search",
            ModalFocus::List => "list",
        };
        return format!(
            "Modal: {focus} | {} matches | Confirm commits | Left/Right browse parts | Space enters list | Ctrl+E edits entry | Esc closes",
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
    parts.push("Select enters section | Esc returns".to_string());
    parts.join(" | ")
}

fn wizard_status_text(app: &App) -> String {
    let Some(sec) = app.config_for_index(app.current_idx) else {
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
            parts.push("Select opens choices".to_string());
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
            parts.push(
                "Select toggles | Confirm opens | Backspace resets | Esc confirms".to_string(),
            );
        }
        Some(SectionState::Checklist(state)) => {
            let checked = state.checked.iter().filter(|&&checked| checked).count();
            parts.push(format!("Checked: {checked}"));
            parts.push("Select toggles | Confirm confirms".to_string());
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
const COLLECTION_MODAL_MIN_HEIGHT: f32 = 240.0;
const COLLECTION_MODAL_CHROME_HEIGHT: f32 = 112.0;
const COLLECTION_MODAL_ROW_HEIGHT: f32 = 28.0;
const COLLECTION_PREVIEW_TITLE_HEIGHT: f32 = 24.0;
const COLLECTION_PREVIEW_CARD_PADDING: f32 = 6.0;

#[derive(Debug, Clone, PartialEq)]
struct CollectionPreviewStripLayout {
    card_tops: Vec<f32>,
    card_heights: Vec<f32>,
    total_height: f32,
    y_offset: f32,
}

fn collection_preview_landing_color() -> Color {
    Color::from_rgb8(0xD9, 0x7A, 0x00)
}

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
            crate::modal_layout::ModalStubKind::NavLeft
            | crate::modal_layout::ModalStubKind::NavRight => {
                app.ui_theme.modal_nav_stub_background
            }
            crate::modal_layout::ModalStubKind::Exit => app.ui_theme.modal_exit_stub_background,
            crate::modal_layout::ModalStubKind::Confirm => {
                app.ui_theme.modal_confirm_stub_background
            }
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

fn semantic_stub_card<'a>(
    app: &'a App,
    stub_kind: crate::modal_layout::ModalStubKind,
    height: f32,
) -> Element<'a, Message> {
    let text_color = match stub_kind {
        crate::modal_layout::ModalStubKind::NavLeft
        | crate::modal_layout::ModalStubKind::NavRight => app.ui_theme.modal_nav_stub_text,
        crate::modal_layout::ModalStubKind::Exit => app.ui_theme.modal_exit_stub_text,
        crate::modal_layout::ModalStubKind::Confirm => app.ui_theme.modal_confirm_stub_text,
    };

    modal_card(
        app,
        container(
            iced::widget::text(stub_kind.symbol().to_string())
                .font(app.ui_theme.font_modal)
                .size(24)
                .color(text_color),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill),
        ModalRenderMode::Preview,
        ModalCardRole::Stub(stub_kind),
        false,
        app.ui_theme.modal_stub_width,
        height,
        1.0,
    )
}

fn render_semantic_modal_shell<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
    content: Element<'a, Message>,
    stub_height: f32,
) -> Element<'a, Message> {
    let semantics = modal.edge_semantics(&app.assigned_values, &app.config.sticky_values);
    row![
        semantic_stub_card(app, semantics.left, stub_height),
        content,
        semantic_stub_card(app, semantics.right, stub_height),
    ]
    .spacing(app.modal_spacer_width())
    .into()
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
    let modal_hints = app.visible_modal_hint_labels();
    for window_pos in snapshot.list_scroll..end {
        let Some(&entry_idx) = snapshot.filtered.get(window_pos) else {
            continue;
        };
        let Some(label) = snapshot.rows.get(entry_idx).cloned() else {
            continue;
        };
        let is_current = window_pos == snapshot.list_cursor;
        let is_confirmed = snapshot.confirmed_row == Some(entry_idx);
        let cursor_accent = modal_row_cursor_accent(&app.ui_theme, is_current, false);
        let confirmed_color = if is_confirmed {
            Some(app.ui_theme.selected)
        } else {
            None
        };
        let border_color = confirmed_row_border_color(&app.ui_theme, is_confirmed, cursor_accent);
        let hint = modal_hints
            .get(window_pos - snapshot.list_scroll)
            .map(|hint| display_hint_label(app, hint))
            .unwrap_or_default();
        let marker_color = modal_row_color(
            cursor_accent,
            confirmed_color,
            app.ui_theme.modal_muted_text,
        );
        let hint_color = modal_row_color(
            cursor_accent,
            confirmed_color,
            app.ui_theme.modal_muted_text,
        );
        let label_color = modal_row_color(
            cursor_accent,
            confirmed_color,
            app.ui_theme.modal_muted_text,
        );
        let row_content = row![
            container(
                text(if is_current { ">" } else { " " })
                    .font(app.ui_theme.font_modal)
                    .color(apply_alpha(marker_color, alpha)),
            )
            .align_left(Length::Fixed(14.0)),
            container(
                text(format!("{hint:<4}"))
                    .font(app.ui_theme.font_modal)
                    .color(apply_alpha(hint_color, alpha)),
            )
            .align_left(Length::Fixed(24.0)),
            container(
                text(label)
                    .font(app.ui_theme.font_modal)
                    .color(apply_alpha(label_color, alpha)),
            )
            .width(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill);
        items.push(
            container(row_content)
                .width(Length::Fill)
                .style(move |_| {
                    modal_row_container_style(border_color.map(|color| apply_alpha(color, alpha)))
                })
                .into(),
        );
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
    if let Some(part_label) =
        modal.current_part_label(&app.assigned_values, &app.config.sticky_values)
    {
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
            .padding([6, 8])
            .style(move |_theme, status| modal_input_style(&app_theme, status))
            .into(),
    );

    let end = (modal.list_scroll + modal.window_size).min(modal.filtered.len());
    let modal_hints = app.visible_modal_hint_labels();
    let mut list_items: Vec<Element<'a, Message>> = Vec::new();
    let confirmed_row = modal.confirmed_row_for_current_list();
    for window_pos in modal.list_scroll..end {
        if let Some(&entry_idx) = modal.filtered.get(window_pos) {
            let label = &modal.all_entries[entry_idx];
            let is_current = window_pos == modal.list_cursor;
            let is_confirmed = confirmed_row == Some(entry_idx);
            let cursor_accent = modal_row_cursor_accent(&ui_theme, is_current, true);
            let confirmed_color = if is_confirmed {
                Some(ui_theme.selected)
            } else {
                None
            };
            let border_color = confirmed_row_border_color(&ui_theme, is_confirmed, cursor_accent);
            let color = modal_row_color(cursor_accent, confirmed_color, ui_theme.modal_muted_text);
            let hint = modal_hints
                .get(window_pos - modal.list_scroll)
                .map(|hint| display_hint_label(app, hint))
                .unwrap_or_default();
            let hint_color = modal_row_color(
                cursor_accent,
                confirmed_color,
                if matches!(modal.focus, ModalFocus::List) {
                    ui_theme.modal_hint_text
                } else {
                    ui_theme.modal_muted_text
                },
            );
            let marker = if is_current { ">" } else { " " };
            let marker_color =
                modal_row_color(cursor_accent, confirmed_color, ui_theme.modal_muted_text);
            let row_content = row![
                container(text(marker).font(ui_theme.font_modal).color(marker_color))
                    .align_left(Length::Fixed(14.0)),
                container(
                    text(format!("{hint:<4}"))
                        .font(ui_theme.font_modal)
                        .color(hint_color),
                )
                .align_left(Length::Fixed(24.0)),
                container(text(label).font(ui_theme.font_modal).color(color)).width(Length::Fill),
            ]
            .spacing(0)
            .width(Length::Fill);
            list_items.push(
                button(
                    container(row_content)
                        .width(Length::Fill)
                        .style(move |_| modal_row_container_style(border_color)),
                )
                .width(Length::Fill)
                .padding(0)
                .on_press(Message::ModalSelect(window_pos))
                .style(move |_theme, _status| plain_modal_row_button_style())
                .into(),
            );
        }
    }
    modal_items.push(
        themed_scrollable_with_theme(ui_theme.clone(), column(list_items).spacing(4))
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
    composition_editing: bool,
    alpha: f32,
) -> Option<Element<'a, Message>> {
    let structured_spans = crate::app::compute_field_composition_spans(
        modal,
        &app.assigned_values,
        &app.config.sticky_values,
    );
    if structured_spans.is_empty() && modal.manual_override.is_none() {
        return None;
    }

    let fade = move |color| {
        if alpha < 1.0 {
            apply_alpha(color, alpha)
        } else {
            color
        }
    };
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
            span::<Message, iced::Font>(span_data.text).color(fade(color))
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
            span::<Message, iced::Font>(span_data.text).color(fade(blend_color(
                base,
                app.ui_theme.modal_muted_text,
                0.45,
            )))
        })
        .collect::<Vec<_>>();

    let panel_width = entry_composition_panel_width(app, modal_width);
    let label_color = fade(blend_color(
        app.ui_theme.modal_hint_text,
        app.ui_theme.text,
        0.25,
    ));
    let panel_background = fade(blend_color(
        app.ui_theme.modal_panel_background,
        app.ui_theme.modal_input_background,
        0.15,
    ));
    let border_color = fade(blend_color(
        app.ui_theme.modal_input_border,
        app.ui_theme.text,
        0.25,
    ));
    let helper_text = if composition_editing {
        "Manual edit mode | Enter or Esc exits | Ctrl+R resets"
    } else if modal.manual_override.is_some() {
        "Manual override active | Ctrl+E edits | Ctrl+R resets"
    } else {
        "Structured composition | Ctrl+E edits current field"
    };
    let helper_color = fade(blend_color(
        app.ui_theme.modal_muted_text,
        app.ui_theme.text,
        0.2,
    ));
    let input_theme = blended_modal_theme(app, alpha);

    let main_content: Element<'a, Message> = if composition_editing {
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
            .color(fade(blend_color(
                app.ui_theme.text,
                app.ui_theme.selected,
                0.18,
            )))
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
                background_style(panel_background, fade(app.ui_theme.text)).border(Border {
                    color: border_color,
                    width: 1.0,
                    radius: 6.0.into(),
                })
            })
            .into(),
    )
}

fn entry_composition_panel_width(app: &App, modal_width: f32) -> f32 {
    let max_width = 980.0_f32.max(modal_width);
    app.viewport_size
        .map(|size| (size.width * 0.9).clamp(modal_width, max_width))
        .unwrap_or(820.0)
        .max(modal_width)
}

fn simple_modal_unit_root_width(layout: &SimpleModalUnitLayout) -> Option<f32> {
    layout
        .sequence
        .snapshots
        .get(layout.sequence.active_sequence_index)
        .map(|snapshot| modal_list_view_dimensions(snapshot).0)
}

fn retained_close_root_width(departure: &crate::app::ModalDepartureLayer) -> Option<f32> {
    let active_list_idx = departure.modal.as_ref()?.field_flow.list_idx;
    if !departure
        .geometry
        .modal_index_range
        .contains(&active_list_idx)
    {
        return None;
    }
    let relative_idx = active_list_idx - departure.geometry.modal_index_range.start;
    departure.geometry.modal_widths.get(relative_idx).copied()
}

fn retained_modal_close_transition(app: &App) -> Option<(&crate::app::ModalDepartureLayer, f32)> {
    match app.modal_transitions.last() {
        Some(crate::app::ModalTransitionLayer::ModalClose {
            departure,
            slide_distance,
        }) => Some((departure, *slide_distance)),
        _ => None,
    }
}

fn retained_composition_close_transition(
    app: &App,
) -> Option<(&crate::app::ModalCompositionLayer, f32)> {
    match app.modal_composition_transition.as_ref() {
        Some(crate::app::ModalCompositionTransition::Close {
            departure,
            slide_distance,
        }) => Some((departure, *slide_distance)),
        _ => None,
    }
}

fn composition_panel_layer<'a>(
    app: &App,
    panel: Element<'a, Message>,
    panel_width: f32,
    shift: f32,
) -> Element<'a, Message> {
    let viewport_width = app
        .viewport_size
        .map(|size| size.width)
        .unwrap_or(panel_width);
    let left = (viewport_width - panel_width) * 0.5 + shift;
    modal_unit::ClipTranslate::auto_height(left, viewport_width, panel).into()
}

fn should_render_modal_overlay(app: &App) -> bool {
    app.modal.is_some() || retained_modal_close_transition(app).is_some()
}

/// Build the modal overlay when a search modal is active, or when a retained close
/// transition still needs to paint after the live modal session has ended.
fn modal_overlay<'a>(
    app: &'a App,
    modal: Option<&'a crate::modal::SearchModal>,
) -> Element<'a, Message> {
    let retained_close = retained_modal_close_transition(app);
    let is_collection_mode = modal.is_some_and(|modal| modal.is_collection_mode());
    let show_collection_preview = is_collection_mode && collection_modal_supports_preview(app);

    let simple_unit_layout: Option<&crate::modal_layout::SimpleModalUnitLayout> =
        if modal.is_some_and(|modal| !modal.is_collection_mode()) {
            app.modal_unit_layout.as_ref()
        } else {
            None
        };
    let (mut modal_width, fallback_height) = if let Some(modal) = modal {
        modal_dimensions_for_content(app, modal, show_collection_preview)
    } else if let Some((departure, _)) = retained_close {
        (
            retained_close_root_width(departure).unwrap_or_else(|| {
                transition_unit_display_width(&departure.geometry, app.ui_theme.modal_stub_width)
            }),
            320.0,
        )
    } else {
        (0.0, 320.0)
    };
    if let Some(layout) = simple_unit_layout {
        if let Some(unit_width) = simple_modal_unit_root_width(layout) {
            modal_width = unit_width;
        }
    }
    let modal_height = if let Some(modal) = modal.filter(|modal| modal.is_collection_mode()) {
        collection_modal_height(app, modal, fallback_height)
    } else {
        modal_height_for_viewport(app.viewport_size.map(|size| size.height), fallback_height)
    };
    let semantic_stub_height = if is_collection_mode {
        collection_modal_envelope_height(app)
    } else {
        modal_height
    };
    let top_offset = if is_collection_mode {
        0.0
    } else {
        modal_top_offset(app)
    };
    let active_modal = if let Some(modal) = modal.filter(|modal| modal.is_collection_mode()) {
        if show_collection_preview {
            collection_modal_split_panes(app, modal, modal_width)
        } else {
            let mut list_items: Vec<Element<'a, Message>> = Vec::new();
            render_collection_modal_items(app, modal, &mut list_items);
            container(
                collection_left_panel(
                    app,
                    modal,
                    container(column(list_items).spacing(2)).height(Length::Fill),
                    collection_list_panel_height(app, modal),
                )
                .width(Length::Fixed(modal_width.min(560.0))),
            )
            .height(Length::Fixed(collection_modal_envelope_height(app)))
            .center_y(Length::Fill)
            .into()
        }
    } else {
        Space::with_width(Length::Shrink).into()
    };

    let modal_stream: Element<'a, Message> = if let Some(modal) = modal {
        if !modal.is_collection_mode() {
            if let Some(current_layout) = app.modal_unit_layout.as_ref() {
                if let Some(current_unit) = current_layout.units.get(app.active_unit_index) {
                    let mut layers = Stack::new();

                    match app.modal_transitions.last() {
                        Some(crate::app::ModalTransitionLayer::ConnectedTransition {
                            arrival,
                            departure,
                            slide_distance,
                        }) => {
                            let p = arrival.eased_progress();
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
                                *slide_distance,
                                true,
                            ));
                        }
                        Some(crate::app::ModalTransitionLayer::ModalOpen {
                            arrival,
                            slide_distance,
                        }) => {
                            let p = arrival.eased_progress();
                            let rendered = build_modal_open_rendered_unit(
                                app.ui_theme.modal_stub_width,
                                current_layout,
                                arrival,
                                p,
                            );
                            layers = layers.push(render_modal_unit(
                                app,
                                &rendered,
                                Some(modal),
                                modal_height,
                                modal_open_shift(p, *slide_distance),
                                true,
                            ));
                        }
                        _ => {
                            let rendered_current = build_rendered_modal_unit(
                                app,
                                modal,
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
                    }
                    layers.width(Length::Fill).into()
                } else {
                    render_semantic_modal_shell(
                        app,
                        modal,
                        modal_card(
                            app,
                            active_simple_modal_content(app, modal, 1.0),
                            ModalRenderMode::Interactive,
                            ModalCardRole::Active,
                            true,
                            modal_width,
                            modal_height,
                            1.0,
                        ),
                        semantic_stub_height,
                    )
                }
            } else {
                render_semantic_modal_shell(
                    app,
                    modal,
                    modal_card(
                        app,
                        active_simple_modal_content(app, modal, 1.0),
                        ModalRenderMode::Interactive,
                        ModalCardRole::Active,
                        true,
                        modal_width,
                        modal_height,
                        1.0,
                    ),
                    semantic_stub_height,
                )
            }
        } else {
            render_semantic_modal_shell(app, modal, active_modal, semantic_stub_height)
        }
    } else if let Some((departure, slide_distance)) = retained_close {
        let p = departure.eased_progress();
        let rendered =
            build_modal_close_rendered_unit(app.ui_theme.modal_stub_width, departure, 1.0 - p);
        render_modal_unit(
            app,
            &rendered,
            None,
            modal_height,
            modal_close_shift(departure.focus_direction, p, slide_distance),
            false,
        )
    } else {
        Space::with_width(Length::Shrink).into()
    };

    let retained_composition_close = retained_composition_close_transition(app);
    let composition_panel = if let Some(modal) = modal.filter(|modal| !modal.is_collection_mode()) {
        let (alpha, shift) = match app.modal_composition_transition.as_ref() {
            Some(crate::app::ModalCompositionTransition::Open {
                arrival,
                slide_distance,
            }) => {
                let p = arrival.eased_progress();
                (p, modal_open_shift(p, *slide_distance))
            }
            _ => (1.0, 0.0),
        };
        let panel_width = entry_composition_panel_width(app, modal_width);
        entry_composition_panel(
            app,
            modal,
            modal_width,
            app.modal_composition_editing,
            alpha,
        )
        .map(|panel| composition_panel_layer(app, panel, panel_width, shift))
    } else if let Some((departure, slide_distance)) = retained_composition_close {
        let p = departure.eased_progress();
        let panel_width = entry_composition_panel_width(app, modal_width);
        entry_composition_panel(app, &departure.modal, modal_width, false, 1.0 - p).map(|panel| {
            composition_panel_layer(
                app,
                panel,
                panel_width,
                modal_close_shift(departure.focus_direction, p, slide_distance),
            )
        })
    } else {
        None
    };
    let base = main_layout(app);
    let overlay_spacing = if is_collection_mode { 0.0 } else { 14.0 };
    Stack::new()
        .push(base)
        .push(
            mouse_area(
                container(
                    column(
                        std::iter::once(Space::with_height(Length::Fixed(top_offset)).into())
                            .chain(
                                composition_panel.into_iter().map(Into::into).chain(
                                    std::iter::once(
                                        container(modal_stream)
                                            .width(Length::Fill)
                                            .center_x(Length::Fill)
                                            .into(),
                                    ),
                                ),
                            )
                            .collect::<Vec<Element<'a, Message>>>(),
                    )
                    .spacing(overlay_spacing),
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
        true,
        crate::app::ModalPaneTarget::Left,
    );
}

fn collection_modal_split_panes<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
    modal_width: f32,
) -> Element<'a, Message> {
    let Some(_state) = modal.collection_state.as_ref() else {
        return Space::with_height(Length::Shrink).into();
    };
    let list_height = collection_list_panel_height(app, modal);
    let envelope_height = collection_modal_envelope_height(app);
    let left_panel_width = collection_left_panel_width(modal);
    let mut left_items = Vec::new();
    render_collection_modal_items(app, modal, &mut left_items);
    let right_panel = collection_modal_preview(app, modal);
    let left_panel = collection_left_panel(
        app,
        modal,
        container(column(left_items).spacing(2)).height(Length::Fill),
        list_height,
    )
    .width(Length::Fixed(left_panel_width))
    .height(Length::Fixed(list_height));
    container(
        row![left_panel, right_panel]
            .spacing(4)
            .height(Length::Fixed(envelope_height))
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(Length::Fixed(modal_width))
    .into()
}

fn collection_left_panel<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
    content: impl Into<Element<'a, Message>>,
    panel_height: f32,
) -> iced::widget::Container<'a, Message> {
    let left_focused = modal
        .collection_state
        .as_ref()
        .is_some_and(|state| matches!(state.focus, CollectionFocus::Collections));
    let title = modal
        .current_part_label(&app.assigned_values, &app.config.sticky_values)
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
        false,
        crate::app::ModalPaneTarget::Left,
    )
    .height(Length::Fixed(panel_height))
}

fn render_modal_rows<'a>(
    app: &'a App,
    list_items: &mut Vec<Element<'a, Message>>,
    rows: &[String],
    row_colors: Option<&[Color]>,
    cursor: usize,
    range: std::ops::Range<usize>,
    pane_focused: bool,
    hints_active: bool,
    target: crate::app::ModalPaneTarget,
) {
    let modal_hints = app.collection_modal_hint_labels();
    let left_visible = app.collection_modal_left_hint_count();
    let modal_hints: Vec<String> = match target {
        crate::app::ModalPaneTarget::Left => modal_hints.into_iter().take(range.len()).collect(),
        crate::app::ModalPaneTarget::Right => modal_hints.into_iter().skip(left_visible).collect(),
    };

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
        let hint_color = if hints_active {
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
                    .style(move |_theme, status| {
                        modal_item_button_style(&app_theme, status, None, None)
                    }),
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
    border_color: Color,
) -> iced::widget::Container<'a, Message> {
    let panel_background = blend_color(
        app.ui_theme.modal_inactive_background,
        app.ui_theme.modal_panel_background,
        0.1,
    );
    let text_border_color = border_color;
    let text_color = blend_color(app.ui_theme.modal_text, app.ui_theme.modal_muted_text, 0.55);
    container(container(content).height(Length::Fill).padding(6))
        .height(Length::Fill)
        .style(move |_| {
            background_style(panel_background, text_color).border(Border {
                color: text_border_color,
                width: 1.0,
                radius: 4.0.into(),
            })
        })
}

fn inactive_preview_border_color(app: &App) -> Color {
    blend_color(
        app.ui_theme.modal_input_border,
        app.ui_theme.modal_muted_text,
        0.35,
    )
}

fn collection_preview_hint_offset(
    collection_count: usize,
    collection_idx: usize,
    hint_pool: usize,
) -> usize {
    let range = modal_row_window(collection_idx, collection_count, hint_pool);
    range.end.saturating_sub(range.start)
}

fn collection_preview_row_elements<'a>(
    app: &'a App,
    rows: &[String],
    cursor: Option<usize>,
    range: std::ops::Range<usize>,
    pane_focused: bool,
    hints_active: bool,
    interactive: bool,
    hint_offset: usize,
) -> Vec<Element<'a, Message>> {
    let range_start = range.start;
    let modal_hints: Vec<String> = app
        .collection_modal_hint_labels()
        .into_iter()
        .skip(hint_offset)
        .take(range.end.saturating_sub(range.start))
        .collect();
    let mut items = Vec::new();

    for row_idx in range {
        let Some(label) = rows.get(row_idx).cloned() else {
            continue;
        };
        let is_current = cursor == Some(row_idx);
        let marker = if is_current { ">" } else { " " };
        let hint = modal_hints
            .get(row_idx.saturating_sub(range_start))
            .map(|hint| display_hint_label(app, hint))
            .unwrap_or_default();
        let hint_color = if hints_active {
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
        let label_color = if is_current {
            if pane_focused {
                app.ui_theme.active
            } else {
                app.ui_theme.modal_text
            }
        } else {
            app.ui_theme.modal_muted_text
        };
        let row = row![
            container(
                text(marker)
                    .font(app.ui_theme.font_modal)
                    .color(marker_color)
            )
            .align_left(Length::Fixed(14.0)),
            container(
                text(format!("{hint:<4}"))
                    .font(app.ui_theme.font_modal)
                    .color(hint_color)
            )
            .align_left(Length::Fixed(24.0)),
            text(label).font(app.ui_theme.font_modal).color(label_color),
        ]
        .spacing(0);

        if interactive {
            let app_theme = app.ui_theme.clone();
            items.push(
                mouse_area(
                    button(row)
                        .width(Length::Fill)
                        .padding([0.0, 0.0])
                        .on_press(Message::ModalRowPressed(
                            crate::app::ModalPaneTarget::Right,
                            row_idx,
                        ))
                        .style(move |_theme, status| {
                            modal_item_button_style(&app_theme, status, None, None)
                        }),
                )
                .on_enter(Message::ModalRowHovered(
                    crate::app::ModalPaneTarget::Right,
                    row_idx,
                ))
                .interaction(iced::mouse::Interaction::Pointer)
                .into(),
            );
        } else {
            items.push(row.into());
        }
    }

    items
}

fn collection_preview_card_content<'a>(
    app: &'a App,
    snapshot: &crate::modal::CollectionPreviewSnapshot,
    interactive: bool,
    pane_focused: bool,
    hints_active: bool,
    available_height: f32,
    hint_offset: usize,
) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = Vec::with_capacity(snapshot.rows.len() + 1);
    items.push(
        text(snapshot.title.clone())
            .font(app.ui_theme.font_modal)
            .color(if interactive {
                app.ui_theme.modal_hint_text
            } else {
                app.ui_theme.modal_muted_text
            })
            .into(),
    );

    let window_size = collection_preview_visible_row_capacity(available_height);
    let range = if interactive {
        let cursor = snapshot.item_cursor.unwrap_or(0);
        modal_row_window(cursor, snapshot.rows.len(), window_size)
    } else {
        0..snapshot.rows.len().min(window_size)
    };
    items.extend(collection_preview_row_elements(
        app,
        &snapshot.rows,
        snapshot.item_cursor,
        range,
        pane_focused,
        hints_active,
        interactive,
        hint_offset,
    ));

    container(column(items).spacing(2))
        .height(Length::Fill)
        .into()
}

fn collection_preview_card<'a>(
    app: &'a App,
    snapshot: &crate::modal::CollectionPreviewSnapshot,
    width: f32,
    height: f32,
    focused: bool,
    pane_focused: bool,
    hints_active: bool,
    hint_offset: usize,
) -> Element<'a, Message> {
    let interactive = focused && pane_focused;
    let content = collection_preview_card_content(
        app,
        snapshot,
        interactive,
        pane_focused,
        hints_active,
        height,
        hint_offset,
    );
    let panel: Element<'a, Message> = if focused {
        modal_subpanel(
            app,
            content,
            pane_focused,
            focused && !pane_focused,
            crate::app::ModalPaneTarget::Right,
        )
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
    } else {
        preview_modal_subpanel(
            app,
            content,
            if focused || pane_focused {
                app.ui_theme.selected
            } else {
                inactive_preview_border_color(app)
            },
        )
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
    };
    panel
}

fn collection_modal_preview<'a>(
    app: &'a App,
    modal: &'a crate::modal::SearchModal,
) -> Element<'a, Message> {
    let Some(state) = modal.collection_state.as_ref() else {
        return Space::with_height(Length::Shrink).into();
    };
    let pane_focused = matches!(state.focus, CollectionFocus::Items(_));
    let Some(strip) = modal.collection_preview_strip() else {
        return Space::with_height(Length::Shrink).into();
    };
    let viewport_height = collection_preview_viewport_height(app, modal);
    let list_height = collection_list_panel_height(app, modal);
    let anchor_top = ((viewport_height - list_height) * 0.5).round();
    let anchor_bottom = (anchor_top + list_height).round();
    let layout =
        collection_preview_strip_layout(&strip, viewport_height, anchor_top, anchor_bottom);
    let strip_width = modal
        .collection_state
        .as_ref()
        .map(|state| collection_preview_metrics(&state.collections).0)
        .unwrap_or("No collection selected".chars().count()) as f32
        * 7.6
        + 72.0;
    let hint_pool = app.data.keybindings.hints.len();
    let collection_count = state.collections.len();

    let mut cards: Vec<Element<'a, Message>> = Vec::new();
    for (idx, snapshot) in strip.previews.iter().enumerate() {
        let focused = idx == strip.focused_index;
        let pane_has_focus = focused && pane_focused;
        let hints_active = focused;
        let hint_offset = collection_preview_hint_offset(collection_count, idx, hint_pool);
        let height = layout
            .card_heights
            .get(idx)
            .copied()
            .unwrap_or_else(|| collection_preview_card_height(snapshot));
        cards.push(collection_preview_card(
            app,
            snapshot,
            strip_width,
            height,
            focused,
            pane_has_focus,
            hints_active,
            hint_offset,
        ));
    }

    container(modal_unit::ClipTranslate::new_2d(
        0.0,
        layout.y_offset,
        strip_width,
        viewport_height,
        column(cards)
            .spacing(COLLECTION_STREAM_SPACING)
            .width(Length::Fixed(strip_width))
            .height(Length::Shrink),
    ))
    .width(Length::FillPortion(5))
    .height(Length::Fixed(viewport_height))
    .center_y(Length::Fill)
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

fn collection_preview_card_height(snapshot: &crate::modal::CollectionPreviewSnapshot) -> f32 {
    COLLECTION_PREVIEW_CARD_PADDING * 2.0
        + COLLECTION_PREVIEW_TITLE_HEIGHT
        + snapshot.rows.len().max(1) as f32 * COLLECTION_MODAL_ROW_HEIGHT
}

fn collection_preview_visible_row_capacity(height: f32) -> usize {
    ((height - COLLECTION_PREVIEW_CARD_PADDING * 2.0 - COLLECTION_PREVIEW_TITLE_HEIGHT)
        / COLLECTION_MODAL_ROW_HEIGHT)
        .floor()
        .max(1.0) as usize
}

fn collection_list_panel_height(app: &App, modal: &crate::modal::SearchModal) -> f32 {
    let collection_rows = modal
        .collection_state
        .as_ref()
        .map(|state| state.collections.len())
        .unwrap_or(0)
        .max(1) as f32;
    let desired_height =
        COLLECTION_MODAL_CHROME_HEIGHT + collection_rows * COLLECTION_MODAL_ROW_HEIGHT;
    desired_height
        .max(COLLECTION_MODAL_MIN_HEIGHT)
        .min(collection_modal_height(app, modal, 320.0))
}

fn collection_modal_envelope_height(app: &App) -> f32 {
    app.viewport_size
        .map(|size| size.height.max(COLLECTION_MODAL_MIN_HEIGHT))
        .unwrap_or(560.0)
}

fn collection_preview_viewport_height(app: &App, modal: &crate::modal::SearchModal) -> f32 {
    let _ = modal;
    collection_modal_envelope_height(app)
}

fn collection_left_panel_width(modal: &crate::modal::SearchModal) -> f32 {
    modal_size_for_labels(&modal.all_entries, true, false)
        .dimensions()
        .0
}

fn collection_modal_height(
    app: &App,
    _modal: &crate::modal::SearchModal,
    fallback_height: f32,
) -> f32 {
    let strict_bound = app
        .viewport_size
        .map(|size| size.height - 2.0 * (app.ui_theme.modal_stub_width + app.modal_spacer_width()))
        .unwrap_or(fallback_height);
    let soft_bound = app
        .viewport_size
        .map(|size| size.height * crate::modal_layout::MODAL_HEIGHT_RATIO)
        .unwrap_or(fallback_height);
    let viewport_ceiling = app
        .viewport_size
        .map(|size| (size.height - 24.0).max(COLLECTION_MODAL_MIN_HEIGHT))
        .unwrap_or(fallback_height);
    let available_height = strict_bound
        .max(soft_bound)
        .min(viewport_ceiling)
        .max(COLLECTION_MODAL_MIN_HEIGHT);
    available_height.max(COLLECTION_MODAL_MIN_HEIGHT)
}

fn collection_preview_strip_layout(
    strip: &crate::modal::CollectionPreviewStrip,
    viewport_height: f32,
    anchor_top: f32,
    anchor_bottom: f32,
) -> CollectionPreviewStripLayout {
    let mut card_tops = Vec::with_capacity(strip.previews.len());
    let mut card_heights = Vec::with_capacity(strip.previews.len());
    let mut current_top = 0.0;
    for (idx, snapshot) in strip.previews.iter().enumerate() {
        let natural_height = collection_preview_card_height(snapshot);
        let effective_height = if idx == strip.focused_index {
            natural_height.min(viewport_height)
        } else {
            natural_height
        };
        card_tops.push(current_top);
        card_heights.push(effective_height);
        current_top += effective_height + COLLECTION_STREAM_SPACING;
    }
    let total_height = if strip.previews.is_empty() {
        viewport_height
    } else {
        current_top - COLLECTION_STREAM_SPACING
    };
    let focused_top = card_tops.get(strip.focused_index).copied().unwrap_or(0.0);
    let focused_height = card_heights
        .get(strip.focused_index)
        .copied()
        .unwrap_or(COLLECTION_MODAL_ROW_HEIGHT);
    let focused_bottom = focused_top + focused_height;
    let mut desired_y = viewport_height * 0.5 - (focused_top + focused_height * 0.5);

    if strip.focused_index == 0 {
        let top_aligned_y = anchor_top - focused_top;
        if focused_top + desired_y > anchor_top {
            desired_y = top_aligned_y;
        }
    }
    if strip.focused_index + 1 == strip.previews.len() {
        let bottom_aligned_y = anchor_bottom - focused_bottom;
        if focused_bottom + desired_y < anchor_bottom {
            desired_y = bottom_aligned_y;
        }
    }

    CollectionPreviewStripLayout {
        card_tops,
        card_heights,
        total_height,
        y_offset: desired_y,
    }
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
        let left_width = (left_chars * 8.0 + 112.0).clamp(360.0, 560.0);
        let right_width = (right_chars * 7.8 + 72.0).clamp(340.0, 620.0);
        let width = (left_width + right_width + 34.0).min(ModalSize::ExtraWide.dimensions().0);
        let capped = app
            .viewport_size
            .map(|size| width.min((size.width - 32.0).max(780.0)))
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
    landing: bool,
    target: crate::app::ModalPaneTarget,
) -> iced::widget::Container<'a, Message> {
    let panel_background = if focused {
        app.ui_theme.modal_active_background
    } else {
        app.ui_theme.modal_inactive_background
    };
    let border_color = if focused {
        app.ui_theme.active
    } else if landing {
        collection_preview_landing_color()
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
    if app.error_modal.is_some() {
        return error_modal_view(app);
    }
    if should_render_modal_overlay(app) {
        modal_overlay(app, app.modal.as_ref())
    } else {
        main_layout(app)
    }
}

fn error_modal_view(app: &App) -> Element<'_, Message> {
    let report = app
        .error_modal
        .as_ref()
        .expect("error modal view requires a report");
    let rendered = app.messages.render(report);
    let reload_key = first_key_label(&app.data.keybindings.data_reload, "\\");
    let copy_key = first_key_label(&app.data.keybindings.copy_note, "c");
    let footer = format!(
        "Press Esc, Backspace, or {reload_key} to reload. Press {copy_key} to copy error text."
    );
    let source_block = error_modal_source_block(app, &rendered);
    let footer_block = column![
        muted_rule(app),
        text(footer)
            .font(app.ui_theme.font_status)
            .color(app.ui_theme.muted),
        text(format!("Error ID: {}", rendered.id))
            .font(app.ui_theme.font_status)
            .color(app.ui_theme.muted),
        error_modal_full_paths(app, &rendered),
    ]
    .spacing(4);
    let description_spans = rendered.description_segments.as_slice();
    let description_spans = error_modal_styled_spans(
        description_spans,
        app.ui_theme.text,
        app.ui_theme.active,
        app.ui_theme.error,
        app.ui_theme.selected,
        app.ui_theme.muted,
        app.ui_theme.modal_input_background,
        app.ui_theme.modal_input_border,
        app.ui_theme.font_modal,
    );
    let description = rich_text::<Message, iced::Theme, iced::Renderer>(description_spans)
        .font(app.ui_theme.font_modal)
        .size(16)
        .width(Length::Fill);

    let fix_block = error_modal_fix_block(app, &rendered);

    let content = column![
        text(rendered.title.clone())
            .font(app.ui_theme.font_heading)
            .size(24)
            .color(app.ui_theme.error),
        description,
        source_block,
        fix_block,
        footer_block,
    ]
    .spacing(18)
    .padding(24)
    .width(Length::Fill);

    let panel_background = error_modal_panel_background(app);
    let panel = container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .style(move |_| {
            background_style(panel_background, app.ui_theme.text).border(Border {
                color: app.ui_theme.error,
                width: 2.0,
                radius: 8.0.into(),
            })
        });

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(18)
        .style(move |_| background_style(app.ui_theme.background, app.ui_theme.text))
        .into()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ErrorModalInlineStyle {
    Normal,
    Red,
    Green,
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErrorModalFixSnippet {
    line_label: Option<String>,
    code: String,
    accent: ErrorModalFixSnippetAccent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorModalFixSnippetAccent {
    Context,
    Suggested,
}

fn error_modal_styled_spans(
    segments: &[crate::diagnostics::RenderedTextSegment],
    base_color: Color,
    param_color: Color,
    red_color: Color,
    green_color: Color,
    muted_color: Color,
    code_background: Color,
    code_border: Color,
    base_font: iced::Font,
) -> Vec<iced::widget::text::Span<'static, Message>> {
    let mut spans = Vec::new();
    let mut buffer = String::new();
    let mut current_is_param: Option<bool> = None;
    let mut style = ErrorModalInlineStyle::Normal;
    let mut paren_depth = 0usize;

    let flush = |spans: &mut Vec<iced::widget::text::Span<'static, Message>>,
                 buffer: &mut String,
                 current_is_param: Option<bool>,
                 style: ErrorModalInlineStyle,
                 paren_depth: usize| {
        if buffer.is_empty() {
            return;
        }
        let color = match style {
            _ if paren_depth > 0 => muted_color,
            ErrorModalInlineStyle::Red => red_color,
            ErrorModalInlineStyle::Green => green_color,
            ErrorModalInlineStyle::Code => {
                if current_is_param.unwrap_or(false) {
                    param_color
                } else {
                    base_color
                }
            }
            ErrorModalInlineStyle::Normal => {
                if current_is_param.unwrap_or(false) {
                    param_color
                } else {
                    base_color
                }
            }
        };
        let mut text_span = span(std::mem::take(buffer)).color(color).font(base_font);
        if style == ErrorModalInlineStyle::Code {
            text_span = text_span
                .background(code_background)
                .border(Border {
                    color: code_border,
                    width: 1.0,
                    radius: 4.0.into(),
                })
                .padding([1.0, 5.0]);
        }
        spans.push(text_span);
    };

    for segment in segments {
        let mut chars = segment.text.chars().peekable();
        while let Some(ch) = chars.next() {
            if style == ErrorModalInlineStyle::Code {
                if ch == '`' {
                    flush(
                        &mut spans,
                        &mut buffer,
                        current_is_param,
                        style,
                        paren_depth,
                    );
                    current_is_param = None;
                    style = ErrorModalInlineStyle::Normal;
                } else {
                    if current_is_param != Some(segment.is_param) {
                        flush(
                            &mut spans,
                            &mut buffer,
                            current_is_param,
                            style,
                            paren_depth,
                        );
                        current_is_param = Some(segment.is_param);
                    }
                    buffer.push(ch);
                }
                continue;
            }

            if ch == '*' {
                flush(
                    &mut spans,
                    &mut buffer,
                    current_is_param,
                    style,
                    paren_depth,
                );
                current_is_param = None;
                if chars.peek() == Some(&'*') {
                    chars.next();
                    style = if style == ErrorModalInlineStyle::Green {
                        ErrorModalInlineStyle::Normal
                    } else {
                        ErrorModalInlineStyle::Green
                    };
                } else {
                    style = if style == ErrorModalInlineStyle::Red {
                        ErrorModalInlineStyle::Normal
                    } else {
                        ErrorModalInlineStyle::Red
                    };
                }
                continue;
            }

            if ch == '`' {
                flush(
                    &mut spans,
                    &mut buffer,
                    current_is_param,
                    style,
                    paren_depth,
                );
                current_is_param = None;
                style = ErrorModalInlineStyle::Code;
                continue;
            }

            if ch == '(' {
                flush(
                    &mut spans,
                    &mut buffer,
                    current_is_param,
                    style,
                    paren_depth,
                );
                if current_is_param != Some(segment.is_param) {
                    current_is_param = Some(segment.is_param);
                }
                paren_depth += 1;
                buffer.push(ch);
                continue;
            }

            if ch == ')' {
                if current_is_param != Some(segment.is_param) {
                    flush(
                        &mut spans,
                        &mut buffer,
                        current_is_param,
                        style,
                        paren_depth,
                    );
                    current_is_param = Some(segment.is_param);
                }
                buffer.push(ch);
                flush(
                    &mut spans,
                    &mut buffer,
                    current_is_param,
                    style,
                    paren_depth,
                );
                paren_depth = paren_depth.saturating_sub(1);
                current_is_param = None;
                continue;
            }

            if current_is_param != Some(segment.is_param) {
                flush(
                    &mut spans,
                    &mut buffer,
                    current_is_param,
                    style,
                    paren_depth,
                );
                current_is_param = Some(segment.is_param);
            }
            buffer.push(ch);
        }
    }

    flush(
        &mut spans,
        &mut buffer,
        current_is_param,
        style,
        paren_depth,
    );
    spans
}

fn error_modal_panel_background(app: &App) -> Color {
    let base = app.ui_theme.modal_panel_background;
    let Some((kind, amount)) = app.error_modal_flash_amount() else {
        return base;
    };
    let flash = match kind {
        ErrorModalFlashKind::Error => app.ui_theme.error,
        ErrorModalFlashKind::Copy => app.ui_theme.preview_copy_flash_background,
    };
    blend_color(base, flash, amount * 0.65)
}

fn error_modal_fix_block<'a>(
    app: &'a App,
    rendered: &crate::diagnostics::RenderedError,
) -> Element<'a, Message> {
    if rendered.fix.trim().is_empty() {
        return Space::with_height(Length::Fixed(0.0)).into();
    }

    let fix_lines = split_rendered_text_lines(&rendered.fix_segments);
    let mut items: Vec<Element<'a, Message>> = vec![
        muted_rule(app),
        text("Next Steps")
            .font(app.ui_theme.font_heading)
            .size(18)
            .color(app.ui_theme.selected)
            .into(),
    ];

    for line in fix_lines {
        if line.is_empty() {
            items.push(Space::with_height(Length::Fixed(6.0)).into());
            continue;
        }

        let line_text = flatten_rendered_text_segments(&line);
        let trimmed = line_text.trim();
        if trimmed.is_empty() {
            items.push(Space::with_height(Length::Fixed(6.0)).into());
            continue;
        }

        if trimmed == "..." {
            items.push(
                text("...")
                    .font(app.ui_theme.font_modal)
                    .color(app.ui_theme.muted)
                    .into(),
            );
            continue;
        }

        if let Some(snippet) = parse_error_modal_fix_snippet(&line_text) {
            items.push(error_modal_fix_snippet_row(app, &snippet));
            continue;
        }

        let spans = error_modal_styled_spans(
            &line,
            app.ui_theme.text,
            app.ui_theme.active,
            app.ui_theme.error,
            app.ui_theme.selected,
            app.ui_theme.muted,
            app.ui_theme.modal_input_background,
            app.ui_theme.modal_input_border,
            app.ui_theme.font_modal,
        );
        items.push(
            rich_text::<Message, iced::Theme, iced::Renderer>(spans)
                .font(app.ui_theme.font_modal)
                .size(16)
                .width(Length::Fill)
                .into(),
        );
    }

    column(items).spacing(8).into()
}

fn error_modal_fix_snippet_row<'a>(
    app: &'a App,
    snippet: &ErrorModalFixSnippet,
) -> Element<'a, Message> {
    let line_color = match snippet.accent {
        ErrorModalFixSnippetAccent::Context => app.ui_theme.active,
        ErrorModalFixSnippetAccent::Suggested => app.ui_theme.selected,
    };
    let label = snippet.line_label.clone().unwrap_or_default();

    row![
        text(label)
            .font(app.ui_theme.font_modal)
            .color(line_color)
            .width(Length::Fixed(44.0)),
        container(
            text(snippet.code.clone())
                .font(app.ui_theme.font_modal)
                .color(line_color),
        )
        .padding([3.0, 8.0])
        .width(Length::Fill)
        .style(move |_| background_style(app.ui_theme.modal_input_background, line_color)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn split_rendered_text_lines(
    segments: &[crate::diagnostics::RenderedTextSegment],
) -> Vec<Vec<crate::diagnostics::RenderedTextSegment>> {
    let mut lines = vec![Vec::new()];

    for segment in segments {
        let mut line_start = 0usize;
        for (idx, ch) in segment.text.char_indices() {
            if ch != '\n' {
                continue;
            }

            if idx > line_start {
                lines
                    .last_mut()
                    .expect("line buffer should exist")
                    .push(crate::diagnostics::RenderedTextSegment {
                        text: segment.text[line_start..idx].to_string(),
                        is_param: segment.is_param,
                    });
            }
            lines.push(Vec::new());
            line_start = idx + ch.len_utf8();
        }

        if line_start < segment.text.len() {
            lines
                .last_mut()
                .expect("line buffer should exist")
                .push(crate::diagnostics::RenderedTextSegment {
                    text: segment.text[line_start..].to_string(),
                    is_param: segment.is_param,
                });
        }
    }

    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }

    lines
}

fn flatten_rendered_text_segments(segments: &[crate::diagnostics::RenderedTextSegment]) -> String {
    segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect()
}

fn parse_error_modal_fix_snippet(line: &str) -> Option<ErrorModalFixSnippet> {
    let trimmed = line.trim_start();

    if let Some(rest) = trimmed.strip_prefix("**ln ") {
        let (line_number, rest) = rest.split_once("**")?;
        if !line_number.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        let code = parse_error_modal_fix_code(rest.trim_start())?;
        return Some(ErrorModalFixSnippet {
            line_label: Some(format!("ln {line_number}")),
            code,
            accent: ErrorModalFixSnippetAccent::Suggested,
        });
    }

    if let Some(rest) = trimmed.strip_prefix("ln ") {
        let (line_number, rest) = rest.split_once(' ')?;
        if !line_number.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        let code = parse_error_modal_fix_code(rest.trim_start())?;
        return Some(ErrorModalFixSnippet {
            line_label: Some(format!("ln {line_number}")),
            code,
            accent: ErrorModalFixSnippetAccent::Context,
        });
    }

    parse_error_modal_fix_code(trimmed).map(|code| ErrorModalFixSnippet {
        line_label: None,
        code,
        accent: ErrorModalFixSnippetAccent::Suggested,
    })
}

fn parse_error_modal_fix_code(text: &str) -> Option<String> {
    let code = text.strip_prefix('`')?.strip_suffix('`')?;
    Some(code.replace("**", ""))
}

fn error_modal_full_paths<'a>(
    app: &'a App,
    rendered: &crate::diagnostics::RenderedError,
) -> Element<'a, Message> {
    if rendered.source_blocks.is_empty() {
        return Space::with_height(Length::Fixed(0.0)).into();
    }

    let items = rendered
        .source_blocks
        .iter()
        .map(|block| {
            text(format!("File: {}", block.file_path))
                .font(app.ui_theme.font_status)
                .color(app.ui_theme.muted)
                .into()
        })
        .collect::<Vec<Element<'a, Message>>>();

    column(items).spacing(4).into()
}

fn error_modal_source_block<'a>(
    app: &'a App,
    rendered: &crate::diagnostics::RenderedError,
) -> Element<'a, Message> {
    if rendered.source_blocks.is_empty() {
        return Space::with_height(Length::Fixed(0.0)).into();
    }

    let mut items: Vec<Element<'_, Message>> = vec![muted_rule(app)];
    let modal_input_background = app.ui_theme.modal_input_background;

    for block in &rendered.source_blocks {
        items.push(
            text(format!("{}:", block.file_name))
                .font(app.ui_theme.font_heading)
                .size(16)
                .color(app.ui_theme.modal)
                .into(),
        );

        let mut previous_line: Option<usize> = None;
        for line in &block.lines {
            if let Some(prev) = previous_line {
                if line.line > prev + 1 {
                    items.push(
                        text("...")
                            .font(app.ui_theme.font_modal)
                            .color(app.ui_theme.muted)
                            .into(),
                    );
                }
            }

            let line_color = match line.role {
                crate::diagnostics::RenderedErrorSourceRole::Owner => app.ui_theme.active,
                crate::diagnostics::RenderedErrorSourceRole::Reference => app.ui_theme.error,
                crate::diagnostics::RenderedErrorSourceRole::Found => app.ui_theme.hint,
            };
            let quoted_line = if line.quoted_line.is_empty() {
                format!("(line {})", line.line)
            } else {
                line.quoted_line.clone()
            };
            items.push(
                row![
                    text(format!("ln {}", line.line))
                        .font(app.ui_theme.font_modal)
                        .color(line_color),
                    container(
                        text(quoted_line)
                            .font(app.ui_theme.font_modal)
                            .color(line_color),
                    )
                    .padding([3.0, 8.0])
                    .width(Length::Fill)
                    .style(move |_| background_style(modal_input_background, line_color)),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into(),
            );
            previous_line = Some(line.line);
        }
    }

    column(items).spacing(8).into()
}

fn muted_rule(app: &App) -> Element<'_, Message> {
    container(Space::with_height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(move |_| background_style(app.ui_theme.muted, app.ui_theme.muted))
        .into()
}

fn first_key_label(keys: &[String], fallback: &str) -> String {
    keys.first()
        .map(|key| match key.as_str() {
            "esc" => "Esc".to_string(),
            "backspace" => "Backspace".to_string(),
            other => other.to_string(),
        })
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        build_connected_transition_rendered_unit, build_modal_close_rendered_unit,
        build_modal_open_rendered_unit, build_rendered_modal_unit, collection_preview_card_height,
        collection_preview_metrics, collection_preview_strip_layout, default_stub_mode,
        entry_composition_panel_width, error_modal_styled_spans, modal_close_shift,
        modal_open_shift, modal_unit_runway_layout, parse_error_modal_fix_snippet,
        retained_close_root_width, retained_modal_close_transition,
        should_render_modal_overlay, simple_modal_unit_root_width,
        split_rendered_text_lines, transition_unit_display_width, ErrorModalFixSnippet,
        ErrorModalFixSnippetAccent, ModalUnitCardKind, ModalUnitSide, COLLECTION_STREAM_SPACING,
    };
    use crate::app::{
        App, FocusDirection, ModalArrivalLayer, ModalDepartureLayer, ModalTransitionEasing,
        ModalTransitionLayer, UnitContentSnapshot, UnitGeometry,
    };
    use crate::config::Config;
    use crate::data::{
        AppData, GroupNoteMeta, HeaderFieldConfig, HierarchyItem, HierarchyList, KeyBindings,
        ModalStart, ResolvedCollectionConfig, RuntimeGroup, RuntimeNode, RuntimeNodeKind,
        RuntimeTemplate, SectionConfig,
    };
    use crate::modal_layout::{
        ModalFocus, ModalListViewSnapshot, ModalStubKind, ModalUnitRange, SimpleModalSequence,
        SimpleModalUnitLayout,
    };
    use crate::sections::collection::CollectionEntry;
    use iced::Color;
    use std::time::Instant;
    use std::{collections::HashMap, path::PathBuf};

    fn item(id: &str, label: &str) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: false,
            output: Some(label.to_string()),
            fields: None,
            branch_fields: Vec::new(),
            assigns: Vec::new(),
        }
    }

    #[test]
    fn error_modal_code_spans_do_not_leak_style_after_closing_backtick() {
        let base = Color::from_rgb(0.1, 0.1, 0.1);
        let param = Color::from_rgb(0.9, 0.8, 0.1);
        let red = Color::from_rgb(0.9, 0.1, 0.1);
        let green = Color::from_rgb(0.1, 0.9, 0.1);
        let muted = Color::from_rgb(0.5, 0.5, 0.5);
        let segments = vec![crate::diagnostics::RenderedTextSegment {
            text: "**ln 8** `  - **collection**: back_all_prone_collection`\n\nb) Update the ID:"
                .to_string(),
            is_param: false,
        }];

        let spans = error_modal_styled_spans(
            &segments,
            base,
            param,
            red,
            green,
            muted,
            Color::from_rgb(0.08, 0.08, 0.08),
            Color::from_rgb(0.4, 0.4, 0.4),
            iced::Font::MONOSPACE,
        );

        assert!(spans.iter().any(|span| {
            span.text.as_ref() == "  - **collection**: back_all_prone_collection"
                && span.color == Some(base)
                && span.highlight.is_some()
        }));
    }

    #[test]
    fn split_rendered_text_lines_preserves_segment_flags_across_newlines() {
        let lines = split_rendered_text_lines(&[
            crate::diagnostics::RenderedTextSegment {
                text: "alpha\nbeta".to_string(),
                is_param: false,
            },
            crate::diagnostics::RenderedTextSegment {
                text: " gamma".to_string(),
                is_param: true,
            },
        ]);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 1);
        assert_eq!(lines[0][0].text, "alpha");
        assert!(!lines[0][0].is_param);
        assert_eq!(lines[1].len(), 2);
        assert_eq!(lines[1][0].text, "beta");
        assert!(!lines[1][0].is_param);
        assert_eq!(lines[1][1].text, " gamma");
        assert!(lines[1][1].is_param);
    }

    #[test]
    fn parse_error_modal_fix_snippet_recognizes_context_line_rows() {
        let snippet = parse_error_modal_fix_snippet("ln 2 `- id: subjective_section`");

        assert_eq!(
            snippet,
            Some(ErrorModalFixSnippet {
                line_label: Some("ln 2".to_string()),
                code: "- id: subjective_section".to_string(),
                accent: ErrorModalFixSnippetAccent::Context,
            })
        );
    }

    #[test]
    fn parse_error_modal_fix_snippet_recognizes_suggested_rows_and_strips_markup() {
        let snippet = parse_error_modal_fix_snippet(
            "**ln 8** `  - **collection**: back_all_prone_collection`",
        );

        assert_eq!(
            snippet,
            Some(ErrorModalFixSnippet {
                line_label: Some("ln 8".to_string()),
                code: "  - collection: back_all_prone_collection".to_string(),
                accent: ErrorModalFixSnippetAccent::Suggested,
            })
        );
    }

    #[test]
    fn parse_error_modal_fix_snippet_recognizes_code_only_rows() {
        let snippet = parse_error_modal_fix_snippet("`  **- id:** demo_field`");

        assert_eq!(
            snippet,
            Some(ErrorModalFixSnippet {
                line_label: None,
                code: "  - id: demo_field".to_string(),
                accent: ErrorModalFixSnippetAccent::Suggested,
            })
        );
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
            revisiting_completed_field: false,
            confirmed_row: None,
            list_cursor: 0,
            list_scroll: 0,
            focus: ModalFocus::List,
        }
    }

    fn branch_modal_for_semantic_boundary() -> crate::modal::SearchModal {
        let child_field = HeaderFieldConfig {
            id: "child_field".to_string(),
            name: "Child".to_string(),
            format: Some("{child_list}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "child_list".to_string(),
                label: Some("Child".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![item("child", "Child Value")],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let mut branch_item = item("branch", "Branch");
        branch_item.output = Some("{child_field}".to_string());
        branch_item.branch_fields = vec![child_field];
        let field = HeaderFieldConfig {
            id: "muscle_field".to_string(),
            name: "Muscle".to_string(),
            format: Some("{muscle}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "muscle".to_string(),
                label: Some("Muscle".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![branch_item],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let mut modal = crate::modal::SearchModal::new_field(
            0,
            field,
            None,
            &HashMap::new(),
            &HashMap::new(),
            5,
        );
        modal.query = "branch".to_string();
        modal.update_filter();
        modal
    }

    fn connected_transition_fixture(
        direction: FocusDirection,
    ) -> (
        SimpleModalUnitLayout,
        ModalArrivalLayer,
        ModalDepartureLayer,
    ) {
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
            modal: None,
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

    fn overlay_test_app() -> App {
        let field = HeaderFieldConfig {
            id: "region".to_string(),
            name: "Region".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "region".to_string(),
                label: Some("Region".to_string()),
                preview: Some("Region".to_string()),
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![item("shoulder", "Shoulder"), item("hip", "Hip")],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let section = SectionConfig {
            id: "request_section".to_string(),
            name: "Request".to_string(),
            map_label: "Request".to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: Some(vec![field.clone()]),
            lists: Vec::new(),
            note_label: None,
            group_id: "intake".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let data = AppData {
            template: RuntimeTemplate {
                id: "test".to_string(),
                children: vec![RuntimeGroup {
                    id: "intake".to_string(),
                    nav_label: "Intake".to_string(),
                    note: GroupNoteMeta::default(),
                    children: vec![RuntimeNode::Section(section.clone())],
                }],
            },
            list_data: HashMap::new(),
            checklist_data: HashMap::new(),
            collection_data: HashMap::new(),
            boilerplate_texts: HashMap::new(),
            keybindings: KeyBindings::default(),
            hotkeys: Default::default(),
        };

        App::new(data, Config::default(), PathBuf::new())
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
    fn collection_preview_strip_layout_prioritizes_focused_height() {
        let strip = crate::modal::CollectionPreviewStrip {
            previews: vec![
                crate::modal::CollectionPreviewSnapshot {
                    title: "Prev".to_string(),
                    rows: vec!["A".to_string()],
                    item_cursor: None,
                },
                crate::modal::CollectionPreviewSnapshot {
                    title: "Current".to_string(),
                    rows: vec![
                        "1".to_string(),
                        "2".to_string(),
                        "3".to_string(),
                        "4".to_string(),
                        "5".to_string(),
                    ],
                    item_cursor: Some(0),
                },
                crate::modal::CollectionPreviewSnapshot {
                    title: "Next".to_string(),
                    rows: vec!["B".to_string()],
                    item_cursor: None,
                },
            ],
            focused_index: 1,
        };

        let focused_natural_height = collection_preview_card_height(&strip.previews[1]);
        let viewport_height = focused_natural_height - 20.0;
        let layout = collection_preview_strip_layout(&strip, viewport_height, 0.0, viewport_height);

        assert_eq!(layout.card_heights[1], focused_natural_height - 20.0);
        assert_eq!(
            layout.total_height,
            layout.card_heights.iter().sum::<f32>() + 2.0 * COLLECTION_STREAM_SPACING
        );
    }

    #[test]
    fn collection_preview_strip_layout_keeps_authored_neighbor_order() {
        let strip = crate::modal::CollectionPreviewStrip {
            previews: vec![
                crate::modal::CollectionPreviewSnapshot {
                    title: "One".to_string(),
                    rows: vec!["A".to_string()],
                    item_cursor: None,
                },
                crate::modal::CollectionPreviewSnapshot {
                    title: "Two".to_string(),
                    rows: vec!["B".to_string()],
                    item_cursor: None,
                },
                crate::modal::CollectionPreviewSnapshot {
                    title: "Three".to_string(),
                    rows: vec!["C".to_string()],
                    item_cursor: None,
                },
                crate::modal::CollectionPreviewSnapshot {
                    title: "Four".to_string(),
                    rows: vec!["D".to_string()],
                    item_cursor: None,
                },
            ],
            focused_index: 2,
        };

        let viewport_height = collection_preview_card_height(&strip.previews[2])
            + 2.0
                * (collection_preview_card_height(&strip.previews[1]) + COLLECTION_STREAM_SPACING);
        let layout = collection_preview_strip_layout(&strip, viewport_height, 0.0, viewport_height);

        assert_eq!(
            layout.card_tops,
            vec![
                0.0,
                collection_preview_card_height(&strip.previews[0]) + COLLECTION_STREAM_SPACING,
                2.0 * (collection_preview_card_height(&strip.previews[0])
                    + COLLECTION_STREAM_SPACING),
                3.0 * (collection_preview_card_height(&strip.previews[0])
                    + COLLECTION_STREAM_SPACING),
            ]
        );
        let focused_center = layout.card_tops[2] + layout.card_heights[2] * 0.5 + layout.y_offset;
        assert_eq!(focused_center, viewport_height * 0.5);
    }

    #[test]
    fn collection_preview_strip_layout_snaps_first_preview_top_to_anchor() {
        let strip = crate::modal::CollectionPreviewStrip {
            previews: vec![
                crate::modal::CollectionPreviewSnapshot {
                    title: "First".to_string(),
                    rows: vec!["A".to_string()],
                    item_cursor: None,
                },
                crate::modal::CollectionPreviewSnapshot {
                    title: "Second".to_string(),
                    rows: vec!["B".to_string()],
                    item_cursor: None,
                },
            ],
            focused_index: 0,
        };

        let viewport_height = 500.0;
        let anchor_top = 80.0;
        let layout = collection_preview_strip_layout(
            &strip,
            viewport_height,
            anchor_top,
            anchor_top + 240.0,
        );

        assert_eq!(layout.card_tops[0] + layout.y_offset, anchor_top);
    }

    #[test]
    fn collection_preview_strip_layout_snaps_last_preview_bottom_to_anchor() {
        let strip = crate::modal::CollectionPreviewStrip {
            previews: vec![
                crate::modal::CollectionPreviewSnapshot {
                    title: "First".to_string(),
                    rows: vec!["A".to_string()],
                    item_cursor: None,
                },
                crate::modal::CollectionPreviewSnapshot {
                    title: "Last".to_string(),
                    rows: vec!["B".to_string()],
                    item_cursor: None,
                },
            ],
            focused_index: 1,
        };

        let viewport_height = 500.0;
        let anchor_bottom = 420.0;
        let layout = collection_preview_strip_layout(
            &strip,
            viewport_height,
            anchor_bottom - 240.0,
            anchor_bottom,
        );

        assert_eq!(
            layout.card_tops[1] + layout.card_heights[1] + layout.y_offset,
            anchor_bottom
        );
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

    #[test]
    fn modal_open_shift_starts_right_and_settles_to_rest() {
        assert_eq!(modal_open_shift(0.0, 240.0), 240.0);
        assert_eq!(modal_open_shift(0.5, 240.0), 120.0);
        assert_eq!(modal_open_shift(1.0, 240.0), 0.0);
    }

    #[test]
    fn modal_close_shift_uses_exit_and_confirm_directions() {
        assert_eq!(
            modal_close_shift(FocusDirection::Backward, 0.5, 240.0),
            120.0
        );
        assert_eq!(
            modal_close_shift(FocusDirection::Forward, 0.5, 240.0),
            -120.0
        );
    }

    #[test]
    fn modal_open_rendered_unit_fades_owned_stubs_with_the_unit() {
        let (layout, arrival, _) = connected_transition_fixture(FocusDirection::Forward);
        let rendered = build_modal_open_rendered_unit(120.0, &layout, &arrival, 0.4);

        let stub_alphas = rendered
            .cards
            .iter()
            .filter_map(|card| match card.kind {
                ModalUnitCardKind::Stub { .. } => Some(card.alpha),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(stub_alphas, vec![0.4, 0.4]);
        assert!(
            rendered
                .cards
                .iter()
                .any(|card| matches!(card.kind, ModalUnitCardKind::Active { .. })
                    && card.alpha == 0.4)
        );
    }

    #[test]
    fn modal_close_rendered_unit_fades_owned_stubs_with_the_unit() {
        let (_, _, departure) = connected_transition_fixture(FocusDirection::Forward);
        let rendered = build_modal_close_rendered_unit(120.0, &departure, 0.3);

        let stub_alphas = rendered
            .cards
            .iter()
            .filter_map(|card| match card.kind {
                ModalUnitCardKind::Stub { .. } => Some(card.alpha),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(stub_alphas, vec![0.3, 0.3]);
        assert!(rendered.cards.iter().all(|card| {
            !matches!(card.kind, ModalUnitCardKind::Active { .. }) || card.alpha == 0.3
        }));
    }

    #[test]
    fn rendered_unit_uses_modal_semantics_at_terminal_preview_boundary() {
        let app = overlay_test_app();
        let modal = branch_modal_for_semantic_boundary();
        let layout = SimpleModalUnitLayout {
            sequence: SimpleModalSequence {
                snapshots: vec![snapshot("Muscle")],
                active_sequence_index: 0,
            },
            units: vec![ModalUnitRange {
                start: 0,
                end: 0,
                shows_stubs: true,
            }],
        };

        let rendered = build_rendered_modal_unit(
            &app,
            &modal,
            &layout,
            &layout.units[0],
            default_stub_mode(ModalUnitSide::Left),
            default_stub_mode(ModalUnitSide::Right),
        );
        let trailing_stub = rendered
            .cards
            .iter()
            .rev()
            .find_map(|card| match card.kind {
                ModalUnitCardKind::Stub {
                    side: ModalUnitSide::Right,
                    stub_kind,
                    ..
                } => Some(stub_kind),
                _ => None,
            });

        assert_eq!(trailing_stub, Some(ModalStubKind::NavRight));
    }

    #[test]
    fn unit_geometry_uses_modal_semantics_at_terminal_preview_boundary() {
        let modal = branch_modal_for_semantic_boundary();
        let layout = SimpleModalUnitLayout {
            sequence: SimpleModalSequence {
                snapshots: vec![snapshot("Muscle")],
                active_sequence_index: 0,
            },
            units: vec![ModalUnitRange {
                start: 0,
                end: 0,
                shows_stubs: true,
            }],
        };

        let geometry =
            UnitGeometry::from_layout(&layout, 0, &modal, &HashMap::new(), &HashMap::new(), 24.0)
                .expect("geometry should build");

        assert_eq!(geometry.leading_stub_kind, Some(ModalStubKind::Exit));
        assert_eq!(geometry.trailing_stub_kind, Some(ModalStubKind::NavRight));
    }

    #[test]
    fn retained_close_transition_keeps_overlay_visible_after_modal_clears() {
        let mut app = overlay_test_app();

        app.handle_key(crate::app::AppKey::Space);
        app.dismiss_modal();

        assert!(app.modal.is_none());
        assert!(matches!(
            app.modal_transitions.last(),
            Some(ModalTransitionLayer::ModalClose { .. })
        ));
        assert!(should_render_modal_overlay(&app));
    }

    #[test]
    fn retained_close_root_width_matches_live_active_modal_width() {
        let mut app = overlay_test_app();
        app.handle_key(crate::app::AppKey::Space);

        let live_width = app
            .modal_unit_layout
            .as_ref()
            .and_then(simple_modal_unit_root_width)
            .expect("simple modal layout should provide active root width");

        app.dismiss_modal();

        let (departure, _) =
            retained_modal_close_transition(&app).expect("close transition should be retained");
        assert_eq!(retained_close_root_width(departure), Some(live_width));
    }

    #[test]
    fn entry_composition_panel_width_handles_modal_width_larger_than_soft_cap() {
        let mut app = overlay_test_app();
        app.viewport_size = Some(iced::Size::new(1391.0, 900.0));

        let panel_width = entry_composition_panel_width(&app, 1252.0);

        assert_eq!(panel_width, 1252.0);
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

        assert_eq!(
            slide_fwd, slide_bwd,
            "forward and backward slide distances must match"
        );
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
                ModalUnitCardKind::Stub {
                    stub_kind: ModalStubKind::NavRight,
                    ..
                }
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
                ModalUnitCardKind::Stub {
                    stub_kind: ModalStubKind::NavLeft,
                    ..
                }
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
