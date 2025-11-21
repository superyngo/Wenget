//! Initialize WenPM

use crate::bucket::Bucket;
use crate::core::Config;
use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::io::{self, Write as IoWrite};
use std::path::PathBuf;

#[cfg(not(windows))]
use std::fs::{self, OpenOptions};

/// Initialize WenPM (create directories and manifests)
pub fn run(yes: bool) -> Result<()> {
    println!("{}", "Initializing WenPM...".cyan());

    let config = Config::new()?;

    if config.is_initialized() {
        println!("{}", "✓ WenPM is already initialized".green());
        println!("  Root: {}", config.paths().root().display());

        // Check and setup wenpm executable if missing
        check_and_setup_wenpm_executable(&config)?;

        // Check if PATH is already configured
        if is_in_path(config.paths().bin_dir())? {
            println!("{}", "✓ WenPM bin directory is in PATH".green());
        } else {
            println!("{}", "⚠ WenPM bin directory is not in PATH".yellow());
            println!();
            setup_path(&config)?;
        }

        // Check if wenpm bucket exists
        if !has_wenpm_bucket(&config)? {
            println!();
            if prompt_add_wenpm_bucket(yes)? {
                add_wenpm_bucket(&config)?;
            }
        } else {
            println!("{}", "✓ WenPM bucket is configured".green());
        }

        return Ok(());
    }

    config.init()?;

    println!("{}", "✓ WenPM initialized successfully!".green());
    println!();
    println!("Created directories:");
    println!("  Root:      {}", config.paths().root().display());
    println!("  Apps:      {}", config.paths().apps_dir().display());
    println!("  Bin:       {}", config.paths().bin_dir().display());
    println!("  Cache:     {}", config.paths().cache_dir().display());
    println!();
    println!("Created manifests:");
    println!("  Sources:   {}", config.paths().sources_json().display());
    println!("  Installed: {}", config.paths().installed_json().display());
    println!();

    // Setup wenpm executable itself
    setup_wenpm_executable(&config)?;

    // Set up PATH
    setup_path(&config)?;

    // Ask about adding wenpm bucket
    println!();
    if prompt_add_wenpm_bucket(yes)? {
        add_wenpm_bucket(&config)?;
    }

    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. List available:       wenpm source list");
    println!("  2. Search packages:      wenpm search <keyword>");
    println!("  3. Install packages:     wenpm add <package-name>");

    Ok(())
}

/// Create wenpm shim with absolute path (Windows)
#[cfg(windows)]
fn create_wenpm_shim(target: &PathBuf, shim: &PathBuf) -> Result<()> {
    use std::fs;

    log::debug!("Creating wenpm shim: {}", shim.display());

    // Use absolute path in shim to avoid relative path issues
    let shim_content = format!("@echo off\r\n\"{}\" %*\r\n", target.display());

    // Create parent directory
    if let Some(parent) = shim.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove existing shim if it exists
    if shim.exists() {
        fs::remove_file(shim)
            .with_context(|| format!("Failed to remove existing shim: {}", shim.display()))?;
    }

    // Write shim file
    fs::write(shim, shim_content)
        .with_context(|| format!("Failed to create shim: {}", shim.display()))?;

    Ok(())
}

/// Create wenpm symlink (Unix)
#[cfg(unix)]
fn create_wenpm_symlink(target: &PathBuf, link: &PathBuf) -> Result<()> {
    use std::os::unix::fs::symlink;

    log::debug!(
        "Creating wenpm symlink: {} -> {}",
        link.display(),
        target.display()
    );

    // Remove existing symlink if it exists
    if link.exists() || link.is_symlink() {
        std::fs::remove_file(link)
            .with_context(|| format!("Failed to remove existing symlink: {}", link.display()))?;
    }

    // Create parent directory
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }

    symlink(target, link)
        .with_context(|| format!("Failed to create symlink: {}", link.display()))?;

    Ok(())
}

/// Check and setup wenpm executable if missing (for already initialized)
fn check_and_setup_wenpm_executable(config: &Config) -> Result<()> {
    let bin_dir = config.paths().bin_dir();

    #[cfg(windows)]
    let wenpm_bin = bin_dir.join("wenpm.cmd");

    #[cfg(unix)]
    let wenpm_bin = bin_dir.join("wenpm");

    // Check if wenpm shim/symlink exists
    if wenpm_bin.exists() {
        println!("{}", "✓ WenPM shim is in bin directory".green());
    } else {
        println!("{}", "⚠ WenPM shim is not in bin directory".yellow());
        println!();
        setup_wenpm_executable(config)?;
    }

    Ok(())
}

