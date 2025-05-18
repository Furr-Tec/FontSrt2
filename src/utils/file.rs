use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{Result, Error};
use crate::models::Config;
use crate::utils::logging::log;

/// Create a directory if it doesn't exist
pub fn ensure_directory_exists(dir: &Path, config: &Config) -> Result<()> {
    if !dir.exists() {
        log(
            config,
            format!("Directory {} does not exist. Creating it now.", dir.display()),
        );
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Safely move a file with fallback to copy+delete if rename fails
pub fn safe_move_file(src: &Path, dest: &Path, config: &Config) -> Result<()> {
    // First try to rename (fast path)
    match fs::rename(src, dest) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If rename fails, log it and try copy+delete
            log(
                config,
                format!("Rename failed for {}, trying copy+delete: {}", src.display(), e),
            );

            // Copy the file
            fs::copy(src, dest)?;

            // Delete the original
            match fs::remove_file(src) {
                Ok(_) => Ok(()),
                Err(e) => {
                    log(
                        config,
                        format!("Warning: Could not delete source file {} after copying: {}", src.display(), e),
                    );
                    // We still consider this a success since the file was copied
                    Ok(())
                }
            }
        }
    }
}

/// Safely move a directory with fallback to recursive copy+delete if rename fails
pub fn safe_move_directory(src_dir: &Path, dest_dir: &Path, config: &Config) -> Result<()> {
    // First try to rename (fast path)
    match fs::rename(src_dir, dest_dir) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If rename fails, log it and try recursive copy+delete
            log(
                config,
                format!("Rename failed for directory {}, trying recursive copy: {}", src_dir.display(), e),
            );

            // Make sure destination directory exists
            ensure_directory_exists(dest_dir, config)?;

            // Copy all contents recursively
            merge_directories(src_dir, dest_dir, config)?;

            // Try to remove the source directory
            match fs::remove_dir_all(src_dir) {
                Ok(_) => Ok(()),
                Err(e) => {
                    log(
                        config,
                        format!("Warning: Could not delete source directory {} after copying: {}", src_dir.display(), e),
                    );
                    // We still consider this a success since the contents were copied
                    Ok(())
                }
            }
        }
    }
}

/// Merge the contents of two directories
pub fn merge_directories(src_dir: &Path, dest_dir: &Path, config: &Config) -> Result<()> {
    let entries = fs::read_dir(src_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default();
        let dest_path = dest_dir.join(file_name);

        if path.is_file() {
            // If destination file already exists, create a unique name
            if dest_path.exists() {
                let mut counter = 1;
                let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
                let extension = path.extension().unwrap_or_default().to_string_lossy();

                let mut unique_path = dest_dir.join(format!("{}_{}.{}", file_stem, counter, extension));

                while unique_path.exists() {
                    counter += 1;
                    unique_path = dest_dir.join(format!("{}_{}.{}", file_stem, counter, extension));
                }

                if let Err(e) = safe_move_file(&path, &unique_path, config) {
                    log(
                        config,
                        format!("Error moving file {}: {}", path.display(), e),
                    );
                }
            } else {
                // Move the file
                if let Err(e) = safe_move_file(&path, &dest_path, config) {
                    log(
                        config,
                        format!("Error moving file {}: {}", path.display(), e),
                    );
                }
            }
        } else if path.is_dir() {
            // Create destination directory if it doesn't exist
            ensure_directory_exists(&dest_path, config)?;

            // Recursively merge subdirectories
            merge_directories(&path, &dest_path, config)?;

            // Remove the now-empty source directory
            if let Err(e) = fs::remove_dir(&path) {
                log(
                    config,
                    format!("Error removing directory {}: {}", path.display(), e),
                );
            }
        }
    }

    Ok(())
}

