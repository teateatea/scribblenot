use crate::config::Config;
use crate::data::{AppData, SectionConfig};
use crate::modal::{CompositeAdvance, ModalFocus, SearchModal};
use crate::sections::{
    block_select::BlockSelectState,
    checklist::ChecklistState,
    free_text::FreeTextState,
    header::HeaderState,
    list_select::{ListSelectMode, ListSelectState},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Wizard,
    Map,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapHintLevel {
    Groups,
    Sections(usize),
}

#[derive(Debug, Clone)]
pub enum SectionState {
    Pending,
    Header(HeaderState),
    FreeText(FreeTextState),
    ListSelect(ListSelectState),
    BlockSelect(BlockSelectState),
    Checklist(ChecklistState),
}

#[derive(Debug, Clone)]
pub struct StatusMsg {
    pub text: String,
    pub is_error: bool,
    pub created_at: Instant,
}

impl StatusMsg {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
            created_at: Instant::now(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
            created_at: Instant::now(),
        }
    }
}

pub struct App {
    pub sections: Vec<SectionConfig>,
    pub section_states: Vec<SectionState>,
    pub current_idx: usize,
    pub data: AppData,
    pub config: Config,
    pub pane_swapped: bool,
    pub show_help: bool,
    pub status: Option<StatusMsg>,
    pub quit: bool,
    pub note_completed: bool,
    pub copy_requested: bool,
    pub data_dir: PathBuf,
    pub focus: Focus,
    pub map_cursor: usize,
    pub map_hint_level: MapHintLevel,
    pub note_scroll: u16,
    pub modal: Option<SearchModal>,
    pub hint_buffer: String,
}

pub fn match_binding_str(binding: &str, key: &KeyEvent) -> bool {
    match binding {
        "down" => key.code == KeyCode::Down,
        "up" => key.code == KeyCode::Up,
        "left" => key.code == KeyCode::Left,
        "right" => key.code == KeyCode::Right,
        "enter" => key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE,
        "esc" => key.code == KeyCode::Esc,
        "space" => key.code == KeyCode::Char(' '),
        "backspace" => key.code == KeyCode::Backspace,
        "shift+enter" => key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::SHIFT),
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            key.code == KeyCode::Char(c)
        }
        _ => false,
    }
}

impl App {
    pub fn new(data: AppData, config: Config, data_dir: PathBuf) -> Self {
        let sections = data.sections.clone();
        let section_states = Self::init_states(&sections, &data);
        let pane_swapped = config.is_swapped();
        Self {
            sections,
            section_states,
            current_idx: 0,
            data,
            config,
            pane_swapped,
            show_help: false,
            status: None,
            quit: false,
            note_completed: false,
            copy_requested: false,
            data_dir,
            focus: Focus::Wizard,
            map_cursor: 0,
            map_hint_level: MapHintLevel::Groups,
            note_scroll: 0,
            modal: None,
            hint_buffer: String::new(),
        }
    }

    fn init_states(sections: &[SectionConfig], data: &AppData) -> Vec<SectionState> {
        sections
            .iter()
            .map(|cfg| match cfg.section_type.as_str() {
                "multi_field" => {
                    let fields = cfg.fields.clone().unwrap_or_default();
                    SectionState::Header(HeaderState::new(fields))
                }
                "free_text" => SectionState::FreeText(FreeTextState::new()),
                "list_select" => {
                    let entries = cfg
                        .data_file
                        .as_ref()
                        .and_then(|f| data.list_data.get(f))
                        .cloned()
                        .unwrap_or_default();
                    SectionState::ListSelect(ListSelectState::new(entries))
                }
                "block_select" => {
                    let regions = data.block_select_data.get(&cfg.id)
                        .cloned()
                        .unwrap_or_default();
                    SectionState::BlockSelect(BlockSelectState::new(regions))
                }
                "checklist" => {
                    let items = cfg
                        .data_file
                        .as_ref()
                        .and_then(|f| data.checklist_data.get(f))
                        .cloned()
                        .unwrap_or_default();
                    SectionState::Checklist(ChecklistState::new(items))
                }
                _ => SectionState::Pending,
            })
            .collect()
    }

    pub fn tick(&mut self) {
        if let Some(ref status) = self.status {
            if status.created_at.elapsed().as_secs() >= 2 {
                self.status = None;
            }
        }
    }

    fn matches_key(&self, key: &KeyEvent, action: &[String]) -> bool {
        for binding in action {
            let matched = match_binding_str(binding, key);
            if matched {
                return true;
            }
        }
        false
    }

