use std::process;
use std::sync::OnceLock;

use clap::Parser;
use console::style;

static VERBOSE: OnceLock<bool> = OnceLock::new();

pub fn verbose() -> bool {
    VERBOSE.get().copied().unwrap_or(false)
}

mod builtins;
mod cli;
mod commands;
mod config;
mod daemon;
mod disksize;
mod registry;
mod scanner;
mod tmutil;
mod updater;

fn main() {
    let cli = cli::Cli::parse();

    let _ = VERBOSE.set(cli.verbose);

    if matches!(
        cli.command,
        cli::Commands::Start
            | cli::Commands::Run
            | cli::Commands::Add { .. }
            | cli::Commands::Remove { .. }
            | cli::Commands::Reset { .. }
            | cli::Commands::Status { .. }
    ) && let Err(detail) = tmutil::check_access()
    {
        eprintln!(
            "{} Full Disk Access may be required. Grant access to your terminal in System Settings > Privacy & Security > Full Disk Access.",
            style("warning:").yellow().bold()
        );
        if verbose() {
            eprintln!("{} {detail}", style("detail:").yellow());
        }
    }

    let result = match cli.command {
        cli::Commands::Start => commands::start::execute(),
        cli::Commands::Stop => commands::stop::execute(),
        cli::Commands::Run => commands::run::execute(),
        cli::Commands::List => commands::list::execute(),
        cli::Commands::Reset { yes } => commands::reset::execute(yes),
        cli::Commands::Add { ref path } => commands::add::execute(path),
        cli::Commands::Remove { ref path } => commands::remove::execute(path),
        cli::Commands::Status { refresh } => commands::status::execute(refresh),
        cli::Commands::Update => commands::update::execute(),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", style("error:").red().bold());
        process::exit(1);
    }
}
