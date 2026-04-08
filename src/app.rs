use crate::config::Config;
use crate::data::{AppData, SectionConfig};
use crate::document::build_initial_document;
use crate::modal::{
    joined_repeating_value, modal_height_for_viewport, modal_window_size_for_height,
    resolved_item_labels_for_list, FieldAdvance, ModalFocus, SearchModal,
};
use crate::sections::{
    block_select::BlockSelectState,
    checklist::ChecklistState,
    free_text::FreeTextState,
    header::HeaderState,
    list_select::{ListSelectMode, ListSelectState},
};
use iced::keyboard::{key::Named, Key, Modifiers};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppKey {
    Char(char),
    CtrlChar(char),
    CtrlC,
    Enter,
    ShiftEnter,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Backspace,
    Tab,
    Space,
    Ignored,
}

pub fn appkey_from_iced(key: Key, modifiers: Modifiers) -> AppKey {
    match key {
        Key::Named(Named::Enter) => {
            if modifiers.contains(Modifiers::SHIFT) {
                AppKey::ShiftEnter
            } else {
                AppKey::Enter
            }
        }
        Key::Named(Named::Escape) => AppKey::Esc,
        Key::Named(Named::Backspace) => AppKey::Backspace,
        Key::Named(Named::Tab) => AppKey::Tab,
        Key::Named(Named::Space) => AppKey::Space,
        Key::Named(Named::ArrowDown) => AppKey::Down,
        Key::Named(Named::ArrowUp) => AppKey::Up,
        Key::Named(Named::ArrowLeft) => AppKey::Left,
        Key::Named(Named::ArrowRight) => AppKey::Right,
        Key::Character(ref s) => {
            let c = s.chars().next().unwrap_or('\0');
            if modifiers.contains(Modifiers::CTRL) {
                if c == 'c' {
                    AppKey::CtrlC
                } else {
                    AppKey::CtrlChar(c)
                }
            } else if c == ' ' {
                AppKey::Space
            } else {
                AppKey::Char(c)
            }
        }
        _ => AppKey::Ignored,
    }
}
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
    pub ui_theme: crate::theme::AppTheme,
    pub pane_swapped: bool,
    pub show_help: bool,
    pub status: Option<StatusMsg>,
    pub quit: bool,
    pub copy_requested: bool,
    pub copy_flash_until: Option<Instant>,
    pub data_dir: PathBuf,
    pub focus: Focus,
    pub map_cursor: usize,
    pub map_return_idx: Option<usize>,
    pub map_hint_level: MapHintLevel,
    pub note_scroll: u16,
    pub modal: Option<SearchModal>,
    pub hint_buffer: String,
    pub editable_note: String,
    pub note_headings_valid: bool,
    pub note_structure_warning: Option<String>,
    pub viewport_size: Option<iced::Size>,
}