    fn is_navigate_down(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.navigate_down)
    }

    fn is_navigate_up(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.navigate_up)
    }

    fn is_select(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.select)
    }

    fn is_confirm(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.confirm)
    }

    fn is_add_entry(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.add_entry)
    }

    fn is_back(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.back)
    }

    fn is_swap_panes(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.swap_panes)
    }

    fn is_help(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.help)
    }

    fn is_quit(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.quit)
    }

    fn is_copy_note(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.copy_note)
    }

    fn is_focus_left(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.focus_left)
    }

    fn is_focus_right(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.focus_right)
    }

    fn is_super_confirm(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.super_confirm)
    }

    fn section_at_top_level(&self) -> bool {
        match self.section_states.get(self.current_idx) {
            Some(SectionState::FreeText(s)) => !s.is_editing(),
            Some(SectionState::ListSelect(s)) => matches!(s.mode, ListSelectMode::Browsing),
            Some(SectionState::BlockSelect(s)) => !s.in_items(),
            Some(SectionState::Checklist(_)) => true,
            _ => false,
        }
    }

    fn handle_map_key(&mut self, key: KeyEvent) {
        if self.is_navigate_down(&key) {
            self.hint_buffer.clear();
            if self.map_cursor + 1 < self.sections.len() {
                self.map_cursor += 1;
                let g = self.group_idx_for_section(self.map_cursor);
                self.map_hint_level = MapHintLevel::Sections(g);
                self.update_note_scroll();
            }
            return;
        }
        if self.is_navigate_up(&key) {
            self.hint_buffer.clear();
            if self.map_cursor > 0 {
                self.map_cursor -= 1;
                let g = self.group_idx_for_section(self.map_cursor);
                self.map_hint_level = MapHintLevel::Sections(g);
                self.update_note_scroll();
            }
            return;
        }
        if self.is_confirm(&key) {
            self.hint_buffer.clear();
            self.current_idx = self.map_cursor;
            self.focus = Focus::Wizard;
            self.map_hint_level = MapHintLevel::Groups;
            return;
        }
        if self.is_back(&key) {
            self.hint_buffer.clear();
            self.focus = Focus::Wizard;
            self.map_hint_level = MapHintLevel::Groups;
            return;
        }

        // Hint key navigation
        if let KeyCode::Char(c) = key.code {
            let case_sensitive = self.config.hint_labels_case_sensitive;
            let ch_str: String = if case_sensitive {
                c.to_string()
            } else {
                c.to_ascii_lowercase().to_string()
            };
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();

            let hints = crate::data::combined_hints(&self.data.keybindings);
            // Build a case-folded hint list matching the typed string's case
            let folded_hints: Vec<String> = hints.iter().map(|h| {
                if case_sensitive { h.to_string() } else { h.to_ascii_lowercase().to_string() }
            }).collect();
            let folded_refs: Vec<&str> = folded_hints.iter().map(String::as_str).collect();

            // Universal group-jump: fires at any map_hint_level
            let n_groups = self.data.groups.len();
            let group_refs: Vec<&str> = folded_refs.iter().take(n_groups).copied().collect();
            match crate::data::resolve_hint(&group_refs, &typed) {
                crate::data::HintResolveResult::Exact(g_idx) => {
                    let flat_idx = crate::data::group_jump_target(&self.data.groups, g_idx);
                    self.map_cursor = flat_idx;
                    self.map_hint_level = MapHintLevel::Sections(g_idx);
                    self.hint_buffer.clear();
                    return;
                }
                crate::data::HintResolveResult::Partial(_) => {
                    // Waiting for second char of group hint
                    return;
                }
                crate::data::HintResolveResult::NoMatch => {
                    // Not a group hint - fall through to per-level logic
                }
            }

            let hint_level = self.map_hint_level.clone();
            match hint_level {
                MapHintLevel::Groups => {
                    // Universal check above handles all group hints.
                    // If we reach here, typed matched no group hint - clear.
                    self.hint_buffer.clear();
                }
                MapHintLevel::Sections(g_idx) => {
                    let group_start: usize = self.data.groups.iter().take(g_idx).map(|g| g.sections.len()).sum();
                    let group_len = self.data.groups.get(g_idx).map(|g| g.sections.len()).unwrap_or(0);
                    let section_refs: Vec<&str> = folded_refs
                        .iter()
                        .skip(n_groups + group_start)
                        .take(group_len)
                        .copied()
                        .collect();
                    match crate::data::resolve_hint(&section_refs, &typed) {
                        crate::data::HintResolveResult::Exact(s_idx) => {
                            let flat_idx = group_start + s_idx;
                            self.current_idx = flat_idx;
                            self.map_cursor = flat_idx;
                            self.focus = Focus::Wizard;
                            self.map_hint_level = MapHintLevel::Groups;
                            self.hint_buffer.clear();
                        }
                        crate::data::HintResolveResult::Partial(_) => {}
                        crate::data::HintResolveResult::NoMatch => {
                            self.hint_buffer.clear();
                        }
                    }
                }
            }
        }
    }

    pub fn group_idx_for_section(&self, flat_idx: usize) -> usize {
        let mut fi = 0usize;
        for (g_idx, group) in self.data.groups.iter().enumerate() {
            for _ in 0..group.sections.len() {
                if fi == flat_idx {
                    return g_idx;
                }
                fi += 1;
            }
        }
        0
    }

    pub fn section_hint_key_idx(&self, flat_idx: usize) -> Option<usize> {
        let hints = crate::data::combined_hints(&self.data.keybindings);
        let n_groups = self.data.groups.len();
        let hint_idx = n_groups + flat_idx;
        if hint_idx < hints.len() {
            Some(hint_idx)
        } else {
            None
        }
    }

    fn update_note_scroll(&mut self) {
        let section_id = self.sections.get(self.map_cursor).map(|s| s.id.clone()).unwrap_or_default();
        self.note_scroll = crate::note::section_start_line(&self.sections, &self.section_states, &self.config.sticky_values, &self.data.groups, &self.data.boilerplate_texts, &section_id);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }

        if self.is_copy_note(&key) {
            self.modal = None;
            self.show_help = false;
            self.copy_requested = true;
            return;
        }

        if self.modal.is_some() {
            self.handle_modal_key(key);
            return;
        }

        if self.show_help {
            if self.is_help(&key) || self.is_back(&key) {
                self.show_help = false;
            }
            return;
        }

        if self.is_quit(&key) && self.focus != Focus::Map {
            let is_hint_key = if let KeyCode::Char(c) = key.code {
                let c_str = c.to_ascii_lowercase().to_string();
                crate::data::combined_hints(&self.data.keybindings).iter().any(|h| h.to_ascii_lowercase() == c_str)
            } else {
                false
            };
            if !is_hint_key {
                self.quit = true;
                return;
            }
        }

        if self.is_help(&key) {
            self.show_help = true;
            return;
        }

        if self.is_swap_panes(&key) {
            self.pane_swapped = !self.pane_swapped;
            self.config.set_swapped(self.pane_swapped);
            let _ = self.config.save(&self.data_dir);
            return;
        }

        // Focus switching: h/← moves left in layout, i/→ moves right
        // Map is always on the outside: left when default, right when swapped
        if self.is_focus_left(&key) {
            if self.focus == Focus::Wizard && !self.pane_swapped {
                // Map is to the left of wizard in default layout
                let g_idx = self.group_idx_for_section(self.current_idx);
                self.hint_buffer.clear();
                self.focus = Focus::Map;
                self.map_cursor = self.current_idx;
                self.map_hint_level = MapHintLevel::Sections(g_idx);
                self.update_note_scroll();
                return;
            } else if self.focus == Focus::Map && self.pane_swapped {
                // Map is to the right; h/← from map returns to wizard
                self.hint_buffer.clear();
                self.current_idx = self.map_cursor;
                self.focus = Focus::Wizard;
                self.map_hint_level = MapHintLevel::Groups;
                return;
            }
        }

        if self.is_focus_right(&key) {
            if self.focus == Focus::Wizard && self.pane_swapped {
                // Map is to the right of wizard in swapped layout
                let g_idx = self.group_idx_for_section(self.current_idx);
                self.hint_buffer.clear();
                self.focus = Focus::Map;
                self.map_cursor = self.current_idx;
                self.map_hint_level = MapHintLevel::Sections(g_idx);
                self.update_note_scroll();
                return;
            } else if self.focus == Focus::Map && !self.pane_swapped {
                // Map is to the left; i/→ from map returns to wizard
                self.hint_buffer.clear();
                self.current_idx = self.map_cursor;
                self.focus = Focus::Wizard;
                self.map_hint_level = MapHintLevel::Groups;
                return;
            }
        }

        // Map focus: all navigation goes to the map handler
        if self.focus == Focus::Map {
            self.handle_map_key(key);
            return;
        }

        // Top-level Esc goes back a section (when not in a sub-context)
        if self.is_back(&key) && self.section_at_top_level() {
            self.go_back_section();
            return;
        }

        let idx = self.current_idx;
        let state = self.section_states.get_mut(idx);
        if state.is_none() {
            return;
        }

        match self.section_states[idx].clone() {
            SectionState::Header(_) => self.handle_header_key(key),
            SectionState::FreeText(_) => self.handle_free_text_key(key),
            SectionState::ListSelect(_) => self.handle_list_select_key(key),
            SectionState::BlockSelect(_) => self.handle_block_select_key(key),
            SectionState::Checklist(_) => self.handle_checklist_key(key),
            SectionState::Pending => {
                if self.is_confirm(&key) || self.is_navigate_down(&key) {
                    self.advance_section();
                }
            }
        }
    }

    fn advance_section(&mut self) {
        if self.current_idx + 1 < self.sections.len() {
            self.current_idx += 1;
        } else {
            self.note_completed = true;
        }
    }

    fn go_back_section(&mut self) {
        if self.current_idx > 0 {
            self.current_idx -= 1;
        }
    }

    fn handle_header_key(&mut self, key: KeyEvent) {
        // Hint key handling: group/section hint → return to map, field hints → jump to field
        if let KeyCode::Char(c) = key.code {
            let hints = crate::data::combined_hints(&self.data.keybindings);
            let case_sensitive = self.config.hint_labels_case_sensitive;
            let ch_str: String = if case_sensitive { c.to_string() } else { c.to_ascii_lowercase().to_string() };
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();

            let folded_hints: Vec<String> = hints.iter().map(|h| {
                if case_sensitive { h.to_string() } else { h.to_ascii_lowercase().to_string() }
            }).collect();
            let folded_refs: Vec<&str> = folded_hints.iter().map(String::as_str).collect();

            let g_idx = self.group_idx_for_section(self.current_idx);

            // Group hint
            let group_hint = folded_refs.get(g_idx).copied().unwrap_or("");
            match crate::data::resolve_hint(&[group_hint], &typed) {
                crate::data::HintResolveResult::Exact(_) => {
                    self.focus = Focus::Map;
                    self.map_cursor = self.current_idx;
                    self.map_hint_level = MapHintLevel::Groups;
                    self.hint_buffer.clear();
                    return;
                }
                crate::data::HintResolveResult::Partial(_) => return, // hold buffer
                crate::data::HintResolveResult::NoMatch => {}
            }

            // Section hint
            if let Some(shi) = self.section_hint_key_idx(self.current_idx) {
                let section_hint = folded_refs.get(shi).copied().unwrap_or("");
                match crate::data::resolve_hint(&[section_hint], &typed) {
                    crate::data::HintResolveResult::Exact(_) => {
                        self.focus = Focus::Map;
                        self.map_cursor = self.current_idx;
                        self.map_hint_level = MapHintLevel::Sections(g_idx);
                        self.hint_buffer.clear();
                        return;
                    }
                    crate::data::HintResolveResult::Partial(_) => return, // hold buffer
                    crate::data::HintResolveResult::NoMatch => {}
                }

                // Field hints: exclude section hint index and group hint index
                let field_hint_indices: Vec<usize> = (0..hints.len())
                    .filter(|&i| i != shi && i != g_idx)
                    .collect();
                let idx = self.current_idx;
                let n_fields = match self.section_states.get(idx) {
                    Some(SectionState::Header(s)) => s.field_configs.len(),
                    _ => 0,
                };
                // Build the candidate list for field hints and resolve
                let field_refs: Vec<&str> = field_hint_indices.iter()
                    .take(n_fields)
                    .filter_map(|&hi| folded_refs.get(hi).copied())
                    .collect();
                match crate::data::resolve_hint(&field_refs, &typed) {
                    crate::data::HintResolveResult::Exact(f_idx) => {
                        if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                            s.field_index = f_idx;
                            s.completed = false;
                        }
                        self.hint_buffer.clear();
                        self.open_header_modal();
                        return;
                    }
                    crate::data::HintResolveResult::Partial(_) => return, // hold buffer
                    crate::data::HintResolveResult::NoMatch => {
                        self.hint_buffer.clear();
                    }
                }
            } else {
                // No section hint index found - clear buffer on no match
                self.hint_buffer.clear();
            }
        }

        if self.is_super_confirm(&key) {
            let idx = self.current_idx;
            let resolved = if let Some(SectionState::Header(s)) = self.section_states.get(idx) {
                s.field_configs.get(s.field_index).map(|cfg| {
                    let confirmed = s.repeated_values.get(s.field_index)
                        .and_then(|v| v.last())
                        .map(|v| v.as_str())
                        .unwrap_or("");
                    crate::sections::multi_field::resolve_multifield_value(
                        confirmed,
                        cfg,
                        &self.config.sticky_values,
                    )
                })
            } else {
                None
            };
            if let Some(crate::sections::multi_field::ResolvedMultiFieldValue::Complete(value)) = resolved {
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.set_current_value(value);
                    let done = s.advance();
                    if done {
                        self.advance_section();
                    }
                }
            }
            return;
        }

        if self.is_back(&key) || self.is_navigate_up(&key) {
            let idx = self.current_idx;
            let went_back = if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                // Normalize out-of-bounds index before going back
                if s.field_index >= s.field_configs.len() && !s.field_configs.is_empty() {
                    s.field_index = s.field_configs.len() - 1;
                    s.repeat_counts[s.field_index] = 0;
                    if let Some(slot) = s.repeated_values.get_mut(s.field_index) {
                        slot.clear();
                    }
                    s.completed = false;
                    true
                } else {
                    s.go_back()
                }
            } else {
                false
            };
            if !went_back {
                self.go_back_section();
            }
            return;
        }

        if self.is_navigate_down(&key) {
            let idx = self.current_idx;
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                let last = s.field_configs.len().saturating_sub(1);
                if s.field_index > last {
                    // Normalize out-of-bounds (completed) index to last field
                    s.field_index = last;
                } else if s.field_index < last {
                    s.field_index += 1;
                }
            }
            return;
        }

        if key.code == KeyCode::Enter {
            self.open_header_modal();
        }
    }

    fn open_header_modal(&mut self) {
        let idx = self.current_idx;
        let field_idx = if let Some(SectionState::Header(s)) = self.section_states.get(idx) {
            s.field_index
        } else {
            return;
        };
        let field_cfg = if let Some(SectionState::Header(s)) = self.section_states.get(idx) {
            s.field_configs.get(field_idx).cloned()
        } else {
            None
        };
        if let Some(cfg) = field_cfg {
            let window_size = crate::data::combined_hints(&self.data.keybindings).len();
            let modal = if let Some(composite) = cfg.composite {
                SearchModal::new_composite(field_idx, cfg.id, composite, &self.config.sticky_values, window_size)
            } else if !cfg.options.is_empty() {
                let mut m = SearchModal::new_simple(field_idx, cfg.id, cfg.options.clone(), window_size);
                if let Some(ref default) = cfg.default {
                    if let Some(pos) = cfg.options.iter().position(|e| e == default) {
                        m.list_cursor = pos;
                        m.sticky_cursor = pos;
                        m.center_scroll();
                    }
                }
                m
            } else {
                return;
            };
            self.modal = Some(modal);
        }
    }

    fn handle_modal_key(&mut self, key: KeyEvent) {
        let hints = crate::data::combined_hints(&self.data.keybindings);

        if key.code == KeyCode::Esc {
            self.modal = None;
            return;
        }

        if self.is_super_confirm(&key) {
            let value = {
                let modal = self.modal.as_ref().unwrap();
                let q = modal.query.trim().to_string();
                if !q.is_empty() {
                    Some(q)
                } else {
                    modal.selected_value().map(String::from)
                }
            };
            match value {
                Some(v) => self.confirm_modal_value(v),
                None => { self.modal = None; }
            }
            return;
        }

        let focus = match &self.modal {
            Some(m) => m.focus.clone(),
            None => return,
        };

        match focus {
            ModalFocus::SearchBar => match key.code {
                KeyCode::Tab => {
                    let query = self.modal.as_ref().unwrap().query.trim().to_string();
                    if !query.is_empty() {
                        self.confirm_modal_value(query);
                    }
                }
                KeyCode::Enter => {
                    if !self.modal.as_ref().unwrap().filtered.is_empty() {
                        self.modal.as_mut().unwrap().focus = ModalFocus::List;
                    }
                }
                KeyCode::Backspace => {
                    let modal = self.modal.as_mut().unwrap();
                    modal.query.pop();
                    modal.update_filter();
                    if modal.query.is_empty() {
                        modal.center_scroll();
                    }
                }
                KeyCode::Char(c) => {
                    let modal = self.modal.as_mut().unwrap();
                    modal.query.push(c);
                    modal.update_filter();
                }
                _ => {}
            },
            ModalFocus::List => match key.code {
                KeyCode::Backspace => {
                    let can_go_back = self.modal.as_ref().map(|m| {
                        m.composite.as_ref().map(|c| c.part_idx > 0).unwrap_or(false)
                    }).unwrap_or(false);
                    if can_go_back {
                        self.composite_go_back();
                    } else {
                        // First part or simple field: exit modal, return to wizard
                        self.modal = None;
                    }
                }
                KeyCode::Char(' ') => {
                    self.modal.as_mut().unwrap().focus = ModalFocus::SearchBar;
                }
                KeyCode::Enter => {
                    if let Some(val) = self.modal.as_ref().unwrap().selected_value().map(String::from) {
                        self.confirm_modal_value(val);
                    }
                }
                KeyCode::Up => {
                    let modal = self.modal.as_mut().unwrap();
                    if modal.list_cursor > 0 {
                        modal.list_cursor -= 1;
                        modal.update_scroll();
                    }
                }
                KeyCode::Down => {
                    let modal = self.modal.as_mut().unwrap();
                    if modal.list_cursor + 1 < modal.filtered.len() {
                        modal.list_cursor += 1;
                        modal.update_scroll();
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(hint_pos) = hints.iter().position(|h| *h == c.to_string().as_str()) {
                        if let Some(val) = self.modal.as_ref().unwrap().hint_value(hint_pos).map(String::from) {
                            self.confirm_modal_value(val);
                        }
                    }
                }
                _ => {}
            },
        }
    }

    fn confirm_modal_value(&mut self, value: String) {
        let idx = self.current_idx;
        let is_composite = self.modal.as_ref().map(|m| m.composite.is_some()).unwrap_or(false);

        if is_composite {
            let advance = self.modal.as_mut().unwrap()
                .advance_composite(value, &mut self.config.sticky_values);
            match advance {
                CompositeAdvance::NextPart => {
                    let preview = compute_composite_preview(self.modal.as_ref().unwrap());
                    let spans = compute_composite_spans(self.modal.as_ref().unwrap());
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        s.set_preview_value(preview);
                        s.composite_spans = Some(spans);
                    }
                    let _ = self.config.save(&self.data_dir);
                }
                CompositeAdvance::Complete(final_value) => {
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        s.composite_spans = None;
                        if let Some(slot) = s.repeated_values.get_mut(s.field_index) {
                            slot.clear();
                        }
                        s.set_current_value(final_value);
                        let done = s.advance();
                        if done {
                            self.advance_section();
                        }
                    }
                    self.modal = None;
                    let _ = self.config.save(&self.data_dir);
                }
            }
        } else {
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.set_current_value(value);
                let done = s.advance();
                if done {
                    self.advance_section();
                }
            }
            self.modal = None;
        }
    }

    fn handle_free_text_key(&mut self, key: KeyEvent) {
        let idx = self.current_idx;
        let is_editing = match &self.section_states[idx] {
            SectionState::FreeText(s) => s.is_editing(),
            _ => false,
        };

        if is_editing {
            if self.is_back(&key) {
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.cancel_entry();
                }
                return;
            }
            // In text input: only Enter confirms, not letter aliases like 't'
            if key.code == KeyCode::Enter {
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.commit_entry();
                }
                return;
            }
            if key.code == KeyCode::Backspace {
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.handle_backspace();
                }
                return;
            }
            if let KeyCode::Char(c) = key.code {
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.handle_char(c);
                }
            }
            return;
        }

        // Browsing mode
        if self.try_navigate_to_map_via_hint(&key) {
            return;
        }

        if self.is_navigate_up(&key) {
            if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                s.navigate_up();
            }
            return;
        }

        if self.is_navigate_down(&key) {
            if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                s.navigate_down();
            }
            return;
        }

        if self.is_add_entry(&key) {
            if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                s.start_new_entry();
            }
            return;
        }

        if self.is_confirm(&key) {
            let has_entries = match &self.section_states[idx] {
                SectionState::FreeText(s) => !s.entries.is_empty(),
                _ => false,
            };
            if has_entries {
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.completed = true;
                }
                self.advance_section();
            } else {
                // Empty = skip
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.skipped = true;
                }
                self.advance_section();
            }
        }
    }

    fn handle_list_select_key(&mut self, key: KeyEvent) {
        let idx = self.current_idx;
        let mode = match &self.section_states[idx] {
            SectionState::ListSelect(s) => {
                match s.mode {
                    ListSelectMode::Browsing => 0,
                    ListSelectMode::AddingLabel => 1,
                    ListSelectMode::AddingOutput => 2,
                }
            }
            _ => return,
        };

        match mode {
            1 | 2 => {
                // Adding mode
                if self.is_back(&key) {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.cancel_add();
                    }
                    return;
                }
                // In text input: only Enter confirms, not letter aliases like 't'
                if key.code == KeyCode::Enter {
                    if mode == 1 {
                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.confirm_label();
                        }
                    } else {
                        let new_entry = if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.confirm_output()
                        } else {
                            None
                        };
                        if let Some(entry) = new_entry {
                            let data_file = self.sections[idx].data_file.clone();
                            if let Some(ref df) = data_file {
                                match self.data.append_list_entry(df, entry.clone()) {
                                    Ok(_) => {
                                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                                            s.entries = self.data.list_data.get(df).cloned().unwrap_or_default();
                                            // Select the newly added entry
                                            let new_idx = s.entries.len().saturating_sub(1);
                                            s.cursor = new_idx;
                                            s.selected_indices.push(new_idx);
                                        }
                                        self.status = Some(StatusMsg::success("Entry added."));
                                    }
                                    Err(e) => {
                                        self.status = Some(StatusMsg::error(format!("Failed to save: {}", e)));
                                    }
                                }
                            }
                        }
                    }
                    return;
                }
                if key.code == KeyCode::Backspace {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.handle_backspace();
                    }
                    return;
                }
                if let KeyCode::Char(c) = key.code {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.handle_char(c);
                    }
                }
            }
            _ => {
                // Browsing mode
                if self.try_navigate_to_map_via_hint(&key) {
                    return;
                }
                if self.is_navigate_up(&key) {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.navigate_up();
                    }
                    return;
                }
                if self.is_navigate_down(&key) {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.navigate_down();
                    }
                    return;
                }
                if self.is_select(&key) {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.toggle_current();
                    }
                    return;
                }
                if self.is_add_entry(&key) {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.start_add_label();
                    }
                    return;
                }
                if self.is_confirm(&key) {
                    let has_selection = match &self.section_states[idx] {
                        SectionState::ListSelect(s) => !s.selected_indices.is_empty(),
                        _ => false,
                    };
                    if has_selection {
                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.completed = true;
                        }
                        self.advance_section();
                    } else {
                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.skipped = true;
                        }
                        self.advance_section();
                    }
                }
            }
        }
    }

    fn handle_block_select_key(&mut self, key: KeyEvent) {
        let idx = self.current_idx;
        let in_items = match &self.section_states[idx] {
            SectionState::BlockSelect(s) => s.in_items(),
            _ => false,
        };

        if in_items {
            if self.is_back(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.exit_items();
                }
                return;
            }
            if self.is_navigate_up(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.navigate_up();
                }
                return;
            }
            if self.is_navigate_down(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.navigate_down();
                }
                return;
            }
            if self.is_select(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.toggle_item();
                }
                return;
            }
            if self.is_confirm(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.exit_items();
                }
            }
        } else {
            // Group list
            if self.try_navigate_to_map_via_hint(&key) {
                return;
            }
            if self.is_navigate_up(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.navigate_up();
                }
                return;
            }
            if self.is_navigate_down(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.navigate_down();
                }
                return;
            }
            if self.is_confirm(&key) || self.is_select(&key) {
                // Enter group to select items
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    if !s.groups.is_empty() {
                        s.enter_group();
                    }
                }
                return;
            }
            if self.is_add_entry(&key) {
                // Confirm all and advance
                let has_any = match &self.section_states[idx] {
                    SectionState::BlockSelect(s) => s.groups.iter().any(|r| r.has_selection()),
                    _ => false,
                };
                if has_any {
                    if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                        s.completed = true;
                    }
                    self.advance_section();
                } else {
                    if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                        s.skipped = true;
                    }
                    self.advance_section();
                }
                return;
            }
            if self.is_back(&key) {
                // Confirm and advance if any selections
                let has_any = match &self.section_states[idx] {
                    SectionState::BlockSelect(s) => s.groups.iter().any(|r| r.has_selection()),
                    _ => false,
                };
                if has_any {
                    if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                        s.completed = true;
                    }
                } else {
                    if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                        s.skipped = true;
                    }
                }
                self.advance_section();
            }
        }
    }

    fn handle_checklist_key(&mut self, key: KeyEvent) {
        let idx = self.current_idx;

        if self.try_navigate_to_map_via_hint(&key) {
            return;
        }

        if self.is_navigate_up(&key) {
            if let SectionState::Checklist(s) = &mut self.section_states[idx] {
                s.navigate_up();
            }
            return;
        }
        if self.is_navigate_down(&key) {
            if let SectionState::Checklist(s) = &mut self.section_states[idx] {
                s.navigate_down();
            }
            return;
        }
        if self.is_select(&key) {
            if let SectionState::Checklist(s) = &mut self.section_states[idx] {
                s.toggle_current();
            }
            return;
        }
        if self.is_confirm(&key) {
            if let SectionState::Checklist(s) = &mut self.section_states[idx] {
                s.completed = true;
            }
            self.advance_section();
        }
    }

    pub fn section_is_completed(&self, idx: usize) -> bool {
        match self.section_states.get(idx) {
            Some(SectionState::Header(s)) => s.completed,
            Some(SectionState::FreeText(s)) => s.completed,
            Some(SectionState::ListSelect(s)) => s.completed,
            Some(SectionState::BlockSelect(s)) => s.completed,
            Some(SectionState::Checklist(s)) => s.completed,
            _ => false,
        }
    }

    pub fn section_is_skipped(&self, idx: usize) -> bool {
        match self.section_states.get(idx) {
            Some(SectionState::FreeText(s)) => s.skipped,
            Some(SectionState::ListSelect(s)) => s.skipped,
            Some(SectionState::BlockSelect(s)) => s.skipped,
            Some(SectionState::Checklist(s)) => s.skipped,
            _ => false,
        }
    }

    /// If the key matches the current section's hint key, switch focus to Map at Sections level.
    /// Returns true if navigation happened.
    fn try_navigate_to_map_via_hint(&mut self, key: &KeyEvent) -> bool {
        if let KeyCode::Char(c) = key.code {
            let hints = crate::data::combined_hints(&self.data.keybindings);
            let case_sensitive = self.config.hint_labels_case_sensitive;
            let ch_str: String = if case_sensitive { c.to_string() } else { c.to_ascii_lowercase().to_string() };
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();

            let folded_hints: Vec<String> = hints.iter().map(|h| {
                if case_sensitive { h.to_string() } else { h.to_ascii_lowercase().to_string() }
            }).collect();
            let folded_refs: Vec<&str> = folded_hints.iter().map(String::as_str).collect();

            let g_idx = self.group_idx_for_section(self.current_idx);

            // Check group hint
            let group_hint = folded_refs.get(g_idx).copied().unwrap_or("");
            match crate::data::resolve_hint(&[group_hint], &typed) {
                crate::data::HintResolveResult::Exact(_) => {
                    self.focus = Focus::Map;
                    self.map_cursor = self.current_idx;
                    self.map_hint_level = MapHintLevel::Groups;
                    self.hint_buffer.clear();
                    return true;
                }
                crate::data::HintResolveResult::Partial(_) => return false, // hold buffer
                crate::data::HintResolveResult::NoMatch => {}
            }

            // Check section hint
            if let Some(shi) = self.section_hint_key_idx(self.current_idx) {
                let section_hint = folded_refs.get(shi).copied().unwrap_or("");
                match crate::data::resolve_hint(&[section_hint], &typed) {
                    crate::data::HintResolveResult::Exact(_) => {
                        self.focus = Focus::Map;
                        self.map_cursor = self.current_idx;
                        self.map_hint_level = MapHintLevel::Sections(g_idx);
                        self.hint_buffer.clear();
                        return true;
                    }
                    crate::data::HintResolveResult::Partial(_) => return false, // hold buffer
                    crate::data::HintResolveResult::NoMatch => {}
                }
            }

            // No hint matched
            self.hint_buffer.clear();
        }
        false
    }

    fn composite_go_back(&mut self) {
        let idx = self.current_idx;

        // Step 1: Pop the last confirmed value and decrement part_idx, extract new part options
        let (new_part_idx, popped_output, new_labels, new_outputs, new_default_cursor) = {
            let modal = match self.modal.as_mut() {
                Some(m) => m,
                None => return,
            };
            let comp = match modal.composite.as_mut() {
                Some(c) => c,
                None => return,
            };
            if comp.part_idx == 0 {
                return;
            }
            let popped = comp.values.pop();
            comp.part_idx -= 1;
            let part = &comp.config.parts[comp.part_idx];
            let labels: Vec<String> = part.options.iter().map(|o| o.label().to_string()).collect();
            let outputs: Vec<String> = part.options.iter().map(|o| o.output().to_string()).collect();
            let dc = part.default_cursor();
            (comp.part_idx, popped, labels, outputs, dc)
        };

        // Step 2: Update modal entries and cursor (restore to previously chosen value)
        let cursor = popped_output
            .as_ref()
            .and_then(|v| new_outputs.iter().position(|e| e == v))
            .unwrap_or(new_default_cursor);
        {
            let modal = self.modal.as_mut().unwrap();
            modal.all_entries = new_labels;
            modal.all_outputs = new_outputs;
            modal.list_cursor = cursor;
            modal.sticky_cursor = cursor;
            modal.query = String::new();
            modal.list_scroll = 0;
            modal.focus = ModalFocus::List;
            modal.update_filter();
        }

        // Step 3: Update header state spans/value
        if new_part_idx == 0 {
            // Back to first part - clear partial state (preload will show via render)
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.composite_spans = None;
                if let Some(slot) = s.repeated_values.get_mut(s.field_index) {
                    slot.clear();
                }
            }
        } else {
            let (preview, spans) = {
                let modal = self.modal.as_ref().unwrap();
                (compute_composite_preview(modal), compute_composite_spans(modal))
            };
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.set_preview_value(preview);
                s.composite_spans = Some(spans);
            }
        }
    }
}

