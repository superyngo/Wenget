//! Wenget - A cross-platform package manager for GitHub binaries

mod bucket;
mod cache;
mod cli;
mod commands;
mod core;
mod downloader;
mod installer;
mod providers;
mod utils;

use clap::CommandFactory;
use cli::{BucketCommands, Cli, Commands, SourceCommands};
use colored::Colorize;

fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Parse CLI arguments
    let cli = Cli::parse_args();

    // Set verbose logging if requested
    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }

    // Handle no command (show help and exit 0)
    let Some(command) = cli.command else {
        let _ = Cli::command().print_help();
        println!(); // Add newline after help
        return;
    };

    // Run the appropriate command
    let result = match command {
        Commands::Init { yes } => commands::run_init(yes),

        Commands::Source { command } => {
            let source_cmd = match command {
                SourceCommands::Add { urls } => commands::source::SourceCommand::Add { urls },
                SourceCommands::Del { names } => commands::source::SourceCommand::Del { names },
                SourceCommands::Import { source } => {
                    commands::source::SourceCommand::Import { source }
                }
                SourceCommands::Export { output, format } => {
                    commands::source::SourceCommand::Export { output, format }
                }
                SourceCommands::Refresh => commands::source::SourceCommand::Refresh,
                SourceCommands::List => commands::source::SourceCommand::List,
                SourceCommands::Info { names } => commands::source::SourceCommand::Info { names },
            };
            commands::run_source(source_cmd)
        }

        Commands::Bucket { command } => {
            let bucket_cmd = match command {
                BucketCommands::Add { name, url } => {
                    commands::bucket::BucketCommand::Add { name, url }
                }
                BucketCommands::Del { names } => commands::bucket::BucketCommand::Del { names },
                BucketCommands::List => commands::bucket::BucketCommand::List,
                BucketCommands::Refresh => commands::bucket::BucketCommand::Refresh,
            };
            commands::run_bucket(bucket_cmd)
        }

        Commands::Install { names, yes } => commands::run_add(names, yes),

        Commands::List => commands::run_list(),

        Commands::Search { names } => commands::run_search(names),

        Commands::Update { names, yes } => commands::run_update(names, yes),

        Commands::Remove { names, yes, force } => commands::run_delete(names, yes, force),
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
