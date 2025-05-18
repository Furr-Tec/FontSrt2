use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::io::{self, Write, Read};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use font_kit::font::Font;
use std::env;
use rayon::prelude::*;
use ttf_parser::Face;
use regex::Regex;
use std::fmt;

#[derive(Clone)]
struct Config {
    debug_mode: bool,
    naming_pattern: NamingPattern,
    group_by_foundry: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NamingPattern {
    FamilySubfamily,          // "Helvetica (Bold)"
    FoundryFamilySubfamily,   // "Adobe Helvetica (Bold)"
    FamilyWeight,             // "Helvetica 700"
    FoundryFamily,            // "Adobe/Helvetica"
}

impl fmt::Display for NamingPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NamingPattern::FamilySubfamily => write!(f, "%Family% (%Subfamily%)"),
            NamingPattern::FoundryFamilySubfamily => write!(f, "%Foundry% %Family% (%Subfamily%)"),
            NamingPattern::FamilyWeight => write!(f, "%Family% %Weight%"),
            NamingPattern::FoundryFamily => write!(f, "%Foundry%/%Family%"),
        }
    }
}

#[derive(Clone)]
struct FontMetadata {
    family_name: String,
    subfamily: String,
    full_name: String,
    foundry: String,
    weight: u16,
    is_italic: bool,
    original_path: PathBuf,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct FontSignature {
    family_name: String,
    weight: u16,
    is_italic: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if help is requested
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("{}", get_help_message());
        return Ok(());
    }

    // Parse naming pattern from arguments
    let naming_pattern = if args.contains(&"--foundry-family-subfamily".to_string()) {
        NamingPattern::FoundryFamilySubfamily
    } else if args.contains(&"--family-weight".to_string()) {
        NamingPattern::FamilyWeight
    } else if args.contains(&"--foundry-family".to_string()) {
        NamingPattern::FoundryFamily
    } else {
        // Default pattern
        NamingPattern::FamilySubfamily
    };

    let config = Config {
        debug_mode: args.contains(&"--debug".to_string()),
        naming_pattern,
        group_by_foundry: false,
    };

    if config.debug_mode {
        println!("Debug mode enabled");
        println!("Using naming pattern: {}", config.naming_pattern);
    }

    // Check for batch mode
    if let Some(batch_file_pos) = args.iter().position(|arg| arg == "--batch") {
        if batch_file_pos + 1 < args.len() {
            let batch_file = Path::new(&args[batch_file_pos + 1]).to_path_buf();
            if batch_file.is_file() {
                return batch_process(&config, &batch_file);
            } else {
                println!("Error: Batch file '{}' not found", batch_file.display());
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Batch file '{}' not found", batch_file.display()),
                )));
            }
        } else {
            println!("Error: --batch option requires a file path");
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--batch option requires a file path",
            )));
        }
    }

    // If not in batch mode, process a single directory
    let font_dir = get_user_input(&config)?;

    // Initialize shared data structures
    let processed_files = Arc::new(Mutex::new(HashSet::new()));
    let _family_folders = Arc::new(Mutex::new(HashMap::new()));
    let _foundry_folders = Arc::new(Mutex::new(HashMap::new()));

    // Ask user what they want to do
    println!("What would you like to do?");
    println!("1. Sort fonts (organize by family)");
    println!("2. Group font folders by foundry");
    print!("Enter your choice (1 or 2): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    match choice {
        "1" => {
            // Process the directory
            organize_fonts(
                &font_dir,
                &config,
                processed_files.clone(),
                _family_folders.clone(),
                _foundry_folders.clone()
            )?;

            println!("Font organization complete!");

            // Ask if user wants to group by foundry
            print!("Would you like to group fonts by foundry? (y/n): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                println!("Grouping fonts by foundry...");
                let mut config_with_foundry = config.clone();
                config_with_foundry.group_by_foundry = true;

                // Group fonts by foundry
                group_by_foundry(
                    &font_dir,
                    &config_with_foundry,
                    processed_files,
                    _family_folders,
                    _foundry_folders
                )?;

                println!("Fonts grouped by foundry successfully!");
            }
        },
        "2" => {
            println!("Grouping fonts by foundry...");
            let mut config_with_foundry = config.clone();
            config_with_foundry.group_by_foundry = true;

            // Group fonts by foundry
            group_by_foundry(
                &font_dir,
                &config_with_foundry,
                processed_files,
                _family_folders,
                _foundry_folders
            )?;

            println!("Fonts grouped by foundry successfully!");
        },
        _ => {
            println!("Invalid choice. Exiting.");
            return Ok(());
        }
    }

    Ok(())
}

