use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use console::style;
use indicatif::ProgressBar;

use crate::{config, daemon, disksize, registry, scanner, tmutil, updater, verbose};

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

    let stale_count = prune_stale(&mut reg);
    let re_applied = reapply_lost(&reg);

    let candidates = scanner::scan(&config, &|_| {});
    let added_paths = reconcile(&mut reg, candidates);

    if stale_count > 0 || !added_paths.is_empty() {
        let total = disksize::calculate_total_size(reg.list());
        reg.saved_bytes = if total > 0 { Some(total) } else { None };
    }
    if stale_count > 0 || re_applied > 0 || !added_paths.is_empty() {
        guard.save(&reg)?;
    }

    spinner.finish_and_clear();
    print_summary(
        re_applied,
        added_paths.len(),
        reg.list().len(),
        reg.saved_bytes,
    );

    Ok(())
}

fn prune_stale(reg: &mut registry::Registry) -> usize {
    let mut count = 0usize;
    for entry in reg.list().to_vec() {
        if !Path::new(&entry).exists() {
            if verbose() {
                eprintln!("{} pruning stale entry: {entry}", style("verbose:").dim());
            }
            reg.remove(&entry);
            count += 1;
        }
    }
    count
}

fn reapply_lost(reg: &registry::Registry) -> usize {
    let entries: Vec<String> = reg.list().to_vec();
    if entries.is_empty() {
        return 0;
    }

    let paths: Vec<PathBuf> = entries.iter().map(PathBuf::from).collect();
    let status = tmutil::are_excluded(&paths);

    let lost: Vec<PathBuf> = paths
        .into_iter()
        .zip(status.iter())
        .filter(|(_, excluded)| !**excluded)
        .map(|(path, _)| path)
        .collect();

    if lost.is_empty() {
        return 0;
    }

    let count = lost.len();
    if let Err(e) = tmutil::add_exclusions(&lost) {
        eprintln!(
            "{} batch re-apply failed: {e}",
            style("warning:").yellow().bold()
        );
        return 0;
    }
    count
}

fn reconcile(reg: &mut registry::Registry, candidates: Vec<PathBuf>) -> Vec<String> {
    let new_candidates: Vec<PathBuf> = candidates
        .into_iter()
        .filter(|p| !reg.contains(&p.to_string_lossy()))
        .collect();

    if new_candidates.is_empty() {
        return vec![];
    }

    let excluded_status = tmutil::are_excluded(&new_candidates);

    let mut added = Vec::new();
    let mut to_exclude: Vec<(PathBuf, String)> = Vec::new();

    for (path, is_excluded) in new_candidates.iter().zip(excluded_status.iter()) {
        let s = path.to_string_lossy().into_owned();

        if *is_excluded {
            reg.add(&s);
            added.push(s);
        } else {
            to_exclude.push((path.clone(), s));
        }
    }

    if !to_exclude.is_empty() {
        let exclude_paths: Vec<PathBuf> = to_exclude.iter().map(|(p, _)| p.clone()).collect();
        if let Err(e) = tmutil::add_exclusions(&exclude_paths) {
            eprintln!(
                "{} batch exclusion failed: {e}",
                style("warning:").yellow().bold()
            );
        } else {
            for (_, s) in to_exclude {
                reg.add(&s);
                added.push(s);
            }
        }
    }

    added
}

fn print_summary(
    re_applied: usize,
    total_added: usize,
    total_managed: usize,
    saved_bytes: Option<u64>,
) {
    if re_applied > 0 {
        println!(
            "{} {} lost {}",
            style("Re-applied").blue().bold(),
            re_applied,
            if re_applied == 1 {
                "exclusion"
            } else {
                "exclusions"
            }
        );
    }

    if total_added > 0 {
        let details = match (total_added == total_managed, saved_bytes) {
            (true, Some(b)) => format!(" ({} saved)", disksize::format_size(b)),
            (true, None) => String::new(),
            (false, Some(b)) => {
                format!(
                    " ({total_managed} total, {} saved)",
                    disksize::format_size(b)
                )
            }
            (false, None) => format!(" ({total_managed} total)"),
        };
        println!(
            "{} {} new {}{details}",
            style("Excluded").blue().bold(),
            total_added,
            if total_added == 1 { "path" } else { "paths" },
        );
    } else if re_applied == 0 {
        println!("{}", style("Nothing new to exclude.").dim());
    }
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
