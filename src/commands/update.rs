//! Update (Upgrade) command implementation

use crate::commands::add;
use crate::core::manifest::PackageSource;
use crate::core::Config;
use crate::providers::base::SourceProvider;
use crate::providers::GitHubProvider;
use anyhow::Result;
use colored::Colorize;

/// Compare two dot-separated version strings.
/// Returns true if `new` is strictly newer than `old`.
fn is_newer_version(old: &str, new: &str) -> bool {
    let parse_parts = |v: &str| {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect::<Vec<_>>()
    };
    let old_parts = parse_parts(old);
    let new_parts = parse_parts(new);
    for i in 0..old_parts.len().max(new_parts.len()) {
        let a = old_parts.get(i).unwrap_or(&0);
        let b = new_parts.get(i).unwrap_or(&0);
        if b > a {
            return true;
        }
        if b < a {
            return false;
        }
    }
    false
}

/// Upgrade installed packages
pub fn run(names: Vec<String>, yes: bool) -> Result<()> {
    // Check for wenget updates first
    if check_and_upgrade_self(yes)? {
        // On Windows, exit after self-update to avoid shell instability
        return Ok(());
    }

    let config = Config::new()?;
    let installed = config.get_or_create_installed()?;

    if installed.packages.is_empty() {
        println!("{}", "No packages installed".yellow());
        return Ok(());
    }

    // Force refresh bucket cache to ensure we have latest versions
    println!("{}", "Refreshing bucket cache...".cyan());
    let cache = config.rebuild_cache()?;

    // Create GitHub provider to fetch latest versions
    let github = GitHubProvider::new()?;

    // Determine which packages to upgrade
    let to_upgrade: Vec<String> = if names.is_empty() || (names.len() == 1 && names[0] == "all") {
        // List upgradeable packages
        let upgradeable = find_upgradeable(&installed, &github, &cache, yes)?;

        if upgradeable.is_empty() {
            println!("{}", "All packages are up to date".green());
            return Ok(());
        }

        println!("{}", "Packages to upgrade:".bold());
        for (name, current, latest) in &upgradeable {
            println!("  • {} {} -> {}", name, current.yellow(), latest.green());
        }
        println!();

        upgradeable.into_iter().map(|(name, _, _)| name).collect()
    } else {
        names
    };

    // Expand: include all installed variants when upgrading a repo
    let mut expanded = Vec::new();
    for name in &to_upgrade {
        // Check if this is a repo name or a specific variant
        if name.contains("::") {
            // Specific variant like "bun::baseline" — validate it exists
            if installed.is_installed(name) {
                expanded.push(name.clone());
            } else {
                eprintln!(
                    "{} '{}' is not installed, skipping (use 'wenget add' to install new packages)",
                    "Warning:".yellow(),
                    name
                );
            }
            continue;
        }

        // Find all installed variants of this repo
        let variants = installed.find_by_repo(name);
        if variants.is_empty() {
            if installed.is_installed(name) {
                // Standalone package or script
                expanded.push(name.clone());
            } else {
                eprintln!(
                    "{} '{}' is not installed, skipping (use 'wenget add' to install new packages)",
                    "Warning:".yellow(),
                    name
                );
            }
        } else {
            for (key, _pkg) in variants {
                expanded.push(key.clone());
            }
        }
    }

    if expanded.is_empty() {
        println!("{}", "No installed packages to update".yellow());
        return Ok(());
    }

    // Use add command to upgrade (reinstall)
    add::run(expanded, yes, None, None, None, None, false, true)
}

