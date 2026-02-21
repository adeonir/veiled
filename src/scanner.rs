use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::builtins;

pub fn parse_git_ignored(repo_path: &Path, output: &str) -> Vec<PathBuf> {
    let mut dirs = HashSet::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let rel = Path::new(line);
        if let Some(first) = rel.components().next() {
            let name = first.as_os_str().to_string_lossy();
            if builtins::is_builtin(&name) {
                dirs.insert(repo_path.join(first));
            }
        }
    }

    dirs.into_iter().collect()
}

pub fn scan_non_git_dir(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return vec![];
    };

    let mut results = Vec::new();

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        if let Some(name) = entry_path.file_name()
            && builtins::is_builtin(&name.to_string_lossy())
        {
            results.push(entry_path);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_git_ignored_extracts_builtin_dirs() {
        let repo = Path::new("/Users/dev/project");
        let output =
            "node_modules/express/index.js\nnode_modules/.package-lock.json\nsrc/main.rs\n";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&repo.join("node_modules")));
    }

    #[test]
    fn parse_git_ignored_filters_non_builtin() {
        let repo = Path::new("/Users/dev/project");
        let output = "logs/app.log\nsrc/generated/types.ts\n";

        let results = parse_git_ignored(repo, output);

        assert!(results.is_empty());
    }

    #[test]
    fn parse_git_ignored_handles_empty_output() {
        let repo = Path::new("/Users/dev/project");
        let results = parse_git_ignored(repo, "");

        assert!(results.is_empty());
    }

    #[test]
    fn parse_git_ignored_deduplicates_same_dir() {
        let repo = Path::new("/Users/dev/project");
        let output = "target/debug/veiled\ntarget/release/veiled\ntarget/.rustc_info.json\n";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&repo.join("target")));
    }

    #[test]
    fn parse_git_ignored_handles_multiple_builtin_dirs() {
        let repo = Path::new("/Users/dev/project");
        let output = "node_modules/pkg/index.js\ntarget/debug/bin\n.next/cache/webpack\n";

        let results = parse_git_ignored(repo, output);

        assert_eq!(results.len(), 3);
        assert!(results.contains(&repo.join("node_modules")));
        assert!(results.contains(&repo.join("target")));
        assert!(results.contains(&repo.join(".next")));
    }

    #[test]
    fn scan_non_git_dir_finds_builtin_dirs() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("node_modules")).unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();

        let results = scan_non_git_dir(dir.path());

        assert_eq!(results.len(), 1);
        assert!(results[0].ends_with("node_modules"));
    }

    #[test]
    fn scan_non_git_dir_skips_non_builtin() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::create_dir(dir.path().join("docs")).unwrap();

        let results = scan_non_git_dir(dir.path());

        assert!(results.is_empty());
    }

    #[test]
    fn scan_non_git_dir_handles_empty_dir() {
        let dir = TempDir::new().unwrap();
        let results = scan_non_git_dir(dir.path());

        assert!(results.is_empty());
    }

    #[test]
    fn scan_non_git_dir_skips_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("node_modules"), "not a dir").unwrap();

        let results = scan_non_git_dir(dir.path());

        assert!(results.is_empty());
    }

    #[test]
    fn scan_non_git_dir_handles_nonexistent_path() {
        let results = scan_non_git_dir(Path::new("/nonexistent/path"));

        assert!(results.is_empty());
    }
}
