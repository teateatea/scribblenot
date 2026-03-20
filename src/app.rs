use crate::config::Config;
use crate::data::{AppData, SectionConfig};
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
    pub data_dir: PathBuf,
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
            data_dir,
        }
    }

    fn init_states(sections: &[SectionConfig], data: &AppData) -> Vec<SectionState> {
        sections
            .iter()
            .map(|cfg| match cfg.section_type.as_str() {
                "header" => SectionState::Header(HeaderState::new()),
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
                    let regions = cfg
                        .data_file
                        .as_ref()
                        .and_then(|f| data.region_data.get(f))
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
            let matched = match binding.as_str() {
                "down" => key.code == KeyCode::Down,
                "up" => key.code == KeyCode::Up,
                "enter" => key.code == KeyCode::Enter,
                "esc" => key.code == KeyCode::Esc,
                "space" => key.code == KeyCode::Char(' '),
                "backspace" => key.code == KeyCode::Backspace,
                s if s.len() == 1 => {
                    let c = s.chars().next().unwrap();
                    key.code == KeyCode::Char(c)
                }
                _ => false,
            };
            if matched {
                return true;
            }
        }
        false
    }

    fn is_navigate_down(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.navigate_down.clone())
    }

    fn is_navigate_up(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.navigate_up.clone())
    }

    fn is_select(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.select.clone())
    }

    fn is_confirm(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.confirm.clone())
    }

    fn is_add_entry(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.add_entry.clone())
    }

    fn is_back(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.back.clone())
    }

    fn is_swap_panes(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.swap_panes.clone())
    }

    fn is_help(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.help.clone())
    }

    fn is_quit(&self, key: &KeyEvent) -> bool {
        self.matches_key(key, &self.data.keybindings.quit.clone())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }

        if self.show_help {
            if self.is_help(&key) || self.is_back(&key) {
                self.show_help = false;
            }
            return;
        }

        if self.is_quit(&key) {
            self.quit = true;
            return;
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

    fn handle_header_key(&mut self, key: KeyEvent) {
        let is_back = self.is_back(&key);
        let is_confirm = self.is_confirm(&key);

        let state = match self.section_states.get_mut(self.current_idx) {
            Some(SectionState::Header(s)) => s,
            _ => return,
        };

        if is_back {
            if state.field_index > 0 {
                state.field_index -= 1;
                state.edit_buf = state.current_field_value().to_string();
            }
            return;
        }

        if is_confirm {
            let done = state.confirm_field();
            if done {
                let _ = state;
                self.advance_section();
            }
            return;
        }

        if key.code == KeyCode::Backspace {
            state.handle_backspace();
            return;
        }

        if let KeyCode::Char(c) = key.code {
            state.handle_char(c);
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
            if self.is_confirm(&key) {
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
            return;
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
                if self.is_confirm(&key) {
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
                return;
            }
            _ => {
                // Browsing mode
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
                    return;
                }
            }
        }
    }

    fn handle_block_select_key(&mut self, key: KeyEvent) {
        let idx = self.current_idx;
        let in_techniques = match &self.section_states[idx] {
            SectionState::BlockSelect(s) => s.in_techniques(),
            _ => false,
        };

        if in_techniques {
            if self.is_back(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.exit_techniques();
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
                    s.toggle_technique();
                }
                return;
            }
            if self.is_confirm(&key) {
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    s.exit_techniques();
                }
                return;
            }
        } else {
            // Region list
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
                // Enter region to select techniques
                if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                    if !s.regions.is_empty() {
                        s.enter_region();
                    }
                }
                return;
            }
            if self.is_add_entry(&key) {
                // Confirm all and advance
                let has_any = match &self.section_states[idx] {
                    SectionState::BlockSelect(s) => s.regions.iter().any(|r| r.has_selection()),
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
                    SectionState::BlockSelect(s) => s.regions.iter().any(|r| r.has_selection()),
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
                return;
            }
        }
    }

    fn handle_checklist_key(&mut self, key: KeyEvent) {
        let idx = self.current_idx;

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
}
