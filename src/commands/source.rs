//! Source command implementation

use crate::core::{Config, Platform};
use crate::providers::{GitHubProvider, SourceProvider};
use anyhow::{Context, Result};
use colored::Colorize;
use glob::Pattern;
use std::fs;

/// Source subcommands
pub enum SourceCommand {
    Add { urls: Vec<String> },
    Del { names: Vec<String> },
    Import { source: String },
    Export { output: Option<String>, format: String },
    Update,
    List,
    Info { names: Vec<String> },
}

/// Run source command
pub fn run(cmd: SourceCommand) -> Result<()> {
    match cmd {
        SourceCommand::Add { urls } => run_add(urls),
        SourceCommand::Del { names } => run_del(names),
        SourceCommand::Import { source } => run_import(source),
        SourceCommand::Export { output, format } => run_export(output, format),
        SourceCommand::Update => run_update(),
        SourceCommand::List => run_list(),
        SourceCommand::Info { names } => run_info(names),
    }
}

/// Add packages from GitHub URLs
fn run_add(urls: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    // Ensure WenPM is initialized
    if !config.is_initialized() {
        config.init()?;
    }

    if urls.is_empty() {
        println!("{}", "No URLs provided".yellow());
        println!("Usage: wenpm source add <url>...");
        return Ok(());
    }

    println!("{} {} package(s)...\n", "Adding".cyan(), urls.len());

    // Load existing manifest
    let mut manifest = config.get_or_create_sources()?;

    // Create provider
    let github = GitHubProvider::new()?;

    // Track results
    let mut added = 0;
    let mut skipped = 0;
    let mut failed = 0;

    // Process each URL
    for url in urls {
        print!("  {} {} ... ", "Processing".cyan(), url);

        match process_url(&github, &url) {
            Ok(package) => {
                let name = package.name.clone();
                let platform_count = package.platforms.len();

                // Try to add package
                if manifest.add_package(package) {
                    println!("{} {} ({} platforms)", "Added".green(), name, platform_count);
                    added += 1;
                } else {
                    println!("{} (already exists)", "Skipped".yellow());
                    skipped += 1;
                }
            }
            Err(e) => {
                println!("{} {}", "Failed".red(), e);
                failed += 1;
            }
        }
    }

    // Save manifest
    config.save_sources(&manifest)?;

    // Summary
    println!();
    println!("{}", "Summary:".bold());
    if added > 0 {
        println!("  {} {} package(s) added", "✓".green(), added);
    }
    if skipped > 0 {
        println!("  {} {} package(s) skipped", "•".cyan(), skipped);
    }
    if failed > 0 {
        println!("  {} {} package(s) failed", "✗".red(), failed);
    }

    println!();
    println!("Total packages in sources: {}", manifest.packages.len());

    Ok(())
}

/// Delete packages from sources
fn run_del(names: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    // Load manifest
    let mut manifest = config.get_or_create_sources()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages in sources".yellow());
        return Ok(());
    }

    if names.is_empty() {
        println!("{}", "No package names or URLs provided".yellow());
        println!("Usage: wenpm source del <name|url>...");
        return Ok(());
    }

    println!("{} package(s)...\n", "Deleting".cyan());

    let mut deleted = 0;
    let mut not_found = 0;

    for name in names {
        print!("  {} {} ... ", "Deleting".cyan(), name);

        // Try to match by name first, then by URL
        let to_delete = if let Some(_) = manifest.find_package(&name) {
            Some(name.clone())
        } else {
            // Try to find by URL
            manifest
                .packages
                .iter()
                .find(|p| p.repo == name)
                .map(|p| p.name.clone())
        };

        if let Some(pkg_name) = to_delete {
            if manifest.remove_package(&pkg_name) {
                println!("{}", "Deleted".green());
                deleted += 1;
            } else {
                println!("{}", "Not found".yellow());
                not_found += 1;
            }
        } else {
            println!("{}", "Not found".yellow());
            not_found += 1;
        }
    }

    // Save manifest
    config.save_sources(&manifest)?;

    // Summary
    println!();
    println!("{}", "Summary:".bold());
    if deleted > 0 {
        println!("  {} {} package(s) deleted", "✓".green(), deleted);
    }
    if not_found > 0 {
        println!("  {} {} package(s) not found", "•".yellow(), not_found);
    }

    println!();
    println!("Total packages in sources: {}", manifest.packages.len());

    Ok(())
}

