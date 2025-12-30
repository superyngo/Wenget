//! Input detection for add command

use crate::installer::script::is_script_input;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    /// GitHub package name or generic identifier
    PackageName,
    /// Local or remote script file
    Script,
    /// Local archive or binary file
    LocalFile,
    /// Direct URL to archive or binary
    DirectUrl,
}

pub fn detect_input_type(input: &str) -> InputType {
    // Check if it's a script first (existing logic)
    if is_script_input(input) {
        return InputType::Script;
    }

    // Check if it's a URL
    if input.starts_with("http://") || input.starts_with("https://") {
        return InputType::DirectUrl;
    }

    // Check if it looks like a local file path
    let path = Path::new(input);
    if path.exists()  // File actually exists
        || path.is_absolute() // Is absolute path
        || input.starts_with("./") || input.starts_with(".\\") // Explicit relative path
        || input.starts_with("../") || input.starts_with("..\\")
    {
        return InputType::LocalFile;
    }

    // Otherwise, assume it's a package name
    InputType::PackageName
}
