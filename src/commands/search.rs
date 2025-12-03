//! Search command implementation

use crate::core::{Config, Platform};
use anyhow::Result;
use colored::Colorize;
use glob::Pattern;

/// Search for packages and scripts
pub fn run(patterns: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    // Load cache
    let cache = config.get_or_rebuild_cache()?;

    if cache.packages.is_empty() && cache.scripts.is_empty() {
        println!("{}", "No packages or scripts in sources".yellow());
        println!("Add buckets with: wenget bucket add <name> <url>");
        return Ok(());
    }

    if patterns.is_empty() {
        println!("{}", "No search pattern provided".yellow());
        println!("Usage: wenget search <name>...");
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
    let matching_packages: Vec<_> = cache
        .packages
        .values()
        .filter(|cached_pkg| {
            let pkg = &cached_pkg.package;
            // Check if name matches any pattern
            let name_matches = glob_patterns
                .iter()
                .any(|pattern| pattern.matches(&pkg.name));

            // Check if supports current platform
            let platform_matches = platform_ids.iter().any(|id| pkg.platforms.contains_key(id));

            name_matches && platform_matches
        })
        .collect();

    // Filter scripts
    let matching_scripts: Vec<_> = cache
        .scripts
        .values()
        .filter(|cached_script| {
            let script = &cached_script.script;
            // Check if name matches any pattern
            let name_matches = glob_patterns
                .iter()
                .any(|pattern| pattern.matches(&script.name));

            // Check if supports current platform
            let platform_matches = script.script_type.is_supported_on_current_platform();

            name_matches && platform_matches
        })
        .collect();

    if matching_packages.is_empty() && matching_scripts.is_empty() {
        println!(
            "{}",
            format!("No packages or scripts found matching: {:?}", patterns).yellow()
        );
        return Ok(());
    }

    // Print header
    println!("{}", format!("Search results for: {:?}", patterns).bold());
    println!();

    // Print packages
    if !matching_packages.is_empty() {
        println!("{}", "Binary Packages:".bold().cyan());
        println!(
            "{:<20} {:<10} {}",
            "NAME".bold(),
            "SIZE".bold(),
            "DESCRIPTION".bold()
        );
        println!("{}", "─".repeat(80));

        for cached_pkg in &matching_packages {
            let pkg = &cached_pkg.package;
            // Find the first matching platform
            let platform_binary = platform_ids
                .iter()
                .find_map(|id| pkg.platforms.get(id))
                .unwrap();

            let size_mb = platform_binary.size as f64 / 1_000_000.0;

            println!(
                "{:<20} {:>8.1} MB  {}",
                pkg.name.green(),
                size_mb,
                truncate(&pkg.description, 50)
            );
        }
        println!();
    }

    // Print scripts
    if !matching_scripts.is_empty() {
        println!("{}", "Scripts:".bold().cyan());
        println!(
            "{:<20} {:<10} {}",
            "NAME".bold(),
            "TYPE".bold(),
            "DESCRIPTION".bold()
        );
        println!("{}", "─".repeat(80));

        for cached_script in &matching_scripts {
            let script = &cached_script.script;
            let script_type = script.script_type.display_name();

            println!(
                "{:<20} {:<10} {}",
                script.name.green(),
                script_type.yellow(),
                truncate(&script.description, 50)
            );
        }
        println!();
    }

    println!(
        "Found: {} package(s), {} script(s)",
        matching_packages.len(),
        matching_scripts.len()
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
