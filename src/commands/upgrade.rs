//! Upgrade command implementation

use crate::commands::install;
use crate::core::Config;
use crate::providers::SourceProvider;
use anyhow::Result;
use colored::Colorize;

/// Upgrade installed packages
pub fn run(names: Vec<String>, yes: bool) -> Result<()> {
    // Handle "wenpm upgrade self"
    if names.len() == 1 && names[0] == "self" {
        return upgrade_self();
    }

    let config = Config::new()?;

    // Load manifests
    let sources = config.get_or_create_sources()?;
    let installed = config.get_or_create_installed()?;

    if installed.packages.is_empty() {
        println!("{}", "No packages installed".yellow());
        return Ok(());
    }

    // Determine which packages to upgrade
    let to_upgrade: Vec<String> = if names.is_empty() || (names.len() == 1 && names[0] == "all") {
        // List upgradeable packages
        let upgradeable = find_upgradeable(&sources, &installed);

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

    // Use install command to upgrade (reinstall)
    install::run(to_upgrade, yes)
}

/// Find upgradeable packages
fn find_upgradeable(
    sources: &crate::core::SourceManifest,
    installed: &crate::core::InstalledManifest,
) -> Vec<(String, String, String)> {
    let mut upgradeable = Vec::new();

    for (name, inst_pkg) in &installed.packages {
        if let Some(src_pkg) = sources.find_package(name) {
            if inst_pkg.version != src_pkg.latest {
                upgradeable.push((
                    name.clone(),
                    inst_pkg.version.clone(),
                    src_pkg.latest.clone(),
                ));
            }
        }
    }

    upgradeable
}

/// Upgrade wenpm itself
fn upgrade_self() -> Result<()> {
    println!("{}", "Upgrading wenpm...".cyan());

    // Get current version
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    // Fetch latest release from GitHub
    let provider = crate::providers::GitHubProvider::new()?;
    let package = provider.fetch_package("https://github.com/superyngo/WenPM")?;

    println!("Latest version: {}", package.latest);

    if current_version == package.latest {
        println!("{}", "✓ Already up to date".green());
        return Ok(());
    }

    println!();
    println!("{}", "Self-upgrade functionality will be available in the next update".yellow());
    println!("For now, please manually download and install the latest version from:");
    println!("  {}", "https://github.com/superyngo/WenPM/releases/latest".cyan());

    Ok(())
}
