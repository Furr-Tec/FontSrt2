use std::path::PathBuf;

/// Metadata extracted from a font file
#[derive(Clone)]
pub struct FontMetadata {
    /// Font family name
    pub family_name: String,
    /// Font subfamily (style variant)
    pub subfamily: String,
    /// Full font name
    #[allow(dead_code)]
    pub full_name: String,
    /// Font foundry name
    pub foundry: String,
    /// Font weight value
    pub weight: u16,
    /// Whether the font is italic
    pub is_italic: bool,
    /// Original path of the font file
    #[allow(dead_code)]
    pub original_path: PathBuf,
}

/// Unique signature for a font variant
#[derive(Hash, Eq, PartialEq, Debug)]
pub struct FontSignature {
    /// Font family name
    pub family_name: String,
    /// Font weight value
    pub weight: u16,
    /// Whether the font is italic
    pub is_italic: bool,
}

impl FontMetadata {
    /// Create a font signature from this metadata
    #[allow(dead_code)]
    pub fn signature(&self) -> FontSignature {
        FontSignature {
            family_name: self.family_name.clone(),
            weight: self.weight,
            is_italic: self.is_italic,
        }
    }
}

