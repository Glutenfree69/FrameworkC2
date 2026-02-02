//! Résolution des SSN (System Service Numbers) via PEB/ntdll
//!
//! Ce module parcourt le PEB pour trouver ntdll.dll et résoudre
//! les numéros de syscall dynamiquement au runtime.

use core::slice;
use std::arch::asm;
use std::ptr::addr_of;

use ntapi::ntldr::PLDR_DATA_TABLE_ENTRY;
use ntapi::ntpebteb::{PPEB, TEB};
use ntapi::ntpsapi::PPEB_LDR_DATA;
use ntapi::FIELD_OFFSET;

use winapi::shared::minwindef::{PUSHORT, PWORD};
use winapi::shared::ntdef::{NULL, PLIST_ENTRY, PUCHAR, PVOID, ULONG};
use winapi::um::winnt::{
    PIMAGE_DATA_DIRECTORY, PIMAGE_DOS_HEADER, PIMAGE_EXPORT_DIRECTORY, PIMAGE_NT_HEADERS,
};

use crate::syscalls::obf::djb2_hash;

/// Lire un qword depuis le segment GS (x64)
///
/// # Safety
///
/// Cette fonction est unsafe car elle effectue une lecture directe de mémoire
/// depuis le segment GS du processeur. L'appelant doit s'assurer que :
///
/// - `offset` pointe vers une adresse valide dans le segment GS
/// - L'offset est aligné sur 8 octets (pour une lecture de qword)
/// - Le code s'exécute en mode x86_64 avec un segment GS valide (configuré par Windows)
///
/// En pratique, cette fonction est sûre lorsqu'utilisée avec des offsets connus
/// du TEB (Thread Environment Block) comme `NT_TIB::_Self`.
#[cfg(target_arch = "x86_64")]
pub unsafe fn __readgsqword(offset: u32) -> u64 {
    let out: u64;
    asm!(
        "mov {}, gs:[{:e}]",
        lateout(reg) out,
        in(reg) offset,
        options(nostack, pure, readonly),
    );
    out
}

/// Obtenir le TEB (Thread Environment Block) du thread courant.
///
/// # Safety
///
/// Cette fonction est unsafe car elle :
///
/// - Accède directement au segment GS pour lire l'adresse du TEB
/// - Retourne un pointeur brut vers une structure système
///
/// L'appelant doit s'assurer que :
///
/// - Le code s'exécute sur Windows en mode x86_64
/// - Le pointeur retourné n'est pas déréférencé après que le thread ait terminé
/// - Les accès au TEB sont synchronisés si partagés entre threads (bien que chaque
///   thread ait son propre TEB)
///
/// Le pointeur retourné est valide pour la durée de vie du thread courant.
pub unsafe fn nt_current_teb() -> *mut TEB {
    use winapi::um::winnt::NT_TIB;
    let teb_offset = FIELD_OFFSET!(NT_TIB, _Self) as u32;
    __readgsqword(teb_offset) as *mut TEB
}

/// Obtenir le PEB (Process Environment Block) du processus courant.
///
/// # Safety
///
/// Cette fonction est unsafe car elle :
///
/// - Appelle `nt_current_teb()` qui accède directement au segment GS
/// - Déréférence le pointeur TEB pour accéder au champ `ProcessEnvironmentBlock`
/// - Retourne un pointeur brut vers une structure système partagée
///
/// L'appelant doit s'assurer que :
///
/// - Le code s'exécute sur Windows en mode x86_64
/// - Les modifications au PEB sont synchronisées car il est partagé entre tous
///   les threads du processus
/// - Le pointeur n'est pas utilisé après la terminaison du processus
///
/// Le pointeur retourné est valide pour la durée de vie du processus.
pub unsafe fn nt_current_peb() -> PPEB {
    (*nt_current_teb()).ProcessEnvironmentBlock
}

/// Calculer la longueur d'une string C
pub fn get_cstr_len(pointer: *const char) -> usize {
    let mut tmp: u64 = pointer as u64;
    unsafe {
        while *(tmp as *const u8) != 0 {
            tmp += 1;
        }
    }
    (tmp - pointer as u64) as _
}

