# WenPM - Wen Package Manager

A cross-platform package manager for GitHub binaries, written in Rust.

WenPM simplifies the installation and management of command-line tools and applications distributed through GitHub Releases. It automatically detects your platform, downloads the appropriate binaries, and manages them in an organized directory structure.

## Features

- **🚀 One-line Installation**: Remote installation scripts for quick setup
- **🔄 Auto-update**: Always installs the latest version from GitHub Releases
- **📦 Bucket System**: Organize packages using bucket manifests
- **🌐 Cross-platform**: Windows, macOS, Linux (multiple architectures)
- **📁 Organized Storage**: All packages in `~/.wenpm/` with proper structure
- **🔍 Smart Search**: Search packages across all configured buckets
- **⚡ Fast Downloads**: Multi-threaded downloads with caching
- **🎯 Platform Detection**: Automatically selects the correct binary for your system

## Quick Install

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/superyngo/WenPM/main/install.ps1 | iex
```

### Linux/macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/WenPM/main/install.sh | bash
```

### Manual Installation

Download the latest release from [GitHub Releases](https://github.com/superyngo/WenPM/releases) and place it in your PATH, or build from source:

```bash
git clone https://github.com/superyngo/WenPM.git
cd WenPM
cargo build --release
```

The binary will be at `target/release/wenpm` (or `wenpm.exe` on Windows).

## Quick Start

```bash
# Initialize WenPM (done automatically with install scripts)
wenpm init

# Add the official WenPM bucket (if not added during init)
wenpm bucket add wenpm https://raw.githubusercontent.com/superyngo/wenpm-bucket/main/manifest.json

# Search for packages
wenpm search ripgrep

# Install a package
wenpm add ripgrep

# List installed packages
wenpm list

# Update package metadata
wenpm update

# Delete a package
wenpm delete ripgrep
```

## Commands

### Package Management

- `wenpm add <name>...` - Install packages
- `wenpm delete <name>...` - Uninstall packages
  - `wenpm del self` - Uninstall WenPM itself
- `wenpm list` - List installed packages
- `wenpm search <keyword>` - Search available packages
- `wenpm update` - Update package metadata from all buckets

### Bucket Management

- `wenpm bucket add <name> <url>` - Add a bucket
- `wenpm bucket remove <name>` - Remove a bucket
- `wenpm bucket list` - List all buckets
- `wenpm bucket refresh` - Rebuild package cache

### Source Management (Legacy)

- `wenpm source add <url>...` - Add package sources
- `wenpm source list` - List package sources
- `wenpm source export -o <file>` - Export sources
- `wenpm source import <file>` - Import sources

### System

- `wenpm init` - Initialize WenPM directories and configuration
- `wenpm --version` - Show version information
- `wenpm --help` - Show help message

### Global Options

- `--yes`, `-y` - Skip confirmation prompts
- `--verbose`, `-v` - Enable verbose logging

## Directory Structure

```
~/.wenpm/
├── apps/                  # Installed applications
│   ├── wenpm/            # WenPM itself
│   └── <package>/        # Each installed package
├── bin/                   # Symlinks/shims (added to PATH)
│   ├── wenpm.cmd         # WenPM shim (Windows)
│   ├── wenpm             # WenPM symlink (Unix)
│   └── <package>.cmd     # Package shims
├── cache/                 # Download and package cache
│   └── packages.json     # Cached package list
├── buckets.json          # Bucket configuration
├── sources.json          # Package sources (legacy)
└── installed.json        # Installed packages info
```

## Bucket System

Buckets are collections of package manifests hosted online. The official WenPM bucket provides curated open-source tools.

### Official Bucket

```bash
wenpm bucket add wenpm https://raw.githubusercontent.com/superyngo/wenpm-bucket/main/manifest.json
```

### Creating Your Own Bucket

Create a `manifest.json` with the following structure:

```json
{
  "name": "my-bucket",
  "description": "My custom bucket",
  "packages": [
    {
      "name": "my-tool",
      "repo": "username/repo",
      "description": "Tool description"
    }
  ]
}
```

Host it on GitHub or any web server, then add it:

```bash
wenpm bucket add my-bucket https://example.com/manifest.json
```

## Platform Support

WenPM supports the following platforms:

| Platform | Architecture | Status |
|----------|--------------|--------|
| Windows | x86_64 (64-bit) | ✅ Supported |
| Windows | i686 (32-bit) | ✅ Supported |
| Linux | x86_64 | ✅ Supported |
| Linux | i686 | ✅ Supported |
| Linux | aarch64 (ARM64) | ✅ Supported |
| Linux | armv7 | ✅ Supported |
| macOS | x86_64 (Intel) | ✅ Supported |
| macOS | aarch64 (Apple Silicon) | ✅ Supported |

## How It Works

1. **Platform Detection**: WenPM automatically detects your OS and architecture
2. **Package Resolution**: Searches buckets for the requested package
3. **Binary Selection**: Identifies the appropriate binary from GitHub Releases
4. **Download**: Downloads and caches the binary
5. **Installation**: Extracts and places the binary in `~/.wenpm/apps/<package>/`
6. **Shim Creation**: Creates a shim/symlink in `~/.wenpm/bin/` for easy access

## Examples

### Install Popular Tools

```bash
# Modern alternatives to classic Unix tools
wenpm add ripgrep fd bat

# Git TUI
wenpm add gitui lazygit

# System monitoring
wenpm add bottom

# Shell prompt
wenpm add starship

# Directory navigation
wenpm add zoxide
```

### Manage Packages

```bash
# Search for a tool
wenpm search rust

# Update metadata and install
wenpm update
wenpm add tokei

# List what's installed
wenpm list

# Remove a package
wenpm delete tokei
```

## Important Disclaimer

**⚠️ NO WARRANTIES OR GUARANTEES**

WenPM is a package manager that facilitates downloading and installing applications from GitHub Releases. **WenPM DOES NOT:**

- ❌ Verify the authenticity or safety of packages
- ❌ Maintain or update the applications themselves
- ❌ Provide usage information or support for installed applications
- ❌ Guarantee the security, stability, or functionality of any package
- ❌ Take responsibility for any damage caused by installed applications

**Users are responsible for:**
- ✅ Verifying the trustworthiness of package sources
- ✅ Understanding what each package does before installing
- ✅ Reviewing the source repositories and releases
- ✅ Accepting all risks associated with installing third-party software

**By using WenPM, you acknowledge that you install packages at your own risk.**

WenPM acts only as a convenience tool for downloading and organizing binaries. The responsibility for verifying, securing, and using applications rests entirely with the user.

## Uninstallation

### Using WenPM
```bash
wenpm del self
```

This will:
1. Remove WenPM from PATH
2. Delete all WenPM directories and installed packages
3. Remove the WenPM executable itself

### Manual Uninstallation

**Windows:**
```powershell
# Remove from PATH, then delete:
Remove-Item -Recurse -Force "$env:USERPROFILE\.wenpm"
```

**Linux/macOS:**
```bash
# Remove from PATH, then delete:
rm -rf ~/.wenpm
```

## Development

### Building from Source

```bash
git clone https://github.com/superyngo/WenPM.git
cd WenPM
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
wenpm/
├── src/
│   ├── bucket.rs         # Bucket management
│   ├── cache.rs          # Package cache
│   ├── cli.rs            # CLI interface
│   ├── commands/         # Command implementations
│   ├── core/             # Core functionality
│   ├── downloader/       # Download logic
│   ├── installer/        # Installation logic
│   ├── providers/        # GitHub API integration
│   └── utils/            # Utilities
├── install.ps1           # Windows installer
└── install.sh            # Unix installer
```

## Troubleshooting

### PATH Not Updated

After installation, you may need to restart your terminal or run:

**Windows:**
```powershell
refreshenv
```

**Linux/macOS:**
```bash
source ~/.bashrc  # or ~/.zshrc, ~/.profile
```

### Package Not Found

```bash
# Update package metadata
wenpm update

# Check available buckets
wenpm bucket list

# Rebuild cache
wenpm bucket refresh
```

### Permission Errors (Linux/macOS)

```bash
# Ensure ~/.wenpm/bin is in PATH and has correct permissions
chmod +x ~/.wenpm/bin/*
```

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - Copyright (c) 2025 wen

See [LICENSE](./LICENSE) for details.

## Credits

Inspired by:
- [Scoop](https://scoop.sh/) - Windows package manager
- [Homebrew](https://brew.sh/) - macOS package manager
- [Obtainium](https://github.com/ImranR98/Obtainium) - Android app manager

## Links

- **GitHub**: https://github.com/superyngo/WenPM
- **Releases**: https://github.com/superyngo/WenPM/releases
- **Issues**: https://github.com/superyngo/WenPM/issues
- **Official Bucket**: https://github.com/superyngo/wenpm-bucket

## Changelog

### v0.2.0 (2025-01-21)
- Add installation scripts for Windows and Unix
- Improve init bucket checking
- Fix self-deletion when executable is inside .wenpm
- Fix shim absolute path issues

### v0.1.0 (2025-01-21)
- Initial release
- Basic package management
- Bucket system
- Cross-platform support
