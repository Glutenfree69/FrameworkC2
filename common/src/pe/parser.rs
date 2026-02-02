//! PE Parser - Extraction des informations d'un fichier PE
//!
//! Ce module parse un fichier PE (DLL/EXE) depuis des bytes bruts
//! et extrait toutes les informations nécessaires pour le chargement.

use super::structs::*;
use std::ptr;

/// Erreurs de parsing PE
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Fichier trop petit
    TooSmall,
    /// Magic DOS invalide
    InvalidDosHeader,
    /// Signature PE invalide
    InvalidPeSignature,
    /// Magic PE invalide (ni PE32 ni PE64)
    InvalidPeMagic,
    /// Architecture non supportée
    UnsupportedArchitecture,
    /// Section invalide
    InvalidSection(String),
    /// Data directory invalide
    InvalidDataDirectory(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::TooSmall => write!(f, "File too small to be a valid PE"),
            ParseError::InvalidDosHeader => write!(f, "Invalid DOS header (no MZ signature)"),
            ParseError::InvalidPeSignature => write!(f, "Invalid PE signature"),
            ParseError::InvalidPeMagic => write!(f, "Invalid PE magic (not PE32/PE64)"),
            ParseError::UnsupportedArchitecture => write!(f, "Unsupported architecture (x64 only)"),
            ParseError::InvalidSection(s) => write!(f, "Invalid section: {}", s),
            ParseError::InvalidDataDirectory(s) => write!(f, "Invalid data directory: {}", s),
        }
    }
}

/// Informations parsées d'un fichier PE
#[derive(Debug)]
pub struct PeParser<'a> {
    /// Bytes bruts du fichier PE
    pub raw: &'a [u8],
    /// DOS Header
    pub dos_header: DosHeader,
    /// NT Headers (avec Optional Header)
    pub nt_headers: NtHeaders64,
    /// Liste des sections
    pub sections: Vec<SectionHeader>,
    /// Offset du premier section header dans le fichier
    pub sections_offset: usize,
}

impl<'a> PeParser<'a> {
    /// Parse un fichier PE depuis des bytes bruts
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        // Vérifier la taille minimale
        if data.len() < std::mem::size_of::<DosHeader>() {
            return Err(ParseError::TooSmall);
        }

        // Lire le DOS Header
        let dos_header: DosHeader =
            unsafe { ptr::read_unaligned(data.as_ptr() as *const DosHeader) };

        // Vérifier le magic MZ
        if dos_header.e_magic != DOS_MAGIC {
            return Err(ParseError::InvalidDosHeader);
        }

        // Vérifier que e_lfanew pointe vers quelque chose de valide
        let pe_offset = dos_header.e_lfanew as usize;
        if pe_offset + std::mem::size_of::<NtHeaders64>() > data.len() {
            return Err(ParseError::TooSmall);
        }

        // Lire les NT Headers
        let nt_headers: NtHeaders64 =
            unsafe { ptr::read_unaligned(data.as_ptr().add(pe_offset) as *const NtHeaders64) };

        // Vérifier la signature PE
        if nt_headers.signature != PE_SIGNATURE {
            return Err(ParseError::InvalidPeSignature);
        }

        // Vérifier le magic (PE64 uniquement pour l'instant)
        if nt_headers.optional_header.magic != PE64_MAGIC {
            return Err(ParseError::InvalidPeMagic);
        }

        // Vérifier l'architecture (x64 uniquement)
        if nt_headers.file_header.machine != IMAGE_FILE_MACHINE_AMD64 {
            return Err(ParseError::UnsupportedArchitecture);
        }

        // Calculer l'offset des section headers
        let sections_offset = pe_offset
            + 4  // Signature
            + std::mem::size_of::<FileHeader>()
            + nt_headers.file_header.size_of_optional_header as usize;

        // Lire les sections
        let num_sections = nt_headers.file_header.number_of_sections as usize;
        let mut sections = Vec::with_capacity(num_sections);

