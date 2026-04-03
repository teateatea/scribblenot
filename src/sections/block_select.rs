use crate::data::{BlockSelectEntry, PartOption};

#[derive(Debug, Clone)]
pub struct RegionState {
    pub label: String,
    pub header: String,
    pub entries: Vec<PartOption>,
    pub technique_selected: Vec<bool>,
}

impl RegionState {
    pub fn from_config(cfg: &BlockSelectEntry) -> Self {
        let technique_selected = cfg.entries.iter().map(|e| e.default_selected()).collect();
        Self {
            label: cfg.label.clone(),
            header: cfg.header.clone(),
            entries: cfg.entries.clone(),
            technique_selected,
        }
    }

    pub fn has_selection(&self) -> bool {
        self.technique_selected.iter().any(|&s| s)
    }

    pub fn toggle_technique(&mut self, idx: usize) {
        if let Some(val) = self.technique_selected.get_mut(idx) {
            *val = !*val;
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockSelectFocus {
    Regions,
    Techniques(usize),
}

#[derive(Debug, Clone)]
pub struct BlockSelectState {
    pub regions: Vec<RegionState>,
    pub region_cursor: usize,
    pub technique_cursor: usize,
    pub focus: BlockSelectFocus,
    pub skipped: bool,
    pub completed: bool,
}

impl BlockSelectState {
    pub fn new(regions: Vec<BlockSelectEntry>) -> Self {
        let region_states = regions.iter().map(RegionState::from_config).collect();
        Self {
            regions: region_states,
            region_cursor: 0,
            technique_cursor: 0,
            focus: BlockSelectFocus::Regions,
            skipped: false,
            completed: false,
        }
    }

    pub fn navigate_up(&mut self) {
        match &self.focus {
            BlockSelectFocus::Regions => {
                if self.region_cursor > 0 {
                    self.region_cursor -= 1;
                }
            }
            BlockSelectFocus::Techniques(_) => {
                if self.technique_cursor > 0 {
                    self.technique_cursor -= 1;
                }
            }
        }
    }

    pub fn navigate_down(&mut self) {
        match &self.focus {
            BlockSelectFocus::Regions => {
                if !self.regions.is_empty() && self.region_cursor < self.regions.len() - 1 {
                    self.region_cursor += 1;
                }
            }
            BlockSelectFocus::Techniques(region_idx) => {
                let region_idx = *region_idx;
                if let Some(region) = self.regions.get(region_idx) {
                    if !region.entries.is_empty()
                        && self.technique_cursor < region.entries.len() - 1
                    {
                        self.technique_cursor += 1;
                    }
                }
            }
        }
    }

    pub fn enter_region(&mut self) {
        let idx = self.region_cursor;
        self.focus = BlockSelectFocus::Techniques(idx);
        self.technique_cursor = 0;
    }

    pub fn exit_techniques(&mut self) {
        self.focus = BlockSelectFocus::Regions;
    }

    pub fn toggle_technique(&mut self) {
        if let BlockSelectFocus::Techniques(region_idx) = self.focus {
            if let Some(region) = self.regions.get_mut(region_idx) {
                region.toggle_technique(self.technique_cursor);
            }
        }
    }

    pub fn in_techniques(&self) -> bool {
        matches!(self.focus, BlockSelectFocus::Techniques(_))
    }

    pub fn current_region_idx(&self) -> Option<usize> {
        match self.focus {
            BlockSelectFocus::Techniques(i) => Some(i),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests_st3_default_selected {
    use super::*;
    use crate::data::{BlockSelectEntry, PartOption};

    fn make_full_entry(id: &str, label: &str, default: bool) -> PartOption {
        PartOption::Full {
            id: id.to_string(),
            label: label.to_string(),
            output: label.to_lowercase(),
            default,
        }
    }

    // ST3-TEST-1: PartOption must expose a default_selected() -> bool method.
    // For Full variants, it returns the `default` field value.
    // For Simple and Labeled variants, it returns true (no default field = always selected).
    #[test]
    fn part_option_default_selected_full_true() {
        let opt = make_full_entry("a", "Alpha", true);
        assert!(opt.default_selected(), "Full with default=true should return true from default_selected()");
    }

    #[test]
    fn part_option_default_selected_full_false() {
        let opt = make_full_entry("b", "Beta", false);
        assert!(!opt.default_selected(), "Full with default=false should return false from default_selected()");
    }

    #[test]
    fn part_option_default_selected_simple() {
        let opt = PartOption::Simple("gamma".to_string());
        assert!(opt.default_selected(), "Simple variant should always return true from default_selected()");
    }

    // ST3-TEST-2: RegionState::from_config where all entries have default=true (or omitted)
    // should initialize technique_selected to all true.
    #[test]
    fn region_state_all_default_true_starts_all_selected() {
        let entry = BlockSelectEntry {
            id: "region_a".to_string(),
            label: "Region A".to_string(),
            header: "Region A Header".to_string(),
            entries: vec![
                make_full_entry("opt1", "Option 1", true),
                make_full_entry("opt2", "Option 2", true),
                make_full_entry("opt3", "Option 3", true),
            ],
        };
        let state = RegionState::from_config(&entry);
        assert_eq!(state.technique_selected.len(), 3);
        assert!(state.technique_selected[0], "entry 0 with default=true should start selected");
        assert!(state.technique_selected[1], "entry 1 with default=true should start selected");
        assert!(state.technique_selected[2], "entry 2 with default=true should start selected");
    }

    // ST3-TEST-3: RegionState::from_config where one entry has default=false
    // should initialize that entry's slot as false, others as true.
    #[test]
    fn region_state_one_default_false_starts_unselected() {
        let entry = BlockSelectEntry {
            id: "region_b".to_string(),
            label: "Region B".to_string(),
            header: "Region B Header".to_string(),
            entries: vec![
                make_full_entry("opt1", "Option 1", true),
                make_full_entry("opt2", "Option 2", false),
                make_full_entry("opt3", "Option 3", true),
            ],
        };
        let state = RegionState::from_config(&entry);
        assert_eq!(state.technique_selected.len(), 3);
        assert!(state.technique_selected[0], "entry 0 with default=true should start selected");
        assert!(!state.technique_selected[1], "entry 1 with default=false should start unselected");
        assert!(state.technique_selected[2], "entry 2 with default=true should start selected");
    }

    // ST3-TEST-4: BlockSelectState::new propagates default selection through from_config.
    // All entries default=true => all selected; one default=false => that one unselected.
    #[test]
    fn block_select_state_new_propagates_defaults() {
        let regions = vec![
            BlockSelectEntry {
                id: "r1".to_string(),
                label: "R1".to_string(),
                header: "R1 Header".to_string(),
                entries: vec![
                    make_full_entry("a", "A", true),
                    make_full_entry("b", "B", false),
                ],
            },
        ];
        let state = BlockSelectState::new(regions);
        assert!(state.regions[0].technique_selected[0], "A (default=true) should start selected");
        assert!(!state.regions[0].technique_selected[1], "B (default=false) should start unselected");
    }
}

#[cfg(test)]
mod tests_st2_region_state_entries_field {
    use super::*;
    use crate::data::{BlockSelectEntry, PartOption};

    fn make_entry(label: &str, opts: Vec<&str>) -> BlockSelectEntry {
        BlockSelectEntry {
            id: label.to_lowercase().replace(' ', "_"),
            label: label.to_string(),
            header: format!("{} header", label),
            entries: opts.iter().map(|s| PartOption::Simple(s.to_string())).collect(),
        }
    }

    // ST2-TEST-1: RegionState must expose an `entries` field of type Vec<PartOption>
    // This test accesses .entries directly; if the field is still named `techniques` it fails to compile.
    #[test]
    fn region_state_has_entries_field() {
        let entry = make_entry("Body", vec!["Option A", "Option B"]);
        let state = RegionState::from_config(&entry);
        // Access the `entries` field — must compile only when field is named `entries`
        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.entries[0].label(), "Option A");
        assert_eq!(state.entries[1].label(), "Option B");
    }

    // ST2-TEST-2: `techniques` field must NOT exist on RegionState.
    // Since Rust is structural, this test accesses `entries` to prove rename happened.
    // navigate_down also references region.entries internally — call it to exercise that path.
    #[test]
    fn block_select_state_navigate_down_uses_entries() {
        let entries = vec![
            make_entry("Arm", vec!["Alpha", "Beta", "Gamma"]),
            make_entry("Leg", vec!["Delta"]),
        ];
        let mut state = BlockSelectState::new(entries);
        state.enter_region();
        // navigate_down internally accesses region.entries (post-rename) — must compile
        state.navigate_down();
        assert_eq!(state.technique_cursor, 1);
    }

    // ST2-TEST-3: BlockSelectState::new accepts Vec<BlockSelectEntry> (already required by ST1,
    // but verify the round-trip populates region.entries correctly end-to-end).
    #[test]
    fn block_select_state_new_populates_region_entries() {
        let entry = make_entry("Torso", vec!["X", "Y"]);
        let state = BlockSelectState::new(vec![entry]);
        assert_eq!(state.regions.len(), 1);
        // .entries must exist and be populated
        assert_eq!(state.regions[0].entries.len(), 2);
    }
}
