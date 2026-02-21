use clap::Parser;

#[allow(dead_code)]
mod builtins;
mod cli;
mod commands;
#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod registry;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Start => commands::start::execute(),
        cli::Commands::Stop => commands::stop::execute(),
        cli::Commands::Run => commands::run::execute(),
        cli::Commands::List => commands::list::execute(),
        cli::Commands::Reset => commands::reset::execute(),
        cli::Commands::Add { ref path } => commands::add::execute(path),
        cli::Commands::Status => commands::status::execute(),
        cli::Commands::Update => commands::update::execute(),
    }
}
