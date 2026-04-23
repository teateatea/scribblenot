// Application state and update logic. Transition data types live in transition.rs
// and are re-exported here so callers can use crate::app::* as before.

use crate::config::Config;
use crate::data::{
    flat_sections_from_template, runtime_navigation, AppData, HeaderFieldConfig, KeyBindings,
    NavigationEntry, RuntimeTemplate, SectionConfig,
};
use crate::document::build_initial_document;
use crate::error_report::ErrorReport;
use crate::messages::Messages;
use crate::modal::{
    joined_repeating_value, resolved_item_labels_for_list, FieldAdvance, ListValueLookup,
    SearchModal,
};
use crate::modal_layout::{
    modal_height_for_viewport, modal_window_size_for_height, ModalFocus, SimpleModalUnitLayout,
};
use crate::sections::{
    checklist::ChecklistState,
    collection::CollectionState,
    free_text::FreeTextState,
    header::{HeaderFieldValue, HeaderState},
    list_select::{ListSelectMode, ListSelectState},
};
pub use crate::transition::{
    unit_display_width, FocusDirection, ModalArrivalLayer, ModalCompositionLayer,
    ModalCompositionTransition, ModalDepartureLayer, ModalTransitionEasing, ModalTransitionLayer,
    UnitContentSnapshot, UnitGeometry,
};
use iced::keyboard::{key::Named, Key, Modifiers};
use std::collections::{BTreeMap, HashMap};
use std::ops::{Index, IndexMut};
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

fn shifted_char(c: char) -> char {
    match c {
        'a'..='z' => c.to_ascii_uppercase(),
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        '`' => '~',
        _ => c,
    }
}

