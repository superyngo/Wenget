//! Installer module for WenPM

pub mod extractor;
pub mod shim;
pub mod symlink;

// Re-export commonly used items
pub use extractor::{extract_archive, find_executable};
pub use shim::create_shim;
pub use symlink::create_symlink;
