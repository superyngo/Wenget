# WenPM Package Sources

This directory contains curated lists of popular CLI tools from GitHub that can be installed via WenPM.

## Available Source Lists

### 📦 `sources-essential.txt`
Essential CLI tools that every developer should have. Includes:
- **Search & Find**: ripgrep, fd
- **File Viewing**: bat
- **Navigation**: zoxide, eza
- **Monitoring**: bottom, dust
- **Development**: tokei, hyperfine, delta
- **Git**: gitui
- **Shell**: starship
- **WenPM Official**: WenPM, cate, wedi

### 📚 `wenpm-sources.txt`
Comprehensive list of popular CLI tools including:
- All tools from essential list
- Additional utilities: nushell, xh, ouch, hexyl
- More git tools: lazygit
- Text processing: sd, xsv, jq
- And more...

## Usage

### Import all sources at once:

WenPM supports two import formats:
- **txt format** - URL list (requires GitHub API calls)
- **json format** - Complete package info (no API calls, faster!)

```bash
# Import essential tools (recommended for beginners)
# txt format - will fetch package info from GitHub
wenpm source import sources-essential.txt

# json format - instant import with complete package info
wenpm source import sources-essential.json

# Import comprehensive list
wenpm source import wenpm-sources.txt

# Import from URL (if hosted on GitHub)
wenpm source import https://raw.githubusercontent.com/superyngo/WenPM/main/sources-essential.json
```

**💡 Pro Tip:** Use JSON format to avoid GitHub API rate limits! JSON files contain complete package information and import instantly without fetching from GitHub.

### Export your sources:

```bash
# Export as txt (URLs only)
wenpm source export -o my-sources.txt

# Export as JSON (complete package info - recommended for sharing!)
wenpm source export -o my-sources.json -f json

# Export to stdout
wenpm source export
```

### After importing, install packages:

```bash
# List available packages
wenpm source list

# Get package information
wenpm source info ripgrep

# Install packages
wenpm add ripgrep fd bat
```

## Tool Descriptions

### 🔍 Search & Find
- **ripgrep** (`rg`) - Extremely fast grep alternative, recursively searches directories
- **fd** - Simple, fast alternative to `find` with intuitive syntax

### 📄 File Viewers
- **bat** - `cat` clone with syntax highlighting and Git integration
- **cate** - Lightweight file viewer with encoding support

### 📁 Navigation
- **zoxide** - Smarter `cd` command that learns your habits
- **eza** - Modern replacement for `ls` with colors and icons

### 📊 System Monitoring
- **bottom** (`btm`) - Cross-platform graphical process/system monitor
- **dust** - More intuitive version of `du` (disk usage)
- **procs** - Modern replacement for `ps` (process viewer)

### 💻 Development Tools
- **tokei** - Count lines of code, quickly
- **hyperfine** - Command-line benchmarking tool
- **delta** - Syntax-highlighting pager for git and diff output

### 🔧 Git Tools
- **gitui** - Blazing fast terminal UI for git
- **lazygit** - Simple terminal UI for git commands
- **wedi** - Git worktree management tool

### 🎨 Shell Enhancement
- **starship** - Minimal, fast, and customizable prompt for any shell
- **nushell** - A new type of shell

### 📝 Text Processing
- **sd** - Intuitive find & replace CLI (`sed` alternative)
- **xsv** - Fast CSV command line toolkit
- **jq** - Lightweight and flexible command-line JSON processor

### 🌐 Network Tools
- **xh** - Friendly and fast tool for sending HTTP requests

### 📦 Compression
- **ouch** - Painless compression and decompression

### 🔢 Other Utilities
- **hexyl** - Command-line hex viewer
- **grex** - Generate regular expressions from test cases
- **watchexec** - Execute commands in response to file modifications

## Contributing

To add a new tool to the sources list:

1. Ensure the tool:
   - Has binary releases on GitHub
   - Provides pre-built binaries for major platforms (Windows, Linux, macOS)
   - Is actively maintained
   - Is a useful CLI tool

2. Add the GitHub repository URL to the appropriate source list

3. Update this README with a brief description

## Notes

- All tools listed here are open source and hosted on GitHub
- Binary availability may vary by platform
- Use `wenpm source info <package>` to check supported platforms
- Some tools may require additional setup or configuration

## Package Manager Comparison

WenPM focuses on installing pre-built binaries from GitHub releases, making it:
- ✅ Fast (no compilation required)
- ✅ Cross-platform (Windows, Linux, macOS)
- ✅ Simple (just download and use)
- ✅ No dependencies on other package managers

## Support

For issues or questions:
- WenPM: https://github.com/superyngo/WenPM/issues
- Each tool has its own repository with documentation and issue tracker
