//! Archive extraction utilities

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

/// Extract an archive file to a destination directory
/// For standalone executables, copies them directly to the destination
pub fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<Vec<String>> {
    log::info!("Extracting: {}", archive_path.display());
    log::debug!("Destination: {}", dest_dir.display());

    // Create destination directory
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create directory: {}", dest_dir.display()))?;

    // Determine archive type by extension
    let filename = archive_path
        .file_name()
        .and_then(|s| s.to_str())
        .context("Invalid file name")?;

    let extracted_files = if is_standalone_executable(filename) {
        // Handle standalone executable
        extract_standalone_executable(archive_path, dest_dir)?
    } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest_dir)?
    } else if filename.ends_with(".tar.xz") {
        extract_tar_xz(archive_path, dest_dir)?
    } else if filename.ends_with(".zip") {
        extract_zip(archive_path, dest_dir)?
    } else {
        anyhow::bail!("Unsupported archive format: {}", filename);
    };

    log::info!("Extracted {} file(s)", extracted_files.len());

    Ok(extracted_files)
}

/// Check if a file is a standalone executable (not an archive)
fn is_standalone_executable(filename: &str) -> bool {
    // Windows executables
    if cfg!(windows) && filename.ends_with(".exe") {
        return true;
    }

    // Unix/Linux/macOS binaries often have no extension or are AppImage
    if cfg!(unix) {
        if filename.ends_with(".AppImage") {
            return true;
        }
        // Check if it has no common archive extension
        let archive_extensions = [".zip", ".tar", ".gz", ".xz", ".bz2", ".7z", ".rar"];
        if !archive_extensions.iter().any(|ext| filename.contains(ext)) {
            // Could be a standalone binary
            return true;
        }
    }

    false
}

/// "Extract" a standalone executable by copying it to the destination directory
fn extract_standalone_executable(executable_path: &Path, dest_dir: &Path) -> Result<Vec<String>> {
    let filename = executable_path
        .file_name()
        .context("Invalid executable filename")?;

    let dest_path = dest_dir.join(filename);

    // Copy the executable to the destination
    fs::copy(executable_path, &dest_path).with_context(|| {
        format!(
            "Failed to copy executable from {} to {}",
            executable_path.display(),
            dest_path.display()
        )
    })?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_path, perms)?;
    }

    let relative_path = filename.to_string_lossy().to_string();
    Ok(vec![relative_path])
}

/// Extract a .tar.gz file
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<Vec<String>> {
    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;

    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    extract_tar_archive(&mut archive, dest_dir)
}

/// Extract a .tar.xz file
fn extract_tar_xz(archive_path: &Path, dest_dir: &Path) -> Result<Vec<String>> {
    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;

    let decoder = XzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    extract_tar_archive(&mut archive, dest_dir)
}

/// Extract a tar archive (common logic for .tar.gz and .tar.xz)
fn extract_tar_archive<R: std::io::Read>(
    archive: &mut Archive<R>,
    dest_dir: &Path,
) -> Result<Vec<String>> {
    let mut extracted_files = Vec::new();

    for entry_result in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let mut entry = entry_result.context("Failed to read entry")?;

        let path = entry.path().context("Failed to get entry path")?;
        let path_str = path.to_string_lossy().to_string();

        // Skip directories
        if path_str.ends_with('/') {
            continue;
        }

        // Extract file
        let dest_path = dest_dir.join(&path);

        // Create parent directory
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        entry
            .unpack(&dest_path)
            .with_context(|| format!("Failed to extract: {}", path_str))?;

        // Set executable permission on Unix
        #[cfg(unix)]
        {
            if is_executable(&mut entry)? {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&dest_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest_path, perms)?;
            }
        }

        extracted_files.push(path_str);
    }

    Ok(extracted_files)
}

/// Check if a tar entry is executable
#[cfg(unix)]
fn is_executable<R: std::io::Read>(entry: &mut tar::Entry<R>) -> Result<bool> {
    let mode = entry.header().mode()?;
    Ok(mode & 0o111 != 0)
}

