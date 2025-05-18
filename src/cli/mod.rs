//! Command-line interface handling and user interaction

mod args;
mod interaction;

pub use args::{parse_args, get_help_message};
pub use interaction::{get_user_input, get_user_choice, ask_group_by_foundry};

use crate::models::{Config, NamingPattern};
use crate::error::Result;

