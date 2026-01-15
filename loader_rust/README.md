# Reflective DLL Loader - Rust

## Compilation

```bash
# Prérequis
rustup install nightly
rustup default nightly
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

Binaire généré : `target/x86_64-pc-windows-gnu/release/reflective_dll_loader.exe`

## Vue d'ensemble

Loader Rust utilisant des **indirect syscalls** pour injecter une DLL reflective dans un processus cible (RuntimeBroker.exe).

### Techniques implémentées

- **Indirect Syscalls** : Bypass des hooks EDR en sautant directement dans ntdll après les hooks
- **PE Parsing** : Parse la DLL pour trouver l'export `ReflectiveLoader`
- **RVA to File Offset** : Convertit les adresses virtuelles en offsets fichier
- **Process Injection** : Injection dans un processus existant (RuntimeBroker.exe)
- **Memory Protection** : RW → RX (pas de RWX suspect)
- **Minimum Rights** : `0x002A` au lieu de `PROCESS_ALL_ACCESS`

### Flow d'injection

```
1. Parse evil.dll embarquée (include_bytes!)
2. Trouve l'export ReflectiveLoader (RVA → File Offset)
3. Énumère les processus → RuntimeBroker.exe
4. NtOpenProcess (droits minimum)
5. NtAllocateVirtualMemory (RW)
6. NtWriteVirtualMemory (copie DLL entière)
7. NtProtectVirtualMemory (RW → RX)
8. NtCreateThreadEx (démarre à base + offset_ReflectiveLoader)
9. NtClose (cleanup)
   ↓
Dans RuntimeBroker.exe :
   ReflectiveLoader() se charge lui-même → DllMain() → Payload
```

## Syscalls utilisés

| Syscall | Usage |
|---------|-------|
| `NtOpenProcess` | Ouvre le processus cible |
| `NtAllocateVirtualMemory` | Alloue mémoire RW distante |
| `NtWriteVirtualMemory` | Copie la DLL |
| `NtProtectVirtualMemory` | Change protection RW → RX |
| `NtCreateThreadEx` | Crée thread distant |
| `NtClose` | Cleanup |

## Pourquoi RuntimeBroker.exe ?

- Toujours présent (2-5 instances)
- Tourne en contexte USER (pas SYSTEM)
- Pas surveillé par les EDR (low profile)
- Si crash → Windows en respawn un autre
- Pas besoin de droits admin

**Note** : RuntimeBroker tourne en AppContainer/Low Integrity, donc les opérations disque (`CreateFileA`) peuvent échouer. Les opérations UI (`MessageBoxA`) fonctionnent.

## Problème d'alignement (`read_unaligned`)

Les données embarquées via `include_bytes!` ne sont pas garanties d'être alignées quand compilées avec MinGW cross-compilation. Le code utilise `std::ptr::read_unaligned` pour tous les accès aux structures PE afin d'éviter les crashes `misaligned pointer dereference`.

## Structure du projet

```
loader_rust/
├── Cargo.toml          # Dépendances (rust_syscalls, ntapi, winapi)
├── .cargo/config.toml  # Configuration cross-compilation
├── src/
│   └── main.rs         # Code principal (commenté en détail)
└── README.md
```

## Configuration

`.cargo/config.toml` :
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

`Cargo.toml` (optimisations) :
```toml
[profile.release]
opt-level = "z"      # Taille minimale
lto = true           # Link Time Optimization
panic = "abort"
strip = true
codegen-units = 1
```

## Ressources

- [rust_syscalls](https://github.com/janoglezcampos/rust_syscalls) - Indirect syscalls pour Rust
- [ReflectiveLdr](https://github.com/rokups/ReflectiveLdr) - Reflective DLL loader
- [Stephen Fewer](https://github.com/stephenfewer/ReflectiveDLLInjection) - Technique originale

## Disclaimer

**USAGE ÉDUCATIF UNIQUEMENT**

Ce projet est destiné à l'apprentissage de la sécurité offensive. Toute utilisation malveillante est strictement interdite.
