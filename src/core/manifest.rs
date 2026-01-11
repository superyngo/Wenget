//! Manifest data structures for Wenget
//!
//! This module defines the core data structures for package metadata:
//! - `Package`: Individual package information
//! - `PlatformBinary`: Platform-specific binary information
//! - `SourceManifest`: The sources.json structure
//! - `InstalledManifest`: The installed.json structure

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Cached interpreter availability results
static INTERPRETER_CACHE: OnceLock<InterpreterCache> = OnceLock::new();

/// Cache for interpreter availability checks
struct InterpreterCache {
    pwsh_available: bool,
    bash_available: bool,
    python_available: bool,
}

impl InterpreterCache {
    fn detect() -> Self {
        Self {
            pwsh_available: std::process::Command::new("pwsh")
                .arg("--version")
                .output()
                .is_ok(),
            bash_available: std::process::Command::new("bash")
                .arg("--version")
                .output()
                .is_ok(),
            python_available: std::process::Command::new("python")
                .arg("--version")
                .output()
                .is_ok()
                || std::process::Command::new("python3")
                    .arg("--version")
                    .output()
                    .is_ok(),
        }
    }
}

fn get_interpreter_cache() -> &'static InterpreterCache {
    INTERPRETER_CACHE.get_or_init(InterpreterCache::detect)
}

/// Script type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ScriptType {
    /// PowerShell script (.ps1)
    PowerShell,
    /// Windows Batch script (.bat, .cmd)
    Batch,
    /// Bash/Shell script (.sh)
    Bash,
    /// Python script (.py)
    Python,
}

impl ScriptType {
    /// Get the file extension for this script type
    pub fn extension(&self) -> &str {
        match self {
            ScriptType::PowerShell => "ps1",
            ScriptType::Batch => "cmd",
            ScriptType::Bash => "sh",
            ScriptType::Python => "py",
        }
    }

    /// Get the display name for this script type
    pub fn display_name(&self) -> &str {
        match self {
            ScriptType::PowerShell => "PowerShell",
            ScriptType::Batch => "Batch",
            ScriptType::Bash => "Bash",
            ScriptType::Python => "Python",
        }
    }

    /// Check if this script type is supported on the current platform.
    ///
    /// This checks if the required interpreter is actually available on the system.
    /// Results are cached for performance.
    pub fn is_supported_on_current_platform(&self) -> bool {
        let cache = get_interpreter_cache();

        match self {
            ScriptType::PowerShell => {
                // PowerShell is available on Windows natively, and on Linux/macOS via pwsh
                if cfg!(target_os = "windows") {
                    true
                } else {
                    cache.pwsh_available
                }
            }
            ScriptType::Batch => {
                // Batch scripts only work on Windows
                cfg!(target_os = "windows")
            }
            ScriptType::Bash => {
                // Bash is available on Linux and macOS, and on Windows via WSL/Git Bash
                if cfg!(target_os = "windows") {
                    cache.bash_available
                } else {
                    true
                }
            }
            ScriptType::Python => cache.python_available,
        }
    }

    /// Check basic OS compatibility without executing commands (for listing)
    /// This is faster than is_supported_on_current_platform and doesn't require
    /// the interpreter to be installed
    pub fn is_os_compatible(&self) -> bool {
        match self {
            ScriptType::PowerShell => {
                // PowerShell scripts work on Windows natively
                // On Unix, they require pwsh but we don't check here
                cfg!(target_os = "windows")
            }
            ScriptType::Batch => {
                // Batch scripts only work on Windows
                cfg!(target_os = "windows")
            }
            ScriptType::Bash => {
                // Bash scripts work natively on Unix-like systems
                // On Windows they require WSL/Git Bash but we don't check here
                !cfg!(target_os = "windows")
            }
            ScriptType::Python => {
                // Python scripts can work on any platform if Python is installed
                // We don't check for Python installation here
                true
            }
        }
    }

    /// Get the platform-specific script type preference order.
    ///
    /// - Windows: PowerShell > Batch > Python > Bash
    /// - Unix: Bash > Python > PowerShell
    pub fn preference_order() -> &'static [ScriptType] {
        #[cfg(target_os = "windows")]
        {
            &[
                ScriptType::PowerShell,
                ScriptType::Batch,
                ScriptType::Python,
                ScriptType::Bash,
            ]
        }

