use std::path::PathBuf;

use console::style;

use crate::{config, registry, tmutil, verbose};

pub fn execute(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let expanded = config::expand_tilde(path);

    let (lookup_path, exists) = match expanded.canonicalize() {
        Ok(canonical) => (canonical, true),
        Err(_) => (normalize_expanded(&expanded), false),
    };

    let lookup_str = lookup_path.to_string_lossy().into_owned();

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;

    if !reg.contains(&lookup_str) {
        return Err(format!("{}: not managed by veiled", lookup_path.display()).into());
    }

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

    reg.remove(&lookup_str);
    guard.save(&reg)?;

    let mut cfg_guard = config::Config::locked()?;
    let mut cfg = cfg_guard.load()?;
    if let Some(pos) = cfg.extra_exclusions.iter().position(|p| p == &lookup_str) {
        cfg.extra_exclusions.remove(pos);
        cfg_guard.save(&cfg)?;
    }

    println!(
        "{} {}",
        style("Removed").green().bold(),
        lookup_path.display()
    );

    Ok(())
}

fn normalize_expanded(path: &PathBuf) -> PathBuf {
    if path.is_absolute() {
        path.clone()
    } else {
        std::env::current_dir().map_or_else(|_| path.clone(), |cwd| cwd.join(path))
    }
}
