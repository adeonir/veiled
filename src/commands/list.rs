use console::style;

use crate::registry;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let reg = registry::Registry::load()?;
    let paths = reg.list();

    if paths.is_empty() {
        println!("{}", style("No exclusions managed by veiled.").dim());
        return Ok(());
    }

    for path in paths {
        println!("{path}");
    }

    Ok(())
}
