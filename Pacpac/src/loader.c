#include "../include/types.h"
#include "../include/pe.h"
#include "../include/loader.h"

/* Prototypes utils.c */
extern void my_memcpy(void* dst, const void* src, SIZE_T size);
extern void my_memset(void* dst, int val, SIZE_T size);

/* =============================================
 * loader.c — PE manual mapper (x64, no-CRT)
 *
 * Inspire de Fatmike-GH/PELoader.
 * Ordre des operations (critique pour Rust) :
 *
 *   1. map_sections         — Allouer + copier headers + sections
 *   2. process_relocations  — Patcher les adresses si delta != 0
 *   3. init_tls             — Allouer index TLS + copier donnees TLS dans TEB
 *   4. resolve_imports      — Charger les DLL + resoudre les fonctions
 *   5. resolve_delay_imports — Idem pour les delay-loaded imports
 *   6. setup_exceptions     — Enregistrer la table RUNTIME_FUNCTION (SEH x64)
 *   7. set_protections      — Appliquer les flags PAGE_* par section
 *   8. flush_icache         — FlushInstructionCache
 *   9. execute_tls_callbacks — Appeler les TLS callbacks (DLL_PROCESS_ATTACH)
 *  10. call_entry_point     — Appeler DllMain
 * ============================================= */

typedef BOOL (*fn_DllMain)(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved);
typedef VOID (*fn_TlsCallback)(PVOID DllHandle, DWORD Reason, PVOID Reserved);

/* =============================================
 * TEB access — offsets x64 pour le TLS
 *
 * TEB layout x64 (offsets qui nous interessent) :
 *   +0x30  Self (TEB*)
 *   +0x58  ThreadLocalStoragePointer (PVOID*)
 *   +0x60  ProcessEnvironmentBlock (PEB*)
 * ============================================= */
static PVOID get_teb(void) {
    PVOID teb;
    __asm__ volatile ("mov %%gs:0x30, %0" : "=r" (teb));
    return teb;
}

/* ThreadLocalStoragePointer = TEB+0x58, c'est un PVOID* (tableau de pointeurs) */
static PVOID* get_tls_array(void) {
    PVOID teb = get_teb();
    return *(PVOID**)((PBYTE)teb + 0x58);
}

/* =============================================
 * 1. map_sections — Allouer et copier les sections
 *
 * cf. Fatmike: PELoader::MapSections
 * ============================================= */
static PBYTE map_sections(PBYTE raw_pe, IMAGE_NT_HEADERS64* nt, API_TABLE* api) {
    IMAGE_SECTION_HEADER* sections;
    PBYTE base;
    WORD i;

    /* Essayer l'adresse preferee, sinon n'importe ou */
    base = (PBYTE)api->VirtualAlloc(
        (PVOID)nt->OptionalHeader.ImageBase,
        nt->OptionalHeader.SizeOfImage,
        MEM_RESERVE | MEM_COMMIT,
        PAGE_READWRITE
    );
    if (!base) {
        base = (PBYTE)api->VirtualAlloc(
            NULL,
            nt->OptionalHeader.SizeOfImage,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE
        );
    }
    if (!base) return NULL;

    /* Copier les headers */
    my_memcpy(base, raw_pe, nt->OptionalHeader.SizeOfHeaders);

    /* Copier chaque section */
    sections = (IMAGE_SECTION_HEADER*)(
        (PBYTE)&nt->OptionalHeader + nt->FileHeader.SizeOfOptionalHeader
    );
    for (i = 0; i < nt->FileHeader.NumberOfSections; i++) {
        DWORD vsize = sections[i].Misc.VirtualSize;
        DWORD rsize = sections[i].SizeOfRawData;
        DWORD copy_size;

        if (rsize == 0) {
            /* Section BSS — zero-init */
            if (vsize > 0)
                my_memset(base + sections[i].VirtualAddress, 0, vsize);
            continue;
        }

        /* cf. Fatmike: min(SizeOfRawData, VirtualSize) */
        copy_size = (rsize < vsize) ? rsize : vsize;
        my_memcpy(
            base + sections[i].VirtualAddress,
            raw_pe + sections[i].PointerToRawData,
            copy_size
        );
    }

    return base;
}

