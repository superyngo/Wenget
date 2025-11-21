//! Path management for WenPM
//!
//! This module provides utilities for managing all WenPM-related paths:
//! - Root directory: ~/.wenpm/
//! - Sources manifest: ~/.wenpm/sources.json
//! - Installed manifest: ~/.wenpm/installed.json
//! - Apps directory: ~/.wenpm/apps/
//! - Bin directory: ~/.wenpm/bin/
//! - Cache directory: ~/.wenpm/cache/

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// WenPM paths manager
#[derive(Debug, Clone)]
pub struct WenPaths {
    /// Root directory (~/.wenpm/)
    root: PathBuf,
}

impl WenPaths {
    /// Create a new WenPaths instance
    ///
    /// # Errors
    /// Returns an error if the home directory cannot be determined
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to determine home directory")?;
        let root = home.join(".wenpm");
        Ok(Self { root })
    }

    /// Get the root directory (~/.wenpm/)
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the sources manifest path (~/.wenpm/sources.json)
    pub fn sources_json(&self) -> PathBuf {
        self.root.join("sources.json")
    }

    /// Get the installed manifest path (~/.wenpm/installed.json)
    pub fn installed_json(&self) -> PathBuf {
        self.root.join("installed.json")
    }

    /// Get the buckets config path (~/.wenpm/buckets.json)
    pub fn buckets_json(&self) -> PathBuf {
        self.root.join("buckets.json")
    }

    /// Get the manifest cache path (~/.wenpm/manifest-cache.json)
    pub fn manifest_cache_json(&self) -> PathBuf {
        self.root.join("manifest-cache.json")
    }

    /// Get the apps directory (~/.wenpm/apps/)
    pub fn apps_dir(&self) -> PathBuf {
        self.root.join("apps")
    }

    /// Get a specific app's directory (~/.wenpm/apps/{name}/)
    pub fn app_dir(&self, name: &str) -> PathBuf {
        self.apps_dir().join(name)
    }

    /// Get a specific app's bin directory (~/.wenpm/apps/{name}/bin/)
    #[allow(dead_code)]
    pub fn app_bin_dir(&self, name: &str) -> PathBuf {
        self.app_dir(name).join("bin")
    }

    /// Get the bin directory (~/.wenpm/bin/)
    pub fn bin_dir(&self) -> PathBuf {
        self.root.join("bin")
    }

    /// Get the cache directory (~/.wenpm/cache/)
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// Get the downloads directory (~/.wenpm/cache/downloads/)
    pub fn downloads_dir(&self) -> PathBuf {
        self.cache_dir().join("downloads")
    }

    /// Initialize all required directories
    ///
    /// Creates the following directories if they don't exist:
    /// - ~/.wenpm/
    /// - ~/.wenpm/apps/
    /// - ~/.wenpm/bin/
    /// - ~/.wenpm/cache/downloads/
    pub fn init_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root).context("Failed to create WenPM root directory")?;

        std::fs::create_dir_all(self.apps_dir()).context("Failed to create apps directory")?;

        std::fs::create_dir_all(self.bin_dir()).context("Failed to create bin directory")?;

        std::fs::create_dir_all(self.downloads_dir())
            .context("Failed to create downloads directory")?;

        Ok(())
    }

    /// Check if WenPM is initialized (root directory exists)
    pub fn is_initialized(&self) -> bool {
        self.root.exists()
    }

    /// Get the symlink/shim path for an app in the bin directory
    ///
    /// On Unix: ~/.wenpm/bin/{name}
    /// On Windows: ~/.wenpm/bin/{name}.cmd
    pub fn bin_shim_path(&self, name: &str) -> PathBuf {
        #[cfg(windows)]
        {
            self.bin_dir().join(format!("{}.cmd", name))
        }

        #[cfg(not(windows))]
        {
            self.bin_dir().join(name)
        }
    }

    /// Get the platform-specific executable name
    ///
    /// On Windows: {name}.exe
    /// On Unix: {name}
    #[allow(dead_code)]
    pub fn executable_name(name: &str) -> String {
        #[cfg(windows)]
        {
            format!("{}.exe", name)
        }

        #[cfg(not(windows))]
        {
            name.to_string()
        }
    }
}

impl Default for WenPaths {
    fn default() -> Self {
        Self::new().expect("Failed to initialize WenPaths")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_creation() {
        let paths = WenPaths::new().unwrap();
        assert!(paths.root().ends_with(".wenpm"));
        assert!(paths.sources_json().ends_with("sources.json"));
        assert!(paths.installed_json().ends_with("installed.json"));
    }

    #[test]
    fn test_app_paths() {
        let paths = WenPaths::new().unwrap();
        let app_dir = paths.app_dir("test");
        assert!(app_dir.ends_with("apps/test") || app_dir.ends_with("apps\\test"));

        let bin_dir = paths.app_bin_dir("test");
        assert!(bin_dir.ends_with("apps/test/bin") || bin_dir.ends_with("apps\\test\\bin"));
    }

    #[test]
    fn test_executable_name() {
        #[cfg(windows)]
        {
            assert_eq!(WenPaths::executable_name("test"), "test.exe");
        }

        #[cfg(not(windows))]
        {
            assert_eq!(WenPaths::executable_name("test"), "test");
        }
    }

    #[test]
    fn test_bin_shim_path() {
        let paths = WenPaths::new().unwrap();
        let shim = paths.bin_shim_path("test");

        #[cfg(windows)]
        {
            assert!(shim.ends_with("bin\\test.cmd"));
        }

        #[cfg(not(windows))]
        {
            assert!(shim.ends_with("bin/test"));
        }
    }
}
