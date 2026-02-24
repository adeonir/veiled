use std::time::Duration;

use console::style;
use indicatif::ProgressBar;

use crate::{daemon, disksize, registry};

pub fn execute(refresh: bool) -> Result<(), Box<dyn std::error::Error>> {
    if daemon::is_installed()? {
        println!("{} {}", style("Daemon:").bold(), style("active").green());
    } else {
        println!("{} {}", style("Daemon:").bold(), style("inactive").yellow());
    }

    let mut guard = registry::Registry::locked()?;
    let mut reg = guard.load()?;
    let count = reg.list().len();

    if count == 0 {
        println!("{}", style("No exclusions managed by veiled.").dim());
        return Ok(());
    }

    if refresh {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Calculating saved space...");
        spinner.enable_steady_tick(Duration::from_millis(80));

        let total = disksize::calculate_total_size(reg.list());
        reg.saved_bytes = Some(total);
        guard.save(&reg)?;

        spinner.finish_and_clear();
    }

    let saved = reg
        .saved_bytes
        .map(|b| format!(" ({} saved)", disksize::format_size(b)));

    println!(
        "{} {} {} excluded by veiled{}",
        style(count).bold(),
        if count == 1 { "path" } else { "paths" },
        if count == 1 { "is" } else { "are" },
        saved.unwrap_or_default(),
    );

    Ok(())
}
