use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use crate::error::{Result, Error};
use crate::models::{Config, FontMetadata, NamingPattern};
use crate::font::metadata::extract_font_metadata;
use crate::utils::{
    ensure_directory_exists,
    safe_move_file,
    clean_name,
    log,
};

/// Format a font name based on naming pattern
pub fn format_font_name(metadata: &FontMetadata, pattern: &NamingPattern) -> String {
    use NamingPattern::*;
    
    match pattern {
        FamilySubfamily => {
            if metadata.subfamily.to_lowercase() == "regular" {
                metadata.family_name.clone()
            } else {
                format!("{} ({})", metadata.family_name, metadata.subfamily)
            }
        },
        FoundryFamilySubfamily => {
            if metadata.subfamily.to_lowercase() == "regular" {
                format!("{} {}", metadata.foundry, metadata.family_name)
            } else {
                format!("{} {} ({})", metadata.foundry, metadata.family_name, metadata.subfamily)
            }
        },
        FamilyWeight => {
            format!("{} {}{}", 
                metadata.family_name, 
                metadata.weight,
                if metadata.is_italic { " Italic" } else { "" }
            )
        },
        FoundryFamily => {
            if metadata.subfamily.to_lowercase() == "regular" {
                format!("{}_{}", metadata.foundry, metadata.family_name)
            } else {
                format!("{}_{} ({})", metadata.foundry, metadata.family_name, metadata.subfamily)
            }
        },
    }
}

/// Generate a filename for a font based on its metadata
pub fn generate_font_filename(metadata: &FontMetadata, pattern: &NamingPattern) -> String {
    let base_name = format_font_name(metadata, pattern);
    let extension = metadata.original_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("ttf")
        .to_lowercase();

    format!("{}.{}", clean_name(&base_name), extension)
}

/// Build the target folder path for a font
fn build_folder_path(base_dir: &Path, metadata: &FontMetadata, config: &Config) -> PathBuf {
    match config.naming_pattern {
        NamingPattern::FoundryFamily => {
            // Create a foundry/family structure
            let foundry_dir = base_dir.join(clean_name(&metadata.foundry));
            foundry_dir.join(clean_name(&metadata.family_name))
        },
        _ => {
            if config.group_by_foundry {
                // If grouping by foundry is enabled, create a foundry/family structure
                let foundry_dir = base_dir.join(clean_name(&metadata.foundry));
                foundry_dir.join(clean_name(&metadata.family_name))
            } else {
                // For all other patterns, just use family name as the directory
                base_dir.join(clean_name(&metadata.family_name))
            }
        }
    }
}

/// Organize fonts in a directory
pub fn organize_fonts(
    dir: &Path,
    config: &Config,
    processed_files: Arc<Mutex<HashSet<PathBuf>>>,
    family_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
    foundry_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
) -> Result<()> {
    let duplicates_dir = dir.join("duplicates");
    ensure_directory_exists(&duplicates_dir, config)?;

    // Collect metadata for all fonts first to help with duplicate detection
    let font_metadata_map: Arc<Mutex<HashMap<PathBuf, FontMetadata>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Map all fonts by their signatures for duplication detection
    let font_signatures: Arc<Mutex<HashMap<String, Vec<PathBuf>>>> = Arc::new(Mutex::new(HashMap::new()));

    // First pass: collect metadata
    fs::read_dir(dir)?
        .par_bridge()
        .for_each(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Skip directories or processed files
                if !path.is_file() || processed_files.lock().unwrap().contains(&path) {
                    return;
                }

                if let Ok(Some(metadata)) = extract_font_metadata(&path, config) {
                    // Add to metadata map
                    font_metadata_map.lock().unwrap().insert(path.clone(), metadata.clone());

                    // Add to signatures for duplicate detection
                    let signature = format!("{}_{}_{}",
                        metadata.family_name,
                        metadata.weight,
                        metadata.is_italic
                    );

                    font_signatures.lock().unwrap()
                        .entry(signature)
                        .or_insert_with(Vec::new)
                        .push(path.clone());
                }
            }
        });

    log(config, format!("Collected metadata for {} fonts", 
        font_metadata_map.lock().unwrap().len()));

    // Create directory structure and move files
    let metadata_map = font_metadata_map.lock().unwrap().clone();
    let metadata_count = metadata_map.len();

    for (path, metadata) in &metadata_map {
        let mut processed_set = processed_files.lock().unwrap();

        if processed_set.contains(path) {
            continue;
        }

        processed_set.insert(path.clone());

        // Determine target directory based on naming pattern
        let target_dir = build_folder_path(dir, metadata, config);
        ensure_directory_exists(&target_dir, config)?;

        // Format new filename based on naming pattern
        let base_name = format_font_name(metadata, &config.naming_pattern);
        let clean_base_name = clean_name(&base_name);

        // Get file extension
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("ttf")
            .to_lowercase();

        // Create new filename
        let new_filename = format!("{}.{}", clean_base_name, extension);
        let new_path = target_dir.join(&new_filename);

        // Handle file move or duplicate
        if new_path.exists() {
            let mut unique_name = new_filename.clone();
            let mut counter = 1;

            while target_dir.join(&unique_name).exists() {
                let name_part = clean_base_name.clone();
                unique_name = format!("{}_{}.{}", name_part, counter, extension);
                counter += 1;
            }

            let final_path = target_dir.join(unique_name);

            log(
                config,
                format!(
                    "Font with same name exists. Renaming {} to {}",
                    path.display(),
                    final_path.display()
                ),
            );

            if let Err(e) = safe_move_file(path, &final_path, config) {
                log(
                    config,
                    format!("Error moving file {}: {}", path.display(), e),
                );
            }
        } else {
            log(
                config,
                format!("Moving {} to {}", path.display(), new_path.display()),
            );

            if let Err(e) = safe_move_file(path, &new_path, config) {
                log(
                    config,
                    format!("Error moving file {}: {}", path.display(), e),
                );
            }
        }
    }

    // Report statistics
    println!("Font organization summary:");
    println!("  - {} fonts processed", metadata_count);

    Ok(())
}

