//! Search command implementation

use crate::core::{Config, Platform};
use anyhow::Result;
use colored::Colorize;
use glob::Pattern;

/// Search for packages
pub fn run(patterns: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    // Load manifest
    let manifest = config.get_or_create_sources()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages in sources".yellow());
        println!("Add packages with: wenpm add <github-url>");
        return Ok(());
    }

    if patterns.is_empty() {
        println!("{}", "No search pattern provided".yellow());
        println!("Usage: wenpm search <name>...");
        return Ok(());
    }

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Compile glob patterns
    let glob_patterns: Vec<Pattern> = patterns
        .iter()
        .map(|p| Pattern::new(p))
        .collect::<Result<_, _>>()?;

    // Filter packages
    let matching_packages: Vec<_> = manifest
        .packages
        .iter()
        .filter(|pkg| {
            // Check if name matches any pattern
            let name_matches = glob_patterns.iter().any(|pattern| pattern.matches(&pkg.name));

            // Check if supports current platform
            let platform_matches = platform_ids
                .iter()
                .any(|id| pkg.platforms.contains_key(id));

            name_matches && platform_matches
        })
        .collect();

    if matching_packages.is_empty() {
        println!(
            "{}",
            format!("No packages found matching: {:?}", patterns).yellow()
        );
        return Ok(());
    }

    // Print header
    println!(
        "{}",
        format!("Search results for: {:?}", patterns).bold()
    );
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
    for pkg in &matching_packages {
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
    println!("Found: {} package(s)", matching_packages.len());

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
