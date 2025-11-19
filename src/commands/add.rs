//! Add command implementation

use crate::core::Config;
use crate::providers::{GitHubProvider, SourceProvider};
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

/// Add packages from GitHub URLs
pub fn run(urls: Vec<String>, source_file: Option<String>) -> Result<()> {
    let config = Config::new()?;

    // Ensure WenPM is initialized
    if !config.is_initialized() {
        config.init()?;
    }

    // Collect all URLs
    let mut all_urls = urls;

    // Load URLs from source file if provided
    if let Some(source_path) = source_file {
        println!("{} {}", "Reading URLs from:".cyan(), source_path);
        let content = if source_path.starts_with("http://") || source_path.starts_with("https://")
        {
            // Fetch from URL
            crate::utils::HttpClient::new()?
                .get_text(&source_path)
                .context("Failed to fetch source file from URL")?
        } else {
            // Read from local file
            fs::read_to_string(&source_path)
                .with_context(|| format!("Failed to read source file: {}", source_path))?
        };

        // Parse URLs (one per line, skip empty lines and comments)
        let file_urls: Vec<String> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect();

        println!("  Found {} URL(s)", file_urls.len());
        all_urls.extend(file_urls);
    }

    if all_urls.is_empty() {
        println!("{}", "No URLs provided".yellow());
        println!("Usage: wenpm add <url>... [--source <file>]");
        return Ok(());
    }

    println!(
        "{} {} package(s)...\n",
        "Adding".cyan(),
        all_urls.len()
    );

    // Load existing manifest
    let mut manifest = config.get_or_create_sources()?;

    // Create provider
    let github = GitHubProvider::new()?;

    // Track results
    let mut added = 0;
    let mut updated = 0;
    let mut failed = 0;

    // Process each URL
    for url in all_urls {
        print!("  {} {} ... ", "Processing".cyan(), url);

        match process_url(&github, &url) {
            Ok(package) => {
                let name = package.name.clone();
                let version = package.latest.clone();
                let platform_count = package.platforms.len();

                // Check if package already exists
                let is_update = manifest.find_package(&name).is_some();

                // Add or update package
                manifest.upsert_package(package);

                if is_update {
                    println!(
                        "{} {} v{} ({} platforms)",
                        "Updated".green(),
                        name,
                        version,
                        platform_count
                    );
                    updated += 1;
                } else {
                    println!(
                        "{} {} v{} ({} platforms)",
                        "Added".green(),
                        name,
                        version,
                        platform_count
                    );
                    added += 1;
                }
            }
            Err(e) => {
                println!("{} {}", "Failed".red(), e);
                failed += 1;
            }
        }
    }

    // Save manifest
    config.save_sources(&manifest)?;

    // Summary
    println!();
    println!("{}", "Summary:".bold());
    if added > 0 {
        println!("  {} {} package(s) added", "✓".green(), added);
    }
    if updated > 0 {
        println!("  {} {} package(s) updated", "✓".green(), updated);
    }
    if failed > 0 {
        println!("  {} {} package(s) failed", "✗".red(), failed);
    }

    println!();
    println!("Total packages in sources: {}", manifest.packages.len());

    Ok(())
}

/// Process a single URL
fn process_url(provider: &GitHubProvider, url: &str) -> Result<crate::core::Package> {
    if !provider.can_handle(url) {
        anyhow::bail!("Unsupported URL (only GitHub is supported currently)");
    }

    provider.fetch_package(url)
}
