// test_nocrt.c
// DLL de test SANS CRT pour le PE Loader
// Compile: x86_64-w64-mingw32-gcc -shared -nostdlib -lkernel32 -luser32 -e DllMain -o test_nocrt.dll test_nocrt.c

// Définitions Windows minimales (pas de windows.h pour éviter le CRT)
typedef void* HANDLE;
typedef void* HMODULE;
typedef void* LPVOID;
typedef unsigned long DWORD;
typedef int BOOL;
typedef unsigned int UINT;

#define NULL ((void*)0)
#define TRUE 1
#define FALSE 0
#define DLL_PROCESS_ATTACH 1
#define DLL_PROCESS_DETACH 0
#define MB_OK 0x00000000
#define MB_ICONINFORMATION 0x00000040

#define WINAPI __stdcall
#define APIENTRY WINAPI

// Déclarations des fonctions Windows (importées de kernel32/user32)
__declspec(dllimport) HANDLE WINAPI CreateThread(
    void* lpThreadAttributes,
    unsigned long dwStackSize,
    DWORD (WINAPI *lpStartAddress)(LPVOID),
    LPVOID lpParameter,
    DWORD dwCreationFlags,
    DWORD* lpThreadId
);

__declspec(dllimport) int WINAPI MessageBoxA(
    HANDLE hWnd,
    const char* lpText,
    const char* lpCaption,
    UINT uType
);

// Thread function pour afficher la MessageBox
DWORD WINAPI ShowMessageThread(LPVOID lpParam) {
    (void)lpParam;
    MessageBoxA(NULL, 
        "PE Loader Success!\n\nLoaded WITHOUT CRT!",
        "Test DLL - No CRT",
        MB_OK | MB_ICONINFORMATION);
    return 0;
}

// Export pour test
__declspec(dllexport) int TestFunction(void) {
    return 42;
}

// DllMain - Entry point
BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved) {
    (void)hModule;
    (void)reserved;
    
    if (reason == DLL_PROCESS_ATTACH) {
        // Créer un thread pour la MessageBox
        CreateThread(NULL, 0, ShowMessageThread, NULL, 0, NULL);
    }
    
    return TRUE;
}
