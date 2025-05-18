use std::path::{Path, PathBuf};
use std::io::{self, Write};
use crate::error::{Result, Error};
use crate::models::Config;
use crate::utils::log;

/// Get the input directory from command line args or user input
pub fn get_user_input(config: &Config) -> Result<PathBuf> {
    let args: Vec<String> = std::env::args().collect();

    // Check if path is provided as command-line argument
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
        return Err(Error::InvalidPath(path));
    }

    log(config, format!("User input directory: {}", path.display()));
    Ok(path)
}

/// Get user choice for organization mode
pub fn get_user_choice() -> Result<String> {
    println!("What would you like to do?");
    println!("1. Sort fonts (organize by family)");
    println!("2. Group font folders by foundry");
    print!("Enter your choice (1 or 2): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    Ok(choice.trim().to_string())
}

/// Ask user if they want to group by foundry
pub fn ask_group_by_foundry() -> Result<bool> {
    print!("Would you like to group fonts by foundry? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase() == "y")
}

