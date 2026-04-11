use crate::config::Config;
use crate::data::{AppData, SectionConfig};
use crate::document::build_initial_document;
use crate::modal::{
    joined_repeating_value, modal_height_for_viewport, modal_window_size_for_height,
    resolved_item_labels_for_list, FieldAdvance, ModalFocus, SearchModal,
};
use crate::sections::{
    checklist::ChecklistState,
    collection::CollectionState,
    free_text::FreeTextState,
    header::{HeaderFieldValue, HeaderState},
    list_select::{ListSelectMode, ListSelectState},
};
use iced::keyboard::{key::Named, Key, Modifiers};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalPaneTarget {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalStreamDirection {
    Forward,
    Backward,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalStreamEasing {
    ExpoInOut,
    ExpoOut,
}

impl ModalStreamEasing {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Self::ExpoInOut => simple_easing::expo_in_out(t),
            Self::ExpoOut => simple_easing::expo_out(t),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModalStreamTransition {
    pub previous_modal: SearchModal,
    pub direction: ModalStreamDirection,
    pub started_at: Instant,
    pub duration: Duration,
    pub easing: ModalStreamEasing,
}

impl ModalStreamTransition {
    pub fn progress(&self) -> f32 {
        let duration = self.duration.as_secs_f32().max(0.001);
        (self.started_at.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0)
    }

    pub fn eased_progress(&self) -> f32 {
        self.easing.apply(self.progress())
    }

    pub fn is_finished(&self) -> bool {
        self.progress() >= 1.0
    }
}

#[derive(Debug, Clone)]
struct ModalRestoreSnapshot {
    field_idx: usize,
    value_index: usize,
    original_value: Option<HeaderFieldValue>,
    original_spans: Option<Vec<(String, bool)>>,
}

#[derive(Debug, Clone)]
pub enum SectionState {
    Pending,
    Header(HeaderState),
    FreeText(FreeTextState),
    ListSelect(ListSelectState),
    Collection(CollectionState),
    Checklist(ChecklistState),
}

#[derive(Debug, Clone)]
pub struct StatusMsg {
    pub text: String,
    pub is_error: bool,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldCompositionSpanKind {
    Literal,
    Confirmed,
    Preview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldCompositionSpan {
    pub text: String,
    pub kind: FieldCompositionSpanKind,
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
    pub evicted_collection_flash_until: HashMap<String, Instant>,
    pub data_dir: PathBuf,
    pub focus: Focus,
    pub map_cursor: usize,
    pub map_return_idx: Option<usize>,
    pub map_hint_level: MapHintLevel,
    pub note_scroll: u16,
    pub modal: Option<SearchModal>,
    pub hint_buffer: String,
    pub modal_mouse_mode: bool,
    modal_restore_snapshot: Option<ModalRestoreSnapshot>,
    pub modal_stream_transition: Option<ModalStreamTransition>,
    pub modal_composition_editing: bool,
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
            &data.groups,
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
            evicted_collection_flash_until: HashMap::new(),
            data_dir,
            focus: Focus::Wizard,
            map_cursor: 0,
            map_return_idx: None,
            map_hint_level: MapHintLevel::Groups,
            note_scroll: 0,
            modal: None,
            hint_buffer: String::new(),
            modal_mouse_mode: false,
            modal_restore_snapshot: None,
            modal_stream_transition: None,
            modal_composition_editing: false,
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

    pub fn activate_modal_mouse_mode(&mut self) {
        if self.modal.is_some() {
            self.modal_mouse_mode = true;
        }
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
        self.modal_mouse_mode = false;
        self.modal_restore_snapshot = None;
        self.modal_stream_transition = None;
        self.modal_composition_editing = false;
        self.hint_buffer.clear();
    }

    pub fn dismiss_modal(&mut self) {
        let nested_preview = self
            .modal
            .as_ref()
            .filter(|modal| !modal.nested_stack.is_empty())
            .map(|modal| {
                (
                    modal.preview_field_value(&self.config.sticky_values),
                    compute_field_spans(modal, &self.config.sticky_values),
                )
            });
        if let Some((preview_value, spans)) = nested_preview {
            if let Some(snapshot) = self.modal_restore_snapshot.take() {
                let idx = self.current_idx;
                if let Some(SectionState::Header(state)) = self.section_states.get_mut(idx) {
                    if let Some(visible_count) =
                        state.repeat_visible_counts.get_mut(snapshot.field_idx)
                    {
                        *visible_count = (*visible_count).max(snapshot.value_index + 1);
                    }
                    if let Some(slot) = state.repeated_values.get_mut(snapshot.field_idx) {
                        if snapshot.value_index < slot.len() {
                            slot[snapshot.value_index] = preview_value;
                        } else {
                            slot.push(preview_value);
                        }
                    }
                    state.composite_spans = Some(spans);
                }
            }
        } else if let Some(snapshot) = self.modal_restore_snapshot.take() {
            let idx = self.current_idx;
            if let Some(SectionState::Header(state)) = self.section_states.get_mut(idx) {
                if let Some(slot) = state.repeated_values.get_mut(snapshot.field_idx) {
                    match snapshot.original_value {
                        Some(value) => {
                            if snapshot.value_index < slot.len() {
                                slot[snapshot.value_index] = value;
                            } else {
                                slot.push(value);
                            }
                        }
                        None => {
                            if snapshot.value_index < slot.len() {
                                slot.remove(snapshot.value_index);
                            }
                        }
                    }
                }
                state.composite_spans = snapshot.original_spans;
            }
        }
        self.modal = None;
        self.modal_mouse_mode = false;
        self.modal_stream_transition = None;
        self.modal_composition_editing = false;
        self.hint_buffer.clear();
    }

    pub fn set_modal_query(&mut self, new_text: String) {
        if let Some(modal) = self.modal.as_mut() {
            if modal.is_collection_mode() {
                return;
            }
            modal.query = new_text;
            modal.update_filter();
        }
    }

    pub fn set_modal_composition_text(&mut self, new_text: String) {
        if let Some(modal) = self.modal.as_mut() {
            if modal.is_collection_mode() {
                return;
            }
            modal.manual_override = Some(new_text);
            self.sync_modal_preview_state(self.current_idx);
        }
    }

    fn start_modal_composition_editing(&mut self) {
        let current_text = self
            .modal
            .as_ref()
            .map(|modal| compute_field_preview(modal, &self.config.sticky_values))
            .unwrap_or_default();
        if let Some(modal) = self.modal.as_mut() {
            if modal.is_collection_mode() {
                return;
            }
            if modal.manual_override.is_none() {
                modal.manual_override = Some(current_text);
            }
            self.modal_composition_editing = true;
            self.sync_modal_preview_state(self.current_idx);
        }
    }

    fn reset_modal_composition_override(&mut self) {
        if let Some(modal) = self.modal.as_mut() {
            modal.manual_override = None;
            self.modal_composition_editing = false;
            self.sync_modal_preview_state(self.current_idx);
        }
    }

    fn handle_modal_composition_key(&mut self, key: AppKey) {
        match key {
            AppKey::Tab
            | AppKey::Enter
            | AppKey::ShiftEnter
            | AppKey::Esc
            | AppKey::CtrlChar('e') => {
                self.modal_composition_editing = false;
            }
            AppKey::Backspace => {
                if let Some(modal) = self.modal.as_mut() {
                    if let Some(text) = modal.manual_override.as_mut() {
                        text.pop();
                    }
                }
                self.sync_modal_preview_state(self.current_idx);
            }
            AppKey::CtrlChar('r') => {
                self.reset_modal_composition_override();
            }
            AppKey::Char(c) => {
                if let Some(modal) = self.modal.as_mut() {
                    modal
                        .manual_override
                        .get_or_insert_with(String::new)
                        .push(c);
                }
                self.sync_modal_preview_state(self.current_idx);
            }
            AppKey::Space => {
                if let Some(modal) = self.modal.as_mut() {
                    modal
                        .manual_override
                        .get_or_insert_with(String::new)
                        .push(' ');
                }
                self.sync_modal_preview_state(self.current_idx);
            }
            _ => {}
        }
    }

    pub fn select_modal_filtered_index(&mut self, filtered_index: usize) {
        if self
            .modal
            .as_ref()
            .is_some_and(|modal| modal.is_collection_mode())
        {
            if let Some(modal) = self.modal.as_mut() {
                if let Some(state) = modal.collection_state.as_mut() {
                    match state.focus {
                        crate::sections::collection::CollectionFocus::Collections => {
                            if filtered_index < state.collections.len() {
                                state.collection_cursor = filtered_index;
                            }
                        }
                        crate::sections::collection::CollectionFocus::Items(collection_idx) => {
                            let item_len = state
                                .collections
                                .get(collection_idx)
                                .map(|collection| collection.items.len())
                                .unwrap_or(0);
                            if filtered_index < item_len {
                                state.item_cursor = filtered_index;
                            }
                        }
                    }
                }
            }
            return;
        }
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

    pub fn focus_modal_pane(&mut self, target: ModalPaneTarget) {
        let Some(modal) = self.modal.as_mut() else {
            return;
        };
        let Some(state) = modal.collection_state.as_mut() else {
            return;
        };
        match target {
            ModalPaneTarget::Left => state.exit_items(),
            ModalPaneTarget::Right => state.enter_collection(),
        }
        self.update_collection_modal_preview();
    }

    pub fn hover_modal_row(&mut self, target: ModalPaneTarget, row_index: usize) {
        if !self.modal_mouse_mode {
            return;
        }
        self.set_modal_row(target, row_index, false);
    }

    pub fn press_modal_row(&mut self, target: ModalPaneTarget, row_index: usize) {
        self.set_modal_row(target, row_index, true);
    }

    fn set_modal_row(&mut self, target: ModalPaneTarget, row_index: usize, activate: bool) {
        let Some(modal) = self.modal.as_mut() else {
            return;
        };
        let Some(state) = modal.collection_state.as_mut() else {
            if activate {
                self.select_modal_filtered_index(row_index);
            }
            return;
        };

        match target {
            ModalPaneTarget::Left => {
                if row_index >= state.collections.len() {
                    return;
                }
                let was_focused = matches!(
                    state.focus,
                    crate::sections::collection::CollectionFocus::Collections
                );
                let same_row = state.collection_cursor == row_index;
                state.collection_cursor = row_index;
                state.exit_items();
                if activate && was_focused && same_row {
                    state.toggle_current_collection();
                }
            }
            ModalPaneTarget::Right => {
                let Some(collection) = state.collections.get(state.collection_cursor) else {
                    return;
                };
                if row_index >= collection.items.len() {
                    return;
                }
                let was_focused = matches!(
                    state.focus,
                    crate::sections::collection::CollectionFocus::Items(_)
                );
                let same_row = state.item_cursor == row_index;
                state.enter_collection();
                state.item_cursor = row_index;
                if activate && was_focused && same_row {
                    state.toggle_current_item();
                }
            }
        }
        self.update_collection_modal_preview();
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
                        .or_else(|| data.list_data.get(&cfg.id))
                        .cloned()
                        .unwrap_or_default();
                    SectionState::ListSelect(ListSelectState::new(entries))
                }
                "collection" => {
                    let collections = data
                        .collection_data
                        .get(&cfg.id)
                        .cloned()
                        .unwrap_or_default();
                    SectionState::Collection(CollectionState::new(collections))
                }
                "checklist" => {
                    let items = cfg
                        .data_file
                        .as_ref()
                        .and_then(|f| data.checklist_data.get(f))
                        .or_else(|| data.checklist_data.get(&cfg.id))
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
            .modal_stream_transition
            .as_ref()
            .is_some_and(ModalStreamTransition::is_finished)
        {
            self.modal_stream_transition = None;
        }
        if self
            .copy_flash_until
            .is_some_and(|until| Instant::now() >= until)
        {
            self.copy_flash_until = None;
        }
        self.evicted_collection_flash_until
            .retain(|_, until| Instant::now() < *until);
    }

    pub fn has_active_text_flash(&self) -> bool {
        !self.evicted_collection_flash_until.is_empty()
    }

    pub fn has_active_modal_stream_transition(&self) -> bool {
        self.modal_stream_transition
            .as_ref()
            .is_some_and(|transition| !transition.is_finished())
    }

    fn start_modal_stream_transition(&mut self, previous_modal: Option<SearchModal>) {
        let Some(previous_modal) = previous_modal else {
            self.modal_stream_transition = None;
            return;
        };
        let Some(current_modal) = self.modal.as_ref() else {
            self.modal_stream_transition = None;
            return;
        };

        if previous_modal
            .list_view_snapshot(&self.config.sticky_values)
            .is_none()
            || current_modal
                .list_view_snapshot(&self.config.sticky_values)
                .is_none()
        {
            self.modal_stream_transition = None;
            return;
        }

        let direction = match current_modal
            .field_flow
            .list_idx
            .cmp(&previous_modal.field_flow.list_idx)
        {
            std::cmp::Ordering::Greater => Some(ModalStreamDirection::Forward),
            std::cmp::Ordering::Less => Some(ModalStreamDirection::Backward),
            std::cmp::Ordering::Equal => None,
        };

        self.modal_stream_transition = direction.map(|direction| ModalStreamTransition {
            previous_modal,
            direction,
            started_at: Instant::now(),
            duration: Duration::from_millis(220),
            easing: ModalStreamEasing::ExpoInOut,
        });
    }

    pub fn collection_text_flash_amount(&self, collection_id: &str) -> Option<f32> {
        let until = *self.evicted_collection_flash_until.get(collection_id)?;
        let duration_ms = self.ui_theme.text_color_flash_duration.max(1);
        let remaining_ms = until
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(u128::from(duration_ms)) as f32;
        if remaining_ms <= 0.0 {
            return None;
        }
        let t = remaining_ms / duration_ms as f32;
        Some(t * t * (3.0 - 2.0 * t))
    }

    fn flash_evicted_collections(&mut self, collection_ids: Vec<String>) {
        if collection_ids.is_empty() {
            return;
        }
        let until = Instant::now()
            + std::time::Duration::from_millis(self.ui_theme.text_color_flash_duration);
        for collection_id in collection_ids {
            self.evicted_collection_flash_until
                .insert(collection_id, until);
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
            Some(SectionState::Collection(s)) => !s.in_items(),
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
        self.modal_mouse_mode = false;
        self.modal_stream_transition = None;
        self.hint_buffer.clear();
        self.data = data;
        self.editable_note = build_initial_document(
            &self.data.groups,
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
            .note_label
            .as_deref()
            .and_then(|heading| find_line_containing(&preview, heading))
        {
            return line;
        }

        let Some(group) = self
            .data
            .groups
            .iter()
            .find(|group| group.sections.iter().any(|cfg| cfg.id == section.id))
        else {
            return 0;
        };

        group
            .note
            .note_label
            .as_deref()
            .and_then(|heading| find_line_containing(&preview, heading))
            .unwrap_or(0)
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
            self.modal_mouse_mode = false;
            self.modal = None;
            self.modal_stream_transition = None;
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
            SectionState::Collection(_) => self.handle_collection_key(key),
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
                                .is_some_and(|field| field.max_entries.is_some())
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
                        .cloned()
                        .unwrap_or(HeaderFieldValue::Text(String::new()));
                    crate::sections::multi_field::resolve_multifield_value(
                        &confirmed,
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
                    s.set_current_value(HeaderFieldValue::Text(value));
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
        let (field_idx, field_cfg, current_value, value_index, original_spans) =
            if let Some(SectionState::Header(s)) = self.section_states.get(idx) {
                let field_idx = s.field_index;
                let value_index = s.active_value_index();
                let current_value = s
                    .repeated_values
                    .get(field_idx)
                    .and_then(|values| values.get(value_index))
                    .cloned();
                (
                    field_idx,
                    s.field_configs.get(field_idx).cloned(),
                    current_value,
                    value_index,
                    s.composite_spans.clone(),
                )
            } else {
                return;
            };
        if let Some(cfg) = field_cfg {
            if cfg.lists.is_empty() && cfg.collections.is_empty() && cfg.fields.is_empty() {
                return;
            }
            let window_size = self.modal_window_size();
            let modal = SearchModal::new_field(
                field_idx,
                cfg,
                current_value.as_ref(),
                &self.config.sticky_values,
                window_size,
            );
            self.modal = Some(modal);
            self.modal_mouse_mode = false;
            self.modal_composition_editing = false;
            self.modal_restore_snapshot = Some(ModalRestoreSnapshot {
                field_idx,
                value_index,
                original_value: current_value,
                original_spans,
            });
        }
    }

    fn handle_modal_key(&mut self, key: AppKey) {
        if self.modal_composition_editing {
            self.handle_modal_composition_key(key);
            return;
        }

        if self.is_super_confirm(&key) {
            self.super_confirm_modal_field();
            return;
        }

        if self
            .modal
            .as_ref()
            .is_some_and(|modal| modal.is_collection_mode())
        {
            self.handle_collection_modal_key(key);
            return;
        }

        if matches!(key, AppKey::Esc) {
            self.dismiss_modal();
            return;
        }

        if matches!(key, AppKey::CtrlChar('e')) {
            self.start_modal_composition_editing();
            return;
        }

        if matches!(key, AppKey::CtrlChar('r')) {
            self.reset_modal_composition_override();
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
                AppKey::Up => {}
                AppKey::Down => {
                    if self
                        .modal
                        .as_ref()
                        .is_some_and(|modal| !modal.filtered.is_empty())
                    {
                        self.modal.as_mut().unwrap().focus = ModalFocus::List;
                    }
                }
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
                        self.dismiss_modal();
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
                    } else {
                        modal.focus = ModalFocus::SearchBar;
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

    fn handle_collection_modal_key(&mut self, key: AppKey) {
        match key {
            AppKey::Esc => {
                self.hint_buffer.clear();
                let went_back = self
                    .modal
                    .as_mut()
                    .is_some_and(|modal| modal.collection_back());
                if !went_back {
                    self.dismiss_modal();
                }
            }
            AppKey::Left => {
                self.hint_buffer.clear();
                if self
                    .modal
                    .as_mut()
                    .is_some_and(|modal| modal.collection_back())
                {
                    self.update_collection_modal_preview();
                }
            }
            AppKey::Right => {
                self.hint_buffer.clear();
                if let Some(modal) = self.modal.as_mut() {
                    modal.collection_enter();
                }
                self.update_collection_modal_preview();
            }
            AppKey::Space => {
                self.hint_buffer.clear();
                let in_items = self
                    .modal
                    .as_ref()
                    .and_then(|modal| modal.collection_state.as_ref())
                    .is_some_and(|state| state.in_items());
                if in_items {
                    if let Some(modal) = self.modal.as_mut() {
                        let _ = modal.collection_back();
                    }
                } else if let Some(modal) = self.modal.as_mut() {
                    modal.collection_enter();
                }
                self.update_collection_modal_preview();
            }
            AppKey::Backspace => {
                self.hint_buffer.clear();
                let went_back = self
                    .modal
                    .as_mut()
                    .is_some_and(|modal| modal.collection_back());
                if !went_back {
                    self.dismiss_modal();
                }
            }
            AppKey::Enter => {
                self.hint_buffer.clear();
                if let Some(modal) = self.modal.as_mut() {
                    let evicted = modal.collection_toggle_current();
                    self.flash_evicted_collections(evicted);
                }
                self.update_collection_modal_preview();
            }
            AppKey::Up => {
                self.hint_buffer.clear();
                if let Some(modal) = self.modal.as_mut() {
                    modal.collection_navigate_up();
                }
            }
            AppKey::Down => {
                self.hint_buffer.clear();
                if let Some(modal) = self.modal.as_mut() {
                    modal.collection_navigate_down();
                }
            }
            AppKey::Char(c) => {
                let case_sensitive = self.config.hint_labels_case_sensitive;
                let ch_str: String = if case_sensitive {
                    c.to_string()
                } else {
                    c.to_ascii_lowercase().to_string()
                };
                let visible_count = self.collection_modal_visible_count();
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
                if let Some(hint_pos) = folded_labels.iter().position(|label| label == &ch_str) {
                    self.hint_buffer.clear();
                    self.toggle_collection_modal_hint(hint_pos);
                    self.update_collection_modal_preview();
                }
            }
            _ => {}
        }
    }

    fn collection_modal_visible_count(&self) -> usize {
        let Some(modal) = self.modal.as_ref() else {
            return 0;
        };
        let Some(state) = modal.collection_state.as_ref() else {
            return 0;
        };
        let len = match state.focus {
            crate::sections::collection::CollectionFocus::Collections => state.collections.len(),
            crate::sections::collection::CollectionFocus::Items(collection_idx) => state
                .collections
                .get(collection_idx)
                .map(|collection| collection.items.len())
                .unwrap_or(0),
        };
        let cursor = match state.focus {
            crate::sections::collection::CollectionFocus::Collections => state.collection_cursor,
            crate::sections::collection::CollectionFocus::Items(_) => state.item_cursor,
        };
        let range = modal_hint_window(cursor, len, self.data.keybindings.hints.len());
        range.end.saturating_sub(range.start)
    }

    fn toggle_collection_modal_hint(&mut self, hint_pos: usize) {
        let Some(modal) = self.modal.as_mut() else {
            return;
        };
        let Some(state) = modal.collection_state.as_mut() else {
            return;
        };
        let hint_pool = self.data.keybindings.hints.len();
        match state.focus {
            crate::sections::collection::CollectionFocus::Collections => {
                let range =
                    modal_hint_window(state.collection_cursor, state.collections.len(), hint_pool);
                let target = range.start + hint_pos;
                if target < state.collections.len() {
                    state.collection_cursor = target;
                    let evicted = state.toggle_current_collection();
                    self.flash_evicted_collections(evicted);
                }
            }
            crate::sections::collection::CollectionFocus::Items(collection_idx) => {
                let item_len = state
                    .collections
                    .get(collection_idx)
                    .map(|collection| collection.items.len())
                    .unwrap_or(0);
                let range = modal_hint_window(state.item_cursor, item_len, hint_pool);
                let target = range.start + hint_pos;
                if target < item_len {
                    state.item_cursor = target;
                    state.toggle_current_item();
                }
            }
        }
    }

    fn update_collection_modal_preview(&mut self) {
        let idx = self.current_idx;
        let Some(modal) = self.modal.as_ref() else {
            return;
        };
        if !modal.is_collection_mode() {
            return;
        }
        let preview = modal.preview_field_value(&self.config.sticky_values);
        let spans = compute_field_spans(modal, &self.config.sticky_values);
        if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
            s.set_preview_value(preview);
            s.composite_spans = Some(spans);
        }
    }

    fn sync_modal_preview_state(&mut self, idx: usize) {
        let Some(modal) = self.modal.as_ref() else {
            return;
        };
        let preview_value = modal.preview_field_value(&self.config.sticky_values);
        if let Some(override_text) = modal.manual_override.clone() {
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.set_preview_value(HeaderFieldValue::ManualOverride {
                    text: override_text,
                    source: Box::new(preview_value),
                });
                s.composite_spans = None;
            }
            return;
        }

        let spans = compute_field_spans(modal, &self.config.sticky_values);
        if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
            s.set_preview_value(preview_value);
            s.composite_spans = Some(spans);
        }
    }

    fn confirm_modal_value(&mut self, value: String) {
        let idx = self.current_idx;
        if self.modal.is_some() {
            let previous_modal = self.modal.clone();
            let window_size = self.modal_window_size();
            let advance = self.modal.as_mut().unwrap().advance_field(
                value,
                &mut self.config.sticky_values,
                window_size,
            );
            match advance {
                FieldAdvance::NextList => {
                    self.sync_modal_preview_state(idx);
                    let _ = self.config.save(&self.data_dir);
                    self.start_modal_stream_transition(previous_modal);
                }
                FieldAdvance::StayOnList => {
                    self.sync_modal_preview_state(idx);
                }
                FieldAdvance::Complete(mut final_value) => {
                    if let Some(override_text) = self
                        .modal
                        .as_ref()
                        .and_then(|modal| modal.manual_override.clone())
                    {
                        final_value = HeaderFieldValue::ManualOverride {
                            text: override_text,
                            source: Box::new(final_value),
                        };
                    }
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        s.composite_spans = None;
                        s.set_current_value(final_value);
                        let done = s.advance();
                        self.sync_section_into_editable_note(idx);
                        if done {
                            self.advance_section();
                        }
                    }
                    self.close_modal();
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

        let previous_modal = self.modal.clone();
        let window_size = self.modal_window_size();
        let advance = self
            .modal
            .as_mut()
            .unwrap()
            .super_confirm_field(&mut self.config.sticky_values, window_size);

        match advance {
            FieldAdvance::NextList | FieldAdvance::StayOnList => {
                self.sync_modal_preview_state(idx);
                let _ = self.config.save(&self.data_dir);
                self.start_modal_stream_transition(previous_modal);
            }
            FieldAdvance::Complete(mut final_value) => {
                if let Some(override_text) = self
                    .modal
                    .as_ref()
                    .and_then(|modal| modal.manual_override.clone())
                {
                    final_value = HeaderFieldValue::ManualOverride {
                        text: override_text,
                        source: Box::new(final_value),
                    };
                }
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.composite_spans = None;
                    s.set_current_value(final_value);
                    let done = s.advance();
                    self.sync_section_into_editable_note(idx);
                    if done {
                        self.advance_section();
                    }
                }
                self.close_modal();
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
            self.status = Some(StatusMsg::error(
                "Custom list entry creation was removed in the typed hierarchy cutover.",
            ));
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

    fn handle_collection_key(&mut self, key: AppKey) {
        let idx = self.current_idx;
        let in_items = match &self.section_states[idx] {
            SectionState::Collection(s) => s.in_items(),
            _ => false,
        };

        if in_items {
            if self.is_back(&key) || self.is_confirm(&key) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    s.exit_items();
                }
                return;
            }
            if self.is_navigate_up(&key) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    s.navigate_up();
                }
                return;
            }
            if self.is_navigate_down(&key) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    s.navigate_down();
                }
                return;
            }
            if self.is_select(&key) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    s.toggle_current_item();
                }
                self.sync_section_into_editable_note(idx);
                return;
            }
            if matches!(key, AppKey::Backspace) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    s.reset_current_collection();
                }
                self.sync_section_into_editable_note(idx);
                return;
            }
            return;
        }

        if self.try_navigate_to_map_via_hint(&key) {
            return;
        }
        if self.is_navigate_up(&key) {
            if let SectionState::Collection(s) = &mut self.section_states[idx] {
                s.navigate_up();
            }
            return;
        }
        if self.is_navigate_down(&key) {
            if let SectionState::Collection(s) = &mut self.section_states[idx] {
                s.navigate_down();
            }
            return;
        }
        if self.is_select(&key) {
            if let SectionState::Collection(s) = &mut self.section_states[idx] {
                let evicted = s.toggle_current_collection();
                self.flash_evicted_collections(evicted);
            }
            self.sync_section_into_editable_note(idx);
            return;
        }
        if self.is_confirm(&key) {
            if let SectionState::Collection(s) = &mut self.section_states[idx] {
                if !s.collections.is_empty() {
                    s.enter_collection();
                }
            }
            return;
        }
        if matches!(key, AppKey::Backspace) {
            if let SectionState::Collection(s) = &mut self.section_states[idx] {
                s.reset_current_collection();
            }
            self.sync_section_into_editable_note(idx);
            return;
        }
        if self.is_back(&key) {
            let has_any = match &self.section_states[idx] {
                SectionState::Collection(s) => s.has_any_active_collection(),
                _ => false,
            };
            if let SectionState::Collection(s) = &mut self.section_states[idx] {
                if has_any {
                    s.completed = true;
                } else {
                    s.skipped = true;
                }
            }
            self.sync_section_into_editable_note(idx);
            self.advance_section();
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
            Some(SectionState::Collection(s)) => s.completed,
            Some(SectionState::Checklist(s)) => s.completed,
            _ => false,
        }
    }

    pub fn section_is_skipped(&self, idx: usize) -> bool {
        match self.section_states.get(idx) {
            Some(SectionState::FreeText(s)) => s.skipped,
            Some(SectionState::ListSelect(s)) => s.skipped,
            Some(SectionState::Collection(s)) => s.skipped,
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
        let previous_modal = self.modal.clone();

        let window_size = self.modal_window_size();
        if self
            .modal
            .as_mut()
            .is_some_and(|modal| modal.go_back_one_step(&self.config.sticky_values, window_size))
        {
            self.sync_modal_preview_state(idx);
            self.start_modal_stream_transition(previous_modal);
            return;
        }
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
                .map(|item| item.output().to_string())
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
            self.sync_modal_preview_state(idx);
        }
        self.start_modal_stream_transition(previous_modal);
    }
}

fn compute_field_spans(
    modal: &crate::modal::SearchModal,
    sticky_values: &std::collections::HashMap<String, String>,
) -> Vec<(String, bool)> {
    compute_field_composition_spans(modal, sticky_values)
        .into_iter()
        .map(|span| {
            (
                span.text,
                matches!(span.kind, FieldCompositionSpanKind::Confirmed),
            )
        })
        .collect()
}

pub fn compute_field_composition_spans(
    modal: &crate::modal::SearchModal,
    sticky_values: &std::collections::HashMap<String, String>,
) -> Vec<FieldCompositionSpan> {
    if let Some(root) = modal.nested_stack.first() {
        let resolved = crate::sections::multi_field::resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::NestedState(Box::new(root.state.clone())),
            &root.field,
            sticky_values,
        );
        return match resolved {
            crate::sections::multi_field::ResolvedMultiFieldValue::Complete(value) => {
                vec![FieldCompositionSpan {
                    text: value,
                    kind: FieldCompositionSpanKind::Confirmed,
                }]
            }
            crate::sections::multi_field::ResolvedMultiFieldValue::Partial(value) => {
                vec![FieldCompositionSpan {
                    text: value,
                    kind: FieldCompositionSpanKind::Preview,
                }]
            }
            crate::sections::multi_field::ResolvedMultiFieldValue::Empty => Vec::new(),
        };
    }
    if modal.is_collection_mode() {
        let preview = modal.collection_preview();
        if preview.is_empty() {
            return Vec::new();
        }
        return vec![FieldCompositionSpan {
            text: preview,
            kind: FieldCompositionSpanKind::Confirmed,
        }];
    }
    let flow = &modal.field_flow;
    let Some(format) = &flow.format else {
        if !flow.values.is_empty() {
            return flow
                .values
                .iter()
                .map(|value| FieldCompositionSpan {
                    text: value.clone(),
                    kind: FieldCompositionSpanKind::Confirmed,
                })
                .collect();
        }
        if let Some(list) = flow.lists.get(flow.list_idx) {
            if !flow.repeat_values.is_empty() {
                return vec![FieldCompositionSpan {
                    text: joined_repeating_value(list, &flow.repeat_values)
                        .unwrap_or_else(|| flow.repeat_values.join(", ")),
                    kind: FieldCompositionSpanKind::Confirmed,
                }];
            }
        }
        return Vec::new();
    };
    let mut spans: Vec<FieldCompositionSpan> = Vec::new();
    let mut literal = String::new();
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            if !literal.is_empty() {
                spans.push(FieldCompositionSpan {
                    text: literal.clone(),
                    kind: FieldCompositionSpanKind::Literal,
                });
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
                    spans.push(FieldCompositionSpan {
                        text: flow.values[i].clone(),
                        kind: FieldCompositionSpanKind::Confirmed,
                    });
                } else if i == flow.list_idx && !flow.repeat_values.is_empty() {
                    spans.push(FieldCompositionSpan {
                        text: joined_repeating_value(&flow.lists[i], &flow.repeat_values)
                            .unwrap_or_else(|| flow.repeat_values.join(", ")),
                        kind: FieldCompositionSpanKind::Confirmed,
                    });
                } else if let Some(value) = fallback_list_value(&flow.lists[i], sticky_values) {
                    spans.push(FieldCompositionSpan {
                        text: value,
                        kind: FieldCompositionSpanKind::Preview,
                    });
                } else {
                    let preview = flow.lists[i].preview.as_deref().unwrap_or("?");
                    spans.push(FieldCompositionSpan {
                        text: preview.to_string(),
                        kind: FieldCompositionSpanKind::Preview,
                    });
                }
            } else if let Some(list) = flow.format_lists.iter().find(|list| list.id == id) {
                spans.push(FieldCompositionSpan {
                    text: fallback_list_value(list, sticky_values).unwrap_or_default(),
                    kind: FieldCompositionSpanKind::Preview,
                });
            }
        } else {
            literal.push(c);
        }
    }
    if !literal.is_empty() {
        spans.push(FieldCompositionSpan {
            text: literal,
            kind: FieldCompositionSpanKind::Literal,
        });
    }
    spans
}

