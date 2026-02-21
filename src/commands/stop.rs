use console::style;

use crate::daemon;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    if !daemon::is_installed() {
        println!("{}", style("Daemon is not running.").dim());
        return Ok(());
    }

    daemon::uninstall()?;

    println!("{}", style("Daemon deactivated.").green().bold());

    Ok(())
}