        #[cfg(not(target_os = "windows"))]
        {
            &[ScriptType::Bash, ScriptType::Python, ScriptType::PowerShell]
        }
    }
}

/// Platform-specific binary information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformBinary {
    /// Download URL for the binary
    pub url: String,

    /// File size in bytes
    pub size: u64,

    /// Optional SHA256 checksum (for future use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Platform-specific script information (for multi-platform scripts)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptPlatform {
    /// Download URL for this platform's script
    pub url: String,

    /// Optional SHA256 checksum
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package name (used as identifier)
    pub name: String,

    /// Short description
    pub description: String,

    /// Repository URL (e.g., https://github.com/user/repo)
    pub repo: String,

    /// Homepage URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// License (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Platform-specific binaries
    /// Key format: "{os}-{arch}" or "{os}-{arch}-{variant}"
    /// Examples: "windows-x86_64", "linux-x86_64-musl", "macos-aarch64"
    pub platforms: HashMap<String, PlatformBinary>,
}

/// Script item metadata (for bucket scripts)
///
/// Supports multi-platform scripts where the same script name
/// can have different implementations for different platforms.
///
/// # Example JSON format:
/// ```json
/// {
///   "name": "rclonemm",
///   "description": "Manage rclone mount through ssh config.",
///   "repo": "https://gist.github.com/superyngo/...",
///   "platforms": {
///     "bash": { "url": "https://.../rclonemm.sh" },
///     "powershell": { "url": "https://.../rclonemm.ps1" }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptItem {
    /// Script name (used as identifier)
    pub name: String,

    /// Short description
    pub description: String,

    /// Repository URL (for reference, e.g., Gist URL)
    pub repo: String,

    /// Platform-specific scripts (key: script type like "bash", "powershell")
    pub platforms: HashMap<ScriptType, ScriptPlatform>,

    /// Homepage URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// License (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl ScriptItem {
    /// Get the best compatible script for the current platform (for display/listing)
    ///
    /// Priority order:
    /// - Windows: PowerShell > Batch > Python > Bash
    /// - Unix: Bash > Python > PowerShell
    ///
    /// Note: This uses `is_os_compatible()` for basic OS-level filtering,
    /// which doesn't check if the actual interpreter is installed.
    /// For installation, use `get_installable_script()` instead.
    ///
    /// Returns the script type and its platform info if a compatible one is found.
    pub fn get_compatible_script(&self) -> Option<(ScriptType, &ScriptPlatform)> {
        for script_type in ScriptType::preference_order() {
            if script_type.is_os_compatible() {
                if let Some(platform) = self.platforms.get(script_type) {
                    return Some((script_type.clone(), platform));
                }
            }
        }
        None
    }

    /// Get the best installable script for the current platform (checks if interpreter exists)
    ///
    /// This is more thorough than `get_compatible_script()` as it actually checks
    /// if the required interpreter is installed on the system.
    ///
    /// Returns the script type and its platform info if an installable one is found.
    pub fn get_installable_script(&self) -> Option<(ScriptType, &ScriptPlatform)> {
        for script_type in ScriptType::preference_order() {
            if script_type.is_supported_on_current_platform() {
                if let Some(platform) = self.platforms.get(script_type) {
                    return Some((script_type.clone(), platform));
                }
            }
        }
        None
    }

    /// Get all available platforms for this script
    #[allow(dead_code)]
    pub fn available_platforms(&self) -> Vec<ScriptType> {
        self.platforms.keys().cloned().collect()
    }

    /// Check if this script has a compatible version for the current platform
    pub fn is_compatible_with_current_platform(&self) -> bool {
        self.get_compatible_script().is_some()
    }

    /// Get a specific platform's script info
    #[allow(dead_code)]
    pub fn get_platform(&self, script_type: &ScriptType) -> Option<&ScriptPlatform> {
        self.platforms.get(script_type)
    }

    /// Get a display string showing available platforms
    pub fn platforms_display(&self) -> String {
        let platforms: Vec<&str> = self.platforms.keys().map(|st| st.display_name()).collect();
        platforms.join(", ")
    }
}

