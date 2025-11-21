//! Update (Upgrade) command implementation

use crate::commands::add;
use crate::core::Config;
use crate::providers::GitHubProvider;
use anyhow::Result;
use colored::Colorize;

/// Upgrade installed packages
pub fn run(names: Vec<String>, yes: bool) -> Result<()> {
    // Handle "wenpm update self"
    if names.len() == 1 && names[0] == "self" {
        return upgrade_self();
    }

    let config = Config::new()?;

    // Load manifests from cache (includes local + bucket sources)
    let sources = config.get_packages_from_cache()?;
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
        let upgradeable = find_upgradeable(&sources, &installed, &github)?;

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
    add::run(to_upgrade, yes)
}

/// Find upgradeable packages
fn find_upgradeable(
    sources: &crate::core::SourceManifest,
    installed: &crate::core::InstalledManifest,
    github: &GitHubProvider,
) -> Result<Vec<(String, String, String)>> {
    let mut upgradeable = Vec::new();

    for (name, inst_pkg) in &installed.packages {
        if let Some(src_pkg) = sources.find_package(name) {
            // Fetch latest version from GitHub
            if let Ok(latest_version) = github.fetch_latest_version(&src_pkg.repo) {
                if inst_pkg.version != latest_version {
                    upgradeable.push((name.clone(), inst_pkg.version.clone(), latest_version));
                }
            }
        }
    }

    Ok(upgradeable)
}

/// Upgrade wenpm itself
fn upgrade_self() -> Result<()> {
    println!("{}", "Upgrading wenpm...".cyan());

    // Get current version
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    // Fetch latest release from GitHub
    let provider = GitHubProvider::new()?;
    let latest_version = provider.fetch_latest_version("https://github.com/superyngo/WenPM")?;

    println!("Latest version: {}", latest_version);

    if current_version == latest_version {
        println!("{}", "✓ Already up to date".green());
        return Ok(());
    }

    println!();
    println!(
        "{}",
        "Self-upgrade functionality will be available in the next update".yellow()
    );
    println!("For now, please manually download and install the latest version from:");
    println!(
        "  {}",
        "https://github.com/superyngo/WenPM/releases/latest".cyan()
    );

    Ok(())
}
