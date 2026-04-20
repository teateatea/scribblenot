use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pane_layout: String,
    #[serde(default = "default_theme_name")]
    pub theme: String,
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

fn default_theme_name() -> String {
    "default-theme".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pane_layout: "default".to_string(),
            theme: default_theme_name(),
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
        if data_dir.as_os_str().is_empty() {
            return Ok(());
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct CurrentDirGuard {
        original: PathBuf,
    }

    impl CurrentDirGuard {
        fn set_to(path: &Path) -> Self {
            let original = std::env::current_dir().expect("current dir");
            std::env::set_current_dir(path).expect("set current dir");
            Self { original }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original).expect("restore current dir");
        }
    }

    #[test]
    fn save_with_empty_path_is_noop() {
        let _lock = cwd_lock().lock().expect("cwd lock");
        let temp = tempdir().expect("temp dir");
        let _cwd = CurrentDirGuard::set_to(temp.path());

        Config::default()
            .save(Path::new(""))
            .expect("empty path save should succeed");

        assert!(
            !temp.path().join("config.yml").exists(),
            "empty data_dir should not write config.yml into the current directory"
        );
    }

    #[test]
    fn save_writes_config_when_data_dir_is_present() {
        let temp = tempdir().expect("temp dir");

        Config::default()
            .save(temp.path())
            .expect("config save should succeed");

        assert!(
            temp.path().join("config.yml").exists(),
            "non-empty data_dir should still write config.yml"
        );
    }
}
