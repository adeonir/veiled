use console::style;

use crate::{daemon, updater};

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{} {}",
        style("Checking for updates...").dim(),
        style(format!("(current: {})", updater::current_version())).dim()
    );

    let result = updater::check()?;

    if result.updated {
        println!(
            "{} {} -> {}",
            style("Updated!").green().bold(),
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
