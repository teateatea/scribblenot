use crate::data::{HierarchyItem, JoinerStyle, ResolvedCollectionConfig};

#[derive(Debug, Clone)]
pub struct CollectionEntry {
    pub id: String,
    pub label: String,
    pub note_label: Option<String>,
    pub joiner_style: Option<JoinerStyle>,
    pub list_labels: Vec<String>,
    pub items: Vec<HierarchyItem>,
    pub item_enabled: Vec<bool>,
    pub item_default_enabled: Vec<bool>,
    pub active: bool,
    pub initialized: bool,
}

impl CollectionEntry {
    pub fn from_config(cfg: &ResolvedCollectionConfig) -> Self {
        let items = cfg
            .lists
            .iter()
            .flat_map(|list| list.items.iter().cloned())
            .collect::<Vec<_>>();
        let item_default_enabled = cfg
            .lists
            .iter()
            .flat_map(|list| list.items.iter().map(|item| item.default_enabled()))
            .collect::<Vec<_>>();
        Self {
            id: cfg.id.clone(),
            label: cfg.label.clone(),
            note_label: cfg.note_label.clone(),
            joiner_style: cfg.joiner_style.clone(),
            list_labels: cfg
                .lists
                .iter()
                .map(|list| list.label.clone().unwrap_or_else(|| list.id.clone()))
                .collect(),
            items,
            item_enabled: item_default_enabled.clone(),
            item_default_enabled,
            active: cfg.default_enabled,
            initialized: cfg.default_enabled,
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
    pub max_actives: Option<usize>,
    pub activation_order: Vec<usize>,
    pub collection_cursor: usize,
    pub item_cursor: usize,
    pub focus: CollectionFocus,
    pub skipped: bool,
    pub completed: bool,
}

impl CollectionState {
    pub fn new(collections: Vec<ResolvedCollectionConfig>) -> Self {
        Self::new_with_limits(collections, true, None)
    }

    pub fn new_with_limits(
        collections: Vec<ResolvedCollectionConfig>,
        use_default_activation: bool,
        max_actives: Option<usize>,
    ) -> Self {
        let mut entries: Vec<CollectionEntry> = collections
            .iter()
            .map(|cfg| {
                let mut entry = CollectionEntry::from_config(cfg);
                if !use_default_activation {
                    entry.active = false;
                    entry.initialized = false;
                }
                entry
            })
            .collect();
        let mut activation_order = entries
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| entry.active.then_some(idx))
            .collect::<Vec<_>>();
        if let Some(limit) = max_actives.filter(|limit| *limit > 0) {
            while activation_order.len() > limit {
                let evicted = activation_order.remove(0);
                if let Some(entry) = entries.get_mut(evicted) {
                    entry.reset();
                }
            }
        }
        Self {
            collections: entries,
            max_actives,
            activation_order,
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

    pub fn toggle_current_collection(&mut self) -> Vec<String> {
        if self.collection_cursor >= self.collections.len() {
            return Vec::new();
        }

        let was_active = self
            .collections
            .get(self.collection_cursor)
            .is_some_and(|collection| collection.active);
        if let Some(collection) = self.collections.get_mut(self.collection_cursor) {
            collection.toggle_active();
        }

        if was_active {
            self.activation_order
                .retain(|&idx| idx != self.collection_cursor);
            return Vec::new();
        }

        self.activation_order
            .retain(|&idx| idx != self.collection_cursor);
        self.activation_order.push(self.collection_cursor);

        let mut evicted_ids = Vec::new();
        if let Some(limit) = self.max_actives.filter(|limit| *limit > 0) {
            while self.activation_order.len() > limit {
                let evicted = self.activation_order.remove(0);
                if let Some(collection) = self.collections.get_mut(evicted) {
                    evicted_ids.push(collection.id.clone());
                    collection.reset();
                }
            }
        }
        evicted_ids
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
        self.activation_order.retain(|&idx| idx != collection_idx);
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
    use crate::data::{HierarchyItem, HierarchyList, ModalStart, ResolvedCollectionConfig};

    fn item(id: &str, label: &str, default_enabled: bool) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: Some(label.to_string()),
            default_enabled,
            output: Some(label.to_string()),
            fields: None,
            branch_fields: Vec::new(),
            assigns: Vec::new(),
        }
    }

    fn collection(id: &str, label: &str, items: Vec<HierarchyItem>) -> ResolvedCollectionConfig {
        ResolvedCollectionConfig {
            id: id.to_string(),
            label: label.to_string(),
            note_label: None,
            default_enabled: false,
            joiner_style: None,
            lists: vec![HierarchyList {
                id: id.to_string(),
                label: Some(label.to_string()),
                preview: None,
                sticky: false,
                default: None,
                modal_start: ModalStart::List,
                joiner_style: None,
                max_entries: None,
                items,
            }],
        }
    }

    #[test]
    fn collection_default_enabled_starts_active() {
        let mut cfg = collection(
            "neck",
            "Neck",
            vec![
                item("swedish", "General Swedish Techniques", true),
                item("fascial", "Fascial Work", false),
            ],
        );
        cfg.default_enabled = true;

        let state = CollectionState::new(vec![cfg]);

        assert!(state.collections[0].active);
        assert!(state.collections[0].initialized);
        assert_eq!(state.collections[0].item_enabled, vec![true, false]);
    }

    #[test]
    fn first_activation_seeds_item_defaults() {
        let state = CollectionState::new(vec![collection(
            "neck",
            "Neck",
            vec![
                item("swedish", "General Swedish Techniques", true),
                item("fascial", "Fascial Work", false),
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
                item("swedish", "General Swedish Techniques", true),
                item("fascial", "Fascial Work", false),
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
                item("swedish", "General Swedish Techniques", true),
                item("fascial", "Fascial Work", false),
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

    #[test]
    fn max_actives_one_turns_collections_into_radio_behavior() {
        let mut state = CollectionState::new_with_limits(
            vec![
                collection("neck", "Neck", vec![item("a", "A", true)]),
                collection("back", "Back", vec![item("b", "B", true)]),
            ],
            true,
            Some(1),
        );

        state.toggle_current_collection();
        assert!(state.collections[0].active);

        state.navigate_down();
        state.toggle_current_collection();

        assert!(!state.collections[0].active);
        assert!(state.collections[1].active);
        assert_eq!(state.activation_order, vec![1]);
    }

    #[test]
    fn max_actives_eviction_uses_oldest_active_collection() {
        let mut state = CollectionState::new_with_limits(
            vec![
                collection("neck", "Neck", vec![item("a", "A", true)]),
                collection("back", "Back", vec![item("b", "B", true)]),
                collection("glutes", "Glutes", vec![item("c", "C", true)]),
            ],
            true,
            Some(2),
        );

        state.toggle_current_collection();
        state.navigate_down();
        state.toggle_current_collection();
        state.navigate_down();
        state.toggle_current_collection();

        assert!(!state.collections[0].active);
        assert!(state.collections[1].active);
        assert!(state.collections[2].active);
        assert_eq!(state.activation_order, vec![1, 2]);
    }

    #[test]
    fn default_active_collections_are_seeded_by_row_order() {
        let mut first = collection("first", "First", vec![item("a", "A", true)]);
        first.default_enabled = true;
        let mut second = collection("second", "Second", vec![item("b", "B", true)]);
        second.default_enabled = true;
        let mut third = collection("third", "Third", vec![item("c", "C", true)]);
        third.default_enabled = true;

        let state = CollectionState::new_with_limits(vec![first, second, third], true, Some(2));

        assert!(!state.collections[0].active);
        assert!(state.collections[1].active);
        assert!(state.collections[2].active);
        assert_eq!(state.activation_order, vec![1, 2]);
    }
}
