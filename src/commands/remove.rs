use std::path::{Component, Path, PathBuf};

use console::style;

use crate::{config, disksize, registry, tmutil, verbose};

pub fn execute(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let expanded = config::expand_tilde(path);

    let (lookup_path, exists) = match expanded.canonicalize() {
        Ok(canonical) => (canonical, true),
        Err(_) => (clean_path(&normalize_expanded(&expanded)?), false),
    };

    let lookup_str = lookup_path.to_string_lossy().into_owned();

    if exists {
        if let Err(e) = tmutil::remove_exclusion(&lookup_path) {
            eprintln!(
                "{} {}: {e}",
                style("warning:").yellow().bold(),
                lookup_path.display()
            );
        }
    } else if verbose() {
        eprintln!(
            "{} {} no longer exists on disk, skipping tmutil",
            style("verbose:").dim(),
            lookup_path.display()
        );
    }

    let mut cfg_guard = config::Config::locked()?;
    let mut cfg = cfg_guard.load()?;
    if let Some(pos) = cfg.extra_exclusions.iter().position(|p| p == &lookup_str) {
        cfg.extra_exclusions.remove(pos);
        cfg_guard.save(&cfg)?;
    }

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;

    if !reg.contains(&lookup_str) {
        return Err(format!("{}: not managed by veiled", lookup_path.display()).into());
    }

    let removed_size = disksize::dir_size(&lookup_path);
    if removed_size > 0 {
        reg.saved_bytes = Some(reg.saved_bytes.unwrap_or(0).saturating_sub(removed_size));
    }

    reg.remove(&lookup_str);
    guard.save(&reg)?;

    println!(
        "{} {}",
        style("Removed").blue().bold(),
        lookup_path.display()
    );

    Ok(())
}

fn normalize_expanded(path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.is_absolute() {
        Ok(path.clone())
    } else {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("could not determine current directory: {e}"))?;
        Ok(cwd.join(path))
    }
}

fn clean_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_path_resolves_parent_dir() {
        assert_eq!(clean_path(Path::new("/a/b/../c")), Path::new("/a/c"));
    }

    #[test]
    fn clean_path_resolves_current_dir() {
        assert_eq!(clean_path(Path::new("/a/./b/./c")), Path::new("/a/b/c"));
    }

    #[test]
    fn clean_path_handles_absolute() {
        assert_eq!(clean_path(Path::new("/a/b/c")), Path::new("/a/b/c"));
    }

    #[test]
    fn clean_path_stops_at_root() {
        assert_eq!(clean_path(Path::new("/a/../../etc")), Path::new("/etc"));
    }
}
