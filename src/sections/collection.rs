use crate::data::{HierarchyItem, HierarchyList};

#[derive(Debug, Clone)]
pub struct CollectionEntry {
    pub label: String,
    pub items: Vec<HierarchyItem>,
    pub item_enabled: Vec<bool>,
    pub item_default_enabled: Vec<bool>,
    pub active: bool,
    pub initialized: bool,
}

impl CollectionEntry {
    pub fn from_config(cfg: &HierarchyList) -> Self {
        let item_default_enabled = cfg
            .items
            .iter()
            .map(|item| item.default_enabled())
            .collect::<Vec<_>>();
        Self {
            label: cfg.label.clone().unwrap_or_default(),
            items: cfg.items.clone(),
            item_enabled: item_default_enabled.clone(),
            item_default_enabled,
            active: false,
            initialized: false,
        }
    }

    pub fn toggle_active(&mut self) {
        if !self.active && !self.initialized {
            self.item_enabled = self.item_default_enabled.clone();
            self.initialized = true;
        }
        self.active = !self.active;
    }

    pub fn toggle_item(&mut self, idx: usize) {
        if let Some(val) = self.item_enabled.get_mut(idx) {
            *val = !*val;
        }
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.initialized = false;
        self.item_enabled = self.item_default_enabled.clone();
    }

    pub fn enabled_count(&self) -> usize {
        self.item_enabled.iter().filter(|&&enabled| enabled).count()
    }
}

#[derive(Debug, Clone)]
pub enum CollectionFocus {
    Collections,
    Items(usize),
}

#[derive(Debug, Clone)]
pub struct CollectionState {
    pub collections: Vec<CollectionEntry>,
    pub collection_cursor: usize,
    pub item_cursor: usize,
    pub focus: CollectionFocus,
    pub skipped: bool,
    pub completed: bool,
}

impl CollectionState {
    pub fn new(collections: Vec<HierarchyList>) -> Self {
        Self {
            collections: collections.iter().map(CollectionEntry::from_config).collect(),
            collection_cursor: 0,
            item_cursor: 0,
            focus: CollectionFocus::Collections,
            skipped: false,
            completed: false,
        }
    }

    pub fn navigate_up(&mut self) {
        match &self.focus {
            CollectionFocus::Collections => {
                if self.collection_cursor > 0 {
                    self.collection_cursor -= 1;
                }
            }
            CollectionFocus::Items(_) => {
                if self.item_cursor > 0 {
                    self.item_cursor -= 1;
                }
            }
        }
    }

    pub fn navigate_down(&mut self) {
        match &self.focus {
            CollectionFocus::Collections => {
                if !self.collections.is_empty()
                    && self.collection_cursor < self.collections.len().saturating_sub(1)
                {
                    self.collection_cursor += 1;
                }
            }
            CollectionFocus::Items(collection_idx) => {
                let collection_idx = *collection_idx;
                if let Some(collection) = self.collections.get(collection_idx) {
                    if !collection.items.is_empty()
                        && self.item_cursor < collection.items.len().saturating_sub(1)
                    {
                        self.item_cursor += 1;
                    }
                }
            }
        }
    }

    pub fn enter_collection(&mut self) {
        if self.collections.is_empty() {
            return;
        }
        self.focus = CollectionFocus::Items(self.collection_cursor);
        self.item_cursor = 0;
    }

    pub fn exit_items(&mut self) {
        self.focus = CollectionFocus::Collections;
    }

    pub fn toggle_current_collection(&mut self) {
        if let Some(collection) = self.collections.get_mut(self.collection_cursor) {
            collection.toggle_active();
        }
    }

    pub fn toggle_current_item(&mut self) {
        if let CollectionFocus::Items(collection_idx) = self.focus {
            if let Some(collection) = self.collections.get_mut(collection_idx) {
                collection.toggle_item(self.item_cursor);
            }
        }
    }

    pub fn reset_current_collection(&mut self) {
        let collection_idx = match self.focus {
            CollectionFocus::Collections => self.collection_cursor,
            CollectionFocus::Items(collection_idx) => collection_idx,
        };
        if let Some(collection) = self.collections.get_mut(collection_idx) {
            collection.reset();
        }
        self.focus = CollectionFocus::Collections;
        self.item_cursor = 0;
    }

    pub fn in_items(&self) -> bool {
        matches!(self.focus, CollectionFocus::Items(_))
    }

    pub fn has_any_active_collection(&self) -> bool {
        self.collections.iter().any(|collection| collection.active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{HierarchyItem, HierarchyList, ModalStart};

    fn item(id: &str, label: &str, default_enabled: Option<bool>) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: label.to_string(),
            default: None,
            default_enabled,
            output: Some(label.to_string()),
            note: None,
            fields: None,
            branch_fields: Vec::new(),
        }
    }

    fn collection(id: &str, label: &str, items: Vec<HierarchyItem>) -> HierarchyList {
        HierarchyList {
            id: id.to_string(),
            label: Some(label.to_string()),
            preview: None,
            sticky: false,
            default: None,
            modal_start: ModalStart::List,
            repeating: None,
            items,
        }
    }

    #[test]
    fn first_activation_seeds_item_defaults() {
        let state = CollectionState::new(vec![collection(
            "neck",
            "Neck",
            vec![
                item("swedish", "General Swedish Techniques", None),
                item("fascial", "Fascial Work", Some(false)),
            ],
        )]);

        assert!(!state.collections[0].active);
        assert_eq!(state.collections[0].item_enabled, vec![true, false]);
    }

    #[test]
    fn toggling_off_and_on_preserves_item_overrides() {
        let mut state = CollectionState::new(vec![collection(
            "neck",
            "Neck",
            vec![
                item("swedish", "General Swedish Techniques", None),
                item("fascial", "Fascial Work", Some(false)),
            ],
        )]);

        state.toggle_current_collection();
        state.enter_collection();
        state.toggle_current_item();
        state.navigate_down();
        state.toggle_current_item();
        state.exit_items();

        assert_eq!(state.collections[0].item_enabled, vec![false, true]);

        state.toggle_current_collection();
        assert!(!state.collections[0].active);
        state.toggle_current_collection();

        assert!(state.collections[0].active);
        assert_eq!(state.collections[0].item_enabled, vec![false, true]);
    }

    #[test]
    fn reset_current_collection_restores_defaults_and_clears_active() {
        let mut state = CollectionState::new(vec![collection(
            "neck",
            "Neck",
            vec![
                item("swedish", "General Swedish Techniques", None),
                item("fascial", "Fascial Work", Some(false)),
            ],
        )]);

        state.toggle_current_collection();
        state.enter_collection();
        state.toggle_current_item();
        state.navigate_down();
        state.toggle_current_item();
        state.reset_current_collection();

        assert!(!state.collections[0].active);
        assert!(!state.collections[0].initialized);
        assert_eq!(state.collections[0].item_enabled, vec![true, false]);
        assert!(matches!(state.focus, CollectionFocus::Collections));
    }
}