fn get_user_input(config: &Config) -> Result<PathBuf, Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if path is provided as command-line argument
    // Look for the first argument that is not a flag and is a valid directory
    for i in 1..args.len() {
        if !args[i].starts_with("--") {
            let path = Path::new(&args[i]).to_path_buf();
            if path.is_dir() {
                log(config, format!("Using directory from command line: {}", path.display()));
                return Ok(path);
            }
        }
    }

    // Otherwise ask for input
    print!("Enter the path to the folder containing font files: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let path = Path::new(input.trim()).to_path_buf();

    if !path.is_dir() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!("The specified path '{}' is not a valid directory", path.display()),
        )));
    }

    log(config, format!("User input directory: {}", path.display()));
    Ok(path)
}

// Function to process a batch of font directories
fn batch_process(config: &Config, batch_file: &Path) -> Result<(), Box<dyn Error>> {
    println!("Batch processing enabled. Reading directories from {}", batch_file.display());

    let content = fs::read_to_string(batch_file)?;
    let dirs: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
        .collect();

    println!("Found {} directories to process", dirs.len());

    for (i, dir_str) in dirs.iter().enumerate() {
        let dir_path = Path::new(dir_str);
        if !dir_path.is_dir() {
            println!("Warning: '{}' is not a valid directory, skipping", dir_str);
            continue;
        }

        println!("\nProcessing directory {}/{}: {}", i + 1, dirs.len(), dir_str);

        // Use the same shared structures for all directories
        let processed_files = Arc::new(Mutex::new(HashSet::new()));
        let _family_folders = Arc::new(Mutex::new(HashMap::new()));
        let _foundry_folders = Arc::new(Mutex::new(HashMap::new()));

        // Process this directory
        organize_fonts(
            dir_path,
            config,
            processed_files.clone(),
            _family_folders.clone(),
            _foundry_folders.clone()
        )?;

        // Ask if user wants to group by foundry for this directory
        print!("Would you like to group fonts by foundry for {}? (y/n): ", dir_str);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            println!("Grouping fonts by foundry for {}...", dir_str);
            let mut config_with_foundry = config.clone();
            config_with_foundry.group_by_foundry = true;

            // Group fonts by foundry
            group_by_foundry(
                dir_path,
                &config_with_foundry,
                processed_files.clone(),
                _family_folders.clone(),
                _foundry_folders.clone()
            )?;

            println!("Fonts grouped by foundry successfully for {}!", dir_str);
        }
    }

    println!("\nBatch processing complete!");
    Ok(())
}

fn is_valid_font_file(path: &Path, config: &Config) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_str().unwrap_or("").to_lowercase();
        if ext == "ttf" || ext == "otf" {
            if let Ok(mut file) = fs::File::open(path) {
                let mut header = [0u8; 4];
                if file.read_exact(&mut header).is_ok() {
                    let is_valid_magic =
                        header == [0x00, 0x01, 0x00, 0x00] || header == [0x4F, 0x54, 0x54, 0x4F];
                    if is_valid_magic {
                        if let Ok(_face) = Face::parse(&fs::read(path).unwrap_or_default(), 0) {
                            log(config, format!("File {} is a valid font", path.display()));
                            return true;
                        }
                    }
                }
            }
        }
    }
    log(config, format!(
        "File {} invalid or unsupported extension",
        path.display()
    ));
    false
}

