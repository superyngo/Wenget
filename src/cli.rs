//! CLI argument parsing for Wenget

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wenget")]
#[command(author = "Wenget Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A cross-platform package manager for GitHub binaries", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage buckets (remote manifest sources)
    Bucket {
        #[command(subcommand)]
        command: BucketCommands,
    },

    /// Add (install) packages from cache or GitHub URL
    #[command(visible_alias = "install")]
    #[command(visible_alias = "a")]
    Add {
        /// Package names, GitHub URLs, or script paths/URLs to add (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,

        /// Custom command name (overrides the default executable name)
        #[arg(short = 'n', long = "name")]
        script_name: Option<String>,
    },

    /// List installed packages
    #[command(visible_alias = "ls")]
    List {
        /// Show all available packages from buckets (not just installed)
        #[arg(short = 'a', long = "all")]
        all: bool,
    },

    /// Show package information from cache or GitHub URL
    Info {
        /// Package names or GitHub URLs to show (supports wildcards * for cache queries)
        names: Vec<String>,
    },

    /// Search for packages
    #[command(visible_alias = "s")]
    Search {
        /// Package names to search (supports wildcards *)
        names: Vec<String>,
    },

    /// Upgrade installed packages
    #[command(visible_alias = "up")]
    Update {
        /// Package names to upgrade, or "all" for all packages (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Delete (remove) installed packages
    #[command(visible_alias = "remove")]
    Del {
        /// Package names to delete (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,

        /// Force deletion (allow deleting wenget itself)
        #[arg(short, long)]
        force: bool,
    },

    /// Initialize Wenget (create directories and set up PATH)
    Init {
        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum BucketCommands {
    /// Add a bucket
    Add {
        /// Bucket name
        name: String,

        /// URL to the manifest.json file
        url: String,
    },

    /// Delete buckets
    Del {
        /// Bucket names to delete
        names: Vec<String>,
    },

    /// List all buckets
    List,

    /// Refresh cache from buckets
    Refresh,
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
