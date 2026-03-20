use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pane_layout: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pane_layout: "default".to_string(),
        }
    }
}

impl Config {
    pub fn load(data_dir: &PathBuf) -> Result<Self> {
        let path = data_dir.join("config.yml");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, data_dir: &PathBuf) -> Result<()> {
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
