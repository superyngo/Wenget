//! Bucket command implementation

use crate::bucket::Bucket;
use crate::core::Config;
use anyhow::Result;
use colored::Colorize;

/// Bucket subcommands
pub enum BucketCommand {
    Add { name: String, url: String },
    Del { names: Vec<String> },
    List,
    Refresh,
}

/// Run bucket command
pub fn run(cmd: BucketCommand) -> Result<()> {
    match cmd {
        BucketCommand::Add { name, url } => run_add(name, url),
        BucketCommand::Del { names } => run_del(names),
        BucketCommand::List => run_list(),
        BucketCommand::Refresh => run_refresh(),
    }
}

/// Add a bucket
fn run_add(name: String, url: String) -> Result<()> {
    let config = Config::new()?;

    // Ensure WenPM is initialized
    if !config.is_initialized() {
        config.init()?;
    }

    println!("{} bucket '{}'...\n", "Adding".cyan(), name);

    // Load bucket config
    let mut bucket_config = config.get_or_create_buckets()?;

    // Create bucket
    let bucket = Bucket {
        name: name.clone(),
        url: url.clone(),
        enabled: true,
        priority: 100,
    };

    // Try to add bucket
    if bucket_config.add_bucket(bucket) {
        // Save config
        config.save_buckets(&bucket_config)?;

        println!("{} Bucket '{}' added", "✓".green(), name);
        println!("  URL: {}", url);

        // Invalidate cache so it will be rebuilt on next access
        config.invalidate_cache()?;

        println!();
        println!("{}", "Cache will be rebuilt on next command.".cyan());
    } else {
        println!("{} Bucket '{}' already exists", "✗".red(), name);
        return Ok(());
    }

    Ok(())
}

/// Delete buckets
fn run_del(names: Vec<String>) -> Result<()> {
    let config = Config::new()?;

    // Load bucket config
    let mut bucket_config = config.get_or_create_buckets()?;

    if bucket_config.buckets.is_empty() {
        println!("{}", "No buckets configured".yellow());
        return Ok(());
    }

    if names.is_empty() {
        println!("{}", "No bucket names provided".yellow());
        println!("Usage: wenpm bucket del <name>...");
        return Ok(());
    }

    println!("{} bucket(s)...\n", "Deleting".cyan());

    let mut deleted = 0;
    let mut not_found = 0;

    for name in names {
        print!("  {} {} ... ", "Deleting".cyan(), name);

        if bucket_config.remove_bucket(&name) {
            println!("{}", "Deleted".green());
            deleted += 1;
        } else {
            println!("{}", "Not found".yellow());
            not_found += 1;
        }
    }

    // Save config
    config.save_buckets(&bucket_config)?;

    // Invalidate cache
    if deleted > 0 {
        config.invalidate_cache()?;
    }

    // Summary
    println!();
    println!("{}", "Summary:".bold());
    if deleted > 0 {
        println!("  {} {} bucket(s) deleted", "✓".green(), deleted);
    }
    if not_found > 0 {
        println!("  {} {} bucket(s) not found", "•".yellow(), not_found);
    }

    println!();
    println!("Total buckets: {}", bucket_config.buckets.len());

    Ok(())
}

/// List buckets
fn run_list() -> Result<()> {
    let config = Config::new()?;

    // Load bucket config
    let bucket_config = config.get_or_create_buckets()?;

    if bucket_config.buckets.is_empty() {
        println!("{}", "No buckets configured".yellow());
        println!();
        println!("Add a bucket with: wenpm bucket add <name> <url>");
        return Ok(());
    }

    // Print header
    println!("{}", "Configured buckets:".bold());
    println!();
    println!(
        "{:<20} {:<10} {}",
        "NAME".bold(),
        "STATUS".bold(),
        "URL".bold()
    );
    println!("{}", "─".repeat(80));

    // Print buckets
    for bucket in &bucket_config.buckets {
        let status = if bucket.enabled {
            "enabled".green()
        } else {
            "disabled".yellow()
        };

        println!(
            "{:<20} {:<18} {}",
            bucket.name.green(),
            status.to_string(),
            bucket.url
        );
    }

    println!();
    println!("Total: {} bucket(s)", bucket_config.buckets.len());

    Ok(())
}

/// Refresh cache from buckets
fn run_refresh() -> Result<()> {
    let config = Config::new()?;

    println!("{} manifest cache...\n", "Refreshing".cyan());

    // Force rebuild cache
    let cache = config.rebuild_cache()?;

    println!();
    println!("{}", "Summary:".bold());

    // Show source statistics
    for (source_name, info) in &cache.sources {
        println!(
            "  {} {} - {} package(s)",
            "✓".green(),
            source_name,
            info.package_count
        );
    }

    println!();
    println!("Total packages in cache: {}", cache.packages.len());
    println!("{}", "Cache refreshed successfully!".green());

    Ok(())
}
