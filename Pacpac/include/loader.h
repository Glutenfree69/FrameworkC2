#ifndef LOADER_H
#define LOADER_H

#include "types.h"

/* =============================================
 * loader.h — PE manual mapper (x64, no-CRT)
 *
 * Inspiré de Fatmike-GH/PELoader pour le support
 * TLS, exception handling, et delay imports
 * necessaires aux binaires Rust.
 * ============================================= */

/* Typedefs pour les fonctions Windows resolues dynamiquement */
typedef VOID    (*fn_ExitProcess)(UINT uExitCode);
typedef PVOID   (*fn_VirtualAlloc)(PVOID lpAddress, SIZE_T dwSize, DWORD flAllocationType, DWORD flProtect);
typedef BOOL    (*fn_VirtualProtect)(PVOID lpAddress, SIZE_T dwSize, DWORD flNewProtect, PDWORD lpflOldProtect);
typedef BOOL    (*fn_VirtualFree)(PVOID lpAddress, SIZE_T dwSize, DWORD dwFreeType);
typedef FARPROC (*fn_GetProcAddress)(HMODULE hModule, LPCSTR lpProcName);
typedef HMODULE (*fn_LoadLibraryA)(LPCSTR lpLibFileName);
typedef BOOL    (*fn_FlushInstructionCache)(HANDLE hProcess, PVOID lpBaseAddress, SIZE_T dwSize);
typedef BOOL    (*fn_RtlAddFunctionTable)(PVOID FunctionTable, DWORD EntryCount, ULONGLONG BaseAddress);

/* Table des API resolues — remplie au demarrage */
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

/* Manual map un PE (DLL) en memoire
 * raw_pe : pointeur vers le PE brut (decompresse)
 * api    : table des API resolues
 * Retourne la base mappee, ou NULL si erreur */
PVOID pe_load(PBYTE raw_pe, API_TABLE* api);

#endif /* LOADER_H */