/// Trouver l'adresse d'un module par son hash (parcours du PEB)
pub fn get_module_addr(hash: ULONG) -> PVOID {
    let ldr: PPEB_LDR_DATA;
    let header: PLIST_ENTRY;
    let mut dt_entry: PLDR_DATA_TABLE_ENTRY;
    let mut entry: PLIST_ENTRY;
    let mut mod_hash: ULONG;
    let mut mod_name: &[u8];
    let mut mod_len: usize;

    unsafe {
        ldr = (*nt_current_peb()).Ldr;
        header = addr_of!((*ldr).InLoadOrderModuleList) as PLIST_ENTRY;
        entry = (*header).Flink;

        while header as u64 != entry as u64 {
            dt_entry = entry as PLDR_DATA_TABLE_ENTRY;
            mod_len = ((*dt_entry).BaseDllName.Length) as usize;
            mod_name = slice::from_raw_parts((*dt_entry).BaseDllName.Buffer as *const u8, mod_len);
            mod_hash = djb2_hash(mod_name) as ULONG;

            if mod_hash == hash {
                return (*dt_entry).DllBase;
            }

            entry = (*entry).Flink;
        }
    }
    NULL
}

/// Trouver l'adresse d'une fonction par son hash dans l'export table
pub fn get_function_addr(module_addr: PVOID, hash: u32) -> PVOID {
    let nt_header: PIMAGE_NT_HEADERS;
    let data_dir: PIMAGE_DATA_DIRECTORY;
    let exp_dir: PIMAGE_EXPORT_DIRECTORY;
    let addr_funcs: PWORD;
    let addr_names: PWORD;
    let addr_ords: PUSHORT;
    let mut str_addr: PUCHAR;
    let mut str_len: usize;
    let addr_list: &[u32];
    let name_list: &[u32];
    let ord_list: &[u16];

    let dos_header: PIMAGE_DOS_HEADER = module_addr as PIMAGE_DOS_HEADER;

    unsafe {
        nt_header = (dos_header as u64 + (*dos_header).e_lfanew as u64) as PIMAGE_NT_HEADERS;
        data_dir = addr_of!((*nt_header).OptionalHeader.DataDirectory[0]) as PIMAGE_DATA_DIRECTORY;

        if (*data_dir).VirtualAddress != 0 {
            exp_dir =
                (dos_header as u64 + (*data_dir).VirtualAddress as u64) as PIMAGE_EXPORT_DIRECTORY;
            addr_funcs = (dos_header as u64 + (*exp_dir).AddressOfFunctions as u64) as PWORD;
            addr_names = (dos_header as u64 + (*exp_dir).AddressOfNames as u64) as PWORD;
            addr_ords = (dos_header as u64 + (*exp_dir).AddressOfNameOrdinals as u64) as PUSHORT;

            name_list =
                slice::from_raw_parts(addr_names as *const u32, (*exp_dir).NumberOfNames as usize);
            ord_list =
                slice::from_raw_parts(addr_ords as *const u16, (*exp_dir).NumberOfNames as usize);
            addr_list =
                slice::from_raw_parts(addr_funcs as *const u32, (*exp_dir).NumberOfNames as usize);

            for iter in 0..(*exp_dir).NumberOfNames as usize {
                str_addr = (dos_header as u64 + name_list[iter] as u64) as PUCHAR;
                str_len = get_cstr_len(str_addr as _);
                if hash == djb2_hash(slice::from_raw_parts(str_addr as _, str_len)) {
                    return (dos_header as u64 + addr_list[ord_list[iter] as usize] as u64)
                        as PVOID;
                }
            }
        }
    }
    NULL
}

/// Résoudre le SSN (System Service Number) et l'adresse du gadget "syscall; ret"
/// Retourne (SSN, adresse_gadget)
pub fn get_ssn(hash: u32) -> (u16, u64) {
    let ntdll_addr = get_module_addr(crate::obf!("ntdll.dll"));
    let funct_addr = get_function_addr(ntdll_addr, hash);

    // Le SSN est à offset +4 dans le prologue de la fonction NT
    // Bytecode typique:
    //   mov r10, rcx    (4C 8B D1)
    //   mov eax, SSN    (B8 XX 00 00 00)  <- SSN ici
    let ssn = unsafe { *((funct_addr as u64 + 4) as *const u16) };

    // Le gadget "syscall; ret" est généralement à offset +0x12
    let ssn_addr = funct_addr as u64 + 0x12;

    (ssn, ssn_addr)
}
