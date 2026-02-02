// test_minimal.c
// DLL ultra-minimaliste SANS IMPORT pour tester le PE Loader
// Compile: x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_minimal.dll test_minimal.c

// Pas d'include, pas de dépendances

// DllMain minimaliste
__attribute__((dllexport))
int __stdcall DllMain(void* hModule, unsigned int reason, void* reserved) {
    (void)hModule;
    (void)reserved;
    
    if (reason == 1) {  // DLL_PROCESS_ATTACH
        // Ne fait RIEN - juste retourne TRUE
        // Si ça crash, le problème est dans le loader lui-même
    }
    
    return 1;  // TRUE
}

// Export bidon pour test
__attribute__((dllexport))
int TestFunction(void) {
    return 42;
}
