use console::style;

use crate::{daemon, disksize, registry};

pub fn execute(refresh: bool) -> Result<(), Box<dyn std::error::Error>> {
    if daemon::is_installed() {
        println!("{} {}", style("Daemon:").bold(), style("active").green());
    } else {
        println!("{} {}", style("Daemon:").bold(), style("inactive").yellow());
    }

    let mut reg = registry::Registry::load()?;
    let count = reg.list().len();

    if count == 0 {
        println!("{}", style("No exclusions managed by veiled.").dim());
        return Ok(());
    }

    if refresh {
        let total = disksize::calculate_total_size(reg.list());
        reg.saved_bytes = Some(total);
        reg.save()?;
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
