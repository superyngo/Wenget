//! Manifest cache management for WenPM
//!
//! The cache merges local sources and bucket sources into a unified view.
//! This reduces GitHub API calls and improves performance.

use crate::bucket::{Bucket, BucketConfig};
use crate::core::{Package, SourceManifest};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Package source origin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PackageSource {
    /// From local sources
    Local,
    /// From a bucket
    Bucket { name: String },
}

impl PackageSource {
    /// Create a bucket source
    pub fn bucket(name: String) -> Self {
        Self::Bucket { name }
    }

    /// Get display name
    #[allow(dead_code)]
    pub fn display(&self) -> String {
        match self {
            Self::Local => "local".to_string(),
            Self::Bucket { name } => format!("bucket:{}", name),
        }
    }
}

/// Package with source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPackage {
    /// The package data
    #[serde(flatten)]
    pub package: Package,

    /// Source origin
    pub source: PackageSource,
}

/// Source information in cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSourceInfo {
    /// Source type
    #[serde(flatten)]
    pub source: PackageSource,

    /// Number of packages from this source
    pub package_count: usize,

    /// Last fetch time (for buckets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fetched: Option<DateTime<Utc>>,

    /// Bucket URL (for buckets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Manifest cache view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestCache {
    /// Cache format version
    pub version: String,

    /// Last update timestamp
    pub last_updated: DateTime<Utc>,

    /// Time-to-live in seconds
    #[serde(default = "default_ttl")]
    pub ttl_seconds: i64,

    /// Source information
    pub sources: HashMap<String, CachedSourceInfo>,

    /// Cached packages (key: repo URL)
    pub packages: HashMap<String, CachedPackage>,
}

fn default_ttl() -> i64 {
    86400 // 24 hours
}

impl ManifestCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            last_updated: Utc::now(),
            ttl_seconds: default_ttl(),
            sources: HashMap::new(),
            packages: HashMap::new(),
        }
    }

    /// Load cache from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read cache: {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse cache: {}", path.display()))
    }

    /// Save cache to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create cache directory: {}", parent.display())
            })?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize cache")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write cache: {}", path.display()))
    }

    /// Check if cache is valid (not expired)
    pub fn is_valid(&self) -> bool {
        let age = Utc::now() - self.last_updated;
        age.num_seconds() < self.ttl_seconds
    }

    /// Add a package to cache
    pub fn add_package(&mut self, package: Package, source: PackageSource) {
        let repo = package.repo.clone();
        self.packages
            .insert(repo, CachedPackage { package, source });
    }

    /// Get all packages as Vec (for compatibility with SourceManifest)
    pub fn get_packages(&self) -> Vec<Package> {
        self.packages
            .values()
            .map(|cp| cp.package.clone())
            .collect()
    }

    /// Convert cache to SourceManifest for compatibility
    pub fn to_source_manifest(&self) -> SourceManifest {
        SourceManifest {
            packages: self.get_packages(),
        }
    }

    /// Find a package by name
    #[allow(dead_code)]
    pub fn find_package(&self, name: &str) -> Option<&CachedPackage> {
        self.packages.values().find(|cp| cp.package.name == name)
    }

    /// Get packages filtered by source
    #[allow(dead_code)]
    pub fn packages_by_source(&self, source_type: &PackageSource) -> Vec<&CachedPackage> {
        self.packages
            .values()
            .filter(|cp| &cp.source == source_type)
            .collect()
    }
}

impl Default for ManifestCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Build cache from local sources and buckets
pub fn build_cache(
    local_manifest: &SourceManifest,
    bucket_config: &BucketConfig,
    fetch_bucket_fn: impl Fn(&Bucket) -> Result<SourceManifest>,
) -> Result<ManifestCache> {
    let mut cache = ManifestCache::new();
    cache.last_updated = Utc::now();

    // 1. Add packages from buckets first (lower priority)
    let enabled_buckets = bucket_config.enabled_buckets();

    for bucket in enabled_buckets {
        let source_key = format!("bucket:{}", bucket.name);
        let now = Utc::now();

        match fetch_bucket_fn(bucket) {
            Ok(manifest) => {
                let package_count = manifest.packages.len();

                // Add packages
                for package in manifest.packages {
                    cache.add_package(package, PackageSource::bucket(bucket.name.clone()));
                }

                // Record source info
                cache.sources.insert(
                    source_key,
                    CachedSourceInfo {
                        source: PackageSource::bucket(bucket.name.clone()),
                        package_count,
                        last_fetched: Some(now),
                        url: Some(bucket.url.clone()),
                    },
                );
            }
            Err(e) => {
                log::warn!("Failed to fetch bucket '{}': {}", bucket.name, e);
                // Continue with other buckets
            }
        }
    }

    // 2. Add packages from local sources (higher priority, will overwrite bucket packages)
    let local_count = local_manifest.packages.len();
    for package in &local_manifest.packages {
        cache.add_package(package.clone(), PackageSource::Local);
    }

    // Record local source info
    cache.sources.insert(
        "local".to_string(),
        CachedSourceInfo {
            source: PackageSource::Local,
            package_count: local_count,
            last_fetched: None,
            url: None,
        },
    );

    Ok(cache)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cache_new() {
        let cache = ManifestCache::new();
        assert_eq!(cache.version, "1.0");
        assert_eq!(cache.packages.len(), 0);
        assert_eq!(cache.sources.len(), 0);
    }

    #[test]
    fn test_add_package() {
        let mut cache = ManifestCache::new();

        let package = Package {
            name: "test".to_string(),
            description: "Test package".to_string(),
            repo: "https://github.com/test/test".to_string(),
            homepage: None,
            license: None,
            platforms: HashMap::new(),
        };

        cache.add_package(package.clone(), PackageSource::Local);
        assert_eq!(cache.packages.len(), 1);

        let cached = cache.find_package("test").unwrap();
        assert_eq!(cached.package.name, "test");
        assert_eq!(cached.source, PackageSource::Local);
    }

    #[test]
    fn test_is_valid() {
        let mut cache = ManifestCache::new();

        // Fresh cache should be valid
        assert!(cache.is_valid());

        // Expired cache should be invalid
        cache.last_updated = Utc::now() - chrono::Duration::days(2);
        assert!(!cache.is_valid());
    }

    #[test]
    fn test_package_source_display() {
        let local = PackageSource::Local;
        assert_eq!(local.display(), "local");

        let bucket = PackageSource::bucket("official".to_string());
        assert_eq!(bucket.display(), "bucket:official");
    }
}
