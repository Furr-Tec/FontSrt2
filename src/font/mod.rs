//! Font processing and metadata extraction functionality

pub mod metadata;
pub mod foundry;
pub mod weight;

pub use metadata::{extract_font_metadata, is_valid_font_file, is_already_organized};
pub use foundry::{extract_foundry, extract_foundry_from_metadata};
pub use weight::{determine_weight, is_italic_font};