fn compute_field_preview(
    modal: &crate::modal::SearchModal,
    sticky_values: &std::collections::HashMap<String, String>,
) -> String {
    if let Some(root) = modal.nested_stack.first() {
        let resolved = crate::sections::multi_field::resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::NestedState(Box::new(root.state.clone())),
            &root.field,
            sticky_values,
        );
        return resolved.display_value().unwrap_or_default().to_string();
    }
    if modal.is_collection_mode() {
        return modal.collection_preview();
    }
    let flow = &modal.field_flow;
    let Some(format) = &flow.format else {
        if !flow.values.is_empty() {
            return flow.values.join(", ");
        }
        if let Some(list) = flow.lists.get(flow.list_idx) {
            if !flow.repeat_values.is_empty() {
                return joined_repeating_value(list, &flow.repeat_values)
                    .unwrap_or_else(|| flow.repeat_values.join(", "));
            }
        }
        return String::new();
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
                || item.ui_label() == *default
                || item.output.as_deref() == Some(default.as_str())
        }) {
            return Some(item.output().to_string());
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

fn modal_hint_window(cursor: usize, len: usize, hint_count: usize) -> std::ops::Range<usize> {
    if len == 0 {
        return 0..0;
    }
    let window_size = hint_count.max(1);
    let start = if cursor >= window_size {
        cursor + 1 - window_size
    } else {
        0
    };
    let end = (start + window_size).min(len);
    start..end
}

#[cfg(all(test, any()))]
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
            collection_data: Default::default(),
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
                max_entries: None,
                max_actives: None,
            },
            HeaderFieldConfig {
                id: "f2".to_string(),
                name: "F2".to_string(),
                options: vec![],
                composite: None,
                default: None,
                max_entries: None,
                max_actives: None,
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
            collection_data: Default::default(),
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
            max_entries: None,
            max_actives: None,
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
            collection_data: Default::default(),
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
            collection_data: Default::default(),
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
            collection_data: Default::default(),
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
            collection_data: Default::default(),
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
            collection_data: Default::default(),
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
            collection_data: Default::default(),
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

#[cfg(test)]
mod composition_span_tests {
    use super::{
        compute_field_composition_spans, App, AppKey, FieldCompositionSpanKind, SectionState,
    };
    use crate::config::Config;
    use crate::data::{
        AppData, GroupNoteMeta, HeaderFieldConfig, HierarchyItem, HierarchyList, KeyBindings,
        ModalStart, RuntimeNodeKind, RuntimeTemplate, SectionConfig, SectionGroup,
    };
    use crate::modal::SearchModal;
    use crate::sections::header::HeaderFieldValue;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn composition_spans_distinguish_literal_confirmed_and_preview_segments() {
        let field = HeaderFieldConfig {
            id: "request".to_string(),
            name: "Request".to_string(),
            format: Some("Treat {place} {region}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "place".to_string(),
                    label: Some("Place".to_string()),
                    preview: Some("?".to_string()),
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "left".to_string(),
                        label: Some("Left".to_string()),
                        default_enabled: true,
                        output: Some("Left".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                    }],
                },
                HierarchyList {
                    id: "region".to_string(),
                    label: Some("Region".to_string()),
                    preview: Some("Region".to_string()),
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "shoulder".to_string(),
                        label: Some("Shoulder".to_string()),
                        default_enabled: true,
                        output: Some("Shoulder".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                    }],
                },
            ],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &sticky_values, 5);
        let _ = modal.advance_field("Left".to_string(), &mut sticky_values, 5);

        let spans = compute_field_composition_spans(&modal, &sticky_values);

        assert_eq!(
            spans
                .iter()
                .map(|span| (&span.text, &span.kind))
                .collect::<Vec<_>>(),
            vec![
                (&"Treat ".to_string(), &FieldCompositionSpanKind::Literal),
                (&"Left".to_string(), &FieldCompositionSpanKind::Confirmed),
                (&" ".to_string(), &FieldCompositionSpanKind::Literal),
                (&"Region".to_string(), &FieldCompositionSpanKind::Preview),
            ]
        );
    }

    #[test]
    fn modal_composition_override_persists_on_confirm_and_reopens_with_source_intact() {
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
                items: vec![HierarchyItem {
                    id: "shoulder".to_string(),
                    label: Some("Shoulder".to_string()),
                    default_enabled: true,
                    output: Some("Shoulder".to_string()),
                    fields: None,
                    branch_fields: Vec::new(),
                }],
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
            data_file: None,
            fields: Some(vec![field]),
            lists: Vec::new(),
            note_label: None,
            group_id: "intake".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let group = SectionGroup {
            id: "intake".to_string(),
            num: None,
            nav_label: "Intake".to_string(),
            sections: vec![section.clone()],
            note: GroupNoteMeta::default(),
        };
        let data = AppData {
            template: RuntimeTemplate {
                id: "test".to_string(),
                children: Vec::new(),
            },
            groups: vec![group],
            sections: vec![section],
            list_data: HashMap::new(),
            checklist_data: HashMap::new(),
            collection_data: HashMap::new(),
            boilerplate_texts: HashMap::new(),
            keybindings: KeyBindings::default(),
        };

        let mut app = App::new(data, Config::default(), PathBuf::new());
        app.open_header_modal();

        app.handle_key(AppKey::CtrlChar('e'));
        app.set_modal_composition_text("Manual shoulder".to_string());
        app.handle_key(AppKey::Enter);
        app.confirm_modal_value("Shoulder".to_string());

        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        match &state.repeated_values[0][0] {
            HeaderFieldValue::ManualOverride { text, source } => {
                assert_eq!(text, "Manual shoulder");
                assert!(matches!(
                    source.as_ref(),
                    HeaderFieldValue::Text(value) if value == "Shoulder"
                ));
            }
            other => panic!("expected manual override, got {other:?}"),
        }
        assert!(app.editable_note.contains("Region: Manual shoulder"));

        app.current_idx = 0;
        app.open_header_modal();

        let modal = app.modal.as_ref().expect("modal should reopen");
        assert_eq!(modal.manual_override.as_deref(), Some("Manual shoulder"));
        assert!(matches!(
            modal.preview_field_value(&app.config.sticky_values),
            HeaderFieldValue::ListState(_)
        ));
    }
}
