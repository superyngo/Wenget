//! Initialize WenPM

use crate::core::Config;
use anyhow::Result;
use colored::Colorize;

/// Initialize WenPM (create directories and manifests)
pub fn run() -> Result<()> {
    println!("{}", "Initializing WenPM...".cyan());

    let config = Config::new()?;

    if config.is_initialized() {
        println!("{}", "✓ WenPM is already initialized".green());
        println!("  Root: {}", config.paths().root().display());
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
    println!("{}", "Next steps:".bold());
    println!("  1. Add packages: wenpm add <github-url>");
    println!("  2. Install:      wenpm install <package-name>");
    println!("  3. Set up PATH:  wenpm setup-path");

    Ok(())
}
