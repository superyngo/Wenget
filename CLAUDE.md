# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Wenget is a cross-platform package manager for GitHub binaries, written in Rust. It simplifies installation and management of command-line tools distributed through GitHub Releases by automatically detecting platforms, downloading appropriate binaries, and managing them in `~/.wenget/`.

## Build and Development Commands

### Building
```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# The release binary will be at target/release/wenget (or wenget.exe on Windows)
```

### Testing
```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests with output visible
cargo test -- --nocapture

# Run tests for a specific module
cargo test bucket::tests
```

### Running Locally
```bash
# Run development build
cargo run -- <command> [args]

# Examples:
cargo run -- init
cargo run -- add ripgrep
cargo run -- list
```

### Code Quality
```bash
# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture

### Core Module Structure

The codebase follows a layered architecture:

1. **CLI Layer** (`src/cli.rs`, `src/main.rs`)
   - Parses command-line arguments using `clap`
   - Routes commands to appropriate handlers in `src/commands/`

2. **Command Layer** (`src/commands/`)
   - Each command has its own module (add.rs, delete.rs, update.rs, etc.)
   - Commands orchestrate core functionality and handle user interaction

3. **Core Functionality** (`src/core/`)
   - `config.rs`: Central configuration and state management
   - `manifest.rs`: Package and script manifest structures
   - `paths.rs`: Directory structure management (user/system-level)
   - `platform.rs`: Platform detection and binary selection logic
   - `privilege.rs`: Elevated privilege detection (root/Administrator)
   - `registry.rs`: Windows registry PATH modification
   - `repair.rs`: JSON corruption detection and auto-repair

4. **Package Resolution** (`src/package_resolver.rs`)
   - Determines if input is a package name or GitHub URL
   - Resolves packages from cache or directly from GitHub
   - Supports glob patterns for package names (e.g., `rip*`)

5. **Provider System** (`src/providers/`)
   - `github.rs`: GitHub API integration for fetching releases and binaries
   - Handles rate limiting and API communication

6. **Bucket System** (`src/bucket.rs`, `src/cache.rs`)
   - Buckets are remote manifest collections (JSON files)
   - Cache merges all bucket sources to reduce GitHub API calls
   - Cache has 24-hour TTL by default

7. **Installation System** (`src/installer/`)
   - `extractor.rs`: Archive extraction (zip, tar.gz, tar.xz)
   - `script.rs`: Script installation (PowerShell, Bash, Python)
   - `shim.rs`: Windows shim creation
   - `symlink.rs`: Unix symlink creation

8. **Download System** (`src/downloader/`)
   - Handles multi-threaded downloads with progress bars
   - Caches downloaded archives

9. **Utility System** (`src/utils/`)
   - `http.rs`: HTTP client wrapper for GitHub API
   - `prompt.rs`: User confirmation utilities (`confirm()`, `confirm_no_default()`)

### Key Data Flow

**Installing a package from a bucket:**
1. User runs `wenget add ripgrep`
2. `PackageResolver` checks cache for package name
3. Cache lookup finds package from bucket manifest
4. `GitHubProvider` fetches latest release if needed
5. `Platform` selects appropriate binary for current OS/arch
6. `Downloader` downloads and caches the archive
7. `Installer` extracts binary to `~/.wenget/apps/ripgrep/`
8. Shim/symlink created in `~/.wenget/bin/`
9. Installation recorded in `installed.json`

**Installing from a GitHub URL:**
1. User runs `wenget add https://github.com/user/repo`
2. `PackageResolver` identifies it as a DirectUrl
3. `GitHubProvider` fetches release info directly from GitHub
4. Platform detection and installation proceed as above
5. Source recorded as `DirectRepo` in `installed.json`

### Directory Structure

**User-level (`~/.wenget/`):**
```
~/.wenget/
├── apps/              # Installed binaries (each in own subdirectory)
├── bin/               # Shims/symlinks (added to PATH)
├── cache/             # Downloaded archives and manifest cache
│   ├── manifest-cache.json
│   └── downloads/
├── buckets.json       # Bucket configuration
└── installed.json     # Installed package metadata
```

**System-level (when running as root/Administrator):**
- Linux: `/opt/wenget/` with symlinks in `/usr/local/bin/`
- Windows: `%ProgramW6432%\wenget\` with bin added to system PATH

The `WenPaths` struct in `src/core/paths.rs` auto-detects privilege level via `is_elevated()` from `privilege.rs`.

## Important Implementation Details

### Platform Detection
- Platform detection is in `src/core/platform.rs`
- Uses fuzzy matching for binary names (e.g., "ripgrep-linux-x64" matches "linux-x86_64")
- Automatically removes platform suffixes from command names
- Fallback detection for ambiguous filenames

### Package Sources
Packages can come from three sources (see `PackageSource` enum in `src/core/manifest.rs`):
- `Bucket { name }`: From a bucket manifest
- `DirectRepo { url }`: Directly from a GitHub URL
- `LocalScript { original_path }`: From a local file

### Script Support
- Scripts can be PowerShell (.ps1), Bash (.sh), Batch (.bat), or Python (.py)
- Scripts are installed to `~/.wenget/apps/<script-name>/`
- Shims are created to execute them from `~/.wenget/bin/`

### GitHub API Rate Limits
- Unauthenticated: 60 requests/hour
- Bucket-based installs don't consume API calls (use cached data)
- Direct URL installs consume ~2 API calls per package
- `wenget update` checks each installed package for updates

### Self-Update
- `wenget update self` updates Wenget itself
- Windows: Uses background cleanup script to handle locked executable
- Unix: Atomic rename with fallback

### Auto-Repair
- Corrupted JSON files (installed.json, buckets.json, manifest-cache.json) are automatically detected
- `repair.rs` creates backups before attempting repair
- Graceful degradation when config files are unreadable

## Testing Notes

- Unit tests are inline with modules using `#[cfg(test)]`
- Integration tests would go in `tests/` directory (not currently present)
- Use `tempfile` crate for testing file operations (see `dev-dependencies`)

## Release Configuration

The `Cargo.toml` has aggressive optimization for small binary size:
- `opt-level = "z"`: Optimize for size
- `lto = true`: Link-time optimization
- `strip = true`: Strip debug symbols
- `panic = "abort"`: Smaller panic handler

## Cross-Platform Considerations

- Windows uses shims (`.cmd` files) in `~/.wenget/bin/`
- Unix uses symlinks in `~/.wenget/bin/`
- Archive formats: .zip (Windows), .tar.gz, .tar.xz (Unix)
- PATH modification differs by platform (handled by `wenget init`)

## Common Gotchas

- Always update `installed.json` when installing/removing packages
- Cache must be rebuilt when buckets are added/removed
- Platform detection requires exact platform string matching for manifest entries
- Shims on Windows must handle spaces in paths correctly
- System-level vs user-level paths are auto-detected via `privilege.rs`
- Self-deletion (`wenget del self`) has special handling when executable is inside .wenget