fn normalized_ctrl_char(c: char) -> char {
    if c.is_ascii_alphabetic() {
        c.to_ascii_lowercase()
    } else {
        c
    }
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
            let mut c = s.chars().next().unwrap_or('\0');
            if modifiers.contains(Modifiers::CTRL) {
                c = normalized_ctrl_char(c);
                if c == 'c' {
                    AppKey::CtrlC
                } else {
                    AppKey::CtrlChar(c)
                }
            } else if c == ' ' {
                AppKey::Space
            } else {
                if modifiers.contains(Modifiers::SHIFT) {
                    c = shifted_char(c);
                }
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
enum CollectionHintTarget {
    Collection(usize),
    Item {
        collection_idx: usize,
        item_idx: usize,
    },
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
pub struct SectionStateStore {
    order: Vec<String>,
    states: HashMap<String, SectionState>,
}

impl SectionStateStore {
    pub fn new(sections: &[SectionConfig], states: Vec<SectionState>) -> Self {
        assert_eq!(
            sections.len(),
            states.len(),
            "section state count must match runtime section count"
        );

        let order = sections.iter().map(|section| section.id.clone()).collect();
        let states = sections
            .iter()
            .map(|section| section.id.clone())
            .zip(states)
            .collect();

        Self { order, states }
    }

    pub fn get(&self, index: usize) -> Option<&SectionState> {
        let node_id = self.order.get(index)?;
        self.states.get(node_id)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut SectionState> {
        let node_id = self.order.get(index)?.clone();
        self.states.get_mut(&node_id)
    }

    pub fn by_id(&self, node_id: &str) -> Option<&SectionState> {
        self.states.get(node_id)
    }
}

impl Index<usize> for SectionStateStore {
    type Output = SectionState;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("missing section state at index {index}"))
    }
}

impl IndexMut<usize> for SectionStateStore {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .unwrap_or_else(|| panic!("missing section state at index {index}"))
    }
}

#[derive(Debug, Clone)]
pub struct StatusMsg {
    pub text: String,
    pub is_error: bool,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorModalFlashKind {
    Error,
    Copy,
}

#[derive(Debug, Clone, Copy)]
pub struct ErrorModalFlash {
    pub kind: ErrorModalFlashKind,
    pub until: Instant,
}

impl ErrorModalFlash {
    fn new(kind: ErrorModalFlashKind, theme: &crate::theme::AppTheme) -> Self {
        Self {
            kind,
            until: Instant::now()
                + std::time::Duration::from_millis(theme.preview_copy_flash_duration_ms),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldCompositionSpanKind {
    Literal,
    Confirmed,
    Active,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AssignmentSourceKey {
    node_id: String,
    field_idx: usize,
    value_idx: usize,
}

pub struct App {
    pub navigation: Vec<NavigationEntry>,
    pub section_states: SectionStateStore,
    pub current_idx: usize,
    pub data: AppData,
    pub config: Config,
    pub assigned_values: HashMap<String, String>,
    pub ui_theme: crate::theme::AppTheme,
    pub pane_swapped: bool,
    pub show_help: bool,
    pub status: Option<StatusMsg>,
    pub error_modal: Option<ErrorReport>,
    pub error_modal_flash: Option<ErrorModalFlash>,
    pub messages: Messages,
    pub copy_override_text: Option<String>,
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
    /// Precomputed layout for the current modal sequence and viewport.
    /// None before the first layout build or whenever the current modal state does not
    /// support simple-unit layout (e.g. query active, collection mode, etc.).
    pub modal_unit_layout: Option<SimpleModalUnitLayout>,
    /// Index into modal_unit_layout.units identifying the currently active unit.
    /// AppState.active_unit_index is the single canonical source of truth; the layout
    /// struct must not carry an independent copy.
    pub active_unit_index: usize,
    /// Prepared neighbors of the active unit (unit indices). None when no unit exists
    /// in that direction or when modal_unit_layout is None.
    pub prev_prepared_unit: Option<usize>,
    pub next_prepared_unit: Option<usize>,
    /// In-flight animation entries, ordered oldest-to-newest (newest = last / topmost).
    pub modal_transitions: Vec<ModalTransitionLayer>,
    pub modal_composition_transition: Option<ModalCompositionTransition>,
    pub modal_composition_editing: bool,
    pub editable_note: String,
    pub note_headings_valid: bool,
    pub note_structure_warning: Option<String>,
    pub viewport_size: Option<iced::Size>,
    assigned_contributions: BTreeMap<AssignmentSourceKey, HashMap<String, String>>,
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

fn messages_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("messages")
}

fn empty_app_data() -> AppData {
    AppData {
        template: RuntimeTemplate {
            id: "error_state".to_string(),
            children: Vec::new(),
        },
        list_data: HashMap::new(),
        checklist_data: HashMap::new(),
        collection_data: HashMap::new(),
        boilerplate_texts: HashMap::new(),
        keybindings: KeyBindings::default(),
        hotkeys: Default::default(),
    }
}

fn error_report_from_anyhow(err: anyhow::Error) -> ErrorReport {
    match err.downcast::<ErrorReport>() {
        Ok(report) => report,
        Err(err) => ErrorReport::generic("data_load_failed", err.to_string()),
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
        let sections = flat_sections_from_template(&data.template);
        let navigation = runtime_navigation(&data.template);
        let section_states = SectionStateStore::new(&sections, Self::init_states(&sections, &data));
        let pane_swapped = config.is_swapped();
        let editable_note = build_initial_document(
            &data.template,
            &section_states,
            &HashMap::new(),
            &config.sticky_values,
            &data.boilerplate_texts,
        );
        let note_validation =
            crate::document::validate_document_structure(&editable_note, &data.template);
        let note_headings_valid = note_validation.is_ok();
        let note_structure_warning = note_validation.err();
        let ui_theme =
            crate::theme::AppTheme::load(&data_dir, &config.theme).unwrap_or_else(|err| {
                eprintln!("Warning: failed to load theme '{}': {err}", config.theme);
                crate::theme::AppTheme::default()
            });
        let messages = Messages::load(&messages_dir());
        Self {
            navigation,
            section_states,
            current_idx: 0,
            data,
            config,
            assigned_values: HashMap::new(),
            ui_theme,
            pane_swapped,
            show_help: false,
            status: None,
            error_modal: None,
            error_modal_flash: None,
            messages,
            copy_override_text: None,
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
            modal_unit_layout: None,
            active_unit_index: 0,
            prev_prepared_unit: None,
            next_prepared_unit: None,
            modal_transitions: Vec::new(),
            modal_composition_transition: None,
            modal_composition_editing: false,
            editable_note,
            note_headings_valid,
            note_structure_warning,
            viewport_size: None,
            assigned_contributions: BTreeMap::new(),
        }
    }

    pub fn new_error_state(report: ErrorReport, config: Config, data_dir: PathBuf) -> Self {
        let data = empty_app_data();
        let ui_theme =
            crate::theme::AppTheme::load(&data_dir, &config.theme).unwrap_or_else(|err| {
                eprintln!("Warning: failed to load theme '{}': {err}", config.theme);
                crate::theme::AppTheme::default()
            });
        let error_modal_flash = Some(ErrorModalFlash::new(ErrorModalFlashKind::Error, &ui_theme));
        Self {
            navigation: Vec::new(),
            section_states: SectionStateStore::new(&[], Vec::new()),
            current_idx: 0,
            data,
            config,
            assigned_values: HashMap::new(),
            ui_theme,
            pane_swapped: false,
            show_help: false,
            status: None,
            error_modal: Some(report),
            error_modal_flash,
            messages: Messages::load(&messages_dir()),
            copy_override_text: None,
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
            modal_unit_layout: None,
            active_unit_index: 0,
            prev_prepared_unit: None,
            next_prepared_unit: None,
            modal_transitions: Vec::new(),
            modal_composition_transition: None,
            modal_composition_editing: false,
            editable_note: String::new(),
            note_headings_valid: true,
            note_structure_warning: None,
            viewport_size: None,
            assigned_contributions: BTreeMap::new(),
        }
    }

    pub fn set_viewport_size(&mut self, size: iced::Size) {
        self.viewport_size = Some(size);
        let window_size = self.modal_window_size();
        if let Some(modal) = self.modal.as_mut() {
            modal.window_size = window_size;
            modal.update_scroll();
        }
        // Geometry-only rebuild: update layout for new viewport, preserve in-flight layers.
        if self.modal.is_some() {
            self.rebuild_modal_unit_layout(false);
        }
    }

    pub fn modal_window_size(&self) -> usize {
        let modal_height =
            modal_height_for_viewport(self.viewport_size.map(|size| size.height), 360.0);
        modal_window_size_for_height(modal_height, self.data.keybindings.hints.len())
    }

    pub fn modal_spacer_width(&self) -> f32 {
        let viewport_width = self
            .viewport_size
            .map(|size| size.width)
            .unwrap_or(f32::INFINITY);
        crate::modal_layout::effective_spacer_width(
            viewport_width,
            self.ui_theme.modal_spacer_width,
        )
        .max(0.0)
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

    /// Reset all modal transition state. Called when the modal closes or transitions
    /// must be invalidated (e.g. data reload, non-simple-mode entry during animation).
    fn settle_modal_transitions(&mut self) {
        self.modal_transitions.clear();
        self.modal_unit_layout = None;
        self.active_unit_index = 0;
        self.prev_prepared_unit = None;
        self.next_prepared_unit = None;
    }

    /// Rebuild the precomputed modal unit layout from the current modal and viewport state.
    ///
    /// `full_reset = false`: steps 1-4 only (geometry update). In-flight layers are preserved;
    /// their frozen geometry is unaffected by the viewport change.
    ///
    /// `full_reset = true`: steps 1-4 + clear modal_transitions (open, data refresh).
    fn rebuild_modal_unit_layout(&mut self, full_reset: bool) {
        // Step 1: build layout from the current modal state.
        let layout = self.modal.as_ref().and_then(|modal| {
            modal.simple_modal_unit_layout(
                &self.assigned_values,
                &self.config.sticky_values,
                self.viewport_size.map(|s| s.width),
                self.ui_theme.modal_spacer_width,
                self.ui_theme.modal_stub_width,
            )
        });

        match &layout {
            Some(layout_ref) => {
                // Step 2: locate the unit whose range contains the active list index.
                let list_idx = self
                    .modal
                    .as_ref()
                    .map(|m| m.field_flow.list_idx)
                    .unwrap_or(0);
                let n = layout_ref.units.len();
                let active_unit = layout_ref
                    .units
                    .iter()
                    .position(|unit| (unit.start..=unit.end).contains(&list_idx))
                    .unwrap_or(0);

                // Steps 3-4: prepared neighbors.
                self.active_unit_index = active_unit;
                self.prev_prepared_unit = if active_unit > 0 {
                    Some(active_unit - 1)
                } else {
                    None
                };
                self.next_prepared_unit = if active_unit + 1 < n {
                    Some(active_unit + 1)
                } else {
                    None
                };
            }
            None => {
                // Non-simple-mode path: focus remains at the active modal at rest.
                self.active_unit_index = 0;
                self.prev_prepared_unit = None;
                self.next_prepared_unit = None;
                // If we lose simple mode during a transition, settle immediately so the
                // next frame renders the active unit at rest without stale layer data.
                if !self.modal_transitions.is_empty() {
                    self.modal_transitions.clear();
                }
            }
        }

        self.modal_unit_layout = layout;

        // Step 5 (full reset only): clear in-flight transitions.
        if full_reset {
            self.modal_transitions.clear();
        }
    }

    fn clear_live_modal_state_preserving_transitions(&mut self) {
        self.modal = None;
        self.modal_mouse_mode = false;
        self.modal_restore_snapshot = None;
        self.modal_composition_editing = false;
        self.hint_buffer.clear();
        self.modal_unit_layout = None;
        self.active_unit_index = 0;
        self.prev_prepared_unit = None;
        self.next_prepared_unit = None;
    }

    #[allow(dead_code)] // retained as the hard-stop teardown path for non-animated callers
    pub fn close_modal(&mut self) {
        self.modal = None;
        self.modal_mouse_mode = false;
        self.modal_restore_snapshot = None;
        self.settle_modal_transitions();
        self.modal_composition_transition = None;
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
                    compute_field_spans(modal, &self.assigned_values, &self.config.sticky_values),
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
        self.fire_modal_exit_transition_if_possible();
        self.clear_live_modal_state_preserving_transitions();
    }

    pub fn set_modal_query(&mut self, new_text: String) {
        if let Some(modal) = self.modal.as_mut() {
            if modal.is_collection_mode() {
                return;
            }
            modal.query = new_text;
            modal.update_filter();
        }
        // Any query edit settles in-flight transitions then rebuilds layout.
        // A non-empty query will make the modal non-simple, setting modal_unit_layout = None.
        if !self.modal_transitions.is_empty() {
            self.modal_transitions.clear();
        }
        if self.modal.is_some() {
            self.rebuild_modal_unit_layout(false);
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
            .map(|modal| {
                compute_field_preview(modal, &self.assigned_values, &self.config.sticky_values)
            })
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
            AppKey::ShiftEnter => {
                self.modal_composition_editing = false;
                self.super_confirm_modal_field();
            }
            AppKey::Tab | AppKey::Enter | AppKey::Esc | AppKey::CtrlChar('e') => {
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
                    let evicted = state.toggle_current_item();
                    self.flash_evicted_collections(evicted);
                }
            }
        }
        self.update_collection_modal_preview();
    }

    fn refresh_note_structure(&mut self) {
        match crate::document::validate_document_structure(&self.editable_note, &self.data.template)
        {
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

        let Some(cfg) = self.config_for_index(idx).cloned() else {
            return;
        };
        let Some(state) = self.section_states.get(idx) else {
            return;
        };

        let body = crate::note::render_editable_section_body(
            &cfg,
            state,
            &self.assigned_values,
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
        // Per-frame pruning: remove completed transition entries.
        // Part 3 queue drain hook: fire the next queued adaptive transition when an
        // arrival completes (insert here).
        self.modal_transitions.retain(|entry| match entry {
            ModalTransitionLayer::ConnectedTransition { arrival, .. } => !arrival.is_finished(),
            ModalTransitionLayer::ModalOpen { arrival, .. } => !arrival.is_finished(),
            ModalTransitionLayer::ModalClose { departure, .. } => !departure.is_finished(),
        });
        if self
            .modal_composition_transition
            .as_ref()
            .is_some_and(|transition| match transition {
                ModalCompositionTransition::Open { arrival, .. } => arrival.is_finished(),
                ModalCompositionTransition::Close { departure, .. } => departure.is_finished(),
            })
        {
            self.modal_composition_transition = None;
        }
        if self
            .copy_flash_until
            .is_some_and(|until| Instant::now() >= until)
        {
            self.copy_flash_until = None;
        }
        if self
            .error_modal_flash
            .is_some_and(|flash| Instant::now() >= flash.until)
        {
            self.error_modal_flash = None;
        }
        self.evicted_collection_flash_until
            .retain(|_, until| Instant::now() < *until);
    }

    pub fn has_active_text_flash(&self) -> bool {
        !self.evicted_collection_flash_until.is_empty()
    }

    pub fn has_active_error_modal_flash(&self) -> bool {
        self.error_modal_flash.is_some()
    }

    /// Returns true when any transition layer is currently animating.
    pub fn has_active_modal_transition(&self) -> bool {
        !self.modal_transitions.is_empty() || self.modal_composition_transition.is_some()
    }

    pub fn error_modal_flash_amount(&self) -> Option<(ErrorModalFlashKind, f32)> {
        let flash = self.error_modal_flash?;
        let duration_ms = self.ui_theme.preview_copy_flash_duration_ms.max(1);
        let remaining_ms = flash
            .until
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(u128::from(duration_ms)) as f32;
        if remaining_ms <= 0.0 {
            return None;
        }
        let t = remaining_ms / duration_ms as f32;
        Some((flash.kind, t * t * (3.0 - 2.0 * t)))
    }

    pub fn flash_error_modal_copy(&mut self) {
        if self.error_modal.is_some() {
            self.error_modal_flash =
                Some(ErrorModalFlash::new(ErrorModalFlashKind::Copy, &self.ui_theme));
        }
    }

    fn show_error_modal(&mut self, report: ErrorReport) {
        self.error_modal = Some(report);
        self.error_modal_flash =
            Some(ErrorModalFlash::new(ErrorModalFlashKind::Error, &self.ui_theme));
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

    fn is_nav_down(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.nav_down)
    }

    fn is_nav_up(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.nav_up)
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

    fn is_nav_left(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.nav_left)
    }

    fn is_nav_right(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.nav_right)
    }

    fn is_super_confirm(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.super_confirm)
    }

    fn is_refresh_theme(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.theme_reload)
    }

    fn is_refresh_data(&self, key: &AppKey) -> bool {
        self.matches_key(key, &self.data.keybindings.data_reload)
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

    fn active_header_field_can_open_modal(&self) -> bool {
        match self.section_states.get(self.current_idx) {
            Some(SectionState::Header(s)) => {
                s.field_configs.get(s.field_index).is_some_and(|cfg| {
                    !(cfg.lists.is_empty() && cfg.collections.is_empty() && cfg.fields.is_empty())
                })
            }
            _ => false,
        }
    }

    fn try_open_wizard_modal_on_nav_right(&mut self, key: &AppKey) -> bool {
        if self.focus != Focus::Wizard || !self.is_nav_right(key) {
            return false;
        }

        if self.active_header_field_can_open_modal() {
            self.open_header_modal();
            return true;
        }

        false
    }

    fn reset_header_section_cursor_to_first_field(&mut self, idx: usize) {
        if let Some(SectionState::Header(state)) = self.section_states.get_mut(idx) {
            if !state.field_configs.is_empty() {
                state.field_index = 0;
                state.repeat_counts[0] = 0;
            }
            state.completed = false;
        }
    }

    fn handle_map_key(&mut self, key: AppKey) {
        if let AppKey::Char(c) = key {
            let ch_str = self.normalize_hint_char(c);
            let assignments = self.section_hint_assignments_only();
            if self.should_prioritize_authored_hints(&assignments, &ch_str) {
                self.hint_buffer.push_str(&ch_str);
                let typed = self.hint_buffer.clone();
                match self.resolve_hint_assignments(&assignments, &typed) {
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
                return;
            }
        }

        if self.is_nav_down(&key) {
            self.hint_buffer.clear();
            if self.map_cursor + 1 < self.navigation.len() {
                self.map_cursor += 1;
                self.current_idx = self.map_cursor;
                let g = self.group_idx_for_section(self.map_cursor);
                self.map_hint_level = MapHintLevel::Sections(g);
                self.update_note_scroll();
            }
            return;
        }
        if self.is_nav_up(&key) {
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
        if self.is_select(&key) {
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
            let ch_str = self.normalize_hint_char(c);
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();
            let assignments = self.section_hint_assignments_only();
            match self.resolve_hint_assignments(&assignments, &typed) {
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
        self.navigation
            .get(flat_idx)
            .map(|entry| entry.group_index)
            .unwrap_or(0)
    }

    pub fn config_for_node_id(&self, node_id: &str) -> Option<&SectionConfig> {
        self.data
            .template
            .children
            .iter()
            .flat_map(|group| group.children.iter())
            .find_map(|node| (node.config().id == node_id).then_some(node.config()))
    }

    pub fn config_for_index(&self, flat_idx: usize) -> Option<&SectionConfig> {
        let node_id = self.navigation.get(flat_idx)?.node_id.as_str();
        self.config_for_node_id(node_id)
    }

    fn section_hint_assignments_only(&self) -> Vec<crate::data::HintLabelAssignment> {
        let explicit_prefixes: Vec<Option<&str>> = self
            .navigation
            .iter()
            .map(|entry| self.data.hotkeys.section(&entry.node_id))
            .collect();
        crate::data::assign_hint_labels(
            &self.data.keybindings.hints,
            &explicit_prefixes,
            self.config.hint_labels_case_sensitive,
        )
    }

    fn visible_header_field_hotkeys<'a>(&'a self, state: &'a HeaderState) -> Vec<Option<&'a str>> {
        (0..state.visible_row_count())
            .map(|row_idx| {
                let (field_idx, repeat_idx) = state.field_index_for_visible_row(row_idx)?;
                let field = state.field_configs.get(field_idx)?;
                let hotkey = self.data.hotkeys.field(&field.id)?;
                if field.max_entries.is_some() {
                    let active_repeat = state.repeat_counts.get(field_idx).copied().unwrap_or(0);
                    (repeat_idx == active_repeat).then_some(hotkey)
                } else {
                    Some(hotkey)
                }
            })
            .collect()
    }

    fn current_wizard_hint_assignments(
        &self,
    ) -> Option<(
        Vec<crate::data::HintLabelAssignment>,
        Vec<crate::data::HintLabelAssignment>,
    )> {
        let SectionState::Header(state) = self.section_states.get(self.current_idx)? else {
            return None;
        };
        let section_prefixes: Vec<Option<&str>> = self
            .navigation
            .iter()
            .map(|entry| self.data.hotkeys.section(&entry.node_id))
            .collect();
        let field_prefixes = self.visible_header_field_hotkeys(state);
        let field_start = section_prefixes.len();
        let mut explicit_prefixes = section_prefixes;
        explicit_prefixes.extend(field_prefixes);
        let assignments = crate::data::assign_hint_labels(
            &self.data.keybindings.hints,
            &explicit_prefixes,
            self.config.hint_labels_case_sensitive,
        );
        Some((
            assignments[..field_start].to_vec(),
            assignments[field_start..].to_vec(),
        ))
    }

    fn current_section_hint_assignments(&self) -> Vec<crate::data::HintLabelAssignment> {
        if self.modal.is_none() && self.focus == Focus::Wizard {
            if let Some((sections, _fields)) = self.current_wizard_hint_assignments() {
                return sections;
            }
        }
        self.section_hint_assignments_only()
    }

    pub fn section_hint_labels(&self) -> Vec<String> {
        self.current_section_hint_assignments()
            .into_iter()
            .map(|assignment| assignment.label)
            .collect()
    }

    pub fn map_hint_labels(&self, group_idx: Option<usize>) -> MapHintLabels {
        let labels = self.section_hint_labels();
        let Some(group_idx) = group_idx else {
            return MapHintLabels {
                sections: Vec::new(),
            };
        };
        let Some(group) = self.data.template.children.get(group_idx) else {
            return MapHintLabels {
                sections: Vec::new(),
            };
        };
        MapHintLabels {
            sections: labels
                .into_iter()
                .zip(self.navigation.iter())
                .filter_map(|(label, entry)| (entry.group_id == group.id).then_some(label))
                .collect(),
        }
    }

    pub fn wizard_hint_labels(&self) -> WizardHintLabels {
        let labels = self
            .current_wizard_hint_assignments()
            .map(|(_sections, fields)| fields)
            .unwrap_or_default();
        WizardHintLabels {
            fields: labels
                .into_iter()
                .map(|assignment| assignment.label)
                .collect(),
        }
    }

    fn normalize_hint_input(&self, value: &str) -> String {
        if self.config.hint_labels_case_sensitive {
            value.to_string()
        } else {
            value.to_ascii_lowercase()
        }
    }

    fn normalize_hint_char(&self, c: char) -> String {
        self.normalize_hint_input(&c.to_string())
    }

    fn resolve_hint_assignments(
        &self,
        assignments: &[crate::data::HintLabelAssignment],
        typed: &str,
    ) -> crate::data::HintResolveResult {
        let folded_labels: Vec<String> = assignments
            .iter()
            .map(|assignment| self.normalize_hint_input(&assignment.label))
            .collect();
        let refs: Vec<&str> = folded_labels.iter().map(String::as_str).collect();
        crate::data::resolve_hint(&refs, typed)
    }

    fn should_prioritize_authored_hints(
        &self,
        assignments: &[crate::data::HintLabelAssignment],
        ch_str: &str,
    ) -> bool {
        if !self.hint_buffer.is_empty() {
            return true;
        }
        assignments
            .iter()
            .filter(|assignment| assignment.authored)
            .map(|assignment| self.normalize_hint_input(&assignment.label))
            .any(|label| label.starts_with(ch_str))
    }

    fn visible_modal_hint_assignments(&self) -> Vec<crate::data::HintLabelAssignment> {
        let Some(modal) = self.modal.as_ref() else {
            return Vec::new();
        };
        let Some(list) = modal.field_flow.lists.get(modal.field_flow.list_idx) else {
            return Vec::new();
        };
        let end = (modal.list_scroll + modal.window_size).min(modal.filtered.len());
        let explicit_prefixes: Vec<Option<&str>> = (modal.list_scroll..end)
            .filter_map(|window_pos| modal.filtered.get(window_pos))
            .map(|&item_idx| {
                list.items
                    .get(item_idx)
                    .and_then(|item| self.data.hotkeys.item(&list.id, &item.id))
            })
            .collect();
        crate::data::assign_hint_labels(
            &self.data.keybindings.hints,
            &explicit_prefixes,
            self.config.hint_labels_case_sensitive,
        )
    }

    pub fn visible_modal_hint_labels(&self) -> Vec<String> {
        self.visible_modal_hint_assignments()
            .into_iter()
            .map(|assignment| assignment.label)
            .collect()
    }

    fn collection_modal_hint_assignments(&self) -> Vec<crate::data::HintLabelAssignment> {
        let Some(modal) = self.modal.as_ref() else {
            return Vec::new();
        };
        let Some(state) = modal.collection_state.as_ref() else {
            return Vec::new();
        };

        let mut explicit_prefixes = Vec::new();
        let hint_pool = self.data.keybindings.hints.len().max(1);
        let left_range =
            modal_hint_window(state.collection_cursor, state.collections.len(), hint_pool);
        explicit_prefixes.extend(left_range.clone().map(|_| None));

        let remaining = hint_pool.saturating_sub(left_range.end.saturating_sub(left_range.start));
        if remaining > 0 {
            let collection_idx = state.collection_cursor;
            let Some(collection) = state.collections.get(collection_idx) else {
                return crate::data::assign_hint_labels(
                    &self.data.keybindings.hints,
                    &explicit_prefixes,
                    self.config.hint_labels_case_sensitive,
                );
            };
            let item_range =
                modal_hint_window(state.item_cursor, collection.items.len(), remaining);
            explicit_prefixes.extend(item_range.map(|item_idx| {
                let list_id = collection.item_list_ids.get(item_idx)?;
                let item = collection.items.get(item_idx)?;
                self.data.hotkeys.item(list_id, &item.id)
            }));
        }

        crate::data::assign_hint_labels(
            &self.data.keybindings.hints,
            &explicit_prefixes,
            self.config.hint_labels_case_sensitive,
        )
    }

    pub fn collection_modal_hint_labels(&self) -> Vec<String> {
        self.collection_modal_hint_assignments()
            .into_iter()
            .map(|assignment| assignment.label)
            .collect()
    }

    fn update_note_scroll(&mut self) {
        self.note_scroll = self.preview_scroll_line_for_index(self.map_cursor);
    }

    fn assignment_source_key(
        &self,
        section_idx: usize,
        field_idx: usize,
        value_idx: usize,
    ) -> AssignmentSourceKey {
        let node_id = self
            .navigation
            .get(section_idx)
            .map(|entry| entry.node_id.clone())
            .unwrap_or_default();
        AssignmentSourceKey {
            node_id,
            field_idx,
            value_idx,
        }
    }

    fn active_assignment_source_key(&self, section_idx: usize) -> Option<AssignmentSourceKey> {
        let SectionState::Header(state) = self.section_states.get(section_idx)? else {
            return None;
        };
        Some(self.assignment_source_key(section_idx, state.field_index, state.active_value_index()))
    }

    fn modal_assignment_source_key(&self) -> Option<AssignmentSourceKey> {
        let snapshot = self.modal_restore_snapshot.as_ref()?;
        Some(self.assignment_source_key(self.current_idx, snapshot.field_idx, snapshot.value_index))
    }

    fn rebuild_assigned_values(&mut self) {
        self.assigned_values.clear();
        for contributions in self.assigned_contributions.values() {
            for (list_id, value) in contributions {
                if !value.is_empty() {
                    self.assigned_values.insert(list_id.clone(), value.clone());
                }
            }
        }
    }

    fn replace_assignments_for_key(
        &mut self,
        key: AssignmentSourceKey,
        assignments: HashMap<String, String>,
    ) {
        if assignments.is_empty() {
            self.assigned_contributions.remove(&key);
        } else {
            self.assigned_contributions.insert(key, assignments);
        }
        self.rebuild_assigned_values();
    }

    fn clear_assignments_for_key(&mut self, key: &AssignmentSourceKey) {
        self.assigned_contributions.remove(key);
        self.rebuild_assigned_values();
    }

    fn confirmed_assignments_for_current_header_field(
        &self,
        section_idx: usize,
        value: &HeaderFieldValue,
    ) -> HashMap<String, String> {
        let Some(SectionState::Header(state)) = self.section_states.get(section_idx) else {
            return HashMap::new();
        };
        state
            .field_configs
            .get(state.field_index)
            .map(|cfg| {
                crate::modal::confirmed_value_assignments(value, cfg, &self.config.sticky_values)
            })
            .unwrap_or_default()
    }

    fn finalize_modal_completion_value(
        &self,
        modal: &SearchModal,
        final_value: HeaderFieldValue,
    ) -> HeaderFieldValue {
        match final_value {
            HeaderFieldValue::Text(_) => modal
                .confirmed_field_value_if_complete(&self.config.sticky_values)
                .unwrap_or(final_value),
            other => other,
        }
    }

    fn header_value_is_structurally_confirmed(
        value: &HeaderFieldValue,
        cfg: &HeaderFieldConfig,
    ) -> bool {
        match value {
            HeaderFieldValue::ExplicitEmpty => true,
            HeaderFieldValue::Text(_) | HeaderFieldValue::ManualOverride { .. } => true,
            HeaderFieldValue::ListState(state) => state.list_idx >= cfg.lists.len(),
            HeaderFieldValue::CollectionState(_) => true,
            HeaderFieldValue::NestedState(state) => !cfg.fields.is_empty() && state.completed,
        }
    }

    pub fn reload_theme(&mut self) -> anyhow::Result<()> {
        self.ui_theme = crate::theme::AppTheme::load(&self.data_dir, &self.config.theme)?;
        Ok(())
    }

    pub fn reload_data(&mut self) -> std::result::Result<(), ErrorReport> {
        let previous_section_id = self
            .navigation
            .get(self.current_idx)
            .map(|entry| entry.node_id.clone());
        let data = AppData::load(self.data_dir.clone()).map_err(error_report_from_anyhow)?;
        self.navigation = runtime_navigation(&data.template);
        let sections = flat_sections_from_template(&data.template);
        self.section_states =
            SectionStateStore::new(&sections, Self::init_states(&sections, &data));
        self.current_idx = previous_section_id
            .as_ref()
            .and_then(|id| {
                self.navigation
                    .iter()
                    .position(|entry| &entry.node_id == id)
            })
            .unwrap_or(0)
            .min(self.navigation.len().saturating_sub(1));
        self.map_cursor = self.current_idx;
        self.map_return_idx = None;
        self.map_hint_level = MapHintLevel::Sections(self.group_idx_for_section(self.current_idx));
        self.modal = None;
        self.modal_mouse_mode = false;
        self.settle_modal_transitions();
        self.hint_buffer.clear();
        self.assigned_values.clear();
        self.assigned_contributions.clear();
        self.editable_note = build_initial_document(
            &data.template,
            &self.section_states,
            &self.assigned_values,
            &self.config.sticky_values,
            &data.boilerplate_texts,
        );
        self.data = data;
        self.error_modal = None;
        self.refresh_note_structure();
        self.update_note_scroll();
        Ok(())
    }

    pub fn error_modal_markdown(&self) -> Option<String> {
        let report = self.error_modal.as_ref()?;
        let rendered = self.messages.render(report);
        let mut markdown = format!("# {}\n\n", rendered.title);
        markdown.push_str(&format!("**Error ID:** `{}`\n\n", rendered.id));
        markdown.push_str(&rendered.description);
        markdown.push_str("\n\n");
        if let Some(source) = rendered.source {
            markdown.push_str("## Source\n\n");
            markdown.push_str(&format!("{}\n\n", source.location));
            if let Some(quoted_line) = source.quoted_line {
                markdown.push_str("```yaml\n");
                markdown.push_str(&quoted_line);
                markdown.push_str("\n```\n\n");
            }
        }
        if !rendered.fix.trim().is_empty() {
            markdown.push_str("## Fix\n\n");
            markdown.push_str(&rendered.fix);
            markdown.push('\n');
        }
        Some(markdown)
    }

    pub fn current_preview_scroll_line(&self) -> u16 {
        match self.focus {
            Focus::Map => self.note_scroll,
            Focus::Wizard => self.preview_scroll_line_for_index(self.current_idx),
        }
    }

    fn preview_scroll_line_for_index(&self, idx: usize) -> u16 {
        let Some(section) = self.config_for_index(idx) else {
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
            .navigation
            .get(idx)
            .and_then(|entry| self.data.template.children.get(entry.group_index))
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
        let line = group_idx + self.map_cursor;
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

        if self.error_modal.is_some() {
            if self.is_copy_note(&key) {
                self.copy_override_text = self.error_modal_markdown();
                self.copy_requested = self.copy_override_text.is_some();
                return;
            }
            if self.is_back(&key) || self.is_refresh_data(&key) {
                match self.reload_data() {
                    Ok(()) => {
                        self.status = Some(StatusMsg::success("Data refreshed from YAML."));
                    }
                    Err(report) => self.show_error_modal(report),
                }
            }
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
            self.settle_modal_transitions();
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
                Err(report) => self.show_error_modal(report),
            }
            return;
        }

        if self.is_swap_panes(&key) {
            self.pane_swapped = !self.pane_swapped;
            self.config.set_swapped(self.pane_swapped);
            let _ = self.config.save(&self.data_dir);
            return;
        }

        if self.try_open_wizard_modal_on_nav_right(&key) {
            return;
        }

        // Focus switching: nav_left moves left in layout, nav_right moves right
        // Map is always on the outside: left when default, right when swapped
        if self.is_nav_left(&key) {
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

        if self.is_nav_right(&key) {
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
                if self.is_confirm(&key) || self.is_nav_down(&key) {
                    self.advance_section();
                }
            }
        }
    }

    fn advance_section(&mut self) {
        if self.current_idx + 1 < self.navigation.len() {
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
            let ch_str = self.normalize_hint_char(c);
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();

            let idx = self.current_idx;
            let (section_assignments, field_assignments) =
                self.current_wizard_hint_assignments().unwrap_or_default();
            match self.resolve_hint_assignments(&section_assignments, &typed) {
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
            match self.resolve_hint_assignments(&field_assignments, &typed) {
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
            let committed_value =
                if let Some(SectionState::Header(s)) = self.section_states.get(idx) {
                    let value_index = s.active_value_index();
                    s.field_configs.get(s.field_index).and_then(|cfg| {
                        let confirmed = s
                            .repeated_values
                            .get(s.field_index)
                            .and_then(|values| values.get(value_index))
                            .cloned();
                        if let Some(value) = confirmed.as_ref().filter(|value| {
                            Self::header_value_is_structurally_confirmed(value, cfg)
                        }) {
                            return Some(value.clone());
                        }
                        match crate::sections::multi_field::resolve_multifield_value(
                            &HeaderFieldValue::Text(String::new()),
                            cfg,
                            &self.assigned_values,
                            &self.config.sticky_values,
                        ) {
                            crate::sections::multi_field::ResolvedMultiFieldValue::Complete(
                                value,
                            ) => Some(HeaderFieldValue::Text(value)),
                            _ => None,
                        }
                    })
                } else {
                    None
                };
            if let Some(committed_value) = committed_value {
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.set_current_value(committed_value);
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
            let assignment_key = self.active_assignment_source_key(idx);
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.blank_active_value();
                s.completed = false;
                if let Some(key) = assignment_key.as_ref() {
                    self.clear_assignments_for_key(key);
                }
                self.sync_section_into_editable_note(idx);
            }
            return;
        }

        if self.is_back(&key) || self.is_nav_up(&key) {
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

        if self.is_nav_down(&key) {
            let idx = self.current_idx;
            let advance_to_next_section =
                if let Some(SectionState::Header(state)) = self.section_states.get_mut(idx) {
                    if state.field_configs.is_empty() {
                        false
                    } else {
                        let last = state.field_configs.len() - 1;
                        if state.field_index > last {
                            // Normalize out-of-bounds (completed) index to last field.
                            state.field_index = last;
                            false
                        } else if state.field_index == last {
                            true
                        } else {
                            let _ = state.go_forward();
                            false
                        }
                    }
                } else {
                    false
                };
            if advance_to_next_section {
                self.advance_section();
                self.reset_header_section_cursor_to_first_field(self.current_idx);
            }
            return;
        }

        if self.is_select(&key) || self.is_nav_right(&key) {
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
                &self.assigned_values,
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
            // Full reset: build layout for the newly opened modal and clear any stale transitions.
            self.rebuild_modal_unit_layout(true);
            self.fire_modal_open_transition_if_possible();
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

        let focus = match &self.modal {
            Some(m) => m.focus.clone(),
            None => return,
        };

        let nav_left_active = match focus {
            ModalFocus::SearchBar => matches!(key, AppKey::Left),
            ModalFocus::List => self.is_nav_left(&key),
        };
        if nav_left_active {
            self.composite_go_back();
            return;
        }

        let nav_right_active = match focus {
            ModalFocus::SearchBar => matches!(key, AppKey::Right),
            ModalFocus::List => self.is_nav_right(&key),
        };
        if nav_right_active {
            if !self.composite_go_forward() && matches!(focus, ModalFocus::List) {
                let _ = self.confirm_modal_from_confirmed_state();
            }
            return;
        }

        match focus {
            ModalFocus::SearchBar => {
                if matches!(key, AppKey::Up) {
                    return;
                }

                if matches!(key, AppKey::Down) {
                    if self
                        .modal
                        .as_ref()
                        .is_some_and(|modal| !modal.filtered.is_empty())
                    {
                        self.modal.as_mut().unwrap().focus = ModalFocus::List;
                    }
                    return;
                }

                if self.is_confirm(&key) {
                    if self
                        .modal
                        .as_ref()
                        .is_some_and(|modal| modal.should_confirm_empty_search_value())
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
                    return;
                }

                if matches!(key, AppKey::Backspace) {
                    let modal = self.modal.as_mut().unwrap();
                    modal.query.pop();
                    modal.update_filter();
                    if modal.query.is_empty() {
                        modal.center_scroll();
                    }
                    return;
                }

                if let AppKey::Char(c) = key {
                    let modal = self.modal.as_mut().unwrap();
                    modal.query.push(c);
                    modal.update_filter();
                    return;
                }

                if matches!(key, AppKey::Space) {
                    if self
                        .modal
                        .as_ref()
                        .is_some_and(|modal| modal.query.is_empty())
                    {
                        if self
                            .modal
                            .as_ref()
                            .is_some_and(|modal| !modal.filtered.is_empty())
                        {
                            self.modal.as_mut().unwrap().focus = ModalFocus::List;
                        }
                        return;
                    }

                    let modal = self.modal.as_mut().unwrap();
                    modal.query.push(' ');
                    modal.update_filter();
                    return;
                }

                return;
            }
            ModalFocus::List => {
                if let AppKey::Char(c) = key {
                    let ch_str = self.normalize_hint_char(c);
                    let assignments = self.visible_modal_hint_assignments();
                    if self.should_prioritize_authored_hints(&assignments, &ch_str) {
                        self.hint_buffer.push_str(&ch_str);
                        let typed = self.hint_buffer.clone();
                        match self.resolve_hint_assignments(&assignments, &typed) {
                            crate::data::HintResolveResult::Exact(hint_pos) => {
                                self.hint_buffer.clear();
                                if let Some(val) = self
                                    .modal
                                    .as_ref()
                                    .and_then(|modal| modal.hint_value(hint_pos))
                                    .map(String::from)
                                {
                                    self.confirm_modal_value(val);
                                }
                            }
                            crate::data::HintResolveResult::Partial(_) => {}
                            crate::data::HintResolveResult::NoMatch => {
                                self.hint_buffer.clear();
                            }
                        }
                        return;
                    }
                }

                if matches!(key, AppKey::Backspace) {
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
                    return;
                }

                if matches!(key, AppKey::Space) {
                    self.hint_buffer.clear();
                    self.modal.as_mut().unwrap().focus = ModalFocus::SearchBar;
                    return;
                }

                if self.is_confirm(&key) {
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
                    return;
                }

                if self.is_nav_up(&key) {
                    self.hint_buffer.clear();
                    let modal = self.modal.as_mut().unwrap();
                    if modal.list_cursor > 0 {
                        modal.list_cursor -= 1;
                        modal.update_scroll();
                    } else {
                        modal.focus = ModalFocus::SearchBar;
                    }
                    return;
                }

                if self.is_nav_down(&key) {
                    self.hint_buffer.clear();
                    let modal = self.modal.as_mut().unwrap();
                    if modal.list_cursor + 1 < modal.filtered.len() {
                        modal.list_cursor += 1;
                        modal.update_scroll();
                    }
                    return;
                }

                if let AppKey::Char(c) = key {
                    let ch_str = self.normalize_hint_char(c);
                    self.hint_buffer.push_str(&ch_str);
                    let typed = self.hint_buffer.clone();
                    let assignments = self.visible_modal_hint_assignments();
                    match self.resolve_hint_assignments(&assignments, &typed) {
                        crate::data::HintResolveResult::Exact(hint_pos) => {
                            self.hint_buffer.clear();
                            if let Some(val) = self
                                .modal
                                .as_ref()
                                .and_then(|modal| modal.hint_value(hint_pos))
                                .map(String::from)
                            {
                                self.confirm_modal_value(val);
                            }
                        }
                        crate::data::HintResolveResult::Partial(_) => return,
                        crate::data::HintResolveResult::NoMatch => {
                            self.hint_buffer.clear();
                        }
                    }
                    return;
                }

                return;
            }
        }
    }

    fn handle_collection_modal_key(&mut self, key: AppKey) {
        if matches!(key, AppKey::Esc) {
            self.hint_buffer.clear();
            let went_back = self
                .modal
                .as_mut()
                .is_some_and(|modal| modal.collection_back());
            if !went_back {
                self.dismiss_modal();
            }
            return;
        }

        if let AppKey::Char(c) = key {
            let ch_str = self.normalize_hint_char(c);
            let assignments = self.collection_modal_hint_assignments();
            if self.should_prioritize_authored_hints(&assignments, &ch_str) {
                self.hint_buffer.push_str(&ch_str);
                let typed = self.hint_buffer.clone();
                match self.resolve_hint_assignments(&assignments, &typed) {
                    crate::data::HintResolveResult::Exact(hint_pos) => {
                        self.hint_buffer.clear();
                        self.toggle_collection_modal_hint(hint_pos);
                        self.update_collection_modal_preview();
                    }
                    crate::data::HintResolveResult::Partial(_) => {}
                    crate::data::HintResolveResult::NoMatch => {
                        self.hint_buffer.clear();
                    }
                }
                return;
            }
        }

        if self.is_nav_left(&key) {
            self.hint_buffer.clear();
            if self
                .modal
                .as_mut()
                .is_some_and(|modal| modal.collection_back())
            {
                self.update_collection_modal_preview();
            } else {
                self.composite_go_back();
            }
            return;
        }

        if self.is_nav_right(&key) {
            self.hint_buffer.clear();
            let in_items = self
                .modal
                .as_ref()
                .and_then(|modal| modal.collection_state.as_ref())
                .is_some_and(|state| state.in_items());
            if in_items {
                let _ = self.confirm_modal_from_confirmed_state();
            } else if let Some(modal) = self.modal.as_mut() {
                modal.collection_enter();
                self.update_collection_modal_preview();
            }
            return;
        }

        if self.is_select(&key) {
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
            return;
        }

        if matches!(key, AppKey::Backspace) {
            self.hint_buffer.clear();
            let went_back = self
                .modal
                .as_mut()
                .is_some_and(|modal| modal.collection_back());
            if !went_back {
                self.dismiss_modal();
            }
            return;
        }

        if self.is_confirm(&key) {
            self.hint_buffer.clear();
            if let Some(modal) = self.modal.as_mut() {
                let evicted = modal.collection_toggle_current();
                self.flash_evicted_collections(evicted);
            }
            self.update_collection_modal_preview();
            return;
        }

        if self.is_nav_up(&key) {
            self.hint_buffer.clear();
            if let Some(modal) = self.modal.as_mut() {
                modal.collection_navigate_up();
            }
            return;
        }

        if self.is_nav_down(&key) {
            self.hint_buffer.clear();
            if let Some(modal) = self.modal.as_mut() {
                modal.collection_navigate_down();
            }
            return;
        }

        if let AppKey::Char(c) = key {
            let ch_str = self.normalize_hint_char(c);
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();
            let assignments = self.collection_modal_hint_assignments();
            match self.resolve_hint_assignments(&assignments, &typed) {
                crate::data::HintResolveResult::Exact(hint_pos) => {
                    self.hint_buffer.clear();
                    self.toggle_collection_modal_hint(hint_pos);
                    self.update_collection_modal_preview();
                }
                crate::data::HintResolveResult::Partial(_) => return,
                crate::data::HintResolveResult::NoMatch => {
                    self.hint_buffer.clear();
                }
            }
        }
    }

    pub fn collection_modal_left_hint_count(&self) -> usize {
        let Some(modal) = self.modal.as_ref() else {
            return 0;
        };
        let Some(state) = modal.collection_state.as_ref() else {
            return 0;
        };
        let hint_pool = self.data.keybindings.hints.len();
        modal_hint_window(state.collection_cursor, state.collections.len(), hint_pool).len()
    }

    fn collection_modal_hint_targets(&self) -> Vec<CollectionHintTarget> {
        let Some(modal) = self.modal.as_ref() else {
            return Vec::new();
        };
        let Some(state) = modal.collection_state.as_ref() else {
            return Vec::new();
        };
        let hint_pool = self.data.keybindings.hints.len();
        let mut targets = Vec::new();
        let left_range =
            modal_hint_window(state.collection_cursor, state.collections.len(), hint_pool);
        targets.extend(left_range.clone().map(CollectionHintTarget::Collection));

        let remaining = hint_pool.saturating_sub(targets.len());
        if remaining > 0 {
            let collection_idx = state.collection_cursor;
            let item_len = state
                .collections
                .get(collection_idx)
                .map(|collection| collection.items.len())
                .unwrap_or(0);
            let item_range = modal_hint_window(state.item_cursor, item_len, remaining);
            targets.extend(item_range.map(|item_idx| CollectionHintTarget::Item {
                collection_idx,
                item_idx,
            }));
        }
        targets
    }

    fn toggle_collection_modal_hint(&mut self, hint_pos: usize) {
        let Some(target) = self.collection_modal_hint_targets().get(hint_pos).copied() else {
            return;
        };
        let Some(modal) = self.modal.as_mut() else {
            return;
        };
        let Some(state) = modal.collection_state.as_mut() else {
            return;
        };
        match target {
            CollectionHintTarget::Collection(collection_idx) => {
                if collection_idx < state.collections.len() {
                    state.collection_cursor = collection_idx;
                    let evicted = state.toggle_current_collection();
                    self.flash_evicted_collections(evicted);
                }
            }
            CollectionHintTarget::Item {
                collection_idx,
                item_idx,
            } => {
                let item_len = state
                    .collections
                    .get(collection_idx)
                    .map(|collection| collection.items.len())
                    .unwrap_or(0);
                if item_idx >= item_len {
                    return;
                }
                let restore_to_collections = matches!(
                    state.focus,
                    crate::sections::collection::CollectionFocus::Collections
                );
                state.collection_cursor = collection_idx;
                state.enter_collection();
                state.item_cursor = item_idx;
                let evicted = state.toggle_current_item();
                if restore_to_collections {
                    state.exit_items();
                }
                self.flash_evicted_collections(evicted);
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
        let spans = compute_field_spans(modal, &self.assigned_values, &self.config.sticky_values);
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

        let spans = compute_field_spans(modal, &self.assigned_values, &self.config.sticky_values);
        if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
            s.set_preview_value(preview_value);
            s.composite_spans = Some(spans);
        }
    }

    fn confirm_modal_value(&mut self, value: String) {
        let idx = self.current_idx;
        if self.modal.is_some() {
            // Dual-layout capture: snapshot pre-mutation state for departure geometry.
            let previous_layout = self.modal_unit_layout.clone();
            let previous_modal = self.modal.as_ref().unwrap().clone();
            let confirm_snapshot_override = self.modal.as_ref().and_then(|modal| {
                modal.preview_current_list_as_confirmed(
                    Some(value.as_str()),
                    &self.assigned_values,
                    &self.config.sticky_values,
                )
            });
            let window_size = self.modal_window_size();
            let advance = self.modal.as_mut().unwrap().advance_field(
                value,
                &self.assigned_values,
                &mut self.config.sticky_values,
                window_size,
            );
            match advance {
                FieldAdvance::NextList => {
                    self.sync_modal_preview_state(idx);
                    let _ = self.config.save(&self.data_dir);
                    self.rebuild_modal_unit_layout(false);
                    self.fire_modal_transition_if_needed(
                        previous_layout,
                        previous_modal,
                        confirm_snapshot_override,
                    );
                }
                FieldAdvance::StayOnList => {
                    self.sync_modal_preview_state(idx);
                }
                FieldAdvance::Complete(final_value) => {
                    let mut committed_value = self
                        .modal
                        .as_ref()
                        .map(|modal| {
                            self.finalize_modal_completion_value(modal, final_value.clone())
                        })
                        .unwrap_or(final_value);
                    if let Some(override_text) = self
                        .modal
                        .as_ref()
                        .and_then(|modal| modal.manual_override.clone())
                    {
                        committed_value = HeaderFieldValue::ManualOverride {
                            text: override_text,
                            source: Box::new(committed_value),
                        };
                    }
                    let modal_assignments =
                        self.confirmed_assignments_for_current_header_field(idx, &committed_value);
                    if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                        s.composite_spans = None;
                        s.set_current_value(committed_value);
                        let done = s.advance();
                        if let Some(key) = self.modal_assignment_source_key() {
                            self.replace_assignments_for_key(key, modal_assignments);
                        }
                        self.sync_section_into_editable_note(idx);
                        if done {
                            self.advance_section();
                        }
                    }
                    self.fire_modal_confirm_transition_if_possible(confirm_snapshot_override);
                    self.clear_live_modal_state_preserving_transitions();
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

        // Dual-layout capture: snapshot pre-mutation state for departure geometry.
        let previous_layout = self.modal_unit_layout.clone();
        let previous_modal = self.modal.as_ref().unwrap().clone();
        let confirm_snapshot_override = self.modal.as_ref().and_then(|modal| {
            modal.preview_current_list_as_confirmed(
                None,
                &self.assigned_values,
                &self.config.sticky_values,
            )
        });
        let window_size = self.modal_window_size();
        let advance = self.modal.as_mut().unwrap().super_confirm_field(
            &self.assigned_values,
            &mut self.config.sticky_values,
            window_size,
        );

        match advance {
            FieldAdvance::NextList | FieldAdvance::StayOnList => {
                self.sync_modal_preview_state(idx);
                let _ = self.config.save(&self.data_dir);
                self.rebuild_modal_unit_layout(false);
                self.fire_modal_transition_if_needed(
                    previous_layout,
                    previous_modal,
                    confirm_snapshot_override,
                );
            }
            FieldAdvance::Complete(final_value) => {
                let mut committed_value = self
                    .modal
                    .as_ref()
                    .map(|modal| self.finalize_modal_completion_value(modal, final_value.clone()))
                    .unwrap_or(final_value);
                if let Some(override_text) = self
                    .modal
                    .as_ref()
                    .and_then(|modal| modal.manual_override.clone())
                {
                    committed_value = HeaderFieldValue::ManualOverride {
                        text: override_text,
                        source: Box::new(committed_value),
                    };
                }
                let modal_assignments =
                    self.confirmed_assignments_for_current_header_field(idx, &committed_value);
                if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                    s.composite_spans = None;
                    s.set_current_value(committed_value);
                    let done = s.advance();
                    if let Some(key) = self.modal_assignment_source_key() {
                        self.replace_assignments_for_key(key, modal_assignments);
                    }
                    self.sync_section_into_editable_note(idx);
                    if done {
                        self.advance_section();
                    }
                }
                self.fire_modal_confirm_transition_if_possible(confirm_snapshot_override);
                self.clear_live_modal_state_preserving_transitions();
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

        if self.is_nav_up(&key) {
            if let SectionState::FreeText(s) = &mut self.section_states[idx] {
                s.navigate_up();
            }
            return;
        }

        if self.is_nav_down(&key) {
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
        if self.is_nav_up(&key) {
            if let SectionState::ListSelect(s) = &mut self.section_states[idx] {
                s.navigate_up();
            }
            return;
        }
        if self.is_nav_down(&key) {
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
            if self.is_nav_up(&key) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    s.navigate_up();
                }
                return;
            }
            if self.is_nav_down(&key) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    s.navigate_down();
                }
                return;
            }
            if self.is_select(&key) {
                if let SectionState::Collection(s) = &mut self.section_states[idx] {
                    let _ = s.toggle_current_item();
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
        if self.is_nav_up(&key) {
            if let SectionState::Collection(s) = &mut self.section_states[idx] {
                s.navigate_up();
            }
            return;
        }
        if self.is_nav_down(&key) {
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

        if self.is_nav_up(&key) {
            if let SectionState::Checklist(s) = &mut self.section_states[idx] {
                s.navigate_up();
            }
            return;
        }
        if self.is_nav_down(&key) {
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
            let ch_str = self.normalize_hint_char(c);
            self.hint_buffer.push_str(&ch_str);
            let typed = self.hint_buffer.clone();
            let assignments = self.section_hint_assignments_only();
            match self.resolve_hint_assignments(&assignments, &typed) {
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

        // Dual-layout capture: snapshot pre-mutation state for departure geometry.
        let previous_layout = self.modal_unit_layout.clone();
        let previous_modal = match self.modal.as_ref() {
            Some(m) => m.clone(),
            None => return,
        };

        let window_size = self.modal_window_size();
        if self.modal.as_mut().is_some_and(|modal| {
            modal.go_back_one_step(
                &self.assigned_values,
                &self.config.sticky_values,
                window_size,
            )
        }) {
            self.sync_modal_preview_state(idx);
            self.rebuild_modal_unit_layout(false);
            self.fire_modal_transition_if_needed(previous_layout, previous_modal, None);
            return;
        }
        let (new_list_idx, popped_output, popped_item_id, new_labels, new_outputs) = {
            let modal = match self.modal.as_mut() {
                Some(m) => m,
                None => return,
            };
            if modal.field_flow.list_idx == 0 {
                let restored = modal.restore_parent_branch(
                    &self.assigned_values,
                    &self.config.sticky_values,
                    window_size,
                );
                if !restored {
                    self.dismiss_modal();
                }
                return;
            }
            let popped = modal.field_flow.values.pop();
            let popped_item_id = modal.field_flow.item_ids.pop();
            modal.field_flow.list_idx -= 1;
            let list = &modal.field_flow.lists[modal.field_flow.list_idx];
            let mut merged_assigned = self.assigned_values.clone();
            merged_assigned.extend(modal.assigned_values(&self.config.sticky_values));
            let labels = resolved_item_labels_for_list(
                list,
                &modal.field_flow.values,
                &modal.field_flow.repeat_values,
                &modal.field_flow.lists,
                &modal.field_flow.format_lists,
                crate::modal::ListValueLookup::new(&merged_assigned, &self.config.sticky_values),
            );
            let outputs: Vec<String> = list
                .items
                .iter()
                .map(|item| item.output().to_string())
                .collect();
            (
                modal.field_flow.list_idx,
                popped,
                popped_item_id,
                labels,
                outputs,
            )
        };

        let cursor = if let Some(item_id) = popped_item_id.as_ref() {
            self.modal
                .as_ref()
                .and_then(|modal| {
                    modal
                        .field_flow
                        .lists
                        .get(new_list_idx)
                        .and_then(|list| list.items.iter().position(|item| &item.id == item_id))
                })
                .or_else(|| {
                    popped_output
                        .as_ref()
                        .and_then(|v| new_outputs.iter().position(|e| e == v))
                })
                .unwrap_or(0)
        } else {
            popped_output
                .as_ref()
                .and_then(|v| new_outputs.iter().position(|e| e == v))
                .unwrap_or(0)
        };
        let assignment_key = if new_list_idx == 0 {
            self.modal_assignment_source_key()
        } else {
            None
        };
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
            if let Some(key) = assignment_key.as_ref() {
                self.clear_assignments_for_key(key);
            }
            if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
                s.composite_spans = None;
                s.clear_active_value();
            }
        } else {
            self.sync_modal_preview_state(idx);
        }
        self.rebuild_modal_unit_layout(false);
        self.fire_modal_transition_if_needed(previous_layout, previous_modal, None);
    }

    fn composite_go_forward(&mut self) -> bool {
        let idx = self.current_idx;

        let previous_layout = self.modal_unit_layout.clone();
        let previous_modal = match self.modal.as_ref() {
            Some(m) => m.clone(),
            None => return false,
        };

        let window_size = self.modal_window_size();
        if self.modal.as_mut().is_some_and(|modal| {
            modal.move_right_without_confirm(
                &self.assigned_values,
                &self.config.sticky_values,
                window_size,
            )
        }) {
            self.sync_modal_preview_state(idx);
            self.rebuild_modal_unit_layout(false);
            self.fire_modal_transition_if_needed(previous_layout, previous_modal, None);
            return true;
        }
        false
    }

    fn confirm_modal_from_confirmed_state(&mut self) -> bool {
        let idx = self.current_idx;
        let Some(modal) = self.modal.as_ref() else {
            return false;
        };
        let Some(mut committed_value) =
            modal.confirmed_field_value_if_complete(&self.config.sticky_values)
        else {
            return false;
        };
        if let Some(override_text) = modal.manual_override.clone() {
            committed_value = HeaderFieldValue::ManualOverride {
                text: override_text,
                source: Box::new(committed_value),
            };
        }
        let modal_assignments =
            self.confirmed_assignments_for_current_header_field(idx, &committed_value);
        if let Some(SectionState::Header(s)) = self.section_states.get_mut(idx) {
            s.composite_spans = None;
            s.set_current_value(committed_value);
            let done = s.advance();
            if let Some(key) = self.modal_assignment_source_key() {
                self.replace_assignments_for_key(key, modal_assignments);
            }
            self.sync_section_into_editable_note(idx);
            if done {
                self.advance_section();
            }
        }
        self.fire_modal_confirm_transition_if_possible(None);
        self.clear_live_modal_state_preserving_transitions();
        let _ = self.config.save(&self.data_dir);
        true
    }

    fn modal_lifecycle_slide_distance(&self, geometry: &UnitGeometry) -> f32 {
        let unit_width = unit_display_width(geometry, self.ui_theme.modal_stub_width);
        let viewport_width = self
            .viewport_size
            .map(|size| size.width)
            .unwrap_or(unit_width);
        (viewport_width + unit_width) * 0.5 + self.modal_spacer_width()
    }

    fn fire_modal_composition_open_transition_if_possible(&mut self) {
        let Some(modal) = self.modal.as_ref() else {
            return;
        };
        if modal.is_collection_mode() || self.modal_unit_layout.is_none() {
            return;
        }
        let effective_spacer_width = self.modal_spacer_width();
        let duration_ms = self.ui_theme.modal_transition_duration.max(1) as u64;
        let easing = self.ui_theme.modal_transition_easing;
        let Some(geometry) = self.modal_unit_layout.as_ref().and_then(|layout| {
            UnitGeometry::from_layout(
                layout,
                self.active_unit_index,
                modal,
                &self.assigned_values,
                &self.config.sticky_values,
                effective_spacer_width,
            )
        }) else {
            return;
        };

        self.modal_composition_transition = Some(ModalCompositionTransition::Open {
            arrival: ModalCompositionLayer {
                modal: modal.clone(),
                focus_direction: FocusDirection::Forward,
                started_at: Instant::now(),
                duration_ms,
                easing,
            },
            slide_distance: self.modal_lifecycle_slide_distance(&geometry),
        });
    }

    fn fire_modal_open_transition_if_possible(&mut self) {
        let effective_spacer_width = self.modal_spacer_width();
        let duration_ms = self.ui_theme.modal_transition_duration.max(1) as u64;
        let easing = self.ui_theme.modal_transition_easing;
        let arriving_unit_index = self.active_unit_index;

        let arrival_geometry = {
            let Some(layout) = self.modal_unit_layout.as_ref() else {
                return;
            };
            let Some(modal) = self.modal.as_ref() else {
                return;
            };
            UnitGeometry::from_layout(
                layout,
                arriving_unit_index,
                modal,
                &self.assigned_values,
                &self.config.sticky_values,
                effective_spacer_width,
            )
        };

        let Some(arrival_geometry) = arrival_geometry else {
            return;
        };

        let slide_distance = self.modal_lifecycle_slide_distance(&arrival_geometry);
        let arrival = ModalArrivalLayer {
            unit_index: arriving_unit_index,
            geometry: arrival_geometry,
            focus_direction: FocusDirection::Forward,
            started_at: Instant::now(),
            duration_ms,
            easing,
        };

        self.modal_transitions
            .push(ModalTransitionLayer::ModalOpen {
                arrival,
                slide_distance,
            });
        self.fire_modal_composition_open_transition_if_possible();
    }

    fn fire_modal_composition_close_transition_if_possible(
        &mut self,
        focus_direction: FocusDirection,
    ) {
        let Some(modal) = self.modal.as_ref() else {
            return;
        };
        if modal.is_collection_mode() || self.modal_unit_layout.is_none() {
            return;
        }
        let effective_spacer_width = self.modal_spacer_width();
        let duration_ms = self.ui_theme.modal_transition_duration.max(1) as u64;
        let easing = self.ui_theme.modal_transition_easing;
        let Some(geometry) = self.modal_unit_layout.as_ref().and_then(|layout| {
            UnitGeometry::from_layout(
                layout,
                self.active_unit_index,
                modal,
                &self.assigned_values,
                &self.config.sticky_values,
                effective_spacer_width,
            )
        }) else {
            return;
        };

        self.modal_composition_transition = Some(ModalCompositionTransition::Close {
            departure: ModalCompositionLayer {
                modal: modal.clone(),
                focus_direction,
                started_at: Instant::now(),
                duration_ms,
                easing,
            },
            slide_distance: self.modal_lifecycle_slide_distance(&geometry),
        });
    }

    fn fire_modal_close_transition_if_possible(
        &mut self,
        focus_direction: FocusDirection,
        active_snapshot_override: Option<crate::modal_layout::ModalListViewSnapshot>,
    ) {
        let effective_spacer_width = self.modal_spacer_width();
        let duration_ms = self.ui_theme.modal_transition_duration.max(1) as u64;
        let easing = self.ui_theme.modal_transition_easing;
        let departing_unit_index = self.active_unit_index;

        let (departure_geometry, departure_content) = {
            let Some(layout) = self.modal_unit_layout.as_ref() else {
                return;
            };
            let Some(modal) = self.modal.as_ref() else {
                return;
            };
            (
                UnitGeometry::from_layout(
                    layout,
                    departing_unit_index,
                    modal,
                    &self.assigned_values,
                    &self.config.sticky_values,
                    effective_spacer_width,
                ),
                UnitContentSnapshot::from_layout_with_active_override(
                    layout,
                    departing_unit_index,
                    active_snapshot_override,
                ),
            )
        };

        let (Some(departure_geometry), Some(departure_content)) =
            (departure_geometry, departure_content)
        else {
            return;
        };

        let slide_distance = self.modal_lifecycle_slide_distance(&departure_geometry);
        let departure = ModalDepartureLayer {
            content: departure_content,
            geometry: departure_geometry,
            modal: self.modal.as_ref().cloned(),
            focus_direction,
            started_at: Instant::now(),
            duration_ms,
            easing,
        };

        self.modal_transitions
            .push(ModalTransitionLayer::ModalClose {
                departure,
                slide_distance,
            });
        self.fire_modal_composition_close_transition_if_possible(focus_direction);
    }

    fn fire_modal_exit_transition_if_possible(&mut self) {
        self.fire_modal_close_transition_if_possible(FocusDirection::Backward, None);
    }

    fn fire_modal_confirm_transition_if_possible(
        &mut self,
        active_snapshot_override: Option<crate::modal_layout::ModalListViewSnapshot>,
    ) {
        self.fire_modal_close_transition_if_possible(
            FocusDirection::Forward,
            active_snapshot_override,
        );
    }

    /// Evaluate whether focus crossed a unit boundary after a modal mutation and, if so,
    /// create a `ConnectedTransition` entry in `modal_transitions`.
    ///
    /// Call sites must capture `previous_layout` and `previous_modal` BEFORE the mutation,
    /// then call `rebuild_modal_unit_layout(false)` BEFORE calling this function so that
    /// `self.modal_unit_layout` and `self.active_unit_index` reflect post-mutation state.
    fn fire_modal_transition_if_needed(
        &mut self,
        previous_layout: Option<SimpleModalUnitLayout>,
        previous_modal: SearchModal,
        active_snapshot_override: Option<crate::modal_layout::ModalListViewSnapshot>,
    ) {
        // Both current and previous states must be in simple mode.
        if self.modal_unit_layout.is_none() || self.modal.is_none() {
            return;
        }
        let Some(ref _prev_layout_check) = previous_layout else {
            return;
        };

        let old_list_idx = previous_modal.field_flow.list_idx;
        let new_list_idx = self.modal.as_ref().unwrap().field_flow.list_idx;

        let focus_direction = match new_list_idx.cmp(&old_list_idx) {
            std::cmp::Ordering::Greater => FocusDirection::Forward,
            std::cmp::Ordering::Less => FocusDirection::Backward,
            std::cmp::Ordering::Equal => return,
        };

        let arriving_unit_index = self.active_unit_index;

        // Collect values that require only shared borrows before we borrow self mutably.
        let effective_spacer_width = self.modal_spacer_width();
        let stub_width = self.ui_theme.modal_stub_width;
        let duration_ms = self.ui_theme.modal_transition_duration.max(1) as u64;
        let easing = self.ui_theme.modal_transition_easing;

        let prev_layout = previous_layout.as_ref().unwrap();
        let departing_unit_index = prev_layout
            .units
            .iter()
            .position(|unit| (unit.start..=unit.end).contains(&old_list_idx))
            .unwrap_or(0);

        // No cross-unit movement: skip.
        if departing_unit_index == arriving_unit_index {
            return;
        }

        // Build frozen geometry inside a block so the borrows of self end before the push.
        let n;
        let dep_geometry;
        let arr_geometry;
        let dep_content;
        {
            let curr_layout = self.modal_unit_layout.as_ref().unwrap();
            n = curr_layout.units.len();
            dep_geometry = UnitGeometry::from_layout(
                prev_layout,
                departing_unit_index,
                &previous_modal,
                &self.assigned_values,
                &self.config.sticky_values,
                effective_spacer_width,
            );
            arr_geometry = UnitGeometry::from_layout(
                curr_layout,
                arriving_unit_index,
                self.modal.as_ref().unwrap(),
                &self.assigned_values,
                &self.config.sticky_values,
                effective_spacer_width,
            );
            dep_content = UnitContentSnapshot::from_layout_with_active_override(
                prev_layout,
                departing_unit_index,
                active_snapshot_override,
            );
        }

        let (Some(dep_geometry), Some(arr_geometry), Some(dep_content)) =
            (dep_geometry, arr_geometry, dep_content)
        else {
            return;
        };

        // Precompute slide_distance from the actual unit widths.
        // Both show stubs: the units share a transition stub, so the gap between their
        // centres shrinks by one stub_width. Otherwise a plain spacer separates them.
        let dep_unit_width = unit_display_width(&dep_geometry, stub_width);
        let arr_unit_width = unit_display_width(&arr_geometry, stub_width);
        let slide_distance = if dep_geometry.shows_stubs && arr_geometry.shows_stubs {
            ((dep_unit_width + arr_unit_width) / 2.0 - stub_width).max(0.0)
        } else {
            (dep_unit_width + arr_unit_width) / 2.0 + effective_spacer_width
        };

        let now = Instant::now();
        let departure = ModalDepartureLayer {
            content: dep_content,
            geometry: dep_geometry,
            modal: Some(previous_modal),
            focus_direction,
            started_at: now,
            duration_ms,
            easing,
        };
        let arrival = ModalArrivalLayer {
            unit_index: arriving_unit_index,
            geometry: arr_geometry,
            focus_direction,
            started_at: now,
            duration_ms,
            easing,
        };

        self.modal_transitions
            .push(ModalTransitionLayer::ConnectedTransition {
                arrival,
                departure,
                slide_distance,
            });

        // Update prepared neighbors for the new active unit.
        self.prev_prepared_unit = if arriving_unit_index > 0 {
            Some(arriving_unit_index - 1)
        } else {
            None
        };
        self.next_prepared_unit = if arriving_unit_index + 1 < n {
            Some(arriving_unit_index + 1)
        } else {
            None
        };
    }
}

fn compute_field_spans(
    modal: &crate::modal::SearchModal,
    assigned_values: &std::collections::HashMap<String, String>,
    sticky_values: &std::collections::HashMap<String, String>,
) -> Vec<(String, bool)> {
    compute_field_composition_spans(modal, assigned_values, sticky_values)
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
    assigned_values: &std::collections::HashMap<String, String>,
    sticky_values: &std::collections::HashMap<String, String>,
) -> Vec<FieldCompositionSpan> {
    if let Some(root) = modal.nested_stack.first() {
        let resolved = crate::sections::multi_field::resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::NestedState(Box::new(root.state.clone())),
            &root.field,
            assigned_values,
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
    let mut merged_assigned = assigned_values.clone();
    merged_assigned.extend(modal.assigned_values(sticky_values));
    let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
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
    let active_value = modal.selected_value().map(str::to_string);
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
                } else if i == flow.list_idx {
                    if let Some(value) = active_value.clone().filter(|value| !value.is_empty()) {
                        spans.push(FieldCompositionSpan {
                            text: value,
                            kind: FieldCompositionSpanKind::Active,
                        });
                    } else if let Some(value) = fallback_list_value(&flow.lists[i], lookup) {
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
                } else if let Some(value) = fallback_list_value(&flow.lists[i], lookup) {
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
                    text: fallback_list_value(list, lookup).unwrap_or_default(),
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
    assigned_values: &std::collections::HashMap<String, String>,
    sticky_values: &std::collections::HashMap<String, String>,
) -> String {
    if let Some(root) = modal.nested_stack.first() {
        let resolved = crate::sections::multi_field::resolve_multifield_value(
            &crate::sections::header::HeaderFieldValue::NestedState(Box::new(root.state.clone())),
            &root.field,
            assigned_values,
            sticky_values,
        );
        return resolved.display_value().unwrap_or_default().to_string();
    }
    if modal.is_collection_mode() {
        return modal.collection_preview();
    }
    let flow = &modal.field_flow;
    let mut merged_assigned = assigned_values.clone();
    merged_assigned.extend(modal.assigned_values(sticky_values));
    let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
    let active_value = modal.selected_value().map(str::to_string);
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
        if let Some(value) = active_value.filter(|value| !value.is_empty()) {
            return value;
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
        } else if i == flow.list_idx {
            active_value
                .clone()
                .filter(|value| !value.is_empty())
                .or_else(|| fallback_list_value(list, lookup))
                .unwrap_or_else(|| list.preview.as_deref().unwrap_or("?").to_string())
        } else {
            fallback_list_value(list, lookup)
                .unwrap_or_else(|| list.preview.as_deref().unwrap_or("?").to_string())
        };
        result = result.replace(&placeholder, &value);
    }
    for list in &flow.format_lists {
        let placeholder = format!("{{{}}}", list.id);
        if result.contains(&placeholder) {
            let value = fallback_list_value(list, lookup).unwrap_or_default();
            result = result.replace(&placeholder, &value);
        }
    }
    result
}

fn fallback_list_value(
    list: &crate::data::HierarchyList,
    lookup: ListValueLookup<'_>,
) -> Option<String> {
    lookup.fallback_value(list)
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
        if let Some(SectionState::Header(s)) = app.section_states.get(0) {
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
        if let Some(SectionState::Header(s)) = app.section_states.get(0) {
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
        compute_field_composition_spans, App, AppKey, FieldCompositionSpanKind, Focus,
        FocusDirection, ModalCompositionTransition, ModalTransitionLayer, SectionState,
    };
    use crate::config::Config;
    use crate::data::{
        flat_sections_from_template, AppData, GroupNoteMeta, HeaderFieldConfig, HierarchyItem,
        HierarchyList, ItemAssignment, JoinerStyle, KeyBindings, ModalStart,
        ResolvedCollectionConfig, RuntimeGroup, RuntimeNode, RuntimeNodeKind, RuntimeTemplate,
        SectionConfig,
    };
    use crate::error_report::ErrorReport;
    use crate::modal::SearchModal;
    use crate::modal_layout::ModalFocus;
    use crate::sections::header::HeaderFieldValue;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn select_only_filtered_modal_match(app: &mut App, query: &str) {
        app.set_modal_query(query.to_string());
        let filtered_len = app
            .modal
            .as_ref()
            .map(|modal| modal.filtered.len())
            .unwrap_or(0);
        assert_eq!(
            filtered_len, 1,
            "expected exactly one filtered match for query '{query}'"
        );
        app.handle_key(AppKey::Enter);
    }

    fn empty_item(id: &str) -> HierarchyItem {
        item(id, "(Empty)", "")
    }

    fn observation_field_with_repeating_search_lists() -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "observation".to_string(),
            name: "Observation".to_string(),
            format: Some("{place}{muscle}: {tag}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "muscle".to_string(),
                    label: Some("Muscle".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("empty_space".to_string()),
                    modal_start: ModalStart::Search,
                    joiner_style: Some(JoinerStyle::Comma),
                    max_entries: None,
                    items: vec![
                        empty_item("empty_space"),
                        item("trap", "Trapezius Upper", "Trapezius (Upper Fibers)"),
                    ],
                },
                HierarchyList {
                    id: "place".to_string(),
                    label: Some("Place".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("empty_space".to_string()),
                    modal_start: ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![empty_item("empty_space"), item("left", "Left", "Left ")],
                },
                HierarchyList {
                    id: "tag".to_string(),
                    label: Some("Tag".to_string()),
                    preview: None,
                    sticky: false,
                    default: Some("empty_space".to_string()),
                    modal_start: ModalStart::Search,
                    joiner_style: Some(JoinerStyle::Comma),
                    max_entries: None,
                    items: vec![
                        empty_item("empty_space"),
                        item("rmt", "Increased RMT", "Increased Resting Muscle Tension"),
                    ],
                },
            ],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    fn request_field_with_nested_repeat_terminator() -> HeaderFieldConfig {
        let appointment_type = HeaderFieldConfig {
            id: "appointment_type".to_string(),
            name: "Appointment Type".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "appointment_type".to_string(),
                label: Some("Appointment Type".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![item(
                    "relax",
                    "Relaxation massage",
                    "Relaxation massage, focusing on ",
                )],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let region = HeaderFieldConfig {
            id: "region".to_string(),
            name: "Region".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "region".to_string(),
                label: Some("Region".to_string()),
                preview: None,
                sticky: false,
                default: Some("empty_space".to_string()),
                modal_start: ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![
                    empty_item("empty_space"),
                    item("shoulder", "Shoulder", "Shoulder"),
                ],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let place = HeaderFieldConfig {
            id: "place".to_string(),
            name: "Place".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "place".to_string(),
                label: Some("Place".to_string()),
                preview: None,
                sticky: false,
                default: Some("empty_space".to_string()),
                modal_start: ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![empty_item("empty_space"), item("left", "Left", "Left ")],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let single_region = HeaderFieldConfig {
            id: "single_region".to_string(),
            name: "Requested Region".to_string(),
            format: Some("{place}{region}".to_string()),
            preview: None,
            fields: vec![region, place],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let requested_regions = HeaderFieldConfig {
            id: "requested_regions".to_string(),
            name: "Requested Regions".to_string(),
            format: Some("{single_region}".to_string()),
            preview: None,
            fields: vec![single_region],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: Some(JoinerStyle::CommaAndThe),
            max_entries: Some(2),
            max_actives: None,
        };
        HeaderFieldConfig {
            id: "request".to_string(),
            name: "Request".to_string(),
            format: Some("{appointment_type}{requested_regions}".to_string()),
            preview: None,
            fields: vec![appointment_type, requested_regions],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    fn item(id: &str, label: &str, output: &str) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: Some(output.to_string()),
            fields: None,
            branch_fields: Vec::new(),
            assigns: Vec::new(),
        }
    }

    fn item_with_assignment(
        id: &str,
        label: &str,
        output: &str,
        list_id: &str,
        item_id: &str,
        assigned_output: &str,
    ) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: Some(output.to_string()),
            fields: None,
            branch_fields: Vec::new(),
            assigns: vec![ItemAssignment {
                list_id: list_id.to_string(),
                item_id: item_id.to_string(),
                output: assigned_output.to_string(),
            }],
        }
    }

    fn list_field(modal_start: ModalStart) -> HeaderFieldConfig {
        HeaderFieldConfig {
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
                modal_start,
                joiner_style: None,
                max_entries: None,
                items: vec![
                    item("shoulder", "Shoulder", "Shoulder"),
                    item("hip", "Hip", "Hip"),
                ],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    fn empty_capable_search_field() -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "optional_region".to_string(),
            name: "Optional Region".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "optional_region".to_string(),
                label: Some("Optional Region".to_string()),
                preview: Some("Region".to_string()),
                sticky: false,
                default: Some("empty_space".to_string()),
                modal_start: ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![
                    empty_item("empty_space"),
                    item("shoulder", "Shoulder", "Shoulder"),
                ],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    fn collection_field() -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: vec![ResolvedCollectionConfig {
                id: "neck".to_string(),
                label: "Neck".to_string(),
                note_label: Some("##### Neck".to_string()),
                default_enabled: false,
                joiner_style: Some(JoinerStyle::CommaAnd),
                lists: vec![HierarchyList {
                    id: "neck_list".to_string(),
                    label: Some("Neck".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("one", "One", "Upper traps"), item("two", "Two", "SCM")],
                }],
            }],
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    fn assigned_time_field() -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "appointment".to_string(),
            name: "Appointment".to_string(),
            format: Some("{start_hour}:{start_minute}{am_pm}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "start_hour".to_string(),
                    label: Some("Start Hour".to_string()),
                    preview: Some("hh".to_string()),
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        item_with_assignment("hour_9", "9", "9", "am_pm", "am_item", "AM"),
                        item_with_assignment("hour_12", "12", "12", "am_pm", "pm_item", "PM"),
                    ],
                },
                HierarchyList {
                    id: "start_minute".to_string(),
                    label: Some("Start Minute".to_string()),
                    preview: Some("mm".to_string()),
                    sticky: false,
                    default: Some("minute_00".to_string()),
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("minute_00", "00", "00"), item("minute_45", "45", "45")],
                },
            ],
            collections: Vec::new(),
            format_lists: vec![HierarchyList {
                id: "am_pm".to_string(),
                label: Some("AM/PM".to_string()),
                preview: Some("XM".to_string()),
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![item("am_item", "AM", "AM"), item("pm_item", "PM", "PM")],
            }],
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    fn repeating_assigned_time_field(max_entries: usize) -> HeaderFieldConfig {
        let mut field = assigned_time_field();
        field.max_entries = Some(max_entries);
        field
    }

    fn scheduled_visit_field() -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "scheduled_visit".to_string(),
            name: "Scheduled Visit".to_string(),
            format: Some("{start_hour}:{start_minute} for {duration} minutes".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "start_hour".to_string(),
                    label: Some("Start Hour".to_string()),
                    preview: Some("hh".to_string()),
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("hour_9", "9", "9"), item("hour_12", "12", "12")],
                },
                HierarchyList {
                    id: "start_minute".to_string(),
                    label: Some("Start Minute".to_string()),
                    preview: Some("mm".to_string()),
                    sticky: false,
                    default: Some("minute_00".to_string()),
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("minute_00", "00", "00"), item("minute_45", "45", "45")],
                },
                HierarchyList {
                    id: "duration".to_string(),
                    label: Some("Duration".to_string()),
                    preview: Some("dur".to_string()),
                    sticky: false,
                    default: Some("duration_60".to_string()),
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        item("duration_30", "30", "30"),
                        item("duration_45", "45", "45"),
                        item("duration_60", "60", "60"),
                    ],
                },
            ],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    fn app_with_single_field_in_data_dir(field: HeaderFieldConfig, data_dir: PathBuf) -> App {
        let section = SectionConfig {
            id: "request_section".to_string(),
            name: "Request".to_string(),
            map_label: "Request".to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: Some(vec![field]),
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

        App::new(data, Config::default(), data_dir)
    }

    fn app_with_single_field(field: HeaderFieldConfig) -> App {
        app_with_single_field_in_data_dir(field, PathBuf::new())
    }

    fn app_with_fields(fields: Vec<HeaderFieldConfig>) -> App {
        let section = SectionConfig {
            id: "request_section".to_string(),
            name: "Request".to_string(),
            map_label: "Request".to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: Some(fields),
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

    fn temp_app_with_single_field(field: HeaderFieldConfig) -> (App, TempDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        let app = app_with_single_field_in_data_dir(field, temp.path().to_path_buf());
        (app, temp)
    }

    #[test]
    fn refresh_data_failure_sets_error_modal() {
        let (mut app, temp) = temp_app_with_single_field(list_field(ModalStart::List));
        std::fs::write(
            temp.path().join("broken.yml"),
            concat!(
                "template:\n  contains:\n    - group: intake\n",
                "groups:\n  - id: intake\n    contains:\n      - section: missing\n",
            ),
        )
        .expect("fixture writes");

        app.handle_key(AppKey::Char('\\'));

        let report = app
            .error_modal
            .as_ref()
            .expect("reload failure should open error modal");
        assert!(report.message.contains("missing section 'missing'"));
        assert!(app.status.is_none());
    }

    #[test]
    fn copy_key_prepares_error_modal_markdown() {
        let report = ErrorReport::generic("yaml_parse_failed", "bad yaml");
        let mut app = App::new_error_state(report, Config::default(), PathBuf::new());

        app.handle_key(AppKey::Char('c'));

        assert!(app.copy_requested);
        let copied = app
            .copy_override_text
            .as_ref()
            .expect("error modal copy should set markdown payload");
        assert!(copied.contains("# YAML Parse Error"));
        assert!(copied.contains("**Error ID:** `yaml_parse_failed`"));
        assert!(!app.quit);
    }

    fn app_with_free_text_sections(sections: Vec<SectionConfig>) -> App {
        let template_children = sections
            .iter()
            .map(|section| RuntimeGroup {
                id: section.group_id.clone(),
                nav_label: section.name.clone(),
                note: GroupNoteMeta::default(),
                children: vec![RuntimeNode::Section(section.clone())],
            })
            .collect::<Vec<_>>();
        let data = AppData {
            template: RuntimeTemplate {
                id: "test".to_string(),
                children: template_children,
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

    fn app_with_header_sections(sections: Vec<SectionConfig>) -> App {
        let data = AppData {
            template: RuntimeTemplate {
                id: "test".to_string(),
                children: vec![RuntimeGroup {
                    id: "intake".to_string(),
                    nav_label: "Intake".to_string(),
                    note: GroupNoteMeta::default(),
                    children: sections.iter().cloned().map(RuntimeNode::Section).collect(),
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

    fn free_text_section(id: &str, name: &str, group_id: &str) -> SectionConfig {
        SectionConfig {
            id: id.to_string(),
            name: name.to_string(),
            map_label: name.to_string(),
            section_type: "free_text".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: None,
            lists: Vec::new(),
            note_label: None,
            group_id: group_id.to_string(),
            node_kind: RuntimeNodeKind::Section,
        }
    }

    fn header_section(id: &str, name: &str, fields: Vec<HeaderFieldConfig>) -> SectionConfig {
        SectionConfig {
            id: id.to_string(),
            name: name.to_string(),
            map_label: name.to_string(),
            section_type: "multi_field".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: Some(fields),
            lists: Vec::new(),
            note_label: None,
            group_id: "intake".to_string(),
            node_kind: RuntimeNodeKind::Section,
        }
    }

    fn read_saved_config(dir: &std::path::Path) -> String {
        std::fs::read_to_string(dir.join("config.yml")).expect("config.yml read failed")
    }

    fn save_config_to_temp(cfg: &Config, dir: &std::path::Path) -> String {
        cfg.save(dir).expect("config save failed");
        read_saved_config(dir)
    }

    fn assert_no_patient_text(haystack: &str, sentinel: &str, label: &str) {
        assert!(
            !haystack.contains(sentinel),
            "{label}: sentinel patient text found in persisted output\nsentinel: {sentinel:?}\noutput:\n{haystack}"
        );
    }

    #[test]
    fn config_save_does_not_include_editable_note() {
        let sentinel = "PATIENT: Jane Doe DOB 1990-01-01 SENTINEL";
        let (mut app, temp) = temp_app_with_single_field(list_field(ModalStart::List));
        app.editable_note = sentinel.to_string();

        let saved = save_config_to_temp(&app.config, temp.path());

        assert_no_patient_text(&saved, sentinel, "direct config save");
    }

    #[test]
    fn config_save_does_not_include_editable_note_after_modal_confirm() {
        let sentinel = "PATIENT: Jane Doe DOB 1990-01-01 SENTINEL";
        let (mut app, temp) = temp_app_with_single_field(list_field(ModalStart::List));
        app.editable_note = sentinel.to_string();
        app.open_header_modal();

        app.confirm_modal_value("Shoulder".to_string());

        let saved = read_saved_config(temp.path());
        assert_no_patient_text(&saved, sentinel, "modal completion save");
    }

    #[test]
    fn modal_save_paths_do_not_persist_editable_note() {
        let sentinel = "PATIENT: Jane Doe DOB 1990-01-01 SENTINEL";
        let (mut app, temp) = temp_app_with_single_field(assigned_time_field());
        app.editable_note = sentinel.to_string();

        app.handle_key(AppKey::Char('`'));
        assert_no_patient_text(&read_saved_config(temp.path()), sentinel, "pane swap save");

        app.open_header_modal();
        app.confirm_modal_value("9".to_string());
        assert_no_patient_text(
            &read_saved_config(temp.path()),
            sentinel,
            "modal intermediate save",
        );

        app.confirm_modal_value("00".to_string());
        assert_no_patient_text(
            &read_saved_config(temp.path()),
            sentinel,
            "modal completion save",
        );
    }

    #[test]
    fn sticky_values_do_not_contain_imported_text_after_save() {
        let sentinel = "PATIENT: Jane Doe DOB 1990-01-01 SENTINEL";
        let (mut app, temp) = temp_app_with_single_field(assigned_time_field());
        app.editable_note = sentinel.to_string();
        assert!(
            app.config
                .sticky_values
                .values()
                .all(|value| !value.contains(sentinel)),
            "sticky values should start without clipboard sentinel text"
        );

        app.open_header_modal();
        app.confirm_modal_value("9".to_string());
        app.confirm_modal_value("00".to_string());

        assert!(
            app.config
                .sticky_values
                .values()
                .all(|value| !value.contains(sentinel)),
            "sticky values must not absorb clipboard sentinel text after save-triggering flows"
        );
        assert_no_patient_text(
            &read_saved_config(temp.path()),
            sentinel,
            "sticky value persistence canary",
        );
    }

    #[test]
    fn group_idx_for_section_uses_runtime_navigation_not_section_group_id() {
        let first = SectionConfig {
            id: "first".to_string(),
            name: "First".to_string(),
            map_label: "FIRST".to_string(),
            section_type: "free_text".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: None,
            lists: Vec::new(),
            note_label: None,
            group_id: "wrong".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let second = SectionConfig {
            id: "second".to_string(),
            name: "Second".to_string(),
            map_label: "SECOND".to_string(),
            section_type: "free_text".to_string(),
            show_field_labels: true,
            data_file: None,
            fields: None,
            lists: Vec::new(),
            note_label: None,
            group_id: "wrong".to_string(),
            node_kind: RuntimeNodeKind::Section,
        };
        let data = AppData {
            template: RuntimeTemplate {
                id: "test".to_string(),
                children: vec![
                    RuntimeGroup {
                        id: "group_a".to_string(),
                        nav_label: "GROUP A".to_string(),
                        note: GroupNoteMeta::default(),
                        children: vec![RuntimeNode::Section(first.clone())],
                    },
                    RuntimeGroup {
                        id: "group_b".to_string(),
                        nav_label: "GROUP B".to_string(),
                        note: GroupNoteMeta::default(),
                        children: vec![RuntimeNode::Section(second.clone())],
                    },
                ],
            },
            list_data: HashMap::new(),
            checklist_data: HashMap::new(),
            collection_data: HashMap::new(),
            boilerplate_texts: HashMap::new(),
            keybindings: KeyBindings::default(),
            hotkeys: Default::default(),
        };

        let app = App::new(data, Config::default(), PathBuf::new());

        assert_eq!(app.group_idx_for_section(0), 0);
        assert_eq!(app.group_idx_for_section(1), 1);
    }

    #[test]
    fn app_new_derives_flat_section_view_from_runtime_template_order() {
        let first = free_text_section("first", "First", "group_a");
        let second = free_text_section("second", "Second", "group_a");
        let data = AppData {
            template: RuntimeTemplate {
                id: "test".to_string(),
                children: vec![RuntimeGroup {
                    id: "group_a".to_string(),
                    nav_label: "GROUP A".to_string(),
                    note: GroupNoteMeta::default(),
                    children: vec![
                        RuntimeNode::Section(first.clone()),
                        RuntimeNode::Section(second.clone()),
                    ],
                }],
            },
            list_data: HashMap::new(),
            checklist_data: HashMap::new(),
            collection_data: HashMap::new(),
            boilerplate_texts: HashMap::new(),
            keybindings: KeyBindings::default(),
            hotkeys: Default::default(),
        };

        let app = App::new(data, Config::default(), PathBuf::new());

        assert_eq!(app.navigation[0].node_id, "first");
        assert_eq!(app.navigation[1].node_id, "second");
        assert_eq!(
            flat_sections_from_template(&app.data.template)
                .iter()
                .map(|section| section.id.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "second"]
        );
        assert_eq!(
            app.config_for_index(0).map(|section| section.id.as_str()),
            Some("first")
        );
        assert_eq!(
            app.config_for_index(1).map(|section| section.id.as_str()),
            Some("second")
        );
    }

    #[test]
    fn nav_right_opens_header_modal_before_swapped_pane_nav() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.pane_swapped = true;

        app.handle_key(AppKey::Char('i'));

        assert!(app.modal.is_some(), "nav_right should open the field modal");
        assert_eq!(app.focus, Focus::Wizard);
        assert_eq!(app.map_return_idx, None);
    }

    #[test]
    fn select_keybind_opens_header_modal_in_wizard() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.data.keybindings.select = vec!["enter".to_string()];
        app.data.keybindings.confirm = vec!["space".to_string()];

        app.handle_key(AppKey::Space);
        assert!(
            app.modal.is_none(),
            "confirm should not open the field modal"
        );

        app.handle_key(AppKey::Enter);
        assert!(app.modal.is_some(), "select should open the field modal");
    }

    #[test]
    fn select_keybind_enters_wizard_from_map() {
        let mut app = app_with_free_text_sections(vec![
            free_text_section("first", "First", "group_a"),
            free_text_section("second", "Second", "group_a"),
        ]);
        app.data.keybindings.select = vec!["enter".to_string()];
        app.data.keybindings.confirm = vec!["space".to_string()];
        app.focus = Focus::Map;
        app.current_idx = 0;
        app.map_cursor = 1;

        app.handle_key(AppKey::Space);
        assert_eq!(app.focus, Focus::Map, "confirm should not leave the map");
        assert_eq!(
            app.current_idx, 0,
            "confirm should not enter the selected section"
        );

        app.handle_key(AppKey::Enter);
        assert_eq!(
            app.focus,
            Focus::Wizard,
            "select should enter the selected section"
        );
        assert_eq!(app.current_idx, 1);
        assert_eq!(app.map_return_idx, None);
    }

    #[test]
    fn nav_down_from_last_header_field_advances_to_next_section_first_field() {
        let mut app = app_with_header_sections(vec![
            header_section(
                "first_section",
                "First",
                vec![list_field(ModalStart::List), list_field(ModalStart::List)],
            ),
            header_section(
                "second_section",
                "Second",
                vec![list_field(ModalStart::List), list_field(ModalStart::List)],
            ),
        ]);

        let SectionState::Header(first_state) = &mut app.section_states[0] else {
            panic!("expected first header state");
        };
        first_state.field_index = 1;

        let SectionState::Header(second_state) = &mut app.section_states[1] else {
            panic!("expected second header state");
        };
        second_state.field_index = 1;
        second_state.repeat_counts[1] = 1;
        second_state.completed = true;

        app.handle_key(AppKey::Down);

        assert_eq!(
            app.current_idx, 1,
            "nav_down should advance to the next section"
        );
        let SectionState::Header(second_state) = &app.section_states[1] else {
            panic!("expected second header state");
        };
        assert_eq!(
            second_state.field_index, 0,
            "next header section should start at field 0"
        );
        assert_eq!(
            second_state.repeat_counts[0], 0,
            "next header section should reset to the first visible repeat slot"
        );
        assert!(
            !second_state.completed,
            "advancing into the next section should clear completed state"
        );
    }

    #[test]
    fn modal_nav_right_browses_next_part_without_confirming() {
        let mut app = app_with_single_field(assigned_time_field());
        app.open_header_modal();

        let modal = app.modal.as_ref().expect("modal should open");
        assert_eq!(modal.focus, ModalFocus::List);

        app.handle_key(AppKey::Char('i'));

        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("00"));
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert!(matches!(
            state.repeated_values[0].first(),
            Some(HeaderFieldValue::ListState(value)) if value.values == vec!["9".to_string()]
                && value.list_idx == 1
        ));
    }

    #[test]
    fn modal_terminal_nav_right_confirms_using_cursor_fallback_when_needed() {
        let mut app = app_with_single_field(assigned_time_field());
        app.open_header_modal();

        app.handle_key(AppKey::Char('i'));
        assert!(
            app.modal.is_some(),
            "first nav_right should browse to next modal"
        );

        app.handle_key(AppKey::Char('i'));

        assert!(
            app.modal.is_none(),
            "terminal nav_right should confirm the field using cursor fallback"
        );
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert!(matches!(
            state.repeated_values[0].first(),
            Some(HeaderFieldValue::ListState(value))
                if value.values == vec!["9".to_string(), "00".to_string()] && value.list_idx == 2
        ));
    }

    #[test]
    fn modal_search_keeps_nav_aliases_as_query_text() {
        let mut app = app_with_single_field(list_field(ModalStart::Search));
        app.open_header_modal();

        app.handle_key(AppKey::Char('n'));
        app.handle_key(AppKey::Char('i'));

        let modal = app.modal.as_ref().expect("modal should remain open");
        assert_eq!(modal.focus, ModalFocus::SearchBar);
        assert_eq!(modal.query, "ni");
    }

    #[test]
    fn modal_search_arrow_down_still_enters_list() {
        let mut app = app_with_single_field(list_field(ModalStart::Search));
        app.open_header_modal();

        app.handle_key(AppKey::Down);

        let modal = app.modal.as_ref().expect("modal should remain open");
        assert_eq!(modal.focus, ModalFocus::List);
    }

    #[test]
    fn modal_search_space_on_empty_query_enters_list() {
        let mut app = app_with_single_field(list_field(ModalStart::Search));
        app.open_header_modal();

        app.handle_key(AppKey::Space);

        let modal = app.modal.as_ref().expect("modal should remain open");
        assert_eq!(modal.focus, ModalFocus::List);
        assert!(modal.query.is_empty());
    }

    #[test]
    fn modal_search_space_after_typing_stays_query_text() {
        let mut app = app_with_single_field(list_field(ModalStart::Search));
        app.open_header_modal();

        app.handle_key(AppKey::Char('h'));
        app.handle_key(AppKey::Space);

        let modal = app.modal.as_ref().expect("modal should remain open");
        assert_eq!(modal.focus, ModalFocus::SearchBar);
        assert_eq!(modal.query, "h ");
    }

    #[test]
    fn map_authored_hotkey_overrides_nav_alias() {
        let mut app = app_with_free_text_sections(vec![
            free_text_section("first", "First", "group_a"),
            free_text_section("second", "Second", "group_a"),
            free_text_section("third", "Third", "group_a"),
        ]);
        app.focus = Focus::Map;
        app.current_idx = 0;
        app.map_cursor = 0;
        app.data
            .hotkeys
            .sections
            .insert("third".to_string(), "n".to_string());

        app.handle_key(AppKey::Char('n'));

        assert_eq!(app.current_idx, 2);
        assert_eq!(app.map_cursor, 2);
    }

    #[test]
    fn wizard_duplicate_authored_prefixes_wait_for_second_character() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.data
            .hotkeys
            .sections
            .insert("request_section".to_string(), "n".to_string());
        app.data
            .hotkeys
            .fields
            .insert("region".to_string(), "n".to_string());

        let section_label = app.section_hint_labels()[0].clone();
        let field_label = app.wizard_hint_labels().fields[0].clone();
        assert!(section_label.starts_with('n'));
        assert!(field_label.starts_with('n'));
        assert_ne!(section_label, field_label);
        assert!(field_label.len() > 1);

        app.handle_key(AppKey::Char('n'));
        assert!(app.modal.is_none(), "shared prefix should remain partial");
        assert_eq!(app.hint_buffer, "n");

        let second_char = field_label.chars().nth(1).expect("field hint needs suffix");
        app.handle_key(AppKey::Char(second_char));
        assert!(app.modal.is_some(), "full field hint should open the modal");
        assert!(app.hint_buffer.is_empty());
    }

    #[test]
    fn modal_list_authored_hotkey_overrides_nav_alias() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.data.hotkeys.items.insert(
            "region".to_string(),
            HashMap::from([("hip".to_string(), "n".to_string())]),
        );
        app.open_header_modal();

        app.handle_key(AppKey::Char('n'));

        assert!(
            app.modal.is_none(),
            "authored item hotkey should confirm immediately"
        );
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert!(matches!(
            state.repeated_values[0].first(),
            Some(HeaderFieldValue::ListState(value))
                if value.values == vec!["Hip".to_string()] && value.item_ids == vec!["hip".to_string()]
        ));
    }

    #[test]
    fn collection_modal_item_hotkey_overrides_nav_alias() {
        let mut app = app_with_single_field(collection_field());
        app.data.hotkeys.items.insert(
            "neck_list".to_string(),
            HashMap::from([("two".to_string(), "n".to_string())]),
        );
        app.open_header_modal();
        app.handle_key(AppKey::Space);

        app.handle_key(AppKey::Char('n'));

        let state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should stay open");
        let collection = state
            .collections
            .first()
            .expect("collection should still exist");
        assert_eq!(
            state.item_cursor, 1,
            "hotkey should target the authored row"
        );
        assert!(
            collection.active,
            "authored item hotkey should activate the collection"
        );
        assert!(
            !collection.item_enabled[1],
            "authored item hotkey should toggle the targeted item state"
        );
    }

    #[test]
    fn collection_modal_select_moves_focus_and_nav_left_returns() {
        let mut app = app_with_single_field(collection_field());
        app.open_header_modal();

        app.handle_key(AppKey::Space);
        let state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should exist after opening");
        assert!(state.in_items(), "select should enter collection items");
        assert_eq!(state.item_cursor, 0);

        app.handle_key(AppKey::Char('n'));
        let state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should remain available");
        assert_eq!(
            state.item_cursor, 1,
            "nav_down should move within collection items"
        );

        app.handle_key(AppKey::Char('h'));
        let state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should remain available");
        assert!(
            !state.in_items(),
            "nav_left should return to the collection list"
        );
    }

    #[test]
    fn collection_modal_nav_left_from_collection_list_dismisses_modal() {
        let mut app = app_with_single_field(collection_field());
        app.open_header_modal();

        app.handle_key(AppKey::Char('h'));

        assert!(
            app.modal.is_none(),
            "nav_left from the collection list should leave the modal"
        );
    }

    #[test]
    fn collection_modal_nav_right_enters_items_from_collection_list() {
        let mut app = app_with_single_field(collection_field());
        app.open_header_modal();

        app.handle_key(AppKey::Char('i'));

        let state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should remain open");
        assert!(
            state.in_items(),
            "nav_right should enter the right-hand pane"
        );
    }

    #[test]
    fn collection_modal_nav_right_from_items_confirms_without_cursor_fallback() {
        let mut app = app_with_single_field(collection_field());
        app.open_header_modal();

        app.handle_key(AppKey::Char('i'));
        app.handle_key(AppKey::Char('n'));
        app.handle_key(AppKey::Char('i'));

        assert!(
            app.modal.is_none(),
            "nav_right from the right-hand pane should commit and close the modal"
        );

        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        let Some(HeaderFieldValue::CollectionState(value)) = state.repeated_values[0].first()
        else {
            panic!("expected confirmed collection state");
        };
        assert!(
            crate::modal::active_collection_ids(value).is_empty(),
            "cursor position alone must not imply an active collection"
        );

        app.open_header_modal();
        let reopened_state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should restore");
        assert!(
            !reopened_state.collections[0].active,
            "reopening should preserve the explicit inactive collection state"
        );
        assert!(
            reopened_state.in_items(),
            "reopening should still restore the focused side"
        );
        assert_eq!(reopened_state.item_cursor, 1);
    }

    #[test]
    fn collection_modal_uses_configured_select_binding_for_side_switch() {
        let mut app = app_with_single_field(collection_field());
        app.data.keybindings.select = vec!["x".to_string()];
        app.open_header_modal();

        app.handle_key(AppKey::Space);
        let state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should stay open");
        assert!(
            !state.in_items(),
            "space should not side-switch once select is rebound"
        );

        app.handle_key(AppKey::Char('x'));
        let state = app
            .modal
            .as_ref()
            .and_then(|modal| modal.collection_state.as_ref())
            .expect("collection state should remain available");
        assert!(
            state.in_items(),
            "configured select binding should enter items"
        );
    }

    #[test]
    fn collection_modal_select_keeps_same_combined_hint_targets() {
        let mut app = app_with_single_field(collection_field());
        app.open_header_modal();

        let before = app.collection_modal_hint_targets();
        app.handle_key(AppKey::Space);
        let after = app.collection_modal_hint_targets();

        assert_eq!(
            before, after,
            "pane switching should not renumber collection hints"
        );
    }

    #[test]
    fn opening_simple_modal_starts_open_transition() {
        let mut app = app_with_single_field(list_field(ModalStart::List));

        app.open_header_modal();

        assert!(app.modal.is_some(), "modal should remain live during open");
        match app.modal_transitions.last() {
            Some(ModalTransitionLayer::ModalOpen { arrival, .. }) => {
                assert_eq!(arrival.focus_direction, FocusDirection::Forward);
            }
            other => panic!("expected modal open transition, got {other:?}"),
        }
        match app.modal_composition_transition.as_ref() {
            Some(ModalCompositionTransition::Open { arrival, .. }) => {
                assert_eq!(arrival.focus_direction, FocusDirection::Forward);
            }
            other => panic!("expected composition open transition, got {other:?}"),
        }
    }

    #[test]
    fn dismissing_simple_modal_retains_close_transition_after_live_modal_clears() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.open_header_modal();

        app.dismiss_modal();

        assert!(
            app.modal.is_none(),
            "live modal should be cleared immediately"
        );
        match app.modal_transitions.last() {
            Some(ModalTransitionLayer::ModalClose { departure, .. }) => {
                assert_eq!(departure.focus_direction, FocusDirection::Backward);
                assert!(
                    departure.modal.is_some(),
                    "close layer should retain modal snapshot"
                );
            }
            other => panic!("expected modal close transition, got {other:?}"),
        }
        match app.modal_composition_transition.as_ref() {
            Some(ModalCompositionTransition::Close { departure, .. }) => {
                assert_eq!(departure.focus_direction, FocusDirection::Backward);
            }
            other => panic!("expected composition close transition, got {other:?}"),
        }
    }

    #[test]
    fn confirming_simple_modal_starts_confirm_close_transition() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.open_header_modal();

        app.confirm_modal_value("Shoulder".to_string());

        assert!(
            app.modal.is_none(),
            "live modal should be cleared after confirm"
        );
        match app.modal_transitions.last() {
            Some(ModalTransitionLayer::ModalClose { departure, .. }) => {
                assert_eq!(departure.focus_direction, FocusDirection::Forward);
                assert!(
                    departure.modal.is_some(),
                    "close layer should retain modal snapshot"
                );
                assert_eq!(
                    departure
                        .content
                        .modals
                        .first()
                        .and_then(|snapshot| snapshot.confirmed_row),
                    Some(0),
                    "confirm close should snapshot the row in confirmed styling before motion"
                );
            }
            other => panic!("expected confirm close transition, got {other:?}"),
        }
        match app.modal_composition_transition.as_ref() {
            Some(ModalCompositionTransition::Close { departure, .. }) => {
                assert_eq!(departure.focus_direction, FocusDirection::Forward);
            }
            other => panic!("expected composition confirm close transition, got {other:?}"),
        }
    }

    #[test]
    fn confirming_across_unit_boundary_snapshots_confirmed_row_before_connected_transition() {
        let mut app = app_with_single_field(scheduled_visit_field());
        app.viewport_size = Some(iced::Size::new(220.0, 900.0));
        app.open_header_modal();

        app.confirm_modal_value("12".to_string());

        match app.modal_transitions.last() {
            Some(ModalTransitionLayer::ConnectedTransition { departure, .. }) => {
                assert_eq!(
                    departure
                        .content
                        .modals
                        .last()
                        .and_then(|snapshot| snapshot.confirmed_row),
                    Some(1),
                    "departing unit should preserve confirmed styling when confirmation crosses into the next unit"
                );
            }
            other => panic!("expected connected transition, got {other:?}"),
        }
    }

    #[test]
    fn non_simple_modal_close_falls_back_to_instant_behavior() {
        let mut app = app_with_single_field(list_field(ModalStart::List));

        app.open_header_modal();
        app.set_modal_query("sho".to_string());
        assert!(app.modal_unit_layout.is_none());

        app.dismiss_modal();
        assert!(app.modal.is_none());
        assert!(app.modal_transitions.is_empty());
    }

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
                        assigns: Vec::new(),
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
                        assigns: Vec::new(),
                    }],
                },
            ],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let assigned_values = HashMap::new();
        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &assigned_values, &sticky_values, 5);
        let _ = modal.advance_field("Left".to_string(), &assigned_values, &mut sticky_values, 5);

        let spans = compute_field_composition_spans(&modal, &assigned_values, &sticky_values);

        assert_eq!(
            spans
                .iter()
                .map(|span| (&span.text, &span.kind))
                .collect::<Vec<_>>(),
            vec![
                (&"Treat ".to_string(), &FieldCompositionSpanKind::Literal),
                (&"Left".to_string(), &FieldCompositionSpanKind::Confirmed),
                (&" ".to_string(), &FieldCompositionSpanKind::Literal),
                (&"Shoulder".to_string(), &FieldCompositionSpanKind::Active),
            ]
        );
    }

    #[test]
    fn composition_spans_use_live_modal_cursor_as_active_segment() {
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
                        assigns: Vec::new(),
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
                    items: vec![
                        HierarchyItem {
                            id: "shoulder".to_string(),
                            label: Some("Shoulder".to_string()),
                            default_enabled: true,
                            output: Some("Shoulder".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        HierarchyItem {
                            id: "hip".to_string(),
                            label: Some("Hip".to_string()),
                            default_enabled: false,
                            output: Some("Hip".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
            ],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let assigned_values = HashMap::new();
        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &assigned_values, &sticky_values, 5);
        let _ = modal.advance_field("Left".to_string(), &assigned_values, &mut sticky_values, 5);
        modal.list_cursor = 1;
        modal.update_scroll();

        let spans = compute_field_composition_spans(&modal, &assigned_values, &sticky_values);

        assert_eq!(
            spans
                .iter()
                .map(|span| (&span.text, &span.kind))
                .collect::<Vec<_>>(),
            vec![
                (&"Treat ".to_string(), &FieldCompositionSpanKind::Literal),
                (&"Left".to_string(), &FieldCompositionSpanKind::Confirmed),
                (&" ".to_string(), &FieldCompositionSpanKind::Literal),
                (&"Hip".to_string(), &FieldCompositionSpanKind::Active),
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
                    assigns: Vec::new(),
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
            show_field_labels: true,
            data_file: None,
            fields: Some(vec![field]),
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
                assert!(matches!(source.as_ref(), HeaderFieldValue::ListState(_)));
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

    #[test]
    fn manual_override_text_survives_export() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.open_header_modal();
        app.handle_key(AppKey::CtrlChar('e'));
        app.set_modal_composition_text("custom override".to_string());
        app.handle_key(AppKey::Enter);
        app.confirm_modal_value("Shoulder".to_string());

        let exported = crate::document::export_editable_document(&app.editable_note);

        assert!(
            app.editable_note.contains("Region: custom override"),
            "editable note should contain override text"
        );
        assert!(
            exported.contains("Region: custom override"),
            "exported note should also contain override text"
        );
    }

    #[test]
    fn composition_override_ctrl_r_reverts_editable_note() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.open_header_modal();
        app.handle_key(AppKey::CtrlChar('e'));
        app.set_modal_composition_text("should not appear".to_string());
        app.handle_key(AppKey::CtrlChar('r'));
        app.confirm_modal_value("Shoulder".to_string());

        assert!(
            app.editable_note.contains("Region: Shoulder"),
            "editable note should contain the modal-resolved value after reset"
        );
        assert!(
            !app.editable_note.contains("should not appear"),
            "editable note must not contain the cleared override text"
        );
    }

    #[test]
    fn composition_override_shift_enter_super_confirms_manual_override() {
        let mut app = app_with_single_field(list_field(ModalStart::List));
        app.open_header_modal();
        app.handle_key(AppKey::CtrlChar('e'));
        app.set_modal_composition_text("Manual shoulder".to_string());

        app.handle_key(AppKey::ShiftEnter);

        assert!(
            app.modal.is_none(),
            "shift+enter should commit and close the modal"
        );
        assert!(
            app.editable_note.contains("Region: Manual shoulder"),
            "super confirm should preserve the manual override text"
        );
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert!(matches!(
            state.repeated_values[0].first(),
            Some(HeaderFieldValue::ManualOverride { text, .. }) if text == "Manual shoulder"
        ));
    }

    #[test]
    fn super_confirm_advances_modal_confirmed_empty_value() {
        let mut app = app_with_fields(vec![
            empty_capable_search_field(),
            list_field(ModalStart::List),
        ]);

        app.open_header_modal();
        app.confirm_modal_value(String::new());

        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert_eq!(
            state.field_index, 1,
            "confirming the empty modal choice should advance to the next field"
        );
        assert!(matches!(
            state.repeated_values[0].first(),
            Some(HeaderFieldValue::ListState(value))
                if value.values == vec![String::new()] && value.list_idx == 1
        ));

        let SectionState::Header(state) = &mut app.section_states[0] else {
            panic!("expected header state");
        };
        state.field_index = 0;
        state.completed = false;

        app.handle_header_key(AppKey::ShiftEnter);

        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert_eq!(
            state.field_index, 1,
            "section-level super confirm should accept the already confirmed empty modal value"
        );
        assert!(matches!(
            state.repeated_values[0].first(),
            Some(HeaderFieldValue::ListState(value))
                if value.values == vec![String::new()] && value.list_idx == 1
        ));
    }

    #[test]
    fn free_text_confirm_via_key_handler_syncs_editable_note() {
        use crate::sections::free_text::{FreeTextMode, FreeTextState};

        let mut app = app_with_free_text_sections(vec![free_text_section(
            "notes_section",
            "Notes",
            "subjective",
        )]);
        app.section_states[0] = SectionState::FreeText(FreeTextState {
            entries: vec!["patient reported pain in left shoulder".to_string()],
            cursor: 0,
            mode: FreeTextMode::Browsing,
            edit_buf: String::new(),
            skipped: false,
            completed: false,
        });

        app.handle_free_text_key(AppKey::Enter);

        assert!(
            app.editable_note
                .contains("patient reported pain in left shoulder"),
            "editable note should reflect the committed free-text entry"
        );
        assert!(
            app.note_headings_valid,
            "document structure should remain valid after sync"
        );
    }

    #[test]
    fn completing_two_free_text_sections_keeps_both_in_editable_note() {
        use crate::sections::free_text::{FreeTextMode, FreeTextState};

        let mut app = app_with_free_text_sections(vec![
            free_text_section("section_a", "SectionA", "group_a"),
            free_text_section("section_b", "SectionB", "group_b"),
        ]);
        app.section_states[0] = SectionState::FreeText(FreeTextState {
            entries: vec!["section a content".to_string()],
            cursor: 0,
            mode: FreeTextMode::Browsing,
            edit_buf: String::new(),
            skipped: false,
            completed: false,
        });

        app.handle_free_text_key(AppKey::Enter);

        app.section_states[1] = SectionState::FreeText(FreeTextState {
            entries: vec!["section b content".to_string()],
            cursor: 0,
            mode: FreeTextMode::Browsing,
            edit_buf: String::new(),
            skipped: false,
            completed: false,
        });

        app.handle_free_text_key(AppKey::Enter);

        assert!(
            app.editable_note.contains("section a content"),
            "section A output should remain in editable note after section B syncs"
        );
        assert!(
            app.editable_note.contains("section b content"),
            "section B output should appear in editable note"
        );
        assert!(
            app.note_headings_valid,
            "document structure should remain valid after both syncs"
        );
    }

    #[test]
    fn modal_confirm_persists_assigned_format_list_value() {
        let mut app = app_with_single_field(assigned_time_field());
        app.open_header_modal();

        app.confirm_modal_value("9".to_string());
        assert!(
            app.modal.is_some(),
            "first list should advance to minute selection"
        );

        app.confirm_modal_value("00".to_string());

        assert_eq!(
            app.assigned_values.get("am_pm").map(String::as_str),
            Some("AM")
        );
        assert!(app.editable_note.contains("Appointment: 9:00AM"));
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        match &state.repeated_values[0][0] {
            HeaderFieldValue::ListState(value) => {
                assert_eq!(value.values, vec!["9".to_string(), "00".to_string()]);
                assert_eq!(value.list_idx, 2);
            }
            other => panic!("expected list state, got {other:?}"),
        }
    }

    #[test]
    fn editing_one_repeating_slot_preserves_neighbor_note_output() {
        let mut app = app_with_single_field(repeating_assigned_time_field(2));
        app.open_header_modal();
        app.confirm_modal_value("9".to_string());
        app.confirm_modal_value("00".to_string());

        app.open_header_modal();
        app.confirm_modal_value("12".to_string());
        app.confirm_modal_value("45".to_string());

        let before = app.editable_note.clone();
        assert!(
            before.contains("Appointment: 9:00AM"),
            "first slot should render before the edit"
        );
        assert!(
            before.contains("Appointment: 12:45PM"),
            "second slot should render before the edit"
        );

        let SectionState::Header(state) = &mut app.section_states[0] else {
            panic!("expected header state");
        };
        state.field_index = 0;
        state.repeat_counts[0] = 0;
        state.completed = false;
        app.open_header_modal();
        app.confirm_modal_value("12".to_string());
        app.confirm_modal_value("00".to_string());

        let after = app.editable_note.clone();
        assert!(
            !after.contains("Appointment: 9:00AM"),
            "edited slot should no longer show the old value"
        );
        assert!(
            after.contains("Appointment: 12:00PM"),
            "edited slot should show the replacement value"
        );
        assert_eq!(
            before.matches("Appointment: 12:45PM").count(),
            after.matches("Appointment: 12:45PM").count(),
            "editing slot 0 should not change slot 1's rendered note output"
        );
    }

    #[test]
    fn reopened_modal_restores_confirmed_cursors_and_nav_right_keeps_original_choice() {
        let mut app = app_with_single_field(assigned_time_field());
        app.open_header_modal();
        app.confirm_modal_value("12".to_string());
        app.confirm_modal_value("45".to_string());

        app.open_header_modal();

        let modal = app.modal.as_ref().expect("modal should reopen");
        assert_eq!(modal.field_flow.list_idx, 0);
        assert_eq!(modal.selected_value(), Some("12"));
        assert_eq!(modal.confirmed_row_for_current_list(), Some(1));

        app.handle_key(AppKey::Char('e'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.selected_value(), Some("9"));

        app.handle_key(AppKey::Char('i'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("45"));
        assert_eq!(modal.confirmed_row_for_current_list(), Some(1));

        app.handle_key(AppKey::Char('h'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 0);
        assert_eq!(modal.selected_value(), Some("9"));
    }

    #[test]
    fn reopened_repeating_modal_restores_slot_specific_confirmed_cursor_despite_global_collision() {
        let mut app = app_with_single_field(repeating_assigned_time_field(2));
        app.open_header_modal();
        app.confirm_modal_value("12".to_string());
        app.confirm_modal_value("45".to_string());

        app.open_header_modal();
        app.confirm_modal_value("9".to_string());
        app.confirm_modal_value("00".to_string());

        assert_eq!(
            app.assigned_values.get("am_pm").map(String::as_str),
            Some("AM"),
            "global assigned cache should reflect the most recently confirmed slot"
        );

        let SectionState::Header(state) = &mut app.section_states[0] else {
            panic!("expected header state");
        };
        state.field_index = 0;
        state.repeat_counts[0] = 0;
        state.completed = false;
        app.open_header_modal();

        let modal = app.modal.as_ref().expect("modal should reopen");
        assert_eq!(modal.field_flow.list_idx, 0);
        assert_eq!(modal.selected_value(), Some("12"));
        assert_eq!(modal.confirmed_row_for_current_list(), Some(1));

        app.handle_key(AppKey::Char('i'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("45"));
        assert_eq!(modal.confirmed_row_for_current_list(), Some(1));
    }

    #[test]
    fn reopened_three_part_modal_preserves_live_cursor_when_leaving_left() {
        let mut app = app_with_single_field(scheduled_visit_field());
        app.open_header_modal();
        app.confirm_modal_value("12".to_string());
        app.confirm_modal_value("45".to_string());
        app.confirm_modal_value("60".to_string());

        app.open_header_modal();
        app.handle_key(AppKey::Char('i'));
        app.handle_key(AppKey::Char('e'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("00"));

        app.handle_key(AppKey::Char('i'));
        app.handle_key(AppKey::Char('e'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 2);
        assert_eq!(modal.selected_value(), Some("45"));

        app.handle_key(AppKey::Char('h'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("00"));

        app.handle_key(AppKey::Char('i'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 2);
        assert_eq!(modal.selected_value(), Some("45"));
    }

    #[test]
    fn reopened_terminal_nav_right_commits_only_explicitly_confirmed_choices() {
        let mut app = app_with_single_field(scheduled_visit_field());
        app.open_header_modal();
        app.confirm_modal_value("12".to_string());
        app.confirm_modal_value("45".to_string());
        app.confirm_modal_value("60".to_string());

        app.open_header_modal();
        app.handle_key(AppKey::Char('i'));
        app.handle_key(AppKey::Char('e'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("00"));

        app.handle_key(AppKey::Char('i'));
        app.handle_key(AppKey::Char('h'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("00"));

        app.handle_key(AppKey::Char('i'));
        app.handle_key(AppKey::Char('e'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 2);
        assert_eq!(modal.selected_value(), Some("45"));

        app.handle_key(AppKey::Char('h'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 1);
        assert_eq!(modal.selected_value(), Some("00"));

        app.handle_key(AppKey::Char('i'));
        let modal = app.modal.as_ref().expect("modal should stay open");
        assert_eq!(modal.field_flow.list_idx, 2);
        assert_eq!(modal.selected_value(), Some("45"));

        app.handle_key(AppKey::Char('i'));

        assert!(
            app.modal.is_none(),
            "terminal nav_right should commit the field"
        );
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert!(matches!(
            state.repeated_values[0].first(),
            Some(HeaderFieldValue::ListState(value))
                if value.values
                    == vec!["12".to_string(), "45".to_string(), "60".to_string()]
                    && value.list_idx == 3
        ));
    }

    #[test]
    fn header_backspace_clears_confirmed_assignment_state() {
        let mut app = app_with_single_field(assigned_time_field());
        app.open_header_modal();
        app.confirm_modal_value("9".to_string());
        app.confirm_modal_value("00".to_string());

        app.handle_header_key(AppKey::Backspace);

        assert!(
            !app.assigned_values.contains_key("am_pm"),
            "backspace should remove the slot's assigned format-list value"
        );
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert!(matches!(
            &state.repeated_values[0][0],
            HeaderFieldValue::ExplicitEmpty
        ));
        assert!(!app.editable_note.contains("Appointment: 9:00AM"));
    }

    #[test]
    fn modal_back_to_first_list_clears_existing_assignment_state() {
        let mut app = app_with_single_field(assigned_time_field());
        app.open_header_modal();
        app.confirm_modal_value("9".to_string());
        app.confirm_modal_value("00".to_string());
        assert_eq!(
            app.assigned_values.get("am_pm").map(String::as_str),
            Some("AM")
        );

        app.open_header_modal();
        app.confirm_modal_value("12".to_string());
        let modal = app.modal.as_ref().expect("modal should still be open");
        assert_eq!(modal.field_flow.list_idx, 1);

        app.composite_go_back();

        let modal = app.modal.as_ref().expect("modal should remain open");
        assert_eq!(modal.field_flow.list_idx, 0);
        assert!(
            !app.assigned_values.contains_key("am_pm"),
            "backing out to the first list should clear the stale assigned value for that slot"
        );
        let SectionState::Header(state) = &app.section_states[0] else {
            panic!("expected header state");
        };
        assert!(
            state.repeated_values[0].is_empty(),
            "backing out to the first list should clear the in-progress preview slot"
        );
    }

    #[test]
    fn confirming_repeating_search_lists_updates_preview_and_export() {
        let mut app = app_with_single_field(observation_field_with_repeating_search_lists());

        app.open_header_modal();
        select_only_filtered_modal_match(&mut app, "trapezius");
        app.handle_key(AppKey::Enter);
        select_only_filtered_modal_match(&mut app, "left");
        select_only_filtered_modal_match(&mut app, "rmt");
        app.handle_key(AppKey::Enter);

        let exported = crate::document::export_editable_document(&app.editable_note);

        assert!(
            app.editable_note.contains(
                "Observation: Left Trapezius (Upper Fibers): Increased Resting Muscle Tension"
            ),
            "editable note should contain confirmed objective field output"
        );
        assert!(
            exported.contains(
                "Observation: Left Trapezius (Upper Fibers): Increased Resting Muscle Tension"
            ),
            "exported preview should contain confirmed objective field output"
        );
    }

    #[test]
    fn confirming_nested_repeat_terminator_updates_preview_and_export() {
        let mut app = app_with_single_field(request_field_with_nested_repeat_terminator());

        app.open_header_modal();
        select_only_filtered_modal_match(&mut app, "relaxation");
        select_only_filtered_modal_match(&mut app, "shoulder");
        select_only_filtered_modal_match(&mut app, "left");
        app.handle_key(AppKey::Enter);
        app.handle_key(AppKey::Enter);

        let exported = crate::document::export_editable_document(&app.editable_note);

        assert!(
            app.editable_note
                .contains("Request: Relaxation massage, focusing on the Left Shoulder"),
            "editable note should contain confirmed appointment request output"
        );
        assert!(
            exported.contains("Request: Relaxation massage, focusing on the Left Shoulder"),
            "exported preview should contain confirmed appointment request output"
        );
        assert!(
            app.modal.is_none(),
            "modal should close after nested repeat terminates"
        );
    }
}
