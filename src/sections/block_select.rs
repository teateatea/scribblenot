use crate::data::{HierarchyItem, HierarchyList};

#[derive(Debug, Clone)]
pub struct BlockSelectGroup {
    pub label: String,
    pub header: String,
    pub entries: Vec<HierarchyItem>,
    pub item_selected: Vec<bool>,
}

impl BlockSelectGroup {
    pub fn from_config(cfg: &HierarchyList) -> Self {
        let item_selected = cfg
            .items
            .iter()
            .map(|e| e.default.unwrap_or(false))
            .collect();
        Self {
            label: cfg.label.clone().unwrap_or_default(),
            header: cfg.label.clone().unwrap_or_default(),
            entries: cfg.items.iter().cloned().collect(),
            item_selected,
        }
    }

    pub fn has_selection(&self) -> bool {
        self.item_selected.iter().any(|&s| s)
    }

    pub fn toggle_item(&mut self, idx: usize) {
        if let Some(val) = self.item_selected.get_mut(idx) {
            *val = !*val;
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockSelectFocus {
    Groups,
    Items(usize),
}

#[derive(Debug, Clone)]
pub struct BlockSelectState {
    pub groups: Vec<BlockSelectGroup>,
    pub group_cursor: usize,
    pub item_cursor: usize,
    pub focus: BlockSelectFocus,
    pub skipped: bool,
    pub completed: bool,
}

impl BlockSelectState {
    pub fn new(regions: Vec<HierarchyList>) -> Self {
        let region_states = regions.iter().map(BlockSelectGroup::from_config).collect();
        Self {
            groups: region_states,
            group_cursor: 0,
            item_cursor: 0,
            focus: BlockSelectFocus::Groups,
            skipped: false,
            completed: false,
        }
    }

    pub fn navigate_up(&mut self) {
        match &self.focus {
            BlockSelectFocus::Groups => {
                if self.group_cursor > 0 {
                    self.group_cursor -= 1;
                }
            }
            BlockSelectFocus::Items(_) => {
                if self.item_cursor > 0 {
                    self.item_cursor -= 1;
                }
            }
        }
    }

    pub fn navigate_down(&mut self) {
        match &self.focus {
            BlockSelectFocus::Groups => {
                if !self.groups.is_empty() && self.group_cursor < self.groups.len() - 1 {
                    self.group_cursor += 1;
                }
            }
            BlockSelectFocus::Items(region_idx) => {
                let region_idx = *region_idx;
                if let Some(region) = self.groups.get(region_idx) {
                    if !region.entries.is_empty() && self.item_cursor < region.entries.len() - 1 {
                        self.item_cursor += 1;
                    }
                }
            }
        }
    }

    pub fn enter_group(&mut self) {
        let idx = self.group_cursor;
        self.focus = BlockSelectFocus::Items(idx);
        self.item_cursor = 0;
    }

    pub fn exit_items(&mut self) {
        self.focus = BlockSelectFocus::Groups;
    }

    pub fn toggle_item(&mut self) {
        if let BlockSelectFocus::Items(region_idx) = self.focus {
            if let Some(region) = self.groups.get_mut(region_idx) {
                region.toggle_item(self.item_cursor);
            }
        }
    }

    pub fn in_items(&self) -> bool {
        matches!(self.focus, BlockSelectFocus::Items(_))
    }

    pub fn current_group_idx(&self) -> Option<usize> {
        match self.focus {
            BlockSelectFocus::Items(i) => Some(i),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests_st3_default_selected {
    use super::*;
    use crate::data::{HierarchyItem, HierarchyList, PartOption};

    fn make_hier_item(id: &str, label: &str, default: Option<bool>) -> HierarchyItem {
        HierarchyItem {
            id: id.to_string(),
            label: label.to_string(),
            default,
            default_enabled: default,
            output: Some(label.to_lowercase()),
            note: None,
            fields: None,
            branch_fields: Vec::new(),
        }
    }

    fn make_hier_list(id: &str, label: &str, items: Vec<HierarchyItem>) -> HierarchyList {
        HierarchyList {
            id: id.to_string(),
            label: Some(label.to_string()),
            preview: None,
            sticky: false,
            default: None,
            modal_start: crate::data::ModalStart::List,
            repeating: None,
            items,
        }
    }

    // ST3-TEST-1: PartOption must expose a default_selected() -> bool method.
    #[test]
    fn part_option_default_selected_full_true() {
        let opt = PartOption::Full {
            id: "a".to_string(),
            label: "Alpha".to_string(),
            output: "alpha".to_string(),
            default: true,
        };
        assert!(
            opt.default_selected(),
            "Full with default=true should return true from default_selected()"
        );
    }

    #[test]
    fn part_option_default_selected_full_false() {
        let opt = PartOption::Full {
            id: "b".to_string(),
            label: "Beta".to_string(),
            output: "beta".to_string(),
            default: false,
        };
        assert!(
            !opt.default_selected(),
            "Full with default=false should return false from default_selected()"
        );
    }

    #[test]
    fn part_option_default_selected_simple() {
        let opt = PartOption::Simple("gamma".to_string());
        assert!(
            opt.default_selected(),
            "Simple variant should always return true from default_selected()"
        );
    }

    // ST3-TEST-2: BlockSelectGroup::from_config where all entries have default=true
    // should initialize item_selected to all true.
    #[test]
    fn region_state_all_default_true_starts_all_selected() {
        let list = make_hier_list(
            "region_a",
            "Region A",
            vec![
                make_hier_item("opt1", "Option 1", Some(true)),
                make_hier_item("opt2", "Option 2", Some(true)),
                make_hier_item("opt3", "Option 3", Some(true)),
            ],
        );
        let state = BlockSelectGroup::from_config(&list);
        assert_eq!(state.item_selected.len(), 3);
        assert!(
            state.item_selected[0],
            "entry 0 with default=true should start selected"
        );
        assert!(
            state.item_selected[1],
            "entry 1 with default=true should start selected"
        );
        assert!(
            state.item_selected[2],
            "entry 2 with default=true should start selected"
        );
    }

    #[test]
    fn region_state_omitted_defaults_start_unselected() {
        let list = make_hier_list(
            "region_c",
            "Region C",
            vec![
                make_hier_item("opt1", "Option 1", None),
                make_hier_item("opt2", "Option 2", None),
            ],
        );
        let state = BlockSelectGroup::from_config(&list);
        assert_eq!(state.item_selected, vec![false, false]);
    }

    // ST3-TEST-3: BlockSelectGroup::from_config where one entry has default=false
    // should initialize that entry's slot as false, others as true.
    #[test]
    fn region_state_one_default_false_starts_unselected() {
        let list = make_hier_list(
            "region_b",
            "Region B",
            vec![
                make_hier_item("opt1", "Option 1", Some(true)),
                make_hier_item("opt2", "Option 2", Some(false)),
                make_hier_item("opt3", "Option 3", Some(true)),
            ],
        );
        let state = BlockSelectGroup::from_config(&list);
        assert_eq!(state.item_selected.len(), 3);
        assert!(
            state.item_selected[0],
            "entry 0 with default=true should start selected"
        );
        assert!(
            !state.item_selected[1],
            "entry 1 with default=false should start unselected"
        );
        assert!(
            state.item_selected[2],
            "entry 2 with default=true should start selected"
        );
    }

    // ST3-TEST-4: BlockSelectState::new propagates default selection through from_config.
    #[test]
    fn block_select_state_new_propagates_defaults() {
        let regions = vec![make_hier_list(
            "r1",
            "R1",
            vec![
                make_hier_item("a", "A", Some(true)),
                make_hier_item("b", "B", Some(false)),
            ],
        )];
        let state = BlockSelectState::new(regions);
        assert!(
            state.groups[0].item_selected[0],
            "A (default=true) should start selected"
        );
        assert!(
            !state.groups[0].item_selected[1],
            "B (default=false) should start unselected"
        );
    }
}

#[cfg(test)]
mod tests_t46_st1_rename {
    use super::*;
    use crate::data::{HierarchyItem, HierarchyList};

    fn make_entry(label: &str, opts: Vec<&str>) -> HierarchyList {
        HierarchyList {
            id: label.to_lowercase().replace(' ', "_"),
            label: Some(label.to_string()),
            preview: None,
            sticky: false,
            default: None,
            modal_start: crate::data::ModalStart::List,
            repeating: None,
            items: opts
                .iter()
                .map(|s| HierarchyItem {
                    id: s.to_lowercase(),
                    label: s.to_string(),
                    default: None,
                    default_enabled: None,
                    output: Some(s.to_string()),
                    note: None,
                    fields: None,
                    branch_fields: Vec::new(),
                })
                .collect(),
        }
    }

    // T46-ST1-TEST-1: BlockSelectGroup must exist as the renamed struct (was RegionState).
    #[test]
    fn block_select_group_struct_exists() {
        let entry = make_entry("Arm", vec!["Alpha", "Beta"]);
        let group: BlockSelectGroup = BlockSelectGroup::from_config(&entry);
        assert_eq!(group.label, "Arm");
    }

    // T46-ST1-TEST-2: BlockSelectGroup must have an `item_selected` field (was technique_selected).
    #[test]
    fn block_select_group_has_item_selected_field() {
        let entry = make_entry("Leg", vec!["X", "Y", "Z"]);
        let group = BlockSelectGroup::from_config(&entry);
        assert_eq!(group.item_selected.len(), 3);
    }

    // T46-ST1-TEST-3: BlockSelectGroup must have a `toggle_item` method (was toggle_technique).
    #[test]
    fn block_select_group_toggle_item() {
        let entry = make_entry("Torso", vec!["P", "Q"]);
        let mut group = BlockSelectGroup::from_config(&entry);
        let before = group.item_selected[0];
        group.toggle_item(0);
        assert_ne!(group.item_selected[0], before);
    }

    // T46-ST1-TEST-4: BlockSelectFocus::Groups variant must exist (was BlockSelectFocus::Regions).
    #[test]
    fn block_select_focus_groups_variant() {
        let focus = BlockSelectFocus::Groups;
        assert!(matches!(focus, BlockSelectFocus::Groups));
    }

    // T46-ST1-TEST-5: BlockSelectFocus::Items variant must exist (was BlockSelectFocus::Techniques).
    #[test]
    fn block_select_focus_items_variant() {
        let focus = BlockSelectFocus::Items(0);
        assert!(matches!(focus, BlockSelectFocus::Items(0)));
    }

    // T46-ST1-TEST-6: BlockSelectState must have a `groups` field (was regions).
    #[test]
    fn block_select_state_has_groups_field() {
        let state = BlockSelectState::new(vec![make_entry("Head", vec!["A"])]);
        assert_eq!(state.groups.len(), 1);
    }

    // T46-ST1-TEST-7: BlockSelectState must have a `group_cursor` field (was region_cursor).
    #[test]
    fn block_select_state_has_group_cursor_field() {
        let state = BlockSelectState::new(vec![make_entry("Neck", vec!["B"])]);
        assert_eq!(state.group_cursor, 0);
    }

    // T46-ST1-TEST-8: BlockSelectState must have an `item_cursor` field (was technique_cursor).
    #[test]
    fn block_select_state_has_item_cursor_field() {
        let state = BlockSelectState::new(vec![make_entry("Shoulder", vec!["C"])]);
        assert_eq!(state.item_cursor, 0);
    }

    // T46-ST1-TEST-9: BlockSelectState must have an `enter_group` method (was enter_region).
    #[test]
    fn block_select_state_enter_group() {
        let mut state = BlockSelectState::new(vec![make_entry("Knee", vec!["D", "E"])]);
        state.enter_group();
        assert!(matches!(state.focus, BlockSelectFocus::Items(_)));
    }

    // T46-ST1-TEST-10: BlockSelectState must have an `exit_items` method (was exit_techniques).
    #[test]
    fn block_select_state_exit_items() {
        let mut state = BlockSelectState::new(vec![make_entry("Elbow", vec!["F"])]);
        state.enter_group();
        state.exit_items();
        assert!(matches!(state.focus, BlockSelectFocus::Groups));
    }

    // T46-ST1-TEST-11: BlockSelectState must have an `in_items` method (was in_techniques).
    #[test]
    fn block_select_state_in_items() {
        let mut state = BlockSelectState::new(vec![make_entry("Wrist", vec!["G"])]);
        assert!(!state.in_items());
        state.enter_group();
        assert!(state.in_items());
    }

    // T46-ST1-TEST-12: BlockSelectState must have a `current_group_idx` method (was current_region_idx).
    #[test]
    fn block_select_state_current_group_idx() {
        let mut state = BlockSelectState::new(vec![make_entry("Ankle", vec!["H"])]);
        assert!(state.current_group_idx().is_none());
        state.enter_group();
        assert_eq!(state.current_group_idx(), Some(0));
    }
}

#[cfg(test)]
mod tests_st2_region_state_entries_field {
    use super::*;
    use crate::data::{HierarchyItem, HierarchyList};

    fn make_entry(label: &str, opts: Vec<&str>) -> HierarchyList {
        HierarchyList {
            id: label.to_lowercase().replace(' ', "_"),
            label: Some(label.to_string()),
            preview: None,
            sticky: false,
            default: None,
            modal_start: crate::data::ModalStart::List,
            repeating: None,
            items: opts
                .iter()
                .map(|s| HierarchyItem {
                    id: s.to_lowercase(),
                    label: s.to_string(),
                    default: None,
                    default_enabled: None,
                    output: Some(s.to_string()),
                    note: None,
                    fields: None,
                    branch_fields: Vec::new(),
                })
                .collect(),
        }
    }

    // ST2-TEST-1: BlockSelectGroup must expose an `entries` field of type Vec<PartOption>
    #[test]
    fn region_state_has_entries_field() {
        let entry = make_entry("Body", vec!["Option A", "Option B"]);
        let state = BlockSelectGroup::from_config(&entry);
        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.entries[0].label(), "Option A");
        assert_eq!(state.entries[1].label(), "Option B");
    }

    // ST2-TEST-2: navigate_down uses entries internally.
    #[test]
    fn block_select_state_navigate_down_uses_entries() {
        let entries = vec![
            make_entry("Arm", vec!["Alpha", "Beta", "Gamma"]),
            make_entry("Leg", vec!["Delta"]),
        ];
        let mut state = BlockSelectState::new(entries);
        state.enter_group();
        state.navigate_down();
        assert_eq!(state.item_cursor, 1);
    }

    // ST2-TEST-3: BlockSelectState::new accepts Vec<HierarchyList> and populates entries.
    #[test]
    fn block_select_state_new_populates_region_entries() {
        let entry = make_entry("Torso", vec!["X", "Y"]);
        let state = BlockSelectState::new(vec![entry]);
        assert_eq!(state.groups.len(), 1);
        assert_eq!(state.groups[0].entries.len(), 2);
    }
}
