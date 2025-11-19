//! Configuration management for WenPM
//!
//! This module handles:
//! - Loading and saving sources.json
//! - Loading and saving installed.json
//! - Directory initialization

use super::manifest::{InstalledManifest, SourceManifest};
use super::paths::WenPaths;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Configuration manager
pub struct Config {
    paths: WenPaths,
}

impl Config {
    /// Create a new Config instance
    pub fn new() -> Result<Self> {
        let paths = WenPaths::new()?;
        Ok(Self { paths })
    }

    /// Get the paths manager
    pub fn paths(&self) -> &WenPaths {
        &self.paths
    }

    /// Initialize WenPM (create directories if needed)
    pub fn init(&self) -> Result<()> {
        self.paths.init_dirs()?;

        // Create empty manifests if they don't exist
        if !self.paths.sources_json().exists() {
            self.save_sources(&SourceManifest::new())?;
        }

        if !self.paths.installed_json().exists() {
            self.save_installed(&InstalledManifest::new())?;
        }

        Ok(())
    }

    /// Check if WenPM is initialized
    pub fn is_initialized(&self) -> bool {
        self.paths.is_initialized()
            && self.paths.sources_json().exists()
            && self.paths.installed_json().exists()
    }

    /// Load sources manifest
    pub fn load_sources(&self) -> Result<SourceManifest> {
        let path = self.paths.sources_json();
        Self::load_json(&path).context("Failed to load sources.json")
    }

    /// Save sources manifest
    pub fn save_sources(&self, manifest: &SourceManifest) -> Result<()> {
        let path = self.paths.sources_json();
        Self::save_json(&path, manifest).context("Failed to save sources.json")
    }

    /// Load installed manifest
    pub fn load_installed(&self) -> Result<InstalledManifest> {
        let path = self.paths.installed_json();
        Self::load_json(&path).context("Failed to load installed.json")
    }

    /// Save installed manifest
    pub fn save_installed(&self, manifest: &InstalledManifest) -> Result<()> {
        let path = self.paths.installed_json();
        Self::save_json(&path, manifest).context("Failed to save installed.json")
    }

    /// Generic JSON loader
    fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from: {}", path.display()))
    }

    /// Generic JSON saver
    fn save_json<T: serde::Serialize>(path: &Path, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .context("Failed to serialize data to JSON")?;

        fs::write(path, json)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        Ok(())
    }

    /// Get or create sources manifest (auto-initialize if needed)
    pub fn get_or_create_sources(&self) -> Result<SourceManifest> {
        if !self.is_initialized() {
            self.init()?;
        }
        self.load_sources()
    }

    /// Get or create installed manifest (auto-initialize if needed)
    pub fn get_or_create_installed(&self) -> Result<InstalledManifest> {
        if !self.is_initialized() {
            self.init()?;
        }
        self.load_installed()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().expect("Failed to create Config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let paths = WenPaths::new().unwrap();

        // For testing, we would need to override the paths
        // This is a simplified version
        let config = Config::new().unwrap();
        (config, temp_dir)
    }

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.is_ok());
    }

    #[test]
    fn test_init() {
        let config = Config::new().unwrap();
        let result = config.init();
        assert!(result.is_ok());
        assert!(config.paths().root().exists());
    }

    #[test]
    fn test_manifest_round_trip() {
        let config = Config::new().unwrap();
        config.init().unwrap();

        let manifest = SourceManifest::new();
        config.save_sources(&manifest).unwrap();

        let loaded = config.load_sources().unwrap();
        assert_eq!(loaded.packages.len(), manifest.packages.len());
    }
}
