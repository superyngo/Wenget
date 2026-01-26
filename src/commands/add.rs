//! Add (Install) command implementation

use crate::core::manifest::{PackageSource, ScriptType};
use crate::core::{Config, InstalledPackage, Platform, WenPaths};
use crate::downloader;
use crate::installer::{
    create_script_shim, detect_script_type, download_script, extract_archive, extract_script_name,
    find_executable_candidates,
    input_detector::{detect_input_type, InputType},
    install_script,
    local::install_local_file,
    normalize_command_name, read_local_script,
};
use crate::package_resolver::{PackageInput, PackageResolver, ResolvedPackage};
use crate::providers::{GitHubProvider, SourceProvider};
use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use std::fs;
use std::path::Path;

#[cfg(windows)]
use crate::installer::create_shim;

#[cfg(unix)]
use crate::installer::create_symlink;

/// Install packages (smart detection: package names from cache or GitHub URLs)
pub fn run(
    names: Vec<String>,
    yes: bool,
    script_name: Option<String>,
    platform: Option<String>,
    version: Option<String>,
    variant_filter: Option<String>,
) -> Result<()> {
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
        println!("  wenget add ./script.ps1         # Install local script");
        println!(
            "  wenget add https://raw.githubusercontent.com/.../script.sh  # Install remote script"
        );
        println!("  wenget add ripgrep -p linux-x64 # Install for specific platform");
        return Ok(());
    }

    // Categorize inputs
    let mut script_inputs = Vec::new();
    let mut local_inputs = Vec::new();
    let mut url_inputs = Vec::new();
    let mut package_inputs = Vec::new();

    for name in &names {
        match detect_input_type(name) {
            InputType::Script => script_inputs.push(name),
            InputType::LocalFile => local_inputs.push(name),
            InputType::DirectUrl => url_inputs.push(name),
            InputType::PackageName => package_inputs.push(name),
        }
    }

    // Handle script installations
    if !script_inputs.is_empty() {
        install_scripts(
            &config,
            &paths,
            &mut installed,
            script_inputs,
            yes,
            script_name.as_deref(),
        )?;
    }

    // Handle local file installations
    if !local_inputs.is_empty() {
        install_local_files(
            &config,
            &paths,
            &mut installed,
            local_inputs,
            yes,
            script_name.as_deref(),
        )?;
    }

    // Handle direct URL installations
    if !url_inputs.is_empty() {
        install_from_urls(
            &config,
            &paths,
            &mut installed,
            url_inputs,
            yes,
            script_name.as_deref(),
        )?;
    }

    // Handle package installations (existing logic)
    if !package_inputs.is_empty() {
        install_packages(
            &config,
            &paths,
            &mut installed,
            package_inputs,
            yes,
            script_name.as_deref(),
            platform.as_deref(),
            version.as_deref(),
            variant_filter.as_deref(),
        )?;
    }

    Ok(())
}

/// Resolve command name to avoid conflicts
///
/// Priority:
/// 1. If variant exists, use base_name-{variant}
/// 2. Otherwise, use base_name
/// 3. If name is taken, try base_name-{number}
/// 4. If is_custom is true, skip variant suffix appending and go directly to conflict checking
fn resolve_command_name(
    base_name: &str,
    variant: Option<&str>,
    installed: &crate::core::InstalledManifest,
    exclude_key: Option<&str>,
    is_custom: bool,
) -> String {
    // 1. If custom name provided, skip variant suffix - just check for conflicts
    if is_custom {
        if !installed.is_command_taken(base_name, exclude_key) {
            return base_name.to_string();
        }
        // Custom name taken, try numeric suffixes
        for i in 1..=99 {
            let numbered = format!("{}-{}", base_name, i);
            if !installed.is_command_taken(&numbered, exclude_key) {
                return numbered;
            }
        }
        return base_name.to_string();
    }

    // 2. If it's a variant, construct the desired command name
    if let Some(var) = variant {
        // Check if base_name already ends with the variant suffix
        // This handles cases where the binary itself contains the variant name
        // e.g., base_name="bun-profile", variant="profile" -> keep as "bun-profile"
        // e.g., base_name="bun-profile", variant="baseline-profile" -> change to "bun-baseline-profile"

        let desired_name = if base_name.ends_with(&format!("-{}", var)) {
            // Base name already ends with variant, use as-is
            base_name.to_string()
        } else if let Some(base_stripped) = extract_repo_name_from_command(base_name, var) {
            // Base name contains part of the variant, reconstruct with full variant
            // e.g., "bun-profile" with variant "baseline-profile" -> "bun-baseline-profile"
            format!("{}-{}", base_stripped, var)
        } else {
            // Normal case: append variant to base name
            format!("{}-{}", base_name, var)
        };

        if !installed.is_command_taken(&desired_name, exclude_key) {
            return desired_name;
        }
        // Desired name taken, try numeric suffixes
        for i in 1..=99 {
            let numbered = format!("{}-{}", desired_name, i);
            if !installed.is_command_taken(&numbered, exclude_key) {
                return numbered;
            }
        }
        return desired_name;
    }

    // 3. No variant - try base_name first
    if !installed.is_command_taken(base_name, exclude_key) {
        return base_name.to_string();
    }

    // 4. Try numeric suffixes for non-variant
    for i in 1..=99 {
        let numbered = format!("{}-{}", base_name, i);
        if !installed.is_command_taken(&numbered, exclude_key) {
            return numbered;
        }
    }

    base_name.to_string()
}

