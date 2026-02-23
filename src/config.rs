use std::fs;
use std::path::{Path, PathBuf};

use console::style;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub search_paths: Vec<String>,
    pub extra_exclusions: Vec<String>,
    pub ignore_paths: Vec<String>,
    pub auto_update: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search_paths: vec!["~/Projects".to_string(), "~/Developer".to_string()],
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

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct LegacyConfig {
    search_paths: Vec<String>,
    extra_exclusions: Vec<String>,
    ignore_paths: Vec<String>,
    auto_update: bool,
}

impl Default for LegacyConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![],
            extra_exclusions: vec![],
            ignore_paths: vec![],
            auto_update: true,
        }
    }
}

impl From<LegacyConfig> for Config {
    fn from(legacy: LegacyConfig) -> Self {
        Self {
            search_paths: legacy.search_paths,
            extra_exclusions: legacy.extra_exclusions,
            ignore_paths: legacy.ignore_paths,
            auto_update: legacy.auto_update,
        }
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".config/veiled/config.toml")
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

pub fn collapse_tilde(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path == home_str {
            return "~".to_string();
        }
        let prefix = format!("{home_str}/");
        if let Some(rest) = path.strip_prefix(&prefix) {
            return format!("~/{rest}");
        }
    }
    path.to_string()
}

fn collapse_paths(config: &mut Config) {
    for path in &mut config.search_paths {
        *path = collapse_tilde(path);
    }
    for path in &mut config.extra_exclusions {
        *path = collapse_tilde(path);
    }
    for path in &mut config.ignore_paths {
        *path = collapse_tilde(path);
    }
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

fn migrate_json(json_path: &Path, toml_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(json_path)?;
    let legacy: LegacyConfig = serde_json::from_str(&content).unwrap_or_default();
    let config: Config = legacy.into();
    save_to(&config, toml_path)?;
    fs::remove_file(json_path)?;
    Ok(())
}

pub fn save(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    save_to(config, &config_path())
}

pub fn save_to(config: &Config, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut collapsed = config.clone();
    collapse_paths(&mut collapsed);
    fs::write(path, toml::to_string_pretty(&collapsed)?)?;
    Ok(())
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    load_from(&config_path())
}

pub fn load_from(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        let json_path = parent.join("config.json");
        if json_path.exists()
            && !path.exists()
            && let Err(e) = migrate_json(&json_path, path)
        {
            eprintln!(
                "{} failed to migrate config.json: {e}",
                style("warning:").yellow().bold()
            );
        }
    }

    let mut config = if path.exists() {
        let content = fs::read_to_string(path)?;
        match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "{} failed to parse {}: {e}",
                    style("warning:").yellow().bold(),
                    path.display()
                );
                Config::default()
            }
        }
    } else {
        let config = Config::default();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml::to_string_pretty(&config)?)?;
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
        let path = dir.path().join("config.toml");

        let config = load_from(&path).unwrap();

        assert!(path.exists());
        assert_eq!(config.search_paths.len(), 2);
        assert!(config.auto_update);
    }

    #[test]
    fn default_config_is_toml_format() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        load_from(&path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("search_paths"));
        assert!(content.contains("auto_update"));
        assert!(!content.contains("searchPaths"));
    }

    #[test]
    fn loads_existing_toml_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let custom = "search_paths = [\"~/Code\", \"~/Work\"]\nextra_exclusions = []\nignore_paths = []\nauto_update = false\n";
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
        let path = dir.path().join("config.toml");

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
        let path = dir.path().join("config.toml");

        fs::write(&path, "{{invalid toml").unwrap();

        let config = load_from(&path).unwrap();

        assert_eq!(config.search_paths.len(), 2);
        assert!(config.auto_update);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = Config::default();
        config.extra_exclusions = vec!["/Users/dev/cache".to_string()];
        save_to(&config, &path).unwrap();

        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded.extra_exclusions.len(), 1);
        assert!(loaded.auto_update);
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested/dir/config.toml");

        let config = Config::default();
        save_to(&config, &path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn handles_partial_config_with_defaults() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        fs::write(&path, "auto_update = false\n").unwrap();

        let config = load_from(&path).unwrap();

        assert!(!config.auto_update);
        assert_eq!(config.search_paths.len(), 2);
        assert_eq!(config.ignore_paths.len(), 3);
    }

    #[test]
    fn migrates_json_to_toml() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("config.json");
        let toml_path = dir.path().join("config.toml");

        let json = r#"{
            "searchPaths": ["~/Code"],
            "extraExclusions": ["/tmp/cache"],
            "ignorePaths": ["~/Library"],
            "autoUpdate": false
        }"#;
        fs::write(&json_path, json).unwrap();

        let config = load_from(&toml_path).unwrap();

        assert!(!json_path.exists());
        assert!(toml_path.exists());
        assert_eq!(config.search_paths.len(), 1);
        assert!(!config.auto_update);
        assert_eq!(config.extra_exclusions.len(), 1);
    }

    #[test]
    fn collapse_tilde_replaces_home_prefix() {
        let home = dirs::home_dir().unwrap().to_string_lossy().into_owned();
        let path = format!("{home}/Projects/app");

        assert_eq!(collapse_tilde(&path), "~/Projects/app");
    }

    #[test]
    fn collapse_tilde_handles_bare_home() {
        let home = dirs::home_dir().unwrap().to_string_lossy().into_owned();

        assert_eq!(collapse_tilde(&home), "~");
    }

    #[test]
    fn collapse_tilde_leaves_non_home_paths_unchanged() {
        assert_eq!(collapse_tilde("/usr/local/bin"), "/usr/local/bin");
    }

    #[test]
    fn save_preserves_tilde_notation() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let home = dirs::home_dir().unwrap().to_string_lossy().into_owned();

        let mut config = Config::default();
        config.search_paths = vec![format!("{home}/Projects")];
        save_to(&config, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("~/Projects"));
        assert!(!content.contains(&home));
    }

    #[test]
    fn save_and_load_roundtrip_preserves_tilde() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let home = dirs::home_dir().unwrap().to_string_lossy().into_owned();

        let mut config = Config::default();
        config.extra_exclusions = vec![format!("{home}/cache")];
        save_to(&config, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("~/cache"));

        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.extra_exclusions[0], format!("{home}/cache"));
    }

    #[test]
    fn migration_skipped_when_toml_exists() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("config.json");
        let toml_path = dir.path().join("config.toml");

        fs::write(&json_path, r#"{"autoUpdate": false}"#).unwrap();
        fs::write(&toml_path, "auto_update = true\n").unwrap();

        let config = load_from(&toml_path).unwrap();

        assert!(json_path.exists());
        assert!(config.auto_update);
    }
}
