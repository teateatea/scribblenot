use crate::app::{App, Focus, MapHintLevel, SectionState};
use crate::data::HeaderFieldConfig;
use crate::modal::{ModalFocus, SearchModal};
use crate::note::{render_note, NoteRenderMode};
use crate::sections::list_select::ListSelectMode;
use crate::theme;
use std::collections::HashMap;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Main layout: content area + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(size);

    let content_area = main_chunks[0];
    let status_area = main_chunks[1];

    // Three-column layout: map always on outside, wizard in middle, preview on other side
    // Default: [Map | Wizard | Preview]
    // Swapped: [Preview | Wizard | Map]
    let map_width = 26u16;
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if app.pane_swapped {
            vec![
                Constraint::Percentage(50),
                Constraint::Min(20),
                Constraint::Length(map_width),
            ]
        } else {
            vec![
                Constraint::Length(map_width),
                Constraint::Min(20),
                Constraint::Percentage(50),
            ]
        })
        .split(content_area);

    let (map_pane, wizard_pane, preview_pane) = if app.pane_swapped {
        (panes[2], panes[1], panes[0])
    } else {
        (panes[0], panes[1], panes[2])
    };

    render_section_map(f, app, map_pane);
    render_wizard_widget(f, app, wizard_pane);
    render_note_pane(f, app, preview_pane);
    render_status_bar(f, app, status_area);
    render_search_modal(f, app, wizard_pane);

    if app.show_help {
        render_help_overlay(f, app, size);
    }
}

/// Returns the styled span(s) for a hint label given the current hint_buffer.
/// `active_color` should be the color that would be used when the hint is active (HINT).
/// When `hint_buffer` is empty, returns a single span with `active_color` (no change).
/// When `hint_buffer` is non-empty and hint starts with buffer: prefix in White/Bold + remainder in active_color.
/// When `hint_buffer` is non-empty and hint does NOT start with buffer: single span in MUTED.
fn hint_spans(hint: &str, hint_buffer: &str, active_color: Color) -> Vec<Span<'static>> {
    if hint_buffer.is_empty() {
        return vec![Span::styled(hint.to_string(), Style::default().fg(active_color))];
    }
    let hint_lower = hint.to_lowercase();
    let buf_lower = hint_buffer.to_lowercase();
    if hint_lower.starts_with(&buf_lower) {
        let prefix = hint[..hint_buffer.len()].to_string();
        let remainder = hint[hint_buffer.len()..].to_string();
        let prefix_span = Span::styled(
            prefix,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        );
        if remainder.is_empty() {
            vec![prefix_span]
        } else {
            vec![
                prefix_span,
                Span::styled(remainder, Style::default().fg(active_color)),
            ]
        }
    } else {
        vec![Span::styled(hint.to_string(), Style::default().fg(theme::MUTED))]
    }
}

