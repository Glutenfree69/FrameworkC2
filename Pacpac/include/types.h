#ifndef TYPES_H
#define TYPES_H

/* =============================================
 * types.h — Windows typedefs sans windows.h
 * Uniquement ce dont on a besoin, x64 only
 * ============================================= */

typedef unsigned char       BYTE;
typedef unsigned short      WORD;
typedef unsigned int        DWORD;
typedef unsigned long long  QWORD;
typedef unsigned long long  ULONGLONG;
typedef unsigned long long  ULONG_PTR;
typedef long long           LONG_PTR;
typedef long                LONG;
typedef int                 BOOL;
typedef unsigned int        UINT;
typedef unsigned short      USHORT;
typedef unsigned long       ULONG;
typedef char                CHAR;
typedef short               SHORT;
typedef unsigned short      WCHAR;   /* wchar_t is 16-bit on Windows, no CRT needed */

typedef void                VOID;
typedef void*               PVOID;
typedef void*               LPVOID;
typedef void*               HANDLE;
typedef void*               HMODULE;
typedef void*               HINSTANCE;
typedef BYTE*               PBYTE;
typedef DWORD*              PDWORD;
typedef WORD*               PWORD;
typedef char*               LPSTR;
typedef const char*         LPCSTR;
typedef WCHAR*              LPWSTR;
typedef const WCHAR*        LPCWSTR;

typedef ULONG_PTR           SIZE_T;
typedef LONG_PTR            SSIZE_T;

/* x64: pointeurs = 8 bytes */
typedef unsigned long long  FARPROC;

#define NULL    ((void*)0)
#define TRUE    1
#define FALSE   0

#define DLL_PROCESS_ATTACH  1
#define DLL_THREAD_ATTACH   2
#define DLL_THREAD_DETACH   3
#define DLL_PROCESS_DETACH  0

/* VirtualAlloc / VirtualProtect constants */
#define MEM_COMMIT          0x00001000
#define MEM_RESERVE         0x00002000
#define MEM_RELEASE         0x00008000

#define PAGE_NOACCESS       0x01
#define PAGE_READONLY       0x02
#define PAGE_READWRITE      0x04
#define PAGE_EXECUTE        0x10
#define PAGE_EXECUTE_READ   0x20
#define PAGE_EXECUTE_READWRITE 0x40

/* Section characteristics flags */
#define IMAGE_SCN_MEM_EXECUTE   0x20000000
#define IMAGE_SCN_MEM_READ      0x40000000
#define IMAGE_SCN_MEM_WRITE     0x80000000

#endif /* TYPES_H */