/// Extract repo name from a command name that may contain partial variant info
/// e.g., "bun-profile" with variant "baseline-profile" -> Some("bun")
/// e.g., "bun" with variant "baseline" -> None
fn extract_repo_name_from_command(command_name: &str, variant: &str) -> Option<String> {
    // Split variant by '-' to get all parts
    let variant_parts: Vec<&str> = variant.split('-').collect();

    // Check if command_name ends with any part of the variant
    for part in &variant_parts {
        if command_name.ends_with(&format!("-{}", part)) {
            // Strip this part and return the base
            if let Some(stripped) = command_name.strip_suffix(&format!("-{}", part)) {
                return Some(stripped.to_string());
            }
        }
    }

    None
}

/// Install scripts from local paths or URLs
fn install_scripts(
    config: &Config,
    paths: &WenPaths,
    installed: &mut crate::core::InstalledManifest,
    script_inputs: Vec<&String>,
    yes: bool,
    custom_name: Option<&str>,
) -> Result<()> {
    println!("{}", "Scripts to install:".bold());

    let mut scripts_to_install: Vec<(String, String, ScriptType, String)> = Vec::new(); // (name, content, type, origin)

    for input in script_inputs {
        // Determine if local or remote
        let is_url = input.starts_with("http://") || input.starts_with("https://");

        // Get script content
        let content = if is_url {
            match download_script(input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} Failed to download {}: {}", "✗".red(), input, e);
                    continue;
                }
            }
        } else {
            let path = Path::new(input);
            match read_local_script(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} Failed to read {}: {}", "✗".red(), input, e);
                    continue;
                }
            }
        };

        // Detect script type
        let script_type = match detect_script_type(input, &content) {
            Some(t) => t,
            None => {
                eprintln!("{} Cannot detect script type for: {}", "✗".red(), input);
                continue;
            }
        };

        // Check platform compatibility
        if !script_type.is_supported_on_current_platform() {
            println!(
                "  {} {} ({}) - {}",
                "⚠".yellow(),
                input,
                script_type.display_name(),
                "not supported on this platform".yellow()
            );
            continue;
        }

        // Determine script name
        let name = if let Some(custom) = custom_name {
            custom.to_string()
        } else {
            match extract_script_name(input) {
                Some(n) => n,
                None => {
                    eprintln!("{} Cannot extract name from: {}", "✗".red(), input);
                    continue;
                }
            }
        };

        // Check if already installed
        if installed.is_installed(&name) {
            println!(
                "  {} {} ({}) - {}",
                "•".yellow(),
                name,
                script_type.display_name(),
                "already installed, will be replaced".yellow()
            );
        } else {
            println!(
                "  {} {} ({}) {}",
                "•".green(),
                name,
                script_type.display_name(),
                "(new)".green()
            );
        }

        scripts_to_install.push((name, content, script_type, input.clone()));
    }

    if scripts_to_install.is_empty() {
        println!("{}", "No scripts to install".yellow());
        return Ok(());
    }

    // Show security warning
    println!();
    println!(
        "{}",
        "⚠  Security Warning: Review scripts before running them!"
            .yellow()
            .bold()
    );

    // Confirm installation
    if !yes && !crate::utils::confirm("\nProceed with installation?")? {
        println!("Installation cancelled");
        return Ok(());
    }

    println!();

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut successful_scripts: Vec<String> = Vec::new();
    let mut failed_scripts: Vec<String> = Vec::new();

    for (name, content, script_type, origin) in scripts_to_install {
        println!(
            "{} {} ({})...",
            "Installing".cyan(),
            name,
            script_type.display_name()
        );

        match install_single_script(paths, &name, &content, &script_type, &origin) {
            Ok(inst_pkg) => {
                installed.upsert_package(name.clone(), inst_pkg);
                if let Err(e) = config.save_installed(installed) {
                    println!("  {} Failed to save installed manifest: {}", "✗".red(), e);
                    fail_count += 1;
                    failed_scripts.push(name);
                    continue;
                }
                println!("  {} Installed successfully", "✓".green());
                success_count += 1;
                successful_scripts.push(name);
            }
            Err(e) => {
                println!("  {} {}", "✗".red(), e);
                fail_count += 1;
                failed_scripts.push(name);
            }
        }
    }

    println!();
    println!("{}", "Summary:".bold());
    if success_count > 0 {
        println!(
            "  {} {} script(s) installed: {}",
            "✓".green(),
            success_count,
            successful_scripts.join(" ")
        );
    }
    if fail_count > 0 {
        println!(
            "  {} {} script(s) failed: {}",
            "✗".red(),
            fail_count,
            failed_scripts.join(" ")
        );
    }

    Ok(())
}

/// Install a single script
fn install_single_script(
    paths: &WenPaths,
    name: &str,
    content: &str,
    script_type: &ScriptType,
    origin: &str,
) -> Result<InstalledPackage> {
    // Install script to app directory
    let files = install_script(paths, name, content, script_type)?;

    println!("  Command will be available as: {}", name);

    // Create shim
    println!("  Creating launcher...");
    create_script_shim(paths, name, script_type)?;

    // Create installed package info
    let inst_pkg = InstalledPackage {
        repo_name: name.to_string(),
        variant: None,
        version: "script".to_string(),
        platform: format!("{}-script", script_type.display_name().to_lowercase()),
        installed_at: Utc::now(),
        install_path: paths.app_dir(name).to_string_lossy().to_string(),
        files,
        source: PackageSource::Script {
            origin: origin.to_string(),
            script_type: script_type.clone(),
        },
        description: format!("{} script from {}", script_type.display_name(), origin),
        command_names: vec![name.to_string()],
        command_name: None,
        asset_name: format!("{}.{}", name, script_type.extension()),
        parent_package: None,
    };

    Ok(inst_pkg)
}

