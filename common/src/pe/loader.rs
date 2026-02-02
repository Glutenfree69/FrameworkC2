//! PE Loader - Chargement de DLL en mémoire
//!
//! Ce module implémente un PE loader complet qui permet de charger
//! n'importe quelle DLL en mémoire sans passer par LoadLibrary.
//!
//! Étapes du chargement:
//! 1. Parser le PE
//! 2. Allouer la mémoire pour l'image
//! 3. Copier les headers
//! 4. Mapper les sections
//! 5. Appliquer les relocations (si base différente)
//! 6. Résoudre les imports (IAT)
//! 7. Protéger les sections
//! 8. Appeler les TLS callbacks (si présents)
//! 9. Appeler DllMain

use std::ptr;
use std::ffi::CString;

use super::structs::*;
use super::parser::{PeParser, ParseError};

// On utilise notre propre type c_void pour éviter les conflits entre libc et winapi
type CVoid = std::ffi::c_void;

// ============================================================
// ERREURS
// ============================================================

/// Erreurs de chargement
#[derive(Debug)]
pub enum LoadError {
    /// Erreur de parsing
    Parse(ParseError),
    /// Échec d'allocation mémoire
    AllocationFailed,
    /// Échec de protection mémoire
    ProtectionFailed(u32),
    /// Module introuvable pour résolution d'import
    ModuleNotFound(String),
    /// Fonction introuvable pour résolution d'import
    FunctionNotFound { module: String, function: String },
    /// Échec d'appel à DllMain
    DllMainFailed,
    /// Relocation impossible (pas de table de relocation + base occupée)
    RelocationRequired,
    /// Erreur interne
    Internal(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Parse(e) => write!(f, "Parse error: {}", e),
            LoadError::AllocationFailed => write!(f, "Memory allocation failed"),
            LoadError::ProtectionFailed(e) => write!(f, "Memory protection failed: 0x{:X}", e),
            LoadError::ModuleNotFound(m) => write!(f, "Module not found: {}", m),
            LoadError::FunctionNotFound { module, function } => {
                write!(f, "Function not found: {}!{}", module, function)
            }
            LoadError::DllMainFailed => write!(f, "DllMain returned FALSE"),
            LoadError::RelocationRequired => write!(f, "Relocation required but not available"),
            LoadError::Internal(s) => write!(f, "Internal error: {}", s),
        }
    }
}

impl From<ParseError> for LoadError {
    fn from(e: ParseError) -> Self {
        LoadError::Parse(e)
    }
}

// ============================================================
// PE LOADER
// ============================================================

/// PE Loader - charge une DLL en mémoire
pub struct PeLoader {
    /// Adresse de base où la DLL est mappée
    base_address: *mut u8,
    /// Taille de l'image en mémoire
    image_size: usize,
    /// Adresse du point d'entrée (DllMain)
    entry_point: Option<DllMainFn>,
    /// La DLL a-t-elle été initialisée (DllMain appelé)?
    initialized: bool,
}

impl PeLoader {
    /// Charge une DLL depuis des bytes bruts (self-injection)
    /// 
    /// Cette fonction:
    /// 1. Parse le PE
    /// 2. Alloue de la mémoire dans le process courant
    /// 3. Mappe la DLL
    /// 4. Appelle DllMain(DLL_PROCESS_ATTACH)
    /// 
    /// # Safety
    /// Cette fonction est unsafe car elle exécute du code arbitraire.
    pub unsafe fn load(dll_bytes: &[u8]) -> Result<Self, LoadError> {
        // 1. Parser le PE
        let pe = PeParser::parse(dll_bytes)?;
        
        // 2. Allouer la mémoire
        let image_size = pe.size_of_image() as usize;
        let base_address = Self::allocate_memory(image_size, pe.image_base())?;
        
        // 3. Copier les headers
        Self::copy_headers(&pe, base_address)?;
        
        // 4. Mapper les sections
        Self::map_sections(&pe, base_address)?;
        
        // 5. Appliquer les relocations (si nécessaire)
        let actual_base = base_address as u64;
        let preferred_base = pe.image_base();
        if actual_base != preferred_base {
            if !pe.has_relocations() {
                // Pas de table de relocation et base différente = impossible
                Self::free_memory(base_address, image_size);
                return Err(LoadError::RelocationRequired);
            }
            Self::apply_relocations(&pe, base_address, preferred_base)?;
        }
        
        // 6. Résoudre les imports (IAT)
        Self::resolve_imports(&pe, base_address)?;
        
        // 7. Protéger les sections
        Self::protect_sections(&pe, base_address)?;
        
        // 8. Appeler les TLS callbacks (si présents)
        if pe.has_tls() {
            Self::call_tls_callbacks(&pe, base_address, DLL_PROCESS_ATTACH)?;
        }
        
        // 9. Obtenir le point d'entrée
        let entry_point = if pe.entry_point_rva() != 0 {
            let ep_addr = base_address.add(pe.entry_point_rva() as usize);
            Some(std::mem::transmute::<*mut u8, DllMainFn>(ep_addr))
        } else {
            None
        };
        
        let mut loader = Self {
            base_address,
            image_size,
            entry_point,
            initialized: false,
        };
        
        // 10. Appeler DllMain
        loader.call_entry_point(DLL_PROCESS_ATTACH)?;
        
        Ok(loader)
    }
    
