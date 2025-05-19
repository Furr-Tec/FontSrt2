use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::fs;
use crate::error::Result;
use crate::models::Config;
use crate::font::metadata::extract_font_metadata;
use crate::utils::{
    ensure_directory_exists,
    safe_move_directory,
    clean_name,
    log,
};

/// Group font families by their foundry
pub fn group_by_foundry(
    dir: &Path,
    config: &Config,
    _processed_files: Arc<Mutex<HashSet<PathBuf>>>,
    family_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
    foundry_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
) -> Result<()> {
    // Create a map to track which family belongs to which foundry
    let mut family_to_foundry: HashMap<String, String> = HashMap::new();

    // First, scan the directory for font files to determine foundry for each family
    for entry in fs::read_dir(dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();

            // Only process directories (font family folders)
            if path.is_dir() && path.file_name().unwrap_or_default() != "duplicates" {
                let family_name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();

                // Scan this family folder for font files to determine foundry
                if let Ok(dir_entries) = fs::read_dir(&path) {
                    for file_entry in dir_entries {
                        if let Ok(file_entry) = file_entry {
                            let file_path = file_entry.path();
                            if file_path.is_file() {
                                if let Ok(Some(metadata)) = extract_font_metadata(&file_path, config) {
                                    family_to_foundry.insert(family_name.clone(), clean_name(&metadata.foundry));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Now move each family folder to its foundry folder
    for (family, foundry) in family_to_foundry {
        let family_dir = dir.join(&family);
        let foundry_dir = dir.join(&foundry);

        // Create foundry directory if it doesn't exist
        ensure_directory_exists(&foundry_dir, config)?;

        // Move family folder to foundry folder
        let target_dir = foundry_dir.join(&family);

        if target_dir.exists() {
            // If target directory already exists, merge contents
            log(
                config,
                format!(
                    "Target directory {} exists, merging contents",
                    target_dir.display()
                ),
            );
            safe_move_directory(&family_dir, &target_dir, config)?;
        } else {
            // Move the entire family directory to the foundry directory
            log(
                config,
                format!(
                    "Moving {} to {}",
                    family_dir.display(),
                    target_dir.display()
                ),
            );
            safe_move_directory(&family_dir, &target_dir, config)?;
        }

        // Update the tracking maps
        family_folders.lock().unwrap().insert(family.clone(), target_dir.clone());
        foundry_folders.lock().unwrap().entry(foundry.clone())
            .or_insert_with(|| foundry_dir.clone());
    }

    Ok(())
}

