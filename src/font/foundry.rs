use font_kit::font::Font;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref FOUNDRY_PATTERNS: [Regex; 2] = [
        Regex::new(r"^(Adobe|Monotype|Linotype|ITC|URW|Bitstream|Google|Microsoft|Apple|IBM|Hoefler|Typekit|FontFont|Emigre|Dalton Maag|Font Bureau|House Industries|P22|Typotheque|Underware|Fontfabric|Fontsmith|Klim|Process|Commercial|Grilli|Production|Sudtipos|Typofonderie|Canada|Rosetta|Darden|Positype|Typonine|Latinotype|Typejockeys|Suitcase|Elsner\+Flake|Scangraphic|Berthold|Letraset|Agfa|Paratype|Fontshop|Letterhead|Neufville)\s+.*").unwrap(),
        Regex::new(r"^.*(LT|MT|ITC|URW|BT|MS|GD|FF|DF|DM|FB|HI|P22|TT|UW|FS|KT|PT|CT|GT|ST|TF|CD|RT|DD|TN|TJ|SC|EF|SG|LS|AG|LH|NV)$").unwrap(),
    ];
}

/// Extract foundry information from font metadata and name
pub fn extract_foundry(font: &Font, family_name: &str) -> String {
    if let Some(foundry) = extract_foundry_from_metadata(font) {
        return foundry;
    }

    for pattern in FOUNDRY_PATTERNS.iter() {
        if let Some(captures) = pattern.captures(family_name) {
            if let Some(foundry) = captures.get(1) {
                return foundry.as_str().to_string();
            }
        }
    }

    extract_foundry_from_abbreviation(family_name)
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Extract foundry information from font metadata
pub fn extract_foundry_from_metadata(font: &Font) -> Option<String> {
    if let Some(postscript_name) = font.postscript_name() {
        let parts: Vec<&str> = postscript_name.split('-').collect();
        if parts.len() > 1 {
            match parts[0] {
                "ADBE" => Some("Adobe"),
                "MONO" => Some("Monotype"),
                "LINO" => Some("Linotype"),
                "ITC" => Some("ITC"),
                "URW" => Some("URW"),
                "BITS" => Some("Bitstream"),
                "GOOG" => Some("Google"),
                "MSFT" => Some("Microsoft"),
                "APPL" => Some("Apple"),
                _ => None,
            }.map(String::from)
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract foundry from font name abbreviations
fn extract_foundry_from_abbreviation(family_name: &str) -> Option<String> {
    if family_name.ends_with("LT") {
        Some("Linotype")
    } else if family_name.ends_with("MT") {
        Some("Monotype")
    } else if family_name.ends_with("ITC") {
        Some("ITC")
    } else if family_name.ends_with("BT") {
        Some("Bitstream")
    } else if family_name.ends_with("MS") {
        Some("Microsoft")
    } else {
        None
    }.map(String::from)
}

