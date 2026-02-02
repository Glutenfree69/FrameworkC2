//! Shell command execution module
//!
//! Executes commands via PowerShell on Windows.

use super::CommandResult;

#[cfg(windows)]
use std::process::Command;
#[cfg(windows)]
use uuid::Uuid;

/// Execute a shell command via PowerShell (Windows only)
#[cfg(windows)]
pub fn execute_shell_command(cmd: &str) -> CommandResult {
    // Build the PowerShell command with UTF-8 encoding prefix
    let ps_command = format!("$OutputEncoding = [System.Text.Encoding]::UTF8; {}", cmd);

    // Execute via PowerShell
    let output = Command::new("powershell")
        .args(["-Command", &ps_command])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let mut result = String::new();

            if !stdout.is_empty() {
                result.push_str(&stdout);
            }

            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push_str("\n\n--- STDERR ---\n");
                }
                result.push_str(&stderr);
            }

            if result.is_empty() {
                result = "(No output)".to_string();
            }

            let filename = format!("cmd_{}.txt", Uuid::new_v4());

            CommandResult::FileOutput {
                content: result,
                filename,
            }
        }
        Err(e) => CommandResult::Error(format!("Failed to execute command: {}", e)),
    }
}

/// Stub for non-Windows platforms
#[cfg(not(windows))]
pub fn execute_shell_command(_cmd: &str) -> CommandResult {
    CommandResult::Error("Shell commands are only supported on Windows".to_string())
}
