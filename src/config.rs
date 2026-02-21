use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub search_paths: Vec<String>,
    pub extra_exclusions: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub auto_update: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search_paths: vec!["~/Projects".to_string()],
            extra_exclusions: vec![],
            ignore_paths: vec![
                "~/.Trash".to_string(),
                "~/Library".to_string(),
                "~/Downloads".to_string(),
            ],
            auto_update: true,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".config/veiled/config.json")
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        if path == "~" {
            return home;
        }
        if let Some(rest) = path.strip_prefix("~/") {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn expand_paths(config: &mut Config) {
    for path in &mut config.search_paths {
        *path = expand_tilde(path).to_string_lossy().into_owned();
    }
    for path in &mut config.extra_exclusions {
        *path = expand_tilde(path).to_string_lossy().into_owned();
    }
    for path in &mut config.ignore_paths {
        *path = expand_tilde(path).to_string_lossy().into_owned();
    }
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    load_from(&config_path())
}

pub fn load_from(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = if path.exists() {
        let file = fs::File::open(path)?;
        serde_json::from_reader(BufReader::new(file)).unwrap_or_default()
    } else {
        let config = Config::default();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(&config)?)?;
        config
    };

    expand_paths(&mut config);
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_default_config_when_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        let config = load_from(&path).unwrap();

        assert!(path.exists());
        assert_eq!(config.search_paths.len(), 1);
        assert!(config.auto_update);
    }

    #[test]
    fn loads_existing_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        let custom = r#"{
            "searchPaths": ["~/Code", "~/Work"],
            "extraExclusions": [],
            "ignorePaths": [],
            "autoUpdate": false
        }"#;
        fs::write(&path, custom).unwrap();

        let config = load_from(&path).unwrap();

        assert_eq!(config.search_paths.len(), 2);
        assert!(!config.auto_update);
    }

    #[test]
    fn expands_tilde_in_paths() {
        let home = dirs::home_dir().unwrap();
        let expanded = expand_tilde("~/Projects");

        assert_eq!(expanded, home.join("Projects"));
    }

    #[test]
    fn expands_bare_tilde() {
        let home = dirs::home_dir().unwrap();
        let expanded = expand_tilde("~");

        assert_eq!(expanded, home);
    }

    #[test]
    fn leaves_absolute_paths_unchanged() {
        let expanded = expand_tilde("/usr/local/bin");

        assert_eq!(expanded, PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn expands_tilde_in_config_paths() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        let config = load_from(&path).unwrap();
        let home = dirs::home_dir().unwrap().to_string_lossy().into_owned();

        assert!(
            config.search_paths[0].starts_with(&home),
            "expected path to start with home dir, got: {}",
            config.search_paths[0]
        );
        assert!(!config.search_paths[0].contains('~'));
    }

    #[test]
    fn falls_back_to_defaults_on_malformed_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        fs::write(&path, "").unwrap();

        let config = load_from(&path).unwrap();

        assert_eq!(config.search_paths.len(), 1);
        assert!(config.auto_update);
    }

    #[test]
    fn handles_partial_config_with_defaults() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");

        fs::write(&path, r#"{"autoUpdate": false}"#).unwrap();

        let config = load_from(&path).unwrap();

        assert!(!config.auto_update);
        assert_eq!(config.search_paths.len(), 1);
        assert_eq!(config.ignore_paths.len(), 3);
    }
}