/// Import packages from a file or URL (supports txt and json formats)
fn run_import(source: String) -> Result<()> {
    let config = Config::new()?;

    // Ensure WenPM is initialized
    if !config.is_initialized() {
        config.init()?;
    }

    println!("{} {}", "Reading from:".cyan(), source);

    let content = if source.starts_with("http://") || source.starts_with("https://") {
        // Fetch from URL
        crate::utils::HttpClient::new()?
            .get_text(&source)
            .context("Failed to fetch source file from URL")?
    } else {
        // Read from local file
        fs::read_to_string(&source)
            .with_context(|| format!("Failed to read source file: {}", source))?
    };

    // Detect format: JSON or txt
    let trimmed = content.trim();
    let is_json = trimmed.starts_with('{') || trimmed.starts_with('[');

    if is_json {
        // Import JSON format (package info)
        import_json(config, &content)
    } else {
        // Import txt format (URLs)
        import_txt(&content)
    }
}

/// Import from JSON format (package info)
fn import_json(config: Config, content: &str) -> Result<()> {
    println!("  {} JSON format", "Detected".cyan());

    // Load existing manifest
    let mut manifest = config.get_or_create_sources()?;

    // Try to parse as array of packages first
    let packages: Vec<crate::core::Package> = if content.trim().starts_with('[') {
        serde_json::from_str(content)
            .context("Failed to parse JSON as package array")?
    } else {
        // Try to parse as SourceManifest
        let source_manifest: crate::core::SourceManifest = serde_json::from_str(content)
            .context("Failed to parse JSON as SourceManifest")?;
        source_manifest.packages
    };

    if packages.is_empty() {
        println!("{}", "No packages found in JSON".yellow());
        return Ok(());
    }

    println!("  Found {} package(s)\n", packages.len());

    // Track results
    let mut added = 0;
    let mut updated = 0;
    let mut skipped = 0;

    // Add each package
    for package in packages {
        let name = package.name.clone();

        if let Some(existing) = manifest.packages.iter_mut().find(|p| p.name == name) {
            // Update existing package
            if existing.platforms != package.platforms {
                *existing = package;
                println!("  {} {} (platforms updated)", "✓".green(), name);
                updated += 1;
            } else {
                println!("  {} {} (already up to date)", "•".cyan(), name);
                skipped += 1;
            }
        } else {
            // Add new package
            println!("  {} {} ({} platforms)", "✓".green(), name, package.platforms.len());
            manifest.packages.push(package);
            added += 1;
        }
    }

    // Save manifest
    config.save_sources(&manifest)?;

    // Summary
    println!();
    println!("{}", "Summary:".bold());
    if added > 0 {
        println!("  {} {} package(s) added", "✓".green(), added);
    }
    if updated > 0 {
        println!("  {} {} package(s) updated", "✓".green(), updated);
    }
    if skipped > 0 {
        println!("  {} {} package(s) skipped", "•".cyan(), skipped);
    }

    println!();
    println!("Total packages in sources: {}", manifest.packages.len());

    Ok(())
}

/// Import from txt format (URLs)
fn import_txt(content: &str) -> Result<()> {
    println!("  {} txt format (URLs)", "Detected".cyan());

    // Parse URLs (one per line, skip empty lines and comments)
    let urls: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();

    println!("  Found {} URL(s)\n", urls.len());

    if urls.is_empty() {
        println!("{}", "No URLs found in source".yellow());
        return Ok(());
    }

    // Use run_add to add all URLs
    run_add(urls)
}

