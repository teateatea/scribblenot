use crate::data::{
    HeaderFieldConfig, HierarchyList, JoinerStyle, ModalStart, ResolvedCollectionConfig,
};
use crate::sections::collection::CollectionState;
use crate::sections::header::{CollectionFieldValue, CollectionSelection, HeaderFieldValue};
use std::collections::HashMap;

pub const MODAL_HEIGHT_RATIO: f32 = 0.8;
const MODAL_CHROME_HEIGHT: f32 = 80.0;
const MODAL_ROW_HEIGHT: f32 = 28.0;
#[derive(Debug, Clone, PartialEq)]
pub enum ModalFocus {
    SearchBar,
    List,
}

#[derive(Debug, Clone)]
pub struct FieldModal {
    pub format: Option<String>,
    pub lists: Vec<HierarchyList>,
    pub format_lists: Vec<HierarchyList>,
    pub list_idx: usize,
    pub values: Vec<String>,
    pub repeat_values: Vec<String>,
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
    pub field_flow: FieldModal,
    pub collection_state: Option<CollectionState>,
    pub collection_format: Option<String>,
    pub branch_stack: Vec<BranchFrame>,
    pub window_size: usize,
}

pub enum FieldAdvance {
    NextList,
    Complete(HeaderFieldValue),
    StayOnList,
}

