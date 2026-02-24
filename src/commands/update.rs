use console::style;

use crate::{daemon, updater};

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let current = updater::current_version();
    println!(
        "{} {}",
        style("Checking for updates...").dim(),
        style(format!("(current: {current})")).dim()
    );

    let result = updater::check()?;

    if result.updated {
        println!(
            "{} {} -> {}",
            style("Updated").blue().bold(),
            result.old_version,
            result.new_version
        );

        if daemon::restart()? {
            println!("{}", style("Daemon restarted.").green().bold());
        }
    } else {
        println!("{}", style("Already up to date.").dim());
    }

    Ok(())
}
