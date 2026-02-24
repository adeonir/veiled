use std::io::{self, Write};
use std::path::{Path, PathBuf};

use console::style;

use crate::{config, registry, tmutil};

pub fn execute(yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let paths = {
        let mut guard = registry::Registry::locked()?;
        let reg = guard.load()?;
        reg.list().to_vec()
    };

    if paths.is_empty() {
        println!("{}", style("No exclusions to remove.").dim());
        return Ok(());
    }

    if !yes {
        print!(
            "Remove {} {}? [y/N] ",
            paths.len(),
            if paths.len() == 1 {
                "exclusion"
            } else {
                "exclusions"
            }
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", style("Aborted.").dim());
            return Ok(());
        }
    }

    let (existing, missing): (Vec<_>, Vec<_>) =
        paths.iter().partition(|p| Path::new(p.as_str()).exists());

    let existing_paths: Vec<PathBuf> = existing.iter().map(|p| PathBuf::from(p.as_str())).collect();

    let mut removed = missing.len();
    let mut failed: Vec<String> = Vec::new();

    if let Err(e) = tmutil::remove_exclusions(&existing_paths) {
        eprintln!(
            "{} batch removal failed, retrying individually: {e}",
            style("warning:").yellow().bold()
        );
        for path in &existing {
            if let Err(e) = tmutil::remove_exclusion(path.as_ref()) {
                eprintln!("{} {path}: {e}", style("warning:").yellow().bold());
                failed.push((*path).clone());
            } else {
                removed += 1;
            }
        }
    } else {
        removed += existing.len();
    }

    let mut cfg_guard = config::Config::locked()?;
    let mut cfg = cfg_guard.load()?;
    if !cfg.extra_exclusions.is_empty() {
        let before = cfg.extra_exclusions.len();
        cfg.extra_exclusions.retain(|p| failed.contains(p));
        if cfg.extra_exclusions.len() < before {
            cfg_guard.save(&cfg)?;
        }
    }

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;
    reg.paths.clone_from(&failed);
    reg.saved_bytes = None;
    guard.save(&reg)?;

    if failed.is_empty() {
        println!(
            "{} {} {}",
            style("Removed:").bold(),
            removed,
            if removed == 1 {
                "exclusion"
            } else {
                "exclusions"
            }
        );
    } else {
        println!(
            "{} {} {}, {} failed",
            style("Removed:").bold(),
            removed,
            if removed == 1 {
                "exclusion"
            } else {
                "exclusions"
            },
            failed.len()
        );
    }

    Ok(())
}
