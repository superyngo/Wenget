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
    FreeBSD,
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
        } else if cfg!(target_os = "freebsd") {
            Os::FreeBSD
        } else {
            panic!("Unsupported operating system")
        }
    }

    /// Get OS keywords for matching
    /// Note: "msvc" is a compiler keyword, not an OS keyword
    pub fn keywords(&self) -> &[&str] {
        match self {
            Os::Windows => &["windows", "win64", "win32", "pc-windows", "win"],
            Os::Linux => &["linux", "unknown-linux"],
            Os::MacOS => &["darwin", "macos", "apple", "osx", "mac"],
            Os::FreeBSD => &["freebsd"],
        }
    }

    /// Get the default architecture for this OS when none is specified
    /// - Windows/Linux: default to x86_64
    /// - Darwin: default to aarch64 (Apple Silicon, Rosetta 2 handles x86_64)
    /// - FreeBSD: no default (requires explicit arch)
    pub fn default_arch(&self) -> Option<Arch> {
        match self {
            Os::Windows | Os::Linux => Some(Arch::X86_64),
            Os::MacOS => Some(Arch::Aarch64),
            Os::FreeBSD => None,
        }
    }

    /// Convert to platform string component
    pub fn as_str(&self) -> &str {
        match self {
            Os::Windows => "windows",
            Os::Linux => "linux",
            Os::MacOS => "macos",
            Os::FreeBSD => "freebsd",
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

    /// Resolve the "x86" keyword based on OS context
    /// Darwin: x86 -> x86_64 (32-bit Mac is obsolete)
    /// Others: x86 -> i686
    pub fn resolve_x86_keyword(os: Os) -> Self {
        match os {
            Os::MacOS => Arch::X86_64,
            _ => Arch::I686,
        }
    }
}

/// Compiler/libc variant for platform-specific binaries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compiler {
    /// GNU libc (glibc)
    Gnu,
    /// musl libc (static linking, Alpine-compatible)
    Musl,
    /// Microsoft Visual C++
    Msvc,
}

impl Compiler {
    /// Get compiler keywords for matching
    pub fn keywords(&self) -> &[&str] {
        match self {
            Compiler::Gnu => &["gnu", "glibc"],
            Compiler::Musl => &["musl"],
            Compiler::Msvc => &["msvc"],
        }
    }

    /// Get priority for this compiler on a given OS
    /// Higher values = preferred
    pub fn priority(&self, os: Os) -> u8 {
        match os {
            Os::Linux => match self {
                Compiler::Musl => 3,
                Compiler::Gnu => 2,
                Compiler::Msvc => 1,
            },
            Os::Windows => match self {
                Compiler::Msvc => 3,
                Compiler::Gnu => 2,
                Compiler::Musl => 1,
            },
            Os::MacOS | Os::FreeBSD => 1, // Uniform priority
        }
    }

    /// Convert to platform string component
    pub fn as_str(&self) -> &str {
        match self {
            Compiler::Gnu => "gnu",
            Compiler::Musl => "musl",
            Compiler::Msvc => "msvc",
        }
    }
}

/// Supported file extensions for binary assets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileExtension {
    Exe,
    Zip,
    TarGz,
    TarXz,
    TarBz2,
    SevenZ,
    Unsupported,
}

impl FileExtension {
    /// Detect file extension from filename
    pub fn from_filename(filename: &str) -> Self {
        let lower = filename.to_lowercase();
        if lower.ends_with(".exe") {
            FileExtension::Exe
        } else if lower.ends_with(".zip") {
            FileExtension::Zip
        } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            FileExtension::TarGz
        } else if lower.ends_with(".tar.xz") {
            FileExtension::TarXz
        } else if lower.ends_with(".tar.bz2") {
            FileExtension::TarBz2
        } else if lower.ends_with(".7z") {
            FileExtension::SevenZ
        } else {
            FileExtension::Unsupported
        }
    }

    /// Get format preference score (higher = preferred)
    pub fn format_score(&self) -> usize {
        match self {
            FileExtension::TarGz => 5,
            FileExtension::TarXz => 4,
            FileExtension::Zip => 3,
            FileExtension::TarBz2 => 3,
            FileExtension::SevenZ => 2,
            FileExtension::Exe => 2,
            FileExtension::Unsupported => 0,
        }
    }
}