        for i in 0..num_sections {
            let section_ptr =
                data.as_ptr() as usize + sections_offset + i * std::mem::size_of::<SectionHeader>();

            if section_ptr + std::mem::size_of::<SectionHeader>()
                > data.as_ptr() as usize + data.len()
            {
                return Err(ParseError::InvalidSection(format!(
                    "Section {} out of bounds",
                    i
                )));
            }

            let section: SectionHeader =
                unsafe { ptr::read_unaligned(section_ptr as *const SectionHeader) };
            sections.push(section);
        }

        Ok(Self {
            raw: data,
            dos_header,
            nt_headers,
            sections,
            sections_offset,
        })
    }

    /// Retourne l'adresse de base préférée
    pub fn image_base(&self) -> u64 {
        self.nt_headers.optional_header.image_base
    }

    /// Retourne la taille de l'image en mémoire
    pub fn size_of_image(&self) -> u32 {
        self.nt_headers.optional_header.size_of_image
    }

    /// Retourne la taille des headers
    pub fn size_of_headers(&self) -> u32 {
        self.nt_headers.optional_header.size_of_headers
    }

    /// Retourne le RVA du point d'entrée
    pub fn entry_point_rva(&self) -> u32 {
        self.nt_headers.optional_header.address_of_entry_point
    }

    /// Retourne l'alignement des sections
    pub fn section_alignment(&self) -> u32 {
        self.nt_headers.optional_header.section_alignment
    }

    /// Retourne l'alignement fichier
    pub fn file_alignment(&self) -> u32 {
        self.nt_headers.optional_header.file_alignment
    }

    /// Retourne un Data Directory par son index
    pub fn data_directory(&self, index: usize) -> Option<&DataDirectory> {
        if index < 16 {
            Some(&self.nt_headers.optional_header.data_directories[index])
        } else {
            None
        }
    }

    /// Vérifie si l'image a une table de relocation
    pub fn has_relocations(&self) -> bool {
        let reloc_dir = self.data_directory(IMAGE_DIRECTORY_ENTRY_BASERELOC);
        reloc_dir
            .map(|d| d.virtual_address != 0 && d.size != 0)
            .unwrap_or(false)
    }

    /// Vérifie si l'image a des imports
    pub fn has_imports(&self) -> bool {
        let import_dir = self.data_directory(IMAGE_DIRECTORY_ENTRY_IMPORT);
        import_dir
            .map(|d| d.virtual_address != 0 && d.size != 0)
            .unwrap_or(false)
    }

    /// Vérifie si l'image a des TLS callbacks
    pub fn has_tls(&self) -> bool {
        let tls_dir = self.data_directory(IMAGE_DIRECTORY_ENTRY_TLS);
        tls_dir
            .map(|d| d.virtual_address != 0 && d.size != 0)
            .unwrap_or(false)
    }

    /// Convertit un RVA en offset fichier
    pub fn rva_to_offset(&self, rva: u32) -> Option<u32> {
        // Si le RVA est dans les headers
        if rva < self.size_of_headers() {
            return Some(rva);
        }

        // Chercher dans les sections
        for section in &self.sections {
            let section_start = section.virtual_address;
            let section_end = section_start + section.virtual_size;

            if rva >= section_start && rva < section_end {
                let offset_in_section = rva - section_start;
                return Some(section.pointer_to_raw_data + offset_in_section);
            }
        }

        None
    }

    /// Lit des bytes à un RVA donné
    pub fn read_at_rva(&self, rva: u32, size: usize) -> Option<&[u8]> {
        let offset = self.rva_to_offset(rva)? as usize;
        if offset + size <= self.raw.len() {
            Some(&self.raw[offset..offset + size])
        } else {
            None
        }
    }

    /// Lit une structure à un RVA donné
    pub fn read_struct_at_rva<T: Copy>(&self, rva: u32) -> Option<T> {
        let offset = self.rva_to_offset(rva)? as usize;
        if offset + std::mem::size_of::<T>() <= self.raw.len() {
            Some(unsafe { ptr::read_unaligned(self.raw.as_ptr().add(offset) as *const T) })
        } else {
            None
        }
    }

    /// Lit une C string (null-terminated) à un RVA donné
    pub fn read_cstr_at_rva(&self, rva: u32) -> Option<&str> {
        let offset = self.rva_to_offset(rva)? as usize;
        if offset >= self.raw.len() {
            return None;
        }

        // Trouver le null terminator
        let bytes = &self.raw[offset..];
        let end = bytes.iter().position(|&b| b == 0)?;

        std::str::from_utf8(&bytes[..end]).ok()
    }

    /// Itère sur les Import Descriptors
    pub fn iter_imports(&self) -> ImportIterator<'_, 'a> {
        let import_dir = self.data_directory(IMAGE_DIRECTORY_ENTRY_IMPORT);
        let rva = import_dir.map(|d| d.virtual_address).unwrap_or(0);

        ImportIterator {
            parser: self,
            current_rva: rva,
        }
    }

    /// Itère sur les blocs de relocation
    pub fn iter_relocations(&self) -> RelocationBlockIterator<'_, 'a> {
        let reloc_dir = self.data_directory(IMAGE_DIRECTORY_ENTRY_BASERELOC);
        let (rva, size) = reloc_dir
            .map(|d| (d.virtual_address, d.size))
            .unwrap_or((0, 0));

        RelocationBlockIterator {
            parser: self,
            current_rva: rva,
            end_rva: rva + size,
        }
    }
}

