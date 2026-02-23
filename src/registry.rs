use std::fs;
use std::io::{BufReader, Seek};
use std::path::{Path, PathBuf};

use console::style;
use fs2::FileExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saved_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_check: Option<i64>,
}

fn registry_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(dir) = std::env::var("VEILED_CONFIG_DIR") {
        return Ok(PathBuf::from(dir).join("registry.json"));
    }
    let home = dirs::home_dir().ok_or("could not determine home directory")?;
    Ok(home.join(".config/veiled/registry.json"))
}

pub struct LockedRegistry {
    file: fs::File,
}

impl LockedRegistry {
    fn acquire(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        file.lock_exclusive()?;
        Ok(Self { file })
    }

    pub fn load(&mut self) -> Result<Registry, Box<dyn std::error::Error>> {
        self.file.rewind()?;
        let metadata = self.file.metadata()?;
        if metadata.len() == 0 {
            return Ok(Registry::default());
        }
        let reader = BufReader::new(&self.file);
        match serde_json::from_reader(reader) {
            Ok(registry) => Ok(registry),
            Err(e) => {
                eprintln!(
                    "{} failed to parse registry: {e}",
                    style("warning:").yellow().bold()
                );
                Ok(Registry::default())
            }
        }
    }

    pub fn save(&mut self, registry: &Registry) -> Result<(), Box<dyn std::error::Error>> {
        self.file.set_len(0)?;
        self.file.rewind()?;
        serde_json::to_writer_pretty(&self.file, registry)?;
        self.file.sync_data()?;
        Ok(())
    }
}

impl Registry {
    pub fn locked() -> Result<LockedRegistry, Box<dyn std::error::Error>> {
        LockedRegistry::acquire(&registry_path()?)
    }

    #[cfg(test)]
    pub fn locked_at(path: &Path) -> Result<LockedRegistry, Box<dyn std::error::Error>> {
        LockedRegistry::acquire(path)
    }

    pub fn add(&mut self, path: &str) {
        if !self.contains(path) {
            self.paths.push(path.to_string());
        }
    }