fn render_section_map(f: &mut Frame, app: &App, area: Rect) {
    let hints = crate::data::combined_hints(&app.data.keybindings);
    let capitalized = app.config.hint_labels_capitalized;
    let map_focused = app.focus == Focus::Map;

    let mut items: Vec<ListItem> = Vec::new();
    let mut cursor_item_idx: Option<usize> = None;
    let mut flat_idx = 0usize;

    for (g_idx, group) in app.data.groups.iter().enumerate() {
        let group_hint_raw = hints.get(g_idx).copied().unwrap_or(" ");
        let group_hint_display = if capitalized { group_hint_raw.to_uppercase() } else { group_hint_raw.to_string() };

        let current_group = app.group_idx_for_section(app.current_idx);
        let group_hint_color = if app.modal.is_some() {
            theme::MUTED
        } else if !map_focused {
            // Wizard mode: show group hint as active for the current section's group
            if g_idx == current_group { theme::HINT } else { theme::MUTED }
        } else {
            // Group hints are always available when map is focused (universal group-jump).
            theme::HINT
        };

        let group_name_style = if !map_focused {
            if g_idx == current_group { theme::displaced_bold() } else { theme::muted_bold() }
        } else {
            theme::bold()
        };

        let group_hint_spans: Vec<Span> = if group_hint_color == theme::HINT && !app.hint_buffer.is_empty() {
            let mut spans = hint_spans(&group_hint_display, &app.hint_buffer, theme::HINT);
            // Append trailing space as part of last span's text
            if let Some(last) = spans.last_mut() {
                let s = last.content.to_string() + " ";
                *last = Span::styled(s, last.style);
            }
            spans
        } else {
            vec![Span::styled(
                format!("{} ", group_hint_display),
                Style::default().fg(group_hint_color),
            )]
        };
        let mut group_line_spans = group_hint_spans;
        group_line_spans.push(Span::styled(group.name.clone(), group_name_style));
        items.push(ListItem::new(Line::from(group_line_spans)));

        // Section hints: all start at n_groups offset (no per-group exclusion needed)
        let n_groups = app.data.groups.len();
        let group_start: usize = app.data.groups.iter().take(g_idx).map(|g| g.sections.len()).sum();

        for (si, section) in group.sections.iter().enumerate() {
            let is_current = flat_idx == app.current_idx;
            let is_map_cursor = map_focused && flat_idx == app.map_cursor;
            let _is_completed = app.section_is_completed(flat_idx);
            let is_skipped = app.section_is_skipped(flat_idx);

            let section_hint_raw = hints.get(n_groups + group_start + si).copied().unwrap_or(" ");
            let section_hint_display = if capitalized { section_hint_raw.to_uppercase() } else { section_hint_raw.to_string() };

            let section_hint_color = if app.modal.is_some() {
                theme::MUTED
            } else if !map_focused {
                if is_current { theme::HINT } else { theme::MUTED }
            } else {
                match &app.map_hint_level {
                    MapHintLevel::Groups => theme::MUTED,
                    MapHintLevel::Sections(active_g) => {
                        if *active_g == g_idx { theme::HINT } else { theme::MUTED }
                    }
                }
            };

            let cursor_char = if is_map_cursor { ">" } else if is_skipped { "-" } else { " " };

            let entry_style = if is_map_cursor {
                theme::active_bold()
            } else if is_current && !map_focused {
                if app.modal.is_some() {
                    theme::displaced_bold()
                } else {
                    theme::active_preview_bold()
                }
            } else if is_skipped {
                theme::dim()
            } else if !map_focused {
                theme::muted()
            } else {
                Style::default()
            };

            if is_map_cursor {
                cursor_item_idx = Some(items.len());
            }

            let section_hint_spans: Vec<Span> = if section_hint_color == theme::HINT && !app.hint_buffer.is_empty() {
                let mut spans = hint_spans(&section_hint_display, &app.hint_buffer, theme::HINT);
                if let Some(last) = spans.last_mut() {
                    let s = last.content.to_string() + " ";
                    *last = Span::styled(s, last.style);
                }
                spans
            } else {
                vec![Span::styled(
                    format!("{} ", section_hint_display),
                    Style::default().fg(section_hint_color),
                )]
            };
            let mut section_line_spans = vec![Span::styled(cursor_char.to_string(), entry_style)];
            section_line_spans.extend(section_hint_spans);
            section_line_spans.push(Span::styled(section.map_label.clone(), entry_style));
            items.push(ListItem::new(Line::from(section_line_spans)));

            flat_idx += 1;
        }
    }

    let list_height = area.height as usize;
    let scroll = if let Some(ci) = cursor_item_idx {
        if items.len() <= list_height {
            0
        } else {
            ci.saturating_sub(list_height / 2).min(items.len().saturating_sub(list_height))
        }
    } else {
        0
    };

    let visible: Vec<ListItem> = items.into_iter().skip(scroll).take(list_height).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Sections ")
        .border_style(if map_focused {
            Style::default()
        } else {
            theme::muted()
        });
    let list = List::new(visible).block(block);
    f.render_widget(list, area);
}

