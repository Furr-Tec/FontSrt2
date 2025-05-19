use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::env;
use std::io::{self, Write};

mod error;
mod models;
mod utils;
mod font;
mod organizer;
mod cli;

use error::{Result, Error};
use models::Config;
use utils::log;
use utils::file::{ensure_directory_exists, safe_move_file, safe_move_directory, merge_directories};
use cli::{parse_args, get_help_message, get_user_input, get_user_choice, ask_group_by_foundry};
use organizer::{organize_fonts, batch_process, group_by_foundry};
use font::{extract_font_metadata, is_valid_font_file, is_already_organized};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check if help is requested
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("{}", get_help_message());
        return Ok(());
    }

    // Initialize configuration
    let config = Config::new(
        args.contains(&"--debug".to_string()),
        parse_args(),
    );

    if config.debug_mode {
        log(&config, "Debug mode enabled".to_string());
        log(&config, format!("Using naming pattern: {}", config.naming_pattern));
    }

    // Check for batch mode
    if let Some(batch_file_pos) = args.iter().position(|arg| arg == "--batch") {
        if batch_file_pos + 1 < args.len() {
            let batch_file = Path::new(&args[batch_file_pos + 1]).to_path_buf();
            if batch_file.is_file() {
                return batch_process(&config, &batch_file);
            } else {
                println!("Error: Batch file '{}' not found", batch_file.display());
                return Err(Error::InvalidPath(batch_file));
            }
        } else {
            println!("Error: --batch option requires a file path");
            return Err(Error::Config("--batch option requires a file path".to_string()));
        }
    }

    // Process single directory
    let font_dir = get_user_input(&config)?;

    // Initialize shared data structures
    let processed_files = Arc::new(Mutex::new(HashSet::new()));
    let family_folders = Arc::new(Mutex::new(HashMap::new()));
    let foundry_folders = Arc::new(Mutex::new(HashMap::new()));

    match get_user_choice()?.as_str() {
        "1" => {
            organize_fonts(
                &font_dir,
                &config,
                processed_files.clone(),
                family_folders.clone(),
                foundry_folders.clone()
            )?;

            println!("Font organization complete!");

            if ask_group_by_foundry()? {
                println!("Grouping fonts by foundry...");
                let config_with_foundry = Config {
                    group_by_foundry: true,
                    ..config
                };

                group_by_foundry(
                    &font_dir,
                    &config_with_foundry,
                    processed_files,
                    family_folders,
                    foundry_folders
                )?;

                println!("Fonts grouped by foundry successfully!");
            }
        },
        "2" => {
            println!("Grouping fonts by foundry...");
            let config_with_foundry = Config {
                group_by_foundry: true,
                ..config
            };

            group_by_foundry(
                &font_dir,
                &config_with_foundry,
                processed_files,
                family_folders,
                foundry_folders
            )?;

            println!("Fonts grouped by foundry successfully!");
        },
        _ => {
            println!("Invalid choice. Exiting.");
        }
    }

    Ok(())
}
 