use std::process;

use clap::Parser;
use console::style;

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

    let result = match cli.command {
        cli::Commands::Start => commands::start::execute(),
        cli::Commands::Stop => commands::stop::execute(),
        cli::Commands::Run => commands::run::execute(),
        cli::Commands::List => commands::list::execute(),
        cli::Commands::Reset { yes } => commands::reset::execute(yes),
        cli::Commands::Add { ref path } => commands::add::execute(path),
        cli::Commands::Status => commands::status::execute(),
        cli::Commands::Update => commands::update::execute(),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", style("error:").red().bold());
        process::exit(1);
    }
}
