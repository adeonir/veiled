use console::style;

use crate::{commands, daemon, registry};

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    if daemon::is_installed() {
        println!("{}", style("Daemon is already running.").dim());
        return Ok(());
    }

    let binary_path =
        std::env::current_exe().map_err(|e| format!("failed to resolve binary path: {e}"))?;

    let plist = daemon::generate_plist(&binary_path);
    daemon::install(&plist)?;

    println!("{}", style("Daemon activated.").green().bold());

    let needs_scan = {
        let mut guard = registry::Registry::locked()?;
        let reg = guard.load()?;
        reg.list().is_empty()
    };

    if needs_scan {
        commands::run::execute()
    } else {
        Ok(())
    }
}