/* =============================================
 * 2. process_relocations — Patcher les adresses
 *
 * cf. Fatmike: PELoader::ApplyRelocations
 * Supporte DIR64, HIGHLOW, HIGH, LOW
 * ============================================= */
static BOOL process_relocations(PBYTE base, IMAGE_NT_HEADERS64* nt, LONG_PTR delta) {
    IMAGE_DATA_DIRECTORY* reloc_dir = &nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC];
    IMAGE_BASE_RELOCATION* block;
    IMAGE_BASE_RELOCATION* block_end;

    if (reloc_dir->VirtualAddress == 0 || reloc_dir->Size == 0)
        return TRUE;

    block     = (IMAGE_BASE_RELOCATION*)(base + reloc_dir->VirtualAddress);
    block_end = (IMAGE_BASE_RELOCATION*)(base + reloc_dir->VirtualAddress + reloc_dir->Size);

    while (block < block_end && block->SizeOfBlock >= sizeof(IMAGE_BASE_RELOCATION)) {
        DWORD num_entries = (block->SizeOfBlock - sizeof(IMAGE_BASE_RELOCATION)) / sizeof(WORD);
        WORD* entries = (WORD*)((PBYTE)block + sizeof(IMAGE_BASE_RELOCATION));
        DWORD i;

        for (i = 0; i < num_entries; i++) {
            WORD type   = entries[i] >> 12;
            WORD offset = entries[i] & 0x0FFF;
            PBYTE target = base + block->VirtualAddress + offset;

            switch (type) {
                case IMAGE_REL_BASED_DIR64:
                    *(ULONGLONG*)target += (ULONGLONG)delta;
                    break;
                case IMAGE_REL_BASED_HIGHLOW:
                    *(DWORD*)target += (DWORD)delta;
                    break;
                case IMAGE_REL_BASED_ABSOLUTE:
                    /* Padding, rien a faire */
                    break;
                default:
                    break;
            }
        }

        block = (IMAGE_BASE_RELOCATION*)((PBYTE)block + block->SizeOfBlock);
    }

    return TRUE;
}

/* =============================================
 * 3. init_tls — Initialiser le Thread Local Storage
 *
 * Approche "Static Index Stealing" (a la Fatmike) :
 *   a) Lire _tls_index du stub (assigne par Windows au demarrage)
 *   b) Ecrire cet index dans le TLS directory de la DLL
 *   c) Copier les donnees TLS initiales (StartAddressOfRawData → EndAddressOfRawData)
 *   d) Ecrire le pointeur dans TEB.ThreadLocalStoragePointer[index]
 *
 * Note : on ne gere pas DLL_THREAD_ATTACH/DETACH pour les
 * nouveaux threads (necesserait un TlsCallbackProxy comme Fatmike).
 * Suffisant pour un usage single-thread (stub → Run()).
 * ============================================= */
extern ULONG _tls_index;

