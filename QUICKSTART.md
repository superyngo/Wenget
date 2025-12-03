# Wenget Quick Start Guide

Get started with Wenget in just a few minutes!

## 🚀 Installation

### Option 1: From Release (Recommended)
```bash
# Download the latest release from GitHub
# Extract and run wenget init
```

### Option 2: Build from Source
```bash
git clone https://github.com/superyngo/Wenget.git
cd Wenget
cargo build --release
./target/release/wenget init
```

## 📦 First Steps

### 1. Initialize Wenget
This creates the necessary directories and sets up PATH:
```bash
wenget init
```

After initialization, **restart your terminal** for PATH changes to take effect.

### 2. Add the Official Bucket
Add the official Wenget bucket with curated packages and scripts:
```bash
wenget bucket add wenget https://raw.githubusercontent.com/superyngo/wenget-bucket/main/manifest.json
```

**💡 Tip:** The init command automatically adds the official bucket, so you can skip this step if you've just initialized!

### 3. View Available Packages and Scripts
See what packages and scripts are available for your platform:
```bash
wenget list --all
```

### 4. Install Your First Package
Let's install `ripgrep`, a blazing fast search tool:
```bash
wenget add ripgrep
```

### 5. Use Your Installed Tool
```bash
rg "search text" .
```

## 🔥 Recommended Starter Pack

Install these essential tools to supercharge your command line:

```bash
# Search tools
wenget add ripgrep fd

# File viewer
wenget add bat

# Navigation
wenget add zoxide eza

# System monitoring
wenget add bottom

# Development
wenget add hyperfine starship
```

## 📖 Common Commands

### Bucket Management
```bash
# Add a bucket (curated package and script collections)
wenget bucket add mybucket https://url/to/manifest.json

# List all buckets
wenget bucket list

# Remove a bucket
wenget bucket del mybucket

# Refresh cache from buckets
wenget bucket refresh
```

### Package and Script Management
```bash
# Install packages and scripts
wenget add ripgrep fd bat
wenget add mini-nano  # Install a script from bucket

# List installed packages
wenget list

# List all available packages and scripts
wenget list --all

# Search for packages and scripts
wenget search grep

# Show detailed information
wenget info ripgrep

# Update installed packages
wenget update ripgrep
wenget update all

# Update Wenget itself
wenget update self

# Remove packages
wenget del ripgrep
```

### Script Installation
```bash
# Install from local file
wenget add ./my-script.ps1

# Install from URL
wenget add https://raw.githubusercontent.com/user/repo/main/script.sh

# Install from bucket
wenget add script-name
```

## 💡 Pro Tips

### 1. Use Wildcards for Search
```bash
# Search for packages matching a pattern
wenget search *grep
```

### 2. Auto-confirm Installations
Skip confirmation prompts with `-y`:
```bash
wenget add ripgrep fd bat -y
```

### 3. Check Before Installing
Always check package or script info before installing:
```bash
wenget info ripgrep
wenget info mini-nano
```

### 4. Mix Packages and Scripts
Install packages and scripts together:
```bash
wenget add ripgrep ./my-script.ps1 bat
```

### 5. Keep Your Tools Updated
Regularly update your installed packages:
```bash
wenget update all
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
1. Make sure you ran `wenget init`
2. Restart your terminal
3. Check PATH manually:
   - Windows: `echo %PATH%`
   - Linux/macOS: `echo $PATH`

### Command not found after installation?
1. Verify installation: `wenget list`
2. Check if binary exists:
   - Windows: `dir %USERPROFILE%\.wenget\bin`
   - Linux/macOS: `ls ~/.wenget/bin`
3. Restart terminal

### Package fails to install?
1. Check platform support: `wenget info <package>`
2. Refresh cache: `wenget bucket refresh`
3. Check GitHub releases manually

### Script not working?
1. Verify script type is supported on your platform
2. Check if required interpreter is installed (PowerShell, Python, Bash)
3. Review security warnings before running scripts

## 📚 Learn More

- [README](README.md) - Full documentation
- [CHANGELOG](CHANGELOG.md) - Version history
- [Report Issues](https://github.com/superyngo/Wenget/issues)

## 🎉 What's Next?

1. Explore more packages: `wenget list --all`
2. Install tools that match your workflow
3. Try installing scripts from buckets
4. Create your own bucket for custom packages!

---

**Happy package managing! 🚀**
