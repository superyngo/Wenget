//! Update (Upgrade) command implementation

use crate::commands::add;
use crate::core::manifest::PackageSource;
use crate::core::Config;
use crate::providers::base::SourceProvider;
use crate::providers::GitHubProvider;
use anyhow::Result;
use colored::Colorize;

/// Upgrade installed packages
pub fn run(names: Vec<String>, yes: bool) -> Result<()> {
    // Handle "wenget update self"
    if names.len() == 1 && names[0] == "self" {
        return upgrade_self();
    }

    let config = Config::new()?;
    let installed = config.get_or_create_installed()?;

    if installed.packages.is_empty() {
        println!("{}", "No packages installed".yellow());
        return Ok(());
    }

    // Create GitHub provider to fetch latest versions
    let github = GitHubProvider::new()?;

    // Determine which packages to upgrade
    let to_upgrade: Vec<String> = if names.is_empty() || (names.len() == 1 && names[0] == "all") {
        // List upgradeable packages
        let upgradeable = find_upgradeable(&config, &installed, &github)?;

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

    // Use add command to upgrade (reinstall)
    add::run(to_upgrade, yes, None, None)
}

/// Find upgradeable packages by checking their sources
fn find_upgradeable(
    config: &Config,
    installed: &crate::core::InstalledManifest,
    github: &GitHubProvider,
) -> Result<Vec<(String, String, String)>> {
    let mut upgradeable = Vec::new();

    for (name, inst_pkg) in &installed.packages {
        // Determine repo URL based on source
        let repo_url = match &inst_pkg.source {
            PackageSource::Bucket { name: bucket_name } => {
                // Get package info from cache for bucket packages
                let cache = config.get_or_rebuild_cache()?;

                // Find package in cache by name (cache is keyed by URL, not name)
                let found = cache
                    .packages
                    .values()
                    .find(|cached_pkg| cached_pkg.package.name == *name);

                if let Some(cached_pkg) = found {
                    cached_pkg.package.repo.clone()
                } else {
                    eprintln!(
                        "{} Package {} not found in bucket {} cache, skipping update check",
                        "Warning:".yellow(),
                        name,
                        bucket_name
                    );
                    continue;
                }
            }
            PackageSource::DirectRepo { url } => {
                // Use the stored repo URL directly
                url.clone()
            }
            PackageSource::Script { .. } => {
                // Scripts don't support updates
                log::debug!("Skipping script '{}' - scripts don't support updates", name);
                continue;
            }
        };

        // Fetch latest version from GitHub
        if let Ok(latest_version) = github.fetch_latest_version(&repo_url) {
            if inst_pkg.version != latest_version {
                upgradeable.push((name.clone(), inst_pkg.version.clone(), latest_version));
            }
        }
    }

    Ok(upgradeable)
}

/// Upgrade wenget itself
fn upgrade_self() -> Result<()> {
    use crate::core::platform::Os;
    use crate::core::{Platform, WenPaths};
    use crate::downloader::download_file;
    use crate::installer::{extract_archive, find_executable};
    use colored::Colorize;
    use std::env;
    use std::fs;

    println!("{}", "Upgrading wenget...".cyan());

    // Get current version
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    // Fetch latest package info from GitHub
    let provider = GitHubProvider::new()?;
    let latest_version = provider.fetch_latest_version("https://github.com/superyngo/wenget")?;

    println!("Latest version: {}", latest_version);

    if current_version == latest_version {
        println!("{}", "✓ Already up to date".green());
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "New version available: {} -> {}",
            current_version, latest_version
        )
        .yellow()
    );
    println!();

    // Get package information including binaries
    let package = provider.fetch_package("https://github.com/superyngo/wenget")?;

    // Select binary for current platform
    let current_platform = Platform::current();
    let platform_id = current_platform.to_string();

    let binary = package
        .platforms
        .get(&platform_id)
        .or_else(|| {
            // Try with musl variant for Linux
            if current_platform.os == Os::Linux {
                package.platforms.get(&format!("{}-musl", platform_id))
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("No binary available for platform: {}", platform_id))?;

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
        log::warn!("Failed to cleanup temp directory: {}: {}", temp_dir.display(), e);
    }

    println!();
    println!(
        "{}",
        "✓ Successfully upgraded to the latest version!".green()
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
            log::warn!("Failed to remove old executable: {}: {}", old_exe.display(), e);
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
