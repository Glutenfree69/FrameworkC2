//! Module PE (Portable Executable)
//!
//! Ce module contient:
//! - `structs`: Structures PE (DOS Header, NT Headers, Sections, etc.)
//! - `parser`: Parsing de fichiers PE
//! - `loader`: Chargement de DLL en mémoire (PE Loader complet)

pub mod loader;
pub mod parser;
pub mod structs;

pub use loader::{LoadError, PeLoader};
pub use parser::PeParser;
pub use structs::*;
