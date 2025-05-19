use std::path::Path;
use std::fs;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use crate::error::Result;
use crate::models::Config;
use super::{processor::organize_fonts, group::group_by_foundry};

/// Process multiple directories listed in a batch file
pub fn batch_process(config: &Config, batch_file: &Path) -> Result<()> {
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
        let family_folders = Arc::new(Mutex::new(HashMap::new()));
        let foundry_folders = Arc::new(Mutex::new(HashMap::new()));

        organize_fonts(
            dir_path,
            config,
            processed_files.clone(),
            family_folders.clone(),
            foundry_folders.clone()
        )?;

        print!("Would you like to group fonts by foundry for {}? (y/n): ", dir_str);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            println!("Grouping fonts by foundry for {}...", dir_str);
            let mut config_with_foundry = config.clone();
            config_with_foundry.group_by_foundry = true;

            group_by_foundry(
                dir_path,
                &config_with_foundry,
                processed_files,
                family_folders,
                foundry_folders
            )?;

            println!("Fonts grouped by foundry successfully for {}!", dir_str);
        }
    }

    println!("\nBatch processing complete!");
    Ok(())
}