fn is_already_organized(path: &Path, metadata: &FontMetadata, config: &Config) -> bool {
    // Check if the file is already in the correct structure and has the correct name

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

fn extract_font_metadata(path: &Path, config: &Config) -> Option<FontMetadata> {
    log(config, format!("Checking file: {}", path.display()));

    if !is_valid_font_file(path, config) {
        return None;
    }

    match Font::from_path(path, 0) {
        Ok(font) => {
            let family_name = font.family_name();
            if family_name.is_empty() {
                log(
                    config,
                    format!("Family name is empty for font at {}", path.display()),
                );
                return None;
            }

            // Extract subfamily name (style)
            let subfamily = font.postscript_name()
                .unwrap_or_else(|| "Regular".to_string())
                .split('-')
                .nth(1)
                .unwrap_or("Regular")
                .to_string();

            // Extract full name
            let full_name = font.postscript_name().unwrap_or_else(|| family_name.clone());

            // Extract or guess foundry
            let foundry = extract_foundry(&font, &family_name);

            // Determine weight
            let weight = determine_weight(&subfamily);

            // Determine if italic
            let is_italic = is_italic_font(&subfamily);

            log(
                config,
                format!(
                    "Metadata for {}: Family: {}, Subfamily: {}, Foundry: {}, Weight: {}, Italic: {}",
                    path.display(),
                    family_name,
                    subfamily,
                    foundry,
                    weight,
                    is_italic
                ),
            );

            Some(FontMetadata {
                family_name,
                subfamily,
                full_name,
                foundry,
                weight,
                is_italic,
                original_path: path.to_path_buf(),
            })
        }
        Err(e) => {
            log(
                config,
                format!("Failed to load font from {}: {}", path.display(), e),
            );
            None
        }
    }
}

fn extract_foundry(font: &Font, family_name: &str) -> String {
    // Try to extract from font metadata
    if let Some(foundry) = extract_foundry_from_metadata(font) {
        return foundry;
    }

    // Try to extract from family name using common patterns
    let foundry_patterns = [
        // Check for common patterns like "Adobe Garamond"
        Regex::new(r"^(Adobe|Monotype|Linotype|ITC|URW|Bitstream|Google|Microsoft|Apple|IBM|Hoefler|Typekit|FontFont|Emigre|Dalton Maag|Font Bureau|House Industries|P22|Typotheque|Underware|Fontfabric|Fontsmith|Klim|Process|Commercial|Grilli|Production|Sudtipos|Typofonderie|Canada|Rosetta|Darden|Positype|Typonine|Latinotype|Typejockeys|Suitcase|Elsner\+Flake|Scangraphic|Berthold|Letraset|Agfa|Paratype|Fontshop|Letterhead|Neufville)\s+.*").unwrap(),

        // Check for patterns like "HelveticaLT" which indicates Linotype
        Regex::new(r"^.*(LT|MT|ITC|URW|BT|MS|GD|FF|DF|DM|FB|HI|P22|TT|UW|FS|KT|PT|CT|GT|ST|TF|CD|RT|DD|TN|TJ|SC|EF|SG|LS|AG|LH|NV)$").unwrap(),

        // Check for common foundry names in the middle of the family name
        Regex::new(r".*(by|from|font by|designed by)\s+(Adobe|Monotype|Linotype|ITC|URW|Bitstream|Google|Microsoft|Apple|IBM|Hoefler|Typekit|FontFont|Emigre|Dalton Maag|Font Bureau|House Industries|P22|Typotheque|Underware|Fontfabric|Fontsmith|Klim|Process|Commercial|Grilli|Production|Sudtipos|Typofonderie|Canada|Rosetta|Darden|Positype|Typonine|Latinotype|Typejockeys|Suitcase|Elsner\+Flake|Scangraphic|Berthold|Letraset|Agfa|Paratype|Fontshop|Letterhead|Neufville)\s+.*").unwrap(),
    ];

    for pattern in &foundry_patterns {
        if let Some(captures) = pattern.captures(family_name) {
            if let Some(foundry) = captures.get(1) {
                return foundry.as_str().to_string();
            } else if let Some(foundry) = captures.get(2) {
                return foundry.as_str().to_string();
            } else if pattern.is_match(family_name) {
                // Extract foundry from font name abbreviation
                if family_name.ends_with("LT") {
                    return "Linotype".to_string();
                } else if family_name.ends_with("MT") {
                    return "Monotype".to_string();
                } else if family_name.ends_with("ITC") {
                    return "ITC".to_string();
                } else if family_name.ends_with("BT") {
                    return "Bitstream".to_string();
                } else if family_name.ends_with("MS") {
                    return "Microsoft".to_string();
                } else if family_name.ends_with("GD") {
                    return "Adobe".to_string();
                } else if family_name.ends_with("FF") {
                    // FF could be FontFont or Fontfabric, prioritize FontFont as it's more common
                    return "FontFont".to_string();
                } else if family_name.ends_with("DF") {
                    return "Emigre".to_string();
                } else if family_name.ends_with("DM") {
                    return "Dalton Maag".to_string();
                } else if family_name.ends_with("FB") {
                    return "Font Bureau".to_string();
                } else if family_name.ends_with("HI") {
                    return "House Industries".to_string();
                } else if family_name.ends_with("P22") {
                    return "P22".to_string();
                } else if family_name.ends_with("TT") {
                    return "Typotheque".to_string();
                } else if family_name.ends_with("UW") {
                    return "Underware".to_string();
                } else if family_name.ends_with("FS") {
                    // FS could be Fontsmith or Fontshop, prioritize Fontsmith
                    return "Fontsmith".to_string();
                } else if family_name.ends_with("KT") {
                    return "Klim".to_string();
                } else if family_name.ends_with("PT") {
                    // PT could be Process, Positype, or Paratype, prioritize Process
                    return "Process".to_string();
                } else if family_name.ends_with("CT") {
                    return "Commercial".to_string();
                } else if family_name.ends_with("GT") {
                    return "Grilli".to_string();
                } else if family_name.ends_with("ST") {
                    return "Sudtipos".to_string();
                } else if family_name.ends_with("TF") {
                    return "Typofonderie".to_string();
                } else if family_name.ends_with("CD") {
                    return "Canada".to_string();
                } else if family_name.ends_with("RT") {
                    return "Rosetta".to_string();
                } else if family_name.ends_with("DD") {
                    return "Darden".to_string();
                } else if family_name.ends_with("TN") {
                    return "Typonine".to_string();
                } else if family_name.ends_with("LT") {
                    // LT could be Linotype or Latinotype, prioritize Linotype as it's more common
                    return "Linotype".to_string();
                } else if family_name.ends_with("TJ") {
                    return "Typejockeys".to_string();
                } else if family_name.ends_with("SC") {
                    return "Suitcase".to_string();
                } else if family_name.ends_with("EF") {
                    return "Elsner+Flake".to_string();
                } else if family_name.ends_with("SG") {
                    return "Scangraphic".to_string();
                } else if family_name.ends_with("BT") {
                    // BT could be Bitstream or Berthold, prioritize Bitstream as it's more common
                    return "Bitstream".to_string();
                } else if family_name.ends_with("LS") {
                    return "Letraset".to_string();
                } else if family_name.ends_with("AG") {
                    return "Agfa".to_string();
                } else if family_name.ends_with("LH") {
                    return "Letterhead".to_string();
                } else if family_name.ends_with("NV") {
                    return "Neufville".to_string();
                }
            }
        }
    }

    // If no foundry detected, use "Unknown"
    "Unknown".to_string()
}

fn extract_foundry_from_metadata(font: &Font) -> Option<String> {
    // Try to extract from font metadata
    // font-kit doesn't expose all OpenType name table entries directly
    // but we can try to infer from what's available

    // Check if the postscript name contains foundry information
    if let Some(postscript_name) = font.postscript_name() {
        // PostScript names often follow the pattern "FoundryName-FontName"
        let parts: Vec<&str> = postscript_name.split('-').collect();
        if parts.len() > 1 {
            // The first part might be the foundry name or a combination of foundry and family
            let potential_foundry = parts[0];

            // Check if it's a known foundry abbreviation
            match potential_foundry {
                "ADBE" => return Some("Adobe".to_string()),
                "MONO" => return Some("Monotype".to_string()),
                "LINO" => return Some("Linotype".to_string()),
                "ITC" => return Some("ITC".to_string()),
                "URW" => return Some("URW".to_string()),
                "BITS" => return Some("Bitstream".to_string()),
                "GOOG" => return Some("Google".to_string()),
                "MSFT" => return Some("Microsoft".to_string()),
                "APPL" => return Some("Apple".to_string()),
                "IBM" => return Some("IBM".to_string()),
                "HOEF" => return Some("Hoefler".to_string()),
                "TKIT" => return Some("Typekit".to_string()),
                "FNTF" => return Some("FontFont".to_string()), // FNTF could be FontFont or Fontfabric, prioritize FontFont
                "EMGR" => return Some("Emigre".to_string()),
                "DLTN" => return Some("Dalton Maag".to_string()),
                "FNTB" => return Some("Font Bureau".to_string()),
                "HIND" => return Some("House Industries".to_string()),
                "P22" => return Some("P22".to_string()),
                "TYPO" => return Some("Typotheque".to_string()),
                "UNDR" => return Some("Underware".to_string()),
                "KLIM" => return Some("Klim".to_string()),
                "PROC" => return Some("Process".to_string()),
                "COMM" => return Some("Commercial".to_string()),
                "GRIL" => return Some("Grilli".to_string()),
                "SUDT" => return Some("Sudtipos".to_string()),
                "TYPF" => return Some("Typofonderie".to_string()),
                "CANA" => return Some("Canada".to_string()),
                "ROSE" => return Some("Rosetta".to_string()),
                "DARD" => return Some("Darden".to_string()),
                "POSI" => return Some("Positype".to_string()),
                "TYPN" => return Some("Typonine".to_string()),
                "LATN" => return Some("Latinotype".to_string()),
                "TYPJ" => return Some("Typejockeys".to_string()),
                "SUIT" => return Some("Suitcase".to_string()),
                "ELSN" => return Some("Elsner+Flake".to_string()),
                "SCAN" => return Some("Scangraphic".to_string()),
                "BERT" => return Some("Berthold".to_string()),
                "LETR" => return Some("Letraset".to_string()),
                "AGFA" => return Some("Agfa".to_string()),
                "PARA" => return Some("Paratype".to_string()),
                "FNTS" => return Some("Fontsmith".to_string()), // FNTS could be Fontsmith or Fontshop, prioritize Fontsmith
                "LTTR" => return Some("Letterhead".to_string()),
                "NEUF" => return Some("Neufville".to_string()),
                _ => {}
            }
        }
    }

    None
}

fn extract_foundry_from_path(path: &Option<PathBuf>) -> Option<String> {
    // Try to extract foundry from the font file path
    if let Some(path) = path {
        // Check if the font is in a directory structure that indicates the foundry
        if let Some(parent) = path.parent() {
            if let Some(dir_name) = parent.file_name() {
                if let Some(dir_str) = dir_name.to_str() {
                    // Check if the directory name is a known foundry
                    let dir_lower = dir_str.to_lowercase();

                    // Check for common foundry names in directory
                    let foundry_names = [
                        "adobe", "monotype", "linotype", "itc", "urw", "bitstream", 
                        "google", "microsoft", "apple", "ibm", "hoefler", "typekit", 
                        "fontfont", "emigre", "dalton maag", "font bureau", 
                        "house industries", "p22", "typotheque", "underware", 
                        "fontfabric", "fontsmith", "klim", "process", "commercial", 
                        "grilli", "sudtipos", "typofonderie", "canada", "rosetta", 
                        "darden", "positype", "typonine", "latinotype", "typejockeys", 
                        "suitcase", "elsner+flake", "scangraphic", "berthold", 
                        "letraset", "agfa", "paratype", "fontshop", "letterhead", 
                        "neufville"
                    ];

                    for &foundry in &foundry_names {
                        if dir_lower.contains(foundry) {
                            // Capitalize first letter of each word
                            return Some(capitalize_words(foundry));
                        }
                    }

                    // Check if the parent directory might be the foundry
                    if let Some(grandparent) = parent.parent() {
                        if let Some(gp_name) = grandparent.file_name() {
                            if let Some(gp_str) = gp_name.to_str() {
                                let gp_lower = gp_str.to_lowercase();

                                for &foundry in &foundry_names {
                                    if gp_lower.contains(foundry) {
                                        // Capitalize first letter of each word
                                        return Some(capitalize_words(foundry));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn determine_weight(subfamily: &str) -> u16 {
    let subfamily_lower = subfamily.to_lowercase();

    // Match standard weight names to numeric weights
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

fn is_italic_font(subfamily: &str) -> bool {
    let subfamily_lower = subfamily.to_lowercase();
    subfamily_lower.contains("italic") || subfamily_lower.contains("oblique")
}

fn ensure_directory_exists(dir: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    if !dir.exists() {
        log(
            config,
            format!("Directory {} does not exist. Creating it now.", dir.display()),
        );
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

// Helper function to capitalize the first letter of each word in a string
fn capitalize_words(s: &str) -> String {
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

fn normalize_family_name(family_name: &str) -> String {
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

// Function to clean a name for use in filenames
fn clean_name(name: &str) -> String {
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

fn organize_fonts(
    dir: &Path,
    config: &Config,
    processed_files: Arc<Mutex<HashSet<PathBuf>>>,
    _family_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
    _foundry_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
) -> Result<(), Box<dyn Error>> {
    let duplicates_dir = dir.join("duplicates");
    ensure_directory_exists(&duplicates_dir, config)?;

    // Collect metadata for all fonts first to help with duplicate detection
    let font_metadata_map: Arc<Mutex<HashMap<PathBuf, FontMetadata>>> = Arc::new(Mutex::new(HashMap::new()));

    // Map all fonts by their signatures for duplication detection
    let font_signatures: Arc<Mutex<HashMap<FontSignature, Vec<PathBuf>>>> = Arc::new(Mutex::new(HashMap::new()));

    // First pass: collect metadata
    fs::read_dir(dir)?
        .par_bridge()
        .for_each(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Skip directories or processed files
                if !path.is_file() || processed_files.lock().unwrap().contains(&path) {
                    return;
                }

                if let Some(metadata) = extract_font_metadata(&path, config) {
                    // Add to metadata map
                    font_metadata_map.lock().unwrap().insert(path.clone(), metadata.clone());

                    // Add to signatures for duplicate detection
                    let signature = FontSignature {
                        family_name: metadata.family_name.clone(),
                        weight: metadata.weight,
                        is_italic: metadata.is_italic,
                    };

                    font_signatures.lock().unwrap()
                        .entry(signature)
                        .or_insert_with(Vec::new)
                        .push(path.clone());
                }
            }
        });

    log(config, format!("Collected metadata for {} fonts", 
        font_metadata_map.lock().unwrap().len()));

    // Create directory structure and move files
    let metadata_map = font_metadata_map.lock().unwrap().clone();
    let metadata_count = metadata_map.len();

    for (path, metadata) in &metadata_map {
        let mut processed_set = processed_files.lock().unwrap();

        if processed_set.contains(path) {
            continue;
        }

        processed_set.insert(path.clone());

        // Determine target directory based on naming pattern
        let target_dir = build_folder_path(dir, metadata, config);
        ensure_directory_exists(&target_dir, config)?;

        // Format new filename based on naming pattern
        let base_name = format_font_name(metadata, &config.naming_pattern);
        let clean_base_name = clean_name(&base_name);

        // Get file extension
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("ttf")
            .to_lowercase();

        // Create new filename
        let new_filename = format!("{}.{}", clean_base_name, extension);
        let new_path = target_dir.join(&new_filename);

        // Handle file move or duplicate
        if new_path.exists() {
            let mut unique_name = new_filename.clone();
            let mut counter = 1;

            while target_dir.join(&unique_name).exists() {
                let name_part = clean_base_name.clone();
                unique_name = format!("{}_{}.{}", name_part, counter, extension);
                counter += 1;
            }

            let final_path = target_dir.join(unique_name);

            log(
                config,
                format!(
                    "Font with same name exists. Renaming {} to {}",
                    path.display(),
                    final_path.display()
                ),
            );

            if let Err(e) = safe_move_file(path, &final_path, config) {
                log(
                    config,
                    format!("Error moving file {}: {}", path.display(), e),
                );
            }
        } else {
            log(
                config,
                format!("Moving {} to {}", path.display(), new_path.display()),
            );

            if let Err(e) = safe_move_file(path, &new_path, config) {
                log(
                    config,
                    format!("Error moving file {}: {}", path.display(), e),
                );
            }
        }
    }

    // Report statistics
    println!("Font organization summary:");
    println!("  - {} fonts processed", metadata_count);

    Ok(())
}

// Simple logging function
fn log(config: &Config, message: String) {
    if config.debug_mode {
        println!("[DEBUG] {}", message);
    }
}

// Function to generate help message
fn get_help_message() -> String {
    r#"Font Organizer - A tool for organizing font collections

USAGE:
    FontSrt [OPTIONS] [DIRECTORY]

ARGS:
    <DIRECTORY>    Path to the directory containing font files (optional)

OPTIONS:
    -h, --help                      Show this help message
    --debug                         Enable debug output
    --batch <FILE>                  Process multiple directories listed in a file
    --foundry-family-subfamily      Use "Foundry Family (Subfamily)" naming pattern
    --family-weight                 Use "Family Weight" naming pattern
    --foundry-family                Use "Foundry/Family" directory structure

By default, fonts are organized using the "Family (Subfamily)" naming pattern.

After organizing fonts, the program will ask if you want to group them by foundry.
This will create a structure where fonts are organized into foundry folders first,
then by family within each foundry folder.
"#.to_string()
}

// Function to generate a filename for a font based on its metadata and the naming pattern
fn generate_font_filename(metadata: &FontMetadata, naming_pattern: &NamingPattern) -> String {
    let base_name = format_font_name(metadata, naming_pattern);
    let extension = metadata.original_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("ttf")
        .to_lowercase();

    format!("{}.{}", clean_name(&base_name), extension)
}

// Function to format a font name based on its metadata and the naming pattern
fn format_font_name(metadata: &FontMetadata, naming_pattern: &NamingPattern) -> String {
    match naming_pattern {
        NamingPattern::FamilySubfamily => {
            if metadata.subfamily.to_lowercase() == "regular" {
                metadata.family_name.clone()
            } else {
                format!("{} ({})", metadata.family_name, metadata.subfamily)
            }
        },
        NamingPattern::FoundryFamilySubfamily => {
            if metadata.subfamily.to_lowercase() == "regular" {
                format!("{} {}", metadata.foundry, metadata.family_name)
            } else {
                format!("{} {} ({})", metadata.foundry, metadata.family_name, metadata.subfamily)
            }
        },
        NamingPattern::FamilyWeight => {
            format!("{} {}{}", 
                metadata.family_name, 
                metadata.weight,
                if metadata.is_italic { " Italic" } else { "" }
            )
        },
        NamingPattern::FoundryFamily => {
            // This pattern is primarily for directory structure, but we'll use it for filename too
            if metadata.subfamily.to_lowercase() == "regular" {
                format!("{}_{}", metadata.foundry, metadata.family_name)
            } else {
                format!("{}_{} ({})", metadata.foundry, metadata.family_name, metadata.subfamily)
            }
        },
    }
}

// Function to build the folder path for a font based on its metadata and the naming pattern
fn build_folder_path(base_dir: &Path, metadata: &FontMetadata, config: &Config) -> PathBuf {
    match config.naming_pattern {
        NamingPattern::FoundryFamily => {
            // Create a foundry/family structure
            let foundry_dir = base_dir.join(clean_name(&metadata.foundry));
            foundry_dir.join(clean_name(&metadata.family_name))
        },
        _ => {
            if config.group_by_foundry {
                // If grouping by foundry is enabled, create a foundry/family structure
                let foundry_dir = base_dir.join(clean_name(&metadata.foundry));
                foundry_dir.join(clean_name(&metadata.family_name))
            } else {
                // For all other patterns, just use family name as the directory
                base_dir.join(clean_name(&metadata.family_name))
            }
        }
    }
}

// Function to group fonts by foundry after initial organization
fn group_by_foundry(
    dir: &Path,
    config: &Config,
    _processed_files: Arc<Mutex<HashSet<PathBuf>>>,
    _family_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
    _foundry_folders: Arc<Mutex<HashMap<String, PathBuf>>>,
) -> Result<(), Box<dyn Error>> {
    // Create a map to track which family belongs to which foundry
    let mut family_to_foundry: HashMap<String, String> = HashMap::new();

    // First, scan the directory for font files to determine foundry for each family
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();

            // Only process directories (font family folders)
            if path.is_dir() && path.file_name().unwrap_or_default() != "duplicates" {
                let family_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

                // Scan this family folder for font files to determine foundry
                let font_files = find_font_files(&path, config)?;

                if !font_files.is_empty() {
                    // Use the first font file to determine the foundry
                    if let Some(metadata) = extract_font_metadata(&font_files[0], config) {
                        family_to_foundry.insert(family_name, clean_name(&metadata.foundry));
                    }
                }
            }
        }
    }

    // Now move each family folder to its foundry folder
    for (family, foundry) in family_to_foundry {
        let family_dir = dir.join(&family);
        let foundry_dir = dir.join(&foundry);

        // Create foundry directory if it doesn't exist
        ensure_directory_exists(&foundry_dir, config)?;

        // Move family folder to foundry folder
        let target_dir = foundry_dir.join(&family);

        if target_dir.exists() {
            // If target directory already exists, merge contents
            log(
                config,
                format!(
                    "Target directory {} already exists, merging contents",
                    target_dir.display()
                ),
            );

            // Move all files from family_dir to target_dir
            merge_directories(&family_dir, &target_dir, config)?;

            // Remove the now-empty family directory
            if let Err(e) = fs::remove_dir_all(&family_dir) {
                log(
                    config,
                    format!("Error removing directory {}: {}", family_dir.display(), e),
                );
            }
        } else {
            // Move the entire family directory to the foundry directory
            log(
                config,
                format!(
                    "Moving {} to {}",
                    family_dir.display(),
                    target_dir.display()
                ),
            );

            if let Err(e) = safe_move_directory(&family_dir, &target_dir, config) {
                log(
                    config,
                    format!("Error moving directory {}: {}", family_dir.display(), e),
                );
            }
        }
    }

    Ok(())
}

// Helper function to find font files in a directory
fn find_font_files(dir: &Path, config: &Config) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut font_files = Vec::new();

    let entries = fs::read_dir(dir)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_file() && is_valid_font_file(&path, config) {
                font_files.push(path);
            } else if path.is_dir() {
                // Recursively search subdirectories
                let mut sub_fonts = find_font_files(&path, config)?;
                font_files.append(&mut sub_fonts);
            }
        }
    }

    Ok(font_files)
}

// Helper function to safely move a file with fallback to copy+delete if rename fails
fn safe_move_file(src: &Path, dest: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    // First try to rename (fast path)
    match fs::rename(src, dest) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If rename fails, log it and try copy+delete
            log(
                config,
                format!("Rename failed for {}, trying copy+delete: {}", src.display(), e),
            );

            // Copy the file
            fs::copy(src, dest)?;

            // Delete the original
            match fs::remove_file(src) {
                Ok(_) => Ok(()),
                Err(e) => {
                    log(
                        config,
                        format!("Warning: Could not delete source file {} after copying: {}", src.display(), e),
                    );
                    // We still consider this a success since the file was copied
                    Ok(())
                }
            }
        }
    }
}

// Helper function to safely move a directory with fallback to recursive copy+delete if rename fails
fn safe_move_directory(src_dir: &Path, dest_dir: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    // First try to rename (fast path)
    match fs::rename(src_dir, dest_dir) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If rename fails, log it and try recursive copy+delete
            log(
                config,
                format!("Rename failed for directory {}, trying recursive copy: {}", src_dir.display(), e),
            );

            // Make sure destination directory exists
            ensure_directory_exists(dest_dir, config)?;

            // Copy all contents recursively
            merge_directories(src_dir, dest_dir, config)?;

            // Try to remove the source directory
            match fs::remove_dir_all(src_dir) {
                Ok(_) => Ok(()),
                Err(e) => {
                    log(
                        config,
                        format!("Warning: Could not delete source directory {} after copying: {}", src_dir.display(), e),
                    );
                    // We still consider this a success since the contents were copied
                    Ok(())
                }
            }
        }
    }
}

// Helper function to merge the contents of two directories
fn merge_directories(src_dir: &Path, dest_dir: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    let entries = fs::read_dir(src_dir)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default();
            let dest_path = dest_dir.join(file_name);

            if path.is_file() {
                // If destination file already exists, create a unique name
                if dest_path.exists() {
                    let mut counter = 1;
                    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    let extension = path.extension().unwrap_or_default().to_string_lossy();

                    let mut unique_path = dest_dir.join(format!("{}_{}.{}", file_stem, counter, extension));

                    while unique_path.exists() {
                        counter += 1;
                        unique_path = dest_dir.join(format!("{}_{}.{}", file_stem, counter, extension));
                    }

                    if let Err(e) = safe_move_file(&path, &unique_path, config) {
                        log(
                            config,
                            format!("Error moving file {}: {}", path.display(), e),
                        );
                    }
                } else {
                    // Move the file
                    if let Err(e) = safe_move_file(&path, &dest_path, config) {
                        log(
                            config,
                            format!("Error moving file {}: {}", path.display(), e),
                        );
                    }
                }
            } else if path.is_dir() {
                // Create destination directory if it doesn't exist
                ensure_directory_exists(&dest_path, config)?;

                // Recursively merge subdirectories
                merge_directories(&path, &dest_path, config)?;

                // Remove the now-empty source directory
                if let Err(e) = fs::remove_dir(&path) {
                    log(
                        config,
                        format!("Error removing directory {}: {}", path.display(), e),
                    );
                }
            }
        }
    }

    Ok(())
}
