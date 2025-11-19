//! WenPM - A cross-platform package manager for GitHub binaries

mod cli;
mod commands;
mod core;
mod downloader;
mod installer;
mod providers;
mod utils;

use cli::{Cli, Commands};
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

        Commands::Add { urls, source } => commands::run_add(urls, source),

        Commands::List => commands::run_list(),

        Commands::Search { names } => commands::run_search(names),

        Commands::Info { names } => commands::run_info(names),

        Commands::Update => commands::run_update(),

        Commands::Install { names, yes } => commands::run_install(names, yes),

        Commands::Upgrade { names, yes } => commands::run_upgrade(names, yes),

        Commands::Delete { names, yes, force } => commands::run_delete(names, yes, force),

        Commands::SetupPath { .. } => {
            eprintln!("{}", "Command not yet implemented".yellow());
            eprintln!("This will be available in Phase 2");
            Ok(())
        }
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