fn render_wizard_widget(f: &mut Frame, app: &App, area: Rect) {
    // When map is focused, wizard previews the map-cursor section
    let map_preview = app.focus == Focus::Map;
    let idx = if map_preview {
        app.map_cursor
    } else {
        app.current_idx
    };
    let section_cfg = &app.sections[idx];

    let hints_active = !map_preview && app.modal.is_none();
    let field_hints = {
        let hints = crate::data::combined_hints(&app.data.keybindings);
        let cap = app.config.hint_labels_capitalized;
        let g_idx = app.group_idx_for_section(idx);
        if let Some(shi) = app.section_hint_key_idx(idx) {
            (0..hints.len())
                .filter(|&i| i != shi && i != g_idx)
                .filter_map(|i| hints.get(i))
                .map(|h| if cap { h.to_uppercase() } else { h.to_string() })
                .collect()
        } else {
            vec![]
        }
    };

    match app.section_states.get(idx) {
        Some(SectionState::Header(s)) => {
            let modal_for_header = if !map_preview { app.modal.as_ref() } else { None };
            render_header_widget(f, area, s, map_preview, &field_hints, hints_active, &app.hint_buffer, &app.config.sticky_values, modal_for_header)
        }
        Some(SectionState::FreeText(s)) => render_free_text_widget(f, area, s, section_cfg, map_preview),
        Some(SectionState::ListSelect(s)) => render_list_select_widget(f, area, s, section_cfg, map_preview),
        Some(SectionState::BlockSelect(s)) => render_block_select_widget(f, area, s, section_cfg, map_preview),
        Some(SectionState::Checklist(s)) => render_checklist_widget(f, area, s, section_cfg, map_preview),
        _ => {}
    }
}

fn render_header_widget(
    f: &mut Frame,
    area: Rect,
    state: &crate::sections::header::HeaderState,
    map_preview: bool,
    field_hints: &[String],
    hints_active: bool,
    hint_buffer: &str,
    sticky_values: &HashMap<String, String>,
    active_modal: Option<&SearchModal>,
) {
    let active_color = if map_preview { theme::ACTIVE_PREVIEW } else { theme::ACTIVE };
    let hint_color = if hints_active { theme::HINT } else { theme::MUTED };

    let block_title = Line::from(Span::raw(" Header "));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(block_title)
        .border_style(if map_preview { theme::muted() } else { Style::default() });
    let inner = block.inner(area);
    f.render_widget(block, area);

    let n = state.field_configs.len();
    let constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(3)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Clamp display index so cursor stays on last field when field_index is past end
    let display_field_index = state.field_index.min(n.saturating_sub(1));

    for i in 0..n.min(chunks.len()) {
        let cfg = &state.field_configs[i];
        let value = &state.values[i];
        let is_active = i == display_field_index;
        let has_value = !value.is_empty();
        let modal_for_field = active_modal.filter(|m| m.field_idx == i);

        let hint_str = field_hints.get(i).map(String::as_str).unwrap_or("");
        let field_title = if !hint_str.is_empty() {
            let hint_title_spans: Vec<Span> = if hints_active && !hint_buffer.is_empty() {
                // Leading space
                let mut spans = vec![Span::raw(" ")];
                spans.extend(hint_spans(hint_str, hint_buffer, theme::HINT));
                spans.push(Span::raw(" "));
                spans
            } else {
                vec![Span::styled(format!(" {} ", hint_str), Style::default().fg(hint_color))]
            };
            let mut title_spans = hint_title_spans;
            title_spans.push(Span::raw(format!("{} ", cfg.name)));
            Line::from(title_spans)
        } else {
            Line::from(Span::raw(format!(" {} ", cfg.name)))
        };

        // Border: orange when modal list-focused (return destination),
        // red when modal search-focused (displaced by search bar),
        // yellow when active, dark green when filled.
        let border_style = if is_active {
            if let Some(modal) = modal_for_field {
                if modal.focus == ModalFocus::SearchBar {
                    theme::displaced()
                } else {
                    Style::default().fg(theme::ACTIVE_PREVIEW)
                }
            } else {
                Style::default().fg(active_color)
            }
        } else if has_value {
            theme::selected_dark()
        } else {
            Style::default()
        };

        let field_block = Block::default()
            .borders(Borders::ALL)
            .title(field_title)
            .border_style(border_style);

        // Case 1: Modal open for this field - show dynamic selection preview (Feature 3)
        if let Some(modal) = modal_for_field {
            let line = build_modal_field_line(modal, sticky_values);
            f.render_widget(Paragraph::new(line).block(field_block), chunks[i]);
            continue;
        }

        // Case 2: Active, mid-composite entry (composite_spans set)
        if is_active && !map_preview {
            if let Some(ref spans) = state.composite_spans {
                let mut line_spans: Vec<Span> = spans.iter().map(|(text, confirmed)| {
                    if *confirmed {
                        Span::styled(text.clone(), theme::selected())
                    } else {
                        Span::styled(text.clone(), theme::muted())
                    }
                }).collect();
                line_spans.push(Span::styled("_", theme::muted()));
                f.render_widget(Paragraph::new(Line::from(line_spans)).block(field_block), chunks[i]);
                continue;
            }
        }

        // Case 3: Has actual value - show in green
        if has_value {
            let display = if is_active && !map_preview {
                format!("{}_", value)
            } else {
                value.clone()
            };
            f.render_widget(Paragraph::new(display).style(theme::selected()).block(field_block), chunks[i]);
            continue;
        }

        // Case 4: No value - show preload in grey, or cursor if active and no preload
        let preload = compute_field_preload(cfg, sticky_values);
        if let Some(p) = preload {
            f.render_widget(Paragraph::new(p).style(theme::muted()).block(field_block), chunks[i]);
        } else {
            let display = if is_active && !map_preview { "_" } else { "" };
            f.render_widget(Paragraph::new(display).block(field_block), chunks[i]);
        }
    }
}

