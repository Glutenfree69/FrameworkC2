// evil.cpp
#include "ReflectiveLdr.h"
#include <windows.h>

// Cette macro exporte automatiquement ReflectiveLoader()
EXPORT_REFLECTIVE_LOADER

BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved)
{
    if (reason == DLL_PROCESS_ATTACH)
    {
        MessageBoxA(NULL,
                   "I am Russian",
                   "Blyat",
                   MB_OK | MB_ICONINFORMATION);
    }
    return TRUE;
}