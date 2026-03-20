#[derive(Debug, Clone)]
pub struct ChecklistState {
    pub items: Vec<String>,
    pub checked: Vec<bool>,
    pub cursor: usize,
    pub skipped: bool,
    pub completed: bool,
}

impl ChecklistState {
    pub fn new(items: Vec<String>) -> Self {
        let checked = vec![true; items.len()];
        Self {
            items,
            checked,
            cursor: 0,
            skipped: false,
            completed: false,
        }
    }

    pub fn toggle_current(&mut self) {
        if let Some(val) = self.checked.get_mut(self.cursor) {
            *val = !*val;
        }
    }

    pub fn navigate_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if !self.items.is_empty() && self.cursor < self.items.len() - 1 {
            self.cursor += 1;
        }
    }
}
