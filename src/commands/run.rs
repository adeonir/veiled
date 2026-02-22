use std::time::{Duration, SystemTime, UNIX_EPOCH};

use console::style;
use indicatif::ProgressBar;

use crate::{config, disksize, registry, scanner, tmutil, updater, verbose};

const UPDATE_COOLDOWN_SECS: i64 = 86_400; // 24 hours

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load()?;

    if config.auto_update {
        auto_update()?;
    }

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;

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
        println!("{}", style("Nothing new to exclude.").dim());
        return Ok(());
    }

    let mut excluded = 0u32;

    for path in &new_paths {
        if let Err(e) = tmutil::add_exclusion(path) {
            eprintln!(
                "{} {}: {e}",
                style("warning:").yellow().bold(),
                path.display()
            );
            continue;
        }
        reg.add(&path.to_string_lossy());
        excluded += 1;
    }

    reg.saved_bytes = Some(disksize::calculate_total_size(reg.list()));
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

    let _ = updater::check();

    Ok(())
}