static void init_tls(PBYTE base, IMAGE_NT_HEADERS64* nt, API_TABLE* api) {
    IMAGE_DATA_DIRECTORY* tls_dir = &nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_TLS];
    IMAGE_TLS_DIRECTORY64* tls;
    DWORD stub_tls_index;
    SIZE_T raw_size, total_size;
    PVOID tls_data;
    PVOID* tls_array;

    if (tls_dir->VirtualAddress == 0 || tls_dir->Size == 0)
        return;

    tls = (IMAGE_TLS_DIRECTORY64*)(base + tls_dir->VirtualAddress);
    if (tls->AddressOfIndex == 0)
        return;

    /* a) Lire l'index statique du stub (assigne par Windows) */
    stub_tls_index = _tls_index;

    /* b) Ecrire cet index dans le TLS directory de la DLL */
    *(DWORD*)(tls->AddressOfIndex) = stub_tls_index;

    /* c) Allouer + copier les donnees TLS initiales */
    if (tls->StartAddressOfRawData == 0 || tls->EndAddressOfRawData == 0)
        return;

    raw_size = (SIZE_T)(tls->EndAddressOfRawData - tls->StartAddressOfRawData);
    total_size = raw_size + tls->SizeOfZeroFill;
    if (total_size == 0)
        return;

    tls_data = api->VirtualAlloc(NULL, total_size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    if (!tls_data)
        return;

    if (raw_size > 0)
        my_memcpy(tls_data, (PVOID)tls->StartAddressOfRawData, raw_size);
    if (tls->SizeOfZeroFill > 0)
        my_memset((PBYTE)tls_data + raw_size, 0, tls->SizeOfZeroFill);

    /* d) Ecrire dans TEB.ThreadLocalStoragePointer[index] */
    tls_array = get_tls_array();
    if (tls_array && stub_tls_index < 64)
        tls_array[stub_tls_index] = tls_data;
}

/* =============================================
 * 4. resolve_imports — Charger les DLL et patcher l'IAT
 *
 * cf. Fatmike: PELoader::ResolveImports
 * ============================================= */
static BOOL resolve_imports(PBYTE base, IMAGE_NT_HEADERS64* nt, API_TABLE* api) {
    IMAGE_DATA_DIRECTORY* import_dir = &nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    IMAGE_IMPORT_DESCRIPTOR* desc;

    if (import_dir->VirtualAddress == 0 || import_dir->Size == 0)
        return TRUE;

    desc = (IMAGE_IMPORT_DESCRIPTOR*)(base + import_dir->VirtualAddress);

    while (desc->Name != 0) {
        const char* dll_name = (const char*)(base + desc->Name);
        HMODULE mod = api->LoadLibraryA(dll_name);
        if (!mod)
            return FALSE;

        {
            ULONGLONG* orig_thunk = (ULONGLONG*)(base + (desc->OriginalFirstThunk ? desc->OriginalFirstThunk : desc->FirstThunk));
            ULONGLONG* thunk      = (ULONGLONG*)(base + desc->FirstThunk);

            while (*orig_thunk) {
                FARPROC func;

                if (*orig_thunk & IMAGE_ORDINAL_FLAG64) {
                    /* Import par ordinal */
                    LPCSTR ordinal = (LPCSTR)(*orig_thunk & 0xFFFF);
                    func = api->GetProcAddress(mod, ordinal);
                } else {
                    /* Import par nom */
                    IMAGE_IMPORT_BY_NAME* import_name = (IMAGE_IMPORT_BY_NAME*)(base + (DWORD)*orig_thunk);
                    func = api->GetProcAddress(mod, import_name->Name);
                }

                if (!func)
                    return FALSE;

                *thunk = (ULONGLONG)func;

                orig_thunk++;
                thunk++;
            }
        }

        desc++;
    }

    return TRUE;
}

/* =============================================
 * 5. resolve_delay_imports — Delay-loaded imports
 *
 * cf. Fatmike: PELoader::ResolveDelayImports
 * Meme principe que les imports normaux mais avec
 * IMAGE_DELAYLOAD_DESCRIPTOR au lieu de IMAGE_IMPORT_DESCRIPTOR
 * ============================================= */
