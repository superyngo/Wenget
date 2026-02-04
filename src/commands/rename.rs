//! Rename command implementation

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Input, Select};
use std::fs;
use std::path::Path;

use crate::core::{Config, InstalledManifest, InstalledPackage};
use crate::installer;

/// Run the rename command
///
/// Supports two modes:
/// 1. Direct: `wenget rename <old_cmd> <new_cmd>` - rename specified command
/// 2. Interactive: `wenget rename <package_name>` - select from multiple commands
pub fn run(old_name: String, new_name: Option<String>, config: &Config) -> Result<()> {
    let paths = config.paths();
    let mut installed = config.load_installed()?;

    // Find the package by command name or package key
    let (pkg_key, old_cmd_name, package) = find_package_and_command(&installed, &old_name)?;

    // If multiple commands exist and no new_name provided, enter interactive mode
    let final_old_cmd = if package.command_names.len() > 1 && new_name.is_none() {
        select_command_interactive(&package)?
    } else {
        old_cmd_name
    };

    // Get or prompt for new name
    let final_new_name = if let Some(new_name) = new_name {
        new_name
    } else {
        prompt_for_new_name(&final_old_cmd)?
    };

    // Validate new name doesn't conflict
    validate_new_name(&installed, &pkg_key, &final_new_name)?;

    println!(
        "{} Renaming command: {} → {}",
        "ℹ".cyan(),
        final_old_cmd.yellow(),
        final_new_name.green()
    );

    // Perform rename
    rename_command(
        paths,
        &mut installed,
        &pkg_key,
        &final_old_cmd,
        &final_new_name,
    )?;

    // Save updated manifest
    config.save_installed(&installed)?;

    println!("{} Successfully renamed command", "✓".green().bold());
    println!(
        "  {} New command: {}",
        "ℹ".cyan(),
        final_new_name.green().bold()
    );

    Ok(())
}

/// Find package by command name or package key
///
/// Returns (package_key, command_name, package_ref)
fn find_package_and_command(
    installed: &InstalledManifest,
    name: &str,
) -> Result<(String, String, InstalledPackage)> {
    // First, try direct package key lookup
    if let Some(package) = installed.packages.get(name) {
        if package.command_names.is_empty() {
            anyhow::bail!("Package '{}' has no commands", name);
        }
        let cmd_name = package.command_names[0].clone();
        return Ok((name.to_string(), cmd_name, package.clone()));
    }

    // Search by repo_name (to support matching variants by repo name)
    for (key, package) in &installed.packages {
        if package.repo_name == name {
            if package.command_names.is_empty() {
                anyhow::bail!("Package '{}' has no commands", name);
            }
            let cmd_name = package.command_names[0].clone();
            return Ok((key.clone(), cmd_name, package.clone()));
        }
    }

    // Search by command name
    for (key, package) in &installed.packages {
        if package.command_names.contains(&name.to_string()) {
            return Ok((key.clone(), name.to_string(), package.clone()));
        }
    }

    anyhow::bail!(
        "Package or command '{}' not found. Use 'wenget ls' to see installed packages.",
        name
    )
}

/// Interactively select a command when package has multiple
fn select_command_interactive(package: &InstalledPackage) -> Result<String> {
    println!("{} Package has multiple commands:", "ℹ".cyan());

    let selection = Select::new()
        .with_prompt("Select command to rename")
        .items(&package.command_names)
        .default(0)
        .interact()
        .context("Failed to get user selection")?;

    Ok(package.command_names[selection].clone())
}

/// Prompt user for new command name
fn prompt_for_new_name(old_name: &str) -> Result<String> {
    let new_name: String = Input::new()
        .with_prompt(format!("New name for '{}'", old_name))
        .interact_text()
        .context("Failed to get user input")?;

    if new_name.trim().is_empty() {
        anyhow::bail!("New name cannot be empty");
    }

    Ok(new_name.trim().to_string())
}

/// Validate that new name doesn't conflict with existing commands
fn validate_new_name(
    installed: &InstalledManifest,
    exclude_key: &str,
    new_name: &str,
) -> Result<()> {
    for (key, package) in &installed.packages {
        if key == exclude_key {
            continue; // Skip the package we're renaming
        }

        if package.command_names.contains(&new_name.to_string()) {
            anyhow::bail!(
                "Command name '{}' is already used by package '{}'",
                new_name,
                key
            );
        }
    }

    Ok(())
}

/// Perform the actual rename operation
///
/// Updates symlink/shim and modifies InstalledPackage.command_names
fn rename_command(
    paths: &crate::core::WenPaths,
    installed: &mut InstalledManifest,
    pkg_key: &str,
    old_cmd: &str,
    new_cmd: &str,
) -> Result<()> {
    let package = installed
        .packages
        .get(pkg_key)
        .context("Package not found in manifest")?;

    // Find the index of the old command name
    let cmd_index = package
        .command_names
        .iter()
        .position(|c| c == old_cmd)
        .context("Command not found in package")?;

    // Get install path
    let install_path = Path::new(&package.install_path);
    if !install_path.exists() {
        anyhow::bail!("Install path does not exist: {}", install_path.display());
    }

    // Remove old symlink/shim
    let old_shim = paths.bin_shim_path(old_cmd);
    if old_shim.exists() {
        fs::remove_file(&old_shim)
            .with_context(|| format!("Failed to remove old shim: {}", old_shim.display()))?;
        log::info!("Removed old shim: {}", old_shim.display());
    }

    // Create new symlink/shim
    #[cfg(unix)]
    {
        // Find the actual binary in the install path that matches the old command
        let binary = find_binary_in_path(install_path, &package.files, old_cmd)?;
        installer::create_symlink(&binary, &paths.bin_dir().join(new_cmd))
            .context("Failed to create new symlink")?;
    }

    #[cfg(windows)]
    {
        let binary = find_binary_in_path(install_path, &package.files, old_cmd)?;
        installer::create_shim(
            &binary,
            &paths.bin_dir().join(format!("{}.cmd", new_cmd)),
            new_cmd,
        )
        .context("Failed to create new shim")?;
    }

    log::info!("Created new shim/symlink: {}", new_cmd);

    // Update command_names in the package
    let package_mut = installed
        .packages
        .get_mut(pkg_key)
        .context("Package disappeared during rename")?;

    package_mut.command_names[cmd_index] = new_cmd.to_string();

    Ok(())
}

