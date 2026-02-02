//! Module de syscalls indirects pour le loader
//!
//! Ce module implémente des indirect syscalls pour Windows x64.
//! Les syscalls sont résolus dynamiquement au runtime via parsing du PEB.
//!
//! Modules:
//! - `obf`: Hash DJB2 pour obfusquer les noms de fonctions (compile-time)
//! - `resolve`: Résolution des SSN (System Service Numbers) via PEB/ntdll
//! - `syscall`: Macro et code assembleur pour exécuter les syscalls indirects

pub mod obf;
pub mod resolve;
pub mod syscall;