/// Extract a .zip file
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<Vec<String>> {
    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;

    let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

    let mut extracted_files = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Failed to read ZIP entry")?;

        let file_path = file
            .enclosed_name()
            .context("Invalid file path in ZIP")?
            .to_owned();

        let dest_path = dest_dir.join(&file_path);

        if file.is_dir() {
            fs::create_dir_all(&dest_path)?;
            continue;
        }

        // Create parent directory
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Extract file
        let mut dest_file = File::create(&dest_path)
            .with_context(|| format!("Failed to create file: {}", dest_path.display()))?;

        std::io::copy(&mut file, &mut dest_file).context("Failed to extract file")?;

        // Set executable permission on Unix
        #[cfg(unix)]
        {
            if let Some(mode) = file.unix_mode() {
                if mode & 0o111 != 0 {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&dest_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&dest_path, perms)?;
                }
            }
        }

        extracted_files.push(file_path.to_string_lossy().to_string());
    }

    Ok(extracted_files)
}

/// Candidate executable with priority score
#[derive(Debug, Clone)]
pub struct ExecutableCandidate {
    /// Relative path to the executable
    pub path: String,
    /// Priority score (higher is better)
    pub score: u32,
    /// Human-readable reason for this candidate
    pub reason: String,
}

/// Check if a filename should be excluded as a non-executable documentation/config file
fn is_excluded_file(filename: &str, file_path: &str) -> bool {
    let lower_name = filename.to_lowercase();
    let lower_path = file_path.to_lowercase();

    // Exclude documentation files by extension
    let doc_extensions = [
        ".md", ".txt", ".rst", ".html", ".htm", ".pdf", ".doc", ".docx", ".1", ".2", ".3", ".4",
        ".5", ".6", ".7", ".8", // man pages
    ];
    if doc_extensions.iter().any(|ext| lower_name.ends_with(ext)) {
        return true;
    }

    // Exclude license/readme files by name pattern
    let excluded_names = [
        "license",
        "licence",
        "copying",
        "unlicense",
        "notice",
        "readme",
        "changelog",
        "changes",
        "history",
        "authors",
        "contributors",
        "credits",
        "thanks",
        "todo",
        "news",
    ];
    if excluded_names.iter().any(|name| lower_name.contains(name)) {
        return true;
    }

    // Exclude config/data files by extension
    let config_extensions = [
        ".yml", ".yaml", ".toml", ".json", ".xml", ".ini", ".cfg", ".conf",
    ];
    if config_extensions
        .iter()
        .any(|ext| lower_name.ends_with(ext))
    {
        return true;
    }

    // Exclude shell completion files (usually in complete/ or completions/ directory)
    let completion_extensions = [".fish", ".bash", ".zsh", ".ps1"];
    let in_completion_dir = lower_path.contains("complete") || lower_path.contains("completion");
    if in_completion_dir
        && completion_extensions
            .iter()
            .any(|ext| lower_name.ends_with(ext))
    {
        return true;
    }

    // Exclude files starting with underscore in completion directories (e.g., _rg for zsh)
    if in_completion_dir && lower_name.starts_with('_') {
        return true;
    }

    false
}

/// Check if a file could be an executable based on its filename
fn could_be_executable(filename: &str, file_path: &str) -> bool {
    let lower_name = filename.to_lowercase();

    if cfg!(windows) {
        // On Windows, must have .exe extension
        lower_name.ends_with(".exe")
    } else {
        // On Unix: check if in bin/ directory OR has no extension in filename
        let in_bin_dir = file_path.contains("bin/");

        // Check if filename has an extension (only check the filename, not the path!)
        let has_extension = filename.contains('.') && !filename.starts_with('.');

        // Executable scripts
        let script_extensions = [".sh"];
        let is_script = script_extensions
            .iter()
            .any(|ext| lower_name.ends_with(ext));

        in_bin_dir || !has_extension || is_script
    }
}

