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

/// Determine if two font family names are similar enough to be grouped together
fn are_family_names_similar(name1: &str, name2: &str) -> bool {
    // If either name is empty, they're not similar
    if name1.is_empty() || name2.is_empty() {
        return false;
    }

    // If the names are identical, they're similar
    if name1 == name2 {
        return true;
    }

    // Normalize both names for comparison
    let norm1 = name1.to_lowercase().replace("_", " ").trim().to_string();
    let norm2 = name2.to_lowercase().replace("_", " ").trim().to_string();

    // If normalized names are identical, they're similar
    if norm1 == norm2 {
        return true;
    }

    // Get the first characters of each name
    let first_char1 = norm1.chars().next();
    let first_char2 = norm2.chars().next();

    // If the first characters are different, the names are not similar
    // This prevents grouping "Hybrea", "Hygge", etc. together
    if first_char1 != first_char2 {
        return false;
    }

    // Check if one name is a prefix of the other (e.g., "Roboto" and "Roboto Slab")
    if norm1.starts_with(&norm2) || norm2.starts_with(&norm1) {
        return true;
    }

    // Split names into words and check if they share the first word
    // This helps with cases like "Breul A" and "Breul B"
    let words1: Vec<&str> = norm1.split_whitespace().collect();
    let words2: Vec<&str> = norm2.split_whitespace().collect();

    if !words1.is_empty() && !words2.is_empty() && words1[0] == words2[0] {
        // If the first words match and they're substantial (not just 1-2 characters)
        // Increased minimum length to 4 characters to be more strict
        if words1[0].len() >= 4 {
            return true;
        }
    }

    // Check if names share a significant common prefix
    let min_len = std::cmp::min(norm1.len(), norm2.len());
    if min_len >= 4 {
        let common_prefix_len = norm1.chars().zip(norm2.chars())
            .take_while(|(c1, c2)| c1 == c2)
            .count();

        // If the common prefix is at least 70% of the shorter name, consider them similar
        // Increased from 50% to 70% to be more strict
        if common_prefix_len >= (min_len * 7) / 10 {
            return true;
        }
    }

    // Check for Levenshtein distance (edit distance)
    // For short names, allow 1 edit; for longer names, allow more edits proportionally
    // Reduced max_distance to be more strict
    let max_distance = std::cmp::max(1, min_len / 5);
    let distance = levenshtein_distance(&norm1, &norm2);

    distance <= max_distance
}

/// Calculate the Levenshtein distance (edit distance) between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let m = s1_chars.len();
    let n = s2_chars.len();

    // Handle empty strings
    if m == 0 { return n; }
    if n == 0 { return m; }

    // Create a matrix of size (m+1) x (n+1)
    let mut matrix = vec![vec![0; n + 1]; m + 1];

    // Initialize the first row and column
    for i in 0..=m {
        matrix[i][0] = i;
    }
    for j in 0..=n {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=m {
        for j in 1..=n {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1       // insertion
                ),
                matrix[i - 1][j - 1] + cost    // substitution
            );
        }
    }

    matrix[m][n]
}

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

    log(config, format!("Initially grouped fonts into {} families", family_groups.len()));

    // Quality of life improvement: Group similar families together to reduce folder count
    // Merge both single-font families and smaller multi-font families into larger similar families
    let mut merged_family_groups: HashMap<String, Vec<(PathBuf, FontMetadata)>> = HashMap::new();
    let mut all_families: Vec<(String, Vec<(PathBuf, FontMetadata)>)> = family_groups.into_iter().collect();

    // Store the original number of families for logging
    let original_family_count = all_families.len();

    // Sort families by size (descending) to prefer merging into larger groups
    all_families.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    // Create a map to track which families have been merged
    let mut merged_families: HashSet<String> = HashSet::new();

    // First pass: identify primary families (largest in each similar group)
    let mut primary_families: Vec<(String, Vec<(PathBuf, FontMetadata)>)> = Vec::new();

    for (i, (family_name, fonts)) in all_families.iter().enumerate() {
        // Skip if this family has already been merged
        if merged_families.contains(family_name) {
            continue;
        }

        // This becomes a primary family
        primary_families.push((family_name.clone(), fonts.clone()));

        // Find all similar families and mark them as merged
        for (j, (other_family, _)) in all_families.iter().enumerate() {
            if i != j && !merged_families.contains(other_family) {
                if are_family_names_similar(family_name, other_family) {
                    merged_families.insert(other_family.clone());
                }
            }
        }
    }

    // Second pass: merge similar families into their primary families
    for (primary_name, primary_fonts) in primary_families {
        let mut all_fonts = primary_fonts;

        // Find all families that should be merged into this primary family
        for (other_name, other_fonts) in &all_families {
            if other_name != &primary_name && are_family_names_similar(&primary_name, other_name) {
                all_fonts.extend(other_fonts.clone());

                log(config, format!(
                    "Merged family '{}' into similar family '{}'",
                    other_name, primary_name
                ));
            }
        }

        // Add the merged family to the result
        merged_family_groups.insert(primary_name, all_fonts);
    }

    // Add any families that weren't merged
    for (family_name, fonts) in &all_families {
        if !merged_families.contains(family_name) && !merged_family_groups.contains_key(family_name) {
            merged_family_groups.insert(family_name.clone(), fonts.clone());
        }
    }

    log(config, format!(
        "After merging similar families: {} families (reduced from {})",
        merged_family_groups.len(), original_family_count
    ));

    // Use the merged family groups for further processing
    family_groups = merged_family_groups;

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
