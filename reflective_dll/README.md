# Reflective DLL Injection

## Compilation

```bash
x86_64-w64-mingw32-g++ -shared -o evil.dll evil.cpp ReflectiveLdr.cpp \
    -Wl,--export-all-symbols \
    -static \
    -luser32
```

## Vue d'ensemble

Implémentation d'une **Reflective DLL Injection** permettant de charger une DLL entièrement en mémoire sans toucher le disque ni utiliser `LoadLibrary`. La DLL contient son propre loader et se charge elle-même.

Basé sur [ReflectiveLdr](https://github.com/rokups/ReflectiveLdr) de rokups, dérivé du travail original de [Stephen Fewer](https://github.com/stephenfewer/ReflectiveDLLInjection).

## Structure

```
reflective_dll/
├── evil.cpp              # Payload (DllMain)
├── ReflectiveLdr.cpp     # Loader reflectif
├── ReflectiveLdr.h       # Header public
├── ReflectiveLdr_p.h     # Header privé (structures PE, PEB)
└── evil.dll              # DLL compilée
```

## Flow d'injection

```
LOADER RUST
  1. Parse le PE pour trouver l'export ReflectiveLoader
  2. Convertit le RVA en offset fichier
  3. NtAllocateVirtualMemory dans le process cible (RW)
  4. NtWriteVirtualMemory (copie la DLL brute)
  5. NtProtectVirtualMemory (RW → RX)
  6. NtCreateThreadEx → base + offset_ReflectiveLoader
     ↓
PROCESS CIBLE
  ReflectiveLoader():
    → Résout les APIs Windows via PEB
    → Alloue mémoire pour l'image mappée
    → Copie headers et sections
    → Résout imports et relocations
    → Appelle DllMain(DLL_PROCESS_ATTACH)
       ↓
    PAYLOAD EXÉCUTÉ
```

## Disclaimer

**USAGE ÉDUCATIF UNIQUEMENT**

Ce projet est destiné à l'apprentissage de la sécurité offensive et des techniques d'évasion. Toute utilisation malveillante est strictement interdite.
