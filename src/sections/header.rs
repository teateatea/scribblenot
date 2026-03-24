use crate::data::HeaderFieldConfig;

#[derive(Debug, Clone)]
pub struct HeaderState {
    pub field_configs: Vec<HeaderFieldConfig>,
    pub values: Vec<String>,
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
            values: vec![String::new(); n],
            field_index: 0,
            completed: false,
            composite_spans: None,
        }
    }

    pub fn current_value(&self) -> &str {
        self.values.get(self.field_index).map(String::as_str).unwrap_or("")
    }

    pub fn set_current_value(&mut self, value: String) {
        if let Some(v) = self.values.get_mut(self.field_index) {
            *v = value;
        }
    }

    /// Advance to the next field. Returns true if all fields are complete.
    pub fn advance(&mut self) -> bool {
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
            true
        } else {
            false
        }
    }

    pub fn get_value(&self, id: &str) -> &str {
        self.field_configs
            .iter()
            .zip(self.values.iter())
            .find(|(cfg, _)| cfg.id == id)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }
}
