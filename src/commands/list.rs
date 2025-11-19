//! List command implementation

use crate::core::{Config, Platform};
use anyhow::Result;
use colored::Colorize;

/// List available packages for the current platform
pub fn run() -> Result<()> {
    let config = Config::new()?;

    // Load manifest
    let manifest = config.get_or_create_sources()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages in sources".yellow());
        println!("Add packages with: wenpm add <github-url>");
        return Ok(());
    }

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Filter packages that support current platform
    let compatible_packages: Vec<_> = manifest
        .packages
        .iter()
        .filter(|pkg| {
            platform_ids
                .iter()
                .any(|id| pkg.platforms.contains_key(id))
        })
        .collect();

    if compatible_packages.is_empty() {
        println!(
            "{}",
            format!("No packages available for {}", platform).yellow()
        );
        return Ok(());
    }

    // Print header
    println!("{}", format!("Available packages for {}", platform).bold());
    println!();
    println!(
        "{:<20} {:<12} {:<10} {}",
        "NAME".bold(),
        "VERSION".bold(),
        "SIZE".bold(),
        "DESCRIPTION".bold()
    );
    println!("{}", "─".repeat(80));

    // Print packages
    for pkg in &compatible_packages {
        // Find the first matching platform
        let platform_binary = platform_ids
            .iter()
            .find_map(|id| pkg.platforms.get(id))
            .unwrap();

        let size_mb = platform_binary.size as f64 / 1_000_000.0;

        println!(
            "{:<20} {:<12} {:>8.1} MB  {}",
            pkg.name.green(),
            pkg.latest,
            size_mb,
            truncate(&pkg.description, 40)
        );
    }

    println!();
    println!(
        "Total: {} package(s)",
        compatible_packages.len()
    );

    Ok(())
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
