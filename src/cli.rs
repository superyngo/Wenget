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
    /// Manage package sources
    Source {
        #[command(subcommand)]
        command: SourceCommands,
    },

    /// Manage buckets (remote manifest sources)
    Bucket {
        #[command(subcommand)]
        command: BucketCommands,
    },

    /// Install packages (alias: add)
    #[command(visible_alias = "a")]
    Add {
        /// Package names to install (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// List installed packages
    #[command(visible_alias = "ls")]
    List,

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

    /// Delete installed packages
    Del {
        /// Package names to delete (supports wildcards *)
        names: Vec<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,

        /// Force deletion (allow deleting wenpm itself)
        #[arg(short, long)]
        force: bool,
    },

    /// Initialize WenPM (create directories and set up PATH)
    Init {
        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum SourceCommands {
    /// Add packages from GitHub repository URLs
    Add {
        /// GitHub repository URLs (can specify multiple)
        urls: Vec<String>,
    },

    /// Delete packages from sources
    Del {
        /// Package names or URLs to delete
        names: Vec<String>,
    },

    /// Import packages from a file or URL
    Import {
        /// Path to file or URL containing repository URLs (one per line)
        source: String,
    },

    /// Export package URLs or package info
    Export {
        /// Output file path (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Export format: txt (URLs) or json (package info)
        #[arg(short, long, default_value = "txt")]
        format: String,
    },

    /// Refresh package metadata from sources
    Refresh,

    /// List available packages from sources
    List,

    /// Show package information
    Info {
        /// Package names to show (supports wildcards *)
        names: Vec<String>,
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
