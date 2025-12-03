# Changelog

All notable changes to Wenget will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.5.1]: https://github.com/superyngo/wenget/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/superyngo/wenget/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/superyngo/wenget/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/superyngo/wenget/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/superyngo/wenget/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/superyngo/wenget/releases/tag/v0.1.0
