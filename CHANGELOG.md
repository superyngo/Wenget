# Changelog

All notable changes to Wenget will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Update Manifest Workflow** - Fix detection of new manifest.json when file was deleted (2026-01-18)
  - Changed `git diff` to `git add` + `git diff --staged` to detect untracked new files

## [2.0.1] - 2026-01-18

### Fixed

- **Batch Installation Error Handling** - Single package failure no longer interrupts entire operation
  - Modified `add` command to continue processing remaining packages when one fails
  - Changed `?` operators to explicit error handling with `fail_count` tracking
  - Applies to: script installations, local file installations, URL installations, and package installations
  - Failed packages are now reported in summary instead of aborting the entire batch
  - Improves `update` command behavior: failed package updates no longer block other updates

- **Release Workflow** - Fix "Update bucket binary" step failing due to .gitignore
  - Changed `git add` to `git add -f` to force-add ignored bucket/wenget binary

## [2.0.0] - 2026-01-16

### Added

- **Windows ARM64 Build Support** - Added aarch64-pc-windows-msvc target to CI/CD pipeline
  - New build artifact: `wenget-windows-aarch64.exe`
  - Supports Windows ARM64 devices (Snapdragon X Elite/Plus laptops, Windows Dev Kit 2023)
  - Uses conservative optimization settings to avoid antivirus false positives (same as other Windows builds)
  - Platform detection already supports ARM64 via existing `aarch64`/`arm64` keywords
  - Includes fallback to x86_64 emulation when ARM64 binary unavailable
  - Total build targets increased from 12 to 13 platforms

- **Version Selection** - Add `-v/--ver` flag to install specific package versions
  - Usage: `wenget add ripgrep -v 14.0.0` or `wenget add ripgrep --ver v14.0.0`
  - Supports both `v1.0.0` and `1.0.0` formats (automatically handles 'v' prefix)
  - Shows clear error message if specified version doesn't exist
  - Note: Use `--verbose` (no short form) for verbose logging to avoid flag conflicts

### Changed

- **Multiple Executable Detection** - Auto-install all executables with valid permissions
  - Packages like `uv` (containing both `uv` and `uvx`) now install all executables automatically
  - Changed from selecting only top-scoring candidate (score >= 80) to all candidates with score > 0
  - Auto-selects up to 3 executables; shows interactive menu for more than 3 (unless `--yes` flag)
  - Captures executables with execution permission even if name doesn't match package name

- **Architecture Filtering** - Improved unsupported architecture detection
  - Expanded UNSUPPORTED_ARCHS list to include all PowerPC variants (ppc, powerpc, powerpc64, powerpc64le)
  - Added RISC-V variants (riscv, riscv32, riscv64)
  - Added MIPS variants (mips, mips64, mipsel, mips64el)
  - Added exotic architectures (alpha, sh4, hppa, ia64, loong64, loongarch64, s390)
  - Prevents misclassification of unsupported binaries (e.g., `uv-powerpc64-unknown-linux-gnu.tar.gz` no longer incorrectly categorized under `linux-x86_64-gnu`)
  - Added pattern detection for unknown architecture-like keywords
  - Windows ARM64 support confirmed in test platforms

### Fixed

- Resolved clippy warning for too many function arguments

### Technical

- Modified executable selection logic in `src/commands/add.rs`
- Added `fetch_release_by_tag()` method to `GitHubProvider` for version-specific fetches
- Added `fetch_package_by_version()` method to fetch packages for specific versions
- Updated install flow to use custom version when specified
- Enhanced `ParsedAsset::contains_unknown_arch_pattern()` to detect unrecognized architecture patterns
- Updated architecture matching logic to check for unsupported patterns before falling back to OS defaults
- Code formatting improvements and clippy compliance

## [1.3.3] - 2026-01-16

### Changed

- **Platform Detection** - Improved architecture keyword matching
  - Added `386` keyword for I686 architecture detection
  - Added `armv6` keyword for Armv7 architecture detection
  - Better compatibility with Go-style binary naming conventions

