// build.rs - Compile C code for UACME integration
//
// This script compiles the UAC bypass C code from UACME project
// and links it with the Rust library.

fn main() {
    // Only compile on Windows target
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    if target_os == "windows" {
        println!("cargo:rerun-if-changed=src/uacme/c/uac_bypass.c");
        println!("cargo:rerun-if-changed=src/uacme/c/uac_bypass.h");
        
        cc::Build::new()
            .file("src/uacme/c/uac_bypass.c")
            .include("src/uacme/c")
            // Windows-specific flags
            .define("WIN32_LEAN_AND_MEAN", None)
            .define("UNICODE", None)
            .define("_UNICODE", None)
            // Optimization and security
            .opt_level(2)
            .warnings(false)  // Suppress warnings from UACME code
            // Compile as static library
            .compile("uac_bypass");
        
        // Link required Windows libraries for COM
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=oleaut32");
        println!("cargo:rustc-link-lib=shell32");
    }
}
