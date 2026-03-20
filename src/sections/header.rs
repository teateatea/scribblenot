use chrono::Local;

#[derive(Debug, Clone)]
pub struct HeaderState {
    pub date: String,
    pub start_time: String,
    pub duration: String,
    pub appointment_type: String,
    pub field_index: usize,
    pub edit_buf: String,
    pub completed: bool,
}

impl HeaderState {
    pub fn new() -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        Self {
            date: today,
            start_time: String::new(),
            duration: String::new(),
            appointment_type: String::new(),
            field_index: 0,
            edit_buf: String::new(),
            completed: false,
        }
    }

    pub fn field_label(&self, idx: usize) -> &'static str {
        match idx {
            0 => "Date (YYYY-MM-DD)",
            1 => "Start Time (e.g. 14:00)",
            2 => "Duration (minutes)",
            3 => "Appointment Type",
            _ => "",
        }
    }

    pub fn current_field_value(&self) -> &str {
        match self.field_index {
            0 => &self.date,
            1 => &self.start_time,
            2 => &self.duration,
            3 => &self.appointment_type,
            _ => "",
        }
    }

    pub fn set_current_field(&mut self, value: String) {
        match self.field_index {
            0 => self.date = value,
            1 => self.start_time = value,
            2 => self.duration = value,
            3 => self.appointment_type = value,
            _ => {}
        }
    }

    pub fn confirm_field(&mut self) -> bool {
        // Save current edit buffer to the field
        let val = self.edit_buf.trim().to_string();
        // If empty and not the last field that can skip, don't advance
        if val.is_empty() && self.field_index < 3 {
            return false;
        }
        if !val.is_empty() {
            self.set_current_field(val);
        }
        self.edit_buf.clear();
        self.field_index += 1;
        if self.field_index >= 4 {
            self.completed = true;
            return true;
        }
        // Pre-fill edit buffer with existing value
        self.edit_buf = self.current_field_value().to_string();
        false
    }

    pub fn handle_char(&mut self, c: char) {
        self.edit_buf.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.edit_buf.pop();
    }
}
