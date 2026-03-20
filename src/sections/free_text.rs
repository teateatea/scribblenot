use chrono::Local;

#[derive(Debug, Clone)]
pub enum FreeTextMode {
    Browsing,
    Editing,
}

#[derive(Debug, Clone)]
pub struct FreeTextState {
    pub entries: Vec<String>,
    pub cursor: usize,
    pub mode: FreeTextMode,
    pub edit_buf: String,
    pub skipped: bool,
    pub completed: bool,
}

impl FreeTextState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            cursor: 0,
            mode: FreeTextMode::Browsing,
            edit_buf: String::new(),
            skipped: false,
            completed: false,
        }
    }

    pub fn today_prefix() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    pub fn start_new_entry(&mut self) {
        let prefix = Self::today_prefix();
        self.edit_buf = format!("{}: ", prefix);
        self.mode = FreeTextMode::Editing;
    }

    pub fn commit_entry(&mut self) {
        let text = self.edit_buf.trim().to_string();
        if !text.is_empty() {
            self.entries.push(text);
            self.cursor = self.entries.len().saturating_sub(1);
        }
        self.edit_buf.clear();
        self.mode = FreeTextMode::Browsing;
    }

    pub fn cancel_entry(&mut self) {
        self.edit_buf.clear();
        self.mode = FreeTextMode::Browsing;
    }

    pub fn handle_char(&mut self, c: char) {
        self.edit_buf.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.edit_buf.pop();
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

    pub fn is_editing(&self) -> bool {
        matches!(self.mode, FreeTextMode::Editing)
    }
}
