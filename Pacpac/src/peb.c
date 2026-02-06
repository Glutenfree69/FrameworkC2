#include "../include/peb.h"
#include "../include/pe.h"
#include "../include/hash.h"

/* =============================================
 * peb.c — PEB Walking + API Resolution par hash
 *
 * Strategie :
 *   1. Lire le PEB depuis le TEB (gs:[0x60] sur x64)
 *   2. Parcourir InLoadOrderModuleList
 *   3. Comparer le hash DJB2 du nom de module
 *   4. Parser la table d'exports pour trouver la fonction
 * ============================================= */

/* Lit le PEB depuis le TEB via le segment register GS (x64) */
static PEB* get_peb(void) {
    PEB* peb;
    __asm__ volatile (
        "mov %%gs:0x60, %0"
        : "=r" (peb)
    );
    return peb;
}

/* =============================================
 * peb_find_module — Trouve un module par hash DJB2
 * Parcourt PEB->Ldr->InLoadOrderModuleList
 * ============================================= */
PVOID peb_find_module(DWORD module_hash) {
    PEB* peb = get_peb();
    if (!peb || !peb->Ldr)
        return NULL;

    LIST_ENTRY* head = &peb->Ldr->InLoadOrderModuleList;
    LIST_ENTRY* curr = head->Flink;

    while (curr != head) {
        LDR_DATA_TABLE_ENTRY* entry = (LDR_DATA_TABLE_ENTRY*)curr;

        if (entry->BaseDllName.Buffer != NULL) {
            DWORD hash = djb2_hash_w(entry->BaseDllName.Buffer);
            if (hash == module_hash)
                return entry->DllBase;
        }

        curr = curr->Flink;
    }

    return NULL;
}

/* =============================================
 * peb_find_export — Trouve une fonction exportee par hash
 * Parse la table d'exports du module
 * ============================================= */
PVOID peb_find_export(PVOID module_base, DWORD function_hash) {
    if (!module_base)
        return NULL;

    PBYTE base = (PBYTE)module_base;

    /* Valider DOS header */
    IMAGE_DOS_HEADER* dos = (IMAGE_DOS_HEADER*)base;
    if (dos->e_magic != IMAGE_DOS_SIGNATURE)
        return NULL;

    /* NT headers */
    IMAGE_NT_HEADERS64* nt = (IMAGE_NT_HEADERS64*)(base + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE)
        return NULL;

    /* Export directory */
    DWORD export_rva = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT].VirtualAddress;
    DWORD export_size = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT].Size;
    if (export_rva == 0)
        return NULL;

    IMAGE_EXPORT_DIRECTORY* exports = (IMAGE_EXPORT_DIRECTORY*)(base + export_rva);

    DWORD* names      = (DWORD*)(base + exports->AddressOfNames);
    WORD*  ordinals   = (WORD*)(base + exports->AddressOfNameOrdinals);
    DWORD* functions  = (DWORD*)(base + exports->AddressOfFunctions);

    for (DWORD i = 0; i < exports->NumberOfNames; i++) {
        const char* name = (const char*)(base + names[i]);
        DWORD hash = djb2_hash_a(name);

        if (hash == function_hash) {
            DWORD func_rva = functions[ordinals[i]];

            /* Verifier si c'est un forwarded export (RVA pointe dans l'export dir) */
            if (func_rva >= export_rva && func_rva < export_rva + export_size)
                return NULL; /* On ne gere pas les forwards pour l'instant */

            return (PVOID)(base + func_rva);
        }
    }

    return NULL;
}

/* =============================================
 * peb_resolve_api — Raccourci module_hash + function_hash
 * ============================================= */
PVOID peb_resolve_api(DWORD module_hash, DWORD function_hash) {
    PVOID module = peb_find_module(module_hash);
    if (!module)
        return NULL;
    return peb_find_export(module, function_hash);
}
