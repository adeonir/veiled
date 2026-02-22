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

    let mut cfg = config::load()?;
    if !cfg.extra_exclusions.contains(&canonical_str) {
        cfg.extra_exclusions.push(canonical_str.clone());
        config::save(&cfg)?;
    }

    tmutil::add_exclusion(&canonical)?;

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;
    reg.add(&canonical_str);
    guard.save(&reg)?;

    println!("{} {}", style("Added").green().bold(), canonical.display());

    Ok(())
}
