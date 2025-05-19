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
    // 1. Clean and early exit for empty input
    let normalized = family_name.trim();
    if normalized.is_empty() {
        return String::from("Unknown");
    }

    // 2. Collect and deduplicate all normalized words splitting on space and camel case
    //     Use a regex for camelCase->camel Case detection
    let camel_case_re = regex::Regex::new(r"([a-z])([A-Z])").unwrap();
    let separated = camel_case_re.replace_all(normalized, "$1 $2");
    let mut tokens: Vec<&str> = separated.split_whitespace().collect();

    // 3. Comprehensive, deduplicated style/weight/variant indicator arrays
    // (Suffixes, embedded, and known concatenated forms)
    let style_indicators = [
        // Primary style/weight tokens (add more as needed for new foundries/variants)
        "Bauhaus", "Text", "SmallCaps", "Variable", "Italic", "Oblique", "Caps", "Display", "Serif", "Sans", "Mono", "MonoIt", "MonoItalic", "Hand", "SC", "LC", "Rounded", "Stencil", "Shadow", "Grotesk", "Poster", "Title",
        "Cmp", "CmpIt", "CmpXBold", "CmpXLight", "CmpMedium", "CmpLight", "Cnd", "CndIt", "CndXBold", "CndXLight", "CndMedium", "CndLight",
        "Bold", "SemiBold", "ExtraBold", "UltraBold", "SemiLight", "Light", "Medium", "Thin", "Black", "Heavy", "Book", "Super", "Hairline", "Outline", "Wide", "Narrow", "Compressed", "Expanded", "Extended",
        "Condensed", "Expanded", "Extended", "Narrow", "Wide", "Compressed", "DemiBold", "Pro", "Std", "Bk", "Lt", "Md", "Bd", "Rg", "Roman", "Script", "Shadow", "Engraved", "NC", "LC", "UC", "Serif", "SansSerif", "Calligraphy", "Gothic", "Regular", "Normal",
        // Additional indicators for exhaustive coverage
        "BlackItalic", "BoldItalic", "BookItalic", "ExtraBoldItalic", "ItalicBold", "ItalicBook", "LightItalic", "MediumItalic", "ObliqueBold", "ObliqueLight", "ObliqueMedium", "ThinItalic", "None", "Insert", "Delete", "SuperBold",
        "CapsItalic", "BlackCapsItalic", "BoldCapsItalic", "AllCaps", "Ultra", "UltraCondensed", "UltraExpanded", "UltraLight", "UltraWide", "XLight", "XBold", "XXBold", "XXLight", "BoldXLight",
        "Number", "Numeral", "Double", "Single", "LC", "SC", "Roman", "LC", "Wide", "Round", "HeavyItalic", "MediumCaps", "Soft", "Expert", "PosterItalic"
    ];

    // 4. Remove *all* tokens that match any indicator, anywhere in the sequence (policy: not just trailing, but all known variants)
    let base_tokens: Vec<&str> = tokens.into_iter().filter(|token| {
        let token_lower = token.to_ascii_lowercase();
        !style_indicators.iter().any(|indicator| {
            let ind_lower = indicator.to_ascii_lowercase();
            token_lower == ind_lower || token_lower.ends_with(&ind_lower)
        })
    }).collect();

    // 5. If all tokens are indicators, fall back to "Unknown" (or original first word)
    if base_tokens.is_empty() {
        return family_name.split_whitespace().next().unwrap_or("Unknown").to_string();
    }

    // 6. Join base tokens back, ensuring clean, trimmed output
    let base_cleaned = base_tokens.join(" ").trim().trim_matches('.').to_string();
    if base_cleaned.is_empty() {
        return String::from("Unknown");
    }
    base_cleaned
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