/// Setup wenpm executable itself in bin directory
fn setup_wenpm_executable(config: &Config) -> Result<()> {
    let current_exe = env::current_exe().context("Failed to get current executable path")?;
    let bin_dir = config.paths().bin_dir();

    #[cfg(windows)]
    {
        let shim_path = bin_dir.join("wenpm.cmd");

        match create_wenpm_shim(&current_exe, &shim_path) {
            Ok(_) => {
                println!("{}", "✓ Created wenpm shim in bin directory".green());
            }
            Err(e) => {
                println!("{} Failed to create wenpm shim: {}", "⚠".yellow(), e);
                println!("  You can manually create a shim to wenpm.exe later");
            }
        }
    }

    #[cfg(unix)]
    {
        let symlink_path = bin_dir.join("wenpm");

        match create_wenpm_symlink(&current_exe, &symlink_path) {
            Ok(_) => {
                println!("{}", "✓ Created wenpm symlink in bin directory".green());
            }
            Err(e) => {
                println!("{} Failed to create wenpm symlink: {}", "⚠".yellow(), e);
                println!("  You can manually link wenpm to the bin directory later");
            }
        }
    }

    println!();
    Ok(())
}

/// Set up PATH for WenPM bin directory
fn setup_path(config: &Config) -> Result<()> {
    let bin_dir = config.paths().bin_dir();
    let bin_dir_str = bin_dir.to_string_lossy();

    println!("{}", "Setting up PATH...".cyan());

    #[cfg(windows)]
    {
        setup_path_windows(&bin_dir_str)?;
    }

    #[cfg(not(windows))]
    {
        setup_path_unix(&bin_dir_str)?;
    }

    Ok(())
}

/// Set up PATH on Windows (modify user environment variable)
#[cfg(windows)]
fn setup_path_windows(bin_dir: &str) -> Result<()> {
    use std::process::Command;

    // Use PowerShell to add to user PATH
    let ps_script = format!(
        r#"
        $oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if ($oldPath -notlike '*{}*') {{
            $newPath = $oldPath + ';{}'
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
            Write-Output 'Added'
        }} else {{
            Write-Output 'Already exists'
        }}
        "#,
        bin_dir, bin_dir
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()
        .context("Failed to execute PowerShell command")?;

    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Added") {
        println!("{}", "✓ Added WenPM bin directory to user PATH".green());
        println!();
        println!("{}", "IMPORTANT:".yellow().bold());
        println!("  Please restart your terminal or command prompt");
        println!("  for the PATH changes to take effect.");
    } else if result.contains("Already exists") {
        println!("{}", "✓ WenPM bin directory is already in PATH".green());
    } else if !output.status.success() {
        println!("{}", "⚠ Failed to automatically update PATH".yellow());
        println!();
        println!("Please manually add the following to your PATH:");
        println!("  {}", bin_dir.cyan());
    }

    Ok(())
}

