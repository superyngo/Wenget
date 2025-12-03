//! Platform detection and binary matching for WenPM
//!
//! This module handles:
//! - Current platform detection (OS + Architecture)
//! - Binary selection from release assets based on platform
//! - Platform string normalization

use std::collections::HashMap;

/// Supported operating systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Os {
    Windows,
    Linux,
    MacOS,
}

impl Os {
    /// Get the current OS
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(target_os = "macos") {
            Os::MacOS
        } else {
            panic!("Unsupported operating system")
        }
    }

    /// Get OS keywords for matching
    pub fn keywords(&self) -> &[&str] {
        match self {
            Os::Windows => &["windows", "win64", "win32", "pc-windows", "msvc", "win"],
            Os::Linux => &["linux", "unknown-linux"],
            Os::MacOS => &["darwin", "macos", "apple", "osx", "mac"],
        }
    }

    /// Convert to platform string component
    pub fn as_str(&self) -> &str {
        match self {
            Os::Windows => "windows",
            Os::Linux => "linux",
            Os::MacOS => "macos",
        }
    }
}

/// Supported architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X86_64,
    I686,
    Aarch64,
    Armv7,
}

impl Arch {
    /// Get the current architecture
    pub fn current() -> Self {
        if cfg!(target_arch = "x86_64") {
            Arch::X86_64
        } else if cfg!(target_arch = "x86") {
            Arch::I686
        } else if cfg!(target_arch = "aarch64") {
            Arch::Aarch64
        } else if cfg!(target_arch = "arm") {
            Arch::Armv7
        } else {
            panic!("Unsupported architecture")
        }
    }

    /// Get architecture keywords for matching
    pub fn keywords(&self) -> &[&str] {
        match self {
            Arch::X86_64 => &["x86_64", "x64", "amd64"],
            Arch::I686 => &["i686", "x86", "i386", "win32"],
            Arch::Aarch64 => &["aarch64", "arm64"],
            Arch::Armv7 => &["armv7", "armhf"],
        }
    }

    /// Convert to platform string component
    pub fn as_str(&self) -> &str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::I686 => "i686",
            Arch::Aarch64 => "aarch64",
            Arch::Armv7 => "armv7",
        }
    }
}

/// Platform information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
        }
    }

    /// Create a platform from components
    pub fn new(os: Os, arch: Arch) -> Self {
        Self { os, arch }
    }

    /// Get all possible platform identifiers for this platform
    ///
    /// Returns variants like:
    /// - "linux-x86_64"
    /// - "linux-x86_64-musl"
    /// - "linux-x86_64-gnu"
    pub fn possible_identifiers(&self) -> Vec<String> {
        let base = format!("{}", self);
        let mut identifiers = vec![base.clone()];

        // Add Linux variants
        if self.os == Os::Linux {
            identifiers.push(format!("{}-musl", base));
            identifiers.push(format!("{}-gnu", base));
        }

        identifiers
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.os.as_str(), self.arch.as_str())
    }
}

/// Binary asset information
#[derive(Debug, Clone)]
pub struct BinaryAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

/// Binary selector for choosing the right asset from releases
pub struct BinarySelector;

impl BinarySelector {
    /// Select the best binary asset for a given platform
    ///
    /// # Arguments
    /// * `assets` - List of available assets
    /// * `platform` - Target platform
    ///
    /// # Returns
    /// The best matching asset, or None if no suitable asset found
    pub fn select_for_platform(assets: &[BinaryAsset], platform: Platform) -> Option<BinaryAsset> {
        let mut scored_assets: Vec<(usize, &BinaryAsset)> = assets
            .iter()
            .filter_map(|asset| {
                let score = Self::score_asset(&asset.name, platform)?;
                Some((score, asset))
            })
            .collect();

        // Sort by score (highest first)
        scored_assets.sort_by(|a, b| b.0.cmp(&a.0));

        scored_assets.first().map(|(_, asset)| (*asset).clone())
    }

