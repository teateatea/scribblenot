use crate::data::CompositeConfig;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ModalFocus {
    SearchBar,
    List,
}

#[derive(Debug, Clone)]
pub struct CompositeModal {
    pub config: CompositeConfig,
    pub part_idx: usize,
    pub values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchModal {
    #[allow(dead_code)]
    pub field_idx: usize,
    pub field_id: String,
    pub query: String,
    pub all_entries: Vec<String>,
    pub all_outputs: Vec<String>,
    pub filtered: Vec<usize>,
    pub list_cursor: usize,
    pub list_scroll: usize,
    pub sticky_cursor: usize,
    pub focus: ModalFocus,
    pub composite: Option<CompositeModal>,
    pub window_size: usize,
}

pub enum CompositeAdvance {
    NextPart,
    Complete(String),
}

impl SearchModal {
    pub fn new_simple(
        field_idx: usize,
        field_id: String,
        entries: Vec<String>,
        window_size: usize,
    ) -> Self {
        let n = entries.len();
        Self {
            field_idx,
            field_id,
            query: String::new(),
            all_outputs: entries.clone(),
            all_entries: entries,
            filtered: (0..n).collect(),
            list_cursor: 0,
            list_scroll: 0,
            sticky_cursor: 0,
            focus: ModalFocus::List,
            composite: None,
            window_size,
        }
    }

    pub fn new_composite(
        field_idx: usize,
        field_id: String,
        config: CompositeConfig,
        sticky_values: &HashMap<String, String>,
        window_size: usize,
    ) -> Self {
        let first_part = &config.parts[0];
        let labels: Vec<String> = first_part
            .options
            .iter()
            .map(|o| o.label().to_string())
            .collect();
        let outputs: Vec<String> = first_part
            .options
            .iter()
            .map(|o| o.output().to_string())
            .collect();
        let n = labels.len();
        let list_cursor = if first_part.sticky {
            let key = format!("{}.{}", field_id, first_part.id);
            sticky_values
                .get(&key)
                .and_then(|val| outputs.iter().position(|e| e == val))
                .unwrap_or_else(|| first_part.default_cursor())
        } else {
            first_part.default_cursor()
        };
        let mut modal = Self {
            field_idx,
            field_id,
            query: String::new(),
            all_entries: labels,
            all_outputs: outputs,
            filtered: (0..n).collect(),
            list_cursor,
            list_scroll: 0,
            sticky_cursor: list_cursor,
            focus: ModalFocus::List,
            composite: Some(CompositeModal {
                config,
                part_idx: 0,
                values: Vec::new(),
            }),
            window_size,
        };
        modal.update_filter();
        modal
    }

    pub fn current_part_label(&self) -> Option<&str> {
        self.composite
            .as_ref()
            .map(|c| c.config.parts[c.part_idx].label.as_str())
    }

    #[allow(dead_code)]
    pub fn composite_progress(&self) -> Option<String> {
        let comp = self.composite.as_ref()?;
        if comp.values.is_empty() {
            return None;
        }
        Some(comp.values.join(" / "))
    }

    pub fn update_filter(&mut self) {
        let q = self.query.to_lowercase();
        if q.is_empty() {
            self.filtered = (0..self.all_entries.len()).collect();
            self.list_cursor = self.sticky_cursor;
        } else {
            self.filtered = self
                .all_entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.to_lowercase().contains(&q))
                .map(|(i, _)| i)
                .collect();
            // Keep sticky item selected if it's in the filtered results, else go to first
            if let Some(pos) = self.filtered.iter().position(|&i| i == self.sticky_cursor) {
                self.list_cursor = pos;
            } else {
                self.list_cursor = 0;
            }
        }
        self.center_scroll();
    }

    pub fn update_scroll(&mut self) {
        let w = self.window_size.max(1);
        if self.list_cursor < self.list_scroll {
            self.list_scroll = self.list_cursor;
        } else if self.list_cursor >= self.list_scroll + w {
            self.list_scroll = self.list_cursor + 1 - w;
        }
    }

    pub fn center_scroll(&mut self) {
        let w = self.window_size.max(1);
        self.list_scroll = self.list_cursor.saturating_sub(w / 2);
        let max_scroll = self.filtered.len().saturating_sub(w);
        if self.list_scroll > max_scroll {
            self.list_scroll = max_scroll;
        }
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.filtered
            .get(self.list_cursor)
            .and_then(|&i| self.all_outputs.get(i))
            .map(String::as_str)
    }

    pub fn hint_value(&self, hint_pos: usize) -> Option<&str> {
        self.filtered
            .get(self.list_scroll + hint_pos)
            .and_then(|&i| self.all_outputs.get(i))
            .map(String::as_str)
    }

    pub fn advance_composite(
        &mut self,
        value: String,
        sticky_values: &mut HashMap<String, String>,
    ) -> CompositeAdvance {
        if let Some(ref mut comp) = self.composite {
            let part = &comp.config.parts[comp.part_idx];
            if part.sticky {
                let key = format!("{}.{}", self.field_id, part.id);
                sticky_values.insert(key, value.clone());
            }
            comp.values.push(value);
            comp.part_idx += 1;

            if comp.part_idx >= comp.config.parts.len() {
                let mut result = comp.config.format.clone();
                for (i, part_val) in comp.values.iter().enumerate() {
                    let placeholder = format!("{{{}}}", comp.config.parts[i].id);
                    result = result.replace(&placeholder, part_val);
                }
                return CompositeAdvance::Complete(result);
            }

            let next_part = &comp.config.parts[comp.part_idx];
            let next_labels: Vec<String> = next_part
                .options
                .iter()
                .map(|o| o.label().to_string())
                .collect();
            let next_outputs: Vec<String> = next_part
                .options
                .iter()
                .map(|o| o.output().to_string())
                .collect();
            self.query = String::new();
            self.list_cursor = if next_part.sticky {
                let key = format!("{}.{}", self.field_id, next_part.id);
                sticky_values
                    .get(&key)
                    .and_then(|val| next_outputs.iter().position(|e| e == val))
                    .unwrap_or_else(|| next_part.default_cursor())
            } else {
                next_part.default_cursor()
            };
            self.sticky_cursor = self.list_cursor;
            self.all_entries = next_labels;
            self.all_outputs = next_outputs;
            self.list_scroll = 0;
            self.update_filter();
            self.focus = ModalFocus::List;
            CompositeAdvance::NextPart
        } else {
            CompositeAdvance::Complete(String::new())
        }
    }
}