static BOOL resolve_delay_imports(PBYTE base, IMAGE_NT_HEADERS64* nt, API_TABLE* api) {
    IMAGE_DATA_DIRECTORY* delay_dir = &nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT];
    IMAGE_DELAYLOAD_DESCRIPTOR* desc;

    if (delay_dir->VirtualAddress == 0 || delay_dir->Size == 0)
        return TRUE;

    desc = (IMAGE_DELAYLOAD_DESCRIPTOR*)(base + delay_dir->VirtualAddress);

    while (desc->DllNameRVA != 0) {
        const char* dll_name = (const char*)(base + desc->DllNameRVA);
        HMODULE mod = api->LoadLibraryA(dll_name);
        if (!mod)
            return FALSE;

        {
            ULONGLONG* name_thunk = (ULONGLONG*)(base + desc->ImportNameTableRVA);
            ULONGLONG* addr_thunk = (ULONGLONG*)(base + desc->ImportAddressTableRVA);

            while (*name_thunk) {
                FARPROC func;

                if (*name_thunk & IMAGE_ORDINAL_FLAG64) {
                    LPCSTR ordinal = (LPCSTR)(*name_thunk & 0xFFFF);
                    func = api->GetProcAddress(mod, ordinal);
                } else {
                    IMAGE_IMPORT_BY_NAME* import_name = (IMAGE_IMPORT_BY_NAME*)(base + (DWORD)*name_thunk);
                    func = api->GetProcAddress(mod, import_name->Name);
                }

                if (!func)
                    return FALSE;

                *addr_thunk = (ULONGLONG)func;

                name_thunk++;
                addr_thunk++;
            }
        }

        desc++;
    }

    return TRUE;
}

/* =============================================
 * 6. setup_exceptions — Enregistrer la table d'exceptions x64
 *
 * cf. Fatmike: PELoader::SetupExceptionHandling
 * Critique pour Rust : le panic unwinding utilise SEH.
 * Sans ca, tout panic = crash immediat.
 * ============================================= */
static void setup_exceptions(PBYTE base, IMAGE_NT_HEADERS64* nt, API_TABLE* api) {
    IMAGE_DATA_DIRECTORY* exc_dir = &nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXCEPTION];
    PRUNTIME_FUNCTION func_table;
    DWORD count;

    if (exc_dir->VirtualAddress == 0 || exc_dir->Size == 0)
        return;

    func_table = (PRUNTIME_FUNCTION)(base + exc_dir->VirtualAddress);
    count = exc_dir->Size / sizeof(RUNTIME_FUNCTION);

    api->RtlAddFunctionTable(func_table, count, (ULONGLONG)base);
}

/* =============================================
 * 7. set_protections — Flags memoire par section
 *
 * cf. Fatmike: PELoader::ApplySectionMemoryProtection
 * ============================================= */
static DWORD section_to_page_protect(DWORD characteristics) {
    BOOL exec  = (characteristics & IMAGE_SCN_MEM_EXECUTE) != 0;
    BOOL read  = (characteristics & IMAGE_SCN_MEM_READ)    != 0;
    BOOL write = (characteristics & IMAGE_SCN_MEM_WRITE)   != 0;

    if (exec && write)  return PAGE_EXECUTE_READWRITE;
    if (exec && read)   return PAGE_EXECUTE_READ;
    if (exec)           return PAGE_EXECUTE;
    if (write)          return PAGE_READWRITE;
    if (read)           return PAGE_READONLY;
    return PAGE_NOACCESS;
}

static void set_protections(PBYTE base, IMAGE_NT_HEADERS64* nt, API_TABLE* api) {
    IMAGE_SECTION_HEADER* sections = (IMAGE_SECTION_HEADER*)(
        (PBYTE)&nt->OptionalHeader + nt->FileHeader.SizeOfOptionalHeader
    );
    WORD i;

    for (i = 0; i < nt->FileHeader.NumberOfSections; i++) {
        DWORD size = sections[i].Misc.VirtualSize;
        DWORD old;

        if (size == 0) continue;

        api->VirtualProtect(
            base + sections[i].VirtualAddress,
            size,
            section_to_page_protect(sections[i].Characteristics),
            &old
        );
    }
}

