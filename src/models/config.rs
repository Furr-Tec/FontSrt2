use std::fmt;

/// Configuration for the font organization process
#[derive(Clone)]
pub struct Config {
    /// Enable debug output
    pub debug_mode: bool,
    /// Pattern to use for naming font files
    pub naming_pattern: NamingPattern,
    /// Whether to group fonts by foundry
    pub group_by_foundry: bool,
}

/// Patterns for naming font files
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NamingPattern {
    /// "Helvetica (Bold)"
    FamilySubfamily,
    /// "Adobe Helvetica (Bold)"
    FoundryFamilySubfamily,
    /// "Helvetica 700"
    FamilyWeight,
    /// "Adobe/Helvetica"
    FoundryFamily,
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

impl Config {
    /// Create a new configuration with default settings
    pub fn new(debug_mode: bool, naming_pattern: NamingPattern) -> Self {
        Self {
            debug_mode,
            naming_pattern,
            group_by_foundry: false,
        }
    }

    /// Parse command line arguments and create a configuration
    #[allow(dead_code)]
    pub fn from_args() -> crate::error::Result<Self> {
        use std::env;
        
        let args: Vec<String> = env::args().collect();
        
        // Check for help flag
        if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
            return Err(crate::error::Error::Config("Help requested".to_string()));
        }
        
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

        Ok(Config {
            debug_mode: args.contains(&"--debug".to_string()),
            naming_pattern,
            group_by_foundry: false,
        })
    }
}

