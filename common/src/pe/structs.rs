//! Structures PE (Portable Executable) pour Windows x64
//!
//! Ce module définit toutes les structures nécessaires pour parser
//! et charger un fichier PE (DLL/EXE) en mémoire.
//!
//! Références:
//! - Microsoft PE/COFF Specification
//! - https://docs.microsoft.com/en-us/windows/win32/debug/pe-format

// ============================================================
// CONSTANTES
// ============================================================

/// Magic number DOS "MZ"
pub const DOS_MAGIC: u16 = 0x5A4D;

/// Signature PE "PE\0\0"
pub const PE_SIGNATURE: u32 = 0x00004550;

/// Magic PE32+ (64-bit)
pub const PE64_MAGIC: u16 = 0x20B;

/// Magic PE32 (32-bit)
pub const PE32_MAGIC: u16 = 0x10B;

// Machine types
pub const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
pub const IMAGE_FILE_MACHINE_I386: u16 = 0x014C;

// DLL Characteristics
pub const IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE: u16 = 0x0040; // ASLR
pub const IMAGE_DLLCHARACTERISTICS_NX_COMPAT: u16 = 0x0100; // DEP
pub const IMAGE_DLLCHARACTERISTICS_NO_SEH: u16 = 0x0400;

// Section characteristics
pub const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;
pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x00000040;
pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;
pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
pub const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
pub const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;
pub const IMAGE_SCN_MEM_DISCARDABLE: u32 = 0x02000000;
pub const IMAGE_SCN_MEM_NOT_CACHED: u32 = 0x04000000;
pub const IMAGE_SCN_MEM_NOT_PAGED: u32 = 0x08000000;

// Data Directory indices
pub const IMAGE_DIRECTORY_ENTRY_EXPORT: usize = 0;
pub const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
pub const IMAGE_DIRECTORY_ENTRY_RESOURCE: usize = 2;
pub const IMAGE_DIRECTORY_ENTRY_EXCEPTION: usize = 3;
pub const IMAGE_DIRECTORY_ENTRY_SECURITY: usize = 4;
pub const IMAGE_DIRECTORY_ENTRY_BASERELOC: usize = 5;
pub const IMAGE_DIRECTORY_ENTRY_DEBUG: usize = 6;
pub const IMAGE_DIRECTORY_ENTRY_TLS: usize = 9;
pub const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: usize = 10;
pub const IMAGE_DIRECTORY_ENTRY_BOUND_IMPORT: usize = 11;
pub const IMAGE_DIRECTORY_ENTRY_IAT: usize = 12;
pub const IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT: usize = 13;
pub const IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR: usize = 14;

// Relocation types
pub const IMAGE_REL_BASED_ABSOLUTE: u16 = 0;
pub const IMAGE_REL_BASED_HIGH: u16 = 1;
pub const IMAGE_REL_BASED_LOW: u16 = 2;
pub const IMAGE_REL_BASED_HIGHLOW: u16 = 3;
pub const IMAGE_REL_BASED_HIGHADJ: u16 = 4;
pub const IMAGE_REL_BASED_DIR64: u16 = 10;

// Memory protection constants
pub const PAGE_NOACCESS: u32 = 0x01;
pub const PAGE_READONLY: u32 = 0x02;
pub const PAGE_READWRITE: u32 = 0x04;
pub const PAGE_WRITECOPY: u32 = 0x08;
pub const PAGE_EXECUTE: u32 = 0x10;
pub const PAGE_EXECUTE_READ: u32 = 0x20;
pub const PAGE_EXECUTE_READWRITE: u32 = 0x40;
pub const PAGE_EXECUTE_WRITECOPY: u32 = 0x80;

// Memory allocation types
pub const MEM_COMMIT: u32 = 0x00001000;
pub const MEM_RESERVE: u32 = 0x00002000;
pub const MEM_DECOMMIT: u32 = 0x00004000;
pub const MEM_RELEASE: u32 = 0x00008000;
pub const MEM_COMMIT_RESERVE: u32 = MEM_COMMIT | MEM_RESERVE;

// DllMain reasons
pub const DLL_PROCESS_ATTACH: u32 = 1;
pub const DLL_THREAD_ATTACH: u32 = 2;
pub const DLL_THREAD_DETACH: u32 = 3;
pub const DLL_PROCESS_DETACH: u32 = 0;