/// Parsed asset information from filename
#[derive(Debug)]
pub struct ParsedAsset {
    pub extension: FileExtension,
    pub os: Option<Os>,
    pub arch: Option<Arch>,
    pub compiler: Option<Compiler>,
}

/// Unsupported architectures to filter out
const UNSUPPORTED_ARCHS: &[&str] = &[
    "s390x", "ppc64", "ppc64le", "riscv64", "mips", "mips64", "sparc64",
];

impl ParsedAsset {
    /// Parse an asset filename into its components
    pub fn from_filename(filename: &str) -> Self {
        let lower = filename.to_lowercase();
        let extension = FileExtension::from_filename(filename);

        // Detect OS
        let (os, _os_inferred) = Self::detect_os(&lower, extension);

        // Detect architecture (context-aware for x86 keyword)
        let arch = Self::detect_arch(&lower, os);

        // Detect compiler
        let compiler = Self::detect_compiler(&lower);

        ParsedAsset {
            extension,
            os,
            arch,
            compiler,
        }
    }

    /// Check if filename contains unsupported architecture keywords
    pub fn contains_unsupported_arch(filename: &str) -> bool {
        let lower = filename.to_lowercase();
        UNSUPPORTED_ARCHS.iter().any(|arch| lower.contains(arch))
    }

    /// Detect OS from filename
    fn detect_os(filename: &str, ext: FileExtension) -> (Option<Os>, bool) {
        // Check explicit OS keywords
        // Note: Order matters! MacOS must be checked before Windows
        // because "darwin" contains "win" as substring
        let all_os = [Os::MacOS, Os::FreeBSD, Os::Linux, Os::Windows];
        for os in all_os {
            for keyword in os.keywords() {
                if filename.contains(keyword) {
                    return (Some(os), false);
                }
            }
        }

        // .exe implies Windows
        if ext == FileExtension::Exe {
            return (Some(Os::Windows), true);
        }

        (None, false)
    }

    /// Detect architecture from filename (context-aware)
    fn detect_arch(filename: &str, os: Option<Os>) -> Option<Arch> {
        // Handle special "x86" keyword first (before generic keyword matching)
        // x86 on Darwin -> x86_64 (32-bit Mac is obsolete)
        // x86 on others -> i686
        if filename.contains("x86") && !filename.contains("x86_64") {
            if let Some(os) = os {
                return Some(Arch::resolve_x86_keyword(os));
            }
            // Default to i686 if OS is unknown
            return Some(Arch::I686);
        }

        // Check all architecture keywords
        let all_arch = [Arch::X86_64, Arch::Aarch64, Arch::Armv7, Arch::I686];
        for arch in all_arch {
            for keyword in arch.keywords() {
                // Skip "x86" since we handled it above
                if *keyword == "x86" {
                    continue;
                }
                if filename.contains(keyword) {
                    return Some(arch);
                }
            }
        }

        None
    }

    /// Detect compiler/libc from filename
    fn detect_compiler(filename: &str) -> Option<Compiler> {
        let all_compilers = [Compiler::Musl, Compiler::Msvc, Compiler::Gnu];
        for compiler in all_compilers {
            for keyword in compiler.keywords() {
                if filename.contains(keyword) {
                    return Some(compiler);
                }
            }
        }
        None
    }
}

