//! Module PE (Portable Executable)
//!
//! Ce module contient:
//! - `structs`: Structures PE (DOS Header, NT Headers, Sections, etc.)
//! - `parser`: Parsing de fichiers PE
//! - `loader`: Chargement de DLL en mémoire (PE Loader complet)

pub mod structs;
pub mod parser;
pub mod loader;

pub use structs::*;
pub use parser::PeParser;
pub use loader::{PeLoader, LoadError};
