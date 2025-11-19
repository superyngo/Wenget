//! Delete command implementation

use crate::core::{Config, WenPaths};
use anyhow::Result;
use colored::Colorize;
use glob::Pattern;
use std::fs;

/// Delete installed packages
pub fn run(names: Vec<String>, yes: bool, force: bool) -> Result<()> {
    let config = Config::new()?;
    let paths = WenPaths::new()?;

    // Load installed manifest
    let mut installed = config.get_or_create_installed()?;

    if installed.packages.is_empty() {
        println!("{}", "No packages installed".yellow());
        return Ok(());
    }

    if names.is_empty() {
        println!("{}", "No package names provided".yellow());
        println!("Usage: wenpm delete <name>...");
        return Ok(());
    }

    // Compile glob patterns
    let glob_patterns: Vec<Pattern> = names
        .iter()
        .map(|p| Pattern::new(p))
        .collect::<Result<_, _>>()?;

    // Find matching packages
    let matching_packages: Vec<String> = installed
        .packages
        .keys()
        .filter(|name| glob_patterns.iter().any(|pattern| pattern.matches(name)))
        .cloned()
        .collect();

    if matching_packages.is_empty() {
        println!(
            "{}",
            format!("No installed packages found matching: {:?}", names).yellow()
        );
        return Ok(());
    }

    // Check for wenpm self-deletion
    if matching_packages.contains(&"wenpm".to_string()) && !force {
        println!("{}", "Cannot delete wenpm itself".red());
        println!("Use --force if you really want to delete it");
        return Ok(());
    }

    // Show packages to delete
    println!("{}", "Packages to delete:".bold());
    for name in &matching_packages {
        let pkg = installed.get_package(name).unwrap();
        println!("  • {} v{}", name.red(), pkg.version);
    }

    // Confirm deletion
    if !yes {
        print!("\nProceed with deletion? [y/N] ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        let response = response.trim().to_lowercase();

        if response != "y" && response != "yes" {
            println!("Deletion cancelled");
            return Ok(());
        }
    }

    println!();

    // Delete each package
    let mut success_count = 0;
    let mut fail_count = 0;

    for name in matching_packages {
        println!("{} {}...", "Deleting".cyan(), name);

        match delete_package(&config, &paths, &mut installed, &name) {
            Ok(()) => {
                println!("  {} Deleted successfully", "✓".green());
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {}", "✗".red(), e);
                fail_count += 1;
            }
        }
    }

    // Save updated manifest
    config.save_installed(&installed)?;

    // Summary
    println!();
    println!("{}", "Summary:".bold());
    if success_count > 0 {
        println!("  {} {} package(s) deleted", "✓".green(), success_count);
    }
    if fail_count > 0 {
        println!("  {} {} package(s) failed", "✗".red(), fail_count);
    }

    Ok(())
}

/// Delete a single package
fn delete_package(
    _config: &Config,
    paths: &WenPaths,
    installed: &mut crate::core::InstalledManifest,
    name: &str,
) -> Result<()> {
    // Remove app directory
    let app_dir = paths.app_dir(name);
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir)?;
    }

    // Remove symlink/shim
    let bin_path = paths.bin_shim_path(name);
    if bin_path.exists() {
        fs::remove_file(&bin_path)?;
    }

    // Remove from installed manifest
    installed.remove_package(name);

    Ok(())
}
