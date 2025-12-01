//! Add (Install) command implementation

use crate::core::manifest::PackageSource;
use crate::core::{Config, InstalledPackage, Platform, WenPaths};
use crate::downloader;
use crate::installer::{create_shim, extract_archive, find_executable};
use crate::package_resolver::{PackageInput, PackageResolver, ResolvedPackage};
use crate::providers::{GitHubProvider, SourceProvider};
use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use std::fs;

#[cfg(unix)]
use crate::installer::create_symlink;

/// Install packages (smart detection: package names from cache or GitHub URLs)
pub fn run(names: Vec<String>, yes: bool) -> Result<()> {
    let config = Config::new()?;
    let paths = WenPaths::new()?;

    // Ensure initialized
    if !config.is_initialized() {
        config.init()?;
    }

    let mut installed = config.get_or_create_installed()?;

    if names.is_empty() {
        println!("{}", "No package names or URLs provided".yellow());
        println!("Usage: wenget add <name|url>...");
        println!();
        println!("Examples:");
        println!("  wenget add ripgrep              # Install from cache");
        println!("  wenget add 'rip*'               # Install matching packages (glob)");
        println!("  wenget add https://github.com/BurntSushi/ripgrep  # Install from URL");
        return Ok(());
    }

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Resolve all inputs and collect packages to install
    let resolver = PackageResolver::new(Config::new()?)?;
    let mut packages_to_install: Vec<ResolvedPackage> = Vec::new();

    for name in &names {
        let input = PackageInput::parse(name);

        match resolver.resolve(&input) {
            Ok(resolved) => {
                for pkg_resolved in resolved {
                    // Check platform support
                    let platform_matches = platform_ids
                        .iter()
                        .any(|id| pkg_resolved.package.platforms.contains_key(id));

                    if !platform_matches {
                        println!(
                            "{} {} does not support current platform",
                            "Warning:".yellow(),
                            pkg_resolved.package.name
                        );
                        continue;
                    }

                    packages_to_install.push(pkg_resolved);
                }
            }
            Err(e) => {
                eprintln!("{} {}: {}", "Error".red().bold(), name, e);
            }
        }
    }

    if packages_to_install.is_empty() {
        println!("{}", "No packages to install".yellow());
        return Ok(());
    }

    // Create GitHub provider to fetch versions
    let github = GitHubProvider::new()?;

    // Show packages to install with versions and handle already-installed packages
    println!("{}", "Packages to install:".bold());

    let mut to_install: Vec<ResolvedPackage> = Vec::new();
    let mut to_update: Vec<ResolvedPackage> = Vec::new();

    for resolved in packages_to_install {
        let pkg_name = &resolved.package.name;
        let repo = &resolved.package.repo;

        // Fetch latest version
        let version = github
            .fetch_latest_version(repo)
            .unwrap_or_else(|_| "unknown".to_string());

        if installed.is_installed(pkg_name) {
            // Package already installed
            let inst_pkg = installed.get_package(pkg_name).unwrap();
            if inst_pkg.version == version {
                println!(
                    "  {} {} v{} {}",
                    "•".cyan(),
                    pkg_name,
                    version,
                    "(already installed, same version)".dimmed()
                );
            } else {
                println!(
                    "  {} {} v{} {} → {}",
                    "•".yellow(),
                    pkg_name,
                    inst_pkg.version.dimmed(),
                    "upgrade to".yellow(),
                    version.green()
                );
                to_update.push(resolved);
            }
        } else {
            // New installation
            println!(
                "  {} {} v{} {}",
                "•".green(),
                pkg_name,
                version,
                "(new)".green()
            );
            to_install.push(resolved);
        }
    }

    // Check if there's anything to do
    if to_install.is_empty() && to_update.is_empty() {
        println!();
        println!("{}", "All packages are already up to date".green());
        return Ok(());
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

    // Install/update packages
    let mut success_count = 0;
    let mut fail_count = 0;

    // Combine new installs and updates
    let all_packages: Vec<_> = to_install.into_iter().chain(to_update).collect();

    // Collect packages to update in cache (packages fetched from GitHub API)
    let mut packages_to_cache: Vec<(crate::core::Package, PackageSource)> = Vec::new();

    for resolved in all_packages {
        let pkg_name = &resolved.package.name;
        let repo_url = &resolved.package.repo;

        // Try to fetch latest package info from GitHub API (includes latest download links)
        // If API rate limit is hit, fallback to cached package info
        let (pkg_to_install, version, using_fallback) = match github.fetch_package(repo_url) {
            Ok(latest_pkg) => {
                // Successfully fetched from GitHub API - use latest download links
                let version = github
                    .fetch_latest_version(repo_url)
                    .unwrap_or_else(|_| "unknown".to_string());
                (latest_pkg, version, false)
            }
            Err(e) => {
                // Failed to fetch from GitHub API (likely rate limit) - use cached package info
                log::warn!(
                    "Failed to fetch latest package info from GitHub API for {}: {}",
                    pkg_name,
                    e
                );
                println!(
                    "  {} Using cached download links (GitHub API unavailable)",
                    "⚠".yellow()
                );

                let version = github
                    .fetch_latest_version(repo_url)
                    .unwrap_or_else(|_| "unknown".to_string());
                (resolved.package.clone(), version, true)
            }
        };

        println!("{} {} v{}...", "Installing".cyan(), pkg_name, version);
        if using_fallback {
            println!(
                "  {} Falling back to bucket source download links",
                "ℹ".cyan()
            );
        }

        match install_package(
            &config,
            &paths,
            &pkg_to_install,
            &platform_ids,
            &version,
            &resolved.source,
        ) {
            Ok(inst_pkg) => {
                installed.upsert_package(pkg_name.clone(), inst_pkg);
                config.save_installed(&installed)?;

                // Collect package for cache update if fetched from GitHub API
                if !using_fallback {
                    packages_to_cache.push((pkg_to_install.clone(), resolved.source.clone()));
                }

                println!("  {} Installed successfully", "✓".green());
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {}", "✗".red(), e);
                fail_count += 1;
            }
        }
        println!();
    }

    // Update cache with latest package info from GitHub API
    if !packages_to_cache.is_empty() {
        match update_cache_with_packages(&config, packages_to_cache) {
            Ok(count) => {
                log::info!("Updated cache with {} latest package(s)", count);
            }
            Err(e) => {
                log::warn!("Failed to update cache: {}", e);
                // Don't fail the entire operation if cache update fails
            }
        }
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
    _config: &Config,
    paths: &WenPaths,
    pkg: &crate::core::Package,
    platform_ids: &[String],
    version: &str,
    source: &PackageSource,
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
        .next_back()
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
        version: version.to_string(),
        platform: platform_id.clone(),
        installed_at: Utc::now(),
        install_path: app_dir.to_string_lossy().to_string(),
        files: extracted_files,
        source: source.clone(),
        description: pkg.description.clone(),
    };

    Ok(inst_pkg)
}

/// Update manifest cache with latest package info from GitHub API
fn update_cache_with_packages(
    config: &Config,
    packages: Vec<(crate::core::Package, PackageSource)>,
) -> Result<usize> {
    // Load current cache
    let mut cache = config.get_or_rebuild_cache()?;

    // Save count before moving packages
    let count = packages.len();

    // Update cache with new package info
    for (package, source) in packages {
        log::debug!(
            "Updating cache with latest info for {} from GitHub API",
            package.name
        );
        cache.add_package(package, source);
    }

    // Save updated cache
    config.save_cache(&cache)?;

    Ok(count)
}