// Import lookup table flag (ordinal vs name)
pub const IMAGE_ORDINAL_FLAG64: u64 = 0x8000000000000000;
pub const IMAGE_ORDINAL_FLAG32: u32 = 0x80000000;

// ============================================================
// STRUCTURES PE
// ============================================================

/// DOS Header - Premier header d'un fichier PE (héritage MS-DOS)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DosHeader {
    /// Magic number "MZ" (0x5A4D)
    pub e_magic: u16,
    /// Bytes on last page of file
    pub e_cblp: u16,
    /// Pages in file
    pub e_cp: u16,
    /// Relocations
    pub e_crlc: u16,
    /// Size of header in paragraphs
    pub e_cparhdr: u16,
    /// Minimum extra paragraphs needed
    pub e_minalloc: u16,
    /// Maximum extra paragraphs needed
    pub e_maxalloc: u16,
    /// Initial (relative) SS value
    pub e_ss: u16,
    /// Initial SP value
    pub e_sp: u16,
    /// Checksum
    pub e_csum: u16,
    /// Initial IP value
    pub e_ip: u16,
    /// Initial (relative) CS value
    pub e_cs: u16,
    /// File address of relocation table
    pub e_lfarlc: u16,
    /// Overlay number
    pub e_ovno: u16,
    /// Reserved words
    pub e_res: [u16; 4],
    /// OEM identifier
    pub e_oemid: u16,
    /// OEM information
    pub e_oeminfo: u16,
    /// Reserved words
    pub e_res2: [u16; 10],
    /// File address of new exe header (PE header offset)
    pub e_lfanew: i32,
}

/// File Header - Informations générales sur le fichier PE
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
    /// Architecture (0x8664 = x64, 0x14C = x86)
    pub machine: u16,
    /// Nombre de sections (.text, .data, etc.)
    pub number_of_sections: u16,
    /// Timestamp de compilation (Unix time)
    pub time_date_stamp: u32,
    /// File offset de la table des symboles (0 si strippé)
    pub pointer_to_symbol_table: u32,
    /// Nombre de symboles
    pub number_of_symbols: u32,
    /// Taille de l'Optional Header
    pub size_of_optional_header: u16,
    /// Flags (DLL, EXE, etc.)
    pub characteristics: u16,
}

/// Data Directory - Pointeur vers une structure de données dans le PE
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DataDirectory {
    /// RVA (Relative Virtual Address) de la structure
    pub virtual_address: u32,
    /// Taille de la structure
    pub size: u32,
}

/// Optional Header 64-bit - Métadonnées essentielles de l'image PE
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OptionalHeader64 {
    /// Magic: 0x20B pour PE64, 0x10B pour PE32
    pub magic: u16,
    /// Version du linker (majeur)
    pub major_linker_version: u8,
    /// Version du linker (mineur)
    pub minor_linker_version: u8,
    /// Taille totale du code exécutable
    pub size_of_code: u32,
    /// Taille des données initialisées
    pub size_of_initialized_data: u32,
    /// Taille des données non initialisées (.bss)
    pub size_of_uninitialized_data: u32,
    /// RVA du point d'entrée (DllMain pour DLL, main pour EXE)
    pub address_of_entry_point: u32,
    /// RVA de la section code
    pub base_of_code: u32,
    /// Adresse de base préférée (souvent 0x140000000 pour PE64)
    pub image_base: u64,
    /// Alignement des sections en mémoire (typiquement 0x1000 = 4KB)
    pub section_alignment: u32,
    /// Alignement des sections dans le fichier (typiquement 0x200 = 512B)
    pub file_alignment: u32,
    /// Version OS minimale (majeur)
    pub major_operating_system_version: u16,
    /// Version OS minimale (mineur)
    pub minor_operating_system_version: u16,
    /// Version de l'image (majeur)
    pub major_image_version: u16,
    /// Version de l'image (mineur)
    pub minor_image_version: u16,
    /// Version du subsystem (majeur)
    pub major_subsystem_version: u16,
    /// Version du subsystem (mineur)
    pub minor_subsystem_version: u16,
    /// Réservé (toujours 0)
    pub win32_version_value: u32,
    /// Taille totale de l'image en mémoire (alignée)
    pub size_of_image: u32,
    /// Taille totale des headers
    pub size_of_headers: u32,
    /// Checksum (vérifié pour drivers)
    pub checksum: u32,
    /// Type de subsystem (GUI=2, Console=3)
    pub subsystem: u16,
    /// Flags DLL (ASLR, DEP, etc.)
    pub dll_characteristics: u16,
    /// Taille de stack réservée
    pub size_of_stack_reserve: u64,
    /// Taille de stack committée
    pub size_of_stack_commit: u64,
    /// Taille de heap réservée
    pub size_of_heap_reserve: u64,
    /// Taille de heap committée
    pub size_of_heap_commit: u64,
    /// Obsolète
    pub loader_flags: u32,
    /// Nombre de Data Directories (16 standard)
    pub number_of_rva_and_sizes: u32,
    /// Table des Data Directories
    pub data_directories: [DataDirectory; 16],
}

