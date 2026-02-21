use console::style;

use crate::registry;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let reg = registry::Registry::load()?;
    let count = reg.list().len();

    if count == 0 {
        println!("{}", style("No exclusions managed by veiled.").dim());
    } else {
        println!(
            "{} {} {} excluded by veiled",
            style(count).bold(),
            if count == 1 { "path" } else { "paths" },
            if count == 1 { "is" } else { "are" },
        );
    }

    Ok(())
}
