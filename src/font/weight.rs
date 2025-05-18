/// Determine the weight value from a subfamily name
pub fn determine_weight(subfamily: &str) -> u16 {
    let subfamily_lower = subfamily.to_lowercase();

    match &subfamily_lower {
        s if s.contains("thin") => 100,
        s if s.contains("extra light") || s.contains("ultralight") => 200,
        s if s.contains("light") => 300,
        s if s.contains("regular") || s.contains("normal") || s.contains("book") => 400,
        s if s.contains("medium") => 500,
        s if s.contains("semibold") || s.contains("demibold") => 600,
        s if s.contains("bold") && !s.contains("extrabold") && !s.contains("semibold") => 700,
        s if s.contains("extrabold") || s.contains("ultrabold") => 800,
        s if s.contains("black") || s.contains("heavy") => 900,
        s if s.contains("extrablack") || s.contains("ultrablack") => 950,
        _ => 400, // Default to regular weight
    }
}

/// Check if a font is italic based on its subfamily name
pub fn is_italic_font(subfamily: &str) -> bool {
    let subfamily_lower = subfamily.to_lowercase();
    subfamily_lower.contains("italic") || subfamily_lower.contains("oblique")
}