/// Find the primary binary file in the install path that matches the command name
///
/// Uses a scoring system similar to find_executable_candidates:
/// - Prioritizes files in bin/ directory
/// - Matches based on command name
/// - Excludes shared libraries (.so) and other non-executable files
fn find_binary_in_path(install_path: &Path, files: &[String], command_name: &str) -> Result<std::path::PathBuf> {
    #[derive(Debug)]
    struct Candidate {
        path: std::path::PathBuf,
        score: u32,
    }

    let mut candidates = Vec::new();

    for file in files {
        let file_path = install_path.join(file);

        // Skip if file doesn't exist
        if !file_path.exists() {
            continue;
        }

        // Get filename
        let filename = match Path::new(file).file_name().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Skip non-executable files
        if !is_likely_executable(&file_path) {
            continue;
        }

        // Skip shared libraries and other non-binary files
        if filename.ends_with(".so")
            || filename.contains(".so.")
            || filename.ends_with(".dylib")
            || filename.ends_with(".dll")
            || filename.ends_with(".a")
            || filename.contains(".pc") // pkg-config files
        {
            continue;
        }

        let mut score = 0u32;
        let name_without_ext = filename.trim_end_matches(".exe");

        // Rule 1: Exact match with command name (highest priority)
        if name_without_ext == command_name {
            score += 100;
        }
        // Rule 2: Partial match
        else if name_without_ext.contains(command_name) || command_name.contains(name_without_ext) {
            score += 50;
        }

        // Rule 3: Located in bin/ directory - strong signal
        if file.contains("bin/") {
            score += 40;
        }

        // Rule 4: Has executable permission (already checked above)
        score += 10;

        if score > 0 {
            candidates.push(Candidate {
                path: file_path,
                score,
            });
        }
    }

    // Sort by score (highest first)
    candidates.sort_by(|a, b| b.score.cmp(&a.score));

    // Return the highest scoring candidate
    if let Some(best) = candidates.first() {
        log::debug!(
            "Selected binary: {} (score: {})",
            best.path.display(),
            best.score
        );
        return Ok(best.path.clone());
    }

    anyhow::bail!(
        "No executable binary found matching command '{}' in install path: {}",
        command_name,
        install_path.display()
    )
}

/// Check if a file is likely to be executable
fn is_likely_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            // Check if file has execute permission (any execute bit set)
            return permissions.mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(windows)]
    {
        // On Windows, check if it's an .exe file
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback for other platforms
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_new_name_success() {
        let mut manifest = InstalledManifest::new();
        let package = InstalledPackage {
            repo_name: "pkg1".to_string(),
            variant: None,
            version: "1.0.0".to_string(),
            platform: "linux-x86_64".to_string(),
            installed_at: chrono::Utc::now(),
            install_path: "/path/to/pkg1".to_string(),
            files: vec![],
            source: crate::core::manifest::PackageSource::Bucket {
                name: "test".to_string(),
            },
            description: String::new(),
            command_names: vec!["oldcmd".to_string()],
            command_name: None,
            asset_name: "pkg1.tar.gz".to_string(),
            parent_package: None,
        };
        manifest.packages.insert("pkg1".to_string(), package);

        assert!(validate_new_name(&manifest, "pkg1", "newcmd").is_ok());
    }

    #[test]
    fn test_validate_new_name_conflict() {
        let mut manifest = InstalledManifest::new();

        let package1 = InstalledPackage {
            repo_name: "pkg1".to_string(),
            variant: None,
            version: "1.0.0".to_string(),
            platform: "linux-x86_64".to_string(),
            installed_at: chrono::Utc::now(),
            install_path: "/path/to/pkg1".to_string(),
            files: vec![],
            source: crate::core::manifest::PackageSource::Bucket {
                name: "test".to_string(),
            },
            description: String::new(),
            command_names: vec!["cmd1".to_string()],
            command_name: None,
            asset_name: "pkg1.tar.gz".to_string(),
            parent_package: None,
        };
        manifest.packages.insert("pkg1".to_string(), package1);

        let package2 = InstalledPackage {
            repo_name: "pkg2".to_string(),
            variant: None,
            version: "1.0.0".to_string(),
            platform: "linux-x86_64".to_string(),
            installed_at: chrono::Utc::now(),
            install_path: "/path/to/pkg2".to_string(),
            files: vec![],
            source: crate::core::manifest::PackageSource::Bucket {
                name: "test".to_string(),
            },
            description: String::new(),
            command_names: vec!["cmd2".to_string()],
            command_name: None,
            asset_name: "pkg2.tar.gz".to_string(),
            parent_package: None,
        };
        manifest.packages.insert("pkg2".to_string(), package2);

        // Try to rename pkg1's cmd to "cmd2" which is already used
        assert!(validate_new_name(&manifest, "pkg1", "cmd2").is_err());
    }
}