## [1.3.2] - 2026-01-15

### Added

- **Interactive Self-Deletion Menu** - Granular control over what gets removed
  - New interactive menu when running `wenget del self` (without `-y` flag)
  - Users can choose which components to remove:
    - Apps & data (~/.wenget/)
    - PATH configuration
    - Wenget binary
  - Multiple selections supported via checkboxes
  - `-y` flag still removes everything for non-interactive use

### Fixed

- **Package Name Matching** - Fixed update command for packages with platform suffixes
  - `wenget update` now correctly finds packages even when installed with variant suffixes (e.g., "uv-pc")
  - Fallback matching strips trailing platform-related suffixes to find base package name
  - Handles cases where old installations have platform identifiers in the package name

- **Self-Update Platform Detection** - Better binary matching for self-updates
  - `wenget update self` now uses same smart platform matching as `add` command
  - Includes libc detection (musl vs glibc), compiler variants, and fallback support
  - Shows informative messages when using compatible fallback binaries
  - More reliable updates across different Linux distributions

### Technical

- Added `show_removal_menu()` function using `dialoguer::MultiSelect`
- Added `RemovalOptions` struct for tracking user's deletion preferences
- Enhanced `extract_variant_from_asset()` to include "pc" suffix (common in Rust target triples)
- Refactored self-update to use `Platform::find_best_match()` API

## [1.3.1] - 2026-01-14

### Fixed

- **CI/CD Workflow Improvements**
  - Added rebase before push in update-manifest workflow to prevent conflicts
  - Fixed bucket binary push logic in release workflow
  - Improved manifest update automation reliability

## [1.3.0] - 2026-01-14

### Added

- **Integrated Bucket Repository** - Merged official bucket repository into main repo
  - New `bucket/` directory containing package manifests
  - Includes README with bucket usage instructions
  - Centralized manifest management in the main repository
  - Simplifies bucket maintenance and distribution

### Changed

- Bucket manifest structure now co-located with main codebase
- Official bucket available directly from the main repository

## [1.2.0] - 2026-01-14

### Added

- **Multi-Package Variant Support** - Install multiple binary variants from a single package
  - Each platform can now have multiple binaries (e.g., baseline, desktop, musl, gnu variants)
  - Interactive MultiSelect dialog to choose which variants to install
  - `--yes` flag installs all variants automatically
  - New `parent_package` tracking for variant relationships
  - New `asset_name` field to track original asset filenames

- **Variant-Aware Commands** - All commands now understand package variants
  - `list`: Tree structure display showing parent packages and their variants (├─, └─)
  - `delete`: Select which variants to remove with MultiSelect dialog
  - `update`: Automatically includes all variants when updating a parent package
  - `info`: Shows variant information and displays multiple packages per platform

- **AGENTS.md** - AI coding agent guidelines for this repository
  - Build/test commands, code style guidelines, and release workflow

### Changed

- **Manifest Structure** - `platforms` now stores `Vec<PlatformBinary>` instead of single `PlatformBinary`
  - Bucket manifests capture ALL available variants for each platform
  - Better support for projects with multiple build configurations

- **Info Command UI** - Beautiful box-style header with improved formatting

- **Input Detection** - GitHub repo URLs now correctly treated as package names
  - Distinguishes between `github.com/owner/repo` (package) and `/releases/download/` (direct URL)

- **Download URL Display** - Shows all download URLs when multiple packages available

### Technical

- Added `extract_variant_from_asset()` function for variant name extraction
- Added `generate_installed_key()` function for consistent installed.json keys
- `PlatformBinary` now includes `asset_name` field
- `InstalledPackage` now includes `asset_name` and `parent_package` fields
- `BinarySelector::extract_platforms()` returns `HashMap<String, Vec<BinaryAsset>>`
- Fixed clippy warnings: `is_some_and()` and `or_default()` usage

## [1.1.1] - 2026-01-12

### Fixed

