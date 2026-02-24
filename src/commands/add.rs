use std::fs;

use console::style;

use crate::{config, registry, tmutil};

pub fn execute(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let expanded = config::expand_tilde(path);
    let canonical = fs::canonicalize(&expanded)
        .map_err(|_| format!("{}: no such directory", expanded.display()))?;

    if !canonical.is_dir() {
        return Err(format!("{}: not a directory", canonical.display()).into());
    }

    let canonical_str = canonical.to_string_lossy().into_owned();

    tmutil::add_exclusion(&canonical)?;

    let mut cfg_guard = config::Config::locked()?;
    let mut cfg = cfg_guard.load()?;
    if !cfg.extra_exclusions.contains(&canonical_str) {
        cfg.extra_exclusions.push(canonical_str.clone());
        cfg_guard.save(&cfg)?;
    }

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;

    for entry in reg.list() {
        if canonical_str != *entry && canonical_str.starts_with(&format!("{entry}/")) {
            eprintln!(
                "{} {} is already covered by {}",
                style("warning:").yellow().bold(),
                canonical.display(),
                entry
            );
            break;
        }
    }

    reg.add(&canonical_str);
    guard.save(&reg)?;

    println!("{} {}", style("Added").blue().bold(), canonical.display());

    Ok(())
}