/// Set up PATH on Unix-like systems (add to shell config)
#[cfg(not(windows))]
fn setup_path_unix(bin_dir: &str) -> Result<()> {
    let home = dirs::home_dir().context("Failed to determine home directory")?;

    // Determine which shell configs to update
    let shell_configs = detect_shell_configs(&home);

    if shell_configs.is_empty() {
        println!("{}", "⚠ No shell configuration files found".yellow());
        println!();
        println!("Please manually add the following to your shell configuration:");
        println!("  export PATH=\"{}:$PATH\"", bin_dir.cyan());
        return Ok(());
    }

    let export_line = format!("\n# WenPM\nexport PATH=\"{}:$PATH\"\n", bin_dir);

    let mut updated_files = Vec::new();
    let mut skipped_files = Vec::new();

    for config_path in shell_configs {
        match update_shell_config(&config_path, &export_line, bin_dir) {
            Ok(true) => updated_files.push(config_path),
            Ok(false) => skipped_files.push(config_path),
            Err(e) => {
                println!(
                    "  {} Failed to update {}: {}",
                    "⚠".yellow(),
                    config_path.display(),
                    e
                );
            }
        }
    }

    if !updated_files.is_empty() {
        println!("{}", "✓ Updated shell configuration files:".green());
        for path in &updated_files {
            println!("  • {}", path.display());
        }
        println!();
        println!("{}", "IMPORTANT:".yellow().bold());
        println!("  Run the following command to apply changes:");
        println!(
            "  source ~/{}",
            updated_files[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .cyan()
        );
        println!();
        println!("  Or restart your terminal");
    }

    if !skipped_files.is_empty() {
        println!("{}", "✓ WenPM is already configured in:".green());
        for path in &skipped_files {
            println!("  • {}", path.display());
        }
    }

    Ok(())
}

/// Detect available shell configuration files
#[cfg(not(windows))]
fn detect_shell_configs(home: &PathBuf) -> Vec<PathBuf> {
    let mut configs = Vec::new();

    // Check for common shell configs
    let candidates = vec![".bashrc", ".bash_profile", ".zshrc", ".profile"];

    for candidate in candidates {
        let path = home.join(candidate);
        if path.exists() {
            configs.push(path);
        }
    }

    // If no configs found, try to create .profile
    if configs.is_empty() {
        let profile = home.join(".profile");
        configs.push(profile);
    }

    configs
}

/// Update a shell configuration file
#[cfg(not(windows))]
fn update_shell_config(config_path: &PathBuf, export_line: &str, bin_dir: &str) -> Result<bool> {
    // Check if already configured
    if config_path.exists() {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        if content.contains(bin_dir) {
            return Ok(false); // Already configured
        }
    }

    // Append to file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path)
        .with_context(|| format!("Failed to open {}", config_path.display()))?;

    file.write_all(export_line.as_bytes())
        .with_context(|| format!("Failed to write to {}", config_path.display()))?;

    Ok(true)
}

/// Check if a directory is in PATH
fn is_in_path(dir: PathBuf) -> Result<bool> {
    let path_var = env::var("PATH").unwrap_or_default();
    let dir_str = dir.to_string_lossy();

    Ok(path_var
        .split(if cfg!(windows) { ';' } else { ':' })
        .any(|p| p == dir_str.as_ref()))
}

/// Prompt user to add wenpm bucket
fn prompt_add_wenpm_bucket(yes: bool) -> Result<bool> {
    if yes {
        return Ok(true);
    }

    println!("{}", "─".repeat(60));
    println!();
    println!("{}", "Add official WenPM bucket?".bold());
    println!();
    println!("The WenPM bucket provides curated open-source tools including:");
    println!("  • ripgrep, fd, bat - Modern CLI utilities");
    println!("  • gitui, zoxide - Enhanced Git and navigation");
    println!("  • starship, bottom - Shell customization and monitoring");
    println!("  • and more...");
    println!();
    print!("Add wenpm bucket? [Y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(input.is_empty() || input == "y" || input == "yes")
}

/// Check if wenpm bucket is already configured
fn has_wenpm_bucket(config: &Config) -> Result<bool> {
    const WENPM_BUCKET_URL: &str =
        "https://raw.githubusercontent.com/superyngo/wenpm-bucket/refs/heads/main/manifest.json";

    match config.get_or_create_buckets() {
        Ok(bucket_config) => {
            // Check if any bucket has the wenpm URL
            Ok(bucket_config
                .buckets
                .iter()
                .any(|b| b.url == WENPM_BUCKET_URL))
        }
        Err(_) => Ok(false),
    }
}

/// Add wenpm bucket
fn add_wenpm_bucket(config: &Config) -> Result<()> {
    const WENPM_BUCKET_NAME: &str = "wenpm";
    const WENPM_BUCKET_URL: &str =
        "https://raw.githubusercontent.com/superyngo/wenpm-bucket/refs/heads/main/manifest.json";

    println!();
    println!("{} wenpm bucket...", "Adding".cyan());

    // Load bucket config
    let mut bucket_config = config.get_or_create_buckets()?;

    // Create bucket
    let bucket = Bucket {
        name: WENPM_BUCKET_NAME.to_string(),
        url: WENPM_BUCKET_URL.to_string(),
        enabled: true,
        priority: 100,
    };

    // Try to add bucket
    if bucket_config.add_bucket(bucket) {
        // Save config
        config.save_buckets(&bucket_config)?;

        println!("{} Bucket '{}' added", "✓".green(), WENPM_BUCKET_NAME);
        println!("  URL: {}", WENPM_BUCKET_URL);

        // Build cache immediately
        match config.rebuild_cache() {
            Ok(cache) => {
                println!();
                println!(
                    "{} {} package(s) available from wenpm bucket",
                    "✓".green(),
                    cache.packages.len()
                );
            }
            Err(e) => {
                println!();
                println!("{} Failed to build cache: {}", "⚠".yellow(), e);
                println!("  You can rebuild it later with: wenpm bucket refresh");
            }
        }
    } else {
        println!(
            "{} Bucket '{}' already exists",
            "✗".yellow(),
            WENPM_BUCKET_NAME
        );
    }

    Ok(())
}
