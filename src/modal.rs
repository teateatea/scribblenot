use crate::data::{
    HeaderFieldConfig, HierarchyList, JoinerStyle, ModalStart, ResolvedCollectionConfig,
};
use crate::modal_layout::{
    build_simple_modal_unit_layout, ModalFocus, ModalListViewSnapshot, ModalStubKind,
    SimpleModalSequence, SimpleModalUnitLayout,
};
use crate::sections::collection::CollectionState;
use crate::sections::header::{
    CollectionFieldValue, CollectionSelection, HeaderFieldValue, HeaderState, ListFieldValue,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FieldModal {
    pub format: Option<String>,
    pub lists: Vec<HierarchyList>,
    pub format_lists: Vec<HierarchyList>,
    pub list_idx: usize,
    pub values: Vec<String>,
    pub item_ids: Vec<String>,
    pub repeat_values: Vec<String>,
    pub repeat_item_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BranchFrame {
    pub parent_flow: FieldModal,
    pub output_format: String,
    pub branch_fields: Vec<HeaderFieldConfig>,
    pub field_idx: usize,
    pub values: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct NestedFrame {
    pub field: HeaderFieldConfig,
    pub state: HeaderState,
}

#[derive(Debug, Clone)]
pub struct SearchModal {
    #[allow(dead_code)]
    pub field_idx: usize,
    #[allow(dead_code)]
    pub field_id: String,
    pub field_name: String,
    pub query: String,
    pub all_entries: Vec<String>,
    pub all_outputs: Vec<String>,
    pub filtered: Vec<usize>,
    pub list_cursor: usize,
    pub list_scroll: usize,
    pub sticky_cursor: usize,
    pub focus: ModalFocus,
    pub confirmed_list_state: Option<ListFieldValue>,
    pub session_cursor_values: Vec<Option<String>>,
    pub session_cursor_item_ids: Vec<Option<String>>,
    pub session_confirmed_values: Vec<Option<String>>,
    pub session_confirmed_item_ids: Vec<Option<String>>,
    pub field_flow: FieldModal,
    pub collection_state: Option<CollectionState>,
    pub collection_format: Option<String>,
    pub branch_stack: Vec<BranchFrame>,
    pub nested_stack: Vec<NestedFrame>,
    pub window_size: usize,
    pub manual_override: Option<String>,
}

#[derive(Debug)]
pub enum FieldAdvance {
    NextList,
    Complete(HeaderFieldValue),
    StayOnList,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionPreviewSnapshot {
    pub title: String,
    pub rows: Vec<String>,
    pub item_cursor: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionPreviewStrip {
    pub previews: Vec<CollectionPreviewSnapshot>,
    pub focused_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalEdgeSemantics {
    pub left: ModalStubKind,
    pub right: ModalStubKind,
}

#[derive(Clone, Copy)]
pub struct ListValueLookup<'a> {
    pub assigned_values: &'a HashMap<String, String>,
    pub sticky_values: &'a HashMap<String, String>,
}

impl<'a> ListValueLookup<'a> {
    pub fn new(
        assigned_values: &'a HashMap<String, String>,
        sticky_values: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            assigned_values,
            sticky_values,
        }
    }

    pub fn raw_fallback_value(self, list: &HierarchyList) -> Option<String> {
        if let Some(value) = self.assigned_values.get(&list.id) {
            return Some(value.clone());
        }
        if list.sticky {
            if let Some(value) = self.sticky_values.get(&list.id) {
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
                let value = item.output().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
        None
    }

    pub fn fallback_value(self, list: &HierarchyList) -> Option<String> {
        self.raw_fallback_value(list)
    }

    pub fn list_initial_cursor(self, list: &HierarchyList, outputs: &[String]) -> usize {
        if let Some(value) = self.assigned_values.get(&list.id) {
            if let Some(pos) = outputs.iter().position(|output| output == value) {
                return pos;
            }
        }
        if list.sticky {
            if let Some(value) = self.sticky_values.get(&list.id) {
                if let Some(pos) = outputs.iter().position(|output| output == value) {
                    return pos;
                }
            }
        }
        if let Some(default) = &list.default {
            if let Some(pos) = list.items.iter().position(|item| {
                item.id == *default
                    || item.ui_label() == *default
                    || item.output.as_deref() == Some(default.as_str())
            }) {
                return pos;
            }
        }
        0
    }
}

impl SearchModal {
    fn build_initial_session_cursor_memory(
        list_count: usize,
        confirmed_list_state: Option<&ListFieldValue>,
    ) -> (Vec<Option<String>>, Vec<Option<String>>) {
        let mut values = vec![None; list_count];
        let mut item_ids = vec![None; list_count];
        if let Some(state) = confirmed_list_state {
            for (idx, value) in state.values.iter().enumerate().take(list_count) {
                values[idx] = Some(value.clone());
            }
            for (idx, item_id) in state.item_ids.iter().enumerate().take(list_count) {
                if !item_id.is_empty() {
                    item_ids[idx] = Some(item_id.clone());
                }
            }
        }
        (values, item_ids)
    }

    fn build_initial_session_confirmed_memory(
        list_count: usize,
        confirmed_list_state: Option<&ListFieldValue>,
    ) -> (Vec<Option<String>>, Vec<Option<String>>) {
        let mut values = vec![None; list_count];
        let mut item_ids = vec![None; list_count];
        if let Some(state) = confirmed_list_state {
            for (idx, value) in state.values.iter().enumerate().take(list_count) {
                values[idx] = Some(value.clone());
            }
            for (idx, item_id) in state.item_ids.iter().enumerate().take(list_count) {
                if !item_id.is_empty() {
                    item_ids[idx] = Some(item_id.clone());
                }
            }
        }
        (values, item_ids)
    }

    fn confirmed_choice_for_list(&self, list_idx: usize) -> (Option<&str>, Option<&str>) {
        let session = (
            self.session_confirmed_values
                .get(list_idx)
                .and_then(|value| value.as_deref()),
            self.session_confirmed_item_ids
                .get(list_idx)
                .and_then(|item_id| item_id.as_deref()),
        );
        if session.0.is_some() || session.1.is_some() {
            return session;
        }
        let Some(state) = self.confirmed_list_state.as_ref() else {
            return (None, None);
        };
        (
            state.values.get(list_idx).map(String::as_str),
            state.item_ids.get(list_idx).map(String::as_str),
        )
    }

    fn authoritative_choice_for_list(&self, list_idx: usize) -> (Option<&str>, Option<&str>) {
        let confirmed = self.confirmed_choice_for_list(list_idx);
        if confirmed.0.is_some() || confirmed.1.is_some() {
            return confirmed;
        }

        let session = self.session_choice_for_list(list_idx);
        if session.0.is_some() || session.1.is_some() {
            return session;
        }

        if list_idx == self.field_flow.list_idx {
            return (
                self.selected_value(),
                self.selected_item_id()
                    .filter(|item_id| !item_id.is_empty()),
            );
        }

        if let Some(value) = self.field_flow.values.get(list_idx) {
            let item_id = self
                .field_flow
                .item_ids
                .get(list_idx)
                .filter(|item_id| !item_id.is_empty())
                .map(String::as_str);
            return (Some(value.as_str()), item_id);
        }

        (None, None)
    }

    fn confirmed_row_for_list(&self, list_idx: usize) -> Option<usize> {
        let list = self.field_flow.lists.get(list_idx)?;
        let (confirmed_value, confirmed_item_id) = self.confirmed_choice_for_list(list_idx);
        if let Some(item_id) = confirmed_item_id {
            if let Some(pos) = list.items.iter().position(|item| item.id == item_id) {
                return Some(pos);
            }
        }
        let confirmed_value = confirmed_value?;
        self.all_outputs
            .iter()
            .position(|output| output == confirmed_value)
    }

    pub fn confirmed_row_for_current_list(&self) -> Option<usize> {
        self.confirmed_row_for_list(self.field_flow.list_idx)
    }

    fn session_choice_for_list(&self, list_idx: usize) -> (Option<&str>, Option<&str>) {
        (
            self.session_cursor_values
                .get(list_idx)
                .and_then(|value| value.as_deref()),
            self.session_cursor_item_ids
                .get(list_idx)
                .and_then(|item_id| item_id.as_deref()),
        )
    }

    pub fn is_revisiting_completed_field(&self) -> bool {
        self.confirmed_list_state
            .as_ref()
            .is_some_and(|state| state.list_idx >= self.field_flow.lists.len())
    }

    fn initial_cursor_for_list(
        &self,
        list: &HierarchyList,
        list_idx: usize,
        outputs: &[String],
        lookup: ListValueLookup<'_>,
    ) -> usize {
        let (session_value, session_item_id) = self.session_choice_for_list(list_idx);
        if let Some(item_id) = session_item_id {
            if let Some(pos) = list.items.iter().position(|item| item.id == item_id) {
                return pos;
            }
        }
        if let Some(value) = session_value {
            if let Some(pos) = outputs.iter().position(|output| output == value) {
                return pos;
            }
        }
        let (confirmed_value, confirmed_item_id) = self.confirmed_choice_for_list(list_idx);
        if let Some(item_id) = confirmed_item_id {
            if let Some(pos) = list.items.iter().position(|item| item.id == item_id) {
                return pos;
            }
        }
        if let Some(value) = confirmed_value {
            if let Some(pos) = outputs.iter().position(|output| output == value) {
                return pos;
            }
        }
        list_initial_cursor(list, outputs, lookup)
    }

    fn navigation_choice_for_current_list(&self) -> (Option<String>, Option<String>) {
        (
            self.selected_value().map(str::to_string),
            self.selected_item_id().map(str::to_string),
        )
    }

    fn remember_current_cursor(&mut self) {
        if self.collection_state.is_some() {
            return;
        }
        let list_idx = self.field_flow.list_idx;
        if self.session_cursor_values.len() <= list_idx {
            return;
        }
        self.session_cursor_values[list_idx] = self.selected_value().map(str::to_string);
        self.session_cursor_item_ids[list_idx] = self.selected_item_id().map(str::to_string);
    }

    fn current_collection_preview_index(&self) -> Option<usize> {
        let state = self.collection_state.as_ref()?;
        Some(match state.focus {
            crate::sections::collection::CollectionFocus::Collections => state.collection_cursor,
            crate::sections::collection::CollectionFocus::Items(collection_idx) => collection_idx,
        })
    }

    fn supports_simple_list_teasers(&self) -> bool {
        self.collection_state.is_none()
            && self.branch_stack.is_empty()
            && self.nested_stack.is_empty()
            && self.query.trim().is_empty()
            && self.filtered.len() == self.all_entries.len()
            && self
                .filtered
                .iter()
                .enumerate()
                .all(|(idx, entry_idx)| idx == *entry_idx)
            && !self.field_flow.lists.is_empty()
    }

    fn semantic_choice_for_current_list(
        &self,
        lookup: ListValueLookup<'_>,
    ) -> (Option<String>, Option<String>) {
        let selected_value = self.selected_value().map(str::to_string);
        let selected_item_id = self.selected_item_id().map(str::to_string);
        if selected_value.is_some() || selected_item_id.is_some() {
            return (selected_value, selected_item_id);
        }

        let list_idx = self.field_flow.list_idx;
        let (session_value, session_item_id) = self.session_choice_for_list(list_idx);
        if session_value.is_some() || session_item_id.is_some() {
            return (
                session_value.map(str::to_string),
                session_item_id.map(str::to_string),
            );
        }

        let (confirmed_value, confirmed_item_id) = self.confirmed_choice_for_list(list_idx);
        if confirmed_value.is_some() || confirmed_item_id.is_some() {
            return (
                confirmed_value.map(str::to_string),
                confirmed_item_id.map(str::to_string),
            );
        }

        (
            current_list_fallback_value(self, lookup),
            current_list_fallback_item_id(self, lookup),
        )
    }

    fn can_go_back_semantically(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> bool {
        let mut preview = self.clone();
        if preview.collection_back() {
            return true;
        }
        if preview.go_back_one_step(assigned_values, sticky_values, self.window_size) {
            return true;
        }
        if preview.field_flow.list_idx > 0 {
            return true;
        }
        preview.restore_parent_branch(assigned_values, sticky_values, self.window_size)
    }

    fn fallback_right_stub_for_ambiguous_choice(&self, list: &HierarchyList) -> ModalStubKind {
        if !self.nested_stack.is_empty()
            || !self.branch_stack.is_empty()
            || self.field_flow.list_idx + 1 < self.field_flow.lists.len()
            || list.items.iter().any(|item| !item.branch_fields.is_empty())
        {
            ModalStubKind::NavRight
        } else {
            ModalStubKind::Confirm
        }
    }

    fn semantic_right_stub(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> ModalStubKind {
        if let Some(state) = self.collection_state.as_ref() {
            return if state.in_items() {
                ModalStubKind::Confirm
            } else {
                ModalStubKind::NavRight
            };
        }

        let Some(list) = self.field_flow.lists.get(self.field_flow.list_idx) else {
            return ModalStubKind::Confirm;
        };
        let lookup = ListValueLookup::new(assigned_values, sticky_values);
        let (value, item_id) = self.semantic_choice_for_current_list(lookup);

        let mut preview = self.clone();
        let assigned_preview = assigned_values.clone();
        let mut sticky_preview = sticky_values.clone();
        let advance = if effective_joiner_style(list).is_some() {
            preview.finish_repeating_current_list(
                value,
                item_id,
                &assigned_preview,
                &mut sticky_preview,
                self.window_size,
            )
        } else if let Some(value) = value {
            Some(preview.advance_field(
                value,
                &assigned_preview,
                &mut sticky_preview,
                self.window_size,
            ))
        } else {
            None
        };

        match advance {
            Some(FieldAdvance::Complete(_)) => ModalStubKind::Confirm,
            Some(FieldAdvance::NextList) | Some(FieldAdvance::StayOnList) => {
                ModalStubKind::NavRight
            }
            None => self.fallback_right_stub_for_ambiguous_choice(list),
        }
    }

    pub fn edge_semantics(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> ModalEdgeSemantics {
        ModalEdgeSemantics {
            left: if self.can_go_back_semantically(assigned_values, sticky_values) {
                ModalStubKind::NavLeft
            } else {
                ModalStubKind::Exit
            },
            right: self.semantic_right_stub(assigned_values, sticky_values),
        }
    }

    fn go_back_simple_list(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> bool {
        if !self.supports_simple_list_teasers() || self.field_flow.list_idx == 0 {
            return false;
        }

        let popped = self.field_flow.values.pop();
        let popped_item_id = self.field_flow.item_ids.pop();
        self.field_flow.list_idx -= 1;
        let list = &self.field_flow.lists[self.field_flow.list_idx];
        let mut merged_assigned = assigned_values.clone();
        merged_assigned.extend(self.assigned_values(sticky_values));
        let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
        let labels = resolved_item_labels_for_list(
            list,
            &self.field_flow.values,
            &self.field_flow.repeat_values,
            &self.field_flow.lists,
            &self.field_flow.format_lists,
            lookup,
        );
        let outputs: Vec<String> = list.items.iter().map(item_output).collect();
        let cursor = popped_item_id
            .as_ref()
            .filter(|item_id| !item_id.is_empty())
            .and_then(|item_id| list.items.iter().position(|item| item.id == **item_id))
            .or_else(|| {
                popped
                    .as_ref()
                    .and_then(|value| outputs.iter().position(|output| output == value))
            })
            .unwrap_or(0);
        self.all_entries = labels;
        self.all_outputs = outputs;
        self.list_cursor = cursor;
        self.sticky_cursor = cursor;
        self.query = String::new();
        self.list_scroll = 0;
        self.focus = ModalFocus::List;
        self.update_filter();
        true
    }

    pub fn assigned_values(
        &self,
        sticky_values: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut assigned = HashMap::new();
        if !self.nested_stack.is_empty() {
            if let Some(root) = self.synced_nested_root_state(sticky_values) {
                if let Some(frame) = self.nested_stack.first() {
                    collect_assignments_from_state(
                        &root,
                        &frame.field,
                        sticky_values,
                        &mut assigned,
                    );
                }
            }
            return assigned;
        }
        for frame in &self.branch_stack {
            collect_assignments_from_flow(&frame.parent_flow, sticky_values, &mut assigned);
        }
        collect_assignments_from_flow(&self.field_flow, sticky_values, &mut assigned);
        assigned
    }

    fn synced_nested_root_state(
        &self,
        sticky_values: &HashMap<String, String>,
    ) -> Option<HeaderState> {
        self.synced_nested_root_state_with_leaf_value(self.preview_active_leaf_value(sticky_values))
    }

    fn synced_nested_root_state_with_leaf_value(
        &self,
        leaf_value: HeaderFieldValue,
    ) -> Option<HeaderState> {
        let mut frames = self.nested_stack.clone();
        if frames.is_empty() {
            return None;
        }
        if let Some(last) = frames.last_mut() {
            last.state.set_preview_value(leaf_value);
        }
        if frames.len() >= 2 {
            for idx in (1..frames.len()).rev() {
                let child_state = frames[idx].state.clone();
                frames[idx - 1]
                    .state
                    .set_preview_value(HeaderFieldValue::NestedState(Box::new(child_state)));
            }
        }
        frames.first().map(|frame| frame.state.clone())
    }

    pub fn new_field(
        field_idx: usize,
        field: HeaderFieldConfig,
        current_value: Option<&HeaderFieldValue>,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> Self {
        if !field.fields.is_empty() {
            return Self::new_nested_field(
                field_idx,
                field,
                current_value,
                assigned_values,
                sticky_values,
                window_size,
            );
        }
        if !field.collections.is_empty() && field.lists.is_empty() {
            return Self::new_collection_field(field_idx, field, current_value, window_size);
        }
        let current_source = current_value.map(HeaderFieldValue::source_value);
        let first_list = &field.lists[0];
        let outputs: Vec<String> = first_list.items.iter().map(item_output).collect();
        let lookup = ListValueLookup::new(assigned_values, sticky_values);
        let manual_override = current_value
            .and_then(HeaderFieldValue::manual_override_text)
            .map(str::to_string);
        let confirmed_list_state = match current_source {
            Some(HeaderFieldValue::ListState(value)) => Some(value.clone()),
            _ => None,
        };
        let (list_cursor, labels, field_flow) = if let Some(value) = confirmed_list_state.as_ref() {
            let completed = value.list_idx >= field.lists.len();
            let saved_list_idx = if completed {
                0
            } else {
                value.list_idx.min(field.lists.len().saturating_sub(1))
            };
            let prefix_len = saved_list_idx.min(value.values.len());
            let active_list = &field.lists[saved_list_idx];
            let outputs: Vec<String> = active_list.items.iter().map(item_output).collect();
            let cursor = if let Some(item_id) = value.item_ids.get(saved_list_idx) {
                active_list
                    .items
                    .iter()
                    .position(|item| &item.id == item_id)
                    .or_else(|| {
                        value
                            .values
                            .get(saved_list_idx)
                            .and_then(|saved| outputs.iter().position(|output| output == saved))
                    })
                    .unwrap_or_else(|| list_initial_cursor(active_list, &outputs, lookup))
            } else if let Some(saved) = value.values.get(saved_list_idx) {
                outputs
                    .iter()
                    .position(|output| output == saved)
                    .unwrap_or_else(|| list_initial_cursor(active_list, &outputs, lookup))
            } else {
                list_initial_cursor(active_list, &outputs, lookup)
            };
            let restored_values = value
                .values
                .iter()
                .take(prefix_len)
                .cloned()
                .collect::<Vec<_>>();
            let restored_repeat_values = if completed {
                Vec::new()
            } else {
                value.repeat_values.clone()
            };
            let labels = resolved_item_labels_for_list(
                active_list,
                &restored_values,
                &restored_repeat_values,
                &field.lists,
                &field.format_lists,
                lookup,
            );
            (
                cursor,
                labels,
                FieldModal {
                    format: field.format.clone(),
                    format_lists: field.format_lists.clone(),
                    lists: field.lists.clone(),
                    list_idx: saved_list_idx,
                    values: restored_values,
                    item_ids: value.item_ids.iter().take(prefix_len).cloned().collect(),
                    repeat_values: restored_repeat_values,
                    repeat_item_ids: if completed {
                        Vec::new()
                    } else {
                        value.repeat_item_ids.clone()
                    },
                },
            )
        } else {
            (
                list_initial_cursor(first_list, &outputs, lookup),
                resolved_item_labels_for_list(
                    first_list,
                    &[],
                    &[],
                    &field.lists,
                    &field.format_lists,
                    lookup,
                ),
                FieldModal {
                    format: field.format.clone(),
                    format_lists: field.format_lists.clone(),
                    lists: field.lists.clone(),
                    list_idx: 0,
                    values: Vec::new(),
                    item_ids: Vec::new(),
                    repeat_values: Vec::new(),
                    repeat_item_ids: Vec::new(),
                },
            )
        };
        let active_outputs = field_flow
            .lists
            .get(field_flow.list_idx)
            .map(|list| list.items.iter().map(item_output).collect::<Vec<_>>())
            .unwrap_or(outputs);
        let n = labels.len();
        let (session_cursor_values, session_cursor_item_ids) =
            Self::build_initial_session_cursor_memory(
                field_flow.lists.len(),
                confirmed_list_state.as_ref(),
            );
        let (session_confirmed_values, session_confirmed_item_ids) =
            Self::build_initial_session_confirmed_memory(
                field_flow.lists.len(),
                confirmed_list_state.as_ref(),
            );
        let mut modal = Self {
            field_idx,
            field_id: field.id,
            field_name: field.name,
            query: String::new(),
            all_entries: labels,
            all_outputs: active_outputs,
            filtered: (0..n).collect(),
            list_cursor,
            list_scroll: 0,
            sticky_cursor: list_cursor,
            focus: modal_start_focus(&field_flow.lists[field_flow.list_idx]),
            confirmed_list_state,
            session_cursor_values,
            session_cursor_item_ids,
            session_confirmed_values,
            session_confirmed_item_ids,
            field_flow,
            collection_state: None,
            collection_format: None,
            branch_stack: Vec::new(),
            nested_stack: Vec::new(),
            window_size,
            manual_override,
        };
        modal.update_filter();
        modal
    }

    fn new_nested_field(
        field_idx: usize,
        field: HeaderFieldConfig,
        current_value: Option<&HeaderFieldValue>,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> Self {
        let manual_override = current_value
            .and_then(HeaderFieldValue::manual_override_text)
            .map(str::to_string);
        let current_value = current_value.map(HeaderFieldValue::source_value);
        let nested_state = match current_value {
            Some(HeaderFieldValue::NestedState(state)) => state.as_ref().clone(),
            _ => HeaderState::new(field.fields.clone()),
        };
        let synthetic_field = synthetic_container_field(&field);
        let mut modal = Self::new_field(
            field_idx,
            synthetic_field,
            None,
            assigned_values,
            sticky_values,
            window_size,
        );
        modal.manual_override = manual_override;
        modal.nested_stack.push(NestedFrame {
            field,
            state: nested_state,
        });
        modal.refresh_nested_leaf(assigned_values, sticky_values, window_size);
        modal
    }

    fn new_collection_field(
        field_idx: usize,
        field: HeaderFieldConfig,
        current_value: Option<&HeaderFieldValue>,
        window_size: usize,
    ) -> Self {
        let manual_override = current_value
            .and_then(HeaderFieldValue::manual_override_text)
            .map(str::to_string);
        let current_value = current_value.map(HeaderFieldValue::source_value);
        let labels = collection_labels(&field.collections);
        let n = labels.len();
        let use_default_activation =
            !matches!(current_value, Some(HeaderFieldValue::ExplicitEmpty));
        let mut collection_state = CollectionState::new_with_limits(
            field.collections,
            use_default_activation,
            field.max_actives,
        );
        if let Some(HeaderFieldValue::CollectionState(value)) = current_value {
            restore_collection_state(&mut collection_state, value);
        }
        Self {
            field_idx,
            field_id: field.id,
            field_name: field.name,
            query: String::new(),
            all_entries: labels,
            all_outputs: Vec::new(),
            filtered: (0..n).collect(),
            list_cursor: 0,
            list_scroll: 0,
            sticky_cursor: 0,
            focus: ModalFocus::List,
            confirmed_list_state: None,
            session_cursor_values: Vec::new(),
            session_cursor_item_ids: Vec::new(),
            session_confirmed_values: Vec::new(),
            session_confirmed_item_ids: Vec::new(),
            field_flow: FieldModal {
                format: field.format.clone(),
                format_lists: field.format_lists,
                lists: field.lists,
                list_idx: 0,
                values: Vec::new(),
                item_ids: Vec::new(),
                repeat_values: Vec::new(),
                repeat_item_ids: Vec::new(),
            },
            collection_state: Some(collection_state),
            collection_format: field.format,
            branch_stack: Vec::new(),
            nested_stack: Vec::new(),
            window_size,
            manual_override,
        }
    }

    fn refresh_nested_leaf(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) {
        if self.nested_stack.is_empty() {
            return;
        }
        self.sync_nested_state_chain();
        self.nested_stack.truncate(1);

        loop {
            let Some(frame) = self.nested_stack.last_mut() else {
                return;
            };
            if frame.state.field_configs.is_empty() {
                return;
            }
            if frame.state.field_index >= frame.state.field_configs.len() {
                frame.state.field_index = frame.state.field_configs.len() - 1;
                frame.state.completed = false;
            }
            let active_idx = frame.state.field_index;
            let active_field = frame.state.field_configs[active_idx].clone();
            let value_index = frame.state.active_value_index();
            let current_value = frame
                .state
                .repeated_values
                .get(active_idx)
                .and_then(|values| values.get(value_index))
                .cloned();

            if active_field.fields.is_empty() {
                self.load_leaf_field(
                    active_field,
                    current_value.as_ref(),
                    assigned_values,
                    sticky_values,
                    window_size,
                );
                return;
            }

            let child_state = match current_value {
                Some(HeaderFieldValue::NestedState(state)) => state.as_ref().clone(),
                _ => HeaderState::new(active_field.fields.clone()),
            };
            frame
                .state
                .set_preview_value(HeaderFieldValue::NestedState(Box::new(child_state.clone())));
            self.nested_stack.push(NestedFrame {
                field: active_field,
                state: child_state,
            });
        }
    }

    fn sync_nested_state_chain(&mut self) {
        if self.nested_stack.len() < 2 {
            return;
        }
        for idx in (1..self.nested_stack.len()).rev() {
            let child_state = self.nested_stack[idx].state.clone();
            self.nested_stack[idx - 1]
                .state
                .set_preview_value(HeaderFieldValue::NestedState(Box::new(child_state)));
        }
    }

    fn trim_active_nested_row(frame: &mut NestedFrame) {
        let field_index = frame.state.field_index;
        let retained = frame
            .state
            .repeated_values
            .get(field_index)
            .map(|values| values.len())
            .unwrap_or(0);
        if let Some(visible) = frame.state.repeat_visible_counts.get_mut(field_index) {
            *visible = (*visible).min(retained.max(1));
        }
    }

    fn force_advance_current_nested_field(state: &mut HeaderState) -> bool {
        state.field_index += 1;
        if state.field_index >= state.field_configs.len() {
            state.completed = true;
            if !state.field_configs.is_empty() {
                state.field_index = state.field_configs.len() - 1;
            }
        } else {
            state.completed = false;
        }
        state.completed
    }

    fn complete_nested_value(
        &mut self,
        mut pending_value: HeaderFieldValue,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        loop {
            let Some(frame) = self.nested_stack.last_mut() else {
                return FieldAdvance::Complete(pending_value);
            };
            let completed = if should_terminate_current_repeat(frame, &pending_value) {
                frame.state.clear_active_value();
                Self::trim_active_nested_row(frame);
                Self::force_advance_current_nested_field(&mut frame.state)
            } else {
                frame.state.set_current_value(pending_value);
                frame.state.advance()
            };

            if !completed {
                self.sync_nested_state_chain();
                self.refresh_nested_leaf(assigned_values, sticky_values, window_size);
                return FieldAdvance::NextList;
            }

            let completed_frame = self.nested_stack.pop().unwrap();
            pending_value = HeaderFieldValue::NestedState(Box::new(completed_frame.state));
        }
    }

    fn load_leaf_field(
        &mut self,
        field: HeaderFieldConfig,
        current_value: Option<&HeaderFieldValue>,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) {
        let manual_override = current_value
            .and_then(HeaderFieldValue::manual_override_text)
            .map(str::to_string);
        let current_value = current_value.map(HeaderFieldValue::source_value);
        self.branch_stack.clear();
        if !field.collections.is_empty() && field.lists.is_empty() {
            let labels = collection_labels(&field.collections);
            let n = labels.len();
            let use_default_activation =
                !matches!(current_value, Some(HeaderFieldValue::ExplicitEmpty));
            let mut collection_state = CollectionState::new_with_limits(
                field.collections.clone(),
                use_default_activation,
                field.max_actives,
            );
            if let Some(HeaderFieldValue::CollectionState(value)) = current_value {
                restore_collection_state(&mut collection_state, value);
            }
            self.field_id = field.id;
            self.field_name = field.name;
            self.query = String::new();
            self.all_entries = labels;
            self.all_outputs = Vec::new();
            self.filtered = (0..n).collect();
            self.list_cursor = 0;
            self.list_scroll = 0;
            self.sticky_cursor = 0;
            self.focus = ModalFocus::List;
            self.confirmed_list_state = None;
            self.session_cursor_values = Vec::new();
            self.session_cursor_item_ids = Vec::new();
            self.session_confirmed_values = Vec::new();
            self.session_confirmed_item_ids = Vec::new();
            self.field_flow = FieldModal {
                format: field.format.clone(),
                format_lists: field.format_lists,
                lists: field.lists,
                list_idx: 0,
                values: Vec::new(),
                item_ids: Vec::new(),
                repeat_values: Vec::new(),
                repeat_item_ids: Vec::new(),
            };
            self.collection_state = Some(collection_state);
            self.collection_format = field.format;
            self.window_size = window_size;
            self.manual_override = manual_override;
            return;
        }

        let Some(_first_list) = field.lists.first() else {
            return;
        };
        let confirmed_list_state = match current_value {
            Some(HeaderFieldValue::ListState(value)) => Some(value.clone()),
            _ => None,
        };
        let (
            saved_values,
            saved_item_ids,
            saved_list_idx,
            saved_repeat_values,
            saved_repeat_item_ids,
        ) = if let Some(value) = confirmed_list_state.as_ref() {
            let completed = value.list_idx >= field.lists.len();
            let saved_list_idx = if completed {
                0
            } else {
                value.list_idx.min(field.lists.len().saturating_sub(1))
            };
            let prefix_len = saved_list_idx.min(value.values.len());
            (
                value.values.iter().take(prefix_len).cloned().collect(),
                value.item_ids.iter().take(prefix_len).cloned().collect(),
                saved_list_idx,
                if completed {
                    Vec::new()
                } else {
                    value.repeat_values.clone()
                },
                if completed {
                    Vec::new()
                } else {
                    value.repeat_item_ids.clone()
                },
            )
        } else {
            (Vec::new(), Vec::new(), 0, Vec::new(), Vec::new())
        };
        let active_list = &field.lists[saved_list_idx];
        let outputs: Vec<String> = active_list.items.iter().map(item_output).collect();
        let lookup = ListValueLookup::new(assigned_values, sticky_values);
        self.confirmed_list_state = confirmed_list_state;
        let (session_cursor_values, session_cursor_item_ids) =
            Self::build_initial_session_cursor_memory(
                field.lists.len(),
                self.confirmed_list_state.as_ref(),
            );
        let (session_confirmed_values, session_confirmed_item_ids) =
            Self::build_initial_session_confirmed_memory(
                field.lists.len(),
                self.confirmed_list_state.as_ref(),
            );
        self.session_cursor_values = session_cursor_values;
        self.session_cursor_item_ids = session_cursor_item_ids;
        self.session_confirmed_values = session_confirmed_values;
        self.session_confirmed_item_ids = session_confirmed_item_ids;
        let mut list_cursor =
            self.initial_cursor_for_list(active_list, saved_list_idx, &outputs, lookup);
        if let Some(value) = current_value.and_then(HeaderFieldValue::as_text) {
            if field.lists.len() == 1 {
                if let Some(pos) = outputs.iter().position(|output| output == value) {
                    list_cursor = pos;
                }
            }
        }
        let labels = resolved_item_labels_for_list(
            active_list,
            &saved_values,
            &saved_repeat_values,
            &field.lists,
            &field.format_lists,
            lookup,
        );
        let n = labels.len();
        self.field_id = field.id;
        self.field_name = field.name;
        self.query = String::new();
        self.all_entries = labels;
        self.all_outputs = outputs;
        self.filtered = (0..n).collect();
        self.list_cursor = list_cursor;
        self.list_scroll = 0;
        self.sticky_cursor = list_cursor;
        self.focus = modal_start_focus(active_list);
        self.field_flow = FieldModal {
            format: field.format,
            format_lists: field.format_lists,
            lists: field.lists,
            list_idx: saved_list_idx,
            values: saved_values,
            item_ids: saved_item_ids,
            repeat_values: saved_repeat_values,
            repeat_item_ids: saved_repeat_item_ids,
        };
        self.collection_state = None;
        self.collection_format = None;
        self.window_size = window_size;
        self.manual_override = manual_override;
        self.update_filter();
    }

    pub fn is_collection_mode(&self) -> bool {
        self.collection_state.is_some()
    }

    pub fn current_part_label(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> Option<String> {
        if let Some(state) = &self.collection_state {
            return Some(collection_part_label(self, state));
        }
        let mut merged_assigned = assigned_values.clone();
        merged_assigned.extend(self.assigned_values(sticky_values));
        let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
        self.field_flow
            .lists
            .get(self.field_flow.list_idx)
            .and_then(|list| list.label.as_deref())
            .map(|label| resolve_display_template(label, &self.field_flow, lookup))
    }

    #[allow(dead_code)]
    pub fn preview_collection(&self) -> Option<&crate::sections::collection::CollectionEntry> {
        let state = self.collection_state.as_ref()?;
        let idx = self.current_collection_preview_index()?;
        state.collections.get(idx)
    }

    pub fn collection_preview_strip(&self) -> Option<CollectionPreviewStrip> {
        let state = self.collection_state.as_ref()?;
        let focused_index = self.current_collection_preview_index()?;
        Some(CollectionPreviewStrip {
            previews: state
                .collections
                .iter()
                .enumerate()
                .map(|(idx, collection)| {
                    collection_preview_snapshot(
                        collection,
                        matches!(
                            state.focus,
                            crate::sections::collection::CollectionFocus::Items(collection_idx)
                                if collection_idx == idx
                        )
                        .then_some(state.item_cursor),
                    )
                })
                .collect(),
            focused_index,
        })
    }

    pub fn list_view_snapshot(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> Option<ModalListViewSnapshot> {
        if !self.supports_simple_list_teasers() {
            return None;
        }

        Some(ModalListViewSnapshot {
            title: self
                .current_part_label(assigned_values, sticky_values)
                .unwrap_or_else(|| self.field_name.clone()),
            query: self.query.clone(),
            rows: self.all_entries.clone(),
            filtered: self.filtered.clone(),
            revisiting_completed_field: self.is_revisiting_completed_field(),
            confirmed_row: self.confirmed_row_for_current_list(),
            list_cursor: self.list_cursor,
            list_scroll: self.list_scroll,
            focus: self.focus.clone(),
        })
    }

    pub fn preview_current_list_as_confirmed(
        &self,
        confirmed_value: Option<&str>,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> Option<ModalListViewSnapshot> {
        if !self.supports_simple_list_teasers() {
            return None;
        }
        let list_idx = self.field_flow.list_idx;
        let list = self.field_flow.lists.get(list_idx)?;
        let lookup = ListValueLookup::new(assigned_values, sticky_values);
        let value = confirmed_value
            .map(str::to_string)
            .or_else(|| self.selected_value().map(str::to_string))
            .or_else(|| current_list_fallback_value(self, lookup))?;
        let item_id = self
            .selected_item_id()
            .filter(|item_id| {
                list.items.iter().any(|item| {
                    item.id == *item_id
                        && (item.output() == value.as_str() || item.ui_label() == value.as_str())
                })
            })
            .map(str::to_string)
            .or_else(|| {
                list.items
                    .iter()
                    .find(|item| {
                        item.output() == value.as_str() || item.ui_label() == value.as_str()
                    })
                    .map(|item| item.id.clone())
            })
            .or_else(|| current_list_fallback_item_id(self, lookup));
        let mut preview = self.clone();
        if preview.session_confirmed_values.len() > list_idx {
            preview.session_confirmed_values[list_idx] = Some(value);
            preview.session_confirmed_item_ids[list_idx] = item_id;
        }
        preview.list_view_snapshot(assigned_values, sticky_values)
    }

    #[allow(dead_code)]
    pub fn peek_prev_list_view(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> Option<ModalListViewSnapshot> {
        if !self.supports_simple_list_teasers() || self.field_flow.list_idx == 0 {
            return None;
        }

        let mut preview = self.clone();
        if !preview.go_back_simple_list(assigned_values, sticky_values) {
            return None;
        }
        preview.list_view_snapshot(assigned_values, sticky_values)
    }

    pub fn peek_prev_list_views(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        limit: usize,
    ) -> Vec<ModalListViewSnapshot> {
        if limit == 0 || !self.supports_simple_list_teasers() || self.field_flow.list_idx == 0 {
            return Vec::new();
        }

        let mut preview = self.clone();
        let mut snapshots = Vec::new();
        while snapshots.len() < limit && preview.go_back_simple_list(assigned_values, sticky_values)
        {
            let Some(snapshot) = preview.list_view_snapshot(assigned_values, sticky_values) else {
                break;
            };
            snapshots.push(snapshot);
        }
        snapshots
    }

    fn advance_preview_to_next_list_snapshot(
        &mut self,
        assigned_values: &mut HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
    ) -> Option<ModalListViewSnapshot> {
        let Some(list) = self.field_flow.lists.get(self.field_flow.list_idx) else {
            return None;
        };

        let advance = if effective_joiner_style(list).is_some() {
            self.finish_repeating_current_list(
                self.selected_value().map(str::to_string),
                self.selected_item_id().map(str::to_string),
                assigned_values,
                sticky_values,
                self.window_size,
            )?
        } else {
            let (selected, selected_item_id) = self.navigation_choice_for_current_list();
            let selected = selected?;
            if branch_for_value(list, &selected).is_some() {
                return None;
            }
            self.finish_current_list(
                selected,
                selected_item_id,
                assigned_values,
                sticky_values,
                self.window_size,
            )
        };

        match advance {
            FieldAdvance::NextList => self.list_view_snapshot(assigned_values, sticky_values),
            FieldAdvance::Complete(_) | FieldAdvance::StayOnList => None,
        }
    }

    #[allow(dead_code)]
    pub fn peek_next_list_view(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> Option<ModalListViewSnapshot> {
        if !self.supports_simple_list_teasers() {
            return None;
        }

        let mut preview = self.clone();
        let mut sticky_preview = sticky_values.clone();
        let mut assigned_preview = assigned_values.clone();
        preview.advance_preview_to_next_list_snapshot(&mut assigned_preview, &mut sticky_preview)
    }

    pub fn peek_next_list_views(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        limit: usize,
    ) -> Vec<ModalListViewSnapshot> {
        if limit == 0 || !self.supports_simple_list_teasers() {
            return Vec::new();
        }

        let mut preview = self.clone();
        let mut assigned_preview = assigned_values.clone();
        let mut sticky_preview = sticky_values.clone();
        let mut snapshots = Vec::new();
        while snapshots.len() < limit {
            let Some(snapshot) = preview
                .advance_preview_to_next_list_snapshot(&mut assigned_preview, &mut sticky_preview)
            else {
                break;
            };
            snapshots.push(snapshot);
        }
        snapshots
    }

    pub fn simple_modal_sequence(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
    ) -> Option<SimpleModalSequence> {
        let current_snapshot = self.list_view_snapshot(assigned_values, sticky_values)?;
        let prev_limit = self.field_flow.list_idx;
        let prev_snapshots = self.peek_prev_list_views(assigned_values, sticky_values, prev_limit);
        let next_limit = self
            .field_flow
            .lists
            .len()
            .saturating_sub(self.field_flow.list_idx + 1);
        let next_snapshots = self.peek_next_list_views(assigned_values, sticky_values, next_limit);
        let active_sequence_index = prev_snapshots.len();
        let mut snapshots = prev_snapshots.into_iter().rev().collect::<Vec<_>>();
        snapshots.push(current_snapshot);
        snapshots.extend(next_snapshots);
        Some(SimpleModalSequence {
            snapshots,
            active_sequence_index,
        })
    }

    pub fn simple_modal_unit_layout(
        &self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        viewport_width: Option<f32>,
        spacer_width: f32,
        stub_width: f32,
    ) -> Option<SimpleModalUnitLayout> {
        let sequence = self.simple_modal_sequence(assigned_values, sticky_values)?;
        build_simple_modal_unit_layout(sequence, viewport_width, spacer_width, stub_width)
    }

    #[allow(dead_code)]
    pub fn composite_progress(&self) -> Option<String> {
        if self.field_flow.values.is_empty() {
            None
        } else {
            Some(self.field_flow.values.join(" / "))
        }
    }

    pub fn update_filter(&mut self) {
        if self.query.trim().is_empty() {
            self.filtered = (0..self.all_entries.len()).collect();
            self.list_cursor = self.sticky_cursor;
        } else {
            self.filtered = self
                .all_entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| modal_query_matches(entry, &self.query))
                .map(|(i, _)| i)
                .collect();
            if let Some(pos) = self.filtered.iter().position(|&i| i == self.sticky_cursor) {
                self.list_cursor = pos;
            } else {
                self.list_cursor = 0;
            }
        }
        self.center_scroll();
    }

    pub fn update_scroll(&mut self) {
        self.center_scroll();
    }

    pub fn center_scroll(&mut self) {
        let w = self.window_size.max(1);
        self.list_scroll = self.list_cursor.saturating_sub(w.saturating_sub(1) / 2);
        let max_scroll = self.filtered.len().saturating_sub(w);
        if self.list_scroll > max_scroll {
            self.list_scroll = max_scroll;
        }
        self.remember_current_cursor();
    }

    pub fn selected_value(&self) -> Option<&str> {
        if self.collection_state.is_some() {
            return None;
        }
        self.filtered
            .get(self.list_cursor)
            .and_then(|&i| self.all_outputs.get(i))
            .map(String::as_str)
    }

    pub fn selected_item_id(&self) -> Option<&str> {
        if self.collection_state.is_some() {
            return None;
        }
        self.filtered
            .get(self.list_cursor)
            .and_then(|&i| {
                self.field_flow
                    .lists
                    .get(self.field_flow.list_idx)?
                    .items
                    .get(i)
            })
            .map(|item| item.id.as_str())
    }

    pub fn should_finish_repeating_from_empty_search(&self) -> bool {
        if !self.query.trim().is_empty() {
            return false;
        }
        self.field_flow
            .lists
            .get(self.field_flow.list_idx)
            .is_some_and(|list| {
                effective_joiner_style(list).is_some()
                    && matches!(&list.modal_start, ModalStart::Search)
                    && !self.field_flow.repeat_values.is_empty()
                    && self.focus == ModalFocus::SearchBar
            })
    }

    pub fn should_confirm_empty_search_value(&self) -> bool {
        if self.should_finish_repeating_from_empty_search() {
            return true;
        }
        if self.focus != ModalFocus::SearchBar || !self.query.trim().is_empty() {
            return false;
        }
        self.selected_value().is_some_and(str::is_empty)
            && self.nested_stack.iter().any(|frame| {
                frame
                    .state
                    .field_configs
                    .get(frame.state.field_index)
                    .is_some_and(|cfg| cfg.max_entries.is_some())
                    && frame.state.active_value_index() > 0
            })
    }

    pub fn hint_value(&self, hint_pos: usize) -> Option<&str> {
        if self.collection_state.is_some() {
            return None;
        }
        self.filtered
            .get(self.list_scroll + hint_pos)
            .and_then(|&i| self.all_outputs.get(i))
            .map(String::as_str)
    }

    pub fn restore_parent_branch(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> bool {
        if self.collection_state.is_some() {
            return false;
        }
        let Some(frame) = self.branch_stack.pop() else {
            return false;
        };
        self.field_flow = frame.parent_flow;
        self.reload_current_list(assigned_values, sticky_values, window_size);
        true
    }

    fn preview_active_leaf_value(
        &self,
        sticky_values: &HashMap<String, String>,
    ) -> HeaderFieldValue {
        if self.collection_state.is_some() {
            return self.collection_value();
        }
        let _ = sticky_values;
        HeaderFieldValue::ListState(ListFieldValue {
            values: self.field_flow.values.clone(),
            item_ids: self.field_flow.item_ids.clone(),
            list_idx: self.field_flow.list_idx,
            repeat_values: self.field_flow.repeat_values.clone(),
            repeat_item_ids: self.field_flow.repeat_item_ids.clone(),
        })
    }

    fn committed_list_state_value(&self) -> Option<ListFieldValue> {
        if self.collection_state.is_some() {
            return None;
        }
        let list_count = self.field_flow.lists.len();
        if list_count == 0 {
            return None;
        }
        let mut values = Vec::with_capacity(list_count);
        let mut item_ids = Vec::with_capacity(list_count);
        for idx in 0..list_count {
            let (value, item_id) = self.authoritative_choice_for_list(idx);
            values.push(value?.to_string());
            item_ids.push(item_id.unwrap_or_default().to_string());
        }
        Some(ListFieldValue {
            values,
            item_ids,
            list_idx: list_count,
            repeat_values: Vec::new(),
            repeat_item_ids: Vec::new(),
        })
    }

    pub fn confirmed_field_value_if_complete(
        &self,
        _sticky_values: &HashMap<String, String>,
    ) -> Option<HeaderFieldValue> {
        if self.collection_state.is_some() {
            return Some(self.collection_value());
        }
        let leaf_value = HeaderFieldValue::ListState(self.committed_list_state_value()?);
        Some(
            self.synced_nested_root_state_with_leaf_value(leaf_value.clone())
                .map(|root| HeaderFieldValue::NestedState(Box::new(root)))
                .unwrap_or(leaf_value),
        )
    }

    fn advance_nested_field(
        &mut self,
        value: String,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        let advance =
            self.advance_active_leaf_field(value, assigned_values, sticky_values, window_size);
        match advance {
            FieldAdvance::NextList | FieldAdvance::StayOnList => {
                let preview = self.preview_active_leaf_value(sticky_values);
                if let Some(frame) = self.nested_stack.last_mut() {
                    frame.state.set_preview_value(preview);
                }
                self.sync_nested_state_chain();
                advance
            }
            FieldAdvance::Complete(final_value) => {
                self.complete_nested_value(final_value, assigned_values, sticky_values, window_size)
            }
        }
    }

    fn super_confirm_nested_field(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        let advance =
            self.super_confirm_active_leaf_field(assigned_values, sticky_values, window_size);
        match advance {
            FieldAdvance::NextList | FieldAdvance::StayOnList => {
                let preview = self.preview_active_leaf_value(sticky_values);
                if let Some(frame) = self.nested_stack.last_mut() {
                    frame.state.set_preview_value(preview);
                }
                self.sync_nested_state_chain();
                advance
            }
            FieldAdvance::Complete(final_value) => {
                self.complete_nested_value(final_value, assigned_values, sticky_values, window_size)
            }
        }
    }

    pub fn go_back_one_step(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> bool {
        if self.nested_stack.is_empty() {
            return false;
        }
        if self.collection_back() {
            let preview = self.preview_active_leaf_value(sticky_values);
            if let Some(frame) = self.nested_stack.last_mut() {
                frame.state.set_preview_value(preview);
            }
            self.sync_nested_state_chain();
            return true;
        }
        if self.restore_parent_branch(assigned_values, sticky_values, window_size) {
            let preview = self.preview_active_leaf_value(sticky_values);
            if let Some(frame) = self.nested_stack.last_mut() {
                frame.state.set_preview_value(preview);
            }
            self.sync_nested_state_chain();
            return true;
        }
        if self.field_flow.list_idx > 0 {
            let popped = self.field_flow.values.pop();
            let popped_item_id = self.field_flow.item_ids.pop();
            self.field_flow.list_idx -= 1;
            let list = &self.field_flow.lists[self.field_flow.list_idx];
            let mut merged_assigned = assigned_values.clone();
            merged_assigned.extend(self.assigned_values(sticky_values));
            let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
            let labels = resolved_item_labels_for_list(
                list,
                &self.field_flow.values,
                &self.field_flow.repeat_values,
                &self.field_flow.lists,
                &self.field_flow.format_lists,
                lookup,
            );
            let outputs: Vec<String> = list.items.iter().map(item_output).collect();
            let cursor = popped_item_id
                .as_ref()
                .filter(|item_id| !item_id.is_empty())
                .and_then(|item_id| list.items.iter().position(|item| item.id == **item_id))
                .or_else(|| {
                    popped
                        .as_ref()
                        .and_then(|value| outputs.iter().position(|output| output == value))
                })
                .unwrap_or(0);
            self.all_entries = labels;
            self.all_outputs = outputs;
            self.list_cursor = cursor;
            self.sticky_cursor = cursor;
            self.query = String::new();
            self.list_scroll = 0;
            self.focus = ModalFocus::List;
            self.update_filter();
            let preview = self.preview_active_leaf_value(sticky_values);
            if let Some(frame) = self.nested_stack.last_mut() {
                frame.state.set_preview_value(preview);
            }
            self.sync_nested_state_chain();
            return true;
        }

        loop {
            let Some(frame) = self.nested_stack.last_mut() else {
                return false;
            };
            if frame.state.go_back() {
                self.sync_nested_state_chain();
                self.refresh_nested_leaf(assigned_values, sticky_values, window_size);
                return true;
            }
            if self.nested_stack.len() == 1 {
                return false;
            }
            self.nested_stack.pop();
        }
    }

    pub fn advance_field(
        &mut self,
        value: String,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        if !self.nested_stack.is_empty() {
            return self.advance_nested_field(value, assigned_values, sticky_values, window_size);
        }
        self.advance_active_leaf_field(value, assigned_values, sticky_values, window_size)
    }

    fn advance_active_leaf_field(
        &mut self,
        value: String,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        if self.collection_state.is_some() {
            return FieldAdvance::Complete(self.collection_value());
        }
        let list = &self.field_flow.lists[self.field_flow.list_idx];
        if let Some((output_format, branch_fields)) = branch_for_value(list, &value) {
            return self.start_branch(
                output_format,
                branch_fields,
                assigned_values,
                sticky_values,
                window_size,
            );
        }
        if effective_joiner_style(list).is_some() {
            if value.trim().is_empty() {
                return self
                    .finish_repeating_current_list(
                        Some(value),
                        self.selected_item_id().map(str::to_string),
                        assigned_values,
                        sticky_values,
                        window_size,
                    )
                    .unwrap_or(FieldAdvance::StayOnList);
            }
            if list.sticky {
                sticky_values.insert(list.id.clone(), value.clone());
            }
            self.field_flow.repeat_values.push(value);
            if let Some(item_id) = self.selected_item_id() {
                self.field_flow.repeat_item_ids.push(item_id.to_string());
            }
            let mut merged_assigned = assigned_values.clone();
            merged_assigned.extend(self.assigned_values(sticky_values));
            let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
            if list
                .max_entries
                .is_some_and(|limit| self.field_flow.repeat_values.len() >= limit)
            {
                return self
                    .finish_repeating_current_list(
                        None,
                        None,
                        assigned_values,
                        sticky_values,
                        window_size,
                    )
                    .unwrap_or(FieldAdvance::StayOnList);
            }
            self.all_entries = resolved_item_labels_for_list(
                list,
                &self.field_flow.values,
                &self.field_flow.repeat_values,
                &self.field_flow.lists,
                &self.field_flow.format_lists,
                lookup,
            );
            self.query = String::new();
            self.focus = modal_start_focus(list);
            self.update_filter();
            return FieldAdvance::StayOnList;
        }
        let advance = self.finish_current_list(
            value,
            self.selected_item_id().map(str::to_string),
            assigned_values,
            sticky_values,
            window_size,
        );
        self.resolve_branch_advance(advance, assigned_values, sticky_values, window_size)
    }

    pub fn super_confirm_field(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        if !self.nested_stack.is_empty() {
            return self.super_confirm_nested_field(assigned_values, sticky_values, window_size);
        }
        self.super_confirm_active_leaf_field(assigned_values, sticky_values, window_size)
    }

    fn super_confirm_active_leaf_field(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        if self.collection_state.is_some() {
            return FieldAdvance::Complete(self.collection_value());
        }
        let selected = self.selected_value().map(str::to_string);
        if let Some(advance) = self.finish_repeating_current_list(
            selected,
            self.selected_item_id().map(str::to_string),
            assigned_values,
            sticky_values,
            window_size,
        ) {
            return advance;
        } else {
            let lookup = ListValueLookup::new(assigned_values, sticky_values);
            let value = self
                .selected_value()
                .map(str::to_string)
                .or_else(|| current_list_fallback_value(self, lookup))
                .unwrap_or_default();
            if let Some((output_format, branch_fields)) =
                branch_for_value(&self.field_flow.lists[self.field_flow.list_idx], &value)
            {
                return self.start_branch(
                    output_format,
                    branch_fields,
                    assigned_values,
                    sticky_values,
                    window_size,
                );
            }
            let advance = self.finish_current_list(
                value,
                self.selected_item_id()
                    .map(str::to_string)
                    .or_else(|| current_list_fallback_item_id(self, lookup)),
                assigned_values,
                sticky_values,
                window_size,
            );
            let advance =
                self.resolve_branch_advance(advance, assigned_values, sticky_values, window_size);
            if matches!(advance, FieldAdvance::Complete(_)) {
                return advance;
            }
        }

        while self.field_flow.list_idx < self.field_flow.lists.len() {
            let lookup = ListValueLookup::new(assigned_values, sticky_values);
            let Some(value) = current_list_fallback_value(self, lookup) else {
                break;
            };
            let advance = self.finish_current_list(
                value,
                current_list_fallback_item_id(self, lookup),
                assigned_values,
                sticky_values,
                window_size,
            );
            let advance =
                self.resolve_branch_advance(advance, assigned_values, sticky_values, window_size);
            if matches!(advance, FieldAdvance::Complete(_)) {
                return advance;
            }
        }
        FieldAdvance::NextList
    }

    fn finish_repeating_current_list(
        &mut self,
        selected: Option<String>,
        selected_item_id: Option<String>,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> Option<FieldAdvance> {
        let Some(list) = self.field_flow.lists.get(self.field_flow.list_idx) else {
            return None;
        };
        let Some(style) = effective_joiner_style(list) else {
            return None;
        };
        let mut values = self.field_flow.repeat_values.clone();
        let mut item_ids = self.field_flow.repeat_item_ids.clone();
        if let Some(value) = selected {
            values.push(value);
            if let Some(item_id) = selected_item_id {
                item_ids.push(item_id);
            }
        } else if values.is_empty() {
            let lookup = ListValueLookup::new(assigned_values, sticky_values);
            if let Some(value) = current_list_fallback_value(self, lookup) {
                values.push(value);
                if let Some(item_id) = current_list_fallback_item_id(self, lookup) {
                    item_ids.push(item_id);
                }
            }
        }
        if list.sticky {
            if let Some(value) = values.last() {
                sticky_values.insert(list.id.clone(), value.clone());
            }
        }
        let joined = join_repeat_values(&values, style);
        self.field_flow.repeat_values.clear();
        self.field_flow.repeat_item_ids.clear();
        let advance = self.finish_current_list(
            joined,
            item_ids.last().cloned(),
            assigned_values,
            sticky_values,
            window_size,
        );
        Some(self.resolve_branch_advance(advance, assigned_values, sticky_values, window_size))
    }

    fn finish_current_list(
        &mut self,
        value: String,
        item_id: Option<String>,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        let list = &self.field_flow.lists[self.field_flow.list_idx];
        let resolved_item_id = item_id
            .filter(|candidate| {
                list.items.iter().any(|item| {
                    item.id == *candidate
                        && (item.output() == value.as_str() || item.ui_label() == value.as_str())
                })
            })
            .or_else(|| {
                list.items
                    .iter()
                    .find(|item| {
                        item.output() == value.as_str() || item.ui_label() == value.as_str()
                    })
                    .map(|item| item.id.clone())
            });
        if list.sticky && effective_joiner_style(list).is_none() {
            sticky_values.insert(list.id.clone(), value.clone());
        }
        if self.session_confirmed_values.len() > self.field_flow.list_idx {
            self.session_confirmed_values[self.field_flow.list_idx] = Some(value.clone());
            self.session_confirmed_item_ids[self.field_flow.list_idx] = resolved_item_id.clone();
        }
        self.field_flow.values.push(value);
        self.field_flow
            .item_ids
            .push(resolved_item_id.unwrap_or_default());
        self.field_flow.list_idx += 1;

        if self.field_flow.list_idx >= self.field_flow.lists.len() {
            let mut merged_assigned = assigned_values.clone();
            merged_assigned.extend(self.assigned_values(sticky_values));
            let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
            return FieldAdvance::Complete(HeaderFieldValue::Text(format_field_value(
                &self.field_flow,
                lookup,
            )));
        }

        let next_list = &self.field_flow.lists[self.field_flow.list_idx];
        let next_outputs: Vec<String> = next_list.items.iter().map(item_output).collect();
        let mut merged_assigned = assigned_values.clone();
        merged_assigned.extend(self.assigned_values(sticky_values));
        let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
        let next_labels = resolved_item_labels_for_list(
            next_list,
            &self.field_flow.values,
            &[],
            &self.field_flow.lists,
            &self.field_flow.format_lists,
            lookup,
        );
        let next_focus = modal_start_focus(next_list);
        self.query = String::new();
        self.list_cursor = self.initial_cursor_for_list(
            next_list,
            self.field_flow.list_idx,
            &next_outputs,
            lookup,
        );
        self.sticky_cursor = self.list_cursor;
        self.window_size = window_size;
        self.all_entries = next_labels;
        self.all_outputs = next_outputs;
        self.list_scroll = 0;
        self.update_filter();
        self.focus = next_focus;
        FieldAdvance::NextList
    }

    fn reload_current_list(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) {
        let Some(list) = self.field_flow.lists.get(self.field_flow.list_idx) else {
            return;
        };
        let next_focus = modal_start_focus(list);
        let outputs: Vec<String> = list.items.iter().map(item_output).collect();
        let mut merged_assigned = assigned_values.clone();
        merged_assigned.extend(self.assigned_values(sticky_values));
        let lookup = ListValueLookup::new(&merged_assigned, sticky_values);
        let labels = resolved_item_labels_for_list(
            list,
            &self.field_flow.values,
            &self.field_flow.repeat_values,
            &self.field_flow.lists,
            &self.field_flow.format_lists,
            lookup,
        );
        self.query = String::new();
        self.list_cursor =
            self.initial_cursor_for_list(list, self.field_flow.list_idx, &outputs, lookup);
        self.sticky_cursor = self.list_cursor;
        self.window_size = window_size;
        self.all_entries = labels;
        self.all_outputs = outputs;
        self.list_scroll = 0;
        self.update_filter();
        self.focus = next_focus;
    }

    pub fn move_right_without_confirm(
        &mut self,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> bool {
        if self.collection_state.is_some() {
            return false;
        }
        if self.field_flow.list_idx + 1 >= self.field_flow.lists.len() {
            return false;
        }
        let list = &self.field_flow.lists[self.field_flow.list_idx];
        if effective_joiner_style(list).is_some() {
            return false;
        }
        let (value, item_id) = self.navigation_choice_for_current_list();
        let Some(value) = value else {
            return false;
        };
        if branch_for_value(list, &value).is_some() {
            return false;
        }

        self.field_flow.values.push(value);
        self.field_flow.item_ids.push(item_id.unwrap_or_default());
        self.field_flow.list_idx += 1;
        self.reload_current_list(assigned_values, sticky_values, window_size);
        true
    }

    fn start_branch(
        &mut self,
        output_format: String,
        branch_fields: Vec<HeaderFieldConfig>,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        let Some(first_field) = branch_fields.first().cloned() else {
            return FieldAdvance::NextList;
        };
        let parent_flow = self.field_flow.clone();
        self.branch_stack.push(BranchFrame {
            parent_flow,
            output_format,
            branch_fields,
            field_idx: 0,
            values: Vec::new(),
        });
        self.load_field_flow(first_field, assigned_values, sticky_values, window_size);
        FieldAdvance::NextList
    }

    fn load_field_flow(
        &mut self,
        field: HeaderFieldConfig,
        assigned_values: &HashMap<String, String>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) {
        if !field.fields.is_empty() {
            let output_format = composite_output_format(&field);
            let branch_fields = field.fields.clone();
            let synthetic_field = synthetic_container_field(&field);
            self.load_field_flow(synthetic_field, assigned_values, sticky_values, window_size);
            let _ = self.start_branch(
                output_format,
                branch_fields,
                assigned_values,
                sticky_values,
                window_size,
            );
            return;
        }
        if !field.collections.is_empty() && field.lists.is_empty() {
            let labels = collection_labels(&field.collections);
            let n = labels.len();
            self.field_id = field.id;
            self.field_name = field.name;
            self.query = String::new();
            self.all_entries = labels;
            self.all_outputs = Vec::new();
            self.filtered = (0..n).collect();
            self.list_cursor = 0;
            self.list_scroll = 0;
            self.sticky_cursor = 0;
            self.focus = ModalFocus::List;
            self.confirmed_list_state = None;
            self.session_cursor_values = Vec::new();
            self.session_cursor_item_ids = Vec::new();
            self.field_flow = FieldModal {
                format: field.format.clone(),
                format_lists: field.format_lists,
                lists: field.lists,
                list_idx: 0,
                values: Vec::new(),
                item_ids: Vec::new(),
                repeat_values: Vec::new(),
                repeat_item_ids: Vec::new(),
            };
            self.collection_state = Some(CollectionState::new_with_limits(
                field.collections,
                true,
                field.max_actives,
            ));
            self.collection_format = field.format;
            self.window_size = window_size;
            return;
        }
        let Some(first_list) = field.lists.first() else {
            return;
        };
        let outputs: Vec<String> = first_list.items.iter().map(item_output).collect();
        let lookup = ListValueLookup::new(assigned_values, sticky_values);
        let list_cursor = list_initial_cursor(first_list, &outputs, lookup);
        let labels = resolved_item_labels_for_list(
            first_list,
            &[],
            &[],
            &field.lists,
            &field.format_lists,
            lookup,
        );
        let n = labels.len();
        self.field_id = field.id;
        self.field_name = field.name;
        self.query = String::new();
        self.all_entries = labels;
        self.all_outputs = outputs;
        self.filtered = (0..n).collect();
        self.list_cursor = list_cursor;
        self.list_scroll = 0;
        self.sticky_cursor = list_cursor;
        self.focus = modal_start_focus(first_list);
        self.confirmed_list_state = None;
        self.session_cursor_values = vec![None; field.lists.len()];
        self.session_cursor_item_ids = vec![None; field.lists.len()];
        self.session_confirmed_values = vec![None; field.lists.len()];
        self.session_confirmed_item_ids = vec![None; field.lists.len()];
        self.field_flow = FieldModal {
            format: field.format,
            format_lists: field.format_lists,
            lists: field.lists,
            list_idx: 0,
            values: Vec::new(),
            item_ids: Vec::new(),
            repeat_values: Vec::new(),
            repeat_item_ids: Vec::new(),
        };
        self.collection_state = None;
        self.collection_format = None;
        self.window_size = window_size;
        self.update_filter();
    }

    fn resolve_branch_advance(
        &mut self,
        mut advance: FieldAdvance,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        while let FieldAdvance::Complete(value) = advance {
            if self.branch_stack.is_empty() {
                return FieldAdvance::Complete(value);
            }
            advance = self.complete_branch_field(
                value.as_text().unwrap_or_default().to_string(),
                assigned_values,
                sticky_values,
                window_size,
            );
        }
        advance
    }

    fn complete_branch_field(
        &mut self,
        value: String,
        assigned_values: &HashMap<String, String>,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        let Some(frame) = self.branch_stack.last_mut() else {
            return FieldAdvance::Complete(HeaderFieldValue::Text(value));
        };
        let field_id = frame
            .branch_fields
            .get(frame.field_idx)
            .map(|field| field.id.clone())
            .unwrap_or_default();
        frame.values.push((field_id, value));

        if frame.field_idx + 1 < frame.branch_fields.len() {
            frame.field_idx += 1;
            let next_field = frame.branch_fields[frame.field_idx].clone();
            self.load_field_flow(next_field, assigned_values, sticky_values, window_size);
            return FieldAdvance::NextList;
        }

        let frame = self.branch_stack.pop().unwrap();
        let branch_value = format_branch_output(&frame.output_format, &frame.values);
        self.field_flow = frame.parent_flow;
        self.advance_field(branch_value, assigned_values, sticky_values, window_size)
    }

    pub fn collection_navigate_up(&mut self) {
        if let Some(state) = self.collection_state.as_mut() {
            state.navigate_up();
        }
    }

    pub fn collection_navigate_down(&mut self) {
        if let Some(state) = self.collection_state.as_mut() {
            state.navigate_down();
        }
    }

    pub fn collection_toggle_current(&mut self) -> Vec<String> {
        if let Some(state) = self.collection_state.as_mut() {
            if state.in_items() {
                return state.toggle_current_item();
            } else {
                return state.toggle_current_collection();
            }
        }
        Vec::new()
    }

    pub fn collection_enter(&mut self) {
        if let Some(state) = self.collection_state.as_mut() {
            if !state.in_items() {
                state.enter_collection();
            }
        }
    }

    pub fn collection_back(&mut self) -> bool {
        if let Some(state) = self.collection_state.as_mut() {
            if state.in_items() {
                state.exit_items();
                return true;
            }
        }
        false
    }

    pub fn collection_preview(&self) -> String {
        self.collection_display_value()
    }

    fn collection_display_value(&self) -> String {
        let Some(state) = &self.collection_state else {
            return String::new();
        };
        if let Some(format) = &self.collection_format {
            let groups = collection_group_values(&state.collections, true);
            let mut result = format.clone();
            let mut replaced_any = false;
            for (id, value) in &groups {
                let placeholder = format!("{{{id}}}");
                if result.contains(&placeholder) {
                    result = result.replace(&placeholder, value);
                    replaced_any = true;
                }
            }
            if replaced_any {
                return result;
            }
            return groups
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>()
                .join("; ");
        }
        format_collection_field_value(&state.collections, false)
    }

    pub fn preview_field_value(&self, sticky_values: &HashMap<String, String>) -> HeaderFieldValue {
        if let Some(root) = self.synced_nested_root_state(sticky_values) {
            return HeaderFieldValue::NestedState(Box::new(root));
        }
        self.preview_active_leaf_value(sticky_values)
    }

    fn collection_value(&self) -> HeaderFieldValue {
        let Some(state) = &self.collection_state else {
            return HeaderFieldValue::ExplicitEmpty;
        };
        HeaderFieldValue::CollectionState(collection_field_value_from_state(state))
    }
}

fn collect_assignments_from_flow(
    flow: &FieldModal,
    sticky_values: &HashMap<String, String>,
    assigned: &mut HashMap<String, String>,
) {
    collect_assignments_from_list_parts(
        flow,
        &flow.item_ids,
        flow.list_idx,
        &flow.repeat_item_ids,
        assigned,
    );
    resolve_assigned_values(flow, sticky_values, assigned);
}

fn collect_assignments_from_list_parts(
    flow: &FieldModal,
    item_ids: &[String],
    list_idx: usize,
    repeat_item_ids: &[String],
    assigned: &mut HashMap<String, String>,
) {
    for (idx, item_id) in item_ids.iter().enumerate() {
        if let Some(list) = flow.lists.get(idx) {
            apply_item_assignments(list, item_id, assigned);
        }
    }
    if let Some(list) = flow.lists.get(list_idx) {
        for item_id in repeat_item_ids {
            apply_item_assignments(list, item_id, assigned);
        }
    }
}

fn collect_assignments_from_state(
    state: &HeaderState,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
    assigned: &mut HashMap<String, String>,
) {
    for (idx, child_cfg) in cfg.fields.iter().enumerate() {
        let values = state
            .repeated_values
            .get(idx)
            .map(|values| values.as_slice())
            .unwrap_or(&[]);
        for value in values {
            collect_assignments_from_value(value, child_cfg, sticky_values, assigned);
        }
    }
}

fn collect_assignments_from_value(
    value: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
    assigned: &mut HashMap<String, String>,
) {
    match value.source_value() {
        HeaderFieldValue::ListState(value) => {
            let flow = FieldModal {
                format: cfg.format.clone(),
                lists: cfg.lists.clone(),
                format_lists: cfg.format_lists.clone(),
                list_idx: value.list_idx,
                values: value.values.clone(),
                item_ids: value.item_ids.clone(),
                repeat_values: value.repeat_values.clone(),
                repeat_item_ids: value.repeat_item_ids.clone(),
            };
            collect_assignments_from_flow(&flow, sticky_values, assigned);
        }
        HeaderFieldValue::NestedState(state) => {
            collect_assignments_from_state(state, cfg, sticky_values, assigned)
        }
        _ => {}
    }
}

fn apply_item_assignments(
    list: &HierarchyList,
    item_id: &str,
    assigned: &mut HashMap<String, String>,
) {
    let Some(item) = list.items.iter().find(|item| item.id == item_id) else {
        return;
    };
    for assign in &item.assigns {
        assigned.insert(assign.list_id.clone(), assign.output.clone());
    }
}

fn resolve_assigned_values(
    flow: &FieldModal,
    sticky_values: &HashMap<String, String>,
    assigned: &mut HashMap<String, String>,
) {
    let raw_assigned = assigned.clone();
    let lookup = ListValueLookup::new(&raw_assigned, sticky_values);
    for (list_id, raw_value) in raw_assigned.iter() {
        assigned.insert(
            list_id.clone(),
            resolve_display_template(raw_value, flow, lookup),
        );
    }
}

fn branch_for_value(list: &HierarchyList, value: &str) -> Option<(String, Vec<HeaderFieldConfig>)> {
    list.items
        .iter()
        .find(|item| item_output(item) == value && !item.branch_fields.is_empty())
        .map(|item| (item.output().to_string(), item.branch_fields.clone()))
}

fn should_terminate_current_repeat(frame: &NestedFrame, final_value: &HeaderFieldValue) -> bool {
    let Some(cfg) = frame.state.field_configs.get(frame.state.field_index) else {
        return false;
    };
    if cfg.max_entries.is_none() {
        return false;
    }
    frame.state.active_value_index() > 0 && !has_concrete_field_content(final_value, cfg)
}

fn has_concrete_field_content(value: &HeaderFieldValue, cfg: &HeaderFieldConfig) -> bool {
    match value {
        HeaderFieldValue::ExplicitEmpty => false,
        HeaderFieldValue::Text(value) => !value.trim().is_empty(),
        HeaderFieldValue::ManualOverride { text, .. } => !text.trim().is_empty(),
        HeaderFieldValue::ListState(value) => {
            value.values.iter().any(|value| !value.trim().is_empty())
                || value
                    .repeat_values
                    .iter()
                    .any(|value| !value.trim().is_empty())
        }
        HeaderFieldValue::CollectionState(value) => decode_collection_display_value(value, cfg)
            .is_some_and(|value| !value.trim().is_empty()),
        HeaderFieldValue::NestedState(state) => {
            cfg.fields.iter().enumerate().any(|(idx, child)| {
                state.repeated_values.get(idx).is_some_and(|values| {
                    values
                        .iter()
                        .any(|value| has_concrete_field_content(value, child))
                })
            })
        }
    }
}

fn format_branch_output(output_format: &str, values: &[(String, String)]) -> String {
    let mut result = output_format.to_string();
    for (field_id, value) in values {
        result = result.replace(&format!("{{{field_id}}}"), value);
    }
    result
}

fn item_id_for_output(list: &HierarchyList, value: &str) -> Option<String> {
    list.items
        .iter()
        .find(|item| item_output(item) == value)
        .map(|item| item.id.clone())
}

fn modal_start_focus(list: &HierarchyList) -> ModalFocus {
    match list.modal_start {
        ModalStart::Search => ModalFocus::SearchBar,
        ModalStart::List => ModalFocus::List,
    }
}

pub fn resolved_item_labels_for_list(
    list: &HierarchyList,
    values: &[String],
    repeat_values: &[String],
    lists: &[HierarchyList],
    format_lists: &[HierarchyList],
    lookup: ListValueLookup<'_>,
) -> Vec<String> {
    let flow = FieldModal {
        format: None,
        lists: lists.to_vec(),
        format_lists: format_lists.to_vec(),
        list_idx: values.len(),
        values: values.to_vec(),
        item_ids: Vec::new(),
        repeat_values: repeat_values.to_vec(),
        repeat_item_ids: Vec::new(),
    };
    list.items
        .iter()
        .map(|item| resolve_display_template(item.ui_label(), &flow, lookup))
        .collect()
}

pub fn confirmed_value_assignments(
    value: &HeaderFieldValue,
    cfg: &HeaderFieldConfig,
    sticky_values: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut assigned = HashMap::new();
    collect_assignments_from_value(value.source_value(), cfg, sticky_values, &mut assigned);
    assigned
}

pub fn format_field_value(flow: &FieldModal, lookup: ListValueLookup<'_>) -> String {
    if let Some(format) = &flow.format {
        let mut result = format.clone();
        for (idx, value) in flow.values.iter().enumerate() {
            if let Some(list) = flow.lists.get(idx) {
                result = result.replace(&format!("{{{}}}", list.id), value);
            }
        }
        for list in &flow.format_lists {
            let placeholder = format!("{{{}}}", list.id);
            if !result.contains(&placeholder) {
                continue;
            }
            let value = lookup.fallback_value(list).unwrap_or_default();
            result = result.replace(&placeholder, &value);
        }
        result
    } else {
        if !flow.values.is_empty() {
            flow.values.first().cloned().unwrap_or_default()
        } else if let Some(list) = flow.lists.get(flow.list_idx) {
            joined_repeating_value(list, &flow.repeat_values)
                .unwrap_or_else(|| flow.repeat_values.join(", "))
        } else {
            String::new()
        }
    }
}

fn current_list_fallback_value(modal: &SearchModal, lookup: ListValueLookup<'_>) -> Option<String> {
    modal
        .field_flow
        .lists
        .get(modal.field_flow.list_idx)
        .and_then(|list| lookup.fallback_value(list))
}

fn current_list_fallback_item_id(
    modal: &SearchModal,
    lookup: ListValueLookup<'_>,
) -> Option<String> {
    let list = modal.field_flow.lists.get(modal.field_flow.list_idx)?;
    let value = lookup.fallback_value(list)?;
    item_id_for_output(list, &value)
}

fn resolve_display_template(
    template: &str,
    flow: &FieldModal,
    lookup: ListValueLookup<'_>,
) -> String {
    resolve_display_template_inner(template, flow, lookup, &mut Vec::new())
}

fn resolve_display_template_inner(
    template: &str,
    flow: &FieldModal,
    lookup: ListValueLookup<'_>,
    resolving_lists: &mut Vec<String>,
) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '{' {
            result.push(c);
            continue;
        }

        let mut id = String::new();
        for c2 in chars.by_ref() {
            if c2 == '}' {
                break;
            }
            id.push(c2);
        }

        if id.is_empty() {
            result.push_str("{}");
        } else if let Some(value) = display_template_value(&id, flow, lookup, resolving_lists) {
            result.push_str(&value);
        } else {
            result.push('{');
            result.push_str(&id);
            result.push('}');
        }
    }
    result
}

fn display_template_value(
    id: &str,
    flow: &FieldModal,
    lookup: ListValueLookup<'_>,
    resolving_lists: &mut Vec<String>,
) -> Option<String> {
    if let Some(idx) = flow.lists.iter().position(|list| list.id == id) {
        if let Some(value) = flow.values.get(idx) {
            return Some(value.clone());
        }
        if idx == flow.list_idx && !flow.repeat_values.is_empty() {
            return joined_repeating_value(&flow.lists[idx], &flow.repeat_values)
                .or_else(|| Some(flow.repeat_values.join(", ")));
        }
        return resolved_lookup_value(&flow.lists[idx], flow, lookup, resolving_lists);
    }

    flow.format_lists
        .iter()
        .find(|list| list.id == id)
        .and_then(|list| resolved_lookup_value(list, flow, lookup, resolving_lists))
}

fn resolved_lookup_value(
    list: &HierarchyList,
    flow: &FieldModal,
    lookup: ListValueLookup<'_>,
    resolving_lists: &mut Vec<String>,
) -> Option<String> {
    let raw_value = lookup.raw_fallback_value(list)?;
    if !raw_value.contains('{') {
        return Some(raw_value);
    }
    if resolving_lists.iter().any(|list_id| list_id == &list.id) {
        return Some(raw_value);
    }
    resolving_lists.push(list.id.clone());
    let resolved = resolve_display_template_inner(&raw_value, flow, lookup, resolving_lists);
    resolving_lists.pop();
    Some(resolved)
}

fn list_initial_cursor(
    list: &HierarchyList,
    outputs: &[String],
    lookup: ListValueLookup<'_>,
) -> usize {
    lookup.list_initial_cursor(list, outputs)
}

fn collection_labels(collections: &[ResolvedCollectionConfig]) -> Vec<String> {
    collections
        .iter()
        .map(|collection| collection.label.clone())
        .collect()
}

fn collection_part_label(modal: &SearchModal, _state: &CollectionState) -> String {
    modal.field_name.clone()
}

pub fn authored_collection_preview(
    collection: &crate::sections::collection::CollectionEntry,
) -> (String, Vec<String>) {
    let title = collection
        .list_labels
        .first()
        .cloned()
        .unwrap_or_else(|| collection.label.clone());
    let lines = collection
        .items
        .iter()
        .zip(collection.item_enabled.iter())
        .map(|(item, enabled)| {
            let marker = if *enabled { "[x]" } else { "[ ]" };
            format!("{marker} {}", item.ui_label())
        })
        .collect();
    (title, lines)
}

fn collection_preview_snapshot(
    collection: &crate::sections::collection::CollectionEntry,
    item_cursor: Option<usize>,
) -> CollectionPreviewSnapshot {
    let (title, rows) = authored_collection_preview(collection);
    CollectionPreviewSnapshot {
        title,
        rows,
        item_cursor,
    }
}

pub fn decode_collection_display_value(
    value: &CollectionFieldValue,
    cfg: &HeaderFieldConfig,
) -> Option<String> {
    let mut state =
        CollectionState::new_with_limits(cfg.collections.clone(), false, cfg.max_actives);
    if restore_collection_state(&mut state, value) {
        Some(format_collection_field_value(
            &state.collections,
            cfg.format.is_some(),
        ))
    } else {
        None
    }
}

pub fn active_collection_ids(value: &CollectionFieldValue) -> Vec<String> {
    value
        .collections
        .iter()
        .filter_map(|collection| collection.active.then_some(collection.id.clone()))
        .collect()
}

fn collection_field_value_from_state(state: &CollectionState) -> CollectionFieldValue {
    let focused_collection_id = state
        .collections
        .get(state.collection_cursor)
        .map(|collection| collection.id.clone());
    let focused_item_id = state
        .collections
        .get(state.collection_cursor)
        .and_then(|collection| collection.items.get(state.item_cursor))
        .map(|item| item.id.clone());
    CollectionFieldValue {
        collections: state
            .collections
            .iter()
            .map(|collection| CollectionSelection {
                id: collection.id.clone(),
                active: collection.active,
                enabled_item_ids: collection
                    .items
                    .iter()
                    .zip(collection.item_enabled.iter())
                    .filter_map(|(item, enabled)| enabled.then_some(item.id.clone()))
                    .collect(),
            })
            .collect(),
        activation_order: state
            .activation_order
            .iter()
            .filter_map(|&idx| state.collections.get(idx))
            .map(|collection| collection.id.clone())
            .collect(),
        focused_collection_id,
        focused_item_id,
        items_focused: state.in_items(),
    }
}

fn restore_collection_state(state: &mut CollectionState, value: &CollectionFieldValue) -> bool {
    state.activation_order.clear();
    for saved in &value.collections {
        let Some(collection) = state
            .collections
            .iter_mut()
            .find(|collection| collection.id == saved.id)
        else {
            continue;
        };
        collection.active = saved.active;
        collection.initialized = collection.active || !saved.enabled_item_ids.is_empty();
        for (item, enabled) in collection
            .items
            .iter()
            .zip(collection.item_enabled.iter_mut())
        {
            *enabled = saved
                .enabled_item_ids
                .iter()
                .any(|selected| selected == &item.id);
        }
    }
    for id in &value.activation_order {
        if let Some(idx) = state
            .collections
            .iter()
            .position(|collection| collection.id == *id && collection.active)
        {
            state.activation_order.push(idx);
        }
    }
    for (idx, collection) in state.collections.iter().enumerate() {
        if collection.active && !state.activation_order.contains(&idx) {
            state.activation_order.push(idx);
        }
    }
    if let Some(limit) = state.max_actives.filter(|limit| *limit > 0) {
        while state.activation_order.len() > limit {
            let evicted = state.activation_order.remove(0);
            if let Some(collection) = state.collections.get_mut(evicted) {
                collection.reset();
            }
        }
    }
    if let Some(collection_id) = value.focused_collection_id.as_ref() {
        if let Some(idx) = state
            .collections
            .iter()
            .position(|collection| &collection.id == collection_id)
        {
            state.collection_cursor = idx;
            state.item_collection_cursor = idx;
        }
    }
    if let Some(collection) = state.collections.get(state.collection_cursor) {
        if let Some(item_id) = value.focused_item_id.as_ref() {
            if let Some(idx) = collection.items.iter().position(|item| &item.id == item_id) {
                state.item_cursor = idx;
            } else {
                state.item_cursor = state
                    .item_cursor
                    .min(collection.items.len().saturating_sub(1));
            }
        } else {
            state.item_cursor = state
                .item_cursor
                .min(collection.items.len().saturating_sub(1));
        }
    } else {
        state.collection_cursor = 0;
        state.item_collection_cursor = 0;
        state.item_cursor = 0;
    }
    state.focus = if value.items_focused && !state.collections.is_empty() {
        crate::sections::collection::CollectionFocus::Items(state.collection_cursor)
    } else {
        crate::sections::collection::CollectionFocus::Collections
    };
    true
}

pub fn format_collection_field_value(
    collections: &[crate::sections::collection::CollectionEntry],
    inline: bool,
) -> String {
    let groups = collection_group_values(collections, inline)
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    if inline {
        groups.join("; ")
    } else {
        groups.join("\n\n")
    }
}

fn collection_group_values(
    collections: &[crate::sections::collection::CollectionEntry],
    inline: bool,
) -> Vec<(String, String)> {
    let mut groups = Vec::new();
    for collection in collections {
        if !collection.active {
            continue;
        }
        let values = collection
            .items
            .iter()
            .zip(collection.item_enabled.iter())
            .filter_map(|(item, enabled)| enabled.then_some(item.output().to_string()))
            .collect::<Vec<_>>();
        let values = dedupe_values(&values);
        if values.is_empty() {
            continue;
        }

        let joined = collection
            .joiner_style
            .as_ref()
            .map(|style| join_repeat_values(&values, style))
            .unwrap_or_else(|| {
                if inline {
                    values.join(", ")
                } else {
                    values.join("\n")
                }
            });

        let rendered = if inline {
            format!("{}: {}", collection.label, joined)
        } else {
            let heading = collection
                .note_label
                .clone()
                .unwrap_or_else(|| format!("##### {}", collection.label));
            format!("{heading}\n{joined}")
        };
        groups.push((collection.id.clone(), rendered));
    }
    groups
}

pub fn joined_repeating_value(list: &HierarchyList, values: &[String]) -> Option<String> {
    effective_joiner_style(list).map(|style| join_repeat_values(values, style))
}

fn join_repeat_values(values: &[String], style: &JoinerStyle) -> String {
    let values = dedupe_values(values);
    match style {
        JoinerStyle::CommaAnd => join_with_final(&values, ", ", " and ", ", and "),
        JoinerStyle::CommaAndThe => {
            let values = values
                .iter()
                .map(|value| format!("the {value}"))
                .collect::<Vec<_>>();
            join_with_final(&values, ", ", " and ", ", and ")
        }
        JoinerStyle::CommaOr => join_with_final(&values, ", ", " or ", ", or "),
        JoinerStyle::Comma => values.join(", "),
        JoinerStyle::Semicolon => values.join("; "),
        JoinerStyle::Slash => values.join("/"),
        JoinerStyle::Newline => values.join("\n"),
    }
}

fn effective_joiner_style(list: &HierarchyList) -> Option<&JoinerStyle> {
    static DEFAULT_REPEAT_LIMIT_JOINER: JoinerStyle = JoinerStyle::CommaAnd;
    list.joiner_style
        .as_ref()
        .or(if list.max_entries.is_some() {
            Some(&DEFAULT_REPEAT_LIMIT_JOINER)
        } else {
            None
        })
}

fn dedupe_values(values: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        if value.trim().is_empty() {
            continue;
        }
        if !result.iter().any(|existing| existing == value) {
            result.push(value.clone());
        }
    }
    result
}

fn join_with_final(values: &[String], separator: &str, two: &str, final_separator: &str) -> String {
    match values {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first}{two}{second}"),
        _ => {
            let last = values.last().cloned().unwrap_or_default();
            let head = &values[..values.len() - 1];
            format!("{}{final_separator}{last}", head.join(separator))
        }
    }
}

fn item_output(item: &crate::data::HierarchyItem) -> String {
    item.output().to_string()
}

fn synthetic_list_for_field(field: &HeaderFieldConfig) -> HierarchyList {
    HierarchyList {
        id: field.id.clone(),
        label: Some(field.name.clone()),
        preview: Some(field.name.clone()),
        sticky: false,
        default: None,
        modal_start: ModalStart::Search,
        joiner_style: field.joiner_style.clone(),
        max_entries: field.max_entries,
        items: vec![crate::data::HierarchyItem {
            id: format!("{}_compose", field.id),
            label: Some(field.name.clone()),
            default_enabled: true,
            output: Some(composite_output_format(field)),
            fields: None,
            branch_fields: field.fields.clone(),
            assigns: Vec::new(),
        }],
    }
}

fn synthetic_container_field(field: &HeaderFieldConfig) -> HeaderFieldConfig {
    HeaderFieldConfig {
        id: field.id.clone(),
        name: field.name.clone(),
        format: Some(format!("{{{}}}", field.id)),
        preview: field.preview.clone(),
        fields: Vec::new(),
        lists: vec![synthetic_list_for_field(field)],
        collections: Vec::new(),
        format_lists: field.format_lists.clone(),
        joiner_style: None,
        max_entries: None,
        max_actives: None,
    }
}

fn composite_output_format(field: &HeaderFieldConfig) -> String {
    if let Some(format) = &field.format {
        return format.clone();
    }
    field
        .fields
        .first()
        .map(|child| format!("{{{}}}", child.id))
        .unwrap_or_default()
}

fn modal_query_matches(label: &str, query: &str) -> bool {
    let normalized_label = label.to_lowercase();
    let words: Vec<String> = label
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();

    query
        .split_whitespace()
        .map(str::to_lowercase)
        .all(|term| term_matches_label(&term, &normalized_label, &words))
}

fn term_matches_label(term: &str, normalized_label: &str, words: &[String]) -> bool {
    words.iter().any(|word| word.starts_with(term)) || normalized_label.contains(term)
}

#[cfg(test)]
mod modal_filter_tests {
    use super::*;
    use crate::data::{HeaderFieldConfig, HierarchyItem, HierarchyList, ModalStart};
    use crate::modal_layout::ModalStubKind;

    #[test]
    fn modal_query_matches_terms_out_of_order() {
        assert!(modal_query_matches("Neck Only HNS", "HNS neck"));
    }

    #[test]
    fn modal_query_matches_word_beginnings() {
        assert!(modal_query_matches("Head, Neck, Shoulders", "hea sho"));
    }

    #[test]
    fn modal_query_requires_every_term() {
        assert!(!modal_query_matches("Neck Only HNS", "HNS foot"));
    }

    fn test_field(joiner_style: Option<JoinerStyle>, modal_start: ModalStart) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "field".to_string(),
            name: "Field".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "list".to_string(),
                label: None,
                preview: None,
                sticky: false,
                default: None,
                modal_start,
                joiner_style: joiner_style.clone(),
                max_entries: None,
                items: vec![HierarchyItem {
                    id: "one".to_string(),
                    label: Some("One".to_string()),
                    default_enabled: true,
                    output: None,
                    fields: None,
                    branch_fields: Vec::new(),
                    assigns: Vec::new(),
                }],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style,
            max_entries: None,
            max_actives: None,
        }
    }

    fn repeating_joiner_field_with_downstream_list() -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "observation".to_string(),
            name: "Observation".to_string(),
            format: Some("{muscle}{place}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "muscle".to_string(),
                    label: Some("Muscle".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::Search,
                    joiner_style: Some(JoinerStyle::Comma),
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "trap".to_string(),
                        label: Some("Trapezius Upper".to_string()),
                        default_enabled: true,
                        output: Some("Trapezius (Upper Fibers)".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                        assigns: Vec::new(),
                    }],
                },
                HierarchyList {
                    id: "place".to_string(),
                    label: Some("Place".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "left".to_string(),
                        label: Some("Left".to_string()),
                        default_enabled: true,
                        output: Some("Left ".to_string()),
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
        }
    }

    #[test]
    fn empty_search_enter_does_not_finish_first_repeating_search_start_item() {
        let modal = SearchModal::new_field(
            0,
            test_field(Some(JoinerStyle::Comma), ModalStart::Search),
            None,
            &HashMap::new(),
            &HashMap::new(),
            5,
        );

        assert!(!modal.should_finish_repeating_from_empty_search());
    }

    #[test]
    fn simple_list_peek_helpers_return_adjacent_real_views() {
        let field = HeaderFieldConfig {
            id: "body_region".to_string(),
            name: "Body Region".to_string(),
            format: Some("{side}{region}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "side".to_string(),
                    label: Some("Side".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        HierarchyItem {
                            id: "left".to_string(),
                            label: Some("Left".to_string()),
                            default_enabled: true,
                            output: Some("Left ".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        HierarchyItem {
                            id: "right".to_string(),
                            label: Some("Right".to_string()),
                            default_enabled: true,
                            output: Some("Right ".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
                HierarchyList {
                    id: "region".to_string(),
                    label: Some("Region".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::Search,
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
                            id: "neck".to_string(),
                            label: Some("Neck".to_string()),
                            default_enabled: true,
                            output: Some("Neck".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
                HierarchyList {
                    id: "pressure".to_string(),
                    label: Some("Pressure".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "medium".to_string(),
                        label: Some("Medium".to_string()),
                        default_enabled: true,
                        output: Some("Medium".to_string()),
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
        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &HashMap::new(), &sticky_values, 5);
        let _ = modal.advance_field("Left ".to_string(), &HashMap::new(), &mut sticky_values, 5);

        let prev = modal
            .peek_prev_list_view(&HashMap::new(), &sticky_values)
            .expect("previous list teaser should exist");
        let next = modal
            .peek_next_list_view(&HashMap::new(), &sticky_values)
            .expect("next list teaser should exist");

        assert_eq!(prev.title, "Side");
        assert_eq!(prev.rows, vec!["Left".to_string(), "Right".to_string()]);
        assert_eq!(prev.list_cursor, 0);
        assert_eq!(prev.confirmed_row, Some(0));
        assert_eq!(next.title, "Pressure");
        assert_eq!(next.rows, vec!["Medium".to_string()]);
        assert_eq!(next.list_cursor, 0);
    }

    #[test]
    fn repeating_joiner_preview_includes_real_downstream_list() {
        let sticky_values = HashMap::new();
        let modal = SearchModal::new_field(
            0,
            repeating_joiner_field_with_downstream_list(),
            None,
            &HashMap::new(),
            &sticky_values,
            5,
        );

        let next = modal
            .peek_next_list_view(&HashMap::new(), &sticky_values)
            .expect("repeat-joiner teaser should preview the downstream list");

        assert_eq!(next.title, "Place");
        assert_eq!(next.rows, vec!["Left".to_string()]);
        assert_eq!(next.list_cursor, 0);
    }

    #[test]
    fn terminal_repeating_joiner_preview_remains_confirm_only() {
        let sticky_values = HashMap::new();
        let modal = SearchModal::new_field(
            0,
            test_field(Some(JoinerStyle::Comma), ModalStart::Search),
            None,
            &HashMap::new(),
            &sticky_values,
            5,
        );

        assert!(modal
            .peek_next_list_view(&HashMap::new(), &sticky_values)
            .is_none());
        assert!(modal
            .peek_next_list_views(&HashMap::new(), &sticky_values, 2)
            .is_empty());
    }

    #[test]
    fn repeat_joiner_semantics_stay_nav_right_when_query_hides_simple_teasers() {
        let mut modal = SearchModal::new_field(
            0,
            repeating_joiner_field_with_downstream_list(),
            None,
            &HashMap::new(),
            &HashMap::new(),
            5,
        );
        modal.query = "trap".to_string();
        modal.update_filter();

        let semantics = modal.edge_semantics(&HashMap::new(), &HashMap::new());

        assert_eq!(semantics.left, ModalStubKind::Exit);
        assert_eq!(semantics.right, ModalStubKind::NavRight);
    }

    #[test]
    fn multi_step_list_snapshot_helpers_follow_sequential_flow() {
        let field = HeaderFieldConfig {
            id: "request".to_string(),
            name: "Request".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "side".to_string(),
                    label: Some("Side".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![
                        HierarchyItem {
                            id: "left".to_string(),
                            label: Some("Left".to_string()),
                            default_enabled: true,
                            output: Some("Left ".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                        HierarchyItem {
                            id: "right".to_string(),
                            label: Some("Right".to_string()),
                            default_enabled: true,
                            output: Some("Right ".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
                HierarchyList {
                    id: "region".to_string(),
                    label: Some("Region".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::Search,
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
                            id: "neck".to_string(),
                            label: Some("Neck".to_string()),
                            default_enabled: true,
                            output: Some("Neck".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        },
                    ],
                },
                HierarchyList {
                    id: "pressure".to_string(),
                    label: Some("Pressure".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "medium".to_string(),
                        label: Some("Medium".to_string()),
                        default_enabled: true,
                        output: Some("Medium".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                        assigns: Vec::new(),
                    }],
                },
                HierarchyList {
                    id: "pace".to_string(),
                    label: Some("Pace".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "slow".to_string(),
                        label: Some("Slow".to_string()),
                        default_enabled: true,
                        output: Some("Slow".to_string()),
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
        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &HashMap::new(), &sticky_values, 5);
        let _ = modal.advance_field("Left ".to_string(), &HashMap::new(), &mut sticky_values, 5);
        let _ = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );

        let previous = modal.peek_prev_list_views(&HashMap::new(), &sticky_values, 3);
        let next = modal.peek_next_list_views(&HashMap::new(), &sticky_values, 3);

        assert_eq!(
            previous
                .iter()
                .map(|snapshot| snapshot.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Region", "Side"]
        );
        assert_eq!(
            next.iter()
                .map(|snapshot| snapshot.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Pace"]
        );
        assert_eq!(next[0].rows, vec!["Slow".to_string()]);
    }

    #[test]
    fn simple_list_peek_helpers_fail_closed_for_unsupported_flows() {
        let sticky_values = HashMap::new();

        let mut filtered_modal = SearchModal::new_field(
            0,
            test_field(None, ModalStart::Search),
            None,
            &HashMap::new(),
            &sticky_values,
            5,
        );
        filtered_modal.query = "One".to_string();
        filtered_modal.update_filter();
        assert!(filtered_modal
            .peek_prev_list_view(&HashMap::new(), &sticky_values)
            .is_none());
        assert!(filtered_modal
            .peek_next_list_view(&HashMap::new(), &sticky_values)
            .is_none());

        let nested = SearchModal::new_field(
            0,
            HeaderFieldConfig {
                id: "request".to_string(),
                name: "Request".to_string(),
                format: None,
                preview: None,
                fields: vec![test_field(None, ModalStart::List)],
                lists: Vec::new(),
                collections: Vec::new(),
                format_lists: Vec::new(),
                joiner_style: None,
                max_entries: None,
                max_actives: None,
            },
            None,
            &HashMap::new(),
            &sticky_values,
            5,
        );
        assert!(nested
            .peek_prev_list_view(&HashMap::new(), &sticky_values)
            .is_none());
        assert!(nested
            .peek_next_list_view(&HashMap::new(), &sticky_values)
            .is_none());

        let mut branch_item = HierarchyItem {
            id: "branch".to_string(),
            label: Some("Branch".to_string()),
            default_enabled: true,
            output: Some("{child}".to_string()),
            fields: None,
            branch_fields: vec![test_field(None, ModalStart::List)],
            assigns: Vec::new(),
        };
        branch_item.branch_fields[0].id = "child".to_string();
        let branch_field = HeaderFieldConfig {
            id: "branch_root".to_string(),
            name: "Branch Root".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "branch_root".to_string(),
                label: Some("Branch Root".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
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
        let branch_modal =
            SearchModal::new_field(0, branch_field, None, &HashMap::new(), &sticky_values, 5);
        assert!(branch_modal
            .peek_next_list_view(&HashMap::new(), &sticky_values)
            .is_none());
    }

    #[test]
    fn collection_preview_strip_follows_focused_collection() {
        let sticky_values = HashMap::new();
        let field = HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: vec![
                ResolvedCollectionConfig {
                    id: "neck".to_string(),
                    label: "Neck".to_string(),
                    note_label: None,
                    default_enabled: false,
                    joiner_style: None,
                    lists: vec![HierarchyList {
                        id: "neck_list".to_string(),
                        label: Some("Neck".to_string()),
                        preview: None,
                        sticky: false,
                        default: None,
                        modal_start: ModalStart::List,
                        joiner_style: None,
                        max_entries: None,
                        items: vec![HierarchyItem {
                            id: "upper".to_string(),
                            label: Some("Upper Traps".to_string()),
                            default_enabled: true,
                            output: Some("Upper Traps".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        }],
                    }],
                },
                ResolvedCollectionConfig {
                    id: "back".to_string(),
                    label: "Back".to_string(),
                    note_label: None,
                    default_enabled: false,
                    joiner_style: None,
                    lists: vec![HierarchyList {
                        id: "back_list".to_string(),
                        label: Some("Back".to_string()),
                        preview: None,
                        sticky: false,
                        default: None,
                        modal_start: ModalStart::List,
                        joiner_style: None,
                        max_entries: None,
                        items: vec![HierarchyItem {
                            id: "erectors".to_string(),
                            label: Some("Erectors".to_string()),
                            default_enabled: true,
                            output: Some("Erectors".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        }],
                    }],
                },
                ResolvedCollectionConfig {
                    id: "glutes".to_string(),
                    label: "Glutes".to_string(),
                    note_label: None,
                    default_enabled: false,
                    joiner_style: None,
                    lists: vec![HierarchyList {
                        id: "glutes_list".to_string(),
                        label: Some("Glutes".to_string()),
                        preview: None,
                        sticky: false,
                        default: None,
                        modal_start: ModalStart::List,
                        joiner_style: None,
                        max_entries: None,
                        items: vec![HierarchyItem {
                            id: "med".to_string(),
                            label: Some("Glute Med".to_string()),
                            default_enabled: true,
                            output: Some("Glute Med".to_string()),
                            fields: None,
                            branch_fields: Vec::new(),
                            assigns: Vec::new(),
                        }],
                    }],
                },
            ],
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let mut modal = SearchModal::new_field(0, field, None, &HashMap::new(), &sticky_values, 5);
        let state = modal.collection_state.as_mut().expect("collection modal");
        state.collection_cursor = 1;
        state.enter_collection();
        state.item_cursor = 0;

        let strip = modal
            .collection_preview_strip()
            .expect("collection preview strip should exist");

        assert_eq!(strip.focused_index, 1);
        assert_eq!(
            strip
                .previews
                .iter()
                .map(|snapshot| snapshot.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Neck", "Back", "Glutes"]
        );
        assert_eq!(strip.previews[1].item_cursor, Some(0));
    }

    #[test]
    fn empty_search_enter_finishes_repeating_search_start_list_after_one_item() {
        let mut modal = SearchModal::new_field(
            0,
            test_field(Some(JoinerStyle::Comma), ModalStart::Search),
            None,
            &HashMap::new(),
            &HashMap::new(),
            5,
        );
        modal.field_flow.repeat_values.push("One".to_string());

        assert!(modal.should_finish_repeating_from_empty_search());
    }

    #[test]
    fn empty_search_enter_does_not_finish_non_repeating_search_start_list() {
        let modal = SearchModal::new_field(
            0,
            test_field(None, ModalStart::Search),
            None,
            &HashMap::new(),
            &HashMap::new(),
            5,
        );

        assert!(!modal.should_finish_repeating_from_empty_search());
    }

    #[test]
    fn non_empty_search_enter_does_not_finish_repeating_search_start_list() {
        let mut modal = SearchModal::new_field(
            0,
            test_field(Some(JoinerStyle::Comma), ModalStart::Search),
            None,
            &HashMap::new(),
            &HashMap::new(),
            5,
        );
        modal.query = "one".to_string();
        modal.update_filter();

        assert!(!modal.should_finish_repeating_from_empty_search());
    }

    #[test]
    fn repeating_list_max_entries_auto_finishes_after_cap() {
        let mut field = test_field(Some(JoinerStyle::Comma), ModalStart::List);
        field.lists[0].max_entries = Some(2);
        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &HashMap::new(), &sticky_values, 5);

        let advance =
            modal.advance_field("One".to_string(), &HashMap::new(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::StayOnList));
        assert_eq!(modal.field_flow.repeat_values, vec!["One".to_string()]);

        let advance =
            modal.advance_field("One".to_string(), &HashMap::new(), &mut sticky_values, 5);
        assert!(matches!(
            advance,
            FieldAdvance::Complete(HeaderFieldValue::Text(value)) if value == "One"
        ));
    }

    #[test]
    fn nested_repeating_field_joins_completed_child_fields() {
        let side_field = HeaderFieldConfig {
            id: "side".to_string(),
            name: "Side".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "side".to_string(),
                label: Some("Side".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![HierarchyItem {
                    id: "left".to_string(),
                    label: Some("Left".to_string()),
                    default_enabled: true,
                    output: Some("left ".to_string()),
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
        let body_part_field = HeaderFieldConfig {
            id: "body_part".to_string(),
            name: "Body Part".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "body_part".to_string(),
                label: Some("Body Part".to_string()),
                preview: None,
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
                        output: Some("shoulder".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                        assigns: Vec::new(),
                    },
                    HierarchyItem {
                        id: "head".to_string(),
                        label: Some("Head".to_string()),
                        default_enabled: true,
                        output: Some("head".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
                        assigns: Vec::new(),
                    },
                ],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let requested_region = HeaderFieldConfig {
            id: "requested_region".to_string(),
            name: "Requested Region".to_string(),
            format: Some("{side}{body_part}".to_string()),
            preview: None,
            fields: vec![side_field, body_part_field],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: Some(JoinerStyle::CommaAndThe),
            max_entries: Some(3),
            max_actives: None,
        };

        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(
            0,
            requested_region.clone(),
            None,
            &HashMap::new(),
            &sticky_values,
            5,
        );

        let advance =
            modal.advance_field("left ".to_string(), &HashMap::new(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::NextList));
        let advance = modal.advance_field(
            "shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        assert!(matches!(
            advance,
            FieldAdvance::Complete(HeaderFieldValue::NestedState(_))
        ));
        let FieldAdvance::Complete(HeaderFieldValue::NestedState(state)) = advance else {
            panic!("nested field should complete into nested state");
        };
        let rendered = crate::sections::multi_field::resolve_multifield_value(
            &HeaderFieldValue::NestedState(state),
            &requested_region,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(matches!(
            rendered,
            crate::sections::multi_field::ResolvedMultiFieldValue::Partial(value)
                | crate::sections::multi_field::ResolvedMultiFieldValue::Complete(value)
                if value == "the left shoulder" || value == "left shoulder"
        ));
    }

    #[test]
    fn nested_field_starts_on_first_real_child_instead_of_wrapper() {
        let appointment_type = HierarchyList {
            id: "appointment_type".to_string(),
            label: Some("Appointment Type".to_string()),
            preview: None,
            sticky: false,
            default: None,
            modal_start: ModalStart::Search,
            joiner_style: None,
            max_entries: None,
            items: vec![HierarchyItem {
                id: "treatment".to_string(),
                label: Some("Treatment massage".to_string()),
                default_enabled: true,
                output: Some("Treatment massage".to_string()),
                fields: None,
                branch_fields: Vec::new(),
                assigns: Vec::new(),
            }],
        };
        let appointment_type_field = HeaderFieldConfig {
            id: "appointment_type".to_string(),
            name: "Appointment Type".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![appointment_type],
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
            fields: vec![HeaderFieldConfig {
                id: "single_region".to_string(),
                name: "Requested Region".to_string(),
                format: None,
                preview: None,
                fields: Vec::new(),
                lists: vec![HierarchyList {
                    id: "region".to_string(),
                    label: Some("Region".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::Search,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![HierarchyItem {
                        id: "shoulder".to_string(),
                        label: Some("Shoulder".to_string()),
                        default_enabled: true,
                        output: Some("shoulder".to_string()),
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
            }],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let request = HeaderFieldConfig {
            id: "request".to_string(),
            name: "Request".to_string(),
            format: Some("{appointment_type}{requested_regions}".to_string()),
            preview: None,
            fields: vec![appointment_type_field, requested_regions],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let modal = SearchModal::new_field(0, request, None, &HashMap::new(), &HashMap::new(), 5);

        assert_eq!(modal.field_id, "appointment_type");
        assert_eq!(modal.all_entries, vec!["Treatment massage".to_string()]);
        assert_eq!(
            modal
                .current_part_label(&HashMap::new(), &HashMap::new())
                .as_deref(),
            Some("Appointment Type")
        );
    }

    #[test]
    fn nested_field_resolves_parent_format_lists_after_child_completion() {
        let year = HierarchyList {
            id: "year".to_string(),
            label: Some("Year".to_string()),
            preview: Some("YYYY".to_string()),
            sticky: true,
            default: None,
            modal_start: ModalStart::Search,
            joiner_style: None,
            max_entries: None,
            items: vec![HierarchyItem {
                id: "year_2026".to_string(),
                label: Some("2026".to_string()),
                default_enabled: true,
                output: Some("2026".to_string()),
                fields: None,
                branch_fields: Vec::new(),
                assigns: Vec::new(),
            }],
        };
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
                items: vec![HierarchyItem {
                    id: "treatment".to_string(),
                    label: Some("Treatment massage".to_string()),
                    default_enabled: true,
                    output: Some("Treatment massage".to_string()),
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
        let request = HeaderFieldConfig {
            id: "request".to_string(),
            name: "Request".to_string(),
            format: Some("{year}: {appointment_type}".to_string()),
            preview: None,
            fields: vec![appointment_type],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: vec![year],
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        let mut sticky_values = HashMap::from([("year".to_string(), "2026".to_string())]);
        let mut modal =
            SearchModal::new_field(0, request.clone(), None, &HashMap::new(), &sticky_values, 5);

        let advance = modal.advance_field(
            "Treatment massage".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );

        let FieldAdvance::Complete(HeaderFieldValue::NestedState(state)) = advance else {
            panic!("nested parent field should complete into nested state");
        };
        let rendered = crate::sections::multi_field::resolve_multifield_value(
            &HeaderFieldValue::NestedState(state),
            &request,
            &HashMap::new(),
            &sticky_values,
        );
        assert!(matches!(
            rendered,
            crate::sections::multi_field::ResolvedMultiFieldValue::Complete(value)
                if value == "2026: Treatment massage"
        ));
    }

    fn request_field_with_repeating_regions() -> HeaderFieldConfig {
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
                items: vec![HierarchyItem {
                    id: "treatment".to_string(),
                    label: Some("Treatment massage".to_string()),
                    default_enabled: true,
                    output: Some("Treatment massage".to_string()),
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
        let region_field = HeaderFieldConfig {
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
                default: None,
                modal_start: ModalStart::Search,
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
        let place_field = HeaderFieldConfig {
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
                default: None,
                modal_start: ModalStart::Search,
                joiner_style: None,
                max_entries: None,
                items: vec![HierarchyItem {
                    id: "left".to_string(),
                    label: Some("Left ".to_string()),
                    default_enabled: true,
                    output: Some("Left ".to_string()),
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
        let single_region = HeaderFieldConfig {
            id: "single_region".to_string(),
            name: "Requested Region".to_string(),
            format: Some("{place}{region}".to_string()),
            preview: None,
            fields: vec![region_field, place_field],
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

    fn find_field_by_id(
        fields: &[HeaderFieldConfig],
        target_id: &str,
    ) -> Option<HeaderFieldConfig> {
        for field in fields {
            if field.id == target_id {
                return Some(field.clone());
            }
            if let Some(found) = find_field_by_id(&field.fields, target_id) {
                return Some(found);
            }
        }
        None
    }

    fn real_appointment_requested_field() -> HeaderFieldConfig {
        let data =
            crate::data::AppData::load(crate::data::find_data_dir()).expect("real data loads");
        for section in crate::data::flat_sections_from_template(&data.template) {
            if let Some(fields) = &section.fields {
                if let Some(field) = find_field_by_id(fields, "appointment_requested_field") {
                    return field;
                }
            }
        }
        panic!("appointment_requested_field should exist in real authored data");
    }

    #[test]
    fn nested_repeating_child_reopens_at_next_region_entry() {
        let request = request_field_with_repeating_regions();
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, request.clone(), None, &HashMap::new(), &sticky_values, 5);

        assert_eq!(modal.field_id, "appointment_type");
        let advance = modal.advance_field(
            "Treatment massage".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        assert!(matches!(advance, FieldAdvance::NextList));
        assert_eq!(modal.field_id, "region");

        let advance = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        assert!(matches!(advance, FieldAdvance::NextList));
        assert_eq!(modal.field_id, "place");

        let advance =
            modal.advance_field("Left ".to_string(), &HashMap::new(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::NextList));
        assert_eq!(modal.field_id, "region");

        let saved = modal.preview_field_value(&sticky_values);
        let reopened =
            SearchModal::new_field(0, request, Some(&saved), &HashMap::new(), &sticky_values, 5);
        assert_eq!(reopened.field_id, "region");
    }

    #[test]
    fn nested_semantics_remain_navigational_before_leaf_completion() {
        let request = request_field_with_repeating_regions();
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, request, None, &HashMap::new(), &sticky_values, 5);

        let _ = modal.advance_field(
            "Treatment massage".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        let _ = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );

        let semantics = modal.edge_semantics(&HashMap::new(), &sticky_values);

        assert_eq!(semantics.left, ModalStubKind::NavLeft);
        assert_eq!(semantics.right, ModalStubKind::NavRight);
    }

    #[test]
    fn nested_partial_leaf_progress_survives_dismiss_and_reopen() {
        let request = request_field_with_repeating_regions();
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, request.clone(), None, &HashMap::new(), &sticky_values, 5);

        let _ = modal.advance_field(
            "Treatment massage".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        let _ = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        assert_eq!(modal.field_id, "place");

        let saved = modal.preview_field_value(&sticky_values);
        let reopened = SearchModal::new_field(
            0,
            request.clone(),
            Some(&saved),
            &HashMap::new(),
            &sticky_values,
            5,
        );
        assert_eq!(reopened.field_id, "place");

        let rendered = crate::sections::multi_field::resolve_multifield_value(
            &saved,
            &request,
            &HashMap::new(),
            &sticky_values,
        );
        assert!(matches!(
            rendered,
            crate::sections::multi_field::ResolvedMultiFieldValue::Partial(value)
                if value.contains("Shoulder")
        ));
    }

    #[test]
    fn nested_back_from_place_returns_to_region() {
        let request = request_field_with_repeating_regions();
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, request, None, &HashMap::new(), &sticky_values, 5);

        let _ = modal.advance_field(
            "Treatment massage".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        let _ = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        assert_eq!(modal.field_id, "place");

        assert!(modal.go_back_one_step(&HashMap::new(), &sticky_values, 5));
        assert_eq!(modal.field_id, "region");
    }

    #[test]
    fn empty_nested_repeat_entry_ends_cycle_without_adding_blank_value() {
        let request = request_field_with_repeating_regions();
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, request.clone(), None, &HashMap::new(), &sticky_values, 5);

        let _ = modal.advance_field(
            "Treatment massage".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        let _ = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        let _ = modal.advance_field("Left ".to_string(), &HashMap::new(), &mut sticky_values, 5);

        assert_eq!(modal.field_id, "region");
        let _ = modal.advance_field(String::new(), &HashMap::new(), &mut sticky_values, 5);
        let advance = modal.advance_field(String::new(), &HashMap::new(), &mut sticky_values, 5);

        let FieldAdvance::Complete(HeaderFieldValue::NestedState(state)) = advance else {
            panic!("request should complete after empty nested repeat terminator");
        };
        let rendered = crate::sections::multi_field::resolve_multifield_value(
            &HeaderFieldValue::NestedState(state),
            &request,
            &HashMap::new(),
            &sticky_values,
        );
        let display = rendered.display_value().unwrap_or_default().to_string();
        assert!(display.contains("Treatment massage"));
        assert!(display.contains("Left Shoulder"));
        assert!(!display.contains(", and "));
    }

    #[test]
    fn real_appointment_requested_field_ends_repeat_before_hitting_max_entries() {
        let request = real_appointment_requested_field();
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, request.clone(), None, &HashMap::new(), &sticky_values, 5);

        let _ = modal.advance_field(
            "Relaxation massage, focusing on ".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        assert_eq!(
            modal
                .nested_stack
                .first()
                .map(|frame| frame.state.field_index),
            Some(1),
            "request root should advance to requested_regions after appointment type"
        );
        let _ = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        let _ = modal.advance_field("Left ".to_string(), &HashMap::new(), &mut sticky_values, 5);
        assert_eq!(
            modal
                .nested_stack
                .first()
                .map(|frame| frame.state.field_index),
            Some(1),
            "request root should still be on requested_regions while adding repeats"
        );

        let _ = modal.advance_field(String::new(), &HashMap::new(), &mut sticky_values, 5);
        let advance = modal.advance_field(String::new(), &HashMap::new(), &mut sticky_values, 5);

        assert!(
            matches!(advance, FieldAdvance::Complete(_)),
            "blank nested repeat entry should finish the repeating field immediately, got {advance:?}"
        );
    }

    #[test]
    fn real_appointment_requested_field_reopens_partial_region_at_place() {
        let request = real_appointment_requested_field();
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, request.clone(), None, &HashMap::new(), &sticky_values, 5);

        let _ = modal.advance_field(
            "Relaxation massage, focusing on ".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        let _ = modal.advance_field(
            "Shoulder".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );

        let saved = modal.preview_field_value(&sticky_values);
        let reopened = SearchModal::new_field(
            0,
            request.clone(),
            Some(&saved),
            &HashMap::new(),
            &sticky_values,
            5,
        );

        assert_eq!(
            reopened
                .current_part_label(&HashMap::new(), &sticky_values)
                .as_deref(),
            Some("(direction)")
        );

        let rendered = crate::sections::multi_field::resolve_multifield_value(
            &saved,
            &request,
            &HashMap::new(),
            &sticky_values,
        );
        assert!(matches!(
            rendered,
            crate::sections::multi_field::ResolvedMultiFieldValue::Partial(value)
                if value.contains("Relaxation massage, focusing on ")
                    && value.contains("Shoulder")
                    && !value.contains("regionpreview")
        ));
    }
}

#[cfg(test)]
mod repeat_join_tests {
    use super::*;

    #[test]
    fn comma_and_the_prefixes_each_unique_value() {
        assert_eq!(
            join_repeat_values(
                &[
                    "head".to_string(),
                    "neck".to_string(),
                    "head".to_string(),
                    "shoulders".to_string()
                ],
                &JoinerStyle::CommaAndThe,
            ),
            "the head, the neck, and the shoulders"
        );
    }

    #[test]
    fn repeat_join_ignores_blank_values() {
        assert_eq!(
            join_repeat_values(
                &[
                    "X".to_string(),
                    "".to_string(),
                    "Y".to_string(),
                    " ".to_string()
                ],
                &JoinerStyle::CommaAndThe,
            ),
            "the X and the Y"
        );
    }

    #[test]
    fn semicolon_joins_values_with_semicolon_space() {
        assert_eq!(
            join_repeat_values(
                &["A".to_string(), "B".to_string(), "C".to_string()],
                &JoinerStyle::Semicolon,
            ),
            "A; B; C"
        );
    }
}

#[cfg(test)]
mod assignment_tests {
    use super::*;
    use crate::data::{
        HeaderFieldConfig, HierarchyItem, HierarchyList, ItemAssignment, ModalStart,
    };

    fn item(id: &str, label: &str, output: &str, assigns: Vec<ItemAssignment>) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: Some(output.to_string()),
            fields: None,
            branch_fields: Vec::new(),
            assigns,
        }
    }

    #[test]
    fn selected_item_assigns_format_list_value_for_rendering() {
        let field = HeaderFieldConfig {
            id: "appointment_time".to_string(),
            name: "Appointment Time".to_string(),
            format: Some("{start_hour}{am_pm}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![HierarchyList {
                id: "start_hour".to_string(),
                label: Some("Start Hour".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![
                    item(
                        "hour_9",
                        "9",
                        "9",
                        vec![ItemAssignment {
                            list_id: "am_pm".to_string(),
                            item_id: "am_item".to_string(),
                            output: "AM".to_string(),
                        }],
                    ),
                    item(
                        "hour_12",
                        "12",
                        "12",
                        vec![ItemAssignment {
                            list_id: "am_pm".to_string(),
                            item_id: "pm_item".to_string(),
                            output: "PM".to_string(),
                        }],
                    ),
                ],
            }],
            collections: Vec::new(),
            format_lists: vec![HierarchyList {
                id: "am_pm".to_string(),
                label: Some("AM/PM".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![
                    item("am_item", "AM", "AM", Vec::new()),
                    item("pm_item", "PM", "PM", Vec::new()),
                ],
            }],
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, field.clone(), None, &HashMap::new(), &sticky_values, 5);

        let advance = modal.advance_field("9".to_string(), &HashMap::new(), &mut sticky_values, 5);
        let assigned = modal.assigned_values(&sticky_values);

        assert_eq!(assigned.get("am_pm").map(String::as_str), Some("AM"));

        let FieldAdvance::Complete(value) = advance else {
            panic!("single-list field should complete after selecting a start hour");
        };
        let rendered = crate::sections::multi_field::resolve_multifield_value(
            &value,
            &field,
            &assigned,
            &sticky_values,
        );

        assert!(matches!(
            rendered,
            crate::sections::multi_field::ResolvedMultiFieldValue::Complete(value) if value == "9AM"
        ));
    }

    #[test]
    fn assigned_item_templates_resolve_in_next_modal_list() {
        let field = HeaderFieldConfig {
            id: "duration".to_string(),
            name: "Duration".to_string(),
            format: Some("{duration_label}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "frequency".to_string(),
                    label: Some("Frequency".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item(
                        "freq_2",
                        "2",
                        "2",
                        vec![
                            ItemAssignment {
                                list_id: "pluralizer".to_string(),
                                item_id: "plural".to_string(),
                                output: "s".to_string(),
                            },
                            ItemAssignment {
                                list_id: "duration_label".to_string(),
                                item_id: "week_label".to_string(),
                                output: "week{pluralizer}".to_string(),
                            },
                        ],
                    )],
                },
                HierarchyList {
                    id: "confirm_duration".to_string(),
                    label: Some("Duration Label".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("confirm", "{duration_label}", "done", Vec::new())],
                },
            ],
            collections: Vec::new(),
            format_lists: vec![
                HierarchyList {
                    id: "pluralizer".to_string(),
                    label: Some("Pluralizer".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("plural", "plural", "s", Vec::new())],
                },
                HierarchyList {
                    id: "duration_label".to_string(),
                    label: Some("Duration Label".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item(
                        "week_label",
                        "week{pluralizer}",
                        "week{pluralizer}",
                        Vec::new(),
                    )],
                },
            ],
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &HashMap::new(), &sticky_values, 5);

        let advance = modal.advance_field("2".to_string(), &HashMap::new(), &mut sticky_values, 5);

        assert!(matches!(advance, FieldAdvance::NextList));
        assert_eq!(modal.all_entries, vec!["weeks".to_string()]);
    }

    #[test]
    fn assigned_empty_string_templates_resolve_in_next_modal_list() {
        let field = HeaderFieldConfig {
            id: "duration".to_string(),
            name: "Duration".to_string(),
            format: Some("{duration_label}".to_string()),
            preview: None,
            fields: Vec::new(),
            lists: vec![
                HierarchyList {
                    id: "frequency".to_string(),
                    label: Some("Frequency".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item(
                        "freq_1",
                        "1",
                        "1",
                        vec![
                            ItemAssignment {
                                list_id: "pluralizer".to_string(),
                                item_id: "singular".to_string(),
                                output: String::new(),
                            },
                            ItemAssignment {
                                list_id: "duration_label".to_string(),
                                item_id: "week_label".to_string(),
                                output: "week{pluralizer}".to_string(),
                            },
                        ],
                    )],
                },
                HierarchyList {
                    id: "confirm_duration".to_string(),
                    label: Some("Duration Label".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("confirm", "{duration_label}", "done", Vec::new())],
                },
            ],
            collections: Vec::new(),
            format_lists: vec![
                HierarchyList {
                    id: "pluralizer".to_string(),
                    label: Some("Pluralizer".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item("singular", "singular", "", Vec::new())],
                },
                HierarchyList {
                    id: "duration_label".to_string(),
                    label: Some("Duration Label".to_string()),
                    preview: None,
                    sticky: false,
                    default: None,
                    modal_start: ModalStart::List,
                    joiner_style: None,
                    max_entries: None,
                    items: vec![item(
                        "week_label",
                        "week{pluralizer}",
                        "week{pluralizer}",
                        Vec::new(),
                    )],
                },
            ],
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, field, None, &HashMap::new(), &sticky_values, 5);

        let advance = modal.advance_field("1".to_string(), &HashMap::new(), &mut sticky_values, 5);

        assert!(matches!(advance, FieldAdvance::NextList));
        assert_eq!(modal.all_entries, vec!["week".to_string()]);
    }
}

#[cfg(test)]
mod branch_field_tests {
    use super::*;
    use crate::data::{HeaderFieldConfig, HierarchyItem, HierarchyList, ModalStart};
    use crate::modal_layout::ModalStubKind;

    fn item(id: &str, label: &str, output: Option<&str>) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: output.map(str::to_string),
            fields: None,
            branch_fields: Vec::new(),
            assigns: Vec::new(),
        }
    }

    fn single_list_field(id: &str, list: HierarchyList) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: id.to_string(),
            name: id.to_string(),
            format: Some(format!("{{{}}}", list.id)),
            preview: None,
            fields: Vec::new(),
            lists: vec![list],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        }
    }

    #[test]
    fn branch_item_inside_repeating_list_adds_branch_result_as_repeat_value() {
        let child_list = HierarchyList {
            id: "child_list".to_string(),
            label: None,
            preview: None,
            sticky: false,
            default: None,
            modal_start: ModalStart::List,
            joiner_style: None,
            max_entries: None,
            items: vec![item("t1", "T1-T12", None)],
        };
        let child_field = single_list_field("ests_mm_field", child_list);
        let mut branch_item = item(
            "ests_mm_item",
            "Erector Thoracic MM",
            Some("{ests_mm_field}"),
        );
        branch_item.branch_fields = vec![child_field];
        let parent_list = HierarchyList {
            id: "muscle".to_string(),
            label: None,
            preview: None,
            sticky: false,
            default: None,
            modal_start: ModalStart::Search,
            joiner_style: Some(JoinerStyle::Comma),
            max_entries: None,
            items: vec![branch_item],
        };
        let parent_field = single_list_field("muscle_field", parent_list);
        let mut sticky_values = HashMap::new();
        let mut modal =
            SearchModal::new_field(0, parent_field, None, &HashMap::new(), &sticky_values, 5);

        let advance = modal.advance_field(
            "{ests_mm_field}".to_string(),
            &HashMap::new(),
            &mut sticky_values,
            5,
        );
        assert!(matches!(advance, FieldAdvance::NextList));

        let advance =
            modal.advance_field("T1-T12".to_string(), &HashMap::new(), &mut sticky_values, 5);

        assert!(matches!(advance, FieldAdvance::StayOnList));
        assert_eq!(modal.field_flow.list_idx, 0);
        assert_eq!(modal.field_flow.repeat_values, vec!["T1-T12".to_string()]);
        assert!(modal.branch_stack.is_empty());
    }

    #[test]
    fn branch_semantics_stay_nav_right_when_preview_layout_is_unavailable() {
        let child_list = HierarchyList {
            id: "child_list".to_string(),
            label: None,
            preview: None,
            sticky: false,
            default: None,
            modal_start: ModalStart::List,
            joiner_style: None,
            max_entries: None,
            items: vec![item("t1", "T1-T12", None)],
        };
        let child_field = single_list_field("ests_mm_field", child_list);
        let mut branch_item = item(
            "ests_mm_item",
            "Erector Thoracic MM",
            Some("{ests_mm_field}"),
        );
        branch_item.branch_fields = vec![child_field];
        let parent_list = HierarchyList {
            id: "muscle".to_string(),
            label: None,
            preview: None,
            sticky: false,
            default: None,
            modal_start: ModalStart::Search,
            joiner_style: None,
            max_entries: None,
            items: vec![branch_item],
        };
        let parent_field = single_list_field("muscle_field", parent_list);
        let mut modal =
            SearchModal::new_field(0, parent_field, None, &HashMap::new(), &HashMap::new(), 5);
        modal.query = "erector".to_string();
        modal.update_filter();

        let semantics = modal.edge_semantics(&HashMap::new(), &HashMap::new());

        assert_eq!(semantics.left, ModalStubKind::Exit);
        assert_eq!(semantics.right, ModalStubKind::NavRight);
    }
}

#[cfg(test)]
mod collection_field_tests {
    use super::*;
    use crate::data::{HierarchyItem, HierarchyList, ModalStart, ResolvedCollectionConfig};
    use crate::modal_layout::ModalStubKind;

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

    fn collection(
        id: &str,
        label: &str,
        joiner_style: Option<JoinerStyle>,
    ) -> ResolvedCollectionConfig {
        ResolvedCollectionConfig {
            id: id.to_string(),
            label: label.to_string(),
            note_label: Some(format!("##### {label}")),
            default_enabled: false,
            joiner_style,
            lists: vec![HierarchyList {
                id: format!("{id}_list"),
                label: Some(label.to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![item("one", "One", "Upper traps"), item("two", "Two", "SCM")],
            }],
        }
    }

    #[test]
    fn collection_field_modal_starts_in_collection_mode() {
        let field = HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: vec![collection("neck", "Neck", Some(JoinerStyle::CommaAnd))],
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let modal = SearchModal::new_field(0, field, None, &HashMap::new(), &HashMap::new(), 5);

        assert!(modal.is_collection_mode());
        assert_eq!(modal.all_entries, vec!["Neck".to_string()]);
    }

    #[test]
    fn collection_semantics_switch_from_enter_to_confirm_inside_items() {
        let field = HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: vec![collection("neck", "Neck", Some(JoinerStyle::CommaAnd))],
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let mut modal = SearchModal::new_field(0, field, None, &HashMap::new(), &HashMap::new(), 5);
        let root_semantics = modal.edge_semantics(&HashMap::new(), &HashMap::new());
        assert_eq!(root_semantics.left, ModalStubKind::Exit);
        assert_eq!(root_semantics.right, ModalStubKind::NavRight);

        modal.collection_enter();
        let item_semantics = modal.edge_semantics(&HashMap::new(), &HashMap::new());
        assert_eq!(item_semantics.left, ModalStubKind::NavLeft);
        assert_eq!(item_semantics.right, ModalStubKind::Confirm);
    }

    #[test]
    fn collection_field_value_formats_grouped_inline() {
        let mut state = CollectionState::new(vec![collection(
            "neck",
            "Neck",
            Some(JoinerStyle::CommaAnd),
        )]);
        state.toggle_current_collection();

        let rendered = format_collection_field_value(&state.collections, true);

        assert_eq!(rendered, "Neck: Upper traps and SCM");
    }

    #[test]
    fn collection_field_value_dedupes_duplicate_outputs_without_joiner_style() {
        let mut cfg = collection("legs", "POST_LEG", None);
        cfg.default_enabled = true;
        cfg.lists = vec![
            HierarchyList {
                id: "upper".to_string(),
                label: Some("Upper".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![
                    item("broad_upper", "Broad", "- Broad Compressions"),
                    item("knee_upper", "Knee", "- Ulnar Kneading"),
                ],
            },
            HierarchyList {
                id: "lower".to_string(),
                label: Some("Lower".to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items: vec![
                    item("broad_lower", "Broad", "- Broad Compressions"),
                    item("toe_lower", "Toe", "- Fingertip Kneading"),
                ],
            },
        ];
        let state = CollectionState::new(vec![cfg]);

        let rendered = format_collection_field_value(&state.collections, false);

        assert_eq!(
            rendered,
            "##### POST_LEG\n- Broad Compressions\n- Ulnar Kneading\n- Fingertip Kneading"
        );
    }

    #[test]
    fn explicit_empty_collection_field_reopens_without_default_enabled_collections() {
        let mut cfg = collection("neck", "Neck", Some(JoinerStyle::CommaAnd));
        cfg.default_enabled = true;
        let field = HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: vec![cfg],
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let modal = SearchModal::new_field(
            0,
            field,
            Some(&crate::sections::header::HeaderFieldValue::ExplicitEmpty),
            &HashMap::new(),
            &HashMap::new(),
            5,
        );

        assert!(modal.is_collection_mode());
        assert!(!modal
            .collection_state
            .as_ref()
            .is_some_and(|state| state.collections[0].active));
    }

    #[test]
    fn collection_field_reopens_with_saved_focus_and_item_cursor() {
        let field = HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: vec![
                collection("neck", "Neck", Some(JoinerStyle::CommaAnd)),
                collection("back", "Back", Some(JoinerStyle::CommaAnd)),
            ],
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let mut modal =
            SearchModal::new_field(0, field.clone(), None, &HashMap::new(), &HashMap::new(), 5);
        let state = modal.collection_state.as_mut().expect("collection modal");
        state.collection_cursor = 1;
        let _ = state.toggle_current_collection();
        state.enter_collection();
        state.item_cursor = 1;

        let value = modal.preview_field_value(&HashMap::new());
        let reopened =
            SearchModal::new_field(0, field, Some(&value), &HashMap::new(), &HashMap::new(), 5);
        let state = reopened
            .collection_state
            .as_ref()
            .expect("collection state should restore");

        assert_eq!(state.collection_cursor, 1);
        assert_eq!(state.item_cursor, 1);
        assert!(matches!(
            state.focus,
            crate::sections::collection::CollectionFocus::Items(1)
        ));
        assert!(state.collections[1].active);
    }

    #[test]
    fn authored_collection_preview_uses_default_markers() {
        let mut cfg = collection("neck", "Neck", Some(JoinerStyle::CommaAnd));
        cfg.default_enabled = true;
        cfg.lists[0].items[0].default_enabled = false;
        let state = CollectionState::new(vec![cfg]);

        let (title, lines) = authored_collection_preview(&state.collections[0]);

        assert_eq!(title, "Neck");
        assert_eq!(lines[0], "[ ] One");
        assert_eq!(lines[1], "[x] Two");
    }
}
