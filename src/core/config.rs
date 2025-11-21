//! Configuration management for WenPM
//!
//! This module handles:
//! - Loading and saving sources.json
//! - Loading and saving installed.json
//! - Loading and saving buckets.json
//! - Loading and saving manifest-cache.json
//! - Directory initialization

use super::manifest::{InstalledManifest, SourceManifest};
use super::paths::WenPaths;
use crate::bucket::BucketConfig;
use crate::cache::ManifestCache;
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
        let json =
            serde_json::to_string_pretty(data).context("Failed to serialize data to JSON")?;

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

    /// Load bucket config
    pub fn load_buckets(&self) -> Result<BucketConfig> {
        let path = self.paths.buckets_json();
        BucketConfig::load(&path)
    }

    /// Save bucket config
    pub fn save_buckets(&self, config: &BucketConfig) -> Result<()> {
        let path = self.paths.buckets_json();
        config.save(&path)
    }

    /// Get or create bucket config
    pub fn get_or_create_buckets(&self) -> Result<BucketConfig> {
        if !self.is_initialized() {
            self.init()?;
        }
        self.load_buckets()
    }

    /// Load manifest cache
    pub fn load_cache(&self) -> Result<ManifestCache> {
        let path = self.paths.manifest_cache_json();
        ManifestCache::load(&path)
    }

    /// Save manifest cache
    pub fn save_cache(&self, cache: &ManifestCache) -> Result<()> {
        let path = self.paths.manifest_cache_json();
        cache.save(&path)
    }

    /// Invalidate cache (delete the cache file)
    pub fn invalidate_cache(&self) -> Result<()> {
        let path = self.paths.manifest_cache_json();
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove cache file: {}", path.display()))?;
        }
        Ok(())
    }

    /// Get or rebuild manifest cache
    /// Returns the cache if valid, otherwise rebuilds it
    pub fn get_or_rebuild_cache(&self) -> Result<ManifestCache> {
        let cache = self.load_cache()?;

        // Check if cache is valid
        if cache.is_valid() && !cache.packages.is_empty() {
            return Ok(cache);
        }

        // Rebuild cache
        self.rebuild_cache()
    }

    /// Force rebuild manifest cache from sources and buckets
    pub fn rebuild_cache(&self) -> Result<ManifestCache> {
        use crate::cache::build_cache;
        use crate::utils::HttpClient;

        let local_manifest = self.get_or_create_sources()?;
        let bucket_config = self.get_or_create_buckets()?;

        // Fetch bucket manifests
        let fetch_bucket = |bucket: &crate::bucket::Bucket| -> Result<SourceManifest> {
            log::info!("Fetching bucket '{}' from {}", bucket.name, bucket.url);

            let http = HttpClient::new()?;
            let content = http
                .get_text(&bucket.url)
                .with_context(|| format!("Failed to fetch bucket from {}", bucket.url))?;

            // Try to parse as SourceManifest
            let manifest: SourceManifest = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse bucket manifest from {}", bucket.url))?;

            Ok(manifest)
        };

        let cache = build_cache(&local_manifest, &bucket_config, fetch_bucket)?;

        // Save cache
        self.save_cache(&cache)?;

        Ok(cache)
    }

    /// Get packages from cache (preferred) or local sources (fallback)
    /// This is the recommended way to get packages for read operations
    pub fn get_packages_from_cache(&self) -> Result<SourceManifest> {
        // Try to get from cache
        match self.get_or_rebuild_cache() {
            Ok(cache) => Ok(cache.to_source_manifest()),
            Err(e) => {
                // Fallback to local sources if cache fails
                log::warn!("Failed to load cache, falling back to local sources: {}", e);
                self.get_or_create_sources()
            }
        }
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

    #[allow(dead_code)]
    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let _paths = WenPaths::new().unwrap();

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
