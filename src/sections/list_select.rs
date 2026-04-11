use crate::data::ListEntry;

#[derive(Debug, Clone)]
pub enum ListSelectMode {
    Browsing,
}

#[derive(Debug, Clone)]
pub struct ListSelectState {
    pub entries: Vec<ListEntry>,
    pub cursor: usize,
    pub selected_indices: Vec<usize>,
    pub mode: ListSelectMode,
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
}
