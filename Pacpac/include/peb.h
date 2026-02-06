#ifndef PEB_H
#define PEB_H

#include "types.h"

/* =============================================
 * peb.h — Structures PEB/LDR pour x64
 * Suffisant pour le PEB walking
 * ============================================= */

typedef struct _UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    LPWSTR Buffer;
} UNICODE_STRING, *PUNICODE_STRING;

typedef struct _LIST_ENTRY {
    struct _LIST_ENTRY* Flink;
    struct _LIST_ENTRY* Blink;
} LIST_ENTRY, *PLIST_ENTRY;

typedef struct _LDR_DATA_TABLE_ENTRY {
    LIST_ENTRY InLoadOrderLinks;
    LIST_ENTRY InMemoryOrderLinks;
    LIST_ENTRY InInitializationOrderLinks;
    PVOID      DllBase;
    PVOID      EntryPoint;
    ULONG      SizeOfImage;
    UNICODE_STRING FullDllName;
    UNICODE_STRING BaseDllName;
    /* on n'a pas besoin du reste */
} LDR_DATA_TABLE_ENTRY, *PLDR_DATA_TABLE_ENTRY;

typedef struct _PEB_LDR_DATA {
    ULONG      Length;
    BOOL       Initialized;
    PVOID      SsHandle;
    LIST_ENTRY InLoadOrderModuleList;
    LIST_ENTRY InMemoryOrderModuleList;
    LIST_ENTRY InInitializationOrderModuleList;
} PEB_LDR_DATA, *PPEB_LDR_DATA;

typedef struct _PEB {
    BYTE       InheritedAddressSpace;
    BYTE       ReadImageFileExecOptions;
    BYTE       BeingDebugged;
    BYTE       _pad0[5];       /* padding x64 */
    PVOID      Mutant;
    PVOID      ImageBaseAddress;
    PPEB_LDR_DATA Ldr;
    /* on n'a pas besoin du reste pour le PEB walking */
} PEB, *PPEB;

/* =============================================
 * Prototypes — API resolution via PEB
 * ============================================= */

/* Trouve un module chargé par hash DJB2 de son nom (case-insensitive) */
PVOID peb_find_module(DWORD module_hash);

/* Parse la table d'exports d'un module pour trouver une fonction par hash DJB2 */
PVOID peb_find_export(PVOID module_base, DWORD function_hash);

/* Raccourci : résout une API en combinant module hash + function hash */
PVOID peb_resolve_api(DWORD module_hash, DWORD function_hash);

#endif /* PEB_H */