/// Export packages to a file (supports txt and json formats)
fn run_export(output: Option<String>, format: String) -> Result<()> {
    let config = Config::new()?;

    // Load manifest
    let manifest = config.get_or_create_sources()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages in sources to export".yellow());
        return Ok(());
    }

    let format_lower = format.to_lowercase();

    let content = match format_lower.as_str() {
        "json" => {
            // Export as JSON (full package info)
            serde_json::to_string_pretty(&manifest.packages)
                .context("Failed to serialize packages to JSON")?
        }
        "txt" | _ => {
            // Export as txt (URLs only)
            let urls: Vec<String> = manifest.packages.iter().map(|p| p.repo.clone()).collect();
            urls.join("\n")
        }
    };

    if let Some(output_path) = output {
        // Write to file
        fs::write(&output_path, &content)
            .with_context(|| format!("Failed to write to file: {}", output_path))?;

        let desc = if format_lower == "json" {
            "package info"
        } else {
            "package URLs"
        };

        println!(
            "{} {} {} to {} (format: {})",
            "Exported".green(),
            manifest.packages.len(),
            desc,
            output_path,
            format_lower
        );
    } else {
        // Print to stdout
        println!("{}", content);
    }

    Ok(())
}

/// Update package metadata from sources
fn run_update() -> Result<()> {
    let config = Config::new()?;

    // Load manifest
    let manifest = config.get_or_create_sources()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages in sources to update".yellow());
        println!("Add packages with: wenpm source add <github-url>");
        return Ok(());
    }

    println!(
        "{} {} package(s)...\n",
        "Updating".cyan(),
        manifest.packages.len()
    );

    // Create provider
    let github = GitHubProvider::new()?;

    // Track results
    let mut updated = 0;
    let mut unchanged = 0;
    let mut failed = 0;

    // Create a new manifest to store updated packages
    let mut new_manifest = crate::core::SourceManifest::new();

    // Update each package
    for pkg in &manifest.packages {
        print!("  {} {} ... ", "Updating".cyan(), pkg.name);

        match github.fetch_package(&pkg.repo) {
            Ok(new_pkg) => {
                // Check if platforms changed
                let platforms_changed = new_pkg.platforms != pkg.platforms;

                if platforms_changed {
                    println!("{} (platforms updated)", "Updated".green());
                    updated += 1;
                } else {
                    println!("{}", "Up to date".green());
                    unchanged += 1;
                }

                new_manifest.packages.push(new_pkg);
            }
            Err(e) => {
                println!("{} {}", "Failed".red(), e);
                // Keep the old package data
                new_manifest.packages.push(pkg.clone());
                failed += 1;
            }
        }
    }

    // Save manifest
    config.save_sources(&new_manifest)?;

    // Summary
    println!();
    println!("{}", "Summary:".bold());
    if updated > 0 {
        println!("  {} {} package(s) updated", "✓".green(), updated);
    }
    if unchanged > 0 {
        println!("  {} {} package(s) unchanged", "•".cyan(), unchanged);
    }
    if failed > 0 {
        println!("  {} {} package(s) failed", "✗".red(), failed);
    }

    Ok(())
}

/// Process a single URL
fn process_url(provider: &GitHubProvider, url: &str) -> Result<crate::core::Package> {
    if !provider.can_handle(url) {
        anyhow::bail!("Unsupported URL (only GitHub is supported currently)");
    }

    provider.fetch_package(url)
}

