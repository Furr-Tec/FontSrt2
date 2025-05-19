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
    // Comprehensive and future-proof indicator set for universal normalization
    // Includes all style, weight, width, feature, script, edition, vendor, region, case, demo/trial, alternate, shortform, and numeric/variant tokens
    // Policy: Any token in this array will be stripped from font family names for grouping
    let style_indicators = [
        // --- General style/weight/width modifiers ---
        "Thin", "UltraThin", "ExtraThin", "Black", "UltraBlack", "ExtraBlack", "Bold", "ExtraBold", "UltraBold", "SemiBold", "DemiBold", "Regular", "Medium", "Book", "Light", "ExtraLight", "UltraLight", "SuperLight",
        "Poster", "Title", "Text", "Subhead", "Deck", "Caption", "Display", "Mini", "Micro", "Small", "Core", "Expert",
        // --- Slant, posture, script ---
        "Italic", "It", "Oblique", "Slant", "Slanted", "Upright", "Roman", "Standing",
        // --- Width/layout ---
        "Condensed", "ExtraCondensed", "UltraCondensed", "Compressed", "SemiCondensed", "Narrow", "Wide", "SemiWide", "Expanded", "ExtraExpanded", "UltraExpanded", "Extended", "ExtraExtended", "UltraExtended",
        // --- Case/feature/variant alternates ---
        "AllCaps", "SmallCaps", "SC", "LC", "UC", "Outline", "OutlineItalic", "Shadow", "ShadowItalic", "Stencil", "StencilItalic",
        "Soft", "Sharp", "Fat", "Jumbo", "Super", "Hairline", "Grotesk", "Grotesque", "Gothic", "Script", "Sans", "Serif", "Mono", "Monospace", "Roman", "Rounded",
        // --- Numerics, version/edition/region ---
        "Number", "Numeral", "Double", "Single", "Ed", "Edition", "Core", "VF", "Var", "Variable", "Preview", "Demo", "Trial", "Icon", "Icons", "Glyph", "Glyphs", "Ornaments", "Inline", "Line", "InlineFill", "Fill", "Stripes", "Dots", "Wood", "3D",
        // --- Language/script/region ---
        "Arabic", "Greek", "Cyrillic", "Devanagari", "CJK", "JP", "JA", "KR", "TC", "SC", "VN", "Intl", "Int", "International", "Latin", "Rus", "Grk", "Kor", "Jap",
        // --- Vendor/industry initials ---
        "TT", "OTF", "TTF", "ITC", "BT", "LT", "MT", "LL", "W1G", "Std", "Pro", "Alt", "Alternate", "Expert", "Prime", "TTF", "WGL", "Intl", "MonoIt", "MonoItalic", "Hand", "Handwritten",
        // --- Optical/feature and obscure ---
        "Caption", "Drop", "Glow", "Cond", "Cnd", "Ext", "Exp", "Cmp", "CmpIt", "CmpXBold", "CmpXLight", "CndIt", "CndXBold", "CndXLight", "CndMedium", "CndLight", "XLight", "XBold", "XXBold", "XXLight", "BK", "Bk", "MD", "Md", "BD", "Bd", "RG", "Rg",
        // --- Compound style shorteners seen in filesystems ---
        "Blk", "SuperBold", "Extra", "Ultra", "Wide", "Narrow", "CN", "Cd", "Wd",
        // --- Numeric indicators (from One to NinetyNine) ---
        "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
        "Eleven", "Twelve", "Thirteen", "Fourteen", "Fifteen", "Sixteen", "Seventeen", "Eighteen", "Nineteen", "Twenty",
        "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety", "Hundred",
        // --- Edition and version ---
        "V1", "V2", "V3", "V4", "V5", "V6",
        // --- Misc collectors ---
        "Base", "Inverted", "AltOne", "AltTwo", "Orig", "Original", "Outline", "Insert", "Delete", "Special", "Syntax", "BookItalic", "RomanItalic", "BookOblique", "MediumOblique", "BoldOblique", "TextBook", "Normal",
        // --- Repeat some ambiguous ones for clarity ---
        "Deck", "Sub", "Ex", "Expert", "Edit", "Ed", "LCI", "Lic", "LicID", "WID", "Goth", "Grotesk", "Grot", "Kap", "Klima", "Fatface",
        // --- Explicitly include hundreds known from industry foundries ---
        "Promo", "Demo", "Trial", "Test", "Doc", "RomanShadow", "ShadowRoman", "ShadowBold", "PosterItalic", "PosterBold",
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


