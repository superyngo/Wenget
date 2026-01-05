//! Delete command implementation

use crate::core::{Config, WenPaths};
use anyhow::{Context, Result};
use colored::Colorize;
use glob::Pattern;
use std::env;
use std::fs;
use std::path::Path;

/// Delete installed packages
pub fn run(names: Vec<String>, yes: bool, force: bool) -> Result<()> {
    // Check for self-deletion request
    if names.len() == 1 && names[0].to_lowercase() == "self" {
        return delete_self(yes);
    }

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
        println!("Usage: wenget del <name>...");
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

    // Check for wenget self-deletion
    if matching_packages.contains(&"wenget".to_string()) && !force {
        println!("{}", "Cannot delete wenget itself".red());
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
    if !yes
        && !crate::utils::prompt::confirm_no_default("\nProceed with deletion?")? {
            println!("Deletion cancelled");
            return Ok(());
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

/// Delete Wenget itself (complete uninstallation)
fn delete_self(yes: bool) -> Result<()> {
    println!("{}", "Wenget Self-Deletion".bold().red());
    println!("{}", "═".repeat(60));
    println!();
    println!(
        "{}",
        "This will COMPLETELY remove Wenget from your system:".yellow()
    );
    println!();

    let paths = WenPaths::new()?;

    println!("  {} All Wenget directories and files:", "1.".bold());
    println!("     {}", paths.root().display());
    println!();
    println!("  {} Wenget from PATH environment variable", "2.".bold());
    println!();
    println!("  {} The wenget executable itself", "3.".bold());

    // Get current executable path
    let exe_path = env::current_exe().context("Failed to get current executable path")?;
    println!("     {}", exe_path.display());
    println!();

    // Confirm deletion
    if !yes {
        println!("{}", "═".repeat(60));
        println!();
        println!("{}", "Are you sure you want to proceed?".bold().red());

        if !crate::utils::prompt::confirm_no_default("")? {
            println!();
            println!("{}", "Deletion cancelled".green());
            return Ok(());
        }
    }

    println!();
    println!("{}", "Proceeding with uninstallation...".cyan());
    println!();

    // Step 1: Remove from PATH
    println!("{} Removing from PATH...", "1.".bold());
    match remove_from_path(&paths.bin_dir()) {
        Ok(()) => println!("   {} PATH updated", "✓".green()),
        Err(e) => println!("   {} Failed to update PATH: {}", "⚠".yellow(), e),
    }
    println!();

    // Check if executable is inside .wenget directory
    let exe_in_wenget = exe_path.starts_with(paths.root());

    // Step 2: Delete Wenget directories
    println!("{} Deleting Wenget directories...", "2.".bold());
    if exe_in_wenget {
        println!(
            "   {} Scheduled for deletion (executable is inside .wenget)",
            "✓".yellow()
        );
        println!("      Directory will be deleted after wenget exits");
    } else if paths.root().exists() {
        match fs::remove_dir_all(paths.root()) {
            Ok(()) => println!("   {} Deleted: {}", "✓".green(), paths.root().display()),
            Err(e) => println!("   {} Failed to delete directory: {}", "✗".red(), e),
        }
    } else {
        println!("   {} Directory already removed", "✓".green());
    }
    println!();

    // Step 3: Delete the executable
    println!("{} Deleting wenget executable...", "3.".bold());
    delete_executable(&exe_path, exe_in_wenget, paths.root())?;

    println!();
    println!("{}", "═".repeat(60));
    println!();
    println!("{}", "Wenget has been uninstalled.".green().bold());
    println!();
    println!("{}", "Thank you for using Wenget!".cyan());
    println!();

    Ok(())
}

/// Remove Wenget bin directory from PATH
fn remove_from_path(bin_dir: &Path) -> Result<()> {
    let bin_dir_str = bin_dir.to_string_lossy();

    #[cfg(windows)]
    {
        remove_from_path_windows(&bin_dir_str)?;
    }

    #[cfg(not(windows))]
    {
        remove_from_path_unix(&bin_dir_str)?;
    }

    Ok(())
}

/// Remove from PATH on Windows
#[cfg(windows)]
fn remove_from_path_windows(bin_dir: &str) -> Result<()> {
    use std::process::Command;

    let ps_script = format!(
        r#"
        $oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if ($oldPath -like '*{}*') {{
            $newPath = ($oldPath -split ';' | Where-Object {{ $_ -ne '{}' }}) -join ';'
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
            Write-Output 'Removed'
        }} else {{
            Write-Output 'Not found'
        }}
        "#,
        bin_dir, bin_dir
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()
        .context("Failed to execute PowerShell command")?;

    let result = String::from_utf8_lossy(&output.stdout);

    if !result.contains("Removed") && !result.contains("Not found") && !output.status.success() {
        return Err(anyhow::anyhow!("PowerShell command failed"));
    }

    Ok(())
}

/// Remove from PATH on Unix-like systems
#[cfg(not(windows))]
fn remove_from_path_unix(bin_dir: &str) -> Result<()> {
    let home = dirs::home_dir().context("Failed to determine home directory")?;

    let shell_configs = vec![
        home.join(".bashrc"),
        home.join(".bash_profile"),
        home.join(".zshrc"),
        home.join(".profile"),
    ];

    for config_path in shell_configs {
        if config_path.exists() {
            if let Err(e) = remove_from_shell_config(&config_path, bin_dir) {
                log::warn!("Failed to update {}: {}", config_path.display(), e);
            }
        }
    }

    Ok(())
}

/// Remove Wenget PATH entry from a shell configuration file
#[cfg(not(windows))]
fn remove_from_shell_config(config_path: &Path, bin_dir: &str) -> Result<()> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;

    // Remove lines containing the Wenget PATH entry
    let new_content: String = content
        .lines()
        .filter(|line| {
            // Skip lines that contain the Wenget bin directory or Wenget comment
            !line.contains(bin_dir) && !line.contains("# Wenget")
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Only write if content changed
    if new_content != content {
        fs::write(config_path, new_content.trim_end())
            .with_context(|| format!("Failed to write to {}", config_path.display()))?;
    }

    Ok(())
}

/// Delete the executable (platform-specific implementation)
fn delete_executable(exe_path: &Path, exe_in_wenget: bool, wenget_root: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        delete_executable_windows(exe_path, exe_in_wenget, wenget_root)
    }

    #[cfg(not(windows))]
    {
        delete_executable_unix(exe_path, exe_in_wenget, wenget_root)
    }
}

/// Delete executable on Windows
/// On Windows, we can't delete a running executable directly,
/// so we use a batch script that waits and then deletes it
#[cfg(windows)]
fn delete_executable_windows(
    exe_path: &Path,
    exe_in_wenget: bool,
    wenget_root: &Path,
) -> Result<()> {
    use std::process::Command;

    // Create a temporary batch script to delete the executable after exit
    let temp_dir = env::temp_dir();
    let script_path = temp_dir.join("wenget_uninstall.bat");

    let exe_path_str = exe_path.to_string_lossy();
    let script_content = if exe_in_wenget {
        // If executable is inside .wenget, delete the entire directory
        let wenget_root_str = wenget_root.to_string_lossy();
        format!(
            r#"@echo off
timeout /t 2 /nobreak >nul
rd /s /q "{}"
del /f /q "%~f0"
"#,
            wenget_root_str
        )
    } else {
        // Otherwise just delete the executable
        format!(
            r#"@echo off
timeout /t 2 /nobreak >nul
del /f /q "{}"
del /f /q "%~f0"
"#,
            exe_path_str
        )
    };

    fs::write(&script_path, script_content).context("Failed to create uninstall script")?;

    // Launch the script in background
    Command::new("cmd")
        .args(["/C", "start", "/min", script_path.to_str().unwrap()])
        .spawn()
        .context("Failed to launch uninstall script")?;

    println!(
        "   {} Scheduled for deletion (will be removed in 2 seconds)",
        "✓".green()
    );

    Ok(())
}

/// Delete executable on Unix
#[cfg(not(windows))]
fn delete_executable_unix(exe_path: &Path, exe_in_wenget: bool, wenget_root: &Path) -> Result<()> {
    use std::process::Command;

    // Create a shell script to delete the executable after exit
    let temp_dir = env::temp_dir();
    let script_path = temp_dir.join("wenget_uninstall.sh");

    let exe_path_str = exe_path.to_string_lossy();
    let script_content = if exe_in_wenget {
        // If executable is inside .wenget, delete the entire directory
        let wenget_root_str = wenget_root.to_string_lossy();
        format!(
            r#"#!/bin/sh
sleep 2
rm -rf "{}"
rm -f "$0"
"#,
            wenget_root_str
        )
    } else {
        // Otherwise just delete the executable
        format!(
            r#"#!/bin/sh
sleep 2
rm -f "{}"
rm -f "$0"
"#,
            exe_path_str
        )
    };

    fs::write(&script_path, script_content).context("Failed to create uninstall script")?;

    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // Launch the script in background
    Command::new("sh")
        .arg(&script_path)
        .spawn()
        .context("Failed to launch uninstall script")?;

    println!(
        "   {} Scheduled for deletion (will be removed in 2 seconds)",
        "✓".green()
    );

    Ok(())
}