- **Token Propagation in Bucket Create** - Fixed GitHub token not being passed to GitHubProvider
  - `ManifestGenerator::with_token()` now correctly passes token to `GitHubProvider::with_token()`
  - Previously token was only passed to `HttpClient`, causing API rate limiting in CI/CD
  - This fixes Gist and repo fetching failures when using `bucket create` with authentication

### Documentation

- Updated README with comprehensive `bucket create` command documentation
  - Added usage examples for token authentication
  - Documented update mode options (overwrite/incremental)
  - Added all available command flags and options

## [1.1.0] - 2026-01-12

### Added

- **GitHub Token Support for Bucket Creation** - Higher API rate limits for manifest generation
  - New `-t, --token` option for `bucket create` command
  - Automatically reads `GITHUB_TOKEN` environment variable if no token provided
  - Authenticated requests get 5,000 requests/hour vs 60/hour unauthenticated
  - Token now properly passed to all GitHub API calls (repos, releases, gists)

- **Update Mode for Manifest Generation** - Control how existing manifests are updated
  - New `-u, --update-mode` option with two modes:
    - `overwrite`: Replace entire manifest file (default behavior)
    - `incremental`: Merge with existing manifest, keeping entries not in current run
  - Enables CI/CD pipelines to run non-interactively with `--update-mode overwrite`

