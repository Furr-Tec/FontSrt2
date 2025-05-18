use crate::models::Config;

/// Log a debug message if debug mode is enabled
pub fn log(config: &Config, message: String) {
    if config.debug_mode {
        println!("[DEBUG] {}", message);
    }
}

