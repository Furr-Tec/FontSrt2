use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use crate::error::Result;
use crate::models::{Config, FontMetadata};
use crate::font::metadata::{extract_font_metadata, extract_root_family};
use crate::utils::{
    ensure_directory_exists,
    safe_move_file,
    clean_name,
    log,
    format_font_name,
    normalize_family_name,
};

// The utility functions for formatting font names and building paths
// have been moved to utils::naming module for better organization


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

    // Group fonts by normalized family name
    let metadata_map = font_metadata_map.lock().unwrap().clone();
    let metadata_count = metadata_map.len();
    
    // Create a map of normalized family names to lists of (path, metadata) pairs
    let mut family_groups: HashMap<String, Vec<(PathBuf, FontMetadata)>> = HashMap::new();
    
    for (path, metadata) in &metadata_map {
        // Use normalized family name as the grouping key
        let root_family = extract_root_family(&metadata.family_name);
        let normalized_root_family = normalize_family_name(&root_family);

        family_groups
            .entry(normalized_root_family)
            .or_insert_with(Vec::new)
            .push((path.clone(), metadata.clone()));
    }
    
    log(config, format!("Grouped fonts into {} families", family_groups.len()));

    // Process each family group
    for (family_name, font_group) in family_groups {
        if font_group.is_empty() {
            continue;
        }
        
        log(config, format!("Processing family group: {} with {} fonts", family_name, font_group.len()));
        
        // Create a directory specifically for this normalized family name
        // Don't rely on build_folder_path which might use the original family name
        let family_dir = if config.group_by_foundry {
            // If grouping by foundry is enabled, create a foundry/family structure
            let first_font = &font_group[0];
            let foundry_name = clean_name(&first_font.1.foundry);
            // Handle potential empty foundry name
            let foundry_dir = if foundry_name.is_empty() {
                dir.join("Unknown_Foundry")
            } else {
                dir.join(foundry_name)
            };
            
            if let Err(e) = ensure_directory_exists(&foundry_dir, config) {
                log(config, format!("Error creating foundry directory {}: {}", foundry_dir.display(), e));
                // Fall back to base directory if foundry directory creation fails
                dir.join(clean_name(&family_name))
            } else {
                foundry_dir.join(clean_name(&family_name))
            }
        } else {
            // Otherwise, use the normalized family name directly
            dir.join(clean_name(&family_name))
        };
        
        // Create the directory once per family
        if let Err(e) = ensure_directory_exists(&family_dir, config) {
            log(config, format!("Error creating family directory {}: {}", family_dir.display(), e));
            // Skip this family group if we can't create the directory
            continue;
        }
        
        log(config, format!("Created directory for family {}: {}", family_name, family_dir.display()));
        
        // Store folder reference for potential foundry grouping later
        if config.group_by_foundry {
            let clean_family = clean_name(&family_name);
            family_folders.lock().unwrap().insert(clean_family.clone(), family_dir.clone());
            
            let first_font = &font_group[0];
            let clean_foundry = clean_name(&first_font.1.foundry);
            let parent_dir = family_dir.parent().unwrap_or(dir).to_path_buf();
            foundry_folders.lock().unwrap().insert(clean_foundry.clone(), parent_dir.clone());
                
            log(config, format!("Registered family folder: {} -> {}", clean_family, family_dir.display()));
            log(config, format!("Registered foundry folder: {} -> {}", clean_foundry, parent_dir.display()));
        }
        
        // Process each font in the family
        for (path, metadata) in font_group {
            let mut processed_set = processed_files.lock().unwrap();
            
            if processed_set.contains(&path) {
                continue;
            }
            
            processed_set.insert(path.clone());
            
            // Format new filename based on naming pattern
            let base_name = format_font_name(&metadata, &config.naming_pattern);
            let clean_base_name = clean_name(&base_name);
            
            // Get file extension
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("ttf")
                .to_lowercase();
            
            // Create new filename
            let new_filename = format!("{}.{}", clean_base_name, extension);
            let new_path = family_dir.join(&new_filename);
            
            // Verify the target directory is correct for this font
            let normalized_font_family = normalize_family_name(&metadata.family_name);
            let expected_dir_name = clean_name(&normalized_font_family);
            let actual_dir_name = family_dir.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
                
            if expected_dir_name != actual_dir_name && !config.group_by_foundry {
                log(
                    config,
                    format!(
                        "WARNING: Font family mismatch - {} should go to {} but is being placed in {}",
                        path.display(),
                        expected_dir_name,
                        actual_dir_name
                    ),
                );
            }
            
            // Handle file move or duplicate
            if new_path.exists() {
                let mut unique_name = new_filename.clone();
                let mut counter = 1;
                
                // Use family_dir instead of target_dir (which no longer exists)
                while family_dir.join(&unique_name).exists() {
                    let name_part = clean_base_name.clone();
                    unique_name = format!("{}_{}.{}", name_part, counter, extension);
                    counter += 1;
                }
                
                let final_path = family_dir.join(unique_name);
                
                log(
                    config,
                    format!(
                        "Font with same name exists. Renaming {} to {}",
                        path.display(),
                        final_path.display()
                    ),
                );
                
                if let Err(e) = safe_move_file(&path, &final_path, config) {
                    log(
                        config,
                        format!("Error moving file {}: {}", path.display(), e),
                    );
                } else {
                    log(
                        config,
                        format!("Successfully moved {} to {}", path.display(), final_path.display()),
                    );
                }
            } else {
                log(
                    config,
                    format!("Moving {} to {}", path.display(), new_path.display()),
                );
                
                if let Err(e) = safe_move_file(&path, &new_path, config) {
                    log(
                        config,
                        format!("Error moving file {}: {}", path.display(), e),
                    );
                } else {
                    log(
                        config,
                        format!("Successfully moved {} to {}", path.display(), new_path.display()),
                    );
                }
            }
        }
    }

    // Report statistics
    println!("Font organization summary:");
    println!("  - {} fonts processed", metadata_count);

    Ok(())
}

