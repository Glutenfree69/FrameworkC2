#![allow(non_snake_case)]

use std::ptr::null_mut;

type HINSTANCE = *mut u8;
type DWORD = u32;
type BOOL = i32;

const DLL_PROCESS_ATTACH: DWORD = 1;

#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBoxA(hWnd: *mut u8, lpText: *const u8, lpCaption: *const u8, uType: u32) -> i32;
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(_: HINSTANCE, reason: DWORD, _: *mut u8) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            MessageBoxA(
                null_mut(),
                b"Pacpac loader OK!\0".as_ptr(),
                b"Test DLL\0".as_ptr(),
                0,
            );
        }
    }
    1
}
