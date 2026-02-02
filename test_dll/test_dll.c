// test_dll.c
// DLL de test simple pour le PE Loader
// Compile: x86_64-w64-mingw32-gcc -shared -o test_dll.dll test_dll.c -luser32

#include <windows.h>

__declspec(dllexport) void TestExport(void) {
    MessageBoxA(NULL, "TestExport called!", "Test DLL", MB_OK);
}

BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved) {
    switch (reason) {
        case DLL_PROCESS_ATTACH:
            MessageBoxA(NULL, 
                "PE Loader Success!\n\nDllMain called with DLL_PROCESS_ATTACH",
                "Test DLL - Loaded!",
                MB_OK | MB_ICONINFORMATION);
            break;
        case DLL_PROCESS_DETACH:
            // Cleanup
            break;
    }
    return TRUE;
}
