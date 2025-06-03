/// Clean a name for use in filenames
pub fn clean_name(name: &str) -> String {
    // Replace invalid filename characters with underscores
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut cleaned = name.to_string();

    for c in invalid_chars {
        cleaned = cleaned.replace(c, "_");
    }

    // Remove leading/trailing spaces and dots
    cleaned = cleaned.trim().trim_matches('.').to_string();

    // Ensure the name is not empty
    if cleaned.is_empty() {
        cleaned = "Unknown".to_string();
    }

    cleaned
}

/// Capitalize the first letter of each word in a string
#[allow(dead_code)]
pub fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Normalize a font family name for more precise folder creation
pub fn normalize_family_name(family_name: &str) -> String {
    // 1. Clean and early exit for empty input
    let normalized = family_name.trim();
    if normalized.is_empty() {
        return String::from("Unknown");
    }

    // 2. Split on spaces and camel case for better tokenization
    let camel_case_re = regex::Regex::new(r"([a-z])([A-Z])").unwrap();
    let separated = camel_case_re.replace_all(normalized, "$1 $2");
    let tokens: Vec<&str> = separated.split_whitespace().collect();

    // 3. For empty token list, return "Unknown"
    if tokens.is_empty() {
        return String::from("Unknown");
    }

    // 4. Use the full name for more accurate grouping
    // This prevents unrelated fonts from being grouped together
    // For example, "Hybrea", "Hybrid", "Hygge Sans" will be in separate folders
    let result = normalized.to_string();
    if result.is_empty() {
        return String::from("Unknown");
    }

    result
}

use crate::models::{FontMetadata, NamingPattern, Config};
use std::path::{Path, PathBuf};

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
                if metadata.is_italic { " Italic " } else { "" }
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
#[allow(dead_code)]
pub fn build_folder_path(base_dir: &Path, metadata: &FontMetadata, config: &Config) -> PathBuf {
    // Normalize the family name first to ensure proper grouping
    let normalized_family = normalize_family_name(&metadata.family_name);

    match config.naming_pattern {
        NamingPattern::FoundryFamily => {
            // Create a foundry/family structure
            let foundry_dir = base_dir.join(clean_name(&metadata.foundry));
            foundry_dir.join(clean_name(&normalized_family))
        },
        _ => {
            if config.group_by_foundry {
                // If grouping by foundry is enabled, create a foundry/family structure
                let foundry_dir = base_dir.join(clean_name(&metadata.foundry));
                foundry_dir.join(clean_name(&normalized_family))
            } else {
                // For all other patterns, just use normalized family name as the directory
                base_dir.join(clean_name(&normalized_family))
            }
        }
    }
}