- **Uncompressed Binary Support** - Install binaries without archive wrappers
  - Detects platform-specific binaries without file extensions (e.g., `m3u8-linux-amd64`)
  - Recognizes binaries with platform keywords (linux, darwin, windows, x86_64, amd64, etc.)
  - Properly handles repos like [llychao/m3u8-downloader](https://github.com/llychao/m3u8-downloader)

- **Enhanced Info Command** - Shows info for manually installed packages
  - Now displays details for packages installed via direct URL or local script
  - Falls back to `installed.json` when package not found in cache
  - Shows source type, origin URL, command name, and installed files

- **Download URL Display** - Shows resolved download URLs during installation
  - Displays the actual binary URL before confirmation
  - Helps users verify what will be downloaded

### Fixed

- **Token Propagation** - `GitHubProvider` now accepts token for authenticated API calls
  - Previously `bucket create` passed token to `HttpClient` but not to `GitHubProvider`
  - This caused API rate limiting issues in CI/CD environments

- **Bucket Fetch Timeout** - Reduced timeout from 30s to 10s for faster failure detection
  - Improves responsiveness when bucket URLs are unreachable

### Technical

- Added `GitHubProvider::with_token()` constructor for authenticated API access
- Added `FileExtension::UncompressedBinary` variant for extensionless binaries
- Added `is_likely_binary_without_extension()` helper for binary detection
- New `display_installed_only_info()` function in info command
- All 72+ unit tests passing

## [1.0.0] - 2026-01-05

### 🎉 Stable Release

This marks the first stable release of Wenget! After extensive development and testing,
Wenget is now production-ready for managing GitHub binaries across platforms.

### Added

- **User Confirmation Utilities** - New `utils/prompt` module
  - `confirm()` function for [Y/n] prompts
  - `confirm_no_default()` function for [y/N] prompts
  - Reduces code duplication across command modules

- **Interpreter Caching** - Performance optimization for script support
  - Cached interpreter availability detection using `OnceLock`
  - PowerShell, Bash, and Python availability checked once per session
  - Significantly faster script compatibility checks

- **Batch Script Path Escaping** - Security improvement
  - New `escape_batch_path()` function for special character handling
  - Properly escapes `&`, `|`, `<`, `>`, `^`, `%`, `!` in paths
  - Prevents potential command injection in Windows batch shims

### Changed

- **Removed Default Trait Panic Risk**
  - Removed `impl Default for Config` to avoid panic on home directory detection failure
  - Removed `impl Default for WenPaths` for the same reason
  - Applications should use explicit `Config::new()` and `WenPaths::new()` calls

- **Script Preference Order** - Extracted to shared function
  - New `ScriptType::preference_order()` returns platform-specific script preference
  - Windows: PowerShell > Batch > Python > Bash
  - Unix: Bash > Python > PowerShell
  - Eliminates code duplication between `get_compatible_script()` and `get_installable_script()`

- **Registry Operations Refactored** (Windows)
  - Extracted shared `modify_system_path_inner()` function
  - Reduced code duplication between `add_to_system_path()` and `remove_from_system_path()`

### Fixed

- **Improved Error Logging**
  - Backup failures now logged with `log::warn!` instead of silent ignore
  - File cleanup failures (temp files, old executables) now logged
  - Better debugging experience for troubleshooting

- **Clippy Warnings Resolved**
  - Collapsed nested `if` statements for better readability
  - Removed unused imports across command modules
  - All clippy warnings addressed

### Technical

- **Code Quality Improvements**
  - Reduced ~120 lines of duplicated code
  - Added ~80 lines of new utility functions
  - All 68 unit tests passing
  - No clippy warnings in codebase

### Documentation

- Added ExecutionPolicy Bypass explanation in script shim generation
- Improved comments for `unreachable!()` macro usage
- Updated inline documentation for new utility functions

## [0.9.1] - 2026-01-05

### Fixed

- Resolved clippy warnings for better code quality

## [0.9.0] - 2026-01-03

### Added

- **Multi-Platform Variant Support** - Bucket manifests now include ALL platform variants
  - `bucket create` now collects all available platform variants (musl, gnu, msvc, etc.) instead of selecting only the highest-scored one
  - Manifests can now contain both `linux-x86_64-musl` and `linux-x86_64-gnu` simultaneously
  - Enables users to choose their preferred variant during installation

- **Smart Platform Fallback System** - Intelligent platform matching with cross-compatibility
  - Automatically suggests compatible fallback platforms when exact match isn't available
  - **Architecture Fallback**:
    - 64-bit systems can install 32-bit binaries (with user confirmation)
    - macOS ARM (Apple Silicon) can install x86_64 binaries via Rosetta 2 (with user confirmation)
    - Windows ARM can install x86_64 binaries via emulation (with user confirmation)
  - **Compiler/libc Fallback**:
    - Linux systems can use musl binaries on glibc systems (automatic, statically linked)
    - Linux systems can use glibc binaries on musl systems (with user confirmation)
    - Windows systems can use different compiler variants (automatic)
  - Clear user prompts explaining the fallback type and compatibility implications
  - Preserves existing scoring-based platform preference system

- **New Platform Matching API**
  - Added `FallbackType` enum to categorize different fallback scenarios
  - Added `PlatformMatch` struct to provide detailed matching information
  - Added `Platform::find_best_match()` for intelligent platform selection
  - Added `BinarySelector::select_all_for_platform()` to retrieve all matching variants
  - Added `ScriptItem::get_installable_script()` for proper interpreter verification during installation

### Changed

- **Enhanced `extract_platforms()`** - Now returns all platform variants instead of only the best match
- **Improved Installation Flow** - Uses new `find_best_match()` API for smarter platform selection
- **Better User Experience** - Informative messages when using fallback platforms

### Fixed

- **Script Platform Compatibility** - Fixed incorrect script platform detection on Windows
  - `list --all` and `info` commands no longer show Bash scripts as compatible on Windows unless Bash is actually installed
  - Separated `is_os_compatible()` (for display) from `is_supported_on_current_platform()` (for installation verification)
  - Installation now properly checks if script interpreter exists before allowing installation

### Technical

- Added comprehensive test coverage for multi-platform and fallback scenarios
- Added `#[allow(dead_code)]` annotations for future-facing APIs
- Fixed all clippy warnings and code formatting issues
- Updated internal platform matching logic to support multiple variants per OS/architecture combination

## [0.8.0] - 2026-01-03

### Added

- **System-Level Installation** - Install scripts now auto-detect elevated privileges
  - Linux/macOS: Running as root installs to `/opt/wenget/app` with symlinks in `/usr/local/bin`
  - Windows: Running as Administrator installs to `%ProgramW6432%\wenget` with system PATH
  - User-level installation remains the default behavior

### Changed

- Refactored bucket management system
- Improved core paths module architecture
- Updated documentation with system-level installation guide

## [0.7.2] - 2025-12-30

### Fixed

- Windows compatibility improvements
- Minor bug fixes and code cleanup

## [0.7.1] - 2025-12-30

### Fixed

- **Linux Self-Update** - Resolved "Text file busy" error on Alpine Linux and other Unix systems
  - Implemented robust atomic rename strategy for updating the running executable
  - Added fallback mechanism to copy if rename fails (cross-filesystem)
  - Improved permission handling and error recovery during updates
- **Code Maintenance**
  - Fixed various clippy warnings and unused imports
  - Improved code hygiene in installer and command modules

## [0.7.0] - 2025-12-30

### Added

- **Platform Selection** - Explicit platform selection for installations
  - Added `-p`/`--platform` flag to `add` command
  - Allows installing binaries for specific platforms (e.g., `linux-x64`)
  - Supports both package and manual URL installations

- **Universal Installation Support** - Complete "Install anything" capability
  - **Local Binaries**: Install local `.exe` or binary files directly (`wenget add ./mytool.exe`)
  - **Local Archives**: Install from local `.zip`/`.tar.gz` (`wenget add ./tools.zip`)
  - **Direct URLs**: Install binaries/archives from any URL (`wenget add https://example.com/tool.zip`)
  - All installations generate shims and integrate seamlessly

- **UX Enhancements**
  - **Command Aliases**: Added convenient short aliases
    - `i` for `info`
    - `rm`, `uninstall` for `del`
  - **Source Visibility**: `wenget list --all` now shows the `SOURCE` column
    - Identify packages from Buckets, Direct URLs, or Scripts instantly

## [0.6.3] - 2025-12-08

### Fixed

- 修復 Linux 平台 update self 功能
- Removed unsupported architectures: s390x, ppc64, ppc64le, riscv64, mips
- Code formatting and clippy linting improvements

### Backward Compatible

- Platform string format unchanged: {os}-{arch} or {os}-{arch}-{compiler}
- Existing manifests continue to work
- New compiler-specific keys are additive

## [0.6.2] - 2025-12-08

### Fixed

- Minor bug fixes and improvements

## [0.6.1] - 2025-12-08

### Fixed

- Code quality improvements
  - Fixed clippy warnings for dead code in tests
  - Fixed pointer argument linting (PathBuf → Path)
  - Added allow attributes where appropriate
- Enhanced code formatting compliance with cargo fmt

## [0.6.0] - 2025-12-07

### Added

- **Advanced platform detection system** - Refactored binary matching logic for better compatibility
  - New 4-component parsing: file extension + OS + architecture + compiler/libc
  - `Compiler` enum supporting GNU, musl, and MSVC variants
  - Context-aware `x86` keyword resolution (macOS → x86_64, others → i686)
  - FreeBSD support with explicit architecture requirement
  - Compiler priority system: Linux prefers musl > gnu, Windows prefers msvc > gnu

### Improved

- **Default architecture handling** - Intelligent fallback for ambiguous binaries
  - Windows/Linux default to x86_64 when architecture not specified
  - macOS defaults to aarch64 (Rosetta 2 can run x86_64 binaries)
  - FreeBSD requires explicit architecture (no default)
  - Explicit architecture matches scored higher than defaults

### Changed

- **Platform detection scoring** - New 4-component scoring algorithm

  - OS match: +100 (mandatory)
  - Explicit arch match: +50
  - Default arch match: +25
  - Compiler priority: +10/20/30 based on OS preference
  - File format: +2 to +5

- Complete refactor of `src/core/platform.rs` with `ParsedAsset` struct
- Added `FileExtension` enum for archive format detection
- Added 17 comprehensive test cases for platform detection

## [0.5.3] - 2025-12-03

### Added

- **Fallback platform detection** - Intelligent handling of release files with ambiguous names
  - Added fallback OS keywords: "win", "mac", "osx", "msvc" for broader matching
  - Automatic architecture assumption when explicit info is missing:
    - Windows/Linux without arch → assumes x86_64 (most common)
    - macOS without arch → assumes aarch64 (Apple Silicon standard)
  - Fallback matches scored lower (125 points) than exact matches (150 points)
  - Warning messages displayed when using fallback assumptions
  - Enables detection of packages like `gitui-win.tar.gz` and `app-mac.zip`

### Fixed

- **Platform detection for ambiguous filenames** - Files like `gitui-win.tar.gz` are now correctly detected
  - Previously required explicit architecture in filename (e.g., `win64`, `x86_64`)
  - Now supports generic OS-only filenames with intelligent fallback
  - Maintains preference for explicitly-named binaries over fallback matches

### Changed

- **.msi file handling** - Removed support for .msi installer packages
  - .msi files now properly excluded from binary selection
  - Focuses on portable archive formats (tar.gz, zip, exe)
  - Avoids conflicts with Windows installer packages that need special handling

### Technical

- Enhanced `BinarySelector::score_asset()` with 2-tier detection logic
- Added `test_fallback_detection_gitui()` test case for validation
- Scoring system: Exact match (OS+Arch=150) > Fallback (OS=100, Fallback Arch=25) > No match

## [0.5.2] - 2025-12-03

### Improved

- **Script installation UX** - Now displays "Command will be available as:" message during script installation
  - Consistent with package installation behavior
  - Shows the command name that will be used to invoke the script
  - Applied to both direct script installations and bucket script installations

### Changed

- **Script filtering in list --all** - Improved platform compatibility filtering
  - Added `is_os_compatible()` method for basic OS compatibility checking
  - Scripts now filtered by native OS support without executing interpreter checks
  - Significantly faster performance (no command execution during listing)
  - Consistent with package filtering behavior (platform-based, not runtime-based)
  - Windows shows PowerShell/Batch/Python scripts only
  - Unix-like systems show Bash/Python scripts only

### Technical

- Script filtering now uses compile-time platform checks instead of runtime interpreter checks
- More efficient `list --all` command with no external command execution

## [0.5.1] - 2025-12-03

### Fixed

- **Script display in list command** - `list --all` now correctly shows scripts from buckets
  - Added TYPE column to distinguish between binaries and scripts
  - Scripts filtered by platform compatibility (PowerShell, Bash, Python, Batch)
  - Fixed issue where scripts were being filtered out due to missing platform field

### Changed

- **List output format** - Added TYPE column showing "binary" for packages and script type for scripts
  - Binary packages shown in cyan
  - Script types shown in magenta (powershell, bash, python, batch)
- **Summary statistics** - Now shows "X package(s), Y script(s) available" format

## [0.5.0] - 2025-12-02

### Added

- **Bucket Script Support** - Install and manage scripts directly from buckets

  - Support for PowerShell (.ps1), Bash (.sh), Batch (.bat/.cmd), and Python (.py) scripts
  - Automatic script type detection and platform compatibility checking
  - Scripts displayed separately in search results with type badges

- **Script Installation** - Multiple installation methods

  - Install from local files: `wenget add ./script.ps1`
  - Install from URLs: `wenget add https://example.com/script.sh`
  - Install from buckets: `wenget add script-name`

- **Smart Command Naming** - Automatic executable name normalization
  - Removes platform suffixes (e.g., `ripgrep-x86_64` → `ripgrep`)
  - Removes architecture indicators (e.g., `tool_amd64` → `tool`)
  - Cleans up file extensions intelligently
  - Custom naming support: `--name custom-command`

### Enhanced

- **Search Command** - Now searches both packages and scripts

  - Separate sections for "Binary Packages" and "Scripts"
  - Shows script type and description for each result

- **Info Command** - Extended to support scripts

  - Displays script-specific metadata (type, URL, platform support)
  - Shows installation status for both packages and scripts

- **List Command** - Enhanced display format

  - Shows command name alongside package name
  - Improved column alignment and truncation
  - Better visual distinction between installed and available items

- **Add Command** - Unified installation interface
  - Detects input type automatically (package name, URL, or script)
  - Mixed installations supported: `wenget add package1 ./script.sh url`
  - Security warnings for script installations
  - Separate success/failure counts for packages and scripts

### Improved

- **Cache System** - Script awareness

  - Scripts cached alongside packages for fast searches
  - Script-specific cache invalidation
  - Platform compatibility filtering

- **Error Handling** - Better script installation feedback
  - Clear messages for unsupported script types
  - Platform compatibility warnings
  - Detailed installation failure reasons

### Technical

- **Architecture** - New script management infrastructure
  - `ScriptItem` type for bucket scripts
  - `ScriptType` enum with platform detection
  - Script shim/launcher creation system
  - Unified package source tracking (Bucket/DirectRepo/Script)

## [0.4.0] - 2025-12-01

### Added

- **Self-update capability** - `wenget update self` command to upgrade Wenget itself
  - Automatic version detection from GitHub releases
  - Platform-specific binary selection
  - Smart executable replacement for Windows and Unix systems
  - Automatic cleanup of old versions

### Improved

- **Windows**: Special handling for locked executables with background cleanup script
- **Unix/Linux/macOS**: Direct executable replacement with permission management
- **Error handling**: Comprehensive error messages and validation

### Documentation

- Updated README with self-update instructions
- Added usage examples for the new command

## [0.3.0] - 2025-11-25

### Changed

- **Remove `source` command** - Eliminated sources.json and all source management
- **Smart `add` command** - Auto-detects package names vs GitHub URLs
- **New `info` command** - Query package details (supports names and URLs)
- **Enhanced `list` command** - Now shows SOURCE column and descriptions
- **Package descriptions** - Stored in installed.json for faster access
- **Integrated resolver** - Name-based operations work for URL-installed packages
- **Improved UX** - Better alignment and formatting in list output

### Breaking Changes

- `source` command removed entirely
- installed.json format changed (added description field)
- Old installed.json files need migration (reinstall packages)

## [0.2.0] - 2025-01-21

### Added

- Installation scripts for Windows and Unix
- Improved init bucket checking

### Fixed

- Self-deletion when executable is inside .wenget
- Shim absolute path issues

## [0.1.0] - 2025-01-21

### Added

- Initial release
- Basic package management
- Bucket system
- Cross-platform support (Windows, macOS, Linux)
- Platform detection and binary selection
- GitHub integration
- Package cache system

[2.0.0]: https://github.com/superyngo/wenget/compare/v1.3.3...v2.0.0
[1.3.3]: https://github.com/superyngo/wenget/compare/v1.3.2...v1.3.3
[1.3.2]: https://github.com/superyngo/wenget/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/superyngo/wenget/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/superyngo/wenget/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/superyngo/wenget/compare/v1.1.1...v1.2.0
[1.1.1]: https://github.com/superyngo/wenget/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/superyngo/wenget/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/superyngo/wenget/compare/v0.9.1...v1.0.0
[0.9.1]: https://github.com/superyngo/wenget/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/superyngo/wenget/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/superyngo/wenget/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/superyngo/wenget/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/superyngo/wenget/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/superyngo/wenget/compare/v0.6.3...v0.7.0
[0.6.3]: https://github.com/superyngo/wenget/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/superyngo/wenget/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/superyngo/wenget/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/superyngo/wenget/compare/v0.5.3...v0.6.0
[0.5.3]: https://github.com/superyngo/wenget/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/superyngo/wenget/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/superyngo/wenget/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/superyngo/wenget/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/superyngo/wenget/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/superyngo/wenget/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/superyngo/wenget/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/superyngo/wenget/releases/tag/v0.1.0
