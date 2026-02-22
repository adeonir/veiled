use console::style;

use crate::registry;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = registry::Registry::locked()?;
    let reg = guard.load()?;
    let paths = reg.list();

    if paths.is_empty() {
        println!("{}", style("No exclusions managed by veiled.").dim());
        return Ok(());
    }

    for path in paths {
        let p = std::path::Path::new(&path);
        match (
            p.parent().and_then(|p| p.to_str()),
            p.file_name().and_then(|n| n.to_str()),
        ) {
            (Some(parent), Some(name)) => {
                println!("{}{name}", style(format!("{parent}/")).dim());
            }
            _ => println!("{path}"),
        }
    }

    Ok(())
}
