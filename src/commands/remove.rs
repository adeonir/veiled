use std::fs;

use console::style;

use crate::{config, registry, tmutil};

pub fn execute(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let expanded = config::expand_tilde(path);
    let canonical = fs::canonicalize(&expanded)
        .map_err(|_| format!("{}: no such directory", expanded.display()))?;

    let canonical_str = canonical.to_string_lossy().into_owned();

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;

    if !reg.contains(&canonical_str) {
        return Err(format!("{}: not managed by veiled", canonical.display()).into());
    }

    tmutil::remove_exclusion(&canonical)?;

    reg.remove(&canonical_str);
    guard.save(&reg)?;

    let mut cfg = config::load()?;
    if let Some(pos) = cfg
        .extra_exclusions
        .iter()
        .position(|p| p == &canonical_str)
    {
        cfg.extra_exclusions.remove(pos);
        config::save(&cfg)?;
    }

    println!(
        "{} {}",
        style("Removed").green().bold(),
        canonical.display()
    );

    Ok(())
}
