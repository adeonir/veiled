use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "veiled", version)]
#[command(about = "A macOS CLI to exclude development artifacts from Time Machine backups")]
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
    Status,
    /// Update binary to the latest version
    Update,
}