/// Install local binary or archive files
fn install_local_files(
    config: &Config,
    paths: &WenPaths,
    installed: &mut crate::core::InstalledManifest,
    files: Vec<&String>,
    yes: bool,
    custom_name: Option<&str>,
) -> Result<()> {
    println!("{}", "Local files to install:".bold());

    for file in &files {
        println!("  • {}", file);
    }

    if !yes && !crate::utils::confirm("\nProceed with installation?")? {
        println!("Installation cancelled");
        return Ok(());
    }

    println!();

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut successful_files: Vec<String> = Vec::new();
    let mut failed_files: Vec<String> = Vec::new();

    for file in files {
        println!("{} {}...", "Installing".cyan(), file);
        let path = Path::new(file);

        match install_local_file(paths, path, custom_name, None) {
            Ok(inst_pkg) => {
                // Use first command name as package name
                let name = match inst_pkg.command_names.first() {
                    Some(n) => n.clone(),
                    None => {
                        println!(
                            "  {} No command names found in installed package",
                            "✗".red()
                        );
                        fail_count += 1;
                        failed_files.push(file.to_string());
                        continue;
                    }
                };
                let display_names = inst_pkg.command_names.join(", ");
                installed.upsert_package(name.clone(), inst_pkg);
                if let Err(e) = config.save_installed(installed) {
                    println!("  {} Failed to save installed manifest: {}", "✗".red(), e);
                    fail_count += 1;
                    failed_files.push(name.clone());
                    continue;
                }
                println!(
                    "  {} Installed successfully as {}",
                    "✓".green(),
                    display_names
                );
                success_count += 1;
                successful_files.push(name);
            }
            Err(e) => {
                println!("  {} Failed to install {}: {}", "✗".red(), file, e);
                fail_count += 1;
                failed_files.push(file.to_string());
            }
        }
        println!();
    }

    println!("{}", "Summary:".bold());
    if success_count > 0 {
        println!(
            "  {} {} file(s) installed: {}",
            "✓".green(),
            success_count,
            successful_files.join(" ")
        );
    }
    if fail_count > 0 {
        println!(
            "  {} {} file(s) failed: {}",
            "✗".red(),
            fail_count,
            failed_files.join(" ")
        );
    }

    Ok(())
}

/// Install binary or archive from direct URLs
fn install_from_urls(
    config: &Config,
    paths: &WenPaths,
    installed: &mut crate::core::InstalledManifest,
    urls: Vec<&String>,
    yes: bool,
    custom_name: Option<&str>,
) -> Result<()> {
    println!("{}", "URLs to install:".bold());

    for url in &urls {
        println!("  • {}", url);
    }

    if !yes && !crate::utils::confirm("\nProceed with installation?")? {
        println!("Installation cancelled");
        return Ok(());
    }

    println!();

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut successful_urls: Vec<String> = Vec::new();
    let mut failed_urls: Vec<String> = Vec::new();

    // Create temp dir for downloads
    let temp_dir = paths.cache_dir().join("downloads");
    fs::create_dir_all(&temp_dir)?;

    for url in urls {
        println!("{} {}...", "Downloading".cyan(), url);

        let filename = match url.split('/').next_back() {
            Some(name) => name,
            None => {
                println!("  {} Invalid URL", "✗".red());
                fail_count += 1;
                failed_urls.push(url.to_string());
                continue;
            }
        };

        // Handle query parameters in URL
        let filename = filename.split('?').next().unwrap_or(filename);
        let download_path = temp_dir.join(filename);

        match downloader::download_file(url, &download_path) {
            Ok(_) => {
                println!("  {} Downloaded", "✓".green());
                println!("{} {}...", "Installing".cyan(), filename);

                match install_local_file(paths, &download_path, custom_name, Some(url.to_string()))
                {
                    Ok(inst_pkg) => {
                        // Use first command name as package name
                        let name = match inst_pkg.command_names.first() {
                            Some(n) => n.clone(),
                            None => {
                                println!(
                                    "  {} No command names found in installed package",
                                    "✗".red()
                                );
                                fail_count += 1;
                                failed_urls.push(filename.to_string());
                                continue;
                            }
                        };
                        let display_names = inst_pkg.command_names.join(", ");
                        installed.upsert_package(name.clone(), inst_pkg);
                        if let Err(e) = config.save_installed(installed) {
                            println!("  {} Failed to save installed manifest: {}", "✗".red(), e);
                            fail_count += 1;
                            failed_urls.push(name.clone());
                            continue;
                        }
                        println!(
                            "  {} Installed successfully as {}",
                            "✓".green(),
                            display_names
                        );
                        success_count += 1;
                        successful_urls.push(name);
                    }
                    Err(e) => {
                        println!("  {} Failed to install {}: {}", "✗".red(), filename, e);
                        fail_count += 1;
                        failed_urls.push(filename.to_string());
                    }
                }
            }
            Err(e) => {
                println!("  {} Failed to download {}: {}", "✗".red(), url, e);
                fail_count += 1;
                failed_urls.push(url.to_string());
            }
        }

        // Clean up downloaded file
        if download_path.exists() {
            if let Err(e) = fs::remove_file(&download_path) {
                log::warn!(
                    "Failed to cleanup downloaded file: {}: {}",
                    download_path.display(),
                    e
                );
            }
        }
        println!();
    }

    println!("{}", "Summary:".bold());
    if success_count > 0 {
        println!(
            "  {} {} URL(s) installed: {}",
            "✓".green(),
            success_count,
            successful_urls.join(" ")
        );
    }
    if fail_count > 0 {
        println!(
            "  {} {} URL(s) failed: {}",
            "✗".red(),
            fail_count,
            failed_urls.join(" ")
        );
    }

    Ok(())
}