/* =============================================
 * 9. execute_tls_callbacks — Appeler les TLS callbacks
 *
 * cf. Fatmike: TlsResolver::ExecuteCallbacks
 * Les binaires Rust initialisent leur runtime via TLS callbacks.
 * ============================================= */
static void execute_tls_callbacks(PBYTE base, IMAGE_NT_HEADERS64* nt, DWORD reason) {
    IMAGE_DATA_DIRECTORY* tls_dir = &nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_TLS];
    IMAGE_TLS_DIRECTORY64* tls;
    fn_TlsCallback* callbacks;

    if (tls_dir->VirtualAddress == 0 || tls_dir->Size == 0)
        return;

    tls = (IMAGE_TLS_DIRECTORY64*)(base + tls_dir->VirtualAddress);

    if (tls->AddressOfCallBacks == 0)
        return;

    callbacks = (fn_TlsCallback*)(tls->AddressOfCallBacks);
    while (*callbacks) {
        (*callbacks)((PVOID)base, reason, NULL);
        callbacks++;
    }
}

/* =============================================
 * pe_load — Point d'entree du manual mapper
 *
 * cf. Fatmike: PELoader::LoadPE
 * Ordre des operations adapte pour supporter Rust.
 * ============================================= */
PVOID pe_load(PBYTE raw_pe, API_TABLE* api) {
    IMAGE_DOS_HEADER*    dos;
    IMAGE_NT_HEADERS64*  nt;
    PBYTE base;
    LONG_PTR delta;

    /* --- Valider le PE --- */
    dos = (IMAGE_DOS_HEADER*)raw_pe;
    if (dos->e_magic != IMAGE_DOS_SIGNATURE)
        return NULL;

    nt = (IMAGE_NT_HEADERS64*)(raw_pe + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE)
        return NULL;

    /* --- 1. Map sections --- */
    base = map_sections(raw_pe, nt, api);
    if (!base)
        return NULL;

    /* Re-parser les NT headers depuis l'image mappee
     * (les headers ont ete copies dans la nouvelle allocation) */
    nt = (IMAGE_NT_HEADERS64*)(base + dos->e_lfanew);

    /* --- 2. Relocations --- */
    delta = (LONG_PTR)((ULONGLONG)base - ((IMAGE_NT_HEADERS64*)(raw_pe + ((IMAGE_DOS_HEADER*)raw_pe)->e_lfanew))->OptionalHeader.ImageBase);
    if (delta != 0) {
        if (!process_relocations(base, nt, delta)) {
            api->VirtualFree(base, 0, MEM_RELEASE);
            return NULL;
        }
    }

    /* --- 3. Imports (AVANT TLS - charge les DLLs dont Rust a besoin) --- */
    if (!resolve_imports(base, nt, api)) {
        api->VirtualFree(base, 0, MEM_RELEASE);
        return NULL;
    }

    /* --- 4. Delay imports --- */
    if (!resolve_delay_imports(base, nt, api)) {
        api->VirtualFree(base, 0, MEM_RELEASE);
        return NULL;
    }

    /* --- 5. TLS init (static index stealing) --- */
    init_tls(base, nt, api);

    /* --- 6. Exception handling (x64 SEH) --- */
    setup_exceptions(base, nt, api);

    /* --- 7. Section protections --- */
    set_protections(base, nt, api);

    /* --- 8. Flush instruction cache --- */
    api->FlushInstructionCache((HANDLE)(LONG_PTR)-1, NULL, 0);

    /* --- 9. TLS callbacks --- */
    execute_tls_callbacks(base, nt, DLL_PROCESS_ATTACH);

    /* --- 10. DllMain --- */
    if (nt->OptionalHeader.AddressOfEntryPoint != 0) {
        fn_DllMain entry = (fn_DllMain)(base + nt->OptionalHeader.AddressOfEntryPoint);
        entry((HINSTANCE)base, DLL_PROCESS_ATTACH, NULL);
    }

    return (PVOID)base;
}
