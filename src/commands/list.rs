//! List command implementation

use crate::core::manifest::PackageSource;
use crate::core::{Config, Platform};
use anyhow::Result;
use colored::Colorize;

/// List installed packages or all available packages
pub fn run(all: bool) -> Result<()> {
    let config = Config::new()?;

    if all {
        // Show all available packages from cache
        list_all_packages(&config)?;
    } else {
        // Show only installed packages
        list_installed_packages(&config)?;
    }

    Ok(())
}

/// List only installed packages
fn list_installed_packages(config: &Config) -> Result<()> {
    // Load installed manifest
    let manifest = config.get_or_create_installed()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages installed".yellow());
        println!("Install packages with: wenget add <name>");
        return Ok(());
    }

    // Print header
    println!("{}", "Installed packages".bold());
    println!();
    println!(
        "{:<20} {:<15} {:<10} {:<12} {}",
        "NAME".bold(),
        "COMMAND".bold(),
        "VERSION".bold(),
        "SOURCE".bold(),
        "DESCRIPTION".bold()
    );
    println!("{}", "─".repeat(100));

    // Convert to sorted vector for consistent display
    let mut packages: Vec<_> = manifest.packages.iter().collect();
    packages.sort_by(|a, b| a.0.cmp(b.0));

    // Print packages
    for (name, pkg) in packages {
        // Get source display
        let source_display = match &pkg.source {
            PackageSource::Bucket { name } => name.clone(),
            PackageSource::DirectRepo { .. } => "url".to_string(),
            PackageSource::Script { script_type, .. } => {
                format!("{}", script_type.display_name().to_lowercase())
            }
        };

        // Truncate description if too long
        let description = if pkg.description.len() > 30 {
            format!("{}...", &pkg.description[..27])
        } else {
            pkg.description.clone()
        };

        println!(
            "{:<20} {:<15} {:<10} {:<12} {}",
            name.green(),
            pkg.command_name.yellow(),
            pkg.version,
            source_display.cyan(),
            description
        );
    }

    println!();
    println!("Total: {} package(s) installed", manifest.packages.len());

    Ok(())
}

/// List all available packages from cache
fn list_all_packages(config: &Config) -> Result<()> {
    // Get packages from cache
    let manifest = config.get_packages_from_cache()?;

    // Load installed packages for marking
    let installed = config.get_or_create_installed()?;

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Filter packages that support current platform
    let mut packages: Vec<_> = manifest
        .packages
        .iter()
        .filter(|pkg| {
            platform_ids
                .iter()
                .any(|pid| pkg.platforms.contains_key(pid))
        })
        .collect();

    if packages.is_empty() {
        println!("{}", "No packages available in buckets".yellow());
        println!("Add a bucket with: wenget bucket add <name> <url>");
        return Ok(());
    }

    // Sort alphabetically
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    // Print header
    println!("{}", "Available packages".bold());
    println!();
    println!("{:<30} {}", "NAME".bold(), "DESCRIPTION".bold());
    println!("{}", "─".repeat(80));

    // Print packages
    for pkg in &packages {
        if installed.is_installed(&pkg.name) {
            // For installed packages, calculate padding manually to account for "(installed)"
            let name_width = 30;
            let installed_suffix = " (installed)";
            let visible_length = pkg.name.len() + installed_suffix.len();
            let padding = if visible_length < name_width {
                name_width - visible_length
            } else {
                1
            };

            let description = if pkg.description.len() > 48 {
                format!("{}...", &pkg.description[..45])
            } else {
                pkg.description.clone()
            };

            print!("{} {}", pkg.name, "(installed)".green());
            print!("{}", " ".repeat(padding));
            println!("{}", description);
        } else {
            let description = if pkg.description.len() > 48 {
                format!("{}...", &pkg.description[..45])
            } else {
                pkg.description.clone()
            };

            println!("{:<30} {}", pkg.name, description);
        }
    }

    println!();
    println!(
        "Total: {} package(s) available ({} installed)",
        packages.len(),
        installed.packages.len()
    );

    Ok(())
}
