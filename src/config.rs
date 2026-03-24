use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pane_layout: String,
    #[serde(default)]
    pub sticky_values: HashMap<String, String>,
    #[serde(default = "default_true")]
    pub hint_labels_capitalized: bool,
    #[serde(default)]
    pub hint_labels_case_sensitive: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pane_layout: "default".to_string(),
            sticky_values: HashMap::new(),
            hint_labels_capitalized: true,
            hint_labels_case_sensitive: false,
        }
    }
}

impl Config {
    pub fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join("config.yml");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, data_dir: &Path) -> Result<()> {
        let path = data_dir.join("config.yml");
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_swapped(&self) -> bool {
        self.pane_layout == "swapped"
    }

    pub fn set_swapped(&mut self, swapped: bool) {
        self.pane_layout = if swapped {
            "swapped".to_string()
        } else {
            "default".to_string()
        };
    }
}
