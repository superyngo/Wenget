//! Manifest data structures for WenPM
//!
//! This module defines the core data structures for package metadata:
//! - `Package`: Individual package information
//! - `PlatformBinary`: Platform-specific binary information
//! - `SourceManifest`: The sources.json structure
//! - `InstalledManifest`: The installed.json structure

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    /// Latest version
    pub latest: String,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Platform-specific binaries
    /// Key format: "{os}-{arch}" or "{os}-{arch}-{variant}"
    /// Examples: "windows-x86_64", "linux-x86_64-musl", "macos-aarch64"
    pub platforms: HashMap<String, PlatformBinary>,
}

/// Source manifest (sources.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceManifest {
    /// List of available packages
    pub packages: Vec<Package>,

    /// Last updated timestamp for the entire manifest
    pub last_updated: DateTime<Utc>,
}

impl SourceManifest {
    /// Create a new empty source manifest
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    /// Find a package by name
    pub fn find_package(&self, name: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Find a package by name (mutable)
    pub fn find_package_mut(&mut self, name: &str) -> Option<&mut Package> {
        self.packages.iter_mut().find(|p| p.name == name)
    }

    /// Add or update a package
    pub fn upsert_package(&mut self, package: Package) {
        if let Some(existing) = self.find_package_mut(&package.name) {
            *existing = package;
        } else {
            self.packages.push(package);
        }
        self.last_updated = Utc::now();
    }

    /// Remove a package by name
    pub fn remove_package(&mut self, name: &str) -> bool {
        let original_len = self.packages.len();
        self.packages.retain(|p| p.name != name);
        self.packages.len() < original_len
    }

    /// Get packages that support a specific platform
    pub fn packages_for_platform(&self, platform: &str) -> Vec<&Package> {
        self.packages
            .iter()
            .filter(|p| p.platforms.contains_key(platform))
            .collect()
    }
}

impl Default for SourceManifest {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn installed_names(&self) -> Vec<&str> {
        self.packages.keys().map(|s| s.as_str()).collect()
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
    fn test_source_manifest_upsert() {
        let mut manifest = SourceManifest::new();

        let package = Package {
            name: "test".to_string(),
            description: "Test package".to_string(),
            repo: "https://github.com/test/test".to_string(),
            homepage: None,
            license: Some("MIT".to_string()),
            latest: "1.0.0".to_string(),
            updated_at: Utc::now(),
            platforms: HashMap::new(),
        };

        manifest.upsert_package(package.clone());
        assert_eq!(manifest.packages.len(), 1);
        assert_eq!(manifest.find_package("test").unwrap().name, "test");

        // Update
        let mut updated = package;
        updated.latest = "2.0.0".to_string();
        manifest.upsert_package(updated);
        assert_eq!(manifest.packages.len(), 1);
        assert_eq!(manifest.find_package("test").unwrap().latest, "2.0.0");
    }

    #[test]
    fn test_installed_manifest() {
        let mut manifest = InstalledManifest::new();

        let package = InstalledPackage {
            version: "1.0.0".to_string(),
            platform: "windows-x86_64".to_string(),
            installed_at: Utc::now(),
            install_path: "C:\\Users\\test\\.wenpm\\apps\\test".to_string(),
            files: vec!["bin/test.exe".to_string()],
        };

        manifest.upsert_package("test".to_string(), package);
        assert!(manifest.is_installed("test"));
        assert_eq!(manifest.get_package("test").unwrap().version, "1.0.0");

        manifest.remove_package("test");
        assert!(!manifest.is_installed("test"));
    }
}
