# WenPM - Wen Package Manager

A cross-platform package manager for GitHub binaries, written in Rust.

## Features

- **No Version Management**: Always installs the latest version
- **Local Metadata**: Complete local manifest management
- **Auto-parse GitHub Releases**: Automatically generates local JSON manifests
- **Multi-source Support**: GitHub / GitLab / self-hosted (planned)
- **Multi-threaded**: Fast downloads and analysis

## Installation

### Quick Install (Coming Soon)

**Unix/Linux/macOS**:
```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/WenPM/main/scripts/install.sh | bash
```

**Windows (PowerShell)**:
```powershell
iwr https://raw.githubusercontent.com/superyngo/WenPM/main/scripts/install.ps1 | iex
```

### Manual Build

```bash
git clone https://github.com/superyngo/WenPM.git
cd WenPM
cargo build --release
```

The binary will be at `target/release/wenpm` (or `wenpm.exe` on Windows).

## Quick Start

```bash
# Initialize WenPM
wenpm init

# Add a package from GitHub (Coming in Phase 2)
wenpm add https://github.com/user/repo

# List available packages
wenpm list

# Install a package
wenpm install package-name

# Upgrade all packages
wenpm upgrade all

# Set up PATH
wenpm setup-path
```

## Commands

### Core Commands

- `wenpm init` - Initialize WenPM (create directories and manifests)
- `wenpm add <url>...` - Add packages from GitHub URLs
- `wenpm list` - List available packages for current platform
- `wenpm search <name>...` - Search for packages
- `wenpm info <name>...` - Show package information
- `wenpm update` - Update package metadata from sources
- `wenpm install <name>...` - Install packages
- `wenpm upgrade [name...]` - Upgrade installed packages
- `wenpm delete <name>...` - Delete installed packages
- `wenpm setup-path` - Set up PATH environment variable

### Global Options

- `-v, --verbose` - Enable verbose logging

## Directory Structure

```
~/.wenpm/
├── sources.json           # Package metadata
├── installed.json         # Installed packages info
├── bin/                   # Symlinks/shims (add to PATH)
├── apps/                  # Installed applications
│   └── <app-name>/
│       └── bin/
└── cache/                 # Download cache
    └── downloads/
```

## Development Status

### Phase 1 (Current) ✅
- [x] Project structure
- [x] Core modules (config, manifest, paths, platform)
- [x] CLI framework
- [x] `init` command

### Phase 2 (In Progress)
- [ ] GitHub Provider
- [ ] `add` command
- [ ] `list/search/info` commands

### Phase 3+
- [ ] Download & installation
- [ ] Update & upgrade
- [ ] Self-upgrade
- [ ] Complete all commands

See [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) for the full roadmap.

## Platform Support

| Platform | Status |
|----------|--------|
| Windows (x86_64) | ✅ Planned |
| Windows (ARM64) | ✅ Planned |
| Linux (x86_64) | ✅ Planned |
| Linux (ARM64) | ✅ Planned |
| macOS (Intel) | ✅ Planned |
| macOS (Apple Silicon) | ✅ Planned |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](./LICENSE) for details.

## Credits

Inspired by:
- [Obtainium](https://github.com/ImranR98/Obtainium) (Android)
- [Scoop](https://scoop.sh/) (Windows)

## Links

- **GitHub**: https://github.com/superyngo/WenPM
- **Issues**: https://github.com/superyngo/WenPM/issues
- **Discussions**: https://github.com/superyngo/WenPM/discussions