/// List available packages from sources
fn run_list() -> Result<()> {
    let config = Config::new()?;

    // Load manifest
    let manifest = config.get_or_create_sources()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages in sources".yellow());
        println!("Add packages with: wenpm source add <github-url>");
        return Ok(());
    }

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Filter packages that support current platform
    let compatible_packages: Vec<_> = manifest
        .packages
        .iter()
        .filter(|pkg| {
            platform_ids
                .iter()
                .any(|id| pkg.platforms.contains_key(id))
        })
        .collect();

    if compatible_packages.is_empty() {
        println!(
            "{}",
            format!("No packages available for {}", platform).yellow()
        );
        return Ok(());
    }

    // Print header
    println!("{}", format!("Available packages for {}", platform).bold());
    println!();
    println!(
        "{:<20} {:<10} {}",
        "NAME".bold(),
        "SIZE".bold(),
        "DESCRIPTION".bold()
    );
    println!("{}", "─".repeat(80));

    // Print packages
    for pkg in &compatible_packages {
        // Find the first matching platform
        let platform_binary = platform_ids
            .iter()
            .find_map(|id| pkg.platforms.get(id))
            .unwrap();

        let size_mb = platform_binary.size as f64 / 1_000_000.0;

        println!(
            "{:<20} {:>8.1} MB  {}",
            pkg.name.green(),
            size_mb,
            truncate(&pkg.description, 50)
        );
    }

    println!();
    println!(
        "Total: {} package(s)",
        compatible_packages.len()
    );

    Ok(())
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Show package information
fn run_info(patterns: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    // Load manifests
    let sources = config.get_or_create_sources()?;
    let installed = config.get_or_create_installed()?;

    // Create GitHub provider to fetch versions
    let github = GitHubProvider::new()?;

    if sources.packages.is_empty() {
        println!("{}", "No packages in sources".yellow());
        println!("Add packages with: wenpm source add <github-url>");
        return Ok(());
    }

    if patterns.is_empty() {
        println!("{}", "No package name provided".yellow());
        println!("Usage: wenpm source info <name>...");
        return Ok(());
    }

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Compile glob patterns
    let glob_patterns: Vec<Pattern> = patterns
        .iter()
        .map(|p| Pattern::new(p))
        .collect::<Result<_, _>>()?;

    // Filter packages
    let matching_packages: Vec<_> = sources
        .packages
        .iter()
        .filter(|pkg| glob_patterns.iter().any(|pattern| pattern.matches(&pkg.name)))
        .collect();

    if matching_packages.is_empty() {
        println!(
            "{}",
            format!("No packages found matching: {:?}", patterns).yellow()
        );
        return Ok(());
    }

    // Display info for each package
    for (i, pkg) in matching_packages.iter().enumerate() {
        if i > 0 {
            println!();
            println!("{}", "─".repeat(80));
            println!();
        }

        println!("{}: {}", "Name".bold(), pkg.name.green());
        println!("{}: {}", "Description".bold(), pkg.description);
        println!("{}: {}", "Repository".bold(), pkg.repo);

        if let Some(homepage) = &pkg.homepage {
            println!("{}: {}", "Homepage".bold(), homepage);
        }

        if let Some(license) = &pkg.license {
            println!("{}: {}", "License".bold(), license);
        }

        // Fetch latest version
        let latest_version = github
            .fetch_latest_version(&pkg.repo)
            .unwrap_or_else(|_| "unknown".to_string());

        println!("{}: {}", "Latest Version".bold(), latest_version);

        // Check if installed
        if let Some(inst_pkg) = installed.get_package(&pkg.name) {
            println!(
                "{}: {} ({})",
                "Installed Version".bold(),
                inst_pkg.version.green(),
                if inst_pkg.version == latest_version {
                    "up to date".green()
                } else {
                    "upgrade available".yellow()
                }
            );
            println!("{}: {}", "Installed At".bold(), inst_pkg.installed_at);
            println!("{}: {}", "Install Path".bold(), inst_pkg.install_path);
        } else {
            println!("{}: {}", "Installed".bold(), "No".red());
        }

        println!();
        println!("{}", "Supported Platforms:".bold());

        for (platform_id, binary) in &pkg.platforms {
            let size_mb = binary.size as f64 / 1_000_000.0;
            let is_current = platform_ids.contains(platform_id);

            let marker = if is_current {
                format!(" {}", "(current)".cyan())
            } else {
                String::new()
            };

            println!("  • {} - {:.1} MB{}", platform_id, size_mb, marker);
        }
    }

    Ok(())
}
