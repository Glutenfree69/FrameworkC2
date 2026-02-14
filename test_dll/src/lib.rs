use windows_sys::Win32::Foundation::{BOOL, TRUE, HMODULE};
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK, MB_ICONINFORMATION};

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    _hinst: HMODULE,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        let title = b"Oh ZEBI UNE DLL\0";
        let message = b"FREE YOUNG THUG\0";

        MessageBoxA(
            core::ptr::null_mut() as _,
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
    TRUE
}