fn render_free_text_widget(
    f: &mut Frame,
    area: Rect,
    state: &crate::sections::free_text::FreeTextState,
    cfg: &crate::data::SectionConfig,
    map_preview: bool,
) {
    let active_color = if map_preview { theme::ACTIVE_PREVIEW } else { theme::ACTIVE };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name))
        .border_style(if map_preview { theme::muted() } else { Style::default() });
    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.is_editing() {
        // Show edit buffer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(inner);

        // Existing entries
        let items: Vec<ListItem> = state
            .entries
            .iter()
            .map(|e| {
                ListItem::new(Span::styled(format!("  {}", e), Style::default()))
            })
            .collect();
        let list = List::new(items);
        f.render_widget(list, chunks[0]);

        // Edit buffer
        let edit_block = Block::default()
            .borders(Borders::ALL)
            .title(" New Entry ")
            .border_style(Style::default().fg(active_color));  // active_color from theme
        let edit_text = format!("{}_", state.edit_buf);
        let edit_para = Paragraph::new(edit_text)
            .style(Style::default().fg(active_color))
            .block(edit_block)
            .wrap(Wrap { trim: false });
        f.render_widget(edit_para, chunks[1]);
    } else {
        // Show list
        if state.entries.is_empty() {
            let hint = Paragraph::new(
                "[a] to add entry, [t/Enter] to skip",
            )
            .style(theme::dim());
            f.render_widget(hint, inner);
        } else {
            let items: Vec<ListItem> = state
                .entries
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let style = if i == state.cursor {
                        Style::default().add_modifier(Modifier::BOLD).fg(active_color)
                    } else if map_preview {
                        theme::muted()
                    } else {
                        theme::selected()
                    };
                    let prefix = if i == state.cursor { "> " } else { "  " };
                    ListItem::new(Span::styled(format!("{}{}", prefix, e), style))
                })
                .collect();
            let list = List::new(items);
            f.render_widget(list, inner);
        }
    }
}

