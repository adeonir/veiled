use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "veiled", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Install binary and activate daemon
    Start,
    /// Deactivate daemon and remove plist
    Stop,
    /// Run a scan manually
    Run,
    /// List all paths excluded by veiled
    List,
    /// Remove all exclusions managed by veiled
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Add a directory to the custom exclusion list
    Add {
        /// Path to exclude
        path: String,
    },
    /// Show daemon state and exclusion stats
    Status {
        /// Recalculate saved space
        #[arg(long)]
        refresh: bool,
    },
    /// Update binary to the latest version
    Update,
}
