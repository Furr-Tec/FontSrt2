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
    let variations = vec![
        "Bold", "Italic", "Black", "Light", "Medium", "Thin", "Alt", "Semibold", "Oblique",
        "Extra", "Semi", "Hairline", "Rounded", "Extrabold", "Condensed", "Compressed", "Display",
        "Inline", "Outline", "Solid", "Stencil", "Regular", "Pro", "LT", "Std", "ASCT", "ESCT",
        "SSCT", "Dem", "Lig", "Med", "Rd", "Soft",
    ];

    let first_word = family_name
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    let mut base_name = first_word;

    for variation in variations {
        base_name = base_name.replace(variation, "");
    }

    base_name.trim().to_string()
}