fn render_list_select_widget(
    f: &mut Frame,
    area: Rect,
    state: &crate::sections::list_select::ListSelectState,
    cfg: &crate::data::SectionConfig,
    map_preview: bool,
) {
    let active_color = if map_preview { theme::ACTIVE_PREVIEW } else { theme::ACTIVE };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name))
        .border_style(if map_preview { theme::muted() } else { Style::default() });
    let inner = block.inner(area);
    f.render_widget(block, area);

    match state.mode {
        ListSelectMode::Browsing => {
            let height = inner.height as usize;
            let n = state.entries.len();
            let scroll = if n <= height {
                0
            } else {
                (state.cursor + 1).saturating_sub(height).min(n - height)
            };
            let items: Vec<ListItem> = state
                .entries
                .iter()
                .enumerate()
                .skip(scroll)
                .take(height)
                .map(|(i, entry)| {
                    let is_sel = state.is_selected(i);
                    let is_cur = i == state.cursor;
                    let check = if is_sel { "[x]" } else { "[ ]" };
                    let prefix = if is_cur { ">" } else { " " };
                    let style = if is_cur {
                        Style::default().add_modifier(Modifier::BOLD).fg(active_color)
                    } else if is_sel {
                        theme::selected()
                    } else if map_preview {
                        theme::muted()
                    } else {
                        Style::default()
                    };
                    ListItem::new(Span::styled(
                        format!("{} {} {}", prefix, check, entry.label),
                        style,
                    ))
                })
                .collect();

            let list = List::new(items);
            f.render_widget(list, inner);
        }
        ListSelectMode::AddingLabel => {
            render_add_entry_form(f, inner, "Label:", &state.add_label_buf, true);
        }
        ListSelectMode::AddingOutput => {
            render_add_entry_form(f, inner, "Output:", &state.add_output_buf, false);
        }
    }
}

fn render_add_entry_form(f: &mut Frame, area: Rect, label: &str, buf: &str, is_label: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let hint = if is_label {
        "Enter label for display, then [Enter] for output"
    } else {
        "Enter output text, then [Enter] to save"
    };

    let hint_para = Paragraph::new(hint).style(theme::dim());
    f.render_widget(hint_para, chunks[0]);

    let field_block = Block::default()
        .borders(Borders::ALL)
        .title(label)
        .border_style(theme::active());
    let text = format!("{}_", buf);
    let para = Paragraph::new(text)
        .style(theme::active())
        .block(field_block);
    f.render_widget(para, chunks[1]);
}

fn render_block_select_widget(
    f: &mut Frame,
    area: Rect,
    state: &crate::sections::block_select::BlockSelectState,
    cfg: &crate::data::SectionConfig,
    map_preview: bool,
) {
    let active_color = if map_preview { theme::ACTIVE_PREVIEW } else { theme::ACTIVE };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name))
        .border_style(if map_preview { theme::muted() } else { Style::default() });
    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.in_techniques() {
        let region_idx = state.current_region_idx().unwrap_or(0);
        if let Some(region) = state.regions.get(region_idx) {
            let title = format!(" {} - Techniques ", region.label);
            let region_block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(active_color));
            let region_inner = region_block.inner(inner);
            f.render_widget(region_block, inner);

            let height = region_inner.height as usize;
            let n = region.techniques.len();
            let scroll = if n <= height {
                0
            } else {
                (state.technique_cursor + 1).saturating_sub(height).min(n - height)
            };
            let items: Vec<ListItem> = region
                .techniques
                .iter()
                .enumerate()
                .skip(scroll)
                .take(height)
                .map(|(i, tech)| {
                    let is_sel = region.technique_selected.get(i).copied().unwrap_or(false);
                    let is_cur = i == state.technique_cursor;
                    let check = if is_sel { "[x]" } else { "[ ]" };
                    let prefix = if is_cur { ">" } else { " " };
                    let style = if is_cur {
                        Style::default().add_modifier(Modifier::BOLD).fg(active_color)
                    } else if is_sel {
                        theme::selected()
                    } else if map_preview {
                        theme::muted()
                    } else {
                        Style::default()
                    };
                    ListItem::new(Span::styled(
                        format!("{} {} {}", prefix, check, tech.label()),
                        style,
                    ))
                })
                .collect();

            let list = List::new(items);
            f.render_widget(list, region_inner);
        }
    } else {
        let height = inner.height as usize;
        let n = state.regions.len();
        let scroll = if n <= height {
            0
        } else {
            (state.region_cursor + 1).saturating_sub(height).min(n - height)
        };
        let items: Vec<ListItem> = state
            .regions
            .iter()
            .enumerate()
            .skip(scroll)
            .take(height)
            .map(|(i, region)| {
                let is_cur = i == state.region_cursor;
                let has_sel = region.has_selection();
                let check = if has_sel { "[x]" } else { "[ ]" };
                let prefix = if is_cur { ">" } else { " " };
                let style = if is_cur {
                    Style::default().add_modifier(Modifier::BOLD).fg(active_color)
                } else if has_sel {
                    theme::selected()
                } else if map_preview {
                    theme::muted()
                } else {
                    Style::default()
                };
                ListItem::new(Span::styled(
                    format!("{} {} {}", prefix, check, region.label),
                    style,
                ))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, inner);
    }
}

