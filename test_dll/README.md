# Test DLLs pour PE Loader

Ce dossier contient des DLLs de test pour valider le fonctionnement du PE Loader.

## ⚠️ Important : Compilation sans CRT

Les DLLs chargées via le PE Loader custom **doivent être compilées sans le C Runtime (CRT)** de MinGW.

**Pourquoi ?** Le CRT de MinGW (`api-ms-win-crt-*.dll`) ne supporte pas d'être initialisé plusieurs fois dans le même process. Comme la beacon elle-même utilise le CRT, charger une autre DLL avec CRT cause un crash.

## DLLs disponibles

### `test_reloc.dll` ✅ (Recommandée)
DLL de test **sans CRT** mais **avec relocations**. Affiche une MessageBox.

Cette DLL peut être chargée à n'importe quelle adresse grâce à sa table de relocations.

```bash
# Compilation
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_reloc.dll test_reloc.c -lkernel32 -luser32

# Chiffrement XOR
python3 ../tools/xor_encrypt.py test_reloc.dll test_reloc.dll.enc 41
```

### `test_nocrt.dll` ✅
DLL de test **sans CRT** mais **sans relocations**. Affiche une MessageBox.

⚠️ Cette DLL ne peut être chargée que si son ImageBase préféré est disponible.

```bash
# Compilation
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_nocrt.dll test_nocrt.c -lkernel32 -luser32

# Chiffrement XOR
python3 ../tools/xor_encrypt.py test_nocrt.dll test_nocrt.dll.enc 41
```

### `test_minimal.dll` ✅
DLL ultra-minimaliste sans imports, utile pour tester le PE Loader lui-même.

```bash
# Compilation
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_minimal.dll test_minimal.c

# Chiffrement XOR  
python3 ../tools/xor_encrypt.py test_minimal.dll test_minimal.dll.enc 41
```

## Règles pour créer des DLLs compatibles

### ✅ À faire

1. **Compiler sans CRT** : Utiliser `-nostdlib -e DllMain`
2. **Importer uniquement des DLLs système** : `kernel32.dll`, `user32.dll`, `ntdll.dll`, etc.
3. **Définir les types Windows manuellement** ou utiliser des headers minimaux
4. **Ne pas bloquer dans DllMain** : Créer un thread pour les opérations longues
5. **Générer des relocations** si possible (voir ci-dessous)

### ❌ À éviter

1. **Ne pas utiliser** `#include <windows.h>` avec le CRT standard
2. **Ne pas utiliser** les fonctions CRT : `printf`, `malloc`, `memcpy`, etc.
3. **Ne pas compiler** avec les options par défaut de MinGW

## Comment générer des relocations sans CRT

Le code x64 utilise l'adressage RIP-relatif, donc le compilateur ne génère pas d'adresses absolues nécessitant des relocations par défaut. Pour forcer la génération de relocations :

**Utiliser des pointeurs de fonction globaux :**

```c
// Ces pointeurs créent des entrées de relocation
static void* g_function_table[4];

void StorePointers(void) {
    // Stocker des adresses dans des variables globales
    g_function_table[0] = (void*)&SomeFunction;
    g_function_table[1] = (void*)&AnotherFunction;
}
```

**Vérifier les relocations :**

```bash
x86_64-w64-mingw32-objdump -x file.dll | grep "Base Relocation"
# Devrait montrer: Entry 5 XXXXXXXX 0000000c Base Relocation Directory [.reloc]
# La taille (0000000c = 12 bytes) doit être > 0
```

## Exemple de DLL compatible

```c
// minimal_dll.c
typedef void* HANDLE;
typedef void* HMODULE;
typedef void* LPVOID;
typedef unsigned long DWORD;
typedef int BOOL;

#define NULL ((void*)0)
#define TRUE 1
#define DLL_PROCESS_ATTACH 1

#define WINAPI __stdcall
#define APIENTRY WINAPI

// Import depuis kernel32.dll
__declspec(dllimport) HANDLE WINAPI CreateThread(
    void*, unsigned long, DWORD (WINAPI*)(LPVOID), LPVOID, DWORD, DWORD*
);

// Ton code ici...
DWORD WINAPI WorkerThread(LPVOID param) {
    // Faire quelque chose...
    return 0;
}

BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved) {
    if (reason == DLL_PROCESS_ATTACH) {
        CreateThread(NULL, 0, WorkerThread, NULL, 0, NULL);
    }
    return TRUE;
}
```

Compilation :
```bash
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o minimal.dll minimal.c -lkernel32
```

## Utilisation avec la beacon

1. Compiler la DLL sans CRT
2. Chiffrer avec XOR : `python3 ../tools/xor_encrypt.py ma_dll.dll ma_dll.dll.enc 41`
3. Dans Discord : attacher le fichier `.enc` et envoyer `!loaddll`

## Protection contre le double chargement

Le PE Loader détecte automatiquement si une DLL identique est déjà chargée (via hash DJB2 du contenu). Si tu essaies de charger la même DLL deux fois, tu recevras :

```
Error: DLL already loaded at 0x... (hash: 0x...)
```
