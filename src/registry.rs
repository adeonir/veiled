use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    pub paths: Vec<String>,
}

fn registry_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".config/veiled/registry.json")
}

impl Registry {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from(&registry_path())
    }

    pub fn load_from(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if path.exists() {
            let file = fs::File::open(path)?;
            let registry = serde_json::from_reader(BufReader::new(file))?;
            Ok(registry)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_to(&registry_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn add(&mut self, path: &str) {
        if !self.contains(path) {
            self.paths.push(path.to_string());
        }
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, path: &str) {
        self.paths.retain(|p| p != path);
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

        let registry = Registry::load_from(&path).unwrap();

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
    fn remove_path() {
        let mut registry = Registry::default();
        registry.add("/Users/dev/a/node_modules");
        registry.add("/Users/dev/b/target");

        registry.remove("/Users/dev/a/node_modules");

        assert_eq!(registry.list().len(), 1);
        assert!(!registry.contains("/Users/dev/a/node_modules"));
        assert!(registry.contains("/Users/dev/b/target"));
    }

    #[test]
    fn contains_check() {
        let mut registry = Registry::default();
        registry.add("/Users/dev/project/.venv");

        assert!(registry.contains("/Users/dev/project/.venv"));
        assert!(!registry.contains("/Users/dev/other/.venv"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("registry.json");

        let mut registry = Registry::default();
        registry.add("/Users/dev/app/node_modules");
        registry.add("/Users/dev/api/target");
        registry.save_to(&path).unwrap();

        let loaded = Registry::load_from(&path).unwrap();

        assert_eq!(loaded.list().len(), 2);
        assert!(loaded.contains("/Users/dev/app/node_modules"));
        assert!(loaded.contains("/Users/dev/api/target"));
    }
}
