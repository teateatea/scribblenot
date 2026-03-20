use crate::app::{App, SectionState};
use crate::note::render_note;
use crate::sections::list_select::ListSelectMode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
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

    // Split content 50/50
    let pane_constraints = [Constraint::Percentage(50), Constraint::Percentage(50)];
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(pane_constraints)
        .split(content_area);

    let (left_pane, right_pane) = if app.pane_swapped {
        (panes[1], panes[0])
    } else {
        (panes[0], panes[1])
    };

    render_wizard_pane(f, app, left_pane);
    render_note_pane(f, app, right_pane);
    render_status_bar(f, app, status_area);

    if app.show_help {
        render_help_overlay(f, app, size);
    }
}

fn render_wizard_pane(f: &mut Frame, app: &App, area: Rect) {
    // Split left pane: section map (30 chars) + wizard
    let map_width = 28u16;
    let left_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(map_width), Constraint::Min(1)])
        .split(area);

    render_section_map(f, app, left_chunks[0]);
    render_wizard_widget(f, app, left_chunks[1]);
}

fn render_section_map(f: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_idx = 0usize;

    for group in &app.data.groups {
        // Group header
        items.push(ListItem::new(Line::from(vec![Span::styled(
            format!(" {}", group.name),
            Style::default().add_modifier(Modifier::BOLD),
        )])));

        for (si, section) in group.sections.iter().enumerate() {
            let is_current = flat_idx == app.current_idx;
            let is_completed = app.section_is_completed(flat_idx);
            let is_skipped = app.section_is_skipped(flat_idx);

            let prefix = if is_skipped { "- " } else { "  " };
            let label = format!("{}{}.{} {}", prefix, group_num(&group.id), si + 1, section.map_label);

            let style = if is_current {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_completed || is_skipped {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default()
            };

            items.push(ListItem::new(Line::from(Span::styled(label, style))));
            flat_idx += 1;
        }
    }

    let block = Block::default()
        .borders(Borders::RIGHT)
        .title(" Sections ");
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn group_num(id: &str) -> usize {
    match id {
        "intake" => 1,
        "subjective" => 2,
        "treatment" => 3,
        "objective" => 4,
        "post_tx" => 5,
        _ => 0,
    }
}

fn render_wizard_widget(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let main_area = chunks[0];
    let footer_area = chunks[1];

    let idx = app.current_idx;
    let section_cfg = &app.sections[idx];

    match app.section_states.get(idx) {
        Some(SectionState::Header(s)) => render_header_widget(f, app, main_area, s),
        Some(SectionState::FreeText(s)) => render_free_text_widget(f, app, main_area, s, section_cfg),
        Some(SectionState::ListSelect(s)) => render_list_select_widget(f, app, main_area, s, section_cfg),
        Some(SectionState::BlockSelect(s)) => render_block_select_widget(f, app, main_area, s, section_cfg),
        Some(SectionState::Checklist(s)) => render_checklist_widget(f, app, main_area, s, section_cfg),
        _ => {}
    }

    // Footer
    let footer_text = Line::from(vec![
        Span::styled("[a] add  ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled("[?] help  ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled("[q] quit", Style::default().add_modifier(Modifier::DIM)),
    ]);
    let footer = Paragraph::new(footer_text);
    f.render_widget(footer, footer_area);
}

fn render_header_widget(
    f: &mut Frame,
    _app: &App,
    area: Rect,
    state: &crate::sections::header::HeaderState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Header ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3); 4])
        .split(inner);

    for i in 0..4 {
        if i >= chunks.len() {
            break;
        }
        let label = state.field_label(i);
        let value = if i == state.field_index {
            format!("{}_", state.edit_buf)
        } else {
            match i {
                0 => state.date.clone(),
                1 => state.start_time.clone(),
                2 => state.duration.clone(),
                3 => state.appointment_type.clone(),
                _ => String::new(),
            }
        };

        let style = if i == state.field_index {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if i < state.field_index {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default()
        };

        let field_block = Block::default()
            .borders(Borders::ALL)
            .title(label)
            .border_style(if i == state.field_index {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });
        let paragraph = Paragraph::new(value).style(style).block(field_block);
        f.render_widget(paragraph, chunks[i]);
    }
}

fn render_free_text_widget(
    f: &mut Frame,
    _app: &App,
    area: Rect,
    state: &crate::sections::free_text::FreeTextState,
    cfg: &crate::data::SectionConfig,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name));
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
            .enumerate()
            .map(|(i, e)| {
                let style = if i == state.cursor && !state.is_editing() {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Span::styled(format!("  {}", e), style))
            })
            .collect();
        let list = List::new(items);
        f.render_widget(list, chunks[0]);

        // Edit buffer
        let edit_block = Block::default()
            .borders(Borders::ALL)
            .title(" New Entry ")
            .border_style(Style::default().fg(Color::Cyan));
        let edit_text = format!("{}_", state.edit_buf);
        let edit_para = Paragraph::new(edit_text)
            .style(Style::default().fg(Color::Cyan))
            .block(edit_block)
            .wrap(Wrap { trim: false });
        f.render_widget(edit_para, chunks[1]);
    } else {
        // Show list
        if state.entries.is_empty() {
            let hint = Paragraph::new(
                "[a] to add entry, [t/Enter] to skip",
            )
            .style(Style::default().add_modifier(Modifier::DIM));
            f.render_widget(hint, inner);
        } else {
            let items: Vec<ListItem> = state
                .entries
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let style = if i == state.cursor {
                        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
                    } else {
                        Style::default()
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
    _app: &App,
    area: Rect,
    state: &crate::sections::list_select::ListSelectState,
    cfg: &crate::data::SectionConfig,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name));
    let inner = block.inner(area);
    f.render_widget(block, area);

    match state.mode {
        ListSelectMode::Browsing => {
            let items: Vec<ListItem> = state
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let is_sel = state.is_selected(i);
                    let is_cur = i == state.cursor;
                    let check = if is_sel { "[x]" } else { "[ ]" };
                    let prefix = if is_cur { ">" } else { " " };
                    let style = if is_cur {
                        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
                    } else if is_sel {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Span::styled(
                        format!("{} {} {}", prefix, check, entry.label),
                        style,
                    ))
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(state.cursor));
            let list = List::new(items);
            f.render_stateful_widget(list, inner, &mut list_state);
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

    let hint_para = Paragraph::new(hint).style(Style::default().add_modifier(Modifier::DIM));
    f.render_widget(hint_para, chunks[0]);

    let field_block = Block::default()
        .borders(Borders::ALL)
        .title(label)
        .border_style(Style::default().fg(Color::Cyan));
    let text = format!("{}_", buf);
    let para = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(field_block);
    f.render_widget(para, chunks[1]);
}

fn render_block_select_widget(
    f: &mut Frame,
    _app: &App,
    area: Rect,
    state: &crate::sections::block_select::BlockSelectState,
    cfg: &crate::data::SectionConfig,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.in_techniques() {
        let region_idx = state.current_region_idx().unwrap_or(0);
        if let Some(region) = state.regions.get(region_idx) {
            let title = format!(" {} - Techniques ", region.label);
            let region_block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan));
            let region_inner = region_block.inner(inner);
            f.render_widget(region_block, inner);

            let items: Vec<ListItem> = region
                .techniques
                .iter()
                .enumerate()
                .map(|(i, tech)| {
                    let is_sel = region.technique_selected.get(i).copied().unwrap_or(false);
                    let is_cur = i == state.technique_cursor;
                    let check = if is_sel { "[x]" } else { "[ ]" };
                    let prefix = if is_cur { ">" } else { " " };
                    let style = if is_cur {
                        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
                    } else if is_sel {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Span::styled(
                        format!("{} {} {}", prefix, check, tech.label),
                        style,
                    ))
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(state.technique_cursor));
            let list = List::new(items);
            f.render_stateful_widget(list, region_inner, &mut list_state);
        }
    } else {
        let items: Vec<ListItem> = state
            .regions
            .iter()
            .enumerate()
            .map(|(i, region)| {
                let is_cur = i == state.region_cursor;
                let has_sel = region.has_selection();
                let check = if has_sel { "[x]" } else { "[ ]" };
                let prefix = if is_cur { ">" } else { " " };
                let style = if is_cur {
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
                } else if has_sel {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };
                ListItem::new(Span::styled(
                    format!("{} {} {}", prefix, check, region.label),
                    style,
                ))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(state.region_cursor));
        let list = List::new(items);
        f.render_stateful_widget(list, inner, &mut list_state);
    }
}

fn render_checklist_widget(
    f: &mut Frame,
    _app: &App,
    area: Rect,
    state: &crate::sections::checklist::ChecklistState,
    cfg: &crate::data::SectionConfig,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", cfg.name));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_checked = state.checked.get(i).copied().unwrap_or(true);
            let is_cur = i == state.cursor;
            let check = if is_checked { "[x]" } else { "[ ]" };
            let prefix = if is_cur { ">" } else { " " };
            let style = if is_cur {
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
            } else if is_checked {
                Style::default().fg(Color::Green)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };
            ListItem::new(Span::styled(
                format!("{} {} {}", prefix, check, item),
                style,
            ))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.cursor));
    let list = List::new(items);
    f.render_stateful_widget(list, inner, &mut list_state);
}