/// Select packages from a platform that has multiple binaries
///
/// If only one binary: auto-select
/// If multiple binaries and --yes: select all
/// Otherwise: show MultiSelect dialog
fn select_packages_for_platform(
    pkg_name: &str,
    binaries: &[crate::core::manifest::PlatformBinary],
    yes: bool,
) -> Result<Vec<usize>> {
    if binaries.len() == 1 {
        // Single package: auto-select
        return Ok(vec![0]);
    }

    if yes {
        // --yes flag: select all
        println!(
            "  {} Found {} packages for {}, selecting all (--yes)",
            "ℹ".cyan(),
            binaries.len(),
            pkg_name
        );
        return Ok((0..binaries.len()).collect());
    }

    // Multiple packages: show selection dialog
    use dialoguer::MultiSelect;

    println!(
        "\n  {} Found {} packages for {}:",
        "ℹ".cyan(),
        binaries.len(),
        pkg_name
    );

    let items: Vec<String> = binaries
        .iter()
        .map(|b| format!("{} ({:.2} MB)", b.asset_name, b.size as f64 / 1_048_576.0))
        .collect();

    let selections = MultiSelect::new()
        .with_prompt("Select packages to install (Space to select, Enter to confirm)")
        .items(&items)
        .interact()?;

    if selections.is_empty() {
        anyhow::bail!("No packages selected");
    }

    Ok(selections)
}

