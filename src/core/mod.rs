//! Core modules for WenPM

pub mod config;
pub mod manifest;
pub mod paths;
pub mod platform;

// Re-export commonly used items
pub use config::Config;
pub use manifest::{InstalledManifest, InstalledPackage, Package, PlatformBinary, SourceManifest};
pub use paths::WenPaths;
pub use platform::{BinaryAsset, BinarySelector, Platform};