    /// Score an asset filename based on how well it matches the platform
    ///
    /// Returns None if the asset should be excluded
    fn score_asset(filename: &str, platform: Platform) -> Option<usize> {
        let filename_lower = filename.to_lowercase();

        // Exclude certain files
        if Self::should_exclude(&filename_lower) {
            return None;
        }

        let mut score = 0;
        let mut has_os_match = false;
        let mut has_arch_match = false;

        // OS matching
        let os_keywords = platform.os.keywords();
        for keyword in os_keywords {
            if filename_lower.contains(keyword) {
                score += 100;
                has_os_match = true;
                break;
            }
        }

        // Architecture matching
        let arch_keywords = platform.arch.keywords();
        for keyword in arch_keywords {
            if filename_lower.contains(keyword) {
                score += 50;
                has_arch_match = true;
                break;
            }
        }

        // Fallback: If OS matches but no architecture specified, assume common architecture
        // This handles files like "gitui-win.tar.gz" or "app-mac.zip"
        if has_os_match && !has_arch_match {
            // Only apply fallback for x86_64 and aarch64 (most common)
            if platform.arch == Arch::X86_64 {
                // Assume x86_64 for Windows/Linux without arch info
                if platform.os == Os::Windows || platform.os == Os::Linux {
                    score += 25; // Lower than explicit arch match (50)
                    eprintln!("⚠️  Fallback assumption: {} -> {}", filename, platform);
                }
            } else if platform.arch == Arch::Aarch64 {
                // Assume aarch64 for macOS without arch info (Apple Silicon is now standard)
                if platform.os == Os::MacOS {
                    score += 25;
                    eprintln!("⚠️  Fallback assumption: {} -> {}", filename, platform);
                }
            }
        }

        // Linux variant preference: musl > gnu/glibc
        if platform.os == Os::Linux {
            if filename_lower.contains("musl") {
                score += 30;
            } else if filename_lower.contains("gnu") || filename_lower.contains("glibc") {
                score += 20;
            }
        }

        // File format preference: tar.gz > zip > 7z
        if filename_lower.ends_with(".tar.gz") || filename_lower.ends_with(".tgz") {
            score += 10;
        } else if filename_lower.ends_with(".zip") {
            score += 8;
        } else if filename_lower.ends_with(".7z") {
            score += 5;
        }

        // Must have OS match at minimum
        if score >= 100 {
            Some(score)
        } else {
            None
        }
    }

    /// Check if a filename should be excluded from selection
    fn should_exclude(filename: &str) -> bool {
        let excludes = [
            "source",
            ".deb",
            ".rpm",
            ".apk",
            ".dmg",
            ".pkg",
            ".sha256",
            ".sha512",
            ".asc",
            ".sig",
            "checksums",
            "checksum",
            ".txt",
            ".md",
        ];

        excludes.iter().any(|&e| filename.contains(e))
    }

    /// Extract platform information from available assets
    ///
    /// Returns a map of platform identifiers to assets
    pub fn extract_platforms(assets: &[BinaryAsset]) -> HashMap<String, BinaryAsset> {
        let mut platforms = HashMap::new();

        // Try all common platform combinations
        let test_platforms = vec![
            Platform::new(Os::Windows, Arch::X86_64),
            Platform::new(Os::Windows, Arch::I686),
            Platform::new(Os::Windows, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::I686),
            Platform::new(Os::Linux, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::Armv7),
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
        ];

        for platform in test_platforms {
            if let Some(asset) = Self::select_for_platform(assets, platform) {
                // For Linux, try to determine the variant
                if platform.os == Os::Linux {
                    let platform_id = if asset.name.to_lowercase().contains("musl") {
                        format!("{}-musl", platform)
                    } else if asset.name.to_lowercase().contains("gnu")
                        || asset.name.to_lowercase().contains("glibc")
                    {
                        format!("{}-gnu", platform)
                    } else {
                        platform.to_string()
                    };
                    platforms.insert(platform_id, asset);
                } else {
                    platforms.insert(platform.to_string(), asset);
                }
            }
        }

        platforms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_platform() {
        let platform = Platform::current();
        println!("Current platform: {}", platform);
        assert!(matches!(platform.os, Os::Windows | Os::Linux | Os::MacOS));
        assert!(matches!(
            platform.arch,
            Arch::X86_64 | Arch::I686 | Arch::Aarch64 | Arch::Armv7
        ));
    }

    #[test]
    fn test_platform_string() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        assert_eq!(platform.to_string(), "windows-x86_64");

        let platform = Platform::new(Os::Linux, Arch::Aarch64);
        assert_eq!(platform.to_string(), "linux-aarch64");
    }