/// Install packages from cache or GitHub (existing logic)
#[allow(clippy::too_many_arguments)]
fn install_packages(
    config: &Config,
    paths: &WenPaths,
    installed: &mut crate::core::InstalledManifest,
    names: Vec<&String>,
    yes: bool,
    custom_name: Option<&str>,
    custom_platform: Option<&str>,
    custom_version: Option<&str>,
    variant_filter: Option<&str>,
) -> Result<()> {
    // Get current platform
    let current_platform = if let Some(_custom_plat) = custom_platform {
        // TODO: Parse custom platform string into Platform struct
        // For now, use fallback to platform_ids logic
        Platform::current()
    } else {
        Platform::current()
    };

    // Load cache once for both script lookup and package resolution
    let cache = config.get_or_rebuild_cache()?;

    // Resolve all inputs and collect packages/scripts to install
    let resolver = PackageResolver::new(config, &cache)?;
    let mut packages_to_install: Vec<(
        String,
        ResolvedPackage,
        crate::core::platform::PlatformMatch,
    )> = Vec::new();
    let mut scripts_to_install: Vec<(String, String, ScriptType, String)> = Vec::new(); // (name, url, type, origin)

    for original_name in &names {
        let input = PackageInput::parse(original_name);

        match resolver.resolve(&input) {
            Ok(resolved) => {
                for pkg_resolved in resolved {
                    // Use smart platform matching
                    let matches = if let Some(custom_plat) = custom_platform {
                        // If custom platform specified, try direct match
                        if pkg_resolved.package.platforms.contains_key(custom_plat) {
                            vec![crate::core::platform::PlatformMatch {
                                platform_id: custom_plat.to_string(),
                                is_exact: true,
                                fallback_type: None,
                                score: 1000,
                            }]
                        } else {
                            vec![]
                        }
                    } else {
                        current_platform.find_best_match(&pkg_resolved.package.platforms)
                    };

                    if matches.is_empty() {
                        println!(
                            "{} {} does not support platform {}",
                            "Warning:".yellow(),
                            pkg_resolved.package.name,
                            current_platform
                        );
                        println!(
                            "  Available platforms: {}",
                            pkg_resolved
                                .package
                                .platforms
                                .keys()
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        continue;
                    }

                    let best_match = &matches[0];

                    // Check if fallback requires confirmation
                    if let Some(fallback_type) = &best_match.fallback_type {
                        if fallback_type.requires_confirmation() && !yes {
                            println!(
                                "{} {} - no exact match for {}, but {} is available",
                                "⚠".yellow(),
                                pkg_resolved.package.name,
                                current_platform,
                                best_match.platform_id
                            );
                            println!("  This is a fallback: {}", fallback_type.description());

                            if !crate::utils::prompt::confirm_no_default("  Install anyway?")? {
                                println!("  Skipped");
                                continue;
                            }
                        } else if !yes {
                            // Fallback doesn't require confirmation, but inform user
                            println!(
                                "{} Using fallback: {} ({})",
                                "ℹ".cyan(),
                                best_match.platform_id,
                                fallback_type.description()
                            );
                        }
                    }

                    packages_to_install.push((
                        original_name.to_string(),
                        pkg_resolved,
                        best_match.clone(),
                    ));
                }
            }
            Err(_) => {
                // If not found as package, check if it's a script in cache
                if let Some(cached_script) = cache.find_script(original_name) {
                    let script = &cached_script.script;

                    // Get installable script for current platform (checks if interpreter exists)
                    if let Some((script_type, platform_info)) = script.get_installable_script() {
                        // Prepare script for installation
                        let source_name = match &cached_script.source {
                            PackageSource::Bucket { name } => format!("bucket:{}", name),
                            _ => "unknown".to_string(),
                        };

                        scripts_to_install.push((
                            script.name.clone(),
                            platform_info.url.clone(),
                            script_type,
                            source_name,
                        ));
                    } else {
                        println!(
                            "{} {} is not supported on current platform (available: {})",
                            "Warning:".yellow(),
                            script.name,
                            script.platforms_display()
                        );
                    }
                } else {
                    eprintln!("{} {}: Not found", "Error".red().bold(), original_name);
                }
            }
        }
    }

    if packages_to_install.is_empty() && scripts_to_install.is_empty() {
        println!("{}", "No packages or scripts to install".yellow());
        return Ok(());
    }

    // Create GitHub provider to fetch versions (for packages)
    let github = if !packages_to_install.is_empty() {
        Some(GitHubProvider::new()?)
    } else {
        None
    };

    // Show packages to install with versions and handle already-installed packages
    if !packages_to_install.is_empty() {
        println!("{}", "Packages to install:".bold());
    }

    let mut to_install: Vec<(
        String,
        ResolvedPackage,
        crate::core::platform::PlatformMatch,
    )> = Vec::new();
    let mut to_update: Vec<(
        String,
        ResolvedPackage,
        crate::core::platform::PlatformMatch,
    )> = Vec::new();

    for (original_name, resolved, platform_match) in packages_to_install {
        let pkg_name = &resolved.package.name;
        let repo = &resolved.package.repo;

        // Fetch version (either custom, or latest from API, falling back to cache)
        // IMPORTANT: Always fetch from GitHub API first to ensure accurate version comparison
        // for update detection. Cached bucket version may be stale.
        let version = if let Some(custom_ver) = custom_version {
            // User specified a version
            custom_ver.to_string()
        } else if let Some(ref gh) = github {
            // Fetch latest version from API for accurate comparison
            gh.fetch_latest_version(repo).unwrap_or_else(|_| {
                // API failed - fall back to cached version
                resolved
                    .package
                    .version
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string())
            })
        } else if let Some(ref v) = resolved.package.version {
            // No GitHub provider available - use cached version
            v.clone()
        } else {
            "unknown".to_string()
        };

        // Check if already installed
        // Determine which key to check based on input type and variant filter
        let variant_key; // Storage for temporary String if needed
        let check_name: &str = if original_name.contains("::") {
            // Input like "bun::baseline" - check that specific variant
            original_name.as_str()
        } else if let Some(filter) = variant_filter {
            // Base name with --variant flag - generate the variant key
            variant_key = crate::core::manifest::generate_installed_key(pkg_name, Some(filter));
            &variant_key
        } else {
            // Base name without variant - check the base package
            pkg_name
        };

        if installed.is_installed(check_name) {
            // Package already installed
            let inst_pkg = installed.get_package(check_name).unwrap();
            if inst_pkg.version == version {
                // Same version installed - ask if user wants to reinstall
                println!(
                    "  {} {} v{} {}",
                    "•".cyan(),
                    check_name,
                    version,
                    "(already installed, same version)".dimmed()
                );
                if !yes && crate::utils::prompt::confirm_no_default("  Reinstall?")? {
                    // User wants to reinstall
                    to_install.push((original_name.clone(), resolved, platform_match));
                }
                // If user says no or --yes flag is used, skip reinstallation
            } else {
                println!(
                    "  {} {} v{} {} → {}",
                    "•".yellow(),
                    check_name,
                    inst_pkg.version.dimmed(),
                    "upgrade to".yellow(),
                    version.green()
                );
                // Show download URLs for the matched platform
                if let Some(binaries) = resolved.package.platforms.get(&platform_match.platform_id)
                {
                    for binary in binaries {
                        println!("    {} {}", "↳".dimmed(), binary.url.dimmed());
                    }
                }
                to_update.push((original_name.clone(), resolved, platform_match));
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
            // Show download URLs for the matched platform
            if let Some(binaries) = resolved.package.platforms.get(&platform_match.platform_id) {
                for binary in binaries {
                    println!("    {} {}", "↳".dimmed(), binary.url.dimmed());
                }
            }
            to_install.push((original_name.clone(), resolved, platform_match));
        }
    }

    // Show scripts to install
    let mut scripts_to_process: Vec<(String, String, ScriptType, String)> = Vec::new();

    if !scripts_to_install.is_empty() {
        println!();
        println!("{}", "Scripts to install:".bold());

        for (name, url, script_type, origin) in scripts_to_install {
            if installed.is_installed(&name) {
                println!(
                    "  {} {} ({}) {}",
                    "•".yellow(),
                    name,
                    script_type.display_name(),
                    "(already installed, will update)".dimmed()
                );
            } else {
                println!(
                    "  {} {} ({}) {}",
                    "•".green(),
                    name,
                    script_type.display_name(),
                    "(new)".green()
                );
            }
            scripts_to_process.push((name, url, script_type, origin));
        }
    }

    // Check if there's anything to do
    if to_install.is_empty() && to_update.is_empty() && scripts_to_process.is_empty() {
        println!();
        println!(
            "{}",
            "All packages and scripts are already up to date".green()
        );
        return Ok(());
    }

    // Confirm installation
    if !yes && !crate::utils::confirm("\nProceed with installation?")? {
        println!("Installation cancelled");
        return Ok(());
    }

    println!();

    // Install/update packages
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut successful_packages: Vec<String> = Vec::new();
    let mut failed_packages: Vec<String> = Vec::new();

    // Combine new installs and updates
    let all_packages: Vec<_> = to_install.into_iter().chain(to_update).collect();

    // Collect packages to update in cache (packages fetched from GitHub API)
    let mut packages_to_cache: Vec<(crate::core::Package, PackageSource)> = Vec::new();

    for (original_input_name, resolved, platform_match) in all_packages {
        let pkg_name = &resolved.package.name;
        let repo_url = &resolved.package.repo;

        // Extract variant from input name (e.g., "bun::baseline" -> Some("baseline"))
        // This takes precedence over the global variant_filter parameter
        let input_variant = if original_input_name.contains("::") {
            original_input_name
                .split("::")
                .nth(1)
                .map(|s| s.to_string())
        } else {
            None
        };

        // Determine effective variant filter: input-specific variant takes precedence
        let effective_variant_filter = input_variant.as_deref().or(variant_filter);

        // Try to fetch package info from GitHub API (includes download links)
        // If API rate limit is hit, fallback to cached package info
        let (pkg_to_install, version, using_fallback) = if let Some(ref gh) = github {
            if let Some(custom_ver) = custom_version {
                // User specified a version - fetch that specific version
                match gh.fetch_package_by_version(repo_url, custom_ver) {
                    Ok(versioned_pkg) => {
                        // Successfully fetched specific version from GitHub API
                        let version = custom_ver.trim_start_matches('v').to_string();
                        (versioned_pkg, version, false)
                    }
                    Err(e) => {
                        // Version not found - error and abort
                        println!("  {} {}", "✗".red(), e);
                        fail_count += 1;
                        continue;
                    }
                }
            } else {
                // No version specified - fetch latest
                match gh.fetch_package(repo_url) {
                    Ok(latest_pkg) => {
                        // Successfully fetched from GitHub API - use latest download links
                        // Version is now included in the package struct
                        let version = latest_pkg
                            .version
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string());
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

                        // Use version from cached package if available
                        let version = resolved
                            .package
                            .version
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string());
                        (resolved.package.clone(), version, true)
                    }
                }
            }
        } else {
            // No GitHub provider available, use cached package info
            let version = resolved
                .package
                .version
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            (resolved.package.clone(), version, true)
        };

        // Get all binaries for this platform
        let binaries = match pkg_to_install.platforms.get(&platform_match.platform_id) {
            Some(bins) => bins,
            None => {
                println!("  {} Platform binary not found", "✗".red());
                fail_count += 1;
                failed_packages.push(pkg_name.to_string());
                continue;
            }
        };

        // Apply variant filter if specified
        let (filtered_binaries, _original_indices): (Vec<_>, Vec<_>) =
            if let Some(filter) = effective_variant_filter {
                binaries
                    .iter()
                    .enumerate()
                    .filter(|(_, binary)| {
                        let variant = crate::core::manifest::extract_variant_from_asset(
                            &binary.asset_name,
                            pkg_name,
                        );
                        variant.as_deref() == Some(filter)
                    })
                    .map(|(idx, binary)| (binary.clone(), idx))
                    .unzip()
            } else {
                (binaries.clone(), (0..binaries.len()).collect())
            };

        // Check if any binaries remain after filtering
        if filtered_binaries.is_empty() {
            if let Some(filter) = effective_variant_filter {
                println!(
                    "  {} No binaries found for variant '{}'. Available variants:",
                    "✗".red(),
                    filter
                );
                for binary in binaries {
                    let variant = crate::core::manifest::extract_variant_from_asset(
                        &binary.asset_name,
                        pkg_name,
                    );
                    if let Some(v) = variant {
                        println!("    - {}", v);
                    } else {
                        println!("    - (default)");
                    }
                }
            }
            fail_count += 1;
            failed_packages.push(pkg_name.to_string());
            continue;
        }

        // Select which packages to install (single, all, or user selection)
        let selected_indices = match select_packages_for_platform(pkg_name, &filtered_binaries, yes)
        {
            Ok(indices) => indices,
            Err(e) => {
                println!("  {} {}", "✗".red(), e);
                fail_count += 1;
                failed_packages.push(pkg_name.to_string());
                continue;
            }
        };

        // Install each selected binary
        let mut parent_key: Option<String> = None;

        for (i, &idx) in selected_indices.iter().enumerate() {
            let binary = &filtered_binaries[idx];

            // Extract variant name from asset_name
            // If platform originally has only one binary and no filters applied, treat as default (no variant)
            let variant = if binaries.len() == 1
                && effective_variant_filter.is_none()
                && custom_platform.is_none()
            {
                // Platform has only one binary originally, treat as default
                None
            } else {
                crate::core::manifest::extract_variant_from_asset(&binary.asset_name, pkg_name)
            };
            let installed_key =
                crate::core::manifest::generate_installed_key(pkg_name, variant.as_deref());

            // Determine parent package (first one is parent, rest are children)
            let parent_package = if i == 0 {
                parent_key = Some(installed_key.clone());
                None
            } else {
                parent_key.clone()
            };

            println!("{} {} v{}...", "Installing".cyan(), installed_key, version);
            if using_fallback {
                println!(
                    "  {} Falling back to bucket source download links",
                    "ℹ".cyan()
                );
            }
            if selected_indices.len() > 1 {
                println!("  {} From: {}", "ℹ".cyan(), binary.asset_name.dimmed());
            }

            match install_package(
                config,
                paths,
                &pkg_to_install,
                &platform_match,
                binary,
                &version,
                &resolved.source,
                &installed_key,
                parent_package.as_deref(),
                custom_name,
                yes,
            ) {
                Ok(inst_pkg) => {
                    installed.upsert_package(installed_key.clone(), inst_pkg);
                    if let Err(e) = config.save_installed(installed) {
                        println!("  {} Failed to save installed manifest: {}", "✗".red(), e);
                        fail_count += 1;
                        failed_packages.push(installed_key.clone());
                        continue;
                    }

                    // Collect package for cache update if fetched from GitHub API
                    // (only once, not for each binary)
                    if i == 0 && !using_fallback {
                        packages_to_cache.push((pkg_to_install.clone(), resolved.source.clone()));
                    }

                    println!("  {} Installed successfully", "✓".green());
                    success_count += 1;
                    successful_packages.push(installed_key.clone());
                }
                Err(e) => {
                    println!("  {} {}", "✗".red(), e);
                    fail_count += 1;
                    failed_packages.push(installed_key.clone());
                }
            }
            println!();
        }
    }

    // Update cache with latest package info from GitHub API
    if !packages_to_cache.is_empty() {
        match update_cache_with_packages(config, packages_to_cache) {
            Ok(count) => {
                log::info!("Updated cache with {} latest package(s)", count);
            }
            Err(e) => {
                log::warn!("Failed to update cache: {}", e);
                // Don't fail the entire operation if cache update fails
            }
        }
    }

    // Install scripts from bucket cache
    let mut script_success_count = 0;
    let mut script_fail_count = 0;
    let mut successful_scripts: Vec<String> = Vec::new();
    let mut failed_scripts: Vec<String> = Vec::new();

    for (name, url, script_type, origin) in scripts_to_process {
        println!(
            "{}",
            format!("Installing {} ({})...", name, script_type.display_name()).bold()
        );

        match install_script_from_bucket(
            config,
            paths,
            installed,
            &name,
            &url,
            script_type.clone(),
            &origin,
            custom_name,
        ) {
            Ok(_) => {
                println!("  {} Installed successfully", "✓".green());
                script_success_count += 1;
                successful_scripts.push(name);
            }
            Err(e) => {
                println!("  {} {}", "✗".red(), e);
                script_fail_count += 1;
                failed_scripts.push(name);
            }
        }
        println!();
    }

    // Summary
    println!("{}", "Summary:".bold());
    if success_count > 0 {
        println!(
            "  {} {} package(s) installed: {}",
            "✓".green(),
            success_count,
            successful_packages.join(" ")
        );
    }
    if fail_count > 0 {
        println!(
            "  {} {} package(s) failed: {}",
            "✗".red(),
            fail_count,
            failed_packages.join(" ")
        );
    }
    if script_success_count > 0 {
        println!(
            "  {} {} script(s) installed: {}",
            "✓".green(),
            script_success_count,
            successful_scripts.join(" ")
        );
    }
    if script_fail_count > 0 {
        println!(
            "  {} {} script(s) failed: {}",
            "✗".red(),
            script_fail_count,
            failed_scripts.join(" ")
        );
    }

    Ok(())
}