/// NT Headers 64-bit - Header principal PE après le DOS header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NtHeaders64 {
    /// Signature "PE\0\0" (0x4550)
    pub signature: u32,
    /// File Header
    pub file_header: FileHeader,
    /// Optional Header 64-bit
    pub optional_header: OptionalHeader64,
}

/// Section Header - Décrit une section du PE (.text, .data, .rdata, etc.)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SectionHeader {
    /// Nom de la section (8 bytes, null-padded)
    pub name: [u8; 8],
    /// Taille en mémoire (peut être > size_of_raw_data pour .bss)
    pub virtual_size: u32,
    /// RVA de la section quand mappée
    pub virtual_address: u32,
    /// Taille dans le fichier (alignée sur file_alignment)
    pub size_of_raw_data: u32,
    /// Offset dans le fichier
    pub pointer_to_raw_data: u32,
    /// Pour les fichiers .obj (0 dans PE final)
    pub pointer_to_relocations: u32,
    /// Pour le debug (0 si strippé)
    pub pointer_to_linenumbers: u32,
    /// Nombre de relocations
    pub number_of_relocations: u16,
    /// Nombre de lignes debug
    pub number_of_linenumbers: u16,
    /// Flags (executable, readable, writable, etc.)
    pub characteristics: u32,
}

impl SectionHeader {
    /// Retourne le nom de la section comme string
    pub fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&c| c == 0).unwrap_or(8);
        std::str::from_utf8(&self.name[..end]).unwrap_or("<invalid>")
    }

    /// Vérifie si la section contient du code
    pub fn is_code(&self) -> bool {
        self.characteristics & IMAGE_SCN_CNT_CODE != 0
    }

    /// Vérifie si la section est exécutable
    pub fn is_executable(&self) -> bool {
        self.characteristics & IMAGE_SCN_MEM_EXECUTE != 0
    }

    /// Vérifie si la section est writable
    pub fn is_writable(&self) -> bool {
        self.characteristics & IMAGE_SCN_MEM_WRITE != 0
    }

    /// Vérifie si la section est readable
    pub fn is_readable(&self) -> bool {
        self.characteristics & IMAGE_SCN_MEM_READ != 0
    }

    /// Convertit les caractéristiques en protection mémoire Windows
    pub fn to_protection(&self) -> u32 {
        let exec = self.characteristics & IMAGE_SCN_MEM_EXECUTE != 0;
        let read = self.characteristics & IMAGE_SCN_MEM_READ != 0;
        let write = self.characteristics & IMAGE_SCN_MEM_WRITE != 0;

        match (exec, read, write) {
            (true, true, true) => PAGE_EXECUTE_READWRITE,
            (true, true, false) => PAGE_EXECUTE_READ,
            (true, false, true) => PAGE_EXECUTE_WRITECOPY,
            (true, false, false) => PAGE_EXECUTE,
            (false, true, true) => PAGE_READWRITE,
            (false, true, false) => PAGE_READONLY,
            (false, false, true) => PAGE_WRITECOPY,
            (false, false, false) => PAGE_NOACCESS,
        }
    }
}

// ============================================================
// STRUCTURES IMPORT
// ============================================================

