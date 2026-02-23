use std::io::{self, Write};

use console::style;

use crate::{config, registry, tmutil};

pub fn execute(yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = registry::Registry::locked()?;
    let reg = guard.load()?;
    let paths = reg.list().to_vec();

    if paths.is_empty() {
        println!("{}", style("No exclusions to remove.").dim());
        return Ok(());
    }

    if !yes {
        print!(
            "Remove {} {}? [y/N] ",
            paths.len(),
            if paths.len() == 1 {
                "exclusion"
            } else {
                "exclusions"
            }
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", style("Aborted.").dim());
            return Ok(());
        }
    }

    let mut removed = 0u32;

    for path in &paths {
        if let Err(e) = tmutil::remove_exclusion(path.as_ref()) {
            eprintln!("{} {path}: {e}", style("warning:").yellow().bold(),);
        } else {
            removed += 1;
        }
    }

    let reg = registry::Registry::default();
    guard.save(&reg)?;

    let mut cfg = config::load()?;
    if !cfg.extra_exclusions.is_empty() {
        cfg.extra_exclusions.clear();
        config::save(&cfg)?;
    }

    println!(
        "{} {} {}",
        style("Removed").green().bold(),
        removed,
        if removed == 1 {
            "exclusion"
        } else {
            "exclusions"
        }
    );

    Ok(())
}