    pub fn remove(&mut self, path: &str) -> bool {
        let len = self.paths.len();
        self.paths.retain(|p| p != path);
        self.paths.len() < len
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|p| p == path)
    }

    pub fn list(&self) -> &[String] {
        &self.paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn loads_empty_when_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        let mut guard = Registry::locked_at(&path).unwrap();
        let registry = guard.load().unwrap();

        assert!(registry.paths.is_empty());
    }

    #[test]
    fn add_and_list() {
        let mut registry = Registry::default();

        registry.add("/Users/dev/project/node_modules");

        assert_eq!(registry.list().len(), 1);
        assert_eq!(registry.list()[0], "/Users/dev/project/node_modules");
    }

    #[test]
    fn add_is_idempotent() {
        let mut registry = Registry::default();

        registry.add("/Users/dev/project/target");
        registry.add("/Users/dev/project/target");

        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn contains_check() {
        let mut registry = Registry::default();
        registry.add("/Users/dev/project/.venv");

        assert!(registry.contains("/Users/dev/project/.venv"));
        assert!(!registry.contains("/Users/dev/other/.venv"));
    }

    #[test]
    fn remove_existing_path() {
        let mut registry = Registry::default();
        registry.add("/Users/dev/project/node_modules");

        assert!(registry.remove("/Users/dev/project/node_modules"));
        assert!(!registry.contains("/Users/dev/project/node_modules"));
        assert!(registry.list().is_empty());
    }

    #[test]
    fn remove_missing_path_returns_false() {
        let mut registry = Registry::default();

        assert!(!registry.remove("/Users/dev/project/node_modules"));
    }

    #[test]
    fn remove_preserves_other_paths() {
        let mut registry = Registry::default();
        registry.add("/Users/dev/project/node_modules");
        registry.add("/Users/dev/project/target");

        registry.remove("/Users/dev/project/node_modules");

        assert_eq!(registry.list().len(), 1);
        assert!(registry.contains("/Users/dev/project/target"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        let mut guard = Registry::locked_at(&path).unwrap();
        let mut registry = Registry::default();
        registry.add("/Users/dev/app/node_modules");
        registry.add("/Users/dev/api/target");
        guard.save(&registry).unwrap();
        drop(guard);

        let mut guard = Registry::locked_at(&path).unwrap();
        let loaded = guard.load().unwrap();

        assert_eq!(loaded.list().len(), 2);
        assert!(loaded.contains("/Users/dev/app/node_modules"));
        assert!(loaded.contains("/Users/dev/api/target"));
    }

    #[test]
    fn saved_bytes_defaults_to_none() {
        let registry = Registry::default();
        assert!(registry.saved_bytes.is_none());
    }

    #[test]
    fn saved_bytes_persists_on_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        let mut guard = Registry::locked_at(&path).unwrap();
        let mut registry = Registry::default();
        registry.add("/Users/dev/project/node_modules");
        registry.saved_bytes = Some(1_073_741_824);
        guard.save(&registry).unwrap();
        drop(guard);

        let mut guard = Registry::locked_at(&path).unwrap();
        let loaded = guard.load().unwrap();

        assert_eq!(loaded.saved_bytes, Some(1_073_741_824));
    }

    #[test]
    fn missing_saved_bytes_loads_as_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        fs::write(&path, r#"{"paths": ["/Users/dev/node_modules"]}"#).unwrap();

        let mut guard = Registry::locked_at(&path).unwrap();
        let loaded = guard.load().unwrap();

        assert_eq!(loaded.list().len(), 1);
        assert!(loaded.saved_bytes.is_none());
    }

    #[test]
    fn locked_load_save_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        let mut guard = Registry::locked_at(&path).unwrap();
        let mut reg = guard.load().unwrap();
        assert!(reg.paths.is_empty());

        reg.add("/Users/dev/project/node_modules");
        reg.saved_bytes = Some(500_000_000);
        reg.last_update_check = Some(1_700_000_000);
        guard.save(&reg).unwrap();
        drop(guard);

        let mut guard = Registry::locked_at(&path).unwrap();
        let loaded = guard.load().unwrap();

        assert_eq!(loaded.list().len(), 1);
        assert!(loaded.contains("/Users/dev/project/node_modules"));
        assert_eq!(loaded.saved_bytes, Some(500_000_000));
        assert_eq!(loaded.last_update_check, Some(1_700_000_000));
    }

    #[test]
    fn locked_creates_file_if_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("subdir/registry.json");

        let mut guard = Registry::locked_at(&path).unwrap();
        let reg = guard.load().unwrap();

        assert!(reg.paths.is_empty());
        assert!(path.exists());
    }

    #[test]
    fn last_update_check_defaults_to_none() {
        let registry = Registry::default();
        assert!(registry.last_update_check.is_none());
    }

    #[test]
    fn falls_back_to_defaults_on_malformed_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        fs::write(&path, "{{invalid json").unwrap();

        let mut guard = Registry::locked_at(&path).unwrap();
        let loaded = guard.load().unwrap();

        assert!(loaded.paths.is_empty());
        assert!(loaded.saved_bytes.is_none());
        assert!(loaded.last_update_check.is_none());
    }

    #[test]
    fn missing_last_update_check_loads_as_none() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        fs::write(
            &path,
            r#"{"paths": ["/Users/dev/node_modules"], "saved_bytes": 1024}"#,
        )
        .unwrap();

        let mut guard = Registry::locked_at(&path).unwrap();
        let loaded = guard.load().unwrap();

        assert_eq!(loaded.list().len(), 1);
        assert_eq!(loaded.saved_bytes, Some(1024));
        assert!(loaded.last_update_check.is_none());
    }
}
