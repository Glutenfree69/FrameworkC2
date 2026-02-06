# Pacpac — Packer C no-CRT pour Loader Rust

Packer éducatif écrit en C pur, sans CRT (C Runtime Library), pour Windows x64.
Le stub déchiffre et mappe manuellement en mémoire une DLL Rust sans qu'elle ne touche jamais le disque.

## Table des matières

- [Architecture globale](#architecture-globale)
- [Etape 1 — Squelette no-CRT](#etape-1--squelette-no-crt)
  - [Que fait le stub à cette étape ?](#que-fait-le-stub-à-cette-étape-)
  - [Structure des fichiers](#structure-des-fichiers)
  - [types.h — Typedefs Windows maison](#typesh--typedefs-windows-maison)
  - [hash.h — DJB2 Hashing](#hashh--djb2-hashing)
  - [peb.h — Structures PEB/LDR](#pebh--structures-pebldr)
  - [pe.h — Structures PE](#peh--structures-pe)
  - [peb.c — PEB Walking + Export Table Parsing](#pebc--peb-walking--export-table-parsing)
  - [entry.c — Point d'entrée et API_TABLE](#entryc--point-dentrée-et-api_table)
  - [utils.c — Fonctions utilitaires](#utilsc--fonctions-utilitaires)
- [Etape 2 — ChaCha20](#etape-2--chacha20)
  - [Pourquoi ChaCha20 ?](#pourquoi-chacha20-)
  - [L'algorithme](#lalgorithme)
  - [chacha20.h / chacha20.c](#chacha20h--chacha20c)
  - [Vérification avec les test vectors RFC 7539](#vérification-avec-les-test-vectors-rfc-7539)
- [Etape 3 — pack.py + LZ4](#etape-3--packpy--lz4)
  - [Le pipeline de packing](#le-pipeline-de-packing)
  - [pack.py — Le builder](#packpy--le-builder)
  - [payload.h — Le résultat](#payloadh--le-résultat)
  - [Pourquoi LZ4 ?](#pourquoi-lz4-)
  - [Usage](#usage)
- [Etape 4 — PE Loader + LZ4 décompresseur](#etape-4--pe-loader--lz4-décompresseur)
  - [lz4.c — Décompresseur LZ4 block](#lz4c--décompresseur-lz4-block)
  - [loader.c — Manual mapper (inspiré de Fatmike-GH/PELoader)](#loaderc--manual-mapper-inspiré-de-fatmike-ghpeloader)
  - [Ordre des opérations (critique pour Rust)](#ordre-des-opérations-critique-pour-rust)
  - [TLS — Thread Local Storage (Static Index Stealing)](#tls--thread-local-storage-static-index-stealing)
  - [Exception handling (x64 SEH)](#exception-handling-x64-seh)
  - [Delay imports](#delay-imports)
  - [entry.c — Le pipeline complet](#entryc--le-pipeline-complet)
- [Etape 5 — Fix TLS : Static Index Stealing](#etape-5--fix-tls--static-index-stealing)
  - [Le crash 0xC000041D](#le-crash-0xc000041d)
  - [Pourquoi TlsAlloc() ne marche pas pour Rust](#pourquoi-tlsalloc-ne-marche-pas-pour-rust)
  - [La solution : section .tls statique dans le stub](#la-solution--section-tls-statique-dans-le-stub)
  - [tls.c — Les symboles TLS](#tlsc--les-symboles-tls)
  - [loader.c — init_tls() reecrit](#loaderc--init_tls-reecrit)
  - [Fichiers modifies](#fichiers-modifies)
- [Compilation](#compilation)
- [Vérifier le binaire](#vérifier-le-binaire)
- [Tester et debugger sur Windows](#tester-et-debugger-sur-windows)
- [Roadmap](#roadmap)
- [Références](#références)

---

## Architecture globale

```
Build time (pack.py) :                          Runtime (stub.exe) :

  loader.dll (Rust, ~1.1 MB)                     stub.exe (~6.5 KB, seul fichier sur disque)
       │                                              │
       ▼                                              ├─ 1. PEB Walking : résout les API Windows
  [LZ4 compress]                                      │      sans aucun import visible (IAT vide)
       │                                              │
       ▼                                              ├─ 2. Déchiffre le blob ChaCha20
  [ChaCha20 encrypt]                                  │
       │                                              ├─ 3. Décompresse LZ4
       ▼                                              │
  payload.h (blob chiffré)                            ├─ 4. Manual map la DLL en mémoire
       │                                              │      (sections, relocs, TLS, imports, SEH)
       └─── compilé dans stub.exe                     │
                                                      └─ 5. Appelle TLS callbacks + DllMain + Run()
                                                           Le loader Rust prend le relais
```

**Pourquoi ?** Le loader Rust est gros (~1.1 MB) et serait facilement analysé par les AV/EDR s'il
touchait le disque. En le chiffrant dans un petit stub C, seul le stub (avec un IAT vide et aucune
string suspecte) est visible sur disque. Le loader n'existe qu'en mémoire.

---

## Etape 1 — Squelette no-CRT

### Que fait le stub à cette étape ?

L'étape 1 pose les fondations. Le binaire fait **une seule chose** : résoudre `ExitProcess` depuis
`kernel32.dll` via PEB walking et l'appeler avec le code 0. C'est le "Hello World" du no-CRT.

Le flow complet :

```
Entry()                          Point d'entrée custom (pas de main)
  │
  ├─ resolve_apis()              Résout 8 API Windows
  │     │
  │     ├─ peb_resolve_api()     Pour chaque API :
  │     │     │
  │     │     ├─ peb_find_module()    1. Lit le PEB via gs:[0x60]
  │     │     │     │                 2. Parcourt InLoadOrderModuleList
  │     │     │     │                 3. Compare le hash DJB2 du nom de module
  │     │     │     └─ → kernel32.dll base address
  │     │     │
  │     │     └─ peb_find_export()    4. Parse le PE header du module
  │     │           │                 5. Parcourt la table d'exports
  │     │           │                 6. Compare le hash DJB2 du nom d'export
  │     │           └─ → adresse de la fonction
  │     │
  │     └─ Remplit API_TABLE avec les 8 adresses
  │
  └─ api.ExitProcess(0)          Appelle ExitProcess, le process se termine
```

### Structure des fichiers

```
Pacpac/
├── include/
│   ├── types.h        Windows typedefs (DWORD, PVOID, HANDLE, etc.)
│   ├── hash.h         DJB2 hash functions + constantes pré-calculées
│   ├── peb.h          Structures PEB/LDR + prototypes de résolution
│   ├── pe.h           Structures PE (DOS, NT, sections, exports, imports, relocs, TLS, exceptions)
│   ├── chacha20.h     ChaCha20 stream cipher (RFC 7539)
│   ├── lz4.h          LZ4 block decompressor
│   ├── loader.h       PE manual mapper + API_TABLE definition
│   └── payload.h      (généré par pack.py — payload chiffré + compressé)
├── src/
│   ├── entry.c        Entry point, resolve_apis(), pipeline decrypt→decompress→map→Run()
│   ├── peb.c          PEB walking, export table parsing
│   ├── utils.c        my_memcpy, my_memset, my_strlen, my_strcmp
│   ├── chacha20.c     ChaCha20 RFC 7539
│   ├── lz4.c          LZ4 block decompressor (~70 lignes)
│   ├── tls.c          Section TLS statique pour le stub (static index stealing)
│   └── loader.c       PE manual mapper (relocations, TLS, imports, delay imports, SEH, protections)
├── payload/           (DLL à packer)
├── .venv/             (Python venv avec lz4)
├── pack.py            Packer script : LZ4 compress → ChaCha20 encrypt → payload.h
├── Makefile           Cross-compilation MinGW depuis macOS
└── build.bat          Compilation MSVC sur Windows
```

---

### types.h — Typedefs Windows maison

**Problème** : On ne peut pas inclure `<windows.h>` — il dépend du CRT et tirerait des centaines
de headers. On redéfinit uniquement ce dont on a besoin.

**Points clés** :

```c
typedef unsigned short      WCHAR;   // Pas wchar_t ! Il vient de <stddef.h> qu'on n'inclut pas
typedef unsigned long long  ULONG_PTR; // Toujours 8 bytes sur x64
typedef void*               PVOID;
typedef unsigned long long  FARPROC;   // Pointeur de fonction sur x64
```

- `WCHAR` = `unsigned short` (16 bits). Sur Windows, les strings internes sont en UTF-16LE.
  `wchar_t` n'est pas un type intrinsèque en C (contrairement au C++), il est défini dans
  `<stddef.h>` qu'on n'inclut pas.

- `ULONG_PTR` et `SIZE_T` = `unsigned long long` sur x64 (8 bytes). Attention : sur x86 ce
  serait `unsigned long` (4 bytes). Notre code est x64-only.

- On définit aussi les constantes `MEM_COMMIT`, `PAGE_READWRITE`, etc. pour VirtualAlloc/VirtualProtect.

---

### hash.h — DJB2 Hashing

**Pourquoi hasher ?** Si on mettait les noms d'API en clair dans le binaire ("GetProcAddress",
"VirtualAlloc", etc.), n'importe quel outil d'analyse statique (strings, FLOSS, CFF Explorer)
les verrait immédiatement. En utilisant des hash, on n'a que des constantes numériques.

**L'algorithme DJB2** (par Daniel J. Bernstein) :

```c
DWORD hash = 5381;
while ((c = *str++))
    hash = hash * 33 + c;    // équivalent à (hash << 5) + hash + c
```

C'est un hash très simple, rapide, avec peu de collisions sur les noms courts (API Windows).
Il n'est PAS cryptographique — juste suffisant pour éviter les strings en clair.

**Deux variantes** :

| Variante | Usage | Particularité |
|----------|-------|---------------|
| `djb2_hash_a(char*)` | Noms d'exports (ASCII) | Sensible à la casse |
| `djb2_hash_w(WCHAR*)` | Noms de modules (UTF-16) | **Case-insensitive** (toLower avant hash) |

La variante wide est case-insensitive parce que Windows peut charger les modules avec n'importe
quelle casse (`KERNEL32.DLL`, `kernel32.dll`, `Kernel32.Dll`...).

**Hashes pré-calculés** :

```c
#define HASH_KERNEL32_DLL          0x7040EE75  // djb2_hash_w(L"kernel32.dll")
#define HASH_GETPROCADDRESS        0xCF31BB1F  // djb2_hash_a("GetProcAddress")
#define HASH_EXITPROCESS           0xB769339E  // djb2_hash_a("ExitProcess")
#define HASH_FLUSHINSTRUCTIONCACHE 0xB7DCEDDD  // djb2_hash_a("FlushInstructionCache")
#define HASH_RTLADDFUNCTIONTABLE   0xBDB9F1AE  // djb2_hash_a("RtlAddFunctionTable")
// ...
```

**Vérification avec Python** (TOUJOURS vérifier, ne jamais deviner les hash) :

```python
def djb2_a(s):
    h = 5381
    for c in s:
        h = ((h << 5) + h + ord(c)) & 0xFFFFFFFF
    return h

def djb2_w(s):
    h = 5381
    for c in s:
        h = ((h << 5) + h + ord(c.lower())) & 0xFFFFFFFF
    return h

print(f"GetProcAddress = 0x{djb2_a('GetProcAddress'):08X}")
print(f"kernel32.dll   = 0x{djb2_w('kernel32.dll'):08X}")
```

---

### peb.h — Structures PEB/LDR

**Le PEB (Process Environment Block)** est une structure maintenue par Windows pour chaque
processus. Elle contient, entre autres, la liste de tous les modules (DLL) chargés.

**Hiérarchie des structures** :

```
TEB (Thread Environment Block)
 └─ offset 0x60 sur x64 → PEB
     └─ Ldr (PEB_LDR_DATA)
         └─ InLoadOrderModuleList (LIST_ENTRY, doubly-linked list)
              ├─ Entry 1: ntdll.dll
              ├─ Entry 2: kernel32.dll    ← on cherche celui-ci
              ├─ Entry 3: kernelbase.dll
              └─ ...
```

Chaque entrée de la liste est un `LDR_DATA_TABLE_ENTRY` qui contient :
- `DllBase` — adresse de base du module en mémoire
- `BaseDllName` — nom du module en UNICODE_STRING (UTF-16)
- `SizeOfImage` — taille de l'image mappée

**LIST_ENTRY** est une liste doublement chaînée circulaire Windows :

```
     ┌──────────────────────────────────┐
     │                                  │
     ▼                                  │
  ┌──────┐    ┌──────┐    ┌──────┐      │
  │ head │───▶│ ent1 │───▶│ ent2 │───── ┘
  │      │◀───│      │◀───│      │
  └──────┘    └──────┘    └──────┘
```

On parcourt avec `curr = head->Flink` et on s'arrête quand `curr == head` (on a fait le tour).

**Accès au PEB sur x64** : Le TEB est pointé par le segment register `GS`. Le PEB est à
l'offset 0x60 du TEB :

```c
PEB* peb;
__asm__ volatile ("mov %%gs:0x60, %0" : "=r" (peb));
```

Pourquoi `GS` et pas `FS` ? Sur x86, le TEB est à `FS:[0x18]` et le PEB à `FS:[0x30]`.
Sur x64, Microsoft a changé pour `GS` — le TEB est à `GS:[0x30]` et le PEB à `GS:[0x60]`.

---

### pe.h — Structures PE

Ce header définit toutes les structures du format PE (Portable Executable) nécessaires pour
le manual mapping complet.

**Structure d'un PE en mémoire** :

```
Base address
 │
 ├─ IMAGE_DOS_HEADER         (commence par "MZ")
 │    └─ e_lfanew ──────────── offset vers NT headers
 │
 ├─ IMAGE_NT_HEADERS64       (commence par "PE\0\0")
 │    ├─ FileHeader           (machine, nombre de sections, etc.)
 │    └─ OptionalHeader       (ImageBase, SizeOfImage, DataDirectory[])
 │         └─ DataDirectory[0]  ── Export Directory
 │         └─ DataDirectory[1]  ── Import Directory
 │         └─ DataDirectory[3]  ── Exception Directory (RUNTIME_FUNCTION, x64 SEH)
 │         └─ DataDirectory[5]  ── Base Relocation
 │         └─ DataDirectory[9]  ── TLS Directory
 │         └─ DataDirectory[13] ── Delay Import Directory
 │
 ├─ IMAGE_SECTION_HEADER[]   (une par section : .text, .rdata, .data, ...)
 │
 ├─ .text section             (code exécutable)
 ├─ .rdata section            (données read-only, export table, import table)
 ├─ .data section             (données read-write)
 └─ ...
```

**RVA (Relative Virtual Address)** : un offset relatif à la base du module en mémoire.
Pour obtenir une adresse absolue : `address = module_base + rva`.

---

### peb.c — PEB Walking + Export Table Parsing

C'est le coeur de la résolution dynamique. Trois fonctions :

#### `get_peb()` (static)

Lit le PEB via inline assembly GCC. Un seul `mov` depuis `gs:0x60`.

#### `peb_find_module(DWORD module_hash)`

1. Récupère le PEB
2. Accède à `PEB->Ldr->InLoadOrderModuleList`
3. Pour chaque module : hash le `BaseDllName`, compare, retourne `DllBase` si match

#### `peb_find_export(PVOID module_base, DWORD function_hash)`

1. Valide DOS + NT headers
2. Parse l'export directory (`DataDirectory[0]`)
3. Parcourt les noms d'exports, compare les hash
4. Gère les **forwarded exports** (détecte mais ne résout pas)

#### `peb_resolve_api(DWORD module_hash, DWORD function_hash)`

Simple wrapper : `find_module` + `find_export`.

---

### entry.c — Point d'entrée et API_TABLE

#### Le concept d'API_TABLE

Plutôt que de résoudre les API à chaque appel, on les résout une fois au démarrage dans une
struct globale :

```c
typedef struct _API_TABLE {
    fn_ExitProcess              ExitProcess;
    fn_VirtualAlloc             VirtualAlloc;
    fn_VirtualProtect           VirtualProtect;
    fn_VirtualFree              VirtualFree;
    fn_GetProcAddress           GetProcAddress;
    fn_LoadLibraryA             LoadLibraryA;
    fn_FlushInstructionCache    FlushInstructionCache;
    fn_RtlAddFunctionTable      RtlAddFunctionTable;
} API_TABLE;
```

| API | Usage |
|-----|-------|
| `GetProcAddress` | Résoudre les imports de la DLL mappée |
| `LoadLibraryA` | Charger les DLL dépendantes de la DLL mappée |
| `VirtualAlloc` | Allouer la mémoire pour l'image + buffers |
| `VirtualProtect` | Protections mémoire des sections (RW → RX, etc.) |
| `VirtualFree` | Libérer la mémoire |
| `ExitProcess` | Terminer proprement (ou sur erreur avec code) |
| `FlushInstructionCache` | Invalider le cache instructions après écriture de code |
| `RtlAddFunctionTable` | Enregistrer les tables SEH x64 (critical pour Rust panics) |

> **Note :** `TlsAlloc` a été retiré de l'API_TABLE. L'ancien code allouait un index TLS
> **dynamique** via `TlsAlloc()`, ce qui causait un crash `0xC000041D` avec les DLL Rust.
> La nouvelle approche utilise le **static index stealing** (voir section TLS ci-dessous).

---

### utils.c — Fonctions utilitaires

Réimplémentations basiques de fonctions libc sans CRT :

- `my_memcpy` — copie n bytes de src vers dst
- `my_memset` — remplit n bytes avec une valeur
- `my_strlen` — longueur d'une string ASCII
- `my_strcmp` — compare deux strings ASCII

---

## Etape 2 — ChaCha20

### Pourquoi ChaCha20 ?

On a besoin de chiffrer le payload embarqué pour qu'il ne soit pas analysable statiquement.
ChaCha20 est un **stream cipher** :

- Simple à implémenter (~100 lignes C, pas de tables S-Box comme AES)
- Rapide en software (pas besoin d'instructions AES-NI)
- Standard (RFC 7539)
- **XOR-based** : encrypt == decrypt (même fonction pour les deux)

### L'algorithme

ChaCha20 opère sur un **state** de 16 mots de 32 bits (64 octets), organisé en matrice 4x4 :

```
"expa"  "nd 3"  "2-by"  "te k"     ← constantes (expand 32-byte k)
 key0    key1    key2    key3       ← clé (8 mots = 256 bits)
 key4    key5    key6    key7
counter nonce0  nonce1  nonce2      ← counter (32 bits) + nonce (96 bits)
```

L'opération de base est le **quarter round** sur 4 mots (a, b, c, d) :

```c
a += b; d ^= a; d = ROTL32(d, 16);
c += d; b ^= c; b = ROTL32(b, 12);
a += b; d ^= a; d = ROTL32(d, 8);
c += d; b ^= c; b = ROTL32(b, 7);
```

Un **block** = 20 rounds (10 double-rounds : colonnes puis diagonales) + feedforward (add original state).
Produit 64 octets de **keystream** qu'on XOR avec les données.

### chacha20.h / chacha20.c

```c
void chacha20_crypt(BYTE* data, DWORD data_len,
                    const BYTE key[32], const BYTE nonce[12],
                    DWORD counter);
```

- `key` : 32 octets (256 bits) — généré aléatoirement par pack.py
- `nonce` : 12 octets (96 bits) — généré aléatoirement par pack.py
- `counter` : 0 (début du stream)
- Comme c'est du XOR, appeler la même fonction avec les mêmes paramètres déchiffre

### Vérification avec les test vectors RFC 7539

Notre implémentation Python (identique au C) passe les 3 tests :

```
RFC 7539 s2.3.2 keystream: True     ← keystream block match
RFC 7539 s2.4.2 encrypt:   True     ← encryption match
Round-trip decrypt:         True     ← decrypt(encrypt(x)) == x
```

Le code Python de vérification est dans pack.py (mêmes fonctions `_rotl32`, `_quarter_round`,
`_chacha20_block`, `chacha20_crypt`).

---

## Etape 3 — pack.py + LZ4

### Le pipeline de packing

```
loader.dll (raw bytes)
     │
     ▼
 LZ4 compress (block mode, pas frame)
     │  ratio typique : 50-70% sur un PE Rust
     ▼
 ChaCha20 encrypt (clé + nonce aléatoires)
     │
     ▼
 payload.h (header C avec les données + métadonnées)
     │
     └── #include dans entry.c → compilé dans stub.exe
```

### pack.py — Le builder

Le packer est un script Python qui s'exécute **sur la machine de l'attaquant** (build time),
pas sur la cible. Il réimplémente ChaCha20 de manière identique au C pour le chiffrement.

**Features** :
- LZ4 compression en block mode (via `lz4.block`, compatible avec notre décompresseur C)
- ChaCha20 RFC 7539 (implémentation Python identique au C)
- Clé/nonce aléatoires par défaut, ou fixables via `--key` / `--nonce`
- Vérification round-trip automatique (decrypt + decompress == original)
- Génère un header C propre avec les tailles et métadonnées

### payload.h — Le résultat

```c
#define PAYLOAD_ORIGINAL_SIZE   123456   // taille du PE original
#define PAYLOAD_COMPRESSED_SIZE 78000    // après LZ4
#define PAYLOAD_ENCRYPTED_SIZE  78000    // == compressed (ChaCha20 ne change pas la taille)

static const BYTE payload_key[32]   = { 0x79, 0xe7, ... };   // clé ChaCha20
static const BYTE payload_nonce[12] = { 0xbd, 0x29, ... };   // nonce ChaCha20
static const BYTE payload_data[]    = { 0x35, 0x22, ... };   // données chiffrées
```

### Pourquoi LZ4 ?

| Critère | LZ4 | LZMA/zlib |
|---------|-----|-----------|
| Décompresseur C | ~70 lignes | ~2000+ lignes |
| Vitesse décompression | Très rapide | Lent |
| Ratio compression | Moyen (50-70%) | Meilleur (30-50%) |
| Complexité no-CRT | Trivial | Cauchemar |

Pour un packer, la taille du décompresseur dans le stub est plus importante que le ratio
de compression. LZ4 est le meilleur compromis.

### Usage

```bash
# Setup (une seule fois)
python3 -m venv .venv
.venv/bin/pip install lz4

# Packer une DLL
.venv/bin/python3 pack.py loader.dll

# Avec clé/nonce fixes (pour le debug)
.venv/bin/python3 pack.py loader.dll --key 0000...00 --nonce 000000000000

# Puis compiler le stub
make
```

---

## Etape 4 — PE Loader + LZ4 décompresseur

### lz4.c — Décompresseur LZ4 block

Le format LZ4 block est un flux de **séquences** :

```
[token] [lit_len_ext*] [literals] [offset] [match_len_ext*]
  │          │             │          │          │
  │          │             │          │          └─ octets supplémentaires si match_len == 15
  │          │             │          └─ 2 bytes LE : backward offset dans l'output
  │          │             └─ lit_len bytes copiés directement
  │          └─ octets supplémentaires si lit_len == 15
  └─ 1 byte : high nibble = lit_len, low nibble = match_len (base = 4)
```

**Détails du token** :
- High nibble (bits 7-4) = longueur des littéraux (0-15)
- Low nibble (bits 3-0) = longueur du match - 4 (0-15, donc match = 4 à 19)
- Si une valeur est 15, on lit des octets supplémentaires jusqu'à en trouver un < 255

**Match** : copie `match_len` bytes depuis `output - offset` (backward reference).
Peut chevaucher (overlap) → copie byte-par-byte obligatoire, pas memcpy.

La dernière séquence du flux n'a pas de match (que des littéraux).

```c
int lz4_decompress(const BYTE* src, DWORD src_len,
                   BYTE* dst, DWORD dst_cap);
// Retourne le nombre d'octets décompressés, ou -1 si erreur
```

---

### loader.c — Manual mapper (inspiré de Fatmike-GH/PELoader)

Le manual mapper charge un PE (DLL) en mémoire sans passer par `LoadLibrary`.
Notre implémentation est adaptée de [Fatmike-GH/PELoader](https://github.com/Fatmike-GH/PELoader)
pour le support Rust (TLS, exceptions).

### Ordre des opérations (critique pour Rust)

L'ordre est calqué sur celui de Fatmike (`PELoader::LoadPE`). Changer l'ordre
peut casser le chargement, surtout pour Rust.

```
pe_load(raw_pe, api)
  │
  ├─ 1. map_sections()          VirtualAlloc + copie headers + sections
  │     cf. Fatmike: PELoader::MapSections
  │
  ├─ 2. process_relocations()   Patcher les adresses absolues (delta rebase)
  │     cf. Fatmike: PELoader::ApplyRelocations
  │     Supporte : DIR64 (x64), HIGHLOW (x86 compat), ABSOLUTE (padding)
  │
  ├─ 3. init_tls()              Initialiser le Thread Local Storage
  │     Approche : "Static Index Stealing" (à la Fatmike)
  │     → Lit _tls_index du stub (assigné par Windows au boot)
  │     → Écrit cet index dans le TLS directory de la DLL
  │     → Copie des données TLS initiales
  │     → Inscription dans TEB.ThreadLocalStoragePointer[index]
  │
  ├─ 4. resolve_imports()       Charger les DLL + patcher l'IAT
  │     cf. Fatmike: PELoader::ResolveImports
  │     Gère : imports par nom ET par ordinal
  │
  ├─ 5. resolve_delay_imports() Idem pour les delay-loaded imports
  │     cf. Fatmike: PELoader::ResolveDelayImports
  │
  ├─ 6. setup_exceptions()      Enregistrer la table RUNTIME_FUNCTION
  │     cf. Fatmike: PELoader::SetupExceptionHandling
  │     → RtlAddFunctionTable() pour le SEH x64
  │
  ├─ 7. set_protections()       VirtualProtect par section
  │     cf. Fatmike: PELoader::ApplySectionMemoryProtection
  │
  ├─ 8. FlushInstructionCache   Invalider le cache après écriture
  │
  ├─ 9. execute_tls_callbacks() Appeler les callbacks TLS
  │     cf. Fatmike: TlsResolver::ExecuteCallbacks
  │     → DLL_PROCESS_ATTACH
  │
  └─ 10. DllMain()              Appeler l'entry point de la DLL
         → DLL_PROCESS_ATTACH
```

### TLS — Thread Local Storage (Static Index Stealing)

**Pourquoi c'est critique pour Rust ?** Le runtime Rust initialise ses thread-locals (panic handler,
allocator, etc.) via les TLS callbacks. Un loader qui ne gère pas le TLS ne peut pas charger un
binaire Rust qui utilise `std`.

#### Le probleme avec `TlsAlloc()` (ancienne approche)

L'ancienne version utilisait `TlsAlloc()` pour obtenir un index TLS **dynamique**. Ca marchait
pour du C classique, mais **crash avec Rust** (code `0xC000041D`).

Pourquoi ? Le runtime Rust (et MSVC CRT) s'attend a un index TLS **statique**, assigne par le
Windows loader au demarrage du processus. Quand le PE a une section `.tls`, Windows :

1. Lit le `IMAGE_TLS_DIRECTORY` dans le PE header
2. Assigne un index statique (typiquement 0)
3. Ecrit cet index a `AddressOfIndex`
4. Alloue et initialise les donnees TLS dans le TEB

Un index dynamique (venant de `TlsAlloc`) n'est pas au bon endroit dans le TEB —
`TlsGetValue()`/`TlsSetValue()` fonctionnent, mais l'acces direct via `gs:[0x58]` (que le
code Rust genere) ne trouve pas les bonnes donnees.

#### La solution : Static Index Stealing

L'idee est de donner une section `.tls` **au stub lui-meme**, pour que Windows lui assigne
un index statique au demarrage. Ensuite, on "vole" cet index pour la DLL chargee manuellement.

C'est l'approche utilisee par [Fatmike-GH/PELoader](https://github.com/Fatmike-GH/PELoader)
dans son `TlsResolver`.

#### `tls.c` — Section TLS statique pour le stub

Le fichier `src/tls.c` reproduit l'approche de `mingw-w64/crt/tlssup.c` en C pur, compatible
`-nostdlib` :

```c
#define SECTION(x) __attribute__((section(x)))

SECTION(".tls")     char *_tls_start = NULL;
SECTION(".tls$ZZZ") char *_tls_end   = NULL;

ULONG _tls_index = 0;  /* Ecrit par Windows au demarrage */

SECTION(".rdata$T")
const IMAGE_TLS_DIRECTORY64 _tls_used = {
    (ULONGLONG) &_tls_start,
    (ULONGLONG) &_tls_end,
    (ULONGLONG) &_tls_index,
    (ULONGLONG) 0,    /* Pas de callbacks pour le stub */
    0, 0
};
```

**Comment ca marche sans CRT ?**

Le linker MinGW cherche le symbole `_tls_used` pour peupler `IMAGE_DATA_DIRECTORY[9]`
(TLS Directory) dans le PE header. C'est un mecanisme du **linker**, pas du CRT. Les
`__attribute__((section(...)))` placent les donnees dans les bonnes sections PE :

| Symbole | Section | Role |
|---------|---------|------|
| `_tls_start` | `.tls` | Debut de la zone TLS statique |
| `_tls_end` | `.tls$ZZZ` | Fin de la zone TLS (le `$ZZZ` force l'ordre apres `.tls`) |
| `_tls_index` | `.bss` (implicite) | Index TLS — ecrit par Windows au load time |
| `_tls_used` | `.rdata$T` | Le `IMAGE_TLS_DIRECTORY64` — lu par le linker |

Le `$` dans les noms de section est une convention MinGW/MSVC : les sections avec le meme
prefixe sont mergees par le linker, triees par suffixe. Donc `.tls` + `.tls$ZZZ` forment
une zone contigue, et `_tls_start`/`_tls_end` en delimitent les bornes.

#### `init_tls()` — Le vol d'index

```
1. stub_tls_index = _tls_index      ← Lire l'index statique (assigne par Windows)
2. DLL.TLS_DIRECTORY.AddressOfIndex = stub_tls_index   ← Ecrire dans la DLL
3. Allouer un bloc memoire pour les donnees TLS de la DLL
4. Copier StartAddressOfRawData → EndAddressOfRawData dans le bloc
5. TEB.ThreadLocalStoragePointer[stub_tls_index] = bloc  ← Inscrire dans le TEB
```

L'index est typiquement 0 (premier module avec du TLS), ce qui est exactement ce que le
code Rust genere quand il accede a ses thread-locals.

#### Layout du TEB x64 (offsets pertinents)

```
TEB (gs:[0x30])
 ├─ +0x30  Self              ← pointeur vers le TEB lui-meme
 ├─ +0x58  ThreadLocalStoragePointer  ← PVOID* (tableau de pointeurs TLS)
 └─ +0x60  ProcessEnvironmentBlock    ← PEB*
```

Le code Rust accede aux TLS via quelque chose comme :
```asm
mov rax, gs:[0x58]         ; ThreadLocalStoragePointer
mov rax, [rax + index*8]   ; pointeur vers le bloc TLS du module
mov rcx, [rax + offset]    ; variable thread-local specifique
```

Si l'index ne correspond pas a celui assigne par le loader Windows, `rax + index*8` pointe
dans le vide → crash `0xC000041D` (exception non geree dans un callback).

#### Verification de la section TLS dans le PE

```bash
x86_64-w64-mingw32-objdump -x stub.exe | grep -i -A 10 "TLS"
# Doit afficher :
#   Entry 9 <addr> 00000028 Thread Storage Directory [.tls]
#   ...
#   .tls   <size>  <addr>  ...  CONTENTS, ALLOC, LOAD, DATA
```

#### Limitation

On ne gere pas `DLL_THREAD_ATTACH/DETACH` pour les nouveaux threads.
Fatmike resout ca avec un `TlsCallbackProxy` enregistre via `.CRT$XLB` (specifique MSVC).
Notre stub no-CRT ne peut pas utiliser cette technique. Suffisant pour un usage single-thread
(stub → `Run()` → le loader Rust gere ses propres threads).

### Exception handling (x64 SEH)

Sur x64, Windows utilise une table de `RUNTIME_FUNCTION` pour le stack unwinding.
Sans enregistrer cette table, tout `panic!()` Rust ou exception C++ crashe immédiatement
au lieu de dérouler proprement la stack.

```c
// cf. Fatmike: PELoader::SetupExceptionHandling
RUNTIME_FUNCTION* table = (base + exception_dir.VirtualAddress);
DWORD count = exception_dir.Size / sizeof(RUNTIME_FUNCTION);
RtlAddFunctionTable(table, count, (DWORD64)base);
```

### Delay imports

Les delay-loaded imports sont résolus paresseusement par Windows normalement, mais
puisqu'on fait du manual mapping, on les résout nous-mêmes immédiatement. Le format
utilise `IMAGE_DELAYLOAD_DESCRIPTOR` au lieu de `IMAGE_IMPORT_DESCRIPTOR`, avec des
champs `ImportNameTableRVA` et `ImportAddressTableRVA`.

---

### entry.c — Le pipeline complet

```c
void Entry(void) {
    /* 1. Résoudre les API via PEB walking */
    resolve_apis();  // 8 API depuis kernel32.dll

    /* 2. Déchiffrer (ChaCha20) */
    decrypted = VirtualAlloc(PAYLOAD_ENCRYPTED_SIZE);
    memcpy(decrypted, payload_data);     // copier le blob chiffré
    chacha20_crypt(decrypted, ...);      // déchiffrement in-place (XOR)

    /* 3. Décompresser (LZ4) */
    decompressed = VirtualAlloc(PAYLOAD_ORIGINAL_SIZE);
    lz4_decompress(decrypted, decompressed);

    /* 4. Manual map la DLL */
    mapped = pe_load(decompressed, &api);  // les 10 étapes du loader

    /* 5. Trouver et appeler Run() */
    run = peb_find_export(mapped, HASH_RUN);  // cherche par hash DJB2
    run();

    ExitProcess(0);
}
```

Les codes d'erreur `ExitProcess` :
- **1** = échec allocation buffer déchiffrement
- **2** = échec allocation buffer décompression
- **3** = échec LZ4 décompression
- **4** = échec PE manual mapping

---

## Etape 5 — Fix TLS : Static Index Stealing

### Le crash 0xC000041D

Apres avoir implemente le PE loader (etape 4), le stub crash avec le code `0xC000041D`
(STATUS_FATAL_USER_CALLBACK_EXCEPTION) en executant les TLS callbacks de la DLL Rust.

Le probleme venait de `init_tls()` qui utilisait `TlsAlloc()` pour allouer un index TLS
**dynamique**. Le runtime Rust attend un index **statique** assigne par le Windows loader.

### Pourquoi TlsAlloc() ne marche pas pour Rust

Quand un PE a une section `.tls`, le Windows loader fait ceci au demarrage :

```
1. Lit IMAGE_DATA_DIRECTORY[9] → IMAGE_TLS_DIRECTORY
2. Assigne un index statique (typiquement 0 pour le 1er module)
3. Ecrit l'index dans *AddressOfIndex
4. Alloue un bloc TLS dans le TEB (ThreadLocalStoragePointer)
5. Copie les donnees TLS initiales dans ce bloc
```

Le code machine genere par Rust pour acceder aux thread-locals ressemble a :

```asm
mov rax, gs:[0x58]         ; TEB.ThreadLocalStoragePointer (tableau de PVOID)
mov rax, [rax + 0*8]       ; index 0 → pointeur vers le bloc TLS du module
mov rcx, [rax + 0x10]      ; variable thread-local a l'offset 0x10
```

L'index est **hard-code** dans le binaire Rust au moment de la compilation. Quand on
utilise `TlsAlloc()`, on obtient un index dynamique (ex: 5), mais le code Rust continue
d'acceder a l'index 0 → il lit les mauvaises donnees ou un pointeur NULL → crash.

### La solution : section .tls statique dans le stub

L'idee est simple :

1. **Donner une section `.tls` au stub** pour que Windows lui assigne un index statique
   au demarrage (typiquement 0)
2. **Voler cet index** pour la DLL chargee manuellement

C'est l'approche "Static Index Stealing" utilisee par
[Fatmike-GH/PELoader](https://github.com/Fatmike-GH/PELoader). La difference avec
l'original de Fatmike (C++ / MSVC) est qu'on reproduit la meme chose en **C pur** avec
des `__attribute__((section(...)))`, compatible `-nostdlib` et MinGW.

```
Au demarrage du process (automatique par Windows) :
  Windows lit _tls_used dans stub.exe
  → assigne _tls_index = 0
  → alloue TEB.ThreadLocalStoragePointer[0]

Au moment du pe_load() :
  init_tls() lit _tls_index (= 0)
  → ecrit 0 dans DLL.TLS_DIRECTORY.AddressOfIndex
  → alloue un nouveau bloc avec les donnees TLS de la DLL
  → ecrase TEB.ThreadLocalStoragePointer[0] avec le nouveau bloc
  → le code Rust accede a gs:[0x58] → index 0 → ses donnees TLS ✓
```

### tls.c — Les symboles TLS

Le fichier `src/tls.c` definit 4 symboles que le linker MinGW utilise pour creer un
TLS directory dans le PE :

```c
SECTION(".tls")     char *_tls_start = NULL;   // Debut zone TLS
SECTION(".tls$ZZZ") char *_tls_end   = NULL;   // Fin zone TLS

ULONG _tls_index = 0;  // Index — ecrit par Windows au boot

SECTION(".rdata$T")
const IMAGE_TLS_DIRECTORY64 _tls_used = {       // Lu par le linker
    &_tls_start, &_tls_end, &_tls_index,
    0,  // pas de callbacks
    0, 0
};
```

**Mecanisme du linker** : le linker MinGW cherche le symbole `_tls_used` pour remplir
`IMAGE_DATA_DIRECTORY[9]` dans le PE header. Ce n'est PAS un mecanisme du CRT — c'est
le linker qui le fait, d'ou la compatibilite avec `-nostdlib`.

Les suffixes `$` dans les noms de section (`.tls$ZZZ`, `.rdata$T`) sont une convention
PE/COFF : les sections avec le meme prefixe sont fusionnees par le linker, triees par
suffixe alphabetique. Ca garantit que `_tls_start` est avant `_tls_end` dans la section
finale.

### loader.c — init_tls() reecrit

L'ancienne version :
```c
tls_index = api->TlsAlloc();        // ← dynamique, crash Rust
*(DWORD*)(tls->AddressOfIndex) = tls_index;
```

La nouvelle version :
```c
extern ULONG _tls_index;             // defini dans tls.c, ecrit par Windows
stub_tls_index = _tls_index;         // ← statique, assigne par le loader
*(DWORD*)(tls->AddressOfIndex) = stub_tls_index;
```

En plus du changement d'index, les appels `init_tls()` et `execute_tls_callbacks()`
qui etaient desactives (commentes) dans `pe_load()` ont ete reactives.

### Fichiers modifies

| Fichier | Modification |
|---------|-------------|
| `src/tls.c` | **Nouveau** — section `.tls` statique + `_tls_used` |
| `src/loader.c` | `init_tls()` reecrit (lit `_tls_index` au lieu de `TlsAlloc`) + appels reactives |
| `include/loader.h` | Retire `fn_TlsAlloc` et champ `TlsAlloc` de `API_TABLE` |
| `include/hash.h` | Retire `HASH_TLSALLOC` |
| `src/entry.c` | Retire la resolution de `TlsAlloc` via PEB walking |
| `Makefile` | Ajoute `src/tls.c` aux sources |

---

## Compilation

### Depuis macOS/Linux (MinGW cross-compilation)

```bash
# Installer MinGW (macOS)
brew install mingw-w64

# Packer une DLL puis compiler
.venv/bin/python3 pack.py loader.dll
make

# Résultat : stub.exe (~6.5 KB)
```

### Depuis Windows (MSVC)

```cmd
REM Ouvrir "x64 Native Tools Command Prompt for VS"
build.bat
```

### Options de compilation expliquées

**MinGW (Makefile)** :

| Flag | Effet |
|------|-------|
| `-Os` | Optimiser pour la taille (pas la vitesse) |
| `-nostdlib` | Ne pas linker le CRT ni la libc |
| `-fno-stack-protector` | Pas de stack canary (nécessite `__stack_chk_fail` du CRT) |
| `-fno-ident` | Ne pas embarquer la version du compilateur dans le binaire |
| `-fpack-struct=8` | Alignement des struct à 8 bytes (convention Windows x64) |
| `-Wl,-eEntry` | Point d'entrée = `Entry` (pas `_start` ni `main`) |
| `-Wl,--no-seh` | Pas de SEH dans le stub lui-même |
| `-Wl,-s` | Stripper les symboles |
| `-Wl,--subsystem,console` | Subsystem console (pas GUI) |

---

## Vérifier le binaire

### 1. Taille

```bash
ls -la stub.exe
# Attendu : ~6.5 KB (sans payload réel, plus gros avec un vrai PE Rust)
```

### 2. IAT (Import Address Table)

```bash
x86_64-w64-mingw32-objdump -x stub.exe | grep -A 30 'Import'
```

La table d'imports devrait être vide (toutes les entrées à 0).

### 3. Strings

```bash
strings stub.exe
```

On ne devrait voir que le DOS stub et les noms de sections. Aucun nom d'API, aucun nom de DLL.

---

## Tester et debugger sur Windows

### Option 1 : x64dbg (recommandé)

[x64dbg](https://x64dbg.com/) est un debugger open-source parfait pour l'analyse PE.

**Walkthrough** :

1. **Breakpoint sur Entry** → le debugger s'arrête automatiquement
2. **Step dans resolve_apis** → vérifier que les 8 API sont résolues (non-NULL dans la struct)
3. **Step dans chacha20_crypt** → vérifier que le buffer est bien déchiffré
4. **Step dans lz4_decompress** → vérifier que le résultat ressemble à un PE (commence par `MZ`)
5. **Step dans pe_load** → observer l'allocation mémoire, la copie des sections
6. **Vérifier les TLS** → observer `_tls_index` (assigné par Windows), `gs:[0x58]` être modifié
7. **Step dans Run()** → le code Rust s'exécute

**Codes de sortie** :

| Code | Signification |
|------|---------------|
| 0 | Succès (Run() terminé) |
| 1 | Échec allocation decrypt buffer |
| 2 | Échec allocation decompress buffer |
| 3 | Échec LZ4 decompress |
| 4 | Échec PE load |
| `int3` | Échec resolve_apis (breakpoint) |

### Option 2 : WinDbg

```
windbg -g stub.exe
```

Commandes utiles :

```
!peb                          Afficher le PEB
!dlls                         Lister tous les modules chargés
bp stub!Entry                 Breakpoint sur Entry
dps @rsp L20                  Dump de la stack
db <addr> L100                Dump hex d'une adresse
```

### Option 3 : Sysinternals / PE-bear / FLOSS

| Outil | Usage |
|-------|-------|
| **Process Monitor** | Logger les opérations du process (filtre sur `stub.exe`) |
| **PE-bear** | Analyse statique PE (sections, imports, exports) |
| **FLOSS** | Extraction de strings obfusquées — si rien ne sort, c'est bon |

---

## Roadmap

| Etape | Description | Status |
|-------|-------------|--------|
| 1 | Squelette no-CRT + PEB walking + ExitProcess | **DONE** (4 KB, IAT vide) |
| 2 | ChaCha20 (RFC 7539) | **DONE** (+500 bytes, vérif test vectors) |
| 3 | pack.py : LZ4 compress + ChaCha20 encrypt → payload.h | **DONE** (round-trip vérif) |
| 4 | PE Loader : manual mapping + LZ4 decompressor + TLS + SEH + delay imports | **DONE** (6.5 KB, inspiré Fatmike) |
| 5 | Fix TLS : Static Index Stealing (remplacement de TlsAlloc par section .tls statique) | **DONE** (fix crash 0xC000041D) |
| 6 | Intégration / test avec une vraie DLL Rust | TODO |

---

## Références

- [Microsoft PE Format Specification](https://docs.microsoft.com/en-us/windows/win32/debug/pe-format)
- [RFC 7539 — ChaCha20 and Poly1305](https://tools.ietf.org/html/rfc7539)
- [Fatmike-GH/PELoader](https://github.com/Fatmike-GH/PELoader) — PE loading + TLS pour Rust (inspiration directe)
- [janoglezcampos/c_syscalls](https://github.com/janoglezcampos/c_syscalls) — PEB walking en C
- [LZ4 Block Format](https://github.com/lz4/lz4/blob/dev/doc/lz4_Block_format.md) — Spec du format LZ4
- [x64dbg](https://x64dbg.com/) — Debugger x64 open-source
- [PE-bear](https://github.com/hasherezade/pe-bear) — Analyseur PE par hasherezade
- [FLOSS](https://github.com/mandiant/flare-floss) — String extraction avancée par Mandiant