/// Import Directory Entry - Décrit les imports d'une DLL
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImportDescriptor {
    /// RVA de l'Import Lookup Table (ou Original First Thunk)
    /// Contient les noms/ordinals des fonctions importées
    pub original_first_thunk: u32,
    /// Timestamp (0 si pas bound)
    pub time_date_stamp: u32,
    /// Forwarder chain (-1 si pas de forwarder)
    pub forwarder_chain: u32,
    /// RVA du nom de la DLL (C string)
    pub name: u32,
    /// RVA de l'Import Address Table (First Thunk)
    /// C'est ici qu'on écrit les adresses résolues
    pub first_thunk: u32,
}

impl ImportDescriptor {
    /// Vérifie si c'est le terminator (tout à zéro)
    pub fn is_null(&self) -> bool {
        self.original_first_thunk == 0 && self.name == 0 && self.first_thunk == 0
    }
}

/// Import By Name - Structure pour les imports par nom
#[repr(C)]
#[derive(Debug)]
pub struct ImportByName {
    /// Hint - index dans l'export table (optionnel, pour accélérer)
    pub hint: u16,
    /// Nom de la fonction (C string, taille variable)
    pub name: [u8; 1], // Variable length, on lit au-delà
}

// ============================================================
// STRUCTURES EXPORT
// ============================================================

/// Export Directory - Table des exports d'une DLL
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExportDirectory {
    /// Flags (réservé, 0)
    pub characteristics: u32,
    /// Timestamp de création
    pub time_date_stamp: u32,
    /// Version majeure
    pub major_version: u16,
    /// Version mineure
    pub minor_version: u16,
    /// RVA du nom de la DLL
    pub name: u32,
    /// Ordinal de base (souvent 1)
    pub base: u32,
    /// Nombre total de fonctions exportées
    pub number_of_functions: u32,
    /// Nombre de fonctions avec nom
    pub number_of_names: u32,
    /// RVA du tableau des adresses de fonctions
    pub address_of_functions: u32,
    /// RVA du tableau des noms (pointeurs vers strings)
    pub address_of_names: u32,
    /// RVA du tableau des ordinals
    pub address_of_name_ordinals: u32,
}

// ============================================================
// STRUCTURES RELOCATION
// ============================================================

/// Base Relocation Block Header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BaseRelocationBlock {
    /// RVA de la page concernée
    pub virtual_address: u32,
    /// Taille du bloc (header + entries)
    pub size_of_block: u32,
}

impl BaseRelocationBlock {
    /// Nombre d'entrées de relocation dans ce bloc
    pub fn entry_count(&self) -> usize {
        if self.size_of_block <= 8 {
            0
        } else {
            ((self.size_of_block - 8) / 2) as usize
        }
    }
}

/// Relocation Entry (16 bits)
/// - Bits 0-11: Offset dans la page
/// - Bits 12-15: Type de relocation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RelocationEntry(pub u16);

impl RelocationEntry {
    /// Offset dans la page (0-4095)
    pub fn offset(&self) -> u16 {
        self.0 & 0x0FFF
    }

    /// Type de relocation
    pub fn reloc_type(&self) -> u16 {
        self.0 >> 12
    }
}

// ============================================================
// STRUCTURES TLS (Thread Local Storage)
// ============================================================

/// TLS Directory 64-bit
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TlsDirectory64 {
    /// Adresse de début des données TLS
    pub start_address_of_raw_data: u64,
    /// Adresse de fin des données TLS
    pub end_address_of_raw_data: u64,
    /// Adresse de l'index TLS
    pub address_of_index: u64,
    /// Adresse du tableau de callbacks
    pub address_of_callbacks: u64,
    /// Taille du template zero-fill
    pub size_of_zero_fill: u32,
    /// Caractéristiques
    pub characteristics: u32,
}

// ============================================================
// TYPE ALIASES ET HELPERS
// ============================================================

/// Type pour DllMain
#[allow(non_snake_case)]
pub type DllMainFn = unsafe extern "system" fn(
    hinstDLL: *mut libc::c_void,
    fdwReason: u32,
    lpvReserved: *mut libc::c_void,
) -> i32;

/// Type pour les TLS Callbacks
#[allow(non_snake_case)]
pub type TlsCallbackFn = unsafe extern "system" fn(
    DllHandle: *mut libc::c_void,
    Reason: u32,
    Reserved: *mut libc::c_void,
);
