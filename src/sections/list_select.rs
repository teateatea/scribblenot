use crate::data::ListEntry;

#[derive(Debug, Clone)]
pub enum ListSelectMode {
    Browsing,
    AddingLabel,
    AddingOutput,
}

#[derive(Debug, Clone)]
pub struct ListSelectState {
    pub entries: Vec<ListEntry>,
    pub cursor: usize,
    pub selected_indices: Vec<usize>,
    pub mode: ListSelectMode,
    pub add_label_buf: String,
    pub add_output_buf: String,
    pub skipped: bool,
    pub completed: bool,
}

impl ListSelectState {
    pub fn new(entries: Vec<ListEntry>) -> Self {
        Self {
            entries,
            cursor: 0,
            selected_indices: Vec::new(),
            mode: ListSelectMode::Browsing,
            add_label_buf: String::new(),
            add_output_buf: String::new(),
            skipped: false,
            completed: false,
        }
    }

    pub fn toggle_current(&mut self) {
        let idx = self.cursor;
        if let Some(pos) = self.selected_indices.iter().position(|&i| i == idx) {
            self.selected_indices.remove(pos);
        } else {
            self.selected_indices.push(idx);
        }
    }

    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected_indices.contains(&idx)
    }

    pub fn navigate_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if !self.entries.is_empty() && self.cursor < self.entries.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn start_add_label(&mut self) {
        self.add_label_buf.clear();
        self.add_output_buf.clear();
        self.mode = ListSelectMode::AddingLabel;
    }

    pub fn confirm_label(&mut self) {
        self.mode = ListSelectMode::AddingOutput;
    }

    pub fn confirm_output(&mut self) -> Option<ListEntry> {
        let label = self.add_label_buf.trim().to_string();
        let output = self.add_output_buf.trim().to_string();
        self.mode = ListSelectMode::Browsing;
        if !label.is_empty() && !output.is_empty() {
            Some(ListEntry { label, output })
        } else {
            None
        }
    }

    pub fn cancel_add(&mut self) {
        self.add_label_buf.clear();
        self.add_output_buf.clear();
        self.mode = ListSelectMode::Browsing;
    }

    pub fn handle_char(&mut self, c: char) {
        match self.mode {
            ListSelectMode::AddingLabel => self.add_label_buf.push(c),
            ListSelectMode::AddingOutput => self.add_output_buf.push(c),
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.mode {
            ListSelectMode::AddingLabel => { self.add_label_buf.pop(); }
            ListSelectMode::AddingOutput => { self.add_output_buf.pop(); }
            _ => {}
        }
    }

}