/// Platform information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
    pub compiler: Option<Compiler>,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
            compiler: None,
        }
    }

    /// Create a platform from components
    pub fn new(os: Os, arch: Arch) -> Self {
        Self {
            os,
            arch,
            compiler: None,
        }
    }

    /// Create a platform with compiler specification
    #[allow(dead_code)]
    pub fn with_compiler(os: Os, arch: Arch, compiler: Compiler) -> Self {
        Self {
            os,
            arch,
            compiler: Some(compiler),
        }
    }

    /// Get all possible platform identifiers for this platform
    ///
    /// Returns variants like:
    /// - "linux-x86_64"
    /// - "linux-x86_64-musl"
    /// - "linux-x86_64-gnu"
    /// - "windows-x86_64-msvc"
    /// - "windows-x86_64-gnu"
    pub fn possible_identifiers(&self) -> Vec<String> {
        let base = format!("{}", self);
        let mut identifiers = vec![base.clone()];

        // Add compiler variants
        match self.os {
            Os::Linux => {
                identifiers.push(format!("{}-musl", base));
                identifiers.push(format!("{}-gnu", base));
            }
            Os::Windows => {
                identifiers.push(format!("{}-msvc", base));
                identifiers.push(format!("{}-gnu", base));
            }
            _ => {}
        }

        identifiers
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.compiler {
            Some(compiler) => write!(
                f,
                "{}-{}-{}",
                self.os.as_str(),
                self.arch.as_str(),
                compiler.as_str()
            ),
            None => write!(f, "{}-{}", self.os.as_str(), self.arch.as_str()),
        }
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
    /// New 4-component scoring algorithm:
    /// - OS match: +100 (mandatory)
    /// - Explicit arch match: +50
    /// - Default arch match: +25
    /// - Compiler priority: +10/20/30 based on OS preference
    /// - File format: +2 to +5
    ///
    /// Returns None if the asset should be excluded
    fn score_asset(filename: &str, platform: Platform) -> Option<usize> {
        let filename_lower = filename.to_lowercase();

        // Exclude certain files
        if Self::should_exclude(&filename_lower) {
            return None;
        }

        // Filter out unsupported architectures
        if ParsedAsset::contains_unsupported_arch(&filename_lower) {
            return None;
        }

        // Parse the asset filename
        let parsed = ParsedAsset::from_filename(filename);

        // Skip if extension is unsupported
        if parsed.extension == FileExtension::Unsupported {
            return None;
        }

        let mut score = 0;

        // OS matching (mandatory)
        let os_matches = match parsed.os {
            Some(os) => os == platform.os,
            None => false,
        };

        if !os_matches {
            return None;
        }
        score += 100;

        // Architecture matching
        match parsed.arch {
            Some(arch) if arch == platform.arch => {
                // Explicit architecture match
                score += 50;
            }
            Some(_) => {
                // Explicit architecture mismatch - exclude
                return None;
            }
            None => {
                // No explicit architecture - check if platform's arch matches OS default
                if let Some(default_arch) = platform.os.default_arch() {
                    if platform.arch == default_arch {
                        // Use default architecture (lower score than explicit)
                        score += 25;
                    }
                    // If platform arch doesn't match default, still allow but no arch bonus
                } else {
                    // OS has no default (FreeBSD) - require explicit arch
                    return None;
                }
            }
        }

        // Compiler scoring based on OS-specific priority
        if let Some(compiler) = parsed.compiler {
            let priority = compiler.priority(platform.os);
            score += (priority as usize) * 10;
        }

        // File format preference
        score += parsed.extension.format_score();

        Some(score)
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
            ".msi",
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
            // Windows
            Platform::new(Os::Windows, Arch::X86_64),
            Platform::new(Os::Windows, Arch::I686),
            Platform::new(Os::Windows, Arch::Aarch64),
            // Linux
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::I686),
            Platform::new(Os::Linux, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::Armv7),
            // macOS
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            // FreeBSD
            Platform::new(Os::FreeBSD, Arch::X86_64),
            Platform::new(Os::FreeBSD, Arch::Aarch64),
        ];

        for platform in test_platforms {
            if let Some(asset) = Self::select_for_platform(assets, platform) {
                let asset_lower = asset.name.to_lowercase();

                // Determine compiler variant from the asset
                let compiler = if asset_lower.contains("musl") {
                    Some(Compiler::Musl)
                } else if asset_lower.contains("msvc") {
                    Some(Compiler::Msvc)
                } else if asset_lower.contains("gnu") || asset_lower.contains("glibc") {
                    Some(Compiler::Gnu)
                } else {
                    None
                };

                // Build platform identifier
                let platform_id = match compiler {
                    Some(c) => format!("{}-{}", platform, c.as_str()),
                    None => platform.to_string(),
                };

                platforms.insert(platform_id, asset);
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
        assert!(matches!(
            platform.os,
            Os::Windows | Os::Linux | Os::MacOS | Os::FreeBSD
        ));
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

        // Test platform with compiler
        let platform = Platform::with_compiler(Os::Linux, Arch::X86_64, Compiler::Musl);
        assert_eq!(platform.to_string(), "linux-x86_64-musl");
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
    fn test_linux_prefers_musl_over_gnu() {
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
        // Should prefer musl (priority 3) over gnu (priority 2)
        assert!(
            selected.unwrap().name.contains("musl"),
            "Linux should prefer musl over gnu"
        );
    }

    #[test]
    fn test_windows_prefers_msvc_over_gnu() {
        let assets = vec![
            BinaryAsset {
                name: "app-windows-x86_64-gnu.zip".to_string(),
                url: "https://example.com/gnu.zip".to_string(),
                size: 1000000,
            },
            BinaryAsset {
                name: "app-windows-x86_64-msvc.zip".to_string(),
                url: "https://example.com/msvc.zip".to_string(),
                size: 1000000,
            },
        ];

        let windows_platform = Platform::new(Os::Windows, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, windows_platform);
        assert!(selected.is_some());
        // Should prefer msvc (priority 3) over gnu (priority 2)
        assert!(
            selected.unwrap().name.contains("msvc"),
            "Windows should prefer msvc over gnu"
        );
    }

    #[test]
    fn test_exe_implies_windows() {
        let assets = vec![BinaryAsset {
            name: "tool.exe".to_string(),
            url: "https://example.com/tool.exe".to_string(),
            size: 1000000,
        }];

        // .exe should match Windows (inferred OS)
        let windows_platform = Platform::new(Os::Windows, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, windows_platform);
        assert!(
            selected.is_some(),
            ".exe file should match Windows platform"
        );

        // .exe should NOT match Linux
        let linux_platform = Platform::new(Os::Linux, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, linux_platform);
        assert!(
            selected.is_none(),
            ".exe file should NOT match Linux platform"
        );
    }

    #[test]
    fn test_x86_darwin_is_x86_64() {
        // On Darwin, x86 keyword should resolve to x86_64 (32-bit Mac is obsolete)
        let assets = vec![BinaryAsset {
            name: "tool-darwin-x86.tar.gz".to_string(),
            url: "https://example.com/tool.tar.gz".to_string(),
            size: 1000000,
        }];

        // Should match x86_64 (not i686)
        let macos_x64 = Platform::new(Os::MacOS, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, macos_x64);
        assert!(selected.is_some(), "darwin-x86 should match macOS x86_64");

        // Should NOT match i686
        let macos_i686 = Platform::new(Os::MacOS, Arch::I686);
        let selected = BinarySelector::select_for_platform(&assets, macos_i686);
        assert!(selected.is_none(), "darwin-x86 should NOT match macOS i686");
    }

    #[test]
    fn test_x86_linux_is_i686() {
        // On Linux, x86 keyword should resolve to i686
        let assets = vec![BinaryAsset {
            name: "tool-linux-x86.tar.gz".to_string(),
            url: "https://example.com/tool.tar.gz".to_string(),
            size: 1000000,
        }];

        // Should match i686
        let linux_i686 = Platform::new(Os::Linux, Arch::I686);
        let selected = BinarySelector::select_for_platform(&assets, linux_i686);
        assert!(selected.is_some(), "linux-x86 should match Linux i686");

        // Should NOT match x86_64
        let linux_x64 = Platform::new(Os::Linux, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, linux_x64);
        assert!(
            selected.is_none(),
            "linux-x86 should NOT match Linux x86_64"
        );
    }

    #[test]
    fn test_darwin_defaults_to_aarch64() {
        // Darwin without explicit arch should default to aarch64 (Rosetta 2 handles x86_64)
        let assets = vec![BinaryAsset {
            name: "tool-darwin.tar.gz".to_string(),
            url: "https://example.com/tool.tar.gz".to_string(),
            size: 1000000,
        }];

        // Should match aarch64 (default for Darwin)
        let macos_aarch64 = Platform::new(Os::MacOS, Arch::Aarch64);
        let selected = BinarySelector::select_for_platform(&assets, macos_aarch64);
        assert!(
            selected.is_some(),
            "darwin without arch should match macOS aarch64 (default)"
        );
    }

    #[test]
    fn test_windows_defaults_to_x86_64() {
        // Windows without explicit arch should default to x86_64
        let assets = vec![BinaryAsset {
            name: "tool-windows.zip".to_string(),
            url: "https://example.com/tool.zip".to_string(),
            size: 1000000,
        }];

        // Should match x86_64 (default for Windows)
        let windows_x64 = Platform::new(Os::Windows, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, windows_x64);
        assert!(
            selected.is_some(),
            "windows without arch should match Windows x86_64 (default)"
        );
    }

    #[test]
    fn test_skip_unsupported_architectures() {
        let unsupported_assets = vec![
            BinaryAsset {
                name: "tool-linux-s390x.tar.gz".to_string(),
                url: "https://example.com/s390x.tar.gz".to_string(),
                size: 1000000,
            },
            BinaryAsset {
                name: "tool-linux-ppc64le.tar.gz".to_string(),
                url: "https://example.com/ppc64le.tar.gz".to_string(),
                size: 1000000,
            },
            BinaryAsset {
                name: "tool-linux-riscv64.tar.gz".to_string(),
                url: "https://example.com/riscv64.tar.gz".to_string(),
                size: 1000000,
            },
        ];

        // None of these should match any supported platform
        let linux_x64 = Platform::new(Os::Linux, Arch::X86_64);
        for asset in &unsupported_assets {
            let assets = vec![asset.clone()];
            let selected = BinarySelector::select_for_platform(&assets, linux_x64);
            assert!(
                selected.is_none(),
                "Unsupported arch {} should be filtered out",
                asset.name
            );
        }
    }

    #[test]
    fn test_freebsd_support() {
        let assets = vec![BinaryAsset {
            name: "tool-freebsd-x86_64.tar.gz".to_string(),
            url: "https://example.com/freebsd.tar.gz".to_string(),
            size: 1000000,
        }];

        let freebsd_x64 = Platform::new(Os::FreeBSD, Arch::X86_64);
        let selected = BinarySelector::select_for_platform(&assets, freebsd_x64);
        assert!(selected.is_some(), "Should match FreeBSD x86_64");

        // FreeBSD without explicit arch should NOT match (no default arch)
        let assets_no_arch = vec![BinaryAsset {
            name: "tool-freebsd.tar.gz".to_string(),
            url: "https://example.com/freebsd.tar.gz".to_string(),
            size: 1000000,
        }];
        let selected = BinarySelector::select_for_platform(&assets_no_arch, freebsd_x64);
        assert!(
            selected.is_none(),
            "FreeBSD without arch should NOT match (requires explicit arch)"
        );
    }

    #[test]
    fn test_fallback_detection_gitui() {
        // Test fallback detection for gitui-style filenames
        let test_cases = vec![
            (
                "gitui-win.tar.gz",
                Platform::new(Os::Windows, Arch::X86_64),
                true,
            ),
            (
                "gitui-mac.tar.gz",
                Platform::new(Os::MacOS, Arch::Aarch64),
                true,
            ),
            (
                "gitui-linux-x86_64.tar.gz",
                Platform::new(Os::Linux, Arch::X86_64),
                true,
            ),
            // .msi files should be excluded
            (
                "gitui-win.msi",
                Platform::new(Os::Windows, Arch::X86_64),
                false,
            ),
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

    #[test]
    fn test_compiler_priority() {
        assert_eq!(Compiler::Musl.priority(Os::Linux), 3);
        assert_eq!(Compiler::Gnu.priority(Os::Linux), 2);
        assert_eq!(Compiler::Msvc.priority(Os::Windows), 3);
        assert_eq!(Compiler::Gnu.priority(Os::Windows), 2);
    }

    #[test]
    fn test_os_default_arch() {
        assert_eq!(Os::Windows.default_arch(), Some(Arch::X86_64));
        assert_eq!(Os::Linux.default_arch(), Some(Arch::X86_64));
        assert_eq!(Os::MacOS.default_arch(), Some(Arch::Aarch64));
        assert_eq!(Os::FreeBSD.default_arch(), None);
    }

    #[test]
    fn test_arch_resolve_x86_keyword() {
        assert_eq!(Arch::resolve_x86_keyword(Os::MacOS), Arch::X86_64);
        assert_eq!(Arch::resolve_x86_keyword(Os::Linux), Arch::I686);
        assert_eq!(Arch::resolve_x86_keyword(Os::Windows), Arch::I686);
    }
}
