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
fn extract_standalone_executable(
    executable_path: &Path,
    dest_dir: &Path,
) -> Result<Vec<String>> {
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

    for entry_result in archive.entries().context("Failed to read archive entries")? {
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
    use std::os::unix::fs::PermissionsExt;
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

/// Find the main executable in extracted files
pub fn find_executable(extracted_files: &[String], package_name: &str) -> Option<String> {
    // First, try to find a file with the package name
    for file in extracted_files {
        let path = Path::new(file);
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            // Remove .exe extension for comparison
            let name_without_ext = filename.trim_end_matches(".exe");

            if name_without_ext == package_name {
                return Some(file.clone());
            }
        }
    }

    // If not found, try to find any executable in bin/ directory
    for file in extracted_files {
        if file.contains("bin/") && (file.ends_with(".exe") || !file.contains('.')) {
            return Some(file.clone());
        }
    }

    // If still not found, return the first .exe file (Windows)
    #[cfg(windows)]
    {
        for file in extracted_files {
            if file.ends_with(".exe") {
                return Some(file.clone());
            }
        }
    }

    None
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
}
