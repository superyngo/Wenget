//! List command implementation

use crate::core::{Config, Platform};
use anyhow::Result;
use colored::Colorize;

/// List installed packages or all available packages
pub fn run(all: bool) -> Result<()> {
    let config = Config::new()?;

    if all {
        // Show all available packages from cache
        list_all_packages(&config)?;
    } else {
        // Show only installed packages
        list_installed_packages(&config)?;
    }

    Ok(())
}

/// List only installed packages
fn list_installed_packages(config: &Config) -> Result<()> {
    // Load installed manifest
    let manifest = config.get_or_create_installed()?;

    if manifest.packages.is_empty() {
        println!("{}", "No packages installed".yellow());
        println!("Install packages with: wenget add <name>");
        return Ok(());
    }

    // Print header
    println!("{}", "Installed packages".bold());
    println!();
    println!(
        "{:<20} {:<15} {:<10} {:<12} {}",
        "NAME".bold(),
        "COMMAND".bold(),
        "VERSION".bold(),
        "SOURCE".bold(),
        "DESCRIPTION".bold()
    );
    println!("{}", "─".repeat(100));

    // Group packages: parent packages and their variants
    let mut parent_packages: Vec<(&String, &crate::core::InstalledPackage)> = Vec::new();
    let mut variants_map: std::collections::HashMap<
        String,
        Vec<(&String, &crate::core::InstalledPackage)>,
    > = std::collections::HashMap::new();

    for (name, pkg) in &manifest.packages {
        if let Some(ref parent) = pkg.parent_package {
            variants_map
                .entry(parent.clone())
                .or_default()
                .push((name, pkg));
        } else {
            parent_packages.push((name, pkg));
        }
    }

    // Sort parent packages
    parent_packages.sort_by(|a, b| a.0.cmp(b.0));

    // Display packages with tree structure
    for (name, pkg) in &parent_packages {
        // Get source display
        let source_display = match &pkg.source {
            crate::core::manifest::PackageSource::Bucket { name } => name.clone(),
            crate::core::manifest::PackageSource::DirectRepo { .. } => "url".to_string(),
            crate::core::manifest::PackageSource::Script { script_type, .. } => {
                script_type.display_name().to_lowercase().to_string()
            }
        };

        // Truncate description if too long
        let description = if pkg.description.len() > 30 {
            format!("{}...", &pkg.description[..27])
        } else {
            pkg.description.clone()
        };

        println!(
            "{:<20} {:<15} {:<10} {:<12} {}",
            name.green(),
            pkg.command_names.join(", ").yellow(),
            pkg.version,
            source_display.cyan(),
            description
        );

        // Display variants (tree structure)
        if let Some(variants) = variants_map.get(*name) {
            let mut sorted_variants = variants.clone();
            sorted_variants.sort_by(|a, b| a.0.cmp(b.0));

            for (i, (var_name, var_pkg)) in sorted_variants.iter().enumerate() {
                let prefix = if i == sorted_variants.len() - 1 {
                    "└─"
                } else {
                    "├─"
                };

                // Truncate variant description
                let var_desc = if var_pkg.description.len() > 25 {
                    format!("{}...", &var_pkg.description[..22])
                } else {
                    var_pkg.description.clone()
                };

                println!(
                    "  {} {:<17} {:<15} {:<10} {}",
                    prefix.dimmed(),
                    var_name.green(),
                    var_pkg.command_names.join(", ").yellow(),
                    var_pkg.version.dimmed(),
                    var_desc.dimmed()
                );
            }
        }
    }

    // Calculate total (including variants)
    let total_variants: usize = variants_map.values().map(|v| v.len()).sum();
    let total_packages = manifest.packages.len();

    println!();
    if total_variants > 0 {
        println!(
            "Total: {} package(s) installed ({} with variants)",
            total_packages,
            parent_packages.len()
        );
    } else {
        println!("Total: {} package(s) installed", total_packages);
    }

    Ok(())
}

/// List all available packages from cache
fn list_all_packages(config: &Config) -> Result<()> {
    // Get packages from cache
    let manifest = config.get_packages_from_cache()?;

    // Load installed packages for marking
    let installed = config.get_or_create_installed()?;

    // Get current platform
    let platform = Platform::current();
    let platform_ids = platform.possible_identifiers();

    // Filter packages that support current platform
    let mut packages: Vec<_> = manifest
        .packages
        .iter()
        .filter(|pkg| {
            platform_ids
                .iter()
                .any(|pid| pkg.platforms.contains_key(pid))
        })
        .collect();

    // Filter scripts that are compatible with current OS
    let scripts: Vec<_> = manifest
        .scripts
        .iter()
        .filter(|script| script.is_compatible_with_current_platform())
        .collect();

    if packages.is_empty() && scripts.is_empty() {
        println!("{}", "No packages available in buckets".yellow());
        println!("Add a bucket with: wenget bucket add <name> <url>");
        return Ok(());
    }

    // Sort alphabetically
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    // Print header
    println!("{}", "Available packages".bold());
    println!();
    println!(
        "{:<30} {:<12} {}",
        "NAME".bold(),
        "TYPE".bold(),
        "DESCRIPTION".bold()
    );
    println!("{}", "─".repeat(80));

    // Print packages
    for pkg in &packages {
        if installed.is_installed(&pkg.name) {
            // For installed packages, calculate padding manually to account for "(installed)"
            let name_width = 30;
            let installed_suffix = " (installed)";
            let visible_length = pkg.name.len() + installed_suffix.len();
            let padding = if visible_length < name_width {
                name_width - visible_length
            } else {
                1
            };

            let description = if pkg.description.len() > 36 {
                format!("{}...", &pkg.description[..33])
            } else {
                pkg.description.clone()
            };

            print!("{} {}", pkg.name, "(installed)".green());
            print!("{}", " ".repeat(padding));
            println!("{:<12} {}", "binary".cyan(), description);
        } else {
            let description = if pkg.description.len() > 36 {
                format!("{}...", &pkg.description[..33])
            } else {
                pkg.description.clone()
            };

            println!("{:<30} {:<12} {}", pkg.name, "binary".cyan(), description);
        }
    }

    // Print scripts
    for script in &scripts {
        // Get the best compatible script type for display
        let script_type_display = script
            .get_compatible_script()
            .map(|(st, _)| st.display_name().to_lowercase())
            .unwrap_or_else(|| script.platforms_display().to_lowercase());

        if installed.is_installed(&script.name) {
            // For installed scripts, calculate padding manually to account for "(installed)"
            let name_width = 30;
            let installed_suffix = " (installed)";
            let visible_length = script.name.len() + installed_suffix.len();
            let padding = if visible_length < name_width {
                name_width - visible_length
            } else {
                1
            };

            let description = if script.description.len() > 36 {
                format!("{}...", &script.description[..33])
            } else {
                script.description.clone()
            };

            print!("{} {}", script.name, "(installed)".green());
            print!("{}", " ".repeat(padding));
            println!("{:<12} {}", script_type_display.magenta(), description);
        } else {
            let description = if script.description.len() > 36 {
                format!("{}...", &script.description[..33])
            } else {
                script.description.clone()
            };

            println!(
                "{:<30} {:<12} {}",
                script.name,
                script_type_display.magenta(),
                description
            );
        }
    }

    println!();
    println!(
        "Total: {} package(s), {} script(s) available ({} installed)",
        packages.len(),
        scripts.len(),
        installed.packages.len()
    );

    Ok(())
}