    /// Retourne l'adresse de base de la DLL chargée
    pub fn base_address(&self) -> *mut u8 {
        self.base_address
    }
    
    /// Retourne la taille de l'image
    pub fn image_size(&self) -> usize {
        self.image_size
    }
    
    /// Décharge la DLL (appelle DllMain(DLL_PROCESS_DETACH) et libère la mémoire)
    /// 
    /// # Safety
    /// Cette fonction est unsafe car elle libère de la mémoire et exécute du code.
    pub unsafe fn unload(mut self) -> Result<(), LoadError> {
        // Appeler DllMain(DLL_PROCESS_DETACH) si initialisé
        if self.initialized {
            let _ = self.call_entry_point(DLL_PROCESS_DETACH);
        }
        
        // Libérer la mémoire
        Self::free_memory(self.base_address, self.image_size);
        
        // Empêcher le drop de libérer à nouveau
        self.base_address = ptr::null_mut();
        self.image_size = 0;
        
        Ok(())
    }
    
    // ========================================================
    // FONCTIONS PRIVÉES
    // ========================================================
    
    /// Alloue de la mémoire pour l'image
    /// Essaie d'abord à l'adresse préférée, sinon laisse le système choisir
    unsafe fn allocate_memory(size: usize, preferred_base: u64) -> Result<*mut u8, LoadError> {
        // D'abord essayer à l'adresse préférée
        let addr = Self::virtual_alloc(
            preferred_base as *mut CVoid,
            size,
            MEM_COMMIT_RESERVE,
            PAGE_READWRITE,
        );
        
        if !addr.is_null() {
            return Ok(addr as *mut u8);
        }
        
        // Sinon laisser le système choisir
        let addr = Self::virtual_alloc(
            ptr::null_mut(),
            size,
            MEM_COMMIT_RESERVE,
            PAGE_READWRITE,
        );
        
        if addr.is_null() {
            return Err(LoadError::AllocationFailed);
        }
        
        Ok(addr as *mut u8)
    }
    
    /// Libère la mémoire allouée
    unsafe fn free_memory(base: *mut u8, _size: usize) {
        if !base.is_null() {
            Self::virtual_free(base as *mut CVoid, 0, MEM_RELEASE);
        }
    }
    
    /// Copie les headers PE dans la mémoire allouée
    unsafe fn copy_headers(pe: &PeParser, base: *mut u8) -> Result<(), LoadError> {
        let headers_size = pe.size_of_headers() as usize;
        ptr::copy_nonoverlapping(pe.raw.as_ptr(), base, headers_size);
        Ok(())
    }
    
    /// Mappe les sections dans la mémoire allouée
    unsafe fn map_sections(pe: &PeParser, base: *mut u8) -> Result<(), LoadError> {
        for section in &pe.sections {
            let dest = base.add(section.virtual_address as usize);
            
            // Copier les données de la section depuis le fichier
            if section.size_of_raw_data > 0 {
                let src = pe.raw.as_ptr().add(section.pointer_to_raw_data as usize);
                let copy_size = std::cmp::min(
                    section.size_of_raw_data as usize,
                    section.virtual_size as usize,
                );
                ptr::copy_nonoverlapping(src, dest, copy_size);
            }
            
            // Zero-fill le reste si virtual_size > raw_size
            if section.virtual_size > section.size_of_raw_data {
                let zero_start = dest.add(section.size_of_raw_data as usize);
                let zero_size = (section.virtual_size - section.size_of_raw_data) as usize;
                ptr::write_bytes(zero_start, 0, zero_size);
            }
        }
        
        Ok(())
    }
    
