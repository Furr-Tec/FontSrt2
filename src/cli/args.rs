use std::env;
use crate::models::NamingPattern;

/// Parse command line arguments into naming pattern
pub fn parse_args() -> NamingPattern {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--foundry-family-subfamily".to_string()) {
        NamingPattern::FoundryFamilySubfamily
    } else if args.contains(&"--family-weight".to_string()) {
        NamingPattern::FamilyWeight
    } else if args.contains(&"--foundry-family".to_string()) {
        NamingPattern::FoundryFamily
    } else {
        NamingPattern::FamilySubfamily
    }
}

/// Get the help message for command-line usage
pub fn get_help_message() -> String {
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