/// Check if a file has executable permission on Unix
#[cfg(unix)]
fn has_executable_permission(file_path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(file_path) {
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(unix))]
#[allow(dead_code)]
fn has_executable_permission(_file_path: &Path) -> bool {
    // On Windows, we rely on .exe extension, not permissions
    true
}

/// Find all possible executables and rank them by priority
/// `extract_dir` is the directory where files were extracted to (used for permission checks)
pub fn find_executable_candidates(
    extracted_files: &[String],
    package_name: &str,
    extract_dir: Option<&Path>,
) -> Vec<ExecutableCandidate> {
    let mut candidates = Vec::new();

    for file in extracted_files {
        let path = Path::new(file);

        // Get just the filename (not the full path)
        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Skip excluded files (docs, licenses, configs, etc.)
        if is_excluded_file(filename, file) {
            continue;
        }

        // Check if this could be an executable
        if !could_be_executable(filename, file) {
            continue;
        }

        // Skip test/debug/benchmark executables
        let lower_file = file.to_lowercase();
        if lower_file.contains("test")
            || lower_file.contains("debug")
            || lower_file.contains("bench")
            || lower_file.contains("example")
        {
            continue;
        }

        let name_without_ext = filename.trim_end_matches(".exe");
        let mut score = 0u32;
        let mut reasons = Vec::new();

        // Check executable permission (Unix only)
        #[cfg(unix)]
        let has_exec_perm = if let Some(dir) = extract_dir {
            let full_path = dir.join(file);
            has_executable_permission(&full_path)
        } else {
            false
        };
        #[cfg(not(unix))]
        let has_exec_perm = false;

        // Rule 0: Has executable permission (Unix) - strong signal
        #[cfg(unix)]
        if has_exec_perm {
            score += 35;
            reasons.push("has exec permission");
        }
        // Suppress unused warning on non-Unix
        #[cfg(not(unix))]
        let _ = extract_dir;

        // Rule 1: Exact match with package name (highest priority)
        if name_without_ext == package_name {
            score += 100;
            reasons.push("exact name match");
        }
        // Rule 2: Partial match or package name contains file name
        else if name_without_ext.contains(package_name) || package_name.contains(name_without_ext)
        {
            score += 50;
            reasons.push("partial name match");
        }
        // Rule 3: Common abbreviation patterns (e.g., ripgrep -> rg)
        else if is_likely_abbreviation(package_name, name_without_ext) {
            score += 40;
            reasons.push("likely abbreviation");
        }

        // Rule 4: Located in bin/ directory
        if file.contains("bin/") {
            score += 30;
            reasons.push("in bin/ directory");
        }

        // Rule 5: Located in target/release/ (Rust projects)
        if file.contains("target/release/") {
            score += 25;
            reasons.push("in target/release/");
        }

        // Rule 6: Shallow directory depth (prefer files closer to root)
        let depth = file.matches('/').count() + file.matches('\\').count();
        if depth <= 1 {
            score += 20;
        } else if depth <= 2 {
            score += 10;
        }

        // Rule 7: Simple filename (fewer special characters)
        if !name_without_ext.contains('-') && !name_without_ext.contains('_') {
            score += 5;
            reasons.push("simple name");
        }

        // Only add if score is above threshold (or has exec permission on Unix)
        // Note: has_exec_perm is always false on non-Unix, so this works cross-platform
        let should_add = score > 0 || has_exec_perm;

        if should_add {
            let reason = if reasons.is_empty() {
                "potential executable".to_string()
            } else {
                reasons.join(", ")
            };

            candidates.push(ExecutableCandidate {
                path: file.clone(),
                score,
                reason,
            });
        }
    }

    // Sort by score (highest first)
    candidates.sort_by(|a, b| b.score.cmp(&a.score));

    candidates
}