    /// Applique les relocations (base relocation)
    unsafe fn apply_relocations(
        pe: &PeParser,
        base: *mut u8,
        original_base: u64,
    ) -> Result<(), LoadError> {
        let delta = (base as u64).wrapping_sub(original_base) as i64;
        
        for (block, entries) in pe.iter_relocations() {
            for entry in entries {
                let reloc_type = entry.reloc_type();
                let offset = entry.offset() as u32;
                let addr = base.add((block.virtual_address + offset) as usize);
                
                match reloc_type {
                    IMAGE_REL_BASED_ABSOLUTE => {
                        // Padding, ignorer
                    }
                    IMAGE_REL_BASED_HIGHLOW => {
                        // 32-bit relocation
                        let ptr = addr as *mut i32;
                        *ptr = (*ptr).wrapping_add(delta as i32);
                    }
                    IMAGE_REL_BASED_DIR64 => {
                        // 64-bit relocation
                        let ptr = addr as *mut i64;
                        *ptr = (*ptr).wrapping_add(delta);
                    }
                    IMAGE_REL_BASED_HIGH => {
                        let ptr = addr as *mut i16;
                        *ptr = (*ptr).wrapping_add((delta >> 16) as i16);
                    }
                    IMAGE_REL_BASED_LOW => {
                        let ptr = addr as *mut i16;
                        *ptr = (*ptr).wrapping_add(delta as i16);
                    }
                    _ => {
                        // Type non supporté, ignorer
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Résout les imports (remplit l'IAT)
    unsafe fn resolve_imports(pe: &PeParser, base: *mut u8) -> Result<(), LoadError> {
        for (desc, dll_name) in pe.iter_imports() {
            // Charger le module
            let module = Self::load_library(dll_name)?;
            
            // RVA de l'ILT (Import Lookup Table) et IAT (Import Address Table)
            let ilt_rva = if desc.original_first_thunk != 0 {
                desc.original_first_thunk
            } else {
                desc.first_thunk
            };
            let iat_rva = desc.first_thunk;
            
            let mut thunk_offset = 0u32;
            
            loop {
                // Lire l'entrée de l'ILT
                let ilt_addr = base.add((ilt_rva + thunk_offset) as usize);
                let thunk_data = *(ilt_addr as *const u64);
                
                // Terminator (null)
                if thunk_data == 0 {
                    break;
                }
                
                // Résoudre l'adresse de la fonction
                let func_addr = if thunk_data & IMAGE_ORDINAL_FLAG64 != 0 {
                    // Import par ordinal
                    let ordinal = (thunk_data & 0xFFFF) as u16;
                    Self::get_proc_address_ordinal(module, ordinal)?
                } else {
                    // Import par nom
                    let hint_name_rva = thunk_data as u32;
                    // +2 pour skipper le hint (u16)
                    let name_rva = hint_name_rva + 2;
                    let func_name = pe.read_cstr_at_rva(name_rva)
                        .ok_or_else(|| LoadError::Internal("Invalid import name".to_string()))?;
                    Self::get_proc_address(module, func_name)?
                };
                
                // Écrire l'adresse dans l'IAT
                let iat_addr = base.add((iat_rva + thunk_offset) as usize) as *mut u64;
                *iat_addr = func_addr as u64;
                
                // Passer au thunk suivant (8 bytes pour PE64)
                thunk_offset += 8;
            }
            
            let _ = iat_rva; // iat_rva est utilisé implicitement via first_thunk
        }
        
        Ok(())
    }
    
    /// Protège les sections avec les permissions appropriées
    unsafe fn protect_sections(pe: &PeParser, base: *mut u8) -> Result<(), LoadError> {
        for section in &pe.sections {
            let addr = base.add(section.virtual_address as usize);
            let size = section.virtual_size as usize;
            let protection = section.to_protection();
            
            let mut old_protect = 0u32;
            let result = Self::virtual_protect(
                addr as *mut CVoid,
                size,
                protection,
                &mut old_protect,
            );
            
            if !result {
                return Err(LoadError::ProtectionFailed(protection));
            }
        }
        
        Ok(())
    }
    
    /// Appelle les TLS callbacks
    unsafe fn call_tls_callbacks(
        pe: &PeParser,
        base: *mut u8,
        reason: u32,
    ) -> Result<(), LoadError> {
        let tls_dir = pe.data_directory(IMAGE_DIRECTORY_ENTRY_TLS)
            .ok_or_else(|| LoadError::Internal("No TLS directory".to_string()))?;
        
        if tls_dir.virtual_address == 0 {
            return Ok(());
        }
        
        // Lire le TLS directory (attention: les adresses sont absolues, pas RVA)
        let tls: TlsDirectory64 = pe.read_struct_at_rva(tls_dir.virtual_address)
            .ok_or_else(|| LoadError::Internal("Invalid TLS directory".to_string()))?;
        
        if tls.address_of_callbacks == 0 {
            return Ok(());
        }
        
        // Les callbacks sont stockés à une adresse absolue
        // On doit calculer le RVA depuis l'adresse de base
        let callbacks_rva = tls.address_of_callbacks.wrapping_sub(pe.image_base()) as u32;
        let callbacks_ptr = base.add(callbacks_rva as usize) as *const u64;
        
        let mut i = 0;
        loop {
            let callback_addr = *callbacks_ptr.add(i);
            if callback_addr == 0 {
                break;
            }
            
            // Convertir l'adresse absolue en adresse réelle
            let callback_rva = callback_addr.wrapping_sub(pe.image_base()) as u32;
            let callback: TlsCallbackFn = std::mem::transmute(base.add(callback_rva as usize));
            
            callback(base as *mut CVoid, reason, ptr::null_mut());
            i += 1;
        }
        
        Ok(())
    }
    
    /// Appelle le point d'entrée (DllMain)
    unsafe fn call_entry_point(&mut self, reason: u32) -> Result<(), LoadError> {
        if let Some(entry_point) = self.entry_point {
            let result = entry_point(
                self.base_address as *mut CVoid,
                reason,
                ptr::null_mut(),
            );
            
            if result == 0 && reason == DLL_PROCESS_ATTACH {
                return Err(LoadError::DllMainFailed);
            }
            
            if reason == DLL_PROCESS_ATTACH {
                self.initialized = true;
            } else if reason == DLL_PROCESS_DETACH {
                self.initialized = false;
            }
        }
        
        Ok(())
    }
    
    // ========================================================
    // WRAPPERS WINDOWS API
    // ========================================================
    
    /// Wrapper pour VirtualAlloc
    #[cfg(windows)]
    unsafe fn virtual_alloc(
        addr: *mut CVoid,
        size: usize,
        alloc_type: u32,
        protect: u32,
    ) -> *mut CVoid {
        use winapi::um::memoryapi::VirtualAlloc;
        use winapi::ctypes::c_void as WinVoid;
        VirtualAlloc(addr as *mut WinVoid, size, alloc_type, protect) as *mut CVoid
    }
    
    #[cfg(not(windows))]
    unsafe fn virtual_alloc(
        _addr: *mut CVoid,
        _size: usize,
        _alloc_type: u32,
        _protect: u32,
    ) -> *mut CVoid {
        ptr::null_mut()
    }
    
    /// Wrapper pour VirtualFree
    #[cfg(windows)]
    unsafe fn virtual_free(addr: *mut CVoid, size: usize, free_type: u32) -> bool {
        use winapi::um::memoryapi::VirtualFree;
        use winapi::ctypes::c_void as WinVoid;
        VirtualFree(addr as *mut WinVoid, size, free_type) != 0
    }
    
    #[cfg(not(windows))]
    unsafe fn virtual_free(_addr: *mut CVoid, _size: usize, _free_type: u32) -> bool {
        false
    }
    
    /// Wrapper pour VirtualProtect
    #[cfg(windows)]
    unsafe fn virtual_protect(
        addr: *mut CVoid,
        size: usize,
        new_protect: u32,
        old_protect: *mut u32,
    ) -> bool {
        use winapi::um::memoryapi::VirtualProtect;
        use winapi::ctypes::c_void as WinVoid;
        VirtualProtect(addr as *mut WinVoid, size, new_protect, old_protect) != 0
    }
    
    #[cfg(not(windows))]
    unsafe fn virtual_protect(
        _addr: *mut CVoid,
        _size: usize,
        _new_protect: u32,
        _old_protect: *mut u32,
    ) -> bool {
        false
    }
    
    /// Wrapper pour LoadLibraryA
    #[cfg(windows)]
    unsafe fn load_library(name: &str) -> Result<*mut CVoid, LoadError> {
        use winapi::um::libloaderapi::LoadLibraryA;
        
        let c_name = CString::new(name)
            .map_err(|_| LoadError::ModuleNotFound(name.to_string()))?;
        
        let handle = LoadLibraryA(c_name.as_ptr());
        if handle.is_null() {
            return Err(LoadError::ModuleNotFound(name.to_string()));
        }
        
        Ok(handle as *mut CVoid)
    }
    
    #[cfg(not(windows))]
    unsafe fn load_library(name: &str) -> Result<*mut CVoid, LoadError> {
        Err(LoadError::ModuleNotFound(name.to_string()))
    }
    
    /// Wrapper pour GetProcAddress (par nom)
    #[cfg(windows)]
    unsafe fn get_proc_address(module: *mut CVoid, name: &str) -> Result<*mut CVoid, LoadError> {
        use winapi::um::libloaderapi::GetProcAddress;
        use winapi::shared::minwindef::HMODULE;
        
        let c_name = CString::new(name)
            .map_err(|_| LoadError::FunctionNotFound {
                module: "unknown".to_string(),
                function: name.to_string(),
            })?;
        
        let addr = GetProcAddress(module as HMODULE, c_name.as_ptr());
        if addr.is_null() {
            return Err(LoadError::FunctionNotFound {
                module: "unknown".to_string(),
                function: name.to_string(),
            });
        }
        
        Ok(addr as *mut CVoid)
    }
    
    #[cfg(not(windows))]
    unsafe fn get_proc_address(_module: *mut CVoid, name: &str) -> Result<*mut CVoid, LoadError> {
        Err(LoadError::FunctionNotFound {
            module: "unknown".to_string(),
            function: name.to_string(),
        })
    }
    
    /// Wrapper pour GetProcAddress (par ordinal)
    #[cfg(windows)]
    unsafe fn get_proc_address_ordinal(
        module: *mut CVoid,
        ordinal: u16,
    ) -> Result<*mut CVoid, LoadError> {
        use winapi::um::libloaderapi::GetProcAddress;
        use winapi::shared::minwindef::HMODULE;
        
        let addr = GetProcAddress(module as HMODULE, ordinal as usize as *const i8);
        if addr.is_null() {
            return Err(LoadError::FunctionNotFound {
                module: "unknown".to_string(),
                function: format!("Ordinal#{}", ordinal),
            });
        }
        
        Ok(addr as *mut CVoid)
    }
    
    #[cfg(not(windows))]
    unsafe fn get_proc_address_ordinal(
        _module: *mut CVoid,
        ordinal: u16,
    ) -> Result<*mut CVoid, LoadError> {
        Err(LoadError::FunctionNotFound {
            module: "unknown".to_string(),
            function: format!("Ordinal#{}", ordinal),
        })
    }
}

impl Drop for PeLoader {
    fn drop(&mut self) {
        // Libérer la mémoire si elle n'a pas déjà été libérée
        if !self.base_address.is_null() {
            unsafe {
                // Essayer d'appeler DllMain(DETACH) si initialisé
                if self.initialized {
                    let _ = self.call_entry_point(DLL_PROCESS_DETACH);
                }
                Self::free_memory(self.base_address, self.image_size);
            }
        }
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_error_display() {
        let err = LoadError::AllocationFailed;
        assert!(err.to_string().contains("allocation"));
    }
    
    #[test]
    fn test_parse_error_conversion() {
        let parse_err = ParseError::InvalidDosHeader;
        let load_err: LoadError = parse_err.into();
        assert!(matches!(load_err, LoadError::Parse(_)));
    }
}
