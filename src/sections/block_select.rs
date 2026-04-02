use crate::data::{BlockSelectEntry, PartOption};

#[derive(Debug, Clone)]
pub struct RegionState {
    pub label: String,
    pub header: String,
    pub techniques: Vec<PartOption>,
    pub technique_selected: Vec<bool>,
}

impl RegionState {
    pub fn from_config(cfg: &BlockSelectEntry) -> Self {
        let technique_selected = vec![false; cfg.entries.len()];
        Self {
            label: cfg.label.clone(),
            header: cfg.header.clone(),
            techniques: cfg.entries.clone(),
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
                    if !region.techniques.is_empty()
                        && self.technique_cursor < region.techniques.len() - 1
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
