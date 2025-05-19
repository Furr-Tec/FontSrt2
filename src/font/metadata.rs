use std::fs;
use std::io::Read;
use std::path::Path;
use font_kit::font::Font;
use ttf_parser::Face;
use crate::models::{Config, FontMetadata, NamingPattern};
use crate::error::{Result, Error};
use crate::utils::{log, clean_name, generate_font_filename};
use super::{foundry::extract_foundry, weight::{determine_weight, is_italic_font}};

/// Check if a file is a valid font file
pub fn is_valid_font_file(path: &Path, config: &Config) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_str().unwrap_or("").to_lowercase();
        if ext == "ttf" || ext == "otf" {
            if let Ok(mut file) = fs::File::open(path) {
                let mut header = [0u8; 4];
                if file.read_exact(&mut header).is_ok() {
                    let is_valid_magic =
                        header == [0x00, 0x01, 0x00, 0x00] || // TTF
                        header == [0x4F, 0x54, 0x54, 0x4F];   // OTF
                    
                    if is_valid_magic {
                        if let Ok(_face) = Face::parse(&fs::read(path).unwrap_or_default(), 0) {
                            log(config, format!("Valid font file: {}", path.display()));
                            return true;
                        }
                    }
                }
            }
        }
    }
    log(config, format!("Invalid font file: {}", path.display()));
    false
}

/// Extract metadata from a font file
pub fn extract_font_metadata(path: &Path, config: &Config) -> Result<Option<FontMetadata>> {
    log(config, format!("Extracting metadata from: {}", path.display()));

    if !is_valid_font_file(path, config) {
        return Ok(None);
    }

    match Font::from_path(path, 0) {
        Ok(font) => {
            let family_name = font.family_name();
            if family_name.is_empty() {
                log(config, format!("Empty family name: {}", path.display()));
                return Ok(None);
            }

            let subfamily = font.postscript_name()
                .unwrap_or_else(|| "Regular".to_string())
                .split('-')
                .nth(1)
                .unwrap_or("Regular")
                .to_string();

            let full_name = font.postscript_name().unwrap_or_else(|| family_name.clone());
            let foundry = extract_foundry(&font, &family_name);
            let weight = determine_weight(&subfamily);
            let is_italic = is_italic_font(&subfamily);

            log(config, format!(
                "Metadata extracted - Family: {}, Subfamily: {}, Foundry: {}, Weight: {}, Italic: {}",
                family_name, subfamily, foundry, weight, is_italic
            ));

            Ok(Some(FontMetadata {
                family_name,
                subfamily,
                full_name,
                foundry,
                weight,
                is_italic,
                original_path: path.to_path_buf(),
            }))
        }
        Err(e) => {
            log(config, format!("Failed to load font: {}", e));
            Err(Error::Font(format!("Failed to load font: {}", e)))
        }
    }
}

/// Check if a file is already organized in the correct structure and has the correct name
#[allow(dead_code)]
pub fn is_already_organized(path: &Path, metadata: &FontMetadata, config: &Config) -> bool {
    // Get parent directories
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };

    // Get grandparent for hierarchy check
    let grandparent = parent.parent();

    // Check for foundry/family structure when using those patterns
    if matches!(config.naming_pattern, NamingPattern::FoundryFamily | NamingPattern::FoundryFamilySubfamily) {
        // Need both parent and grandparent
        if grandparent.is_none() {
            return false;
        }

        let parent_name = match parent.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        let grandparent_name = match grandparent.unwrap().file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        // Check if parent is family name and grandparent is foundry name
        if clean_name(&metadata.family_name) != parent_name || 
           clean_name(&metadata.foundry) != grandparent_name {
            return false;
        }
    } else {
        // For other patterns, just check if parent is family name
        let parent_name = match parent.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        if clean_name(&metadata.family_name) != parent_name {
            return false;
        }
    }

    // Now check filename
    let expected_filename = generate_font_filename(metadata, &config.naming_pattern);
    let actual_filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    actual_filename == expected_filename
}