/// Itérateur sur les Import Descriptors
pub struct ImportIterator<'p, 'a> {
    parser: &'p PeParser<'a>,
    current_rva: u32,
}

impl<'p, 'a: 'p> Iterator for ImportIterator<'p, 'a> {
    type Item = (ImportDescriptor, &'p str); // (descriptor, dll_name)

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_rva == 0 {
            return None;
        }

        let desc: ImportDescriptor = self.parser.read_struct_at_rva(self.current_rva)?;

        // Le terminator a tout à zéro
        if desc.is_null() {
            return None;
        }

        // Lire le nom de la DLL
        let dll_name = self.parser.read_cstr_at_rva(desc.name)?;

        // Avancer au prochain descriptor
        self.current_rva += std::mem::size_of::<ImportDescriptor>() as u32;

        Some((desc, dll_name))
    }
}

/// Itérateur sur les blocs de relocation
pub struct RelocationBlockIterator<'p, 'a> {
    parser: &'p PeParser<'a>,
    current_rva: u32,
    end_rva: u32,
}

impl<'p, 'a> Iterator for RelocationBlockIterator<'p, 'a> {
    type Item = (BaseRelocationBlock, Vec<RelocationEntry>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_rva == 0 || self.current_rva >= self.end_rva {
            return None;
        }

        let block: BaseRelocationBlock = self.parser.read_struct_at_rva(self.current_rva)?;

        // Vérifier la validité du bloc
        if block.size_of_block == 0 || block.size_of_block < 8 {
            return None;
        }

        // Lire les entries
        let entry_count = block.entry_count();
        let mut entries = Vec::with_capacity(entry_count);

        let entries_rva = self.current_rva + 8; // Après le header du bloc
        for i in 0..entry_count {
            let entry_rva = entries_rva + (i * 2) as u32;
            if let Some(entry) = self.parser.read_struct_at_rva::<RelocationEntry>(entry_rva) {
                entries.push(entry);
            }
        }

        // Avancer au prochain bloc
        self.current_rva += block.size_of_block;

        Some((block, entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::InvalidDosHeader;
        assert!(err.to_string().contains("DOS"));
    }

    #[test]
    fn test_parse_too_small() {
        let data = [0u8; 10];
        let result = PeParser::parse(&data);
        assert!(matches!(result, Err(ParseError::TooSmall)));
    }

    #[test]
    fn test_parse_invalid_dos() {
        let mut data = [0u8; 100];
        data[0] = 0x00; // Pas "MZ"
        let result = PeParser::parse(&data);
        assert!(matches!(result, Err(ParseError::InvalidDosHeader)));
    }
}
