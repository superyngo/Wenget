//! Core modules for WenPM

pub mod config;
pub mod manifest;
pub mod paths;
pub mod platform;
pub mod repair;

// Re-export commonly used items
pub use config::Config;
pub use manifest::{InstalledManifest, InstalledPackage, Package, PlatformBinary};
pub use paths::WenPaths;
#[allow(unused_imports)]
pub use platform::{Arch, BinaryAsset, BinarySelector, Compiler, FileExtension, Os, Platform};
