//! Update command implementation

use crate::core::Config;
use crate::providers::{GitHubProvider, SourceProvider};
use anyhow::Result;
use colored::Colorize;

/// Update package metadata from sources
pub fn run() -> Result<()> {
    let config = Config::new()?;

    // Load manifest
    let mut manifest = config.get_or_create_sources()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages in sources to update".yellow());
        println!("Add packages with: wenpm add <github-url>");
        return Ok(());
    }

    println!("{} {} package(s)...\n", "Updating".cyan(), manifest.packages.len());

    // Create provider
    let github = GitHubProvider::new()?;

    // Track results
    let mut updated = 0;
    let mut unchanged = 0;
    let mut failed = 0;

    // Update each package
    for pkg in &mut manifest.packages {
        print!("  {} {} ... ", "Updating".cyan(), pkg.name);

        match github.fetch_package(&pkg.repo) {
            Ok(new_pkg) => {
                if new_pkg.latest != pkg.latest {
                    println!(
                        "{} {} -> {}",
                        "Updated".green(),
                        pkg.latest,
                        new_pkg.latest
                    );
                    *pkg = new_pkg;
                    updated += 1;
                } else {
                    println!("{} v{}", "Up to date".green(), pkg.latest);
                    unchanged += 1;
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
    if updated > 0 {
        println!("  {} {} package(s) updated", "✓".green(), updated);
    }
    if unchanged > 0 {
        println!("  {} {} package(s) unchanged", "•".cyan(), unchanged);
    }
    if failed > 0 {
        println!("  {} {} package(s) failed", "✗".red(), failed);
    }

    Ok(())
}
