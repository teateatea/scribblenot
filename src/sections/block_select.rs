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
        let technique_selected = vec![false; cfg.entries.len()];
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
