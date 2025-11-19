//! CLI argument parsing for WenPM

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wenpm")]
#[command(author = "WenPM Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A cross-platform package manager for GitHub binaries", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add packages from GitHub repository URLs
    #[command(visible_alias = "a")]
    Add {
        /// GitHub repository URLs (can specify multiple)
        urls: Vec<String>,

        /// Read URLs from a file or online URL (one per line)
        #[arg(short, long)]
        source: Option<String>,
    },

    /// List available packages for the current platform
    #[command(visible_alias = "ls")]
    List,

    /// Search for packages
    #[command(visible_alias = "s")]
    Search {
        /// Package names to search (supports wildcards *)
        names: Vec<String>,
    },

    /// Show package information
    Info {
        /// Package names to show (supports wildcards *)
        names: Vec<String>,
    },

    /// Update package metadata from sources
    #[command(visible_alias = "up")]
    Update,

    /// Install packages
    #[command(visible_alias = "i")]
    Install {
        /// Package names to install (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Upgrade installed packages
    #[command(visible_alias = "ug")]
    Upgrade {
        /// Package names to upgrade, or "all" for all packages (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Delete installed packages
    #[command(visible_alias = "rm")]
    Delete {
        /// Package names to delete (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,

        /// Force deletion (allow deleting wenpm itself)
        #[arg(short, long)]
        force: bool,
    },

    /// Set up PATH environment variable
    SetupPath {
        /// Show instructions without modifying PATH
        #[arg(long)]
        dry_run: bool,
    },

    /// Initialize WenPM (create directories and manifests)
    Init,
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
