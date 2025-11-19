//! WenPM - A cross-platform package manager for GitHub binaries

mod cli;
mod commands;
mod core;
mod downloader;
mod installer;
mod providers;
mod utils;

use cli::{Cli, Commands, SourceCommands};
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

    // Run the appropriate command
    let result = match cli.command {
        Commands::Init => commands::run_init(),

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
                SourceCommands::Update => commands::source::SourceCommand::Update,
                SourceCommands::List => commands::source::SourceCommand::List,
                SourceCommands::Info { names } => commands::source::SourceCommand::Info { names },
            };
            commands::run_source(source_cmd)
        }

        Commands::Add { names, yes } => commands::run_add(names, yes),

        Commands::List => commands::run_list(),

        Commands::Search { names } => commands::run_search(names),

        Commands::Update { names, yes } => commands::run_update(names, yes),

        Commands::Del { names, yes, force } => commands::run_delete(names, yes, force),
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
