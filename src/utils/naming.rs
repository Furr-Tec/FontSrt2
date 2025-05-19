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
        // Removed " Sans" to fix ElaSans variants normalization
        " Serif", // Serif variant
        " Mono", // Mono variant
    ];
    
    // Also add the embedded preserve terms (without spaces)
    let embedded_preserve_terms = [
        "Alt", "CF", "NF", "SC", "Text", "Display",
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
    
    // Step 1.5: Handle embedded style/weight indicators without spaces (e.g., "ElaSansBlack" -> "ElaSans")
    let embedded_style_indicators = [
        "Bold", "Italic", "Black", "Light", "Medium", "Thin", "Regular", 
        "Semibold", "SemiBold", "DemiBold", "ExtraBold", "UltraBold", "ExtraLight", "UltraLight",
        "Condensed", "Expanded", "Extended", "Narrow", "Wide", "Compressed",
        "Oblique", "Slant", "Slanted", "BoldItalic", "LightItalic", "BlackItalic", "MediumItalic",
        "ThinItalic", "SemiBoldItalic", "SemiLightItalic", "Variable", "Hairline", "Book",
        "Heavy", "Ultra", "Super", "Poster", "Title", "Caps",
        // Add common variations that might appear in concatenated form
        "SansBold", "SansBlack", "SansLight", "SansMedium", "SansThin", "SansRegular",
        "SansItalic", "SansBoldItalic", "SansOblique", "SansCondensed", "SansExtended"
    ];
    
    // Special case for font families like "ElaSans" and their variants
    let mut normalized_name = normalized.clone();
    let mut made_changes = false;
    
    // Note: Sans, Serif, and Mono are typically not standalone style indicators by themselves
    // They're often legitimate parts of family names (like "Roboto Sans") so we don't include them individually
    // but we do handle compound cases like "SansBold", "SerifItalic", etc.
    
    // Special handling for ElaSans-like fonts (where Sans is part of the base name)
    // We want to treat ElaSansBlack as a style variant of ElaSans, not as a separate family
    let common_sans_families = ["ElaSans", "CirceSans", "ModernSans", "NexaSans", "ProximaSans"];
    for family in &common_sans_families {
        if normalized_name.starts_with(family) && normalized_name.len() > family.len() {
            // Check if what follows is a style indicator
            let suffix = &normalized_name[family.len()..];
            for indicator in &embedded_style_indicators {
                if suffix.starts_with(indicator) {
                    normalized_name = family.to_string();
                    made_changes = true;
                    break;
                }
            }
            if made_changes {
                break;
            }
        }
    }
    
    // Loop through potentially multiple passes to handle nested/consecutive style indicators
    loop {
        made_changes = false;
        let mut best_match = None;
        let mut best_match_pos = normalized_name.len();
        
        // Find the earliest embedded style indicator
        for indicator in &embedded_style_indicators {
            // Skip case for embedded preserve terms
            let mut should_skip = false;
            for term in &embedded_preserve_terms {
                if indicator == term {
                    should_skip = true;
                    break;
                }
            }
            if should_skip {
                continue;
            }
            
            // Check if indicator is embedded and not at the start
            if let Some(pos) = normalized_name.find(indicator) {
                // Ensure it's not at the very beginning of the string
                if pos > 0 {
                    // Check if it's truly embedded (not just part of another word)
                    let prefix_char = normalized_name.chars().nth(pos - 1).unwrap_or(' ');
                    
                    // Only consider it a match if it's:
                    // - Not preceded by a space (which would be caught by suffix checks)
                    // - Not in the middle of a legitimate word
                    if prefix_char != ' ' {
                        // Handle special cases like "SansBold" where "Sans" should be preserved
                        let prefix = &normalized_name[0..pos];
                        
                        // Skip if the prefix ends with a preserved term without space
                        // But make exception for known font families where Sans is part of the name
                        let mut is_preserved_prefix = false;
                        
                        // Check if this is a family where "Sans" should be preserved as part of base name
                        let is_sans_family_name = prefix.ends_with("Sans") && 
                            (prefix == "ElaSans" || prefix == "CirceSans" || 
                             prefix == "ModernSans" || prefix == "NexaSans" || 
                             prefix == "ProximaSans" || prefix.contains("Sans"));
                        
                        // Only preserve "Sans" as a separate term if it's not part of a known family name
                        if !is_sans_family_name {
                            for term in &["Serif", "Mono"] {
                                if prefix.ends_with(term) {
                                    is_preserved_prefix = true;
                                    break;
                                }
                            }
                        }
                        
                        // If we're not dealing with a preserved prefix and this match is earlier than our best match
                        if !is_preserved_prefix && pos < best_match_pos {
                            best_match = Some((pos, indicator.len()));
                            best_match_pos = pos;
                        }
                    }
                }
            }
        }
        
        // Apply the best match if found
        if let Some((pos, len)) = best_match {
            // Get the part before the indicator
            let prefix = &normalized_name[0..pos];
            // Get the part after the indicator (if any)
            let suffix = if pos + len < normalized_name.len() {
                &normalized_name[pos + len..]
            } else {
                ""
            };
            
            // Only normalize if the prefix is at least 2 characters (to avoid over-normalization)
            // Lowered from 3 to better handle font families with shorter names
            if prefix.len() >= 2 {
                // When handling consecutive style indicators, we need to keep processing
                normalized_name = format!("{}{}", prefix, suffix);
                made_changes = true;
            } else {
                // If prefix is too short, don't normalize and exit the loop
                made_changes = false;
            }
        } else {
            // No match found, exit the loop
            made_changes = false;
        }
        
        // Break the loop if no changes were made in this iteration
        if !made_changes {
            break;
        }
    }
    
    // Update normalized with our processed name
    normalized = normalized_name;
    
    // Step 2: Handle weight and style variations in family names (with spaces)
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
    let suffixes = [" Pro ", " Std ", " LT ", " MT ", " MS ", " Bk ", " Lt ", " Md ", " Bd ", " Rg ", " Sans "];
    for suffix in &suffixes {
        if normalized.ends_with(suffix) {
            normalized = normalized[0..normalized.len() - suffix.len()].trim().to_string();
        }
    }
    
    // Step 5: Special case handling for specific patterns we've identified
    // Group "Einer Grotesk " and "Einer Grotesk Hairline " together
    if normalized.starts_with("Einer Grotesk ") {
        normalized = "Einer Grotesk ".to_string();
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
    
    // Final cleanup pass for ElaSans-like font variants
    // This handles any cases that might have been missed by earlier logic
    let final_style_indicators = [
        "Bold", "Italic", "Black", "Light", "Medium", "Thin", "Regular", 
        "SemiBold", "ExtraBold", "UltraBold", "ExtraLight", "UltraLight",
        "Condensed", "Expanded", "Heavy", "Book"
    ];
    
    // Check for combined font names where a style is concatenated to base name
    let check_name = normalized.clone();
    for indicator in &final_style_indicators {
        // Find position of this style indicator
        if let Some(pos) = check_name.find(indicator) {
            // Only process if this isn't at the beginning of the string
            if pos > 0 {
                let prefix = &check_name[0..pos];
                // If prefix ends with "Sans" and is at least 3 chars long, this is likely a family name
                if prefix.ends_with("Sans") && prefix.len() >= 4 {
                    return prefix.to_string();
                }
            }
        }
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


