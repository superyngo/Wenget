//! Install command implementation

use crate::core::{Config, InstalledPackage, Platform, WenPaths};
use crate::downloader;
use crate::installer::{create_shim, create_symlink, extract_archive, find_executable};
use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use glob::Pattern;
use std::fs;

/// Install packages
pub fn run(names: Vec<String>, yes: bool) -> Result<()> {
    let config = Config::new()?;
    let paths = WenPaths::new()?;

    // Ensure initialized
    if !config.is_initialized() {
        config.init()?;
    }

    // Load manifests
    let sources = config.get_or_create_sources()?;
    let mut installed = config.get_or_create_installed()?;

    if names.is_empty() {
        println!("{}", "No package names provided".yellow());
        println!("Usage: wenpm install <name>...");
        return Ok(());
    }

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Compile glob patterns
    let glob_patterns: Vec<Pattern> = names
        .iter()
        .map(|p| Pattern::new(p))
        .collect::<Result<_, _>>()?;

    // Find matching packages
    let matching_packages: Vec<_> = sources
        .packages
        .iter()
        .filter(|pkg| {
            // Check if name matches
            let name_matches = glob_patterns.iter().any(|pattern| pattern.matches(&pkg.name));

            // Check if platform is supported
            let platform_matches = platform_ids
                .iter()
                .any(|id| pkg.platforms.contains_key(id));

            name_matches && platform_matches
        })
        .collect();

    if matching_packages.is_empty() {
        println!(
            "{}",
            format!("No matching packages found for: {:?}", names).yellow()
        );
        return Ok(());
    }

    // Show packages to install
    println!("{}", "Packages to install:".bold());
    for pkg in &matching_packages {
        let already_installed = installed.is_installed(&pkg.name);
        let status = if already_installed {
            format!("(reinstall)").yellow()
        } else {
            format!("(new)").green()
        };

        println!("  • {} v{} {}", pkg.name, pkg.latest, status);
    }

    // Confirm installation
    if !yes {
        print!("\nProceed with installation? [Y/n] ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        let response = response.trim().to_lowercase();

        if !response.is_empty() && response != "y" && response != "yes" {
            println!("Installation cancelled");
            return Ok(());
        }
    }

    println!();

    // Install each package
    let mut success_count = 0;
    let mut fail_count = 0;

    for pkg in matching_packages {
        println!("{} {} v{}...", "Installing".cyan(), pkg.name, pkg.latest);

        match install_package(&config, &paths, pkg, &platform_ids) {
            Ok(inst_pkg) => {
                installed.upsert_package(pkg.name.clone(), inst_pkg);
                config.save_installed(&installed)?;

                println!("  {} {}", "✓".green(), "Installed successfully");
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {}", "✗".red(), e);
                fail_count += 1;
            }
        }
        println!();
    }

    // Summary
    println!("{}", "Summary:".bold());
    if success_count > 0 {
        println!("  {} {} package(s) installed", "✓".green(), success_count);
    }
    if fail_count > 0 {
        println!("  {} {} package(s) failed", "✗".red(), fail_count);
    }

    Ok(())
}

/// Install a single package
fn install_package(
    config: &Config,
    paths: &WenPaths,
    pkg: &crate::core::Package,
    platform_ids: &[String],
) -> Result<InstalledPackage> {
    // Find platform binary
    let (platform_id, binary) = platform_ids
        .iter()
        .find_map(|id| pkg.platforms.get(id).map(|b| (id, b)))
        .context("No binary found for current platform")?;

    // Download binary
    println!("  Downloading from {}...", binary.url);

    let download_dir = paths.downloads_dir();
    fs::create_dir_all(&download_dir)?;

    // Determine file extension from URL
    let filename = binary
        .url
        .split('/')
        .last()
        .context("Invalid download URL")?;

    let download_path = download_dir.join(filename);

    downloader::download_file(&binary.url, &download_path)?;

    // Extract to app directory
    let app_dir = paths.app_dir(&pkg.name);

    println!("  Extracting to {}...", app_dir.display());

    // Remove existing installation
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir)?;
    }

    let extracted_files = extract_archive(&download_path, &app_dir)?;

    // Find executable
    let exe_relative = find_executable(&extracted_files, &pkg.name)
        .context("Failed to find executable in archive")?;

    let exe_path = app_dir.join(&exe_relative);

    if !exe_path.exists() {
        anyhow::bail!("Executable not found: {}", exe_path.display());
    }

    println!("  Found executable: {}", exe_relative);

    // Create symlink/shim
    let bin_path = paths.bin_shim_path(&pkg.name);

    println!("  Creating launcher at {}...", bin_path.display());

    #[cfg(unix)]
    {
        create_symlink(&exe_path, &bin_path)?;
    }

    #[cfg(windows)]
    {
        create_shim(&exe_path, &bin_path, &pkg.name)?;
    }

    // Clean up download
    fs::remove_file(&download_path)?;

    // Create installed package info
    let inst_pkg = InstalledPackage {
        version: pkg.latest.clone(),
        platform: platform_id.clone(),
        installed_at: Utc::now(),
        install_path: app_dir.to_string_lossy().to_string(),
        files: extracted_files,
    };

    Ok(inst_pkg)
}