/// Find upgradeable packages by checking their sources
fn find_upgradeable(
    installed: &crate::core::InstalledManifest,
    github: &GitHubProvider,
    cache: &crate::cache::ManifestCache,
    yes: bool,
) -> Result<Vec<(String, String, String)>> {
    let mut upgradeable = Vec::new();

    // Group packages by repo_name to check only once per repo
    let grouped = installed.group_by_repo();

    for (repo_name, variants) in grouped {
        // Use the first variant to get version and source info
        let (_key, inst_pkg) = variants[0];

        // Determine repo URL based on source
        let repo_url = match &inst_pkg.source {
            PackageSource::Bucket { name: bucket_name } => {
                // Get package info from cache for bucket packages
                // Find package in cache by repo_name
                let found = cache
                    .packages
                    .values()
                    .find(|cached_pkg| cached_pkg.package.name == repo_name);

                if let Some(cached_pkg) = found {
                    cached_pkg.package.repo.clone()
                } else {
                    eprintln!(
                        "{} Package {} not found in bucket {} cache, skipping update check",
                        "Warning:".yellow(),
                        repo_name,
                        bucket_name
                    );
                    continue;
                }
            }
            PackageSource::DirectRepo { url } => {
                // Use the stored repo URL directly
                url.clone()
            }
            PackageSource::Script { origin, .. } => {
                // Check if this is a bucket-sourced script
                if !origin.starts_with("bucket:") {
                    log::debug!(
                        "Skipping non-bucket script '{}' - no update source",
                        repo_name
                    );
                    continue;
                }

                // Look up the script in the refreshed cache
                let cached_script = cache.find_script(&repo_name);
                if cached_script.is_none() {
                    log::debug!("Script '{}' not found in cache, skipping update", repo_name);
                    continue;
                }

                let cached_script = cached_script.unwrap();

                // Get the installable script URL for current platform
                if let Some((_script_type, platform_info)) =
                    cached_script.script.get_installable_script()
                {
                    let cache_url = &platform_info.url;

                    // Compare with installed download_url
                    let needs_update = match &inst_pkg.download_url {
                        Some(installed_url) => installed_url != cache_url,
                        None => true, // No stored URL = always update (legacy installs)
                    };

                    if needs_update {
                        upgradeable.push((
                            repo_name.clone(),
                            inst_pkg.download_url.clone().unwrap_or_default(),
                            cache_url.clone(),
                        ));
                    }
                }

                continue;
            }
        };

        // Fetch latest version from GitHub
        match github.fetch_latest_version(&repo_url) {
            Ok(latest_version) => {
                if inst_pkg.version != latest_version {
                    upgradeable.push((repo_name.clone(), inst_pkg.version.clone(), latest_version));
                }
            }
            Err(e) => {
                // API failed - try to use cache version as fallback
                log::debug!(
                    "GitHub API failed for {}: {}. Trying cache fallback...",
                    repo_name,
                    e
                );

                // Find package in cache by repo_name
                if let Some(cached_pkg) = cache
                    .packages
                    .values()
                    .find(|p| p.package.name == repo_name)
                {
                    if let Some(cache_version) = &cached_pkg.package.version {
                        let should_upgrade = if inst_pkg.version == "local" {
                            if yes {
                                true
                            } else {
                                eprintln!(
                                    "{} {} is locally installed, cache has version {} available",
                                    "Info:".cyan(),
                                    repo_name,
                                    cache_version
                                );
                                crate::utils::prompt::confirm(&format!(
                                    "  Overwrite local {} with cached version {}?",
                                    repo_name, cache_version
                                ))?
                            }
                        } else {
                            is_newer_version(&inst_pkg.version, cache_version)
                        };

                        if should_upgrade {
                            eprintln!(
                                "{} Using cached version for {}: {} (API unavailable)",
                                "Info:".cyan(),
                                repo_name,
                                cache_version
                            );
                            upgradeable.push((
                                repo_name.clone(),
                                inst_pkg.version.clone(),
                                cache_version.clone(),
                            ));
                        }
                    } else {
                        eprintln!(
                            "{} No version info in cache for {}, skipping",
                            "Warning:".yellow(),
                            repo_name
                        );
                    }
                } else {
                    eprintln!(
                        "{} Failed to check updates for {}: API error and no cache available",
                        "Warning:".yellow(),
                        repo_name
                    );
                }
            }
        }
    }

    Ok(upgradeable)
}

/// Check for wenget updates and prompt user
/// Returns true if wenget was updated on Windows (caller should exit)
fn check_and_upgrade_self(yes: bool) -> Result<bool> {
    let current_version = env!("CARGO_PKG_VERSION");

    println!("{}", "Checking for wenget updates...".dimmed());

    // Try to check latest version - don't fail the whole update if this fails
    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Failed to create GitHub provider for self-check: {}", e);
            return Ok(false);
        }
    };

    let latest_version = match provider.fetch_latest_version("https://github.com/superyngo/wenget")
    {
        Ok(v) => v,
        Err(e) => {
            log::debug!("Failed to check wenget updates: {}", e);
            return Ok(false);
        }
    };

    if current_version == latest_version {
        return Ok(false);
    }

    println!(
        "{} {} -> {}",
        "New wenget version available:".yellow().bold(),
        current_version.yellow(),
        latest_version.green()
    );

    let should_update = if yes {
        true
    } else {
        crate::utils::confirm("Update wenget first?")?
    };

    if !should_update {
        println!();
        return Ok(false);
    }

    // Perform self-update, passing provider and known version to avoid redundant API calls
    upgrade_self_with_provider(provider, &latest_version)?;

    // On Windows, recommend restarting shell
    #[cfg(windows)]
    {
        println!();
        println!(
            "{}",
            "⚠  Please restart your shell, then run 'wenget update' again to update packages."
                .yellow()
                .bold()
        );
        return Ok(true); // Signal caller to exit
    }

    #[cfg(not(windows))]
    {
        println!();
        Ok(false) // Continue with package updates on Unix
    }
}