fn render_checklist_widget(
    f: &mut Frame,
    area: Rect,
    state: &crate::sections::checklist::ChecklistState,
    cfg: &crate::data::SectionConfig,
    map_preview: bool,
) {
    let active_color = if map_preview { theme::ACTIVE_PREVIEW } else { theme::ACTIVE };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name))
        .border_style(if map_preview { theme::muted() } else { Style::default() });
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    let n = state.items.len();
    let scroll = if n <= height {
        0
    } else {
        (state.cursor + 1).saturating_sub(height).min(n - height)
    };
    let items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .skip(scroll)
        .take(height)
        .map(|(i, item)| {
            let is_checked = state.checked.get(i).copied().unwrap_or(true);
            let is_cur = i == state.cursor;
            let check = if is_checked { "[x]" } else { "[ ]" };
            let prefix = if is_cur { ">" } else { " " };
            let style = if is_cur {
                Style::default().add_modifier(Modifier::BOLD).fg(active_color)
            } else if map_preview {
                theme::muted()
            } else if is_checked {
                theme::selected()
            } else {
                theme::dim()
            };
            ListItem::new(Span::styled(
                format!("{} {} {}", prefix, check, item),
                style,
            ))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn render_search_modal(f: &mut Frame, app: &App, wizard_area: Rect) {
    let modal = match &app.modal {
        Some(m) => m,
        None => return,
    };

    let modal_width = wizard_area.width.saturating_sub(4).min(60);
    let hints = crate::data::combined_hints(&app.data.keybindings);
    // Height based on total entries so modal doesn't jump when filtering
    // 2 outer borders + 3 search bar (with its own borders) + list items
    let list_height = modal.all_entries.len().min(hints.len()) as u16;
    let modal_height = (2 + 3 + list_height).min(wizard_area.height.saturating_sub(2));
    let x = wizard_area.x + wizard_area.width.saturating_sub(modal_width) / 2;
    let y = wizard_area.y + wizard_area.height.saturating_sub(modal_height) / 2;
    let area = Rect::new(x, y, modal_width, modal_height);

    f.render_widget(Clear, area);

    let title = if let Some(label) = modal.current_part_label() {
        format!(" {} ", label)
    } else {
        " Search ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(theme::modal());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    // Search bar - yellow when active (typing), modal color border otherwise
    let search_active = modal.focus == ModalFocus::SearchBar;
    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if search_active { theme::active() } else { theme::modal() });
    let query_text = if search_active {
        format!("{}_", modal.query)
    } else {
        "[space] search".to_string()
    };
    let search_para = Paragraph::new(query_text)
        .style(if search_active { theme::active() } else { theme::muted() })
        .block(search_block);
    f.render_widget(search_para, chunks[0]);

    // List - render visible window starting at list_scroll
    if modal.filtered.is_empty() {
        let no_results = Paragraph::new("  no results")
            .style(theme::dim());
        f.render_widget(no_results, chunks[1]);
        return;
    }

    let list_focused = modal.focus == ModalFocus::List;
    let scroll = modal.list_scroll;
    let window_end = (scroll + hints.len()).min(modal.filtered.len());
    let hint_color = if list_focused { theme::HINT } else { theme::MUTED };
    let items: Vec<ListItem> = modal.filtered[scroll..window_end]
        .iter()
        .enumerate()
        .map(|(hint_idx, &entry_idx)| {
            let abs_idx = scroll + hint_idx;
            let entry = &modal.all_entries[entry_idx];
            let hint = hints.get(hint_idx).copied().unwrap_or(" ");
            let is_cur = abs_idx == modal.list_cursor;
            let entry_style = if is_cur && list_focused {
                theme::active_bold()   // yellow: currently browsing list
            } else if is_cur {
                theme::active_preview_bold()   // orange: list is return destination when searching
            } else {
                Style::default()
            };
            let prefix = if is_cur { ">" } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", prefix), entry_style),
                Span::styled(format!("{} ", hint), Style::default().fg(hint_color)),
                Span::styled(entry.clone(), entry_style),
            ]))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[1]);
}

