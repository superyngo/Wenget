# AGENTS.md

Guidelines for AI coding agents working on this Rust codebase.

## Build & Test Commands

```bash
# Build
cargo build              # Development build
cargo build --release    # Release build (optimized for size)

# Check without building
cargo check

# Run all tests
cargo test

# Run single test by name
cargo test test_config_creation

# Run tests in a specific module
cargo test bucket::tests
cargo test core::repair::tests

# Run tests with stdout/stderr visible
cargo test -- --nocapture

# Run specific test with output visible
cargo test test_add_bucket -- --nocapture

# Code quality
cargo fmt                # Format code
cargo fmt --check        # Check formatting without changing
cargo clippy             # Run linter

# Run development build
cargo run -- <command> [args]
cargo run -- add ripgrep
cargo run -- list --all
```

## Project Structure

```
src/
├── main.rs              # Entry point, command dispatch
├── cli.rs               # clap argument definitions
├── bucket.rs            # Bucket config management
├── cache.rs             # Manifest cache management
├── package_resolver.rs  # Package name/URL resolution
├── commands/            # CLI command implementations
│   ├── mod.rs
│   ├── add.rs, delete.rs, update.rs, list.rs, ...
├── core/                # Core data structures & utilities
│   ├── config.rs        # Config file management
│   ├── manifest.rs      # Package/script manifest structs
│   ├── platform.rs      # OS/arch detection & matching
│   ├── paths.rs         # Directory path management
│   ├── privilege.rs     # Root/admin detection
│   ├── repair.rs        # JSON file repair utilities
│   └── registry.rs      # Windows registry PATH ops
├── providers/           # External data sources
│   ├── github.rs        # GitHub API integration
│   └── base.rs          # Provider trait
├── installer/           # Binary/script installation
│   ├── extractor.rs     # Archive extraction
│   ├── shim.rs          # Windows shim creation
│   ├── symlink.rs       # Unix symlink creation
│   └── script.rs        # Script installation
├── downloader/          # File download with progress
└── utils/               # HTTP client, prompts
```

## Code Style Guidelines

### Imports

Order imports in groups separated by blank lines:
1. Standard library (`std::`)
2. External crates (alphabetical)
3. Internal crate modules (`crate::`, `super::`)

```rust
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::Config;
use super::base::SourceProvider;
```

### Error Handling

- Use `anyhow::Result<T>` for functions that can fail
- Use `thiserror` for defining custom error types
- Add context with `.context()` or `.with_context()`
- Return `anyhow::bail!()` for early error returns

```rust
pub fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;
    
    serde_json::from_str(&content)
        .context("Failed to parse JSON")
}
```

### Naming Conventions

- **Types/Structs/Enums**: `PascalCase` (`InstalledPackage`, `ScriptType`)
- **Functions/Methods**: `snake_case` (`fetch_latest_release`, `install_package`)
- **Constants**: `SCREAMING_SNAKE_CASE` (`INTERPRETER_CACHE`)
- **Modules**: `snake_case` (`package_resolver.rs`)
- **Boolean methods**: Use `is_`/`has_` prefix (`is_installed`, `has_updates`)

### Documentation

- Add `//!` module-level docs at the top of each file
- Use `///` for public function/struct documentation
- Document non-obvious behavior and edge cases

```rust
//! Bucket management for WenPM
//!
//! Buckets are remote manifest sources that can be added to WenPM.

/// A bucket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    /// Bucket name (unique identifier)
    pub name: String,
}
```

### Struct Patterns

- Derive common traits: `Debug, Clone, Serialize, Deserialize`
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
- Implement `Default` via `new()` method pattern
- Use `#[allow(dead_code)]` for intentionally unused helper methods

### Testing

- Place unit tests in the same file using `#[cfg(test)]` module
- Use `tempfile::TempDir` for file operation tests
- Test both success and error paths

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bucket_config_new() {
        let config = BucketConfig::new();
        assert_eq!(config.buckets.len(), 0);
    }
}
```

### CLI Arguments (clap)

- Use derive macros with `#[command]` and `#[arg]` attributes
- Add `visible_alias` for common shortcuts
- Use `short` and `long` for flags

```rust
#[derive(Parser)]
#[command(name = "wenget")]
pub struct Cli {
    #[arg(short = 'y', long)]
    pub yes: bool,
}
```

### Platform-Specific Code

Use conditional compilation for platform differences:

```rust
#[cfg(unix)]
use crate::installer::create_symlink;

#[cfg(windows)]
use crate::installer::create_shim;
```

### Logging

- Use `log` macros (`log::info!`, `log::warn!`, `log::error!`)
- Use `colored` crate for user-facing terminal output
- Print user messages to stdout, errors to stderr

```rust
log::info!("Fetching bucket from {}", url);
println!("{} {}", "Installing".cyan(), name);
eprintln!("{} {}", "Error:".red().bold(), e);
```

## Key Implementation Notes

- `Config` is the main entry point for loading/saving state
- `WenPaths` manages all directory paths (user vs system level)
- Platform detection uses fuzzy matching for binary names
- Cache has 24-hour TTL; invalidate after bucket changes
- Always update `installed.json` after install/remove operations
- JSON files have auto-repair on parse errors with backup

## Release Workflow

When releasing a new version, follow these steps:

### 1. Commit All Updates
- Ensure all changes are committed
- Organize commit messages clearly describing the updates
- Run `cargo fmt` and `cargo clippy` before committing

### 2. Determine Version Number
- Review changes to suggest appropriate version bump:
  - **PATCH** (x.x.+1): Bug fixes, minor improvements
  - **MINOR** (x.+1.0): New features, backward compatible
  - **MAJOR** (+1.0.0): Breaking changes
- Ask user to confirm the new version number

### 3. Collect Version Changes
- Gather all changes since last release
- Categorize: Added, Changed, Fixed, Removed
- Write clear, user-facing descriptions

### 4. Update Documentation
- Update `Cargo.toml` version field
- Update `README.md` if needed (new features, usage changes)
- Update `CHANGELOG.md` with new version section:
  ```markdown
  ## [x.x.x] - YYYY-MM-DD
  ### Added
  - New feature description
  ### Changed
  - Modified behavior
  ### Fixed
  - Bug fix description
  ```

### 5. Create Tag and Release
```bash
# Create annotated tag
git tag -a vX.X.X -m "Release vX.X.X"

# Or with release notes
git tag -a vX.X.X -m "Release vX.X.X

- Feature 1
- Fix 1
"
```

### 6. Push and Publish
```bash
# Push commits and tags
git push origin main
git push origin vX.X.X

# GitHub Actions will automatically build and create release
```

The `.github/workflows/release.yml` workflow will:
- Trigger on version tags (`v*`)
- Build binaries for all platforms
- Create GitHub Release with artifacts
