# Release Notes - v0.5.0

## 🎉 Major Features

### 📜 Bucket Script Support

Wenget now supports installing and managing scripts directly from bucket manifests! You can now distribute PowerShell, Bash, Batch, and Python scripts alongside binary packages.

**Key Features:**
- ✅ Install scripts from buckets with `wenget add <script-name>`
- ✅ Search scripts with `wenget search`
- ✅ View script information with `wenget info`
- ✅ Automatic platform compatibility checking
- ✅ Unified management with binary packages

**Example:**
```bash
# Search for scripts
wenget search my-script

# View script details
wenget info my-script

# Install script from bucket
wenget add my-script
```

**Bucket Manifest Example:**
```json
{
  "packages": [...],
  "scripts": [
    {
      "name": "my-script",
      "description": "Useful PowerShell script",
      "url": "https://raw.githubusercontent.com/user/repo/main/script.ps1",
      "script_type": "powershell",
      "repo": "https://github.com/user/repo"
    }
  ]
}
```

### 🔧 Smart Command Name Normalization

Wenget now automatically removes platform-specific suffixes from executable names for cleaner command names.

**Examples:**
- `cate-windows-x86_64.exe` → `cate`
- `bat-v0.24.0-x86_64.exe` → `bat`
- `tool-linux-aarch64` → `tool`
- `git-lfs.exe` → `git-lfs` (preserves multi-word names)

**Features:**
- ✅ Automatically detects and removes platform suffixes
- ✅ Preserves tool names with hyphens (like `git-lfs`)
- ✅ Can be overridden with `--name` parameter
- ✅ Recorded in `installed.json` for reference

---

## 🚀 Enhancements

### Cache System
- **Extended cache structure** to support scripts alongside packages
- **Improved cache building** to process both packages and scripts from buckets
- **Backward compatible** with existing cache files

### Search Command
- Now searches both packages and scripts
- Separate sections for binary packages and scripts
- Shows script type (PowerShell, Bash, etc.)

### Info Command
- Displays detailed script information
- Shows platform compatibility for scripts
- Unified interface for packages and scripts

### List Command
- New `COMMAND` column showing the actual command name
- Better alignment and formatting
- Command names highlighted for visibility

---

## 📊 Technical Improvements

### New Data Structures
- `ScriptItem` - Script metadata in bucket manifests
- `CachedScript` - Script information in cache
- Extended `SourceManifest` with `scripts` field
- Extended `ManifestCache` with `scripts` HashMap

### Code Quality
- Improved command name normalization algorithm
- Better separation of package and script handling
- Enhanced error messages
- More comprehensive testing

---

## 📝 Breaking Changes

**None** - This release is fully backward compatible with v0.4.0:
- ✅ Existing buckets without scripts continue to work
- ✅ Old cache files are automatically upgraded
- ✅ All existing commands and features remain unchanged
- ✅ `installed.json` already supported script tracking

---

## 📚 Documentation Updates

- Updated README with script support examples
- Added bucket manifest structure for scripts
- Updated field descriptions for packages and scripts
- Added examples for creating script-enabled buckets

---

## 🔄 Migration from v0.4.0

No migration needed! Simply update Wenget:

```bash
wenget update self
```

Or download the latest release and your existing configuration, buckets, and installed packages will work seamlessly.

---

## 🙏 Acknowledgments

This release brings Wenget closer to being a complete package and script management solution. Thank you for using Wenget!

---

## 📦 Installation

### Quick Install

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/superyngo/Wenget/main/install.ps1 | iex
```

**Linux/macOS (Bash):**
```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/Wenget/main/install.sh | bash
```

### Manual Download

Download the latest release from [GitHub Releases](https://github.com/superyngo/Wenget/releases/tag/v0.5.0)

---

## 🐛 Bug Reports & Feature Requests

Please report issues or suggest features at: https://github.com/superyngo/Wenget/issues
