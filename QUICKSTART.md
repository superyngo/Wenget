# WenPM Quick Start Guide

Get started with WenPM in just a few minutes!

## 🚀 Installation

### Option 1: From Release (Recommended)
```bash
# Download the latest release from GitHub
# Extract and run wenpm init
```

### Option 2: Build from Source
```bash
git clone https://github.com/superyngo/WenPM.git
cd WenPM
cargo build --release
./target/release/wenpm init
```

## 📦 First Steps

### 1. Initialize WenPM
This creates the necessary directories and sets up PATH:
```bash
wenpm init
```

After initialization, **restart your terminal** for PATH changes to take effect.

### 2. Import Package Sources
Import our curated list of essential CLI tools:
```bash
# Recommended: JSON format (faster, no API calls)
wenpm source import sources-essential.json

# Alternative: txt format (will fetch from GitHub API)
wenpm source import sources-essential.txt
```

**💡 Tip:** JSON format is recommended as it contains complete package info and doesn't require GitHub API calls!

### 3. View Available Packages
See what packages are available for your platform:
```bash
wenpm source list
```

### 4. Install Your First Package
Let's install `ripgrep`, a blazing fast search tool:
```bash
wenpm add ripgrep
```

### 5. Use Your Installed Tool
```bash
rg "search text" .
```

## 🔥 Recommended Starter Pack

Install these essential tools to supercharge your command line:

```bash
# Search tools
wenpm add ripgrep fd

# File viewer
wenpm add bat

# Navigation
wenpm add zoxide eza

# System monitoring
wenpm add bottom

# Development
wenpm add hyperfine starship
```

## 📖 Common Commands

### Source Management
```bash
# List available packages from sources
wenpm source list

# Show package information
wenpm source info ripgrep

# Update package metadata
wenpm source update

# Add a specific package source
wenpm source add https://github.com/user/repo

# Export your sources list
wenpm source export -o my-sources.txt        # txt format (URLs)
wenpm source export -o my-sources.json -f json  # JSON format (full info)
```

### Package Management
```bash
# Install packages
wenpm add ripgrep fd bat

# List installed packages
wenpm list

# Search for packages (in sources)
wenpm search grep

# Update installed packages
wenpm update ripgrep
wenpm update all

# Remove packages
wenpm del ripgrep
```

## 💡 Pro Tips

### 1. Use Wildcards
```bash
# Install multiple packages matching a pattern
wenpm add *grep

# Update all packages from a specific author
wenpm update sharkdp/*
```

### 2. Auto-confirm Installations
Skip confirmation prompts with `-y`:
```bash
wenpm add ripgrep fd bat -y
```

### 3. Check Before Installing
Always check package info before installing:
```bash
wenpm source info ripgrep
```

### 4. Keep Sources Updated
Regularly update your package sources to get the latest versions:
```bash
wenpm source update
```

## 🌟 Tool Showcase

### ripgrep (rg)
**Ultra-fast text search**
```bash
# Search for "TODO" in all files
rg TODO

# Search in specific file types
rg "error" --type rust

# Case-insensitive search
rg -i "hello"
```

### fd
**Modern find alternative**
```bash
# Find files by name
fd config

# Find by extension
fd -e rs

# Execute commands on results
fd -e jpg -x convert {} {.}.png
```

### bat
**Cat with syntax highlighting**
```bash
# View a file with syntax highlighting
bat README.md

# Show line numbers
bat -n file.rs

# View with paging
bat --paging=always large-file.log
```

### zoxide
**Smarter cd command**
```bash
# After using cd a few times, jump directly
z documents
z proj
z down
```

### eza
**Modern ls replacement**
```bash
# List files with details
eza -la

# Tree view
eza --tree

# Git status integration
eza --git
```

### bottom (btm)
**System monitor**
```bash
# Launch the TUI
btm

# Basic mode
btm --basic
```

### starship
**Cross-shell prompt**
```bash
# Already configured after installation!
# Your prompt will show git status, language versions, etc.
```

## 🔧 Troubleshooting

### PATH not updated?
1. Make sure you ran `wenpm init`
2. Restart your terminal
3. Check PATH manually:
   - Windows: `echo %PATH%`
   - Linux/macOS: `echo $PATH`

### Command not found after installation?
1. Verify installation: `wenpm list`
2. Check if binary exists:
   - Windows: `dir %USERPROFILE%\.wenpm\bin`
   - Linux/macOS: `ls ~/.wenpm/bin`
3. Restart terminal

### Package fails to install?
1. Check platform support: `wenpm source info <package>`
2. Update sources: `wenpm source update`
3. Check GitHub releases manually

## 📚 Learn More

- [Full Command Reference](SOURCES.md)
- [Package Sources](SOURCES.md)
- [Report Issues](https://github.com/superyngo/WenPM/issues)

## 🎉 What's Next?

1. Explore more packages: `wenpm source list`
2. Install tools that match your workflow
3. Share your sources list: `wenpm source export`
4. Contribute packages to the community!

---

**Happy package managing! 🚀**