/// Build a styled Line showing the current modal selection within the field's composite format.
/// Confirmed parts are green, the current part is yellow+bold, future parts show sticky/default/preview in grey.
/// For simple (non-composite) fields, shows the current selection in yellow+bold.
fn build_modal_field_line(modal: &SearchModal, sticky_values: &HashMap<String, String>) -> Line<'static> {
    let current_selection = modal.selected_value().map(String::from).unwrap_or_default();

    if let Some(ref comp) = modal.composite {
        let format = &comp.config.format;
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut literal = String::new();
        let mut chars = format.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if !literal.is_empty() {
                    spans.push(Span::raw(literal.clone()));
                    literal.clear();
                }
                let mut id = String::new();
                for c2 in chars.by_ref() {
                    if c2 == '}' { break; }
                    id.push(c2);
                }
                if let Some(part_pos) = comp.config.parts.iter().position(|p| p.id == id) {
                    if part_pos < comp.values.len() {
                        // Already confirmed - green
                        spans.push(Span::styled(comp.values[part_pos].clone(), theme::selected()));
                    } else if part_pos == comp.part_idx {
                        // Currently being chosen - yellow + bold
                        spans.push(Span::styled(current_selection.clone(), theme::active_bold()));
                    } else {
                        // Future part - grey, use sticky > default > preview
                        let part = &comp.config.parts[part_pos];
                        let val = if part.sticky {
                            let key = format!("{}.{}", modal.field_id, part.id);
                            sticky_values.get(&key).cloned()
                                .or_else(|| resolve_part_default(part))
                                .or_else(|| part.preview.clone())
                        } else {
                            resolve_part_default(part)
                                .or_else(|| part.preview.clone())
                        }.unwrap_or_else(|| format!("{{{}}}", id));
                        spans.push(Span::styled(val, theme::muted()));
                    }
                }
            } else {
                literal.push(c);
            }
        }
        if !literal.is_empty() {
            spans.push(Span::raw(literal));
        }
        Line::from(spans)
    } else {
        // Simple field: current selection in yellow+bold
        Line::from(vec![Span::styled(current_selection, theme::active_bold())])
    }
}

/// Compute the preload value for a header field from sticky > default > preview.
/// Returns None if no preload is available (no default, sticky, or preview configured).
fn compute_field_preload(cfg: &HeaderFieldConfig, sticky_values: &HashMap<String, String>) -> Option<String> {
    if let Some(ref composite) = cfg.composite {
        let mut result = composite.format.clone();
        for part in &composite.parts {
            let val = if part.sticky {
                let key = format!("{}.{}", cfg.id, part.id);
                sticky_values.get(&key).cloned()
                    .or_else(|| resolve_part_default(part))
                    .or_else(|| part.preview.clone())
            } else {
                resolve_part_default(part)
                    .or_else(|| part.preview.clone())
            };
            let part_value = val.unwrap_or_else(|| format!("{{{}}}", part.id));
            result = result.replace(&format!("{{{}}}", part.id), &part_value);
        }
        Some(result)
    } else {
        // Simple field: use default if available
        cfg.default.clone()
    }
}