fn compute_composite_spans(modal: &crate::modal::SearchModal) -> Vec<(String, bool)> {
    let comp = match &modal.composite {
        Some(c) => c,
        None => return vec![],
    };
    let format = &comp.config.format;
    let mut spans: Vec<(String, bool)> = Vec::new();
    let mut literal = String::new();
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if !literal.is_empty() {
                spans.push((literal.clone(), false));
                literal.clear();
            }
            let mut id = String::new();
            for c2 in chars.by_ref() {
                if c2 == '}' { break; }
                id.push(c2);
            }
            if let Some(i) = comp.config.parts.iter().position(|p| p.id == id) {
                if i < comp.values.len() {
                    spans.push((comp.values[i].clone(), true));
                } else {
                    let preview = comp.config.parts[i].preview.as_deref().unwrap_or("?");
                    spans.push((preview.to_string(), false));
                }
            }
        } else {
            literal.push(c);
        }
    }
    if !literal.is_empty() {
        spans.push((literal, false));
    }
    spans
}

fn compute_composite_preview(modal: &crate::modal::SearchModal) -> String {
    let comp = match &modal.composite {
        Some(c) => c,
        None => return String::new(),
    };
    let mut result = comp.config.format.clone();
    for (i, val) in comp.values.iter().enumerate() {
        let placeholder = format!("{{{}}}", comp.config.parts[i].id);
        result = result.replace(&placeholder, val);
    }
    for part in &comp.config.parts[comp.part_idx..] {
        let placeholder = format!("{{{}}}", part.id);
        let preview_str = part.preview.as_deref().unwrap_or("?");
        result = result.replace(&placeholder, preview_str);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn matches_key_shift_enter_binding_recognized() {
        // match_binding_str is the pub free-function extracted from matches_key in the implementation.
        // This test will fail to compile until that function is added and handles "shift+enter".
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        assert!(
            match_binding_str("shift+enter", &key),
            "match_binding_str(\"shift+enter\", Enter+SHIFT) should return true"
        );
    }

    #[test]
    fn matches_key_shift_enter_does_not_match_plain_enter() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(
            !match_binding_str("shift+enter", &key),
            "match_binding_str(\"shift+enter\", plain Enter) should return false"
        );
    }

    #[test]
    fn matches_key_super_confirm_binding_in_keybindings() {
        let kb = crate::data::KeyBindings::default();
        // Verify that the default super_confirm binding contains "shift+enter"
        assert!(
            kb.super_confirm.iter().any(|b| b == "shift+enter"),
            "KeyBindings::default().super_confirm should contain \"shift+enter\""
        );
    }

    #[test]
    fn super_confirm_fills_default_and_advances() {
        use crate::data::{AppData, HeaderFieldConfig, KeyBindings, SectionConfig, SectionGroup};
        use crate::config::Config;
        use std::path::PathBuf;

        let fields = vec![
            HeaderFieldConfig { id: "f1".to_string(), name: "F1".to_string(), options: vec![], composite: None, default: Some("hello".to_string()), repeat_limit: None },
            HeaderFieldConfig { id: "f2".to_string(), name: "F2".to_string(), options: vec![], composite: None, default: None, repeat_limit: None },
        ];
        let section = SectionConfig {
            id: "s1".to_string(), name: "S1".to_string(), map_label: "S1".to_string(),
            section_type: "multi_field".to_string(), data_file: None, date_prefix: None,
            options: vec![], composite: None, fields: Some(fields),
            is_intake: false, heading_search_text: None, heading_label: None, note_render_slot: None,
        };
        let group = SectionGroup { id: "g1".to_string(), num: None, name: "G1".to_string(), sections: vec![section.clone()] };
        let data = AppData {
            groups: vec![group], sections: vec![section],
            list_data: Default::default(), checklist_data: Default::default(),
            block_select_data: Default::default(), boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };
        let mut app = App::new(data, Config::default(), PathBuf::new());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        app.handle_header_key(key);
        if let Some(SectionState::Header(s)) = app.section_states.get(0) {
            assert_eq!(s.repeated_values[0].last().map(|s| s.as_str()).unwrap_or(""), "hello", "field 0 should be filled with its default");
            assert_eq!(s.field_index, 1, "field_index should advance to 1");
        } else {
            panic!("expected Header state at index 0");
        }
    }

    #[test]
    fn super_confirm_no_op_when_no_default() {
        use crate::data::{AppData, HeaderFieldConfig, KeyBindings, SectionConfig, SectionGroup};
        use crate::config::Config;
        use std::path::PathBuf;

        let fields = vec![
            HeaderFieldConfig { id: "f1".to_string(), name: "F1".to_string(), options: vec![], composite: None, default: None, repeat_limit: None },
        ];
        let section = SectionConfig {
            id: "s1".to_string(), name: "S1".to_string(), map_label: "S1".to_string(),
            section_type: "multi_field".to_string(), data_file: None, date_prefix: None,
            options: vec![], composite: None, fields: Some(fields),
            is_intake: false, heading_search_text: None, heading_label: None, note_render_slot: None,
        };
        let group = SectionGroup { id: "g1".to_string(), num: None, name: "G1".to_string(), sections: vec![section.clone()] };
        let data = AppData {
            groups: vec![group], sections: vec![section],
            list_data: Default::default(), checklist_data: Default::default(),
            block_select_data: Default::default(), boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };
        let mut app = App::new(data, Config::default(), PathBuf::new());
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        app.handle_header_key(key);
        if let Some(SectionState::Header(s)) = app.section_states.get(0) {
            assert_eq!(s.field_index, 0, "field_index should stay at 0 when no default");
        } else {
            panic!("expected Header state at index 0");
        }
    }
}