    #[test]
    fn test_binary_selection() {
        let assets = vec![
            BinaryAsset {
                name: "app-windows-x86_64.zip".to_string(),
                url: "https://example.com/windows.zip".to_string(),
                size: 1000000,
            },
            BinaryAsset {
                name: "app-linux-x86_64-musl.tar.gz".to_string(),
                url: "https://example.com/linux.tar.gz".to_string(),
                size: 1000000,
            },
            BinaryAsset {
                name: "source.tar.gz".to_string(),
                url: "https://example.com/source.tar.gz".to_string(),
                size: 500000,
            },
        ];

        let windows_platform = Platform::new(Os::Windows, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, windows_platform);
        assert!(selected.is_some());
        assert!(selected.unwrap().name.contains("windows"));

        let linux_platform = Platform::new(Os::Linux, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, linux_platform);
        assert!(selected.is_some());
        assert!(selected.unwrap().name.contains("linux"));
    }

    #[test]
    fn test_should_exclude() {
        assert!(BinarySelector::should_exclude("source.tar.gz"));
        assert!(BinarySelector::should_exclude("app.deb"));
        assert!(BinarySelector::should_exclude("checksums.txt"));
        assert!(!BinarySelector::should_exclude("app-linux-x86_64.tar.gz"));
    }

    #[test]
    fn test_linux_variant_preference() {
        let assets = vec![
            BinaryAsset {
                name: "app-linux-x86_64-gnu.tar.gz".to_string(),
                url: "https://example.com/gnu.tar.gz".to_string(),
                size: 1000000,
            },
            BinaryAsset {
                name: "app-linux-x86_64-musl.tar.gz".to_string(),
                url: "https://example.com/musl.tar.gz".to_string(),
                size: 1000000,
            },
        ];

        let linux_platform = Platform::new(Os::Linux, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, linux_platform);
        assert!(selected.is_some());
        // Should prefer musl
        assert!(selected.unwrap().name.contains("musl"));
    }

    #[test]
    fn test_fallback_detection_gitui() {
        // Test fallback detection for gitui-style filenames
        let test_cases = vec![
            ("gitui-win.tar.gz", Platform::new(Os::Windows, Arch::X86_64), true),
            ("gitui-win.msi", Platform::new(Os::Windows, Arch::X86_64), true),
            ("gitui-mac.tar.gz", Platform::new(Os::MacOS, Arch::Aarch64), true),
            ("gitui-linux-x86_64.tar.gz", Platform::new(Os::Linux, Arch::X86_64), true),
        ];

        for (filename, platform, should_match) in test_cases {
            let assets = vec![BinaryAsset {
                name: filename.to_string(),
                url: format!("https://example.com/{}", filename),
                size: 1000000,
            }];

            let selected = BinarySelector::select_for_platform(&assets, platform);

            if should_match {
                assert!(
                    selected.is_some(),
                    "Expected {} to match platform {}",
                    filename,
                    platform
                );
            } else {
                assert!(
                    selected.is_none(),
                    "Expected {} NOT to match platform {}",
                    filename,
                    platform
                );
            }
        }
    }
}