/// Install a single package
#[allow(clippy::too_many_arguments)]
fn install_package(
    config: &Config,
    paths: &WenPaths,
    pkg: &crate::core::Package,
    platform_match: &crate::core::platform::PlatformMatch,
    binary: &crate::core::manifest::PlatformBinary,
    version: &str,
    source: &PackageSource,
    installed_key: &str,
    _parent_package: Option<&str>, // Deprecated parameter, kept for compatibility
    custom_name: Option<&str>,
    yes: bool,
) -> Result<InstalledPackage> {
    // Log if using fallback
    if let Some(fallback_type) = &platform_match.fallback_type {
        log::info!(
            "Using fallback platform {} ({})",
            platform_match.platform_id,
            fallback_type.description()
        );
    }

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

    // Extract to app directory (use installed_key for directory name)
    let app_dir = paths.app_dir(installed_key);

    println!("  Extracting to {}...", app_dir.display());

    // Remove existing installation
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir)?;
    }

    let extracted_files = extract_archive(&download_path, &app_dir)?;

    // Find executable candidates (pass app_dir for Unix permission checks)
    let candidates = find_executable_candidates(&extracted_files, &pkg.name, Some(&app_dir));

    if candidates.is_empty() {
        anyhow::bail!(
            "Failed to find executable in archive. Extracted files:\n{}",
            extracted_files.join("\n")
        );
    }

    // Select executables
    let selected_executables = if candidates.len() == 1 {
        // Single candidate - auto-select
        let selected = &candidates[0];
        println!(
            "  Found executable: {} ({})",
            selected.path, selected.reason
        );
        vec![candidates[0].path.clone()]
    } else {
        // Multiple candidates - select all with valid scores (exec permission or name match)
        // On Unix, exec permission gives +35 score, name match gives +50
        // Files without any match get score 0 and should be filtered out
        let auto_select: Vec<_> = candidates
            .iter()
            .filter(|c| c.score > 0) // All valid candidates
            .collect();

        if auto_select.len() <= 3 || yes {
            // Auto-select if reasonable count (<=3) or --yes flag
            println!("  Found {} executables:", auto_select.len());
            for c in &auto_select {
                println!("    {} ({})", c.path, c.reason);
            }
            auto_select.into_iter().map(|c| c.path.clone()).collect()
        } else {
            // Too many candidates - show interactive selection
            use dialoguer::MultiSelect;

            println!("  Found {} possible executables:", candidates.len());

            let items: Vec<String> = candidates
                .iter()
                .map(|c| format!("{} (score: {}, {})", c.path, c.score, c.reason))
                .collect();

            let selections = MultiSelect::new()
                .with_prompt("Select executables to install (Space to select, Enter to confirm)")
                .items(&items)
                .interact()?;

            if selections.is_empty() {
                anyhow::bail!("No executables selected");
            }

            selections
                .into_iter()
                .map(|i| candidates[i].path.clone())
                .collect()
        }
    };

    // Install all selected executables
    let mut command_names = Vec::new();

    // Extract repo_name and variant from installed_key for resolve_command_name
    // installed_key format: "repo_name" or "repo_name::variant"
    let (_, variant_opt) = if let Some(pos) = installed_key.find("::") {
        (
            installed_key[..pos].to_string(),
            Some(installed_key[pos + 2..].to_string()),
        )
    } else {
        (installed_key.to_string(), None)
    };

    // Load installed manifest for command name resolution
    let installed_manifest = config.get_or_create_installed()?;

    for exe_relative in selected_executables {
        let exe_path = app_dir.join(&exe_relative);

        if !exe_path.exists() {
            anyhow::bail!("Executable not found: {}", exe_path.display());
        }

        // Extract the actual command name from the executable path
        let (base_name, is_custom) = if let Some(custom) = custom_name {
            // Use custom name if provided (only for first executable)
            if command_names.is_empty() {
                (custom.to_string(), true)
            } else {
                // For additional executables, use auto-detected name
                let raw_name = exe_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .context("Failed to extract command name")?;
                (normalize_command_name(raw_name), false)
            }
        } else {
            // Auto-detect and normalize command name
            let raw_name = exe_path
                .file_name()
                .and_then(|s| s.to_str())
                .context("Failed to extract command name")?;

            // Apply smart normalization to remove platform suffixes
            (normalize_command_name(raw_name), false)
        };

        // Resolve command name with variant to avoid conflicts
        let resolved_name = resolve_command_name(
            &base_name,
            variant_opt.as_deref(),
            &installed_manifest,
            Some(installed_key),
            is_custom,
        );

        println!("  Command will be available as: {}", resolved_name);

        // Create symlink/shim using the resolved name
        let bin_path = paths.bin_shim_path(&resolved_name);

        println!("  Creating launcher at {}...", bin_path.display());

        #[cfg(unix)]
        {
            create_symlink(&exe_path, &bin_path)?;
        }

        #[cfg(windows)]
        {
            create_shim(&exe_path, &bin_path, &resolved_name)?;
        }

        command_names.push(resolved_name);
    }

    // Clean up download
    fs::remove_file(&download_path)?;

    // Extract repo_name and variant from installed_key
    // installed_key format: "repo_name" or "repo_name::variant"
    let (repo_name, variant) = if let Some(pos) = installed_key.find("::") {
        (
            installed_key[..pos].to_string(),
            Some(installed_key[pos + 2..].to_string()),
        )
    } else {
        (installed_key.to_string(), None)
    };

    // Command names were already resolved during symlink creation
    let resolved_command_names = command_names;

    // Create installed package info
    let inst_pkg = InstalledPackage {
        repo_name,
        variant,
        version: version.to_string(),
        platform: platform_match.platform_id.clone(),
        installed_at: Utc::now(),
        install_path: app_dir.to_string_lossy().to_string(),
        files: extracted_files,
        source: source.clone(),
        description: pkg.description.clone(),
        command_names: resolved_command_names,
        command_name: None,
        asset_name: binary.asset_name.clone(),
        parent_package: None, // Deprecated field
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

/// Install a script from bucket cache
#[allow(clippy::too_many_arguments)]
fn install_script_from_bucket(
    config: &Config,
    paths: &WenPaths,
    installed: &mut crate::core::InstalledManifest,
    name: &str,
    url: &str,
    script_type: ScriptType,
    origin: &str,
    custom_name: Option<&str>,
) -> Result<()> {
    println!("  Downloading script from {}...", url);

    // Download script content
    let content = download_script(url)?;

    // Determine the final command name
    let command_name = custom_name.unwrap_or(name);

    println!("  Installing script as '{}'...", command_name);

    // Install script to app directory
    let files = install_script(paths, command_name, &content, &script_type)?;

    println!("  Command will be available as: {}", command_name);

    // Create shim
    println!("  Creating launcher...");
    create_script_shim(paths, command_name, &script_type)?;

    // Create installed package info
    let inst_pkg = InstalledPackage {
        repo_name: command_name.to_string(),
        variant: None,
        version: "script".to_string(),
        platform: std::env::consts::OS.to_string(),
        installed_at: Utc::now(),
        install_path: paths.app_dir(command_name).display().to_string(),
        files,
        source: PackageSource::Script {
            origin: origin.to_string(),
            script_type: script_type.clone(),
        },
        description: format!("{} script from bucket", script_type.display_name()),
        command_names: vec![command_name.to_string()],
        command_name: None,
        asset_name: format!("{}.{}", name, script_type.extension()),
        parent_package: None,
    };

    // Update installed manifest
    installed.upsert_package(name.to_string(), inst_pkg);
    config.save_installed(installed)?;

    Ok(())
}
