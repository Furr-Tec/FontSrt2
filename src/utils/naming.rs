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

/// Normalize a font family name by removing common variations
pub fn normalize_family_name(family_name: &str) -> String {
    // Special case handling for known problematic patterns
    let mut normalized = family_name.to_string();
    
    // Special handling for "Variable" fonts - they should group with the base family
    if normalized.ends_with(" Variable") {
        return normalized[0..normalized.len() - 9].trim().to_string();
    }
    
    // Do not normalize these terms as they should stay as different families
    let preserve_terms = [
        " Alt", // Preserve "Alt" variants as separate families
        " CF", // Common alternate family indicator
        " NF", // NerdFont variant
        " SC", // Small Caps as separate family
        " Text", // Text variant as separate family
        " Display", // Display variant as separate family
        " Sans", // Sans variant
        " Serif", // Serif variant
        " Mono", // Mono variant
    ];
    
    // Check if name contains any preserved terms before normalizing
    for term in &preserve_terms {
        if normalized.contains(term) {
            // If it contains a preserved term, we need to handle separately
            return normalized;
        }
    }
    
    // Step 1: Handle numbered variants (e.g., "Eina 01" -> "Eina")
    if let Some(idx) = normalized.find(|c: char| c.is_digit(10)) {
        let prefix = &normalized[0..idx].trim();
        if !prefix.is_empty() {
            // Check if we should extract just the base name
            let next_char = normalized.chars().nth(idx + 1);
            if next_char.map_or(false, |c| c.is_digit(10) || c.is_whitespace()) {
                normalized = prefix.to_string();
            }
        }
    }
    
    // Step 2: Handle weight and style variations in family names
    let style_indicators = [
        " Bold", " Italic", " Black", " Light", " Medium", " Thin", " Regular", 
        " Semibold", " SemiBold", " DemiBold", " ExtraBold", " UltraBold", " ExtraLight", " UltraLight",
        " Condensed", " Expanded", " Extended", " Narrow", " Wide", " Compressed",
        " Oblique", " Slant", " Slanted", " BoldItalic", " LightItalic", " BlackItalic", " MediumItalic",
        " ThinItalic", " SemiBoldItalic", " SemiLightItalic", " Variable", " Hairline", " Book",
        " Heavy", " Ultra", " Super", " Poster", " Title"
    ];
    
    for indicator in &style_indicators {
        if normalized.ends_with(indicator) {
            normalized = normalized[0..normalized.len() - indicator.len()].trim().to_string();
        }
    }
    
    // Step 3: Handle common variant indicators in the middle of names
    let variant_indicators = [" Variable ", " Var ", " Pro ", " Std ", " Bk ", " Lt "];
    for indicator in &variant_indicators {
        if normalized.contains(indicator) {
            normalized = normalized.replace(indicator, " ");
        }
    }
    
    // Step 4: Handle suffixes
    let suffixes = [" Pro", " Std", " LT", " MT", " MS", " Bk", " Lt", " Md", " Bd", " Rg"];
    for suffix in &suffixes {
        if normalized.ends_with(suffix) {
            normalized = normalized[0..normalized.len() - suffix.len()].trim().to_string();
        }
    }
    
    // Step 5: Special case handling for specific patterns we've identified
    // Group "Einer Grotesk" and "Einer Grotesk Hairline" together
    if normalized.starts_with("Einer Grotesk") {
        normalized = "Einer Grotesk".to_string();
    }
    
    // Step 6: Cleanup any remaining whitespace
    normalized = normalized.trim().to_string();
    
    // Step 7: Ensure we didn't strip everything away
    if normalized.is_empty() {
        // Fallback to first word
        normalized = family_name
            .split_whitespace()
            .next()
            .unwrap_or(family_name)
            .to_string();
    }
    
    normalized
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

