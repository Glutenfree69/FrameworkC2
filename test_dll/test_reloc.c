// test_reloc.c
// DLL de test SANS CRT mais AVEC relocations
// Compile: x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_reloc.dll test_reloc.c -lkernel32 -luser32

// Définitions Windows minimales
typedef void* HANDLE;
typedef void* HMODULE;
typedef void* LPVOID;
typedef unsigned long DWORD;
typedef unsigned long SIZE_T;
typedef int BOOL;
typedef unsigned int UINT;
typedef DWORD* LPDWORD;

#define NULL ((void*)0)
#define TRUE 1
#define FALSE 0
#define DLL_PROCESS_ATTACH 1
#define MB_OK 0x00000000
#define MB_ICONINFORMATION 0x00000040

#define WINAPI __stdcall
#define APIENTRY WINAPI

// Thread proc type
typedef DWORD (WINAPI *LPTHREAD_START_ROUTINE)(LPVOID);

// Import depuis kernel32/user32
__declspec(dllimport) HANDLE WINAPI CreateThread(
    void* lpThreadAttributes,
    SIZE_T dwStackSize,
    LPTHREAD_START_ROUTINE lpStartAddress,
    LPVOID lpParameter,
    DWORD dwCreationFlags,
    LPDWORD lpThreadId
);

__declspec(dllimport) int WINAPI MessageBoxA(
    HANDLE hWnd,
    const char* lpText,
    const char* lpCaption,
    UINT uType
);

// ===== FORCER DES RELOCATIONS =====
// Pointeurs de fonction - créent des entrées de relocation sur x64
typedef HANDLE (WINAPI *CreateThreadFunc)(void*, SIZE_T, LPTHREAD_START_ROUTINE, LPVOID, DWORD, LPDWORD);
typedef int (WINAPI *MsgBoxFunc)(HANDLE, const char*, const char*, UINT);

// Variables globales avec pointeurs
static CreateThreadFunc g_pCreateThread = NULL;
static MsgBoxFunc g_pMessageBox = NULL;

// Données globales
static const char* g_title = "Test DLL with Relocations";
static const char* g_message = "PE Loader Success!\n\nLoaded WITHOUT CRT but WITH relocations!";

// Table de pointeurs pour forcer plus de relocations
static void* g_vtable[8];

// Thread function
DWORD WINAPI ShowMessageThread(LPVOID lpParam) {
    (void)lpParam;
    MessageBoxA(NULL, g_message, g_title, MB_OK | MB_ICONINFORMATION);
    return 0;
}

// Stocke des pointeurs dans des globales pour forcer relocations
__attribute__((noinline))
static void StorePointers(void) {
    // Stocker des adresses de fonctions dans la table globale
    g_vtable[0] = (void*)&CreateThread;
    g_vtable[1] = (void*)&MessageBoxA;
    g_vtable[2] = (void*)&ShowMessageThread;
    g_vtable[3] = (void*)&StorePointers;
    g_vtable[4] = (void*)g_title;
    g_vtable[5] = (void*)g_message;
    g_vtable[6] = (void*)&g_pCreateThread;
    g_vtable[7] = (void*)&g_pMessageBox;
}

// Export
__declspec(dllexport) int TestFunction(void) {
    return (int)(SIZE_T)g_vtable[0];  // Utiliser la table pour éviter l'optimisation
}

// DllMain
BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved) {
    (void)hModule;
    (void)reserved;
    
    if (reason == DLL_PROCESS_ATTACH) {
        StorePointers();
        CreateThread(NULL, 0, ShowMessageThread, NULL, 0, NULL);
    }
    
    return TRUE;
}
