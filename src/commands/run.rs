use std::time::{Duration, SystemTime, UNIX_EPOCH};

use console::style;
use indicatif::ProgressBar;

use std::path::Path;

use crate::{config, daemon, disksize, registry, scanner, tmutil, updater, verbose};

const UPDATE_COOLDOWN_SECS: i64 = 86_400; // 24 hours

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load()?;

    if config.auto_update {
        auto_update()?;
    }

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;

    let mut re_applied = 0u32;
    let mut stale: Vec<String> = Vec::new();
    for entry in reg.list().to_vec() {
        let p = Path::new(&entry);
        if !p.exists() {
            stale.push(entry);
            continue;
        }
        if p.is_dir() && !tmutil::is_excluded(p).unwrap_or(true) {
            if let Err(e) = tmutil::add_exclusion(p) {
                eprintln!("{} {entry}: {e}", style("warning:").yellow().bold(),);
            } else {
                re_applied += 1;
            }
        }
    }

    for entry in &stale {
        if verbose() {
            eprintln!("{} pruning stale entry: {entry}", style("verbose:").dim(),);
        }
        reg.remove(entry);
    }

    if re_applied > 0 {
        println!(
            "{} {} lost {}",
            style("Re-applied").green().bold(),
            re_applied,
            if re_applied == 1 {
                "exclusion"
            } else {
                "exclusions"
            }
        );
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Scanning...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let paths = scanner::scan(&config);

    spinner.finish_and_clear();

    let new_paths: Vec<_> = paths
        .into_iter()
        .filter(|p| !reg.contains(&p.to_string_lossy()))
        .collect();

    if new_paths.is_empty() {
        if !stale.is_empty() {
            guard.save(&reg)?;
        }
        println!("{}", style("Nothing new to exclude.").dim());
        return Ok(());
    }

    let mut excluded = 0u32;
    let mut added_paths: Vec<String> = Vec::new();

    for path in &new_paths {
        if let Err(e) = tmutil::add_exclusion(path) {
            eprintln!(
                "{} {}: {e}",
                style("warning:").yellow().bold(),
                path.display()
            );
            continue;
        }
        let s = path.to_string_lossy().into_owned();
        reg.add(&s);
        added_paths.push(s);
        excluded += 1;
    }

    let new_size = disksize::calculate_total_size(&added_paths);
    reg.saved_bytes = Some(reg.saved_bytes.unwrap_or(0).saturating_add(new_size));
    guard.save(&reg)?;

    println!(
        "{} {} new {} ({} total)",
        style("Excluded").green().bold(),
        excluded,
        if excluded == 1 { "path" } else { "paths" },
        reg.list().len()
    );

    Ok(())
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs().cast_signed())
}

fn auto_update() -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;

    let now = now_epoch();

    if let Some(last) = reg.last_update_check
        && last <= now
        && now - last < UPDATE_COOLDOWN_SECS
    {
        if verbose() {
            eprintln!(
                "{} skipping update check (last checked {}s ago)",
                style("verbose:").dim(),
                now - last
            );
        }
        return Ok(());
    }

    reg.last_update_check = Some(now);
    guard.save(&reg)?;
    drop(guard);

    match updater::check() {
        Ok(result) if result.updated => {
            if let Err(e) = daemon::restart()
                && verbose()
            {
                eprintln!("{} daemon restart failed: {e}", style("verbose:").dim());
            }
        }
        Err(e) if verbose() => {
            eprintln!("{} auto-update failed: {e}", style("verbose:").dim());
        }
        _ => {}
    }

    Ok(())
}
