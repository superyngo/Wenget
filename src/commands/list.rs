//! List command implementation

use crate::core::Config;
use anyhow::Result;
use colored::Colorize;

/// List installed packages
pub fn run() -> Result<()> {
    let config = Config::new()?;

    // Load installed manifest
    let manifest = config.get_or_create_installed()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages installed".yellow());
        println!("Install packages with: wenpm add <name>");
        return Ok(());
    }

    // Print header
    println!("{}", "Installed packages".bold());
    println!();
    println!(
        "{:<20} {:<15} {}",
        "NAME".bold(),
        "VERSION".bold(),
        "PLATFORM".bold()
    );
    println!("{}", "─".repeat(80));

    // Convert to sorted vector for consistent display
    let mut packages: Vec<_> = manifest.packages.iter().collect();
    packages.sort_by(|a, b| a.0.cmp(b.0));

    // Print packages
    for (name, pkg) in packages {
        println!("{:<20} {:<15} {}", name.green(), pkg.version, pkg.platform);
    }

    println!();
    println!("Total: {} package(s) installed", manifest.packages.len());

    Ok(())
}
