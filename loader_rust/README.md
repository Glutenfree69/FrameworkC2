# 🦀 Rust Syscall Loader

Un loader éducatif utilisant les **indirect syscalls** en Rust pour l'injection de code sur Windows.

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
7. [Reflective DLL Injection (v5)](#-reflective-dll-injection-v5)
8. [Installation & Compilation](#-installation--compilation)
9. [Génération du Shellcode](#-génération-du-shellcode)
10. [Techniques Avancées](#-techniques-avancées)
11. [Troubleshooting](#-troubleshooting)
12. [Ressources](#-ressources)

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
│   │              │  │             │  │  CODE)     │    │
│   │  Stack 1     │  │  Stack 2    │  │  Stack 3   │    │
│   └──────────────┘  └─────────────┘  └────────────┘    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

Quand tu fais `NtCreateThreadEx` dans RuntimeBroker, tu crées un **nouveau thread** qui :
- Partage toute la mémoire du process (donc peut lire/écrire ton code)
- A son propre stack et registres
- Démarre à l'adresse que tu lui donnes

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

Chaque thread a un **quantum** (temps d'exécution). Quand il expire, le scheduler passe au suivant. C'est pour ça que ton code peut s'exécuter "en parallèle" du code légitime de RuntimeBroker.

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
│   │  (UWP)      │    │  (autre)    │    │ (TON CODE)  │    │
│   │             │    │             │    │             │    │
│   │  Running    │    │  Waiting    │    │  Running ✓  │    │
│   └─────────────┘    └─────────────┘    └─────────────┘    │
│                                              │              │
│                                              ▼              │
│                                      Exécute payload       │
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

Quand le code termine:
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

#### Flux avec Indirect Syscalls (v4+)

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
| `NtOpenProcess` | Ouvre le processus cible avec droits minimum |
| `NtAllocateVirtualMemory` | Alloue de la mémoire RW dans le processus cible |
| `NtWriteVirtualMemory` | Écrit le code dans la mémoire distante |
| `NtProtectVirtualMemory` | Change la protection mémoire RW → RX |
| `NtCreateThreadEx` | Crée un thread distant pour exécuter le code |
| `NtClose` | Ferme les handles (process et thread) |

---

## 🔍 Comment les EDR Détectent les Menaces

### Modèle de Détection Multi-Couches

```
1. SIGNATURE STATIQUE
   └── Scan du fichier sur disque
   └── Pattern matching (shellcode connu, strings suspectes)
   └── ❌ Bypassé par XOR encryption / DLL custom

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
| Reflective DLL | Pas de LoadLibrary, DLL en mémoire pure |
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

---

## 🏗️ Architecture du Loader

### Structure du Projet

```
loader_rust/
├── Cargo.toml          # Dépendances et configuration
├── .cargo/
│   └── config.toml     # Configuration cross-compilation
├── src/
│   └── main.rs         # Code principal avec syscalls + PE parsing
└── README.md           # Ce fichier
```

---

## 🔄 Flow d'Exécution Détaillé

### v4 : Shellcode Injection

```
LOADER:
  ├── FindProcess("RuntimeBroker.exe") → PID
  ├── NtOpenProcess → process_handle
  ├── NtAllocateVirtualMemory (RW)
  ├── XOR decrypt shellcode
  ├── NtWriteVirtualMemory (copie shellcode)
  ├── NtProtectVirtualMemory (RW → RX)
  ├── NtCreateThreadEx → thread démarre au début du buffer
  ├── NtClose (cleanup)
  └── exit
```

### v5 : Reflective DLL Injection

```
LOADER:
  ├── Parse PE de evil.dll embarquée
  │   ├── DOS Header → NT Headers
  │   ├── Export Directory → trouve "ReflectiveLoader"
  │   └── Convertit RVA → File Offset
  ├── FindProcess("RuntimeBroker.exe") → PID
  ├── NtOpenProcess → process_handle
  ├── NtAllocateVirtualMemory (RW, taille = DLL entière)
  ├── NtWriteVirtualMemory (copie DLL brute)
  ├── NtProtectVirtualMemory (RW → RX)
  ├── NtCreateThreadEx → thread démarre à (base + ReflectiveLoader_offset)
  │                       avec base_address en lpParameter
  ├── NtClose (cleanup)
  └── exit

        DANS RUNTIMEBROKER (ReflectiveLoader):
          ├── Reçoit base_address via lpParameter
          ├── Ldr() : trouve kernel32/ntdll via PEB
          ├── MapImageAndExecute():
          │   ├── Alloue mémoire propre
          │   ├── Copie sections (.text, .data, etc.)
          │   ├── Résout imports
          │   ├── Applique relocations
          │   └── Appelle DllMain(DLL_PROCESS_ATTACH)
          └── Ta payload s'exécute !
```

---

## 🪞 Reflective DLL Injection (v5)

### Concept

Au lieu d'injecter du shellcode brut, on injecte une **DLL entière** qui contient son propre loader. La DLL sait comment se charger en mémoire sans l'aide de `LoadLibrary`.

### Avantages vs Shellcode

| Aspect | Shellcode | Reflective DLL |
|--------|-----------|----------------|
| Taille payload | Limité | Illimité |
| Langage | ASM | C/C++ |
| Debugging | Difficile | Facile |
| Fonctionnalités | Basique | Complètes (imports, etc.) |
| Extensibilité | Faible | Haute |

### PE Parsing dans le Loader Rust

Le loader doit trouver l'offset de `ReflectiveLoader` dans la DLL :

```
evil.dll (fichier brut):
┌─────────────────────────────────────────────────────────┐
│ DOS Header                                              │
│   └── e_lfanew → offset vers NT Headers                │
├─────────────────────────────────────────────────────────┤
│ NT Headers                                              │
│   └── OptionalHeader.DataDirectories[0] → Export Dir   │
├─────────────────────────────────────────────────────────┤
│ Section Headers                                         │
│   └── .text: VirtualAddress=0x1000, PointerToRawData=0x600 │
├─────────────────────────────────────────────────────────┤
│ .text section (code)                                    │
│   └── ReflectiveLoader() quelque part ici              │
├─────────────────────────────────────────────────────────┤
│ Export Directory                                        │
│   └── "ReflectiveLoader" → RVA 0x3075                  │
└─────────────────────────────────────────────────────────┘
```

### RVA vs File Offset

**Problème critique** : On injecte la DLL **brute** (fichier), pas mappée en mémoire.

```
RVA (Relative Virtual Address) = adresse quand la DLL est mappée
File Offset = position dans le fichier sur disque

Ce sont deux choses DIFFÉRENTES !
```

**Conversion** :
```
ReflectiveLoader RVA: 0x3075
Section .text: RVA=0x1000, FileOffset=0x600

Offset dans section = 0x3075 - 0x1000 = 0x2075
File offset = 0x600 + 0x2075 = 0x2675
```

Le loader Rust fait cette conversion automatiquement via `rva_to_offset()`.

### Documentation Complète

Voir **[../reflective_dll/README.md](../reflective_dll/README.md)** pour :
- Explication détaillée du code C++ (ReflectiveLdr)
- Modifications apportées pour compatibilité MinGW
- Problèmes de compilation et solutions
- Guide de debugging

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
# Release build (recommandé)
cargo build --release --target x86_64-pc-windows-gnu
```

Le binaire sera généré dans :
```
target/x86_64-pc-windows-gnu/release/calc_loader.exe
```

### Configuration Cross-Compilation

`.cargo/config.toml` :

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

## 🔧 Génération du Shellcode (v4)

> **Note** : Pour v5 (Reflective DLL), voir [../reflective_dll/README.md](../reflective_dll/README.md)

### Commande de Génération

```bash
# ⚠️ IMPORTANT: Utiliser EXITFUNC=thread !
msfvenom -p windows/x64/exec CMD=calc.exe EXITFUNC=thread -f rust
```

### EXITFUNC Expliqué

| Option | Fonction appelée | Effet |
|--------|------------------|-------|
| `process` | `ExitProcess()` | ❌ Tue tout le process hôte |
| `thread` | `ExitThread()` | ✅ Tue juste le thread injecté |
| `seh` | Exception SEH | Exit via exception handler |
| `none` | Rien | Le shellcode continue/crash |

---

## 🎯 Techniques Avancées

| Technique | Description | Difficulté |
|-----------|-------------|------------|
| Indirect Syscalls | Saute dans ntdll après le hook | ⭐⭐ |
| Reflective DLL | DLL qui se charge elle-même | ⭐⭐⭐ |
| Hell's Gate | Récupère SSN dynamiquement | ⭐⭐⭐ |
| Halo's Gate | Hell's Gate + gère les hooks | ⭐⭐⭐⭐ |
| Sleep Obfuscation | Chiffre pendant les sleep | ⭐⭐⭐ |
| Module Stomping | Écrase une DLL légitime | ⭐⭐⭐⭐ |
| Process Hollowing | Remplace le code d'un process suspendu | ⭐⭐⭐ |
| Thread Hijacking | Détourne un thread existant | ⭐⭐⭐⭐ |

---

## 🐛 Troubleshooting

### Erreur de compilation "linker not found"

```bash
which x86_64-w64-mingw32-gcc
# Si pas trouvé:
brew install mingw-w64
```

### Erreur "requires nightly"

```bash
rustup default nightly
```

### NtOpenProcess failed: 0xC0000022 (STATUS_ACCESS_DENIED)

Le process cible tourne sous SYSTEM. Utilise `RuntimeBroker.exe` (tourne toujours sous ton user).

### RuntimeBroker.exe not found

Ouvre les Settings Windows ou une app UWP pour en spawner un.

### v5: DllMain jamais appelé

1. Vérifie les imports de la DLL : `objdump -p evil.dll | grep "DLL Name"`
2. Doit montrer seulement `KERNEL32.dll` (et `api-ms-win-crt-*`)
3. Si `libgcc` ou `libstdc++` présents → recompile avec `-static`

### v5: Process crash immédiatement

1. Vérifie que `GetPEB()` utilise l'inline ASM MinGW (pas `__readgsqword`)
2. Vérifie que `ReflectiveLoader` utilise `lpParameter` comme base address
3. Voir [../reflective_dll/README.md](../reflective_dll/README.md) pour les modifications requises

---

## 📊 Versions

| Version | Techniques | Description |
|---------|------------|-------------|
| v1 | Direct syscalls | Shellcode dans le loader |
| v2 | + XOR + RW→RX | Encryption + meilleure protection mémoire |
| v3 | + Process injection (notepad) | Spawn notepad + injection |
| v4 | + Indirect syscalls + RuntimeBroker | Injection dans process existant |
| **v5** | **+ Reflective DLL Injection** | **DLL complète avec PE parsing** |

---

## 📖 Ressources

- [rust_syscalls](https://github.com/janoglezcampos/rust_syscalls) - Bibliothèque syscalls Rust
- [ReflectiveLdr](https://github.com/rokups/ReflectiveLdr) - Base du reflective loader
- [Stephen Fewer - ReflectiveDLLInjection](https://github.com/stephenfewer/ReflectiveDLLInjection) - Original
- [Red Team Notes](https://www.ired.team/) - Tutoriels techniques offensives

---

## ⚖️ Disclaimer

Ce projet est fourni uniquement à des fins éducatives. L'auteur n'est pas responsable de toute utilisation malveillante. Testez uniquement sur des systèmes dont vous êtes propriétaire ou sur lesquels vous avez une autorisation explicite.

---

**Happy Hacking! 🦀**