pub fn match_binding_str(binding: &str, key: &AppKey) -> bool {
    match binding {
        "down" => matches!(key, AppKey::Down),
        "up" => matches!(key, AppKey::Up),
        "left" => matches!(key, AppKey::Left),
        "right" => matches!(key, AppKey::Right),
        "enter" => matches!(key, AppKey::Enter),
        "esc" => matches!(key, AppKey::Esc),
        "space" => matches!(key, AppKey::Space),
        "backspace" => matches!(key, AppKey::Backspace),
        "shift+enter" => matches!(key, AppKey::ShiftEnter),
        s if s
            .strip_prefix("ctrl+")
            .is_some_and(|suffix| suffix.len() == 1) =>
        {
            let c = s.strip_prefix("ctrl+").unwrap().chars().next().unwrap();
            matches!(key, AppKey::CtrlChar(k) if *k == c)
                || (c == 'c' && matches!(key, AppKey::CtrlC))
        }
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            matches!(key, AppKey::Char(k) if *k == c)
        }
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct MapHintLabels {
    pub sections: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WizardHintLabels {
    pub fields: Vec<String>,
}

impl App {
    pub fn new(data: AppData, config: Config, data_dir: PathBuf) -> Self {
        let sections = data.sections.clone();
        let section_states = Self::init_states(&sections, &data);
        let pane_swapped = config.is_swapped();
        let editable_note = build_initial_document(
            &sections,
            &section_states,
            &config.sticky_values,
            &data.boilerplate_texts,
        );
        let note_validation =
            crate::document::validate_document_structure(&editable_note, &sections);
        let note_headings_valid = note_validation.is_ok();
        let note_structure_warning = note_validation.err();
        let ui_theme =
            crate::theme::AppTheme::load(&data_dir, &config.theme).unwrap_or_else(|err| {
                eprintln!("Warning: failed to load theme '{}': {err}", config.theme);
                crate::theme::AppTheme::default()
            });
        Self {
            sections,
            section_states,
            current_idx: 0,
            data,
            config,
            ui_theme,
            pane_swapped,
            show_help: false,
            status: None,
            quit: false,
            copy_requested: false,
            copy_flash_until: None,
            data_dir,
            focus: Focus::Wizard,
            map_cursor: 0,
            map_return_idx: None,
            map_hint_level: MapHintLevel::Groups,
            note_scroll: 0,
            modal: None,
            hint_buffer: String::new(),
            editable_note,
            note_headings_valid,
            note_structure_warning,
            viewport_size: None,
        }
    }

    pub fn set_viewport_size(&mut self, size: iced::Size) {
        self.viewport_size = Some(size);
        let window_size = self.modal_window_size();
        if let Some(modal) = self.modal.as_mut() {
            modal.window_size = window_size;
            modal.update_scroll();
        }
    }

    pub fn modal_window_size(&self) -> usize {
        let modal_height =
            modal_height_for_viewport(self.viewport_size.map(|size| size.height), 360.0);
        modal_window_size_for_height(modal_height, self.data.keybindings.hints.len())
    }

    pub fn set_editable_note(&mut self, new_text: String) {
        self.editable_note = new_text;
        self.refresh_note_structure();
    }

    pub fn set_modal_query(&mut self, new_text: String) {
        if let Some(modal) = self.modal.as_mut() {
            modal.query = new_text;
            modal.update_filter();
        }
    }

    pub fn select_modal_filtered_index(&mut self, filtered_index: usize) {
        let value = {
            let Some(modal) = self.modal.as_mut() else {
                return;
            };
            if filtered_index >= modal.filtered.len() {
                return;
            }
            modal.list_cursor = filtered_index;
            modal.update_scroll();
            modal.selected_value().map(str::to_string)
        };

        if let Some(value) = value {
            self.confirm_modal_value(value);
        }
    }

    fn refresh_note_structure(&mut self) {
        match crate::document::validate_document_structure(&self.editable_note, &self.sections) {
            Ok(()) => {
                self.note_headings_valid = true;
                self.note_structure_warning = None;
            }
            Err(message) => {
                self.note_headings_valid = false;
                self.note_structure_warning = Some(message);
            }
        }
    }

    fn sync_section_into_editable_note(&mut self, idx: usize) {
        if !self.note_headings_valid {
            self.status = Some(StatusMsg::error(
                self.note_structure_warning.clone().unwrap_or_else(|| {
                    "Document structure is invalid; structured sync is blocked.".to_string()
                }),
            ));
            return;
        }

        let Some(cfg) = self.sections.get(idx) else {
            return;
        };
        let Some(state) = self.section_states.get(idx) else {
            return;
        };

        let body = crate::note::render_editable_section_body(
            cfg,
            state,
            &self.config.sticky_values,
            crate::note::NoteRenderMode::Preview,
        );

        match crate::document::replace_managed_section_body(&self.editable_note, &cfg.id, &body) {
            Some(updated) => {
                self.editable_note = updated;
                self.refresh_note_structure();
            }
            None => {
                self.note_headings_valid = false;
                self.note_structure_warning =
                    Some(format!("Missing managed markers for section '{}'.", cfg.id));
                self.status = Some(StatusMsg::error(
                    self.note_structure_warning.clone().unwrap_or_else(|| {
                        "Document structure is invalid; structured sync is blocked.".to_string()
                    }),
                ));
            }
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
                    let regions = data
                        .block_select_data
                        .get(&cfg.id)
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
        if self
            .copy_flash_until
            .is_some_and(|until| Instant::now() >= until)
        {
            self.copy_flash_until = None;
        }
    }

    fn matches_key(&self, key: &AppKey, action: &[String]) -> bool {
        for binding in action {
            let matched = match_binding_str(binding, key);
            if matched {
                return true;
            }
        }
        false
    }

    fn is_navigate_down(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.navigate_down)
    }

    fn is_navigate_up(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.navigate_up)
    }

    fn is_select(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.select)
    }

    fn is_confirm(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.confirm)
    }

    fn is_add_entry(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.add_entry)
    }

    fn is_back(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.back)
    }

    fn is_swap_panes(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.swap_panes)
    }

    fn is_help(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.help)
    }

    fn is_quit(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.quit)
    }

    fn is_copy_note(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.copy_note)
    }

    fn is_focus_left(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.focus_left)
    }

    fn is_focus_right(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.focus_right)
    }

    fn is_super_confirm(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.super_confirm)
    }

    fn is_refresh_theme(&self, key: &AppKey) -> bool {
        matches!(key, AppKey::Char('/'))
    }

    fn is_refresh_data(&self, key: &AppKey) -> bool {
        matches!(key, AppKey::Char('\\'))
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

    fn handle_map_key(&mut self, key: AppKey) {
        if self.is_navigate_down(&key) {
            self.hint_buffer.clear();
            if self.map_cursor + 1 < self.sections.len() {
                self.map_cursor += 1;
                self.current_idx = self.map_cursor;
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
                self.current_idx = self.map_cursor;
                let g = self.group_idx_for_section(self.map_cursor);
                self.map_hint_level = MapHintLevel::Sections(g);
                self.update_note_scroll();
            }
            return;
        }
        if self.is_confirm(&key) {
            self.hint_buffer.clear();
            self.current_idx = self.map_cursor;
            self.map_return_idx = None;
            self.focus = Focus::Wizard;
            self.map_hint_level = MapHintLevel::Groups;
            return;
        }
        if self.is_back(&key) {
            self.hint_buffer.clear();
            if let Some(return_idx) = self.map_return_idx.take() {
                self.current_idx = return_idx;
            }
            self.focus = Focus::Wizard;
            self.map_hint_level = MapHintLevel::Groups;
            return;
        }

        // Hint key navigation
        if let AppKey::Char(c) = key {
            let case_sensitive = self.config.hint_labels_case_sensitive;
            let ch_str: String = if case_sensitive {
                c.to_string()
            } else {
                c.to_ascii_lowercase().to_string()
            };
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();

            let section_refs_owned: Vec<String> = self
                .section_hint_labels()
                .iter()
                .map(|h| {
                    if case_sensitive {
                        h.to_string()
                    } else {
                        h.to_ascii_lowercase().to_string()
                    }
                })
                .collect();
            let section_refs: Vec<&str> = section_refs_owned.iter().map(String::as_str).collect();
            match crate::data::resolve_hint(&section_refs, &typed) {
                crate::data::HintResolveResult::Exact(flat_idx) => {
                    self.current_idx = flat_idx;
                    self.map_cursor = flat_idx;
                    self.map_hint_level =
                        MapHintLevel::Sections(self.group_idx_for_section(flat_idx));
                    self.update_note_scroll();
                    self.hint_buffer.clear();
                }
                crate::data::HintResolveResult::Partial(_) => {}
                crate::data::HintResolveResult::NoMatch => {
                    self.hint_buffer.clear();
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

    fn fixed_hint_labels(&self, count: usize) -> Vec<String> {
        crate::data::generate_fixed_length_hints(&self.data.keybindings.hints, count)
    }

    fn max_header_field_count(&self) -> usize {
        self.section_states
            .iter()
            .filter_map(|state| match state {
                SectionState::Header(header) => Some(header.visible_row_count()),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }

    fn navigation_hint_labels(&self) -> Vec<String> {
        self.fixed_hint_labels(self.sections.len() + self.max_header_field_count())
    }

    pub fn section_hint_labels(&self) -> Vec<String> {
        self.navigation_hint_labels()
            .into_iter()
            .take(self.sections.len())
            .collect()
    }

    pub fn map_hint_labels(&self, group_idx: Option<usize>) -> MapHintLabels {
        let labels = self.section_hint_labels();
        let group_start: usize = group_idx
            .map(|idx| {
                self.data
                    .groups
                    .iter()
                    .take(idx)
                    .map(|group| group.sections.len())
                    .sum()
            })
            .unwrap_or(0);
        let group_len = group_idx
            .and_then(|idx| self.data.groups.get(idx))
            .map(|group| group.sections.len())
            .unwrap_or(0);
        MapHintLabels {
            sections: labels
                .into_iter()
                .skip(group_start)
                .take(group_len)
                .collect(),
        }
    }

    pub fn wizard_hint_labels(&self, field_count: usize) -> WizardHintLabels {
        let labels = self.navigation_hint_labels();
        let field_start = self.sections.len();
        WizardHintLabels {
            fields: labels
                .into_iter()
                .skip(field_start)
                .take(field_count)
                .collect(),
        }
    }

    fn update_note_scroll(&mut self) {
        self.note_scroll = self.preview_scroll_line_for_index(self.map_cursor);
    }

    pub fn reload_theme(&mut self) -> anyhow::Result<()> {
        self.ui_theme = crate::theme::AppTheme::load(&self.data_dir, &self.config.theme)?;
        Ok(())
    }

    pub fn reload_data(&mut self) -> anyhow::Result<()> {
        let previous_section_id = self.sections.get(self.current_idx).map(|s| s.id.clone());
        let data = AppData::load(self.data_dir.clone())?;
        self.sections = data.sections.clone();
        self.section_states = Self::init_states(&self.sections, &data);
        self.current_idx = previous_section_id
            .as_ref()
            .and_then(|id| self.sections.iter().position(|section| &section.id == id))
            .unwrap_or(0)
            .min(self.sections.len().saturating_sub(1));
        self.map_cursor = self.current_idx;
        self.map_return_idx = None;
        self.map_hint_level = MapHintLevel::Sections(self.group_idx_for_section(self.current_idx));
        self.modal = None;
        self.hint_buffer.clear();
        self.data = data;
        self.editable_note = build_initial_document(
            &self.sections,
            &self.section_states,
            &self.config.sticky_values,
            &self.data.boilerplate_texts,
        );
        self.refresh_note_structure();
        self.update_note_scroll();
        Ok(())
    }

    pub fn current_preview_scroll_line(&self) -> u16 {
        match self.focus {
            Focus::Map => self.note_scroll,
            Focus::Wizard => self.preview_scroll_line_for_index(self.current_idx),
        }
    }

    fn preview_scroll_line_for_index(&self, idx: usize) -> u16 {
        let Some(section) = self.sections.get(idx) else {
            return 0;
        };
        let preview = crate::document::export_editable_document(&self.editable_note);

        if let Some(line) = crate::note::managed_heading_for_section(section)
            .and_then(|heading| find_line_containing(&preview, &heading))
        {
            return line;
        }

        if let Some(line) = section
            .heading_search_text
            .as_deref()
            .and_then(|heading| find_line_containing(&preview, heading))
        {
            return line;
        }

        let Some(group_id) = self
            .data
            .groups
            .iter()
            .find(|group| group.sections.iter().any(|cfg| cfg.id == section.id))
            .map(|group| group.id.as_str())
        else {
            return 0;
        };

        let group_anchor = match group_id {
            "subjective" => "## SUBJECTIVE",
            "treatment" => "## TREATMENT / PLAN",
            "objective" => "## OBJECTIVE / OBSERVATIONS",
            "post_tx" => "## POST-TREATMENT",
            _ => "",
        };

        find_line_containing(&preview, group_anchor).unwrap_or(0)
    }

    pub fn current_map_scroll_line(&self) -> u16 {
        let group_idx = self.group_idx_for_section(self.map_cursor);
        let prior_sections: usize = self
            .data
            .groups
            .iter()
            .take(group_idx)
            .map(|group| group.sections.len())
            .sum();
        let line = group_idx + self.map_cursor.saturating_sub(prior_sections) + prior_sections;
        line.saturating_sub(4) as u16
    }

    fn text_entry_active(&self) -> bool {
        matches!(
            self.section_states.get(self.current_idx),
            Some(SectionState::FreeText(s)) if s.is_editing()
        ) || matches!(
            self.section_states.get(self.current_idx),
            Some(SectionState::ListSelect(s))
                if matches!(s.mode, ListSelectMode::AddingLabel | ListSelectMode::AddingOutput)
        )
    }

    pub fn handle_key(&mut self, key: AppKey) {
        if matches!(key, AppKey::Ignored) {
            return;
        }

        // Ctrl+C always quits
        if matches!(key, AppKey::CtrlC) {
            self.quit = true;
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

        if !self.text_entry_active() && self.is_copy_note(&key) {
            self.modal = None;
            self.show_help = false;
            self.copy_requested = true;
            return;
        }

        if !self.text_entry_active() && self.is_quit(&key) && self.focus != Focus::Map {
            let is_hint_key = if let AppKey::Char(c) = key {
                let c_str = c.to_ascii_lowercase().to_string();
                crate::data::combined_hints(&self.data.keybindings)
                    .iter()
                    .any(|h| h.to_ascii_lowercase() == c_str)
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

        if !self.text_entry_active() && self.is_refresh_theme(&key) {
            match self.reload_theme() {
                Ok(()) => {
                    self.status = Some(StatusMsg::success(format!(
                        "Theme refreshed: {}",
                        self.config.theme
                    )));
                }
                Err(err) => {
                    self.status = Some(StatusMsg::error(format!("Theme refresh failed: {err}")));
                }
            }
            return;
        }

        if !self.text_entry_active() && self.is_refresh_data(&key) {
            match self.reload_data() {
                Ok(()) => {
                    self.status = Some(StatusMsg::success("Data refreshed from YAML."));
                }
                Err(err) => {
                    self.status = Some(StatusMsg::error(format!("Data refresh failed: {err}")));
                }
            }
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
                self.map_return_idx = Some(self.current_idx);
                self.focus = Focus::Map;
                self.map_cursor = self.current_idx;
                self.map_hint_level = MapHintLevel::Sections(g_idx);
                self.update_note_scroll();
                return;
            } else if self.focus == Focus::Map && self.pane_swapped {
                // Map is to the right; h/← from map returns to wizard
                self.hint_buffer.clear();
                self.current_idx = self.map_cursor;
                self.map_return_idx = None;
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
                self.map_return_idx = Some(self.current_idx);
                self.focus = Focus::Map;
                self.map_cursor = self.current_idx;
                self.map_hint_level = MapHintLevel::Sections(g_idx);
                self.update_note_scroll();
                return;
            } else if self.focus == Focus::Map && !self.pane_swapped {
                // Map is to the left; i/→ from map returns to wizard
                self.hint_buffer.clear();
                self.current_idx = self.map_cursor;
                self.map_return_idx = None;
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
            self.status = Some(StatusMsg::success("End of note reached. Press c to copy."));
        }
    }

    fn go_back_section(&mut self) {
        if self.current_idx > 0 {
            self.current_idx -= 1;
        }
    }

    fn handle_header_key(&mut self, key: AppKey) {
        // Hint key handling: group/section hint -> return to map, field hints -> jump to field
        if let AppKey::Char(c) = key {
            let case_sensitive = self.config.hint_labels_case_sensitive;
            let ch_str: String = if case_sensitive {
                c.to_string()
            } else {
                c.to_ascii_lowercase().to_string()
            };
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();

            let idx = self.current_idx;
            let n_fields = match self.section_states.get(idx) {
                Some(SectionState::Header(s)) => s.visible_row_count(),
                _ => 0,
            };
            let labels = self.wizard_hint_labels(n_fields);

            let folded_fields: Vec<String> = labels
                .fields
                .iter()
                .map(|h| {
                    if case_sensitive {
                        h.to_string()
                    } else {
                        h.to_ascii_lowercase().to_string()
                    }
                })
                .collect();
            let field_refs: Vec<&str> = folded_fields.iter().map(String::as_str).collect();

            let section_refs_owned: Vec<String> = self
                .section_hint_labels()
                .iter()
                .map(|h| {
                    if case_sensitive {
                        h.to_string()
                    } else {
                        h.to_ascii_lowercase().to_string()
                    }
                })
                .collect();
            let section_refs: Vec<&str> = section_refs_owned.iter().map(String::as_str).collect();
            match crate::data::resolve_hint(&section_refs, &typed) {
                crate::data::HintResolveResult::Exact(flat_idx) => {
                    self.current_idx = flat_idx;
                    self.map_cursor = flat_idx;
                    self.map_hint_level =
                        MapHintLevel::Sections(self.group_idx_for_section(flat_idx));
                    self.update_note_scroll();
                    self.hint_buffer.clear();
                    return;
                }
                crate::data::HintResolveResult::Partial(_) => return,
                crate::data::HintResolveResult::NoMatch => {}
            }

            // Build the candidate list for field hints and resolve
            match crate::data::resolve_hint(&field_refs, &typed) {
                crate::data::HintResolveResult::Exact(row_idx) => {
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        if let Some((field_idx, repeat_idx)) =
                            s.field_index_for_visible_row(row_idx)
                        {
                            s.field_index = field_idx;
                            if s.field_configs
                                .get(field_idx)
                                .is_some_and(|field| field.repeat_limit.is_some())
                            {
                                s.repeat_counts[field_idx] = repeat_idx;
                            }
                        }
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
        }

        if self.is_super_confirm(&key) {
            let idx = self.current_idx;
            let resolved = if let Some(SectionState::Header(s)) = self.section_states.get(idx) {
                s.field_configs.get(s.field_index).map(|cfg| {
                    let confirmed = s
                        .repeated_values
                        .get(s.field_index)
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
            if let Some(crate::sections::multi_field::ResolvedMultiFieldValue::Complete(value)) =
                resolved
            {
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.set_current_value(value);
                    let done = s.advance();
                    self.sync_section_into_editable_note(idx);
                    if done {
                        self.advance_section();
                    }
                }
            }
            return;
        }

        if matches!(key, AppKey::Backspace) {
            let idx = self.current_idx;
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.blank_active_value();
                s.completed = false;
                self.sync_section_into_editable_note(idx);
            }
            return;
        }

        if self.is_back(&key) || self.is_navigate_up(&key) {
            let idx = self.current_idx;
            let went_back = if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx)
            {
                // Normalize out-of-bounds index before going back
                if s.field_index >= s.field_configs.len() && !s.field_configs.is_empty() {
                    s.field_index = s.field_configs.len() - 1;
                    s.repeat_counts[s.field_index] = 0;
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
                } else {
                    let _ = s.go_forward();
                }
            }
            return;
        }

        if matches!(key, AppKey::Enter) {
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
            if cfg.lists.is_empty() {
                return;
            }
            let window_size = self.modal_window_size();
            let modal =
                SearchModal::new_field(field_idx, cfg, &self.config.sticky_values, window_size);
            self.modal = Some(modal);
        }
    }

    fn handle_modal_key(&mut self, key: AppKey) {
        if matches!(key, AppKey::Esc) {
            self.hint_buffer.clear();
            self.modal = None;
            return;
        }

        if self.is_super_confirm(&key) {
            self.super_confirm_modal_field();
            return;
        }

        if matches!(key, AppKey::Left) {
            self.composite_go_back();
            return;
        }

        if matches!(key, AppKey::Right) {
            if let Some(value) = self
                .modal
                .as_ref()
                .and_then(|modal| modal.selected_value().map(str::to_string))
            {
                self.confirm_modal_value(value);
            }
            return;
        }

        let focus = match &self.modal {
            Some(m) => m.focus.clone(),
            None => return,
        };

        match focus {
            ModalFocus::SearchBar => match key {
                AppKey::Tab => {
                    let query = self.modal.as_ref().unwrap().query.trim().to_string();
                    if !query.is_empty() {
                        self.confirm_modal_value(query);
                    }
                }
                AppKey::Enter => {
                    if self
                        .modal
                        .as_ref()
                        .is_some_and(|modal| modal.should_finish_repeating_from_empty_search())
                    {
                        self.confirm_modal_value(String::new());
                        return;
                    }

                    let only_value = self.modal.as_ref().and_then(|modal| {
                        if modal.filtered.len() == 1 {
                            modal.selected_value().map(String::from)
                        } else {
                            None
                        }
                    });
                    if let Some(value) = only_value {
                        self.confirm_modal_value(value);
                    } else if self
                        .modal
                        .as_ref()
                        .is_some_and(|modal| !modal.filtered.is_empty())
                    {
                        self.modal.as_mut().unwrap().focus = ModalFocus::List;
                    }
                }
                AppKey::Backspace => {
                    let modal = self.modal.as_mut().unwrap();
                    modal.query.pop();
                    modal.update_filter();
                    if modal.query.is_empty() {
                        modal.center_scroll();
                    }
                }
                AppKey::Char(c) => {
                    let modal = self.modal.as_mut().unwrap();
                    modal.query.push(c);
                    modal.update_filter();
                }
                AppKey::Space => {
                    let modal = self.modal.as_mut().unwrap();
                    modal.query.push(' ');
                    modal.update_filter();
                }
                _ => {}
            },
            ModalFocus::List => match key {
                AppKey::Backspace => {
                    self.hint_buffer.clear();
                    let can_go_back = self
                        .modal
                        .as_ref()
                        .map(|m| m.field_flow.list_idx > 0)
                        .unwrap_or(false);
                    if can_go_back {
                        self.composite_go_back();
                    } else {
                        // First part or simple field: exit modal, return to wizard
                        self.modal = None;
                    }
                }
                AppKey::Space => {
                    self.hint_buffer.clear();
                    self.modal.as_mut().unwrap().focus = ModalFocus::SearchBar;
                }
                AppKey::Enter => {
                    self.hint_buffer.clear();
                    if let Some(val) = self
                        .modal
                        .as_ref()
                        .unwrap()
                        .selected_value()
                        .map(String::from)
                    {
                        self.confirm_modal_value(val);
                    }
                }
                AppKey::Up => {
                    self.hint_buffer.clear();
                    let modal = self.modal.as_mut().unwrap();
                    if modal.list_cursor > 0 {
                        modal.list_cursor -= 1;
                        modal.update_scroll();
                    }
                }
                AppKey::Down => {
                    self.hint_buffer.clear();
                    let modal = self.modal.as_mut().unwrap();
                    if modal.list_cursor + 1 < modal.filtered.len() {
                        modal.list_cursor += 1;
                        modal.update_scroll();
                    }
                }
                AppKey::Char(c) => {
                    let case_sensitive = self.config.hint_labels_case_sensitive;
                    let ch_str: String = if case_sensitive {
                        c.to_string()
                    } else {
                        c.to_ascii_lowercase().to_string()
                    };
                    let visible_count = self
                        .modal
                        .as_ref()
                        .map(|modal| {
                            let end =
                                (modal.list_scroll + modal.window_size).min(modal.filtered.len());
                            end.saturating_sub(modal.list_scroll)
                        })
                        .unwrap_or(0);
                    let labels: Vec<String> = self
                        .data
                        .keybindings
                        .hints
                        .iter()
                        .take(visible_count)
                        .cloned()
                        .collect();
                    let folded_labels: Vec<String> = labels
                        .iter()
                        .map(|h| {
                            if case_sensitive {
                                h.to_string()
                            } else {
                                h.to_ascii_lowercase()
                            }
                        })
                        .collect();
                    if let Some(hint_pos) = folded_labels.iter().position(|label| label == &ch_str)
                    {
                        if let Some(val) = self
                            .modal
                            .as_ref()
                            .unwrap()
                            .hint_value(hint_pos)
                            .map(String::from)
                        {
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
        if self.modal.is_some() {
            let window_size = self.modal_window_size();
            let advance = self.modal.as_mut().unwrap().advance_field(
                value,
                &mut self.config.sticky_values,
                window_size,
            );
            match advance {
                FieldAdvance::NextList => {
                    let preview = compute_field_preview(
                        self.modal.as_ref().unwrap(),
                        &self.config.sticky_values,
                    );
                    let spans = compute_field_spans(
                        self.modal.as_ref().unwrap(),
                        &self.config.sticky_values,
                    );
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        s.set_preview_value(preview);
                        s.composite_spans = Some(spans);
                    }
                    let _ = self.config.save(&self.data_dir);
                }
                FieldAdvance::StayOnList => {
                    let preview = compute_field_preview(
                        self.modal.as_ref().unwrap(),
                        &self.config.sticky_values,
                    );
                    let spans = compute_field_spans(
                        self.modal.as_ref().unwrap(),
                        &self.config.sticky_values,
                    );
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        s.set_preview_value(preview);
                        s.composite_spans = Some(spans);
                    }
                }
                FieldAdvance::Complete(final_value) => {
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        s.composite_spans = None;
                        s.set_current_value(final_value);
                        let done = s.advance();
                        self.sync_section_into_editable_note(idx);
                        if done {
                            self.advance_section();
                        }
                    }
                    self.modal = None;
                    let _ = self.config.save(&self.data_dir);
                }
            }
        }
    }

    fn super_confirm_modal_field(&mut self) {
        let idx = self.current_idx;
        if self.modal.is_none() {
            return;
        }

        let window_size = self.modal_window_size();
        let advance = self
            .modal
            .as_mut()
            .unwrap()
            .super_confirm_field(&mut self.config.sticky_values, window_size);

        match advance {
            FieldAdvance::NextList | FieldAdvance::StayOnList => {
                let preview =
                    compute_field_preview(self.modal.as_ref().unwrap(), &self.config.sticky_values);
                let spans =
                    compute_field_spans(self.modal.as_ref().unwrap(), &self.config.sticky_values);
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.set_preview_value(preview);
                    s.composite_spans = Some(spans);
                }
                let _ = self.config.save(&self.data_dir);
            }
            FieldAdvance::Complete(final_value) => {
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.composite_spans = None;
                    s.set_current_value(final_value);
                    let done = s.advance();
                    self.sync_section_into_editable_note(idx);
                    if done {
                        self.advance_section();
                    }
                }
                self.modal = None;
                let _ = self.config.save(&self.data_dir);
            }
        }
    }

    fn handle_free_text_key(&mut self, key: AppKey) {
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
            if matches!(key, AppKey::Enter) {
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.commit_entry();
                }
                self.sync_section_into_editable_note(idx);
                return;
            }
            if matches!(key, AppKey::Backspace) {
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.handle_backspace();
                }
                return;
            }
            if let AppKey::Char(c) = key {
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
                self.sync_section_into_editable_note(idx);
                self.advance_section();
            } else {
                // Empty = skip
                if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                    s.skipped = true;
                }
                self.sync_section_into_editable_note(idx);
                self.advance_section();
            }
        }
    }

    fn handle_list_select_key(&mut self, key: AppKey) {
        let idx = self.current_idx;
        let mode = match &self.section_states[idx] {
            SectionState::ListSelect(s) => match s.mode {
                ListSelectMode::Browsing => 0,
                ListSelectMode::AddingLabel => 1,
                ListSelectMode::AddingOutput => 2,
            },
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
                if matches!(key, AppKey::Enter) {
                    if mode == 1 {
                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.confirm_label();
                        }
                    } else {
                        let new_entry =
                            if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                                s.confirm_output()
                            } else {
                                None
                            };
                        if let Some(entry) = new_entry {
                            let data_file = self.sections[idx].data_file.clone();
                            if let Some(ref df) = data_file {
                                match self.data.append_list_entry(df, entry.clone()) {
                                    Ok(_) => {
                                        if let SectionState::ListSelect(s) =
                                            &mut self.section_states[idx]
                                        {
                                            s.entries = self
                                                .data
                                                .list_data
                                                .get(df)
                                                .cloned()
                                                .unwrap_or_default();
                                            // Select the newly added entry
                                            let new_idx = s.entries.len().saturating_sub(1);
                                            s.cursor = new_idx;
                                            s.selected_indices.push(new_idx);
                                        }
                                        self.sync_section_into_editable_note(idx);
                                        self.status = Some(StatusMsg::success("Entry added."));
                                    }
                                    Err(e) => {
                                        self.status = Some(StatusMsg::error(format!(
                                            "Failed to save: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                        }
                    }
                    return;
                }
                if matches!(key, AppKey::Backspace) {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.handle_backspace();
                    }
                    return;
                }
                if let AppKey::Char(c) = key {
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
                    self.sync_section_into_editable_note(idx);
                    return;
                }
                if self.is_add_entry(&key) {
                    if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                        s.start_add_label();
                    }
                    return;
                }
                if self.is_confirm(&key) {
                    let (has_selection, has_entries) = match &self.section_states[idx] {
                        SectionState::ListSelect(s) => {
                            (!s.selected_indices.is_empty(), !s.entries.is_empty())
                        }
                        _ => (false, false),
                    };
                    if has_selection {
                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.completed = true;
                        }
                        self.sync_section_into_editable_note(idx);
                        self.advance_section();
                    } else if has_entries {
                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.toggle_current();
                        }
                        self.sync_section_into_editable_note(idx);
                    } else {
                        if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                            s.skipped = true;
                        }
                        self.sync_section_into_editable_note(idx);
                        self.advance_section();
                    }
                }
            }
        }
    }

    fn handle_block_select_key(&mut self, key: AppKey) {
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
                self.sync_section_into_editable_note(idx);
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
                    self.sync_section_into_editable_note(idx);
                    self.advance_section();
                } else {
                    if let SectionState::BlockSelect(s) = &mut self.section_states[idx] {
                        s.skipped = true;
                    }
                    self.sync_section_into_editable_note(idx);
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
                self.sync_section_into_editable_note(idx);
                self.advance_section();
            }
        }
    }

    fn handle_checklist_key(&mut self, key: AppKey) {
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
            self.sync_section_into_editable_note(idx);
            return;
        }
        if self.is_confirm(&key) {
            if let SectionState::Checklist(s) = &mut self.section_states[idx] {
                s.completed = true;
            }
            self.sync_section_into_editable_note(idx);
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
    fn try_navigate_to_map_via_hint(&mut self, key: &AppKey) -> bool {
        if let AppKey::Char(c) = *key {
            let case_sensitive = self.config.hint_labels_case_sensitive;
            let ch_str: String = if case_sensitive {
                c.to_string()
            } else {
                c.to_ascii_lowercase().to_string()
            };
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();

            let section_refs_owned: Vec<String> = self
                .section_hint_labels()
                .iter()
                .map(|h| {
                    if case_sensitive {
                        h.to_string()
                    } else {
                        h.to_ascii_lowercase().to_string()
                    }
                })
                .collect();
            let section_refs: Vec<&str> = section_refs_owned.iter().map(String::as_str).collect();
            match crate::data::resolve_hint(&section_refs, &typed) {
                crate::data::HintResolveResult::Exact(flat_idx) => {
                    self.current_idx = flat_idx;
                    self.map_cursor = flat_idx;
                    self.map_hint_level =
                        MapHintLevel::Sections(self.group_idx_for_section(flat_idx));
                    self.update_note_scroll();
                    self.hint_buffer.clear();
                    return true;
                }
                crate::data::HintResolveResult::Partial(_) => return false,
                crate::data::HintResolveResult::NoMatch => {}
            }

            // No hint matched
            self.hint_buffer.clear();
        }
        false
    }

    fn composite_go_back(&mut self) {
        let idx = self.current_idx;

        let window_size = self.modal_window_size();
        let (new_list_idx, popped_output, new_labels, new_outputs) = {
            let modal = match self.modal.as_mut() {
                Some(m) => m,
                None => return,
            };
            if modal.field_flow.list_idx == 0 {
                let _ = modal.restore_parent_branch(&self.config.sticky_values, window_size);
                return;
            }
            let popped = modal.field_flow.values.pop();
            modal.field_flow.list_idx -= 1;
            let list = &modal.field_flow.lists[modal.field_flow.list_idx];
            let labels = resolved_item_labels_for_list(
                list,
                &modal.field_flow.values,
                &modal.field_flow.repeat_values,
                &modal.field_flow.lists,
                &self.config.sticky_values,
            );
            let outputs: Vec<String> = list
                .items
                .iter()
                .map(|item| item.output.clone().unwrap_or_else(|| item.label.clone()))
                .collect();
            (modal.field_flow.list_idx, popped, labels, outputs)
        };

        let cursor = popped_output
            .as_ref()
            .and_then(|v| new_outputs.iter().position(|e| e == v))
            .unwrap_or(0);
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

        if new_list_idx == 0 {
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.composite_spans = None;
                s.clear_active_value();
            }
        } else {
            let (preview, spans) = {
                let modal = self.modal.as_ref().unwrap();
                (
                    compute_field_preview(modal, &self.config.sticky_values),
                    compute_field_spans(modal, &self.config.sticky_values),
                )
            };
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.set_preview_value(preview);
                s.composite_spans = Some(spans);
            }
        }
    }
}

fn compute_field_spans(
    modal: &crate::modal::SearchModal,
    sticky_values: &std::collections::HashMap<String, String>,
) -> Vec<(String, bool)> {
    let flow = &modal.field_flow;
    let Some(format) = &flow.format else {
        return flow
            .values
            .iter()
            .map(|value| (value.clone(), true))
            .collect();
    };
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
                if c2 == '}' {
                    break;
                }
                id.push(c2);
            }
            if let Some(i) = flow.lists.iter().position(|list| list.id == id) {
                if i < flow.values.len() {
                    spans.push((flow.values[i].clone(), true));
                } else if i == flow.list_idx && !flow.repeat_values.is_empty() {
                    spans.push((
                        joined_repeating_value(&flow.lists[i], &flow.repeat_values)
                            .unwrap_or_else(|| flow.repeat_values.join(", ")),
                        true,
                    ));
                } else if let Some(value) = fallback_list_value(&flow.lists[i], sticky_values) {
                    spans.push((value, false));
                } else {
                    let preview = flow.lists[i].preview.as_deref().unwrap_or("?");
                    spans.push((preview.to_string(), false));
                }
            } else if let Some(list) = flow.format_lists.iter().find(|list| list.id == id) {
                spans.push((
                    fallback_list_value(list, sticky_values).unwrap_or_default(),
                    false,
                ));
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

fn compute_field_preview(
    modal: &crate::modal::SearchModal,
    sticky_values: &std::collections::HashMap<String, String>,
) -> String {
    let flow = &modal.field_flow;
    let Some(format) = &flow.format else {
        return flow.values.join(", ");
    };
    let mut result = format.clone();
    for (i, val) in flow.values.iter().enumerate() {
        let placeholder = format!("{{{}}}", flow.lists[i].id);
        result = result.replace(&placeholder, val);
    }
    for (i, list) in flow.lists.iter().enumerate().skip(flow.list_idx) {
        let placeholder = format!("{{{}}}", list.id);
        let value = if i == flow.list_idx && !flow.repeat_values.is_empty() {
            joined_repeating_value(list, &flow.repeat_values)
                .unwrap_or_else(|| flow.repeat_values.join(", "))
        } else {
            fallback_list_value(list, sticky_values)
                .unwrap_or_else(|| list.preview.as_deref().unwrap_or("?").to_string())
        };
        result = result.replace(&placeholder, &value);
    }
    for list in &flow.format_lists {
        let placeholder = format!("{{{}}}", list.id);
        if result.contains(&placeholder) {
            let value = fallback_list_value(list, sticky_values).unwrap_or_default();
            result = result.replace(&placeholder, &value);
        }
    }
    result
}

fn fallback_list_value(
    list: &crate::data::HierarchyList,
    sticky_values: &std::collections::HashMap<String, String>,
) -> Option<String> {
    if list.sticky {
        if let Some(value) = sticky_values.get(&list.id) {
            if !value.is_empty() {
                return Some(value.clone());
            }
        }
    }
    if let Some(default) = &list.default {
        if let Some(item) = list.items.iter().find(|item| {
            item.id == *default
                || item.label == *default
                || item.output.as_deref() == Some(default.as_str())
        }) {
            return Some(item.output.clone().unwrap_or_else(|| item.label.clone()));
        }
    }
    None
}

fn find_line_containing(text: &str, needle: &str) -> Option<u16> {
    if needle.is_empty() {
        return None;
    }
    text.lines()
        .position(|line| line.contains(needle))
        .map(|idx| idx as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_key_shift_enter_binding_recognized() {
        assert!(
            match_binding_str("shift+enter", &AppKey::ShiftEnter),
            "match_binding_str(\"shift+enter\", AppKey::ShiftEnter) should return true"
        );
    }

    #[test]
    fn matches_key_shift_enter_does_not_match_plain_enter() {
        assert!(
            !match_binding_str("shift+enter", &AppKey::Enter),
            "match_binding_str(\"shift+enter\", AppKey::Enter) should return false"
        );
    }

    #[test]
    fn matches_key_super_confirm_binding_in_keybindings() {
        let kb = crate::data::KeyBindings::default();
        assert!(
            kb.super_confirm.iter().any(|b| b == "shift+enter"),
            "KeyBindings::default().super_confirm should contain \"shift+enter\""
        );
    }

    #[test]
    fn matches_key_ctrl_q_binding_recognized() {
        assert!(
            match_binding_str("ctrl+q", &AppKey::CtrlChar('q')),
            "match_binding_str(\"ctrl+q\", AppKey::CtrlChar('q')) should return true"
        );
    }

    #[test]
    fn copy_key_is_text_when_free_text_editor_is_active() {
        use crate::config::Config;
        use crate::data::{AppData, KeyBindings, SectionConfig, SectionGroup};
        use crate::sections::free_text::{FreeTextMode, FreeTextState};
        use std::path::PathBuf;

        let section = SectionConfig {
            id: "subjective_section".to_string(),
            name: "Subjective".to_string(),
            map_label: "Subjective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## SUBJECTIVE".to_string()),
            heading_label: None,
            note_render_slot: Some("subjective_section".to_string()),
        };
        let data = AppData {
            groups: vec![SectionGroup {
                id: "subjective".to_string(),
                num: None,
                name: "Subjective".to_string(),
                sections: vec![section.clone()],
            }],
            sections: vec![section],
            list_data: Default::default(),
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };
        let mut app = App::new(data, Config::default(), PathBuf::new());
        app.section_states[0] = SectionState::FreeText(FreeTextState {
            entries: Vec::new(),
            cursor: 0,
            mode: FreeTextMode::Editing,
            edit_buf: String::new(),
            skipped: false,
            completed: false,
        });

        app.handle_key(AppKey::Char('c'));

        assert!(!app.copy_requested);
        match &app.section_states[0] {
            SectionState::FreeText(state) => assert_eq!(state.edit_buf, "c"),
            _ => panic!("expected free text state"),
        }
    }

    #[test]
    fn super_confirm_fills_default_and_advances() {
        use crate::config::Config;
        use crate::data::{AppData, HeaderFieldConfig, KeyBindings, SectionConfig, SectionGroup};
        use std::path::PathBuf;

        let fields = vec![
            HeaderFieldConfig {
                id: "f1".to_string(),
                name: "F1".to_string(),
                options: vec![],
                composite: None,
                default: Some("hello".to_string()),
                repeat_limit: None,
            },
            HeaderFieldConfig {
                id: "f2".to_string(),
                name: "F2".to_string(),
                options: vec![],
                composite: None,
                default: None,
                repeat_limit: None,
            },
        ];
        let section = SectionConfig {
            id: "s1".to_string(),
            name: "S1".to_string(),
            map_label: "S1".to_string(),
            section_type: "multi_field".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: Some(fields),
            is_intake: false,
            heading_search_text: None,
            heading_label: None,
            note_render_slot: None,
        };
        let group = SectionGroup {
            id: "g1".to_string(),
            num: None,
            name: "G1".to_string(),
            sections: vec![section.clone()],
        };
        let data = AppData {
            groups: vec![group],
            sections: vec![section],
            list_data: Default::default(),
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };
        let mut app = App::new(data, Config::default(), PathBuf::new());
        app.handle_header_key(AppKey::ShiftEnter);
        if let Some(SectionState::Header(s)) = app.section_states.first() {
            assert_eq!(
                s.repeated_values[0]
                    .last()
                    .map(|s| s.as_str())
                    .unwrap_or(""),
                "hello",
                "field 0 should be filled with its default"
            );
            assert_eq!(s.field_index, 1, "field_index should advance to 1");
        } else {
            panic!("expected Header state at index 0");
        }
    }

    #[test]
    fn super_confirm_no_op_when_no_default() {
        use crate::config::Config;
        use crate::data::{AppData, HeaderFieldConfig, KeyBindings, SectionConfig, SectionGroup};
        use std::path::PathBuf;

        let fields = vec![HeaderFieldConfig {
            id: "f1".to_string(),
            name: "F1".to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit: None,
        }];
        let section = SectionConfig {
            id: "s1".to_string(),
            name: "S1".to_string(),
            map_label: "S1".to_string(),
            section_type: "multi_field".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: Some(fields),
            is_intake: false,
            heading_search_text: None,
            heading_label: None,
            note_render_slot: None,
        };
        let group = SectionGroup {
            id: "g1".to_string(),
            num: None,
            name: "G1".to_string(),
            sections: vec![section.clone()],
        };
        let data = AppData {
            groups: vec![group],
            sections: vec![section],
            list_data: Default::default(),
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };
        let mut app = App::new(data, Config::default(), PathBuf::new());
        app.handle_header_key(AppKey::ShiftEnter);
        if let Some(SectionState::Header(s)) = app.section_states.first() {
            assert_eq!(
                s.field_index, 0,
                "field_index should stay at 0 when no default"
            );
        } else {
            panic!("expected Header state at index 0");
        }
    }

    #[test]
    fn sync_section_into_editable_note_updates_only_current_managed_block() {
        use crate::config::Config;
        use crate::data::{AppData, KeyBindings, SectionConfig, SectionGroup};
        use crate::sections::free_text::FreeTextState;
        use std::path::PathBuf;

        let subjective = SectionConfig {
            id: "subjective_section".to_string(),
            name: "Subjective".to_string(),
            map_label: "Subjective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## SUBJECTIVE".to_string()),
            heading_label: None,
            note_render_slot: Some("subjective_section".to_string()),
        };
        let objective = SectionConfig {
            id: "objective_section".to_string(),
            name: "Objective".to_string(),
            map_label: "Objective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## OBJECTIVE / OBSERVATIONS".to_string()),
            heading_label: None,
            note_render_slot: Some("objective_section".to_string()),
        };
        let group_subjective = SectionGroup {
            id: "subjective".to_string(),
            num: None,
            name: "Subjective".to_string(),
            sections: vec![subjective.clone()],
        };
        let group_objective = SectionGroup {
            id: "objective".to_string(),
            num: None,
            name: "Objective".to_string(),
            sections: vec![objective.clone()],
        };
        let data = AppData {
            groups: vec![group_subjective, group_objective],
            sections: vec![subjective, objective],
            list_data: Default::default(),
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };

        let mut app = App::new(data, Config::default(), PathBuf::new());
        app.editable_note = app.editable_note.replace(
            "<!-- scribblenot:section id=objective_section:end -->",
            "User free edits stay here.\n<!-- scribblenot:section id=objective_section:end -->",
        );

        app.current_idx = 0;
        app.section_states[0] = SectionState::FreeText(FreeTextState {
            entries: vec!["subjective synced".to_string()],
            edit_buf: String::new(),
            mode: crate::sections::free_text::FreeTextMode::Browsing,
            cursor: 0,
            completed: true,
            skipped: false,
        });
        app.sync_section_into_editable_note(0);

        assert!(app.editable_note.contains("subjective synced"));
        assert!(app.editable_note.contains("User free edits stay here."));
        assert!(app.note_headings_valid);
    }

    #[test]
    fn set_editable_note_recomputes_invalid_structure_warning() {
        use crate::config::Config;
        use crate::data::{AppData, KeyBindings, SectionConfig, SectionGroup};
        use std::path::PathBuf;

        let subjective = SectionConfig {
            id: "subjective_section".to_string(),
            name: "Subjective".to_string(),
            map_label: "Subjective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## SUBJECTIVE".to_string()),
            heading_label: None,
            note_render_slot: Some("subjective_section".to_string()),
        };
        let group = SectionGroup {
            id: "subjective".to_string(),
            num: None,
            name: "Subjective".to_string(),
            sections: vec![subjective.clone()],
        };
        let data = AppData {
            groups: vec![group],
            sections: vec![subjective],
            list_data: Default::default(),
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };

        let mut app = App::new(data, Config::default(), PathBuf::new());
        let broken = app.editable_note.replace(
            "<!-- scribblenot:section id=subjective_section:start -->",
            "",
        );
        app.set_editable_note(broken);

        assert!(!app.note_headings_valid);
        assert!(app
            .note_structure_warning
            .as_deref()
            .unwrap_or("")
            .contains("managed section"));
    }

    #[test]
    fn list_select_enter_selects_current_before_confirming_section() {
        use crate::config::Config;
        use crate::data::{AppData, KeyBindings, ListEntry, SectionConfig, SectionGroup};
        use std::collections::HashMap;
        use std::path::PathBuf;

        let section = SectionConfig {
            id: "objective_section".to_string(),
            name: "Objective".to_string(),
            map_label: "Objective".to_string(),
            section_type: "list_select".to_string(),
            data_file: Some("objective.yml".to_string()),
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## OBJECTIVE / OBSERVATIONS".to_string()),
            heading_label: None,
            note_render_slot: Some("objective_section".to_string()),
        };
        let group = SectionGroup {
            id: "objective".to_string(),
            num: None,
            name: "Objective".to_string(),
            sections: vec![section.clone()],
        };
        let mut list_data = HashMap::new();
        list_data.insert(
            "objective.yml".to_string(),
            vec![ListEntry {
                label: "Upper traps".to_string(),
                output: "Upper traps: Increased resting muscle tension".to_string(),
            }],
        );
        let data = AppData {
            groups: vec![group],
            sections: vec![section],
            list_data,
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };

        let mut app = App::new(data, Config::default(), PathBuf::new());

        app.handle_list_select_key(AppKey::Enter);
        match &app.section_states[0] {
            SectionState::ListSelect(state) => {
                assert_eq!(state.selected_indices, vec![0]);
                assert!(!state.completed);
                assert!(!state.skipped);
            }
            _ => panic!("expected list select state"),
        }
        assert_eq!(app.current_idx, 0);
        assert!(app
            .editable_note
            .contains("Upper traps: Increased resting muscle tension"));

        app.handle_list_select_key(AppKey::Enter);
        match &app.section_states[0] {
            SectionState::ListSelect(state) => {
                assert!(state.completed);
                assert!(!state.skipped);
            }
            _ => panic!("expected list select state"),
        }
        assert!(app
            .status
            .as_ref()
            .is_some_and(|status| status.text.contains("End of note")));
    }

    #[test]
    fn preview_scroll_tracks_map_cursor_in_clean_preview() {
        use crate::config::Config;
        use crate::data::{AppData, KeyBindings, SectionConfig, SectionGroup};
        use crate::sections::free_text::{FreeTextMode, FreeTextState};
        use std::path::PathBuf;

        let subjective = SectionConfig {
            id: "subjective_section".to_string(),
            name: "Subjective".to_string(),
            map_label: "Subjective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## SUBJECTIVE".to_string()),
            heading_label: None,
            note_render_slot: Some("subjective_section".to_string()),
        };
        let objective = SectionConfig {
            id: "objective_section".to_string(),
            name: "Objective".to_string(),
            map_label: "Objective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## OBJECTIVE / OBSERVATIONS".to_string()),
            heading_label: None,
            note_render_slot: Some("objective_section".to_string()),
        };
        let data = AppData {
            groups: vec![
                SectionGroup {
                    id: "subjective".to_string(),
                    num: None,
                    name: "Subjective".to_string(),
                    sections: vec![subjective.clone()],
                },
                SectionGroup {
                    id: "objective".to_string(),
                    num: None,
                    name: "Objective".to_string(),
                    sections: vec![objective.clone()],
                },
            ],
            sections: vec![subjective, objective],
            list_data: Default::default(),
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };

        let mut app = App::new(data, Config::default(), PathBuf::new());
        app.section_states[0] = SectionState::FreeText(FreeTextState {
            entries: vec!["subjective body".to_string()],
            edit_buf: String::new(),
            mode: FreeTextMode::Browsing,
            cursor: 0,
            completed: true,
            skipped: false,
        });
        app.sync_section_into_editable_note(0);
        app.section_states[1] = SectionState::FreeText(FreeTextState {
            entries: vec!["objective body".to_string()],
            edit_buf: String::new(),
            mode: FreeTextMode::Browsing,
            cursor: 0,
            completed: true,
            skipped: false,
        });
        app.sync_section_into_editable_note(1);

        app.current_idx = 0;
        app.map_cursor = 1;
        app.focus = Focus::Map;
        app.update_note_scroll();

        let preview = crate::document::export_editable_document(&app.editable_note);
        let objective_line = find_line_containing(&preview, "#### OBJECTIVE").unwrap();
        let subjective_line = find_line_containing(&preview, "#### SUBJECTIVE").unwrap();

        assert_eq!(app.current_preview_scroll_line(), objective_line);
        assert_ne!(app.current_preview_scroll_line(), subjective_line);
    }

    #[test]
    fn map_browsing_previews_wizard_section_but_back_restores_original() {
        use crate::config::Config;
        use crate::data::{AppData, KeyBindings, SectionConfig, SectionGroup};
        use std::path::PathBuf;

        let subjective = SectionConfig {
            id: "subjective_section".to_string(),
            name: "Subjective".to_string(),
            map_label: "Subjective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## SUBJECTIVE".to_string()),
            heading_label: None,
            note_render_slot: Some("subjective_section".to_string()),
        };
        let objective = SectionConfig {
            id: "objective_section".to_string(),
            name: "Objective".to_string(),
            map_label: "Objective".to_string(),
            section_type: "free_text".to_string(),
            data_file: None,
            date_prefix: None,
            options: vec![],
            composite: None,
            fields: None,
            is_intake: false,
            heading_search_text: Some("## OBJECTIVE / OBSERVATIONS".to_string()),
            heading_label: None,
            note_render_slot: Some("objective_section".to_string()),
        };
        let data = AppData {
            groups: vec![
                SectionGroup {
                    id: "subjective".to_string(),
                    num: None,
                    name: "Subjective".to_string(),
                    sections: vec![subjective.clone()],
                },
                SectionGroup {
                    id: "objective".to_string(),
                    num: None,
                    name: "Objective".to_string(),
                    sections: vec![objective.clone()],
                },
            ],
            sections: vec![subjective, objective],
            list_data: Default::default(),
            checklist_data: Default::default(),
            block_select_data: Default::default(),
            boilerplate_texts: Default::default(),
            keybindings: KeyBindings::default(),
            data_dir: PathBuf::new(),
        };

        let mut app = App::new(data, Config::default(), PathBuf::new());
        app.focus = Focus::Map;
        app.map_return_idx = Some(0);

        app.handle_key(AppKey::Down);
        assert_eq!(app.current_idx, 1);
        assert_eq!(app.focus, Focus::Map);

        app.handle_key(AppKey::Esc);
        assert_eq!(app.current_idx, 0);
        assert_eq!(app.focus, Focus::Wizard);
    }
}