/// Check if name2 is likely an abbreviation of name1
fn is_likely_abbreviation(full_name: &str, abbrev: &str) -> bool {
    // Simple heuristic: check if abbrev matches first letters of words in full_name
    if abbrev.len() < 2 || abbrev.len() > full_name.len() {
        return false;
    }

    // Extract first letters of each word/segment
    let segments: Vec<&str> = full_name.split(&['-', '_'][..]).collect();
    if segments.len() > 1 {
        let first_letters: String = segments.iter().filter_map(|s| s.chars().next()).collect();

        if first_letters.to_lowercase() == abbrev.to_lowercase() {
            return true;
        }
    }

    // Check if abbrev is first N chars of full_name
    full_name.to_lowercase().starts_with(&abbrev.to_lowercase())
}

/// Find the main executable in extracted files
/// Returns the best candidate if found
pub fn find_executable(extracted_files: &[String], package_name: &str) -> Option<String> {
    let candidates = find_executable_candidates(extracted_files, package_name, None);
    candidates.first().map(|c| c.path.clone())
}

/// Normalize a command name by removing platform-specific suffixes
///
/// Strategy: Check if filename contains platform keywords. If yes, remove everything
/// from the first `-` or `_`. Finally, always remove `.exe` extension.
///
/// Examples:
///   "cate-windows-x86_64.exe" -> "cate"
///   "bat-v0.24-x86_64.exe" -> "bat"
///   "git-lfs.exe" -> "git-lfs"
///   "ripgrep.exe" -> "ripgrep"
///   "tool-linux-aarch64" -> "tool"
pub fn normalize_command_name(name: &str) -> String {
    // Platform keywords to detect platform-specific suffixes
    let platform_keywords = [
        "windows", "linux", "darwin", "macos", "freebsd", "netbsd", "openbsd", "x86_64", "aarch64",
        "arm64", "armv7", "i686", "x64", "x86", "pc", "unknown", "gnu", "musl", "msvc",
    ];

    // Check if filename contains any platform keywords (case-insensitive)
    let lower_name = name.to_lowercase();
    let has_platform_suffix = platform_keywords.iter().any(|kw| lower_name.contains(kw));

    let result = if has_platform_suffix {
        // Find first `-` or `_` and remove everything from there
        if let Some(pos) = name.find(['-', '_']) {
            &name[..pos]
        } else {
            name
        }
    } else {
        // No platform keywords, keep original name
        name
    };

    // Always remove .exe extension at the end
    result.trim_end_matches(".exe").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_executable() {
        let files = vec![
            "ripgrep-15.1.0/README.md".to_string(),
            "ripgrep-15.1.0/bin/rg.exe".to_string(),
            "ripgrep-15.1.0/doc/guide.md".to_string(),
        ];

        let exe = find_executable(&files, "rg");
        assert_eq!(exe, Some("ripgrep-15.1.0/bin/rg.exe".to_string()));
    }

    #[test]
    #[cfg(unix)]
    fn test_find_executable_ripgrep_linux() {
        // This is the actual file list from ripgrep Linux release
        let files = vec![
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/doc/CHANGELOG.md".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/doc/FAQ.md".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/doc/GUIDE.md".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/doc/rg.1".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/UNLICENSE".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/README.md".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/LICENSE-MIT".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/COPYING".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/complete/_rg.ps1".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/complete/_rg".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/complete/rg.fish".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/complete/rg.bash".to_string(),
            "ripgrep-15.1.0-aarch64-unknown-linux-gnu/rg".to_string(),
        ];

        let exe = find_executable(&files, "ripgrep");
        // Should find 'rg' even though package name is 'ripgrep'
        // rg is an abbreviation of ripgrep (r + g from rip-grep)
        assert_eq!(
            exe,
            Some("ripgrep-15.1.0-aarch64-unknown-linux-gnu/rg".to_string())
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_find_executable_ripgrep_windows() {
        // Windows version with .exe extension
        let files = vec![
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/doc/CHANGELOG.md".to_string(),
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/doc/FAQ.md".to_string(),
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/UNLICENSE".to_string(),
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/README.md".to_string(),
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/LICENSE-MIT".to_string(),
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/COPYING".to_string(),
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/complete/_rg.ps1".to_string(),
            "ripgrep-15.1.0-x86_64-pc-windows-msvc/rg.exe".to_string(),
        ];

        let exe = find_executable(&files, "ripgrep");
        // Should find 'rg.exe' even though package name is 'ripgrep'
        assert_eq!(
            exe,
            Some("ripgrep-15.1.0-x86_64-pc-windows-msvc/rg.exe".to_string())
        );
    }

    #[test]
    fn test_is_excluded_file() {
        // Documentation files
        assert!(is_excluded_file("README.md", "foo/README.md"));
        assert!(is_excluded_file("CHANGELOG.md", "doc/CHANGELOG.md"));
        assert!(is_excluded_file("rg.1", "doc/rg.1")); // man page

        // License files
        assert!(is_excluded_file("LICENSE", "foo/LICENSE"));
        assert!(is_excluded_file("LICENSE-MIT", "foo/LICENSE-MIT"));
        assert!(is_excluded_file("COPYING", "foo/COPYING"));
        assert!(is_excluded_file("UNLICENSE", "foo/UNLICENSE"));

        // Completion files in completion directory
        assert!(is_excluded_file("rg.fish", "complete/rg.fish"));
        assert!(is_excluded_file("rg.bash", "completions/rg.bash"));
        assert!(is_excluded_file("_rg", "complete/_rg")); // zsh completion
        assert!(is_excluded_file("_rg.ps1", "complete/_rg.ps1"));

        // NOT excluded: regular executables
        assert!(!is_excluded_file("rg", "foo/rg"));
        assert!(!is_excluded_file("ripgrep", "foo/ripgrep"));
        assert!(!is_excluded_file("tool.sh", "bin/tool.sh"));
    }

    #[test]
    fn test_could_be_executable() {
        // Windows
        #[cfg(windows)]
        {
            assert!(could_be_executable("tool.exe", "foo/tool.exe"));
            assert!(!could_be_executable("tool", "foo/tool"));
        }

        // Unix
        #[cfg(unix)]
        {
            // No extension = could be executable
            assert!(could_be_executable("rg", "foo/rg"));
            assert!(could_be_executable("ripgrep", "foo/ripgrep"));

            // In bin/ = could be executable
            assert!(could_be_executable("tool", "bin/tool"));

            // Script = could be executable
            assert!(could_be_executable("run.sh", "foo/run.sh"));

            // Has extension = not executable (unless script)
            assert!(!could_be_executable("rg.fish", "foo/rg.fish"));
            assert!(!could_be_executable("config.toml", "foo/config.toml"));
        }
    }

    #[test]
    fn test_normalize_command_name() {
        // Files with platform suffixes - should remove from first - or _
        assert_eq!(normalize_command_name("cate-windows-x86_64.exe"), "cate");
        assert_eq!(normalize_command_name("bat-v0.24-x86_64.exe"), "bat");
        assert_eq!(normalize_command_name("tool-linux-aarch64"), "tool");
        assert_eq!(normalize_command_name("app-darwin-x86_64.exe"), "app");
        assert_eq!(
            normalize_command_name("ripgrep-13.0.0-x86_64-pc-windows-msvc.exe"),
            "ripgrep"
        );
        assert_eq!(normalize_command_name("fd_v8.7.0_x86_64.exe"), "fd");

        // Files without platform suffixes - should keep name but remove .exe
        assert_eq!(normalize_command_name("ripgrep.exe"), "ripgrep");
        assert_eq!(normalize_command_name("git-lfs.exe"), "git-lfs");
        assert_eq!(normalize_command_name("gh-cli.exe"), "gh-cli");
        assert_eq!(normalize_command_name("node-sass.exe"), "node-sass");

        // Unix executables without .exe
        assert_eq!(normalize_command_name("ripgrep"), "ripgrep");
        assert_eq!(normalize_command_name("git-lfs"), "git-lfs");

        // Edge cases
        assert_eq!(normalize_command_name("tool.exe"), "tool");
        assert_eq!(normalize_command_name("tool"), "tool");
    }
}