fn render_note_pane(f: &mut Frame, app: &App, area: Rect) {
    let note_text = render_note(&app.sections, &app.section_states);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Note Preview ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let para = Paragraph::new(note_text)
        .wrap(Wrap { trim: false })
        .style(Style::default());
    f.render_widget(para, inner);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (text, style) = if let Some(ref status) = app.status {
        let s = if status.is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        (status.text.clone(), s)
    } else if app.note_completed {
        (
            "Note complete! Note copied to clipboard. [q] quit".to_string(),
            Style::default().fg(Color::Green),
        )
    } else {
        let section = app.sections.get(app.current_idx);
        let name = section.map(|s| s.name.as_str()).unwrap_or("");
        (
            format!(
                " Section {}/{}: {}  |  [?] help  [q] quit  [`] swap panes",
                app.current_idx + 1,
                app.sections.len(),
                name
            ),
            Style::default().add_modifier(Modifier::DIM),
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
    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            " KEYBINDINGS",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            " Navigate down : {:?}",
            kb.navigate_down
        )),
        Line::from(format!(
            " Navigate up   : {:?}",
            kb.navigate_up
        )),
        Line::from(format!(" Select        : {:?}", kb.select)),
        Line::from(format!(
            " Confirm/Next  : {:?}",
            kb.confirm
        )),
        Line::from(format!(
            " Add Entry     : {:?}",
            kb.add_entry
        )),
        Line::from(format!(" Back/Cancel   : {:?}", kb.back)),
        Line::from(format!(
            " Swap Panes    : {:?}",
            kb.swap_panes
        )),
        Line::from(format!(" Help          : {:?}", kb.help)),
        Line::from(format!(" Quit          : {:?}", kb.quit)),
        Line::from(""),
        Line::from(Span::styled(
            " SECTION BEHAVIOR",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(" Header       : Fill 4 fields, [Enter] to advance each"),
        Line::from(" Free Text    : [a] add entry, [Enter] confirm/skip"),
        Line::from(" List Select  : [Space/s] toggle, [Enter/t] confirm"),
        Line::from(" Block Select : [Enter/t] drill into region, [Space/s] toggle"),
        Line::from("                [Esc] back, [a/d] confirm all regions"),
        Line::from(" Checklist    : [Space/s] toggle, [Enter/t] confirm"),
        Line::from(""),
        Line::from(Span::styled(
            " Press [?] or [Esc] to close",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(Color::Cyan));
    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, overlay_area);
}
