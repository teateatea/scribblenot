use crate::data::HeaderFieldConfig;

#[derive(Debug, Clone)]
pub struct HeaderState {
    pub field_configs: Vec<HeaderFieldConfig>,
    pub repeated_values: Vec<Vec<String>>,
    pub repeat_counts: Vec<usize>,
    pub field_index: usize,
    pub completed: bool,
    /// Styled spans for composite preview: (text, is_confirmed). Present while a composite is mid-entry.
    pub composite_spans: Option<Vec<(String, bool)>>,
}

impl HeaderState {
    pub fn new(fields: Vec<HeaderFieldConfig>) -> Self {
        let n = fields.len();
        Self {
            field_configs: fields,
            repeated_values: vec![Vec::new(); n],
            repeat_counts: vec![0; n],
            field_index: 0,
            completed: false,
            composite_spans: None,
        }
    }

    pub fn set_current_value(&mut self, value: String) {
        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
            slot.push(value);
        }
    }

    /// Overwrite (or set) the last entry in the current slot's vec for live previews.
    /// Does not add a new confirmed entry.
    pub fn set_preview_value(&mut self, value: String) {
        if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
            if slot.is_empty() {
                slot.push(value);
            } else {
                *slot.last_mut().unwrap() = value;
            }
        }
    }

    /// Advance to the next field. Returns true if all fields are complete.
    pub fn advance(&mut self) -> bool {
        let limit = self
            .field_configs
            .get(self.field_index)
            .and_then(|cfg| cfg.repeat_limit);
        if let Some(lim) = limit {
            let count = self.repeat_counts[self.field_index] + 1;
            self.repeat_counts[self.field_index] = count;
            if count < lim {
                return false; // re-queue: stay at same field_index
            }
        }
        self.field_index += 1;
        if self.field_index >= self.field_configs.len() {
            self.completed = true;
        }
        self.completed
    }

    /// Go back one field. Returns true if went back, false if already at first field.
    pub fn go_back(&mut self) -> bool {
        if self.field_index > 0 {
            self.field_index -= 1;
            self.repeat_counts[self.field_index] = 0;
            if let Some(slot) = self.repeated_values.get_mut(self.field_index) {
                slot.clear();
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod header_state_repeat_limit_tests {
    use super::*;
    use crate::data::HeaderFieldConfig;

    fn make_field(id: &str, repeat_limit: Option<usize>) -> HeaderFieldConfig {
        HeaderFieldConfig {
            id: id.to_string(),
            name: id.to_string(),
            options: vec![],
            composite: None,
            default: None,
            repeat_limit,
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

    // ST49-2-TEST-3: old `values: Vec<String>` field must NOT exist (compilation would fail)
    // We test the new shape: repeated_values is Vec<Vec<String>>, not Vec<String>
    #[test]
    fn repeated_values_inner_type_is_vec_string() {
        let fields = vec![make_field("a", None)];
        let mut state = HeaderState::new(fields);
        // This assignment only compiles if repeated_values[0] is a Vec<String>
        state.repeated_values[0] = vec!["hello".to_string()];
        assert_eq!(state.repeated_values[0][0], "hello");
    }

    // ST49-2-TEST-4: set_current_value appends to current slot's vec, not overwrites
    #[test]
    fn set_current_value_appends_to_slot() {
        let fields = vec![make_field("a", None)];
        let mut state = HeaderState::new(fields);
        state.set_current_value("first".to_string());
        state.set_current_value("second".to_string());
        assert_eq!(
            state.repeated_values[0].len(),
            2,
            "set_current_value should append; slot should have 2 entries"
        );
        assert_eq!(state.repeated_values[0][0], "first");
        assert_eq!(state.repeated_values[0][1], "second");
    }

    // ST49-2-TEST-5: advance() with repeat_limit not yet reached stays at same field_index and increments counter
    #[test]
    fn advance_stays_at_same_field_when_repeat_limit_not_reached() {
        // repeat_limit = 2 means we can collect 2 entries (counter goes 0->1->2, advance at 2)
        let fields = vec![make_field("a", Some(2)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        // first advance: counter is 0, limit is 2; not yet reached -> stay, counter becomes 1
        let done = state.advance();
        assert_eq!(
            state.field_index, 0,
            "field_index should remain 0 when repeat_limit not yet reached"
        );
        assert_eq!(
            state.repeat_counts[0], 1,
            "repeat_count should increment to 1 after first advance"
        );
        assert!(!done, "should not be done when re-queuing");
    }

    // ST49-2-TEST-6: advance() when repeat_limit is reached moves to next field
    #[test]
    fn advance_moves_to_next_field_when_repeat_limit_reached() {
        // repeat_limit = 1: after 1 collection (counter=1), the next advance should proceed
        let fields = vec![make_field("a", Some(1)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        // First advance: counter goes 0->1, limit=1, not yet reached (limit means max repeats, so 1 means collect once)
        // Interpretation: limit=1 means one repeat allowed; counter starts at 0; after first advance counter=1 which equals limit -> move on
        let _done = state.advance(); // counter 0->1, equals limit -> advance
        assert_eq!(
            state.field_index, 1,
            "field_index should advance to 1 when repeat_limit of 1 is reached"
        );
    }

    // ST49-2-TEST-7: advance() with no repeat_limit always advances (normal behavior)
    #[test]
    fn advance_without_repeat_limit_always_advances() {
        let fields = vec![make_field("a", None), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        let done = state.advance();
        assert_eq!(
            state.field_index, 1,
            "without repeat_limit, advance should move field_index to 1"
        );
        assert!(
            !done,
            "should not be done after first advance with 2 fields"
        );
    }

    // ST49-2-TEST-8: advance() on last field with no repeat_limit sets completed = true
    #[test]
    fn advance_on_last_field_sets_completed() {
        let fields = vec![make_field("only", None)];
        let mut state = HeaderState::new(fields);
        let done = state.advance();
        assert!(done, "should return true when all fields complete");
        assert!(state.completed, "completed flag should be set");
    }

    // ST49-2-TEST-9: go_back() clears repeat_count for the current slot (exits repeat loop)
    #[test]
    fn go_back_clears_repeat_count_for_current_slot() {
        let fields = vec![make_field("a", Some(3)), make_field("b", None)];
        let mut state = HeaderState::new(fields);
        // Simulate being at field 1 with some repeat count set for field 0
        state.field_index = 1;
        state.repeat_counts[0] = 2; // pretend we repeated twice
                                    // go_back from field 1 goes to field 0 and should clear its repeat_count
        state.go_back();
        assert_eq!(
            state.field_index, 0,
            "field_index should be 0 after go_back from field 1"
        );
        assert_eq!(
            state.repeat_counts[0], 0,
            "repeat_count for the slot being returned to should be cleared"
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
        // repeat_limit = 2 -> expect 2 repeats (counter goes 0->1 stay, 1->2 advance)
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