/// Upgrade wenget itself
fn upgrade_self_with_provider(provider: GitHubProvider, latest_version: &str) -> Result<()> {
    use crate::core::{Platform, WenPaths};
    use crate::downloader::download_file;
    use crate::installer::{extract_archive, find_executable};
    use colored::Colorize;
    use std::env;
    use std::fs;

    println!("{}", "Upgrading wenget...".cyan());

    // Get package information including binaries
    let package = provider.fetch_package("https://github.com/superyngo/wenget")?;

    // Select binary for current platform
    // Note: Uses same platform matching logic as add command (see add.rs:592)
    // This handles libc detection (musl vs glibc), compiler variants, and fallbacks
    let current_platform = Platform::current();
    let matches = current_platform.find_best_match(&package.platforms);

    if matches.is_empty() {
        anyhow::bail!(
            "No binary available for platform: {}. Available platforms: {}",
            current_platform,
            package
                .platforms
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let best_match = &matches[0];
    let binaries = &package.platforms[&best_match.platform_id];

    // Show fallback information if using compatible binary
    if let Some(fallback_type) = &best_match.fallback_type {
        println!(
            "  {} Using compatible binary: {} ({})",
            "ℹ".cyan(),
            best_match.platform_id,
            fallback_type.description()
        );
    }

    // For self-update, just use the first binary if multiple exist
    let binary = binaries
        .first()
        .ok_or_else(|| anyhow::anyhow!("No binaries found for platform"))?;

    println!("Downloading: {}", binary.url);

    // Determine download file name from URL
    let filename = binary
        .url
        .rsplit('/')
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid download URL"))?;

    // Download to temporary directory
    let paths = WenPaths::new()?;
    let temp_dir = paths.cache_dir().join("self-upgrade");
    fs::create_dir_all(&temp_dir)?;

    let download_path = temp_dir.join(filename);
    download_file(&binary.url, &download_path)?;

    // Extract archive
    let extract_dir = temp_dir.join("extracted");
    fs::create_dir_all(&extract_dir)?;

    println!("{}", "Extracting...".cyan());
    let extracted_files = extract_archive(&download_path, &extract_dir)?;

    // Find the wenget executable
    let exe_relative_path = find_executable(&extracted_files, "wenget")
        .ok_or_else(|| anyhow::anyhow!("Could not find wenget executable in archive"))?;

    let new_exe_path = extract_dir.join(&exe_relative_path);

    if !new_exe_path.exists() {
        anyhow::bail!("Extracted executable not found: {}", new_exe_path.display());
    }

    // Get current executable path
    let current_exe = env::current_exe()?;

    println!("{}", "Installing new version...".cyan());

    // Platform-specific replacement logic
    #[cfg(windows)]
    {
        replace_exe_windows(&current_exe, &new_exe_path)?;
    }

    #[cfg(not(windows))]
    {
        replace_exe_unix(&current_exe, &new_exe_path)?;
    }

    // Clean up temporary files
    if let Err(e) = fs::remove_dir_all(&temp_dir) {
        log::warn!(
            "Failed to cleanup temp directory: {}: {}",
            temp_dir.display(),
            e
        );
    }

    println!();
    println!(
        "{}",
        format!("✓ Successfully upgraded to v{}!", latest_version).green()
    );
    println!("Please restart your terminal or run 'wenget --version' to verify.");

    Ok(())
}

/// Replace executable on Windows
///
/// Windows locks running executables, so we use a multi-step process:
/// 1. Rename current exe to .old
/// 2. Copy new exe to original location
/// 3. Create a cleanup script to delete .old file
#[cfg(windows)]
fn replace_exe_windows(
    current_exe: &std::path::PathBuf,
    new_exe: &std::path::PathBuf,
) -> Result<()> {
    use std::fs;
    use std::process::Command;

    let old_exe = current_exe.with_extension("exe.old");

    // Rename current executable
    if old_exe.exists() {
        fs::remove_file(&old_exe)?;
    }
    fs::rename(current_exe, &old_exe)?;

    // Copy new executable to the original location
    fs::copy(new_exe, current_exe)?;

    // Create cleanup script
    let cleanup_script = current_exe.parent().unwrap().join("wenget_cleanup.cmd");

    let script_content = format!(
        r#"@echo off
timeout /t 2 /nobreak >nul
del /f /q "{}"
del /f /q "%~f0"
"#,
        old_exe.display()
    );

    fs::write(&cleanup_script, script_content)?;

    // Start cleanup script in background
    let _ = Command::new("cmd")
        .args(["/C", "start", "/B", cleanup_script.to_str().unwrap()])
        .spawn();

    Ok(())
}

/// Replace executable on Unix (Linux/macOS)
///
/// This function uses a robust strategy to replace the running executable:
/// 1. The new executable is made executable (`chmod 755`).
/// 2. The current running executable is renamed to `*.old`.
/// 3. An atomic `fs::rename` is attempted to move the new executable into place.
/// 4. If `rename` fails (e.g., cross-device link), it falls back to `fs::copy`.
/// 5. The `*.old` file is removed on a best-effort basis.
#[cfg(not(windows))]
fn replace_exe_unix(current_exe: &std::path::PathBuf, new_exe: &std::path::PathBuf) -> Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Set permissions on the new executable before doing anything else.
    fs::set_permissions(new_exe, fs::Permissions::from_mode(0o755))?;

    let old_exe = current_exe.with_extension("old");

    // Remove any existing .old file to avoid confusion.
    if old_exe.exists() {
        if let Err(e) = fs::remove_file(&old_exe) {
            log::warn!(
                "Failed to remove old executable: {}: {}",
                old_exe.display(),
                e
            );
        }
    }

    // 1. Rename the currently running executable.
    if let Err(e) = fs::rename(current_exe, &old_exe) {
        return Err(anyhow::anyhow!(
            "Failed to rename running executable: {}. Try running with sudo.",
            e
        ));
    }

    // 2. Move the new executable into place. Try atomic rename first.
    if let Err(rename_err) = fs::rename(new_exe, current_exe) {
        // Rename failed, likely a cross-device link error (EXDEV). Fall back to copying.
        log::warn!(
            "Atomic rename failed: {}. Falling back to copy.",
            rename_err
        );
        match fs::copy(new_exe, current_exe) {
            Ok(_) => {
                // Permissions may not be preserved by `copy`, so set them again.
                fs::set_permissions(current_exe, fs::Permissions::from_mode(0o755))?;
            }
            Err(copy_err) => {
                // Copy failed. Try to restore the original executable.
                log::error!("Failed to copy new executable: {}", copy_err);
                if let Err(restore_err) = fs::rename(&old_exe, current_exe) {
                    log::error!(
                        "CRITICAL: Failed to restore original executable: {}",
                        restore_err
                    );
                }
                return Err(copy_err.into());
            }
        }
    }

    // 3. Clean up the old executable (best-effort).
    if let Err(e) = fs::remove_file(&old_exe) {
        log::warn!(
            "Failed to remove old executable: {}. It can be removed manually.",
            e
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.0", "2.0.0"));
        assert!(is_newer_version("1.73.2", "1.73.3"));
        assert!(is_newer_version("0.10.12", "0.11.2"));
        assert!(is_newer_version("0.9.0", "0.14.1"));
        assert!(is_newer_version("1.0", "1.0.1"));
        assert!(is_newer_version("1.3.0", "1.3.6"));

        // Not newer — same version
        assert!(!is_newer_version("1.0.0", "1.0.0"));

        // Not newer — cache is older (the bug case)
        assert!(!is_newer_version("1.73.3", "1.73.2"));
        assert!(!is_newer_version("2.61.0", "2.60.0"));
        assert!(!is_newer_version("0.14.1", "0.9.0"));
        assert!(!is_newer_version("2.89.0", "2.88.1"));

        // Handles v prefix
        assert!(is_newer_version("1.0.0", "v2.0.0"));
        assert!(!is_newer_version("v2.0.0", "1.0.0"));
    }
}
