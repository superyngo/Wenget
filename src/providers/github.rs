//! GitHub provider implementation

use super::base::SourceProvider;
use crate::core::{BinaryAsset, BinarySelector, Package, Platform, PlatformBinary};
use crate::utils::HttpClient;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;

/// GitHub provider
pub struct GitHubProvider {
    http: HttpClient,
}

impl GitHubProvider {
    /// Create a new GitHub provider
    pub fn new() -> Result<Self> {
        Ok(Self {
            http: HttpClient::new()?,
        })
    }

    /// Parse GitHub URL to extract owner and repo
    ///
    /// Supports:
    /// - https://github.com/owner/repo
    /// - https://github.com/owner/repo/
    /// - https://github.com/owner/repo.git
    fn parse_github_url(&self, url: &str) -> Result<(String, String)> {
        let url = url.trim_end_matches('/').trim_end_matches(".git");

        let parts: Vec<&str> = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("github.com/")
            .split('/')
            .collect();

        if parts.len() < 2 {
            anyhow::bail!("Invalid GitHub URL: {}", url);
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        Ok((owner, repo))
    }

    /// Fetch latest release from GitHub API
    fn fetch_latest_release(&self, owner: &str, repo: &str) -> Result<GitHubRelease> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );

        self.http
            .get_json(&url)
            .with_context(|| format!("Failed to fetch latest release for {}/{}", owner, repo))
    }

    /// Get repository information
    fn fetch_repo_info(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        self.http
            .get_json(&url)
            .with_context(|| format!("Failed to fetch repo info for {}/{}", owner, repo))
    }
}

impl SourceProvider for GitHubProvider {
    fn fetch_package(&self, url: &str) -> Result<Package> {
        log::info!("Fetching package from: {}", url);

        // Parse URL
        let (owner, repo) = self.parse_github_url(url)?;

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

        // Convert GitHub assets to BinaryAsset
        let assets: Vec<BinaryAsset> = release
            .assets
            .iter()
            .map(|a| BinaryAsset {
                name: a.name.clone(),
                url: a.browser_download_url.clone(),
                size: a.size,
            })
            .collect();

        // Extract platforms using BinarySelector
        let platform_map = BinarySelector::extract_platforms(&assets);

        if platform_map.is_empty() {
            anyhow::bail!(
                "No matching binaries found for any platform in {}/{}",
                owner,
                repo
            );
        }

        // Convert to PlatformBinary map
        let platforms: HashMap<String, PlatformBinary> = platform_map
            .into_iter()
            .map(|(platform_id, asset)| {
                (
                    platform_id,
                    PlatformBinary {
                        url: asset.url,
                        size: asset.size,
                        checksum: None,
                    },
                )
            })
            .collect();

        // Create package
        let package = Package {
            name: repo.clone(),
            description: repo_info.description.unwrap_or_else(|| repo.clone()),
            repo: url.to_string(),
            homepage: Some(repo_info.html_url),
            license: repo_info.license.map(|l| l.name),
            latest: release.tag_name.trim_start_matches('v').to_string(),
            updated_at: Utc::now(),
            platforms,
        };

        log::info!(
            "✓ Found {} v{} with {} platform(s)",
            package.name,
            package.latest,
            package.platforms.len()
        );

        Ok(package)
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("github.com")
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

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    description: Option<String>,
    html_url: String,
    license: Option<GitHubLicense>,
}

#[derive(Debug, Deserialize)]
struct GitHubLicense {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        let provider = GitHubProvider::new().unwrap();

        let (owner, repo) = provider
            .parse_github_url("https://github.com/user/repo")
            .unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");

        let (owner, repo) = provider
            .parse_github_url("https://github.com/user/repo/")
            .unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");

        let (owner, repo) = provider
            .parse_github_url("https://github.com/user/repo.git")
            .unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_can_handle() {
        let provider = GitHubProvider::new().unwrap();
        assert!(provider.can_handle("https://github.com/user/repo"));
        assert!(!provider.can_handle("https://gitlab.com/user/repo"));
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