impl SearchModal {
    pub fn new_field(
        field_idx: usize,
        field: HeaderFieldConfig,
        current_value: Option<&HeaderFieldValue>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> Self {
        if !field.fields.is_empty() {
            return Self::new_nested_field(field_idx, field, current_value, sticky_values, window_size);
        }
        if !field.collections.is_empty() && field.lists.is_empty() {
            return Self::new_collection_field(field_idx, field, current_value, window_size);
        }
        let first_list = &field.lists[0];
        let outputs: Vec<String> = first_list.items.iter().map(item_output).collect();
        let list_cursor = list_initial_cursor(first_list, &outputs, sticky_values);
        let labels =
            resolved_item_labels_for_list(first_list, &[], &[], &field.lists, sticky_values);
        let n = labels.len();
        let mut modal = Self {
            field_idx,
            field_id: field.id,
            field_name: field.name,
            query: String::new(),
            all_entries: labels,
            all_outputs: outputs,
            filtered: (0..n).collect(),
            list_cursor,
            list_scroll: 0,
            sticky_cursor: list_cursor,
            focus: modal_start_focus(first_list),
            field_flow: FieldModal {
                format: field.format,
                format_lists: field.format_lists,
                lists: field.lists,
                list_idx: 0,
                values: Vec::new(),
                repeat_values: Vec::new(),
            },
            collection_state: None,
            collection_format: None,
            branch_stack: Vec::new(),
            window_size,
        };
        modal.update_filter();
        modal
    }

    fn new_nested_field(
        field_idx: usize,
        field: HeaderFieldConfig,
        current_value: Option<&HeaderFieldValue>,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> Self {
        let synthetic_list = synthetic_list_for_field(&field);
        let synthetic_field = HeaderFieldConfig {
            id: field.id.clone(),
            name: field.name.clone(),
            format: None,
            fields: Vec::new(),
            lists: vec![synthetic_list],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };
        Self::new_field(field_idx, synthetic_field, current_value, sticky_values, window_size)
    }

    fn new_collection_field(
        field_idx: usize,
        field: HeaderFieldConfig,
        current_value: Option<&HeaderFieldValue>,
        window_size: usize,
    ) -> Self {
        let labels = collection_labels(&field.collections);
        let n = labels.len();
        let use_default_activation = !matches!(current_value, Some(HeaderFieldValue::ExplicitEmpty));
        let mut collection_state =
            CollectionState::new_with_limits(field.collections, use_default_activation, field.max_actives);
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
            field_flow: FieldModal {
                format: field.format.clone(),
                format_lists: field.format_lists,
                lists: field.lists,
                list_idx: 0,
                values: Vec::new(),
                repeat_values: Vec::new(),
            },
            collection_state: Some(collection_state),
            collection_format: field.format,
            branch_stack: Vec::new(),
            window_size,
        }
    }

    pub fn is_collection_mode(&self) -> bool {
        self.collection_state.is_some()
    }

    pub fn current_part_label(&self, sticky_values: &HashMap<String, String>) -> Option<String> {
        if let Some(state) = &self.collection_state {
            return Some(collection_part_label(self, state));
        }
        self.field_flow
            .lists
            .get(self.field_flow.list_idx)
            .and_then(|list| list.label.as_deref())
            .map(|label| resolve_display_template(label, &self.field_flow, sticky_values))
    }

    pub fn preview_collection(&self) -> Option<&crate::sections::collection::CollectionEntry> {
        let state = self.collection_state.as_ref()?;
        let idx = match state.focus {
            crate::sections::collection::CollectionFocus::Collections => state.collection_cursor,
            crate::sections::collection::CollectionFocus::Items(collection_idx) => collection_idx,
        };
        state.collections.get(idx)
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
        self.reload_current_list(sticky_values, window_size);
        true
    }

    pub fn advance_field(
        &mut self,
        value: String,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        if self.collection_state.is_some() {
            return FieldAdvance::Complete(self.collection_value());
        }
        let list = &self.field_flow.lists[self.field_flow.list_idx];
        if let Some((output_format, branch_fields)) = branch_for_value(list, &value) {
            return self.start_branch(output_format, branch_fields, sticky_values, window_size);
        }
        if effective_joiner_style(list).is_some() {
            if value.trim().is_empty() {
                return self
                    .finish_repeating_current_list(Some(value), sticky_values, window_size)
                    .unwrap_or(FieldAdvance::StayOnList);
            }
            if list.sticky {
                sticky_values.insert(list.id.clone(), value.clone());
            }
            self.field_flow.repeat_values.push(value);
            if list
                .max_entries
                .is_some_and(|limit| self.field_flow.repeat_values.len() >= limit)
            {
                return self
                    .finish_repeating_current_list(None, sticky_values, window_size)
                    .unwrap_or(FieldAdvance::StayOnList);
            }
            self.all_entries = resolved_item_labels_for_list(
                list,
                &self.field_flow.values,
                &self.field_flow.repeat_values,
                &self.field_flow.lists,
                sticky_values,
            );
            self.query = String::new();
            self.focus = modal_start_focus(list);
            self.update_filter();
            return FieldAdvance::StayOnList;
        }
        let advance = self.finish_current_list(value, sticky_values, window_size);
        self.resolve_branch_advance(advance, sticky_values, window_size)
    }

    pub fn super_confirm_field(
        &mut self,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        if self.collection_state.is_some() {
            return FieldAdvance::Complete(self.collection_value());
        }
        let selected = self.selected_value().map(str::to_string);
        if let Some(advance) =
            self.finish_repeating_current_list(selected, sticky_values, window_size)
        {
            return advance;
        } else {
            let value = self
                .selected_value()
                .map(str::to_string)
                .or_else(|| current_list_fallback_value(self, sticky_values))
                .unwrap_or_default();
            if let Some((output_format, branch_fields)) =
                branch_for_value(&self.field_flow.lists[self.field_flow.list_idx], &value)
            {
                return self.start_branch(output_format, branch_fields, sticky_values, window_size);
            }
            let advance = self.finish_current_list(value, sticky_values, window_size);
            let advance = self.resolve_branch_advance(advance, sticky_values, window_size);
            if matches!(advance, FieldAdvance::Complete(_)) {
                return advance;
            }
        }

        while self.field_flow.list_idx < self.field_flow.lists.len() {
            let Some(value) = current_list_fallback_value(self, sticky_values) else {
                break;
            };
            let advance = self.finish_current_list(value, sticky_values, window_size);
            let advance = self.resolve_branch_advance(advance, sticky_values, window_size);
            if matches!(advance, FieldAdvance::Complete(_)) {
                return advance;
            }
        }

        FieldAdvance::NextList
    }

    fn finish_repeating_current_list(
        &mut self,
        selected: Option<String>,
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
        if let Some(value) = selected {
            values.push(value);
        } else if values.is_empty() {
            if let Some(value) = current_list_fallback_value(self, sticky_values) {
                values.push(value);
            }
        }
        if list.sticky {
            if let Some(value) = values.last() {
                sticky_values.insert(list.id.clone(), value.clone());
            }
        }
        let joined = join_repeat_values(&values, style);
        self.field_flow.repeat_values.clear();
        let advance = self.finish_current_list(joined, sticky_values, window_size);
        Some(self.resolve_branch_advance(advance, sticky_values, window_size))
    }

    fn finish_current_list(
        &mut self,
        value: String,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        let list = &self.field_flow.lists[self.field_flow.list_idx];
        if list.sticky && effective_joiner_style(list).is_none() {
            sticky_values.insert(list.id.clone(), value.clone());
        }
        self.field_flow.values.push(value);
        self.field_flow.list_idx += 1;

        if self.field_flow.list_idx >= self.field_flow.lists.len() {
            return FieldAdvance::Complete(HeaderFieldValue::Text(format_field_value(
                &self.field_flow,
                sticky_values,
            )));
        }

        let next_list = &self.field_flow.lists[self.field_flow.list_idx];
        let next_outputs: Vec<String> = next_list.items.iter().map(item_output).collect();
        let next_labels = resolved_item_labels_for_list(
            next_list,
            &self.field_flow.values,
            &[],
            &self.field_flow.lists,
            sticky_values,
        );
        let next_focus = modal_start_focus(next_list);
        self.query = String::new();
        self.list_cursor = list_initial_cursor(next_list, &next_outputs, sticky_values);
        self.sticky_cursor = self.list_cursor;
        self.window_size = window_size;
        self.all_entries = next_labels;
        self.all_outputs = next_outputs;
        self.list_scroll = 0;
        self.update_filter();
        self.focus = next_focus;
        FieldAdvance::NextList
    }

    fn reload_current_list(&mut self, sticky_values: &HashMap<String, String>, window_size: usize) {
        let Some(list) = self.field_flow.lists.get(self.field_flow.list_idx) else {
            return;
        };
        let next_focus = modal_start_focus(list);
        let outputs: Vec<String> = list.items.iter().map(item_output).collect();
        let labels = resolved_item_labels_for_list(
            list,
            &self.field_flow.values,
            &self.field_flow.repeat_values,
            &self.field_flow.lists,
            sticky_values,
        );
        self.query = String::new();
        self.list_cursor = list_initial_cursor(list, &outputs, sticky_values);
        self.sticky_cursor = self.list_cursor;
        self.window_size = window_size;
        self.all_entries = labels;
        self.all_outputs = outputs;
        self.list_scroll = 0;
        self.update_filter();
        self.focus = next_focus;
    }

    fn start_branch(
        &mut self,
        output_format: String,
        branch_fields: Vec<HeaderFieldConfig>,
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
        self.load_field_flow(first_field, sticky_values, window_size);
        FieldAdvance::NextList
    }

    fn load_field_flow(
        &mut self,
        field: HeaderFieldConfig,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) {
        if !field.fields.is_empty() {
            let synthetic_field = HeaderFieldConfig {
                id: field.id.clone(),
                name: field.name.clone(),
                format: None,
                fields: Vec::new(),
                lists: vec![synthetic_list_for_field(&field)],
                collections: Vec::new(),
                format_lists: Vec::new(),
                joiner_style: None,
                max_entries: None,
                max_actives: None,
            };
            self.load_field_flow(synthetic_field, sticky_values, window_size);
            return;
        }
        if !field.collections.is_empty() && field.lists.is_empty() {
            let labels = collection_labels(&field.collections);
            let n = labels.len();
            self.field_id = field.id;
            self.query = String::new();
            self.all_entries = labels;
            self.all_outputs = Vec::new();
            self.filtered = (0..n).collect();
            self.list_cursor = 0;
            self.list_scroll = 0;
            self.sticky_cursor = 0;
            self.focus = ModalFocus::List;
            self.field_flow = FieldModal {
                format: field.format.clone(),
                format_lists: field.format_lists,
                lists: field.lists,
                list_idx: 0,
                values: Vec::new(),
                repeat_values: Vec::new(),
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
        let list_cursor = list_initial_cursor(first_list, &outputs, sticky_values);
        let labels =
            resolved_item_labels_for_list(first_list, &[], &[], &field.lists, sticky_values);
        let n = labels.len();
        self.field_id = field.id;
        self.query = String::new();
        self.all_entries = labels;
        self.all_outputs = outputs;
        self.filtered = (0..n).collect();
        self.list_cursor = list_cursor;
        self.list_scroll = 0;
        self.sticky_cursor = list_cursor;
        self.focus = modal_start_focus(first_list);
        self.field_flow = FieldModal {
            format: field.format,
            format_lists: field.format_lists,
            lists: field.lists,
            list_idx: 0,
            values: Vec::new(),
            repeat_values: Vec::new(),
        };
        self.collection_state = None;
        self.collection_format = None;
        self.window_size = window_size;
        self.update_filter();
    }

    fn resolve_branch_advance(
        &mut self,
        mut advance: FieldAdvance,
        sticky_values: &mut HashMap<String, String>,
        window_size: usize,
    ) -> FieldAdvance {
        while let FieldAdvance::Complete(value) = advance {
            if self.branch_stack.is_empty() {
                return FieldAdvance::Complete(value);
            }
            advance = self.complete_branch_field(
                value.as_text().unwrap_or_default().to_string(),
                sticky_values,
                window_size,
            );
        }
        advance
    }

    fn complete_branch_field(
        &mut self,
        value: String,
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
            self.load_field_flow(next_field, sticky_values, window_size);
            return FieldAdvance::NextList;
        }

        let frame = self.branch_stack.pop().unwrap();
        let branch_value = format_branch_output(&frame.output_format, &frame.values);
        self.field_flow = frame.parent_flow;
        self.advance_field(branch_value, sticky_values, window_size)
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
                state.toggle_current_item();
                return Vec::new();
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
        if self.collection_state.is_some() {
            return self.collection_value();
        }
        HeaderFieldValue::text(compute_preview_text(self, sticky_values))
    }

    fn collection_value(&self) -> HeaderFieldValue {
        let Some(state) = &self.collection_state else {
            return HeaderFieldValue::ExplicitEmpty;
        };
        HeaderFieldValue::CollectionState(collection_field_value_from_state(state))
    }
}

fn branch_for_value(list: &HierarchyList, value: &str) -> Option<(String, Vec<HeaderFieldConfig>)> {
    list.items
        .iter()
        .find(|item| item_output(item) == value && !item.branch_fields.is_empty())
        .map(|item| {
            (
                item.output().to_string(),
                item.branch_fields.clone(),
            )
        })
}

fn format_branch_output(output_format: &str, values: &[(String, String)]) -> String {
    let mut result = output_format.to_string();
    for (field_id, value) in values {
        result = result.replace(&format!("{{{field_id}}}"), value);
    }
    result
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
    sticky_values: &HashMap<String, String>,
) -> Vec<String> {
    let flow = FieldModal {
        format: None,
        lists: lists.to_vec(),
        format_lists: Vec::new(),
        list_idx: values.len(),
        values: values.to_vec(),
        repeat_values: repeat_values.to_vec(),
    };
    list.items
        .iter()
        .map(|item| resolve_display_template(item.ui_label(), &flow, sticky_values))
        .collect()
}

pub fn format_field_value(flow: &FieldModal, sticky_values: &HashMap<String, String>) -> String {
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
            let value = fallback_list_value(list, sticky_values).unwrap_or_default();
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

fn current_list_fallback_value(
    modal: &SearchModal,
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    modal
        .field_flow
        .lists
        .get(modal.field_flow.list_idx)
        .and_then(|list| fallback_list_value(list, sticky_values))
}

fn resolve_display_template(
    template: &str,
    flow: &FieldModal,
    sticky_values: &HashMap<String, String>,
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
        } else if let Some(value) = display_template_value(&id, flow, sticky_values) {
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
    sticky_values: &HashMap<String, String>,
) -> Option<String> {
    if let Some(idx) = flow.lists.iter().position(|list| list.id == id) {
        if let Some(value) = flow.values.get(idx) {
            return Some(value.clone());
        }
        if idx == flow.list_idx && !flow.repeat_values.is_empty() {
            return joined_repeating_value(&flow.lists[idx], &flow.repeat_values)
                .or_else(|| Some(flow.repeat_values.join(", ")));
        }
        return fallback_list_value(&flow.lists[idx], sticky_values);
    }

    flow.format_lists
        .iter()
        .find(|list| list.id == id)
        .and_then(|list| fallback_list_value(list, sticky_values))
}

fn list_initial_cursor(
    list: &HierarchyList,
    outputs: &[String],
    sticky_values: &HashMap<String, String>,
) -> usize {
    if list.sticky {
        if let Some(value) = sticky_values.get(&list.id) {
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

fn fallback_list_value(
    list: &HierarchyList,
    sticky_values: &HashMap<String, String>,
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

pub fn decode_collection_display_value(
    value: &CollectionFieldValue,
    cfg: &HeaderFieldConfig,
) -> Option<String> {
    let mut state = CollectionState::new_with_limits(cfg.collections.clone(), false, cfg.max_actives);
    if restore_collection_state(&mut state, value) {
        Some(format_collection_field_value(&state.collections, cfg.format.is_some()))
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
        for (item, enabled) in collection.items.iter().zip(collection.item_enabled.iter_mut()) {
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
    true
}

fn compute_preview_text(modal: &SearchModal, sticky_values: &HashMap<String, String>) -> String {
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
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>();
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
    list.joiner_style.as_ref().or(if list.max_entries.is_some() {
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
        }],
    }
}

fn composite_output_format(field: &HeaderFieldConfig) -> String {
    if let Some(format) = &field.format {
        return format.clone();
    }
    field.fields
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

pub fn modal_height_for_viewport(viewport_height: Option<f32>, fallback_height: f32) -> f32 {
    viewport_height
        .map(|height| (height * MODAL_HEIGHT_RATIO).max(160.0))
        .unwrap_or(fallback_height)
}

pub fn modal_window_size_for_height(modal_height: f32, hint_count: usize) -> usize {
    let hint_cap = hint_count.max(1);
    let available_rows = ((modal_height - MODAL_CHROME_HEIGHT) / MODAL_ROW_HEIGHT)
        .floor()
        .max(1.0) as usize;
    available_rows.min(hint_cap)
}

#[cfg(test)]
mod modal_sizing_tests {
    use super::*;

    #[test]
    fn modal_height_uses_eighty_percent_of_viewport_when_known() {
        assert_eq!(modal_height_for_viewport(Some(1000.0), 360.0), 800.0);
    }

    #[test]
    fn modal_height_uses_fallback_before_resize_event() {
        assert_eq!(modal_height_for_viewport(None, 360.0), 360.0);
    }

    #[test]
    fn modal_window_size_is_capped_by_hint_count() {
        assert_eq!(modal_window_size_for_height(1000.0, 12), 12);
    }

    #[test]
    fn modal_window_size_shrinks_for_short_viewports() {
        assert_eq!(modal_window_size_for_height(192.0, 12), 4);
    }
}

#[cfg(test)]
mod modal_filter_tests {
    use super::*;
    use crate::data::{HeaderFieldConfig, HierarchyItem, HierarchyList, ModalStart};

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

    fn test_field(
        joiner_style: Option<JoinerStyle>,
        modal_start: ModalStart,
    ) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: "field".to_string(),
            name: "Field".to_string(),
            format: None,
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
                }],
            }],
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style,
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
            5,
        );

        assert!(!modal.should_finish_repeating_from_empty_search());
    }

    #[test]
    fn empty_search_enter_finishes_repeating_search_start_list_after_one_item() {
        let mut modal = SearchModal::new_field(
            0,
            test_field(Some(JoinerStyle::Comma), ModalStart::Search),
            None,
            &HashMap::new(),
            5,
        );
        modal.field_flow.repeat_values.push("One".to_string());

        assert!(modal.should_finish_repeating_from_empty_search());
    }

    #[test]
    fn empty_search_enter_does_not_finish_non_repeating_search_start_list() {
        let modal =
            SearchModal::new_field(0, test_field(None, ModalStart::Search), None, &HashMap::new(), 5);

        assert!(!modal.should_finish_repeating_from_empty_search());
    }

    #[test]
    fn non_empty_search_enter_does_not_finish_repeating_search_start_list() {
        let mut modal = SearchModal::new_field(
            0,
            test_field(Some(JoinerStyle::Comma), ModalStart::Search),
            None,
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
        let mut modal = SearchModal::new_field(0, field, None, &sticky_values, 5);

        let advance = modal.advance_field("One".to_string(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::StayOnList));
        assert_eq!(modal.field_flow.repeat_values, vec!["One".to_string()]);

        let advance = modal.advance_field("One".to_string(), &mut sticky_values, 5);
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
                    },
                    HierarchyItem {
                        id: "head".to_string(),
                        label: Some("Head".to_string()),
                        default_enabled: true,
                        output: Some("head".to_string()),
                        fields: None,
                        branch_fields: Vec::new(),
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
            fields: vec![side_field, body_part_field],
            lists: Vec::new(),
            collections: Vec::new(),
            format_lists: Vec::new(),
            joiner_style: Some(JoinerStyle::CommaAndThe),
            max_entries: Some(3),
            max_actives: None,
        };

        let mut sticky_values = HashMap::new();
        let mut modal = SearchModal::new_field(0, requested_region, None, &sticky_values, 5);

        let advance = modal.advance_field("{side}{body_part}".to_string(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::NextList));
        let advance = modal.advance_field("left ".to_string(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::NextList));
        let advance = modal.advance_field("shoulder".to_string(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::StayOnList));
        assert_eq!(modal.field_flow.repeat_values, vec!["left shoulder".to_string()]);
        assert!(modal.should_finish_repeating_from_empty_search());

        let advance = modal.advance_field(String::new(), &mut sticky_values, 5);
        assert!(matches!(
            advance,
            FieldAdvance::Complete(HeaderFieldValue::Text(value)) if value == "the left shoulder"
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
mod branch_field_tests {
    use super::*;
    use crate::data::{HeaderFieldConfig, HierarchyItem, HierarchyList, ModalStart};

    fn item(id: &str, label: &str, output: Option<&str>) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: output.map(str::to_string),
            fields: None,
            branch_fields: Vec::new(),
        }
    }

    fn single_list_field(id: &str, list: HierarchyList) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: id.to_string(),
            name: id.to_string(),
            format: Some(format!("{{{}}}", list.id)),
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
        let mut modal = SearchModal::new_field(0, parent_field, None, &sticky_values, 5);

        let advance = modal.advance_field("{ests_mm_field}".to_string(), &mut sticky_values, 5);
        assert!(matches!(advance, FieldAdvance::NextList));

        let advance = modal.advance_field("T1-T12".to_string(), &mut sticky_values, 5);

        assert!(matches!(advance, FieldAdvance::StayOnList));
        assert_eq!(modal.field_flow.list_idx, 0);
        assert_eq!(modal.field_flow.repeat_values, vec!["T1-T12".to_string()]);
        assert!(modal.branch_stack.is_empty());
    }
}

#[cfg(test)]
mod collection_field_tests {
    use super::*;
    use crate::data::{HierarchyItem, HierarchyList, ModalStart, ResolvedCollectionConfig};

    fn item(id: &str, label: &str, output: &str) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled: true,
            output: Some(output.to_string()),
            fields: None,
            branch_fields: Vec::new(),
        }
    }

    fn collection(id: &str, label: &str, joiner_style: Option<JoinerStyle>) -> ResolvedCollectionConfig {
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
                items: vec![
                    item("one", "One", "Upper traps"),
                    item("two", "Two", "SCM"),
                ],
            }],
        }
    }

    #[test]
    fn collection_field_modal_starts_in_collection_mode() {
        let field = HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
            fields: Vec::new(),
            lists: Vec::new(),
            collections: vec![collection("neck", "Neck", Some(JoinerStyle::CommaAnd))],
            format_lists: Vec::new(),
            joiner_style: None,
            max_entries: None,
            max_actives: None,
        };

        let modal = SearchModal::new_field(0, field, None, &HashMap::new(), 5);

        assert!(modal.is_collection_mode());
        assert_eq!(modal.all_entries, vec!["Neck".to_string()]);
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
    fn explicit_empty_collection_field_reopens_without_default_enabled_collections() {
        let mut cfg = collection("neck", "Neck", Some(JoinerStyle::CommaAnd));
        cfg.default_enabled = true;
        let field = HeaderFieldConfig {
            id: "regions".to_string(),
            name: "Regions".to_string(),
            format: None,
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
            5,
        );

        assert!(modal.is_collection_mode());
        assert!(
            !modal
                .collection_state
                .as_ref()
                .is_some_and(|state| state.collections[0].active)
        );
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
