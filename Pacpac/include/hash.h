#ifndef HASH_H
#define HASH_H

#include "types.h"

/* =============================================
 * hash.h — DJB2 hashing pour API resolution
 * ============================================= */

/* DJB2 hash runtime — pour strings ASCII (noms d'exports) */
static inline DWORD djb2_hash_a(const char* str) {
    DWORD hash = 5381;
    int c;
    while ((c = *str++))
        hash = ((hash << 5) + hash) + c;  /* hash * 33 + c */
    return hash;
}

/* DJB2 hash runtime — pour strings wide (noms de modules), case-insensitive */
static inline DWORD djb2_hash_w(const WCHAR* str) {
    DWORD hash = 5381;
    WCHAR c;
    while ((c = *str++)) {
        /* toLower ASCII range */
        if (c >= 0x41 && c <= 0x5A)  /* 'A' .. 'Z' */
            c += 0x20;
        hash = ((hash << 5) + hash) + (DWORD)c;
    }
    return hash;
}

/* =============================================
 * Hashes pre-calcules pour les modules/fonctions
 * Calculés avec :  djb2("kernel32.dll") etc.
 * ============================================= */

#define HASH_KERNEL32_DLL       0x7040EE75  /* djb2_hash_w(L"kernel32.dll") */
#define HASH_NTDLL_DLL          0x22D3B5ED  /* djb2_hash_w(L"ntdll.dll") */

#define HASH_GETPROCADDRESS     0xCF31BB1F  /* djb2_hash_a("GetProcAddress") */
#define HASH_LOADLIBRARYA       0x5FBFF0FB  /* djb2_hash_a("LoadLibraryA") */
#define HASH_VIRTUALALLOC       0x382C0F97  /* djb2_hash_a("VirtualAlloc") */
#define HASH_VIRTUALPROTECT     0x844FF18D  /* djb2_hash_a("VirtualProtect") */
#define HASH_VIRTUALFREE        0x668FCF2E  /* djb2_hash_a("VirtualFree") */
#define HASH_EXITPROCESS        0xB769339E  /* djb2_hash_a("ExitProcess") */
#define HASH_FLUSHINSTRUCTIONCACHE 0xB7DCEDDD /* djb2_hash_a("FlushInstructionCache") */
#define HASH_RTLADDFUNCTIONTABLE   0xBDB9F1AE /* djb2_hash_a("RtlAddFunctionTable") */

#endif /* HASH_H */