fn resolve_part_default(part: &crate::data::CompositePart) -> Option<String> {
    if part.default.is_none() {
        return None;
    }
    let cursor = part.default_cursor();
    part.options.get(cursor).map(|o| o.output().to_string())
}

fn render_note_pane(f: &mut Frame, app: &App, area: Rect) {
    let note_text = render_note(&app.sections, &app.section_states, &app.config.sticky_values, NoteRenderMode::Preview);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Note Preview ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let para = Paragraph::new(note_text)
        .wrap(Wrap { trim: false })
        .style(Style::default())
        .scroll((app.note_scroll, 0));
    f.render_widget(para, inner);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let quit_key = app.data.keybindings.quit.first().map(|s| s.as_str()).unwrap_or("q");
    let copy_key = app.data.keybindings.copy_note.first().map(|s| s.as_str()).unwrap_or("c");
    let (text, style) = if let Some(ref status) = app.status {
        let s = if status.is_error { theme::error() } else { theme::selected() };
        (status.text.clone(), s)
    } else if app.note_completed {
        (
            format!("Note complete! [{}] copy  [{}] quit", copy_key, quit_key),
            theme::selected(),
        )
    } else {
        let section = app.sections.get(app.current_idx);
        let name = section.map(|s| s.name.as_str()).unwrap_or("");
        (
            format!(
                " {}/{}  {}   [?] help  [{}] quit  [{}] copy  [`] swap  [h/i] map",
                app.current_idx + 1,
                app.sections.len(),
                name,
                quit_key,
                copy_key
            ),
            theme::dim(),
        )
    };

    let para = Paragraph::new(text).style(style);
    f.render_widget(para, area);
}

fn render_help_overlay(f: &mut Frame, app: &App, area: Rect) {
    let width = 60u16.min(area.width.saturating_sub(4));
    let height = 24u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    f.render_widget(Clear, overlay_area);

    let kb = &app.data.keybindings;
    let fmt = |keys: &[String]| keys.join("  ");
    let lines: Vec<Line> = vec![
        Line::from(Span::styled("KEYBINDINGS", theme::bold())),
        Line::from(""),
        Line::from(format!("Navigate down    {}", fmt(&kb.navigate_down))),
        Line::from(format!("Navigate up      {}", fmt(&kb.navigate_up))),
        Line::from(format!("Select           {}", fmt(&kb.select))),
        Line::from(format!("Confirm/Next     {}", fmt(&kb.confirm))),
        Line::from(format!("Add Entry        {}", fmt(&kb.add_entry))),
        Line::from(format!("Back/Cancel      {}", fmt(&kb.back))),
        Line::from(format!("Focus left       {}", fmt(&kb.focus_left))),
        Line::from(format!("Focus right      {}", fmt(&kb.focus_right))),
        Line::from(format!("Swap Panes       {}", fmt(&kb.swap_panes))),
        Line::from(format!("Help             {}", fmt(&kb.help))),
        Line::from(format!("Quit             {}", fmt(&kb.quit))),
        Line::from(format!("Copy Note        {}", fmt(&kb.copy_note))),
        Line::from(""),
        Line::from(Span::styled("SECTION BEHAVIOR", theme::bold())),
        Line::from(""),
        Line::from("Header       fill 4 fields, Enter to advance each"),
        Line::from("Free Text    a to add entry, Enter confirm/skip"),
        Line::from("List Select  space/s toggle, Enter confirm"),
        Line::from("Regions      Enter drill into region, space toggle"),
        Line::from("             Esc back, a/d confirm all"),
        Line::from("Checklist    space/s toggle, Enter confirm"),
        Line::from(""),
        Line::from(Span::styled("? or Esc to close", theme::dim())),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(theme::modal());
    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, overlay_area);
}
