use crate::data::HeaderFieldConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSelection {
    pub id: String,
    pub active: bool,
    pub enabled_item_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionFieldValue {
    pub collections: Vec<CollectionSelection>,
    pub activation_order: Vec<String>,
    pub focused_collection_id: Option<String>,
    pub focused_item_id: Option<String>,
    pub items_focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListFieldValue {
    pub values: Vec<String>,
    pub item_ids: Vec<String>,
    pub list_idx: usize,
    pub repeat_values: Vec<String>,
    pub repeat_item_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderFieldValue {
    ExplicitEmpty,
    Text(String),
    ManualOverride {
        text: String,
        source: Box<HeaderFieldValue>,
    },
    ListState(ListFieldValue),
    CollectionState(CollectionFieldValue),
    NestedState(Box<HeaderState>),
}

impl HeaderFieldValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value.as_str()),
            Self::ManualOverride { text, .. } => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn source_value(&self) -> &HeaderFieldValue {
        match self {
            Self::ManualOverride { source, .. } => source.source_value(),
            _ => self,
        }
    }

    pub fn manual_override_text(&self) -> Option<&str> {
        match self {
            Self::ManualOverride { text, .. } => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn is_manual_override(&self) -> bool {
        matches!(self, Self::ManualOverride { .. })
    }
}

#[derive(Debug, Clone)]
pub struct HeaderState {
    pub field_configs: Vec<HeaderFieldConfig>,
    pub repeated_values: Vec<Vec<HeaderFieldValue>>,
    pub repeat_counts: Vec<usize>,
    pub repeat_visible_counts: Vec<usize>,
    pub field_index: usize,
    pub completed: bool,
    /// Styled spans for composite preview: (text, is_confirmed). Present while a composite is mid-entry.
    pub composite_spans: Option<Vec<(String, bool)>>,
}

impl PartialEq for HeaderState {
    fn eq(&self, other: &Self) -> bool {
        self.repeated_values == other.repeated_values
            && self.repeat_counts == other.repeat_counts
            && self.repeat_visible_counts == other.repeat_visible_counts
            && self.field_index == other.field_index
            && self.completed == other.completed
            && self.composite_spans == other.composite_spans
    }
}

impl Eq for HeaderState {}

impl HeaderState {
    pub fn new(fields: Vec<HeaderFieldConfig>) -> Self {
        let n = fields.len();
        Self {
            field_configs: fields,
            repeated_values: vec![Vec::new(); n],
            repeat_counts: vec![0; n],
            repeat_visible_counts: vec![0; n],
            field_index: 0,
            completed: false,
            composite_spans: None,
        }
    }

    pub fn set_current_value(&mut self, value: HeaderFieldValue) {
        let value_index = self.active_value_index();
        self.reveal_active_value();
        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
            if value_index < slot.len() {
                slot[value_index] = value;
            } else {
                slot.push(value);
            }
        }
    }

    /// Overwrite (or set) the active entry in the current slot's vec for live previews.
    /// Does not add a new confirmed entry.
    pub fn set_preview_value(&mut self, value: HeaderFieldValue) {
        let value_index = self.active_value_index();
        self.reveal_active_value();
        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
            if value_index < slot.len() {
                slot[value_index] = value;
            } else {
                slot.push(value);
            }
        }
    }

    pub fn active_value_index(&self) -> usize {
        let Some(cfg) = self.field_configs.get(self.field_index) else {
            return 0;
        };
        if let Some(limit) = cfg.max_entries {
            let max_index = limit.saturating_sub(1);
            self.repeat_counts
                .get(self.field_index)
                .copied()
                .unwrap_or(0)
                .min(max_index)
        } else {
            0
        }
    }

    pub fn visible_value_count(&self, field_index: usize) -> usize {
        let confirmed_count = self
            .repeated_values
            .get(field_index)
            .map(|slot| slot.len())
            .unwrap_or(0);
        let visible_count = self
            .repeat_visible_counts
            .get(field_index)
            .copied()
            .unwrap_or(0);
        confirmed_count.max(visible_count)
    }

    pub fn visible_row_count_for_field(&self, field_index: usize) -> usize {
        let Some(cfg) = self.field_configs.get(field_index) else {
            return 0;
        };
        if let Some(limit) = cfg.max_entries {
            self.visible_value_count(field_index)
                .max(1)
                .min(limit.max(1))
        } else {
            1
        }
    }

    pub fn visible_row_count(&self) -> usize {
        (0..self.field_configs.len())
            .map(|field_index| self.visible_row_count_for_field(field_index))
            .sum()
    }

    pub fn field_index_for_visible_row(&self, row_index: usize) -> Option<(usize, usize)> {
        let mut current_row = 0;
        for field_index in 0..self.field_configs.len() {
            let row_count = self.visible_row_count_for_field(field_index);
            if row_index < current_row + row_count {
                return Some((field_index, row_index - current_row));
            }
            current_row += row_count;
        }
        None
    }

    fn reveal_active_value(&mut self) {
        let value_index = self.active_value_index();
        if let Some(visible_count) = self.repeat_visible_counts.get_mut(self.field_index) {
            *visible_count = (*visible_count).max(value_index + 1);
        }
    }

    pub fn clear_active_value(&mut self) {
        let value_index = self.active_value_index();
        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
            if value_index < slot.len() {
                slot.remove(value_index);
            }
        }
    }

    pub fn blank_active_value(&mut self) {
        let value_index = self.active_value_index();
        self.reveal_active_value();
        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
            if value_index < slot.len() {
                slot[value_index] = HeaderFieldValue::ExplicitEmpty;
            } else {
                while slot.len() < value_index {
                    slot.push(HeaderFieldValue::Text(String::new()));
                }
                slot.push(HeaderFieldValue::ExplicitEmpty);
            }
        }
    }

    /// Advance to the next field. Returns true if all fields are complete.
    pub fn advance(&mut self) -> bool {
        let limit = self
            .field_configs
            .get(self.field_index)
            .and_then(|cfg| cfg.max_entries);
        if let Some(lim) = limit {
            let count = self.repeat_counts[self.field_index] + 1;
            self.repeat_counts[self.field_index] = count;
            if let Some(visible_count) = self.repeat_visible_counts.get_mut(self.field_index) {
                let next_visible_count = if count < lim { count + 1 } else { count };
                *visible_count = (*visible_count).max(next_visible_count.min(lim));
            }
            if count < lim {
                return false; // re-queue: stay at same field_index
            }
        }
        self.field_index += 1;
        if self.field_index >= self.field_configs.len() {
            self.completed = true;
            if !self.field_configs.is_empty() {
                self.field_index = self.field_configs.len() - 1;
            }
        }
        self.completed
    }

    /// Go back one field. Returns true if went back, false if already at first field.
    pub fn go_back(&mut self) -> bool {
        let can_step_back_within_repeat = self
            .field_configs
            .get(self.field_index)
            .is_some_and(|cfg| cfg.max_entries.is_some())
            && self
                .repeat_counts
                .get(self.field_index)
                .is_some_and(|count| *count > 0);
        if can_step_back_within_repeat {
            self.repeat_counts[self.field_index] -= 1;
            self.completed = false;
            return true;
        }

        if self.field_index > 0 {
            self.field_index -= 1;
            self.repeat_counts[self.field_index] = self
                .field_configs
                .get(self.field_index)
                .and_then(|cfg| cfg.max_entries)
                .map(|limit| {
                    self.repeated_values
                        .get(self.field_index)
                        .map(|slot| slot.len().saturating_sub(1))
                        .unwrap_or(0)
                        .min(limit.saturating_sub(1))
                })
                .unwrap_or(0);
            self.completed = false;
            true
        } else {
            false
        }
    }

    /// Move forward within already revealed repeat rows or to the next field.
    pub fn go_forward(&mut self) -> bool {
        if self.field_configs.is_empty() {
            return false;
        }

        let can_step_forward_within_repeat = self
            .field_configs
            .get(self.field_index)
            .and_then(|cfg| cfg.max_entries)
            .is_some_and(|limit| {
                let active_index = self.active_value_index();
                let visible_count = self.visible_value_count(self.field_index).min(limit);
                active_index + 1 < visible_count
            });
        if can_step_forward_within_repeat {
            self.repeat_counts[self.field_index] += 1;
            self.completed = false;
            return true;
        }

        let last = self.field_configs.len().saturating_sub(1);
        if self.field_index < last {
            self.field_index += 1;
            self.completed = false;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod header_state_max_entries_tests {
    use super::*;
    use crate::data::HeaderFieldConfig;

    fn make_field(id: &str, max_entries: Option<usize>) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: id.to_string(),
            name: id.to_string(),
            format: None,
            preview: None,
            fields: Vec::new(),
            lists: vec![],
            collections: vec![],
            format_lists: vec![],
            joiner_style: None,
            max_entries,
            max_actives: None,
        }
    }

    // ST49-2-TEST-1: new() initializes repeated_values as Vec<Vec<String>> with one empty inner vec per field
    #[test]
    fn new_initializes_repeated_values_shape() {
        let fields = vec![make_field("a", None), make_field("b", Some(3))];
        let state = HeaderState::new(fields);
        // repeated_values must exist with 2 slots, each an empty vec
        assert_eq!(
            state.repeated_values.len(),
            2,
            "repeated_values should have one slot per field"
        );
        assert!(
            state.repeated_values[0].is_empty(),
            "slot 0 should start empty"
        );
        assert!(
            state.repeated_values[1].is_empty(),
            "slot 1 should start empty"
        );
    }

    // ST49-2-TEST-2: new() initializes repeat_counts as Vec<usize> with one zero per field
    #[test]
    fn new_initializes_repeat_counts_to_zero() {
        let fields = vec![make_field("a", None), make_field("b", Some(3))];
        let state = HeaderState::new(fields);
        assert_eq!(
            state.repeat_counts.len(),
            2,
            "repeat_counts should have one entry per field"
        );
        assert_eq!(
            state.repeat_counts[0], 0,
            "repeat_count[0] should start at 0"
        );
        assert_eq!(
            state.repeat_counts[1], 0,
            "repeat_count[1] should start at 0"
        );
    }

    // ST49-2-TEST-4: set_current_value appends to current slot's vec, not overwrites
    #[test]
    fn set_current_value_appends_to_slot() {
        let fields = vec![make_field("a", Some(2)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        state.set_current_value(HeaderFieldValue::Text("first".to_string()));
        state.advance();
        state.set_current_value(HeaderFieldValue::Text("second".to_string()));
        assert_eq!(
            state.repeated_values[0].len(),
            2,
            "set_current_value should append; slot should have 2 entries"
        );
        assert_eq!(
            state.repeated_values[0][0],
            HeaderFieldValue::Text("first".to_string())
        );
        assert_eq!(
            state.repeated_values[0][1],
            HeaderFieldValue::Text("second".to_string())
        );
    }

    // ST49-2-TEST-5: advance() with max_entries not yet reached stays at same field_index and increments counter
    #[test]
    fn advance_stays_at_same_field_when_max_entries_not_reached() {
        // max_entries = 2 means we can collect 2 entries (counter goes 0->1->2, advance at 2)
        let fields = vec![make_field("a", Some(2)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        // first advance: counter is 0, limit is 2; not yet reached -> stay, counter becomes 1
        let done = state.advance();
        assert_eq!(
            state.field_index, 0,
            "field_index should remain 0 when max_entries not yet reached"
        );
        assert_eq!(
            state.repeat_counts[0], 1,
            "repeat_count should increment to 1 after first advance"
        );
        assert!(!done, "should not be done when re-queuing");
    }

    // ST49-2-TEST-6: advance() when max_entries is reached moves to next field
    #[test]
    fn advance_moves_to_next_field_when_max_entries_reached() {
        // max_entries = 1: after 1 collection (counter=1), the next advance should proceed
        let fields = vec![make_field("a", Some(1)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        // First advance: counter goes 0->1, limit=1, not yet reached (limit means max repeats, so 1 means collect once)
        // Interpretation: limit=1 means one repeat allowed; counter starts at 0; after first advance counter=1 which equals limit -> move on
        let _done = state.advance(); // counter 0->1, equals limit -> advance
        assert_eq!(
            state.field_index, 1,
            "field_index should advance to 1 when max_entries of 1 is reached"
        );
    }

    // ST49-2-TEST-7: advance() with no max_entries always advances (normal behavior)
    #[test]
    fn advance_without_max_entries_always_advances() {
        let fields = vec![make_field("a", None), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        let done = state.advance();
        assert_eq!(
            state.field_index, 1,
            "without max_entries, advance should move field_index to 1"
        );
        assert!(
            !done,
            "should not be done after first advance with 2 fields"
        );
    }

    // ST49-2-TEST-8: advance() on last field with no max_entries sets completed = true
    #[test]
    fn advance_on_last_field_sets_completed() {
        let fields = vec![make_field("only", None)];
        let mut state = HeaderState::new(fields);
        let done = state.advance();
        assert!(done, "should return true when all fields complete");
        assert!(state.completed, "completed flag should be set");
    }

    // ST49-2-TEST-9: go_back() returns to the last confirmed repeat slot
    #[test]
    fn go_back_returns_to_last_confirmed_repeat_slot() {
        let fields = vec![make_field("a", Some(3)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        // Simulate being at field 1 with some repeat count set for field 0
        state.repeated_values[0] = vec![
            HeaderFieldValue::Text("first".to_string()),
            HeaderFieldValue::Text("second".to_string()),
        ];
        state.field_index = 1;
        state.repeat_counts[0] = 2; // pretend we repeated twice
                                    // go_back from field 1 goes to field 0 and should select the last repeat value
        state.go_back();
        assert_eq!(
            state.field_index, 0,
            "field_index should be 0 after go_back from field 1"
        );
        assert_eq!(
            state.repeat_counts[0], 1,
            "repeat_count for the slot being returned to should select the last confirmed value"
        );
    }

    #[test]
    fn go_back_preserves_confirmed_values_for_returned_field() {
        let fields = vec![make_field("a", Some(3)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        state.repeated_values[0].push(HeaderFieldValue::Text("keep me".to_string()));
        state.field_index = 1;

        state.go_back();

        assert_eq!(state.field_index, 0);
        assert_eq!(
            state.repeated_values[0],
            vec![HeaderFieldValue::Text("keep me".to_string())]
        );
    }

    // ST49-2-TEST-10: go_back() at field 0 returns false and does not change state
    #[test]
    fn go_back_at_first_field_returns_false() {
        let fields = vec![make_field("a", None)];
        let mut state = HeaderState::new(fields);
        let result = state.go_back();
        assert!(!result, "go_back at first field should return false");
        assert_eq!(state.field_index, 0, "field_index should remain 0");
    }

    // ST49-2-TEST-11: multiple advance calls cycle through repeat slots correctly
    #[test]
    fn advance_cycles_through_repeat_slots_with_limit_2() {
        // max_entries = 2 -> expect 2 repeats (counter goes 0->1 stay, 1->2 advance)
        let fields = vec![make_field("a", Some(2)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        // advance 1: counter 0->1, limit=2, not reached -> stay at 0
        state.advance();
        assert_eq!(state.field_index, 0);
        assert_eq!(state.repeat_counts[0], 1);
        // advance 2: counter 1->2, limit=2, reached -> move to field 1
        state.advance();
        assert_eq!(
            state.field_index, 1,
            "after 2 repeats, should advance to field 1"
        );
        assert_eq!(state.repeat_counts[0], 2);
    }
}
