//! GitHub provider implementation

use super::base::SourceProvider;
use crate::core::{BinaryAsset, BinarySelector, Package, PlatformBinary};
use crate::utils::HttpClient;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// GitHub provider
pub struct GitHubProvider {
    http: HttpClient,
}

impl GitHubProvider {
    /// Create a new GitHub provider without authentication
    pub fn new() -> Result<Self> {
        Self::with_token(None)
    }

    /// Create a new GitHub provider with optional token for authentication
    pub fn with_token(token: Option<String>) -> Result<Self> {
        Ok(Self {
            http: HttpClient::with_token(token)?,
        })
    }

    /// Parse GitHub URL to extract owner and repo
    ///
    /// Supports:
    /// - https://github.com/owner/repo
    /// - https://github.com/owner/repo/
    /// - https://github.com/owner/repo.git
    pub fn parse_github_url(url: &str) -> Option<(String, String)> {
        let url = url.trim_end_matches('/').trim_end_matches(".git");

        let parts: Vec<&str> = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("github.com/")
            .split('/')
            .collect();

        if parts.len() >= 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Fetch latest release from GitHub API
    pub fn fetch_latest_release(&self, owner: &str, repo: &str) -> Result<GitHubRelease> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );

        self.http
            .get_json(&url)
            .with_context(|| format!("Failed to fetch latest release for {}/{}", owner, repo))
    }

    /// Get repository information
    pub fn fetch_repo_info(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        self.http
            .get_json(&url)
            .with_context(|| format!("Failed to fetch repo info for {}/{}", owner, repo))
    }

    /// Fetch latest version for a repository
    pub fn fetch_latest_version(&self, repo_url: &str) -> Result<String> {
        let (owner, repo) = Self::parse_github_url(repo_url)
            .ok_or_else(|| anyhow::anyhow!("Invalid GitHub URL: {}", repo_url))?;
        let release = self.fetch_latest_release(&owner, &repo)?;
        Ok(release.tag_name.trim_start_matches('v').to_string())
    }

    /// Convert GitHub release assets to platform binaries map
    ///
    /// This is the shared logic used by both `fetch_package` and bucket manifest generation.
    /// Returns a map where each platform can have MULTIPLE binaries (Vec<PlatformBinary>).
    pub fn extract_platform_binaries(
        assets: &[GitHubAsset],
    ) -> HashMap<String, Vec<PlatformBinary>> {
        // Convert GitHub assets to BinaryAsset
        let binary_assets: Vec<BinaryAsset> = assets
            .iter()
            .map(|a| BinaryAsset {
                name: a.name.clone(),
                url: a.browser_download_url.clone(),
                size: a.size,
            })
            .collect();

        // Extract platforms using BinarySelector (now returns Vec<BinaryAsset> per platform)
        let platform_map = BinarySelector::extract_platforms(&binary_assets);

        // Convert to Vec<PlatformBinary> map
        platform_map
            .into_iter()
            .map(|(platform_id, assets_vec)| {
                let binaries: Vec<PlatformBinary> = assets_vec
                    .into_iter()
                    .map(|asset| PlatformBinary {
                        url: asset.url,
                        size: asset.size,
                        checksum: None,
                        asset_name: asset.name, // NEW: Store original asset filename
                    })
                    .collect();
                (platform_id, binaries)
            })
            .collect()
    }
}

impl SourceProvider for GitHubProvider {
    fn fetch_package(&self, url: &str) -> Result<Package> {
        log::info!("Fetching package from: {}", url);

        // Parse URL
        let (owner, repo) = Self::parse_github_url(url)
            .ok_or_else(|| anyhow::anyhow!("Invalid GitHub URL: {}", url))?;

        // Fetch repo info for description and license
        let repo_info = self.fetch_repo_info(&owner, &repo)?;

        // Fetch latest release
        let release = self.fetch_latest_release(&owner, &repo)?;

        if release.assets.is_empty() {
            anyhow::bail!(
                "No binary assets found in latest release for {}/{}",
                owner,
                repo
            );
        }

        // Use shared platform extraction logic
        let platforms = Self::extract_platform_binaries(&release.assets);

        if platforms.is_empty() {
            anyhow::bail!(
                "No matching binaries found for any platform in {}/{}",
                owner,
                repo
            );
        }

        // Create package
        let package = Package {
            name: repo.clone(),
            description: repo_info.description.unwrap_or_else(|| repo.clone()),
            repo: url.to_string(),
            homepage: Some(repo_info.html_url),
            license: repo_info.license.map(|l| l.name),
            platforms,
        };

        let version = release.tag_name.trim_start_matches('v').to_string();
        log::info!(
            "✓ Found {} v{} with {} platform(s)",
            package.name,
            version,
            package.platforms.len()
        );

        Ok(package)
    }

    fn name(&self) -> &str {
        "GitHub"
    }
}

impl Default for GitHubProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create GitHub provider")
    }
}

// GitHub API response structures

/// GitHub release information
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    /// Release tag name (e.g., "v1.0.0")
    pub tag_name: String,
    /// Release assets (downloadable files)
    pub assets: Vec<GitHubAsset>,
}

/// GitHub release asset (downloadable file)
#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    /// Asset filename
    pub name: String,
    /// Direct download URL
    pub browser_download_url: String,
    /// File size in bytes
    pub size: u64,
}

/// GitHub repository information
#[derive(Debug, Deserialize)]
pub struct GitHubRepo {
    /// Repository name
    pub name: String,
    /// Repository description
    pub description: Option<String>,
    /// Repository URL
    pub html_url: String,
    /// Homepage URL (if set)
    pub homepage: Option<String>,
    /// License information
    pub license: Option<GitHubLicense>,
}

/// GitHub license information
#[derive(Debug, Deserialize)]
pub struct GitHubLicense {
    /// License name (e.g., "MIT License")
    pub name: String,
    /// SPDX identifier (e.g., "MIT")
    pub spdx_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        let result = GitHubProvider::parse_github_url("https://github.com/user/repo");
        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");

        let result = GitHubProvider::parse_github_url("https://github.com/user/repo/");
        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");

        let result = GitHubProvider::parse_github_url("https://github.com/user/repo.git");
        assert!(result.is_some());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");

        // Invalid URL
        let result = GitHubProvider::parse_github_url("https://github.com/user");
        assert!(result.is_none());
    }

    #[test]
    #[ignore] // Requires network access
    fn test_fetch_package() {
        let provider = GitHubProvider::new().unwrap();
        // Test with a real repo that has releases
        let result = provider.fetch_package("https://github.com/BurntSushi/ripgrep");
        assert!(result.is_ok());
    }
}
