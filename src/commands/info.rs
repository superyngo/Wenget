//! Info command implementation

use crate::core::{Config, Platform};
use anyhow::Result;
use colored::Colorize;
use glob::Pattern;

/// Show package information
pub fn run(patterns: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    // Load manifests
    let sources = config.get_or_create_sources()?;
    let installed = config.get_or_create_installed()?;

    if sources.packages.is_empty() {
        println!("{}", "No packages in sources".yellow());
        println!("Add packages with: wenpm add <github-url>");
        return Ok(());
    }

    if patterns.is_empty() {
        println!("{}", "No package name provided".yellow());
        println!("Usage: wenpm info <name>...");
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
    let matching_packages: Vec<_> = sources
        .packages
        .iter()
        .filter(|pkg| glob_patterns.iter().any(|pattern| pattern.matches(&pkg.name)))
        .collect();

    if matching_packages.is_empty() {
        println!(
            "{}",
            format!("No packages found matching: {:?}", patterns).yellow()
        );
        return Ok(());
    }

    // Display info for each package
    for (i, pkg) in matching_packages.iter().enumerate() {
        if i > 0 {
            println!();
            println!("{}", "─".repeat(80));
            println!();
        }

        println!("{}: {}", "Name".bold(), pkg.name.green());
        println!("{}: {}", "Description".bold(), pkg.description);
        println!("{}: {}", "Repository".bold(), pkg.repo);

        if let Some(homepage) = &pkg.homepage {
            println!("{}: {}", "Homepage".bold(), homepage);
        }

        if let Some(license) = &pkg.license {
            println!("{}: {}", "License".bold(), license);
        }

        println!("{}: {}", "Latest Version".bold(), pkg.latest);

        // Check if installed
        if let Some(inst_pkg) = installed.get_package(&pkg.name) {
            println!(
                "{}: {} ({})",
                "Installed Version".bold(),
                inst_pkg.version.green(),
                if inst_pkg.version == pkg.latest {
                    "up to date".green()
                } else {
                    "upgrade available".yellow()
                }
            );
            println!("{}: {}", "Installed At".bold(), inst_pkg.installed_at);
            println!("{}: {}", "Install Path".bold(), inst_pkg.install_path);
        } else {
            println!("{}: {}", "Installed".bold(), "No".red());
        }

        println!();
        println!("{}", "Supported Platforms:".bold());

        for (platform_id, binary) in &pkg.platforms {
            let size_mb = binary.size as f64 / 1_000_000.0;
            let is_current = platform_ids.contains(platform_id);

            let marker = if is_current {
                format!(" {}", "(current)".cyan())
            } else {
                String::new()
            };

            println!("  • {} - {:.1} MB{}", platform_id, size_mb, marker);
        }
    }

    Ok(())
}
