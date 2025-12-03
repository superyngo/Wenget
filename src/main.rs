//! Wenget - A cross-platform package manager for GitHub binaries

mod bucket;
mod cache;
mod cli;
mod commands;
mod core;
mod downloader;
mod installer;
mod package_resolver;
mod providers;
mod utils;

use clap::CommandFactory;
use cli::{BucketCommands, Cli, Commands};
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

        Commands::Add { names, yes, script_name } => commands::run_add(names, yes, script_name),

        Commands::List { all } => commands::run_list(all),

        Commands::Info { names } => commands::run_info(names),

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