/// Source manifest (sources.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceManifest {
    /// List of available packages
    pub packages: Vec<Package>,

    /// List of available scripts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scripts: Vec<ScriptItem>,
}

impl SourceManifest {
    /// Create a new empty source manifest
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            scripts: Vec::new(),
        }
    }

    /// Get packages that support a specific platform
    #[allow(dead_code)]
    pub fn packages_for_platform(&self, platform: &str) -> Vec<&Package> {
        self.packages
            .iter()
            .filter(|p| p.platforms.contains_key(platform))
            .collect()
    }

    /// Get scripts that are supported on the current platform
    #[allow(dead_code)]
    pub fn scripts_for_current_platform(&self) -> Vec<&ScriptItem> {
        self.scripts
            .iter()
            .filter(|s| s.is_compatible_with_current_platform())
            .collect()
    }
}

impl Default for SourceManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Package source tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PackageSource {
    /// Package installed from a bucket
    Bucket { name: String },
    /// Package installed directly from a GitHub repository URL
    DirectRepo { url: String },
    /// Script installed from local path or URL
    Script {
        /// Original source (local path or URL)
        origin: String,
        /// Script type
        script_type: ScriptType,
    },
}

/// Installed package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// Installed version
    pub version: String,

    /// Platform identifier
    pub platform: String,

    /// Installation timestamp
    pub installed_at: DateTime<Utc>,

    /// Installation path
    pub install_path: String,

    /// List of installed files (relative to install_path)
    pub files: Vec<String>,

    /// Package source (where it was installed from)
    pub source: PackageSource,

    /// Package description
    pub description: String,

    /// Command names (the names used to invoke the tools)
    #[serde(default)]
    pub command_names: Vec<String>,

    /// Legacy single command name (for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_name: Option<String>,
}

/// Installed manifest (installed.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledManifest {
    /// Map of package name to installed package info
    pub packages: HashMap<String, InstalledPackage>,
}

impl InstalledManifest {
    /// Create a new empty installed manifest
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Check if a package is installed
    pub fn is_installed(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    /// Get installed package info
    pub fn get_package(&self, name: &str) -> Option<&InstalledPackage> {
        self.packages.get(name)
    }

    /// Add or update an installed package
    pub fn upsert_package(&mut self, name: String, package: InstalledPackage) {
        self.packages.insert(name, package);
    }

    /// Remove an installed package
    pub fn remove_package(&mut self, name: &str) -> Option<InstalledPackage> {
        self.packages.remove(name)
    }

    /// Get all installed package names
    #[allow(dead_code)]
    pub fn installed_names(&self) -> Vec<&str> {
        self.packages.keys().map(|s| s.as_str()).collect()
    }

    /// Migrate old format (single command_name) to new format (command_names vec)
    pub fn migrate(&mut self) {
        for package in self.packages.values_mut() {
            // If command_names is empty but command_name exists, migrate
            if package.command_names.is_empty() {
                if let Some(ref name) = package.command_name {
                    package.command_names = vec![name.clone()];
                }
            }
        }
    }
}

impl Default for InstalledManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_manifest_new() {
        let manifest = SourceManifest::new();
        assert_eq!(manifest.packages.len(), 0);
    }

    #[test]
    fn test_installed_manifest() {
        let mut manifest = InstalledManifest::new();

        let package = InstalledPackage {
            version: "1.0.0".to_string(),
            platform: "windows-x86_64".to_string(),
            installed_at: Utc::now(),
            install_path: "C:\\Users\\test\\.wenget\\apps\\test".to_string(),
            files: vec!["bin/test.exe".to_string()],
            source: PackageSource::Bucket {
                name: "test-bucket".to_string(),
            },
            description: "Test package".to_string(),
            command_names: vec!["test".to_string()],
            command_name: None,
        };

        manifest.upsert_package("test".to_string(), package);
        assert!(manifest.is_installed("test"));
        assert_eq!(manifest.get_package("test").unwrap().version, "1.0.0");

        manifest.remove_package("test");
        assert!(!manifest.is_installed("test"));
    }
}
