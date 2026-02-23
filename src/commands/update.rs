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

        let was_installed = daemon::is_installed()?;

        if was_installed {
            daemon::uninstall()?;

            let binary_path = std::env::current_exe()
                .map_err(|e| format!("failed to resolve binary path: {e}"))?;

            let plist = daemon::generate_plist(&binary_path)?;
            daemon::install(&plist)?;

            println!("{}", style("Daemon restarted.").green().bold());
        }
    } else {
        println!("{}", style("Already up to date.").dim());
    }

    Ok(())
}
