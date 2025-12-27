/*
 * syscalls_stub.c - Stubs temporaires pour cross-compilation
 *
 * Note: Ce fichier est un placeholder pour permettre la compilation sur macOS.
 * Sur Windows, utiliser syscalls.c généré par SysWhispers3 à la place.
 */

#include "syscalls.h"

// Stubs - retournent tous STATUS_NOT_IMPLEMENTED
#define STATUS_NOT_IMPLEMENTED 0xC0000002

// Les déclarations sont dans syscalls.h, on fournit juste les implémentations

NTSTATUS Sw3NtClose(IN HANDLE Handle) {
    (void)Handle;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtCreateFile(
    OUT PHANDLE FileHandle,
    IN ACCESS_MASK DesiredAccess,
    IN POBJECT_ATTRIBUTES ObjectAttributes,
    OUT PIO_STATUS_BLOCK IoStatusBlock,
    IN PLARGE_INTEGER AllocationSize,
    IN ULONG FileAttributes,
    IN ULONG ShareAccess,
    IN ULONG CreateDisposition,
    IN ULONG CreateOptions,
    IN PVOID EaBuffer,
    IN ULONG EaLength)
{
    (void)FileHandle; (void)DesiredAccess; (void)ObjectAttributes;
    (void)IoStatusBlock; (void)AllocationSize; (void)FileAttributes;
    (void)ShareAccess; (void)CreateDisposition; (void)CreateOptions;
    (void)EaBuffer; (void)EaLength;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtDeviceIoControlFile(
    IN HANDLE FileHandle,
    IN HANDLE Event,
    IN PIO_APC_ROUTINE ApcRoutine,
    IN PVOID ApcContext,
    OUT PIO_STATUS_BLOCK IoStatusBlock,
    IN ULONG IoControlCode,
    IN PVOID InputBuffer,
    IN ULONG InputBufferLength,
    OUT PVOID OutputBuffer,
    IN ULONG OutputBufferLength)
{
    (void)FileHandle; (void)Event; (void)ApcRoutine; (void)ApcContext;
    (void)IoStatusBlock; (void)IoControlCode; (void)InputBuffer;
    (void)InputBufferLength; (void)OutputBuffer; (void)OutputBufferLength;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtReadFile(
    IN HANDLE FileHandle,
    IN HANDLE Event,
    IN PIO_APC_ROUTINE ApcRoutine,
    OUT PVOID ApcContext,
    OUT PIO_STATUS_BLOCK IoStatusBlock,
    IN PVOID Buffer,
    IN ULONG Length,
    IN PLARGE_INTEGER ByteOffset,
    IN PULONG Key)
{
    (void)FileHandle; (void)Event; (void)ApcRoutine; (void)ApcContext;
    (void)IoStatusBlock; (void)Buffer; (void)Length;
    (void)ByteOffset; (void)Key;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtWriteFile(
    IN HANDLE FileHandle,
    IN HANDLE Event,
    IN PIO_APC_ROUTINE ApcRoutine,
    IN PVOID ApcContext,
    OUT PIO_STATUS_BLOCK IoStatusBlock,
    IN PVOID Buffer,
    IN ULONG Length,
    IN PLARGE_INTEGER ByteOffset,
    IN PULONG Key)
{
    (void)FileHandle; (void)Event; (void)ApcRoutine; (void)ApcContext;
    (void)IoStatusBlock; (void)Buffer; (void)Length;
    (void)ByteOffset; (void)Key;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtWaitForSingleObject(
    IN HANDLE ObjectHandle,
    IN BOOLEAN Alertable,
    IN PLARGE_INTEGER TimeOut)
{
    (void)ObjectHandle; (void)Alertable; (void)TimeOut;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtTerminateProcess(
    IN HANDLE ProcessHandle,
    IN NTSTATUS ExitStatus)
{
    (void)ProcessHandle; (void)ExitStatus;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtQueryInformationProcess(
    IN HANDLE ProcessHandle,
    IN PROCESSINFOCLASS ProcessInformationClass,
    OUT PVOID ProcessInformation,
    IN ULONG ProcessInformationLength,
    OUT PULONG ReturnLength)
{
    (void)ProcessHandle; (void)ProcessInformationClass;
    (void)ProcessInformation; (void)ProcessInformationLength;
    (void)ReturnLength;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtAllocateVirtualMemory(
    IN HANDLE ProcessHandle,
    IN OUT PVOID *BaseAddress,
    IN ULONG ZeroBits,
    IN OUT PSIZE_T RegionSize,
    IN ULONG AllocationType,
    IN ULONG Protect)
{
    (void)ProcessHandle; (void)BaseAddress; (void)ZeroBits;
    (void)RegionSize; (void)AllocationType; (void)Protect;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtFreeVirtualMemory(
    IN HANDLE ProcessHandle,
    IN OUT PVOID *BaseAddress,
    IN OUT PSIZE_T RegionSize,
    IN ULONG FreeType)
{
    (void)ProcessHandle; (void)BaseAddress;
    (void)RegionSize; (void)FreeType;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtCreateUserProcess(
    OUT PHANDLE ProcessHandle,
    OUT PHANDLE ThreadHandle,
    IN ACCESS_MASK ProcessDesiredAccess,
    IN ACCESS_MASK ThreadDesiredAccess,
    IN POBJECT_ATTRIBUTES ProcessObjectAttributes,
    IN POBJECT_ATTRIBUTES ThreadObjectAttributes,
    IN ULONG ProcessFlags,
    IN ULONG ThreadFlags,
    IN PVOID ProcessParameters,
    IN struct _PS_CREATE_INFO* CreateInfo,
    IN struct _PS_ATTRIBUTE_LIST* AttributeList)
{
    (void)ProcessHandle; (void)ThreadHandle;
    (void)ProcessDesiredAccess; (void)ThreadDesiredAccess;
    (void)ProcessObjectAttributes; (void)ThreadObjectAttributes;
    (void)ProcessFlags; (void)ThreadFlags; (void)ProcessParameters;
    (void)CreateInfo; (void)AttributeList;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtQuerySystemInformation(
    IN SYSTEM_INFORMATION_CLASS SystemInformationClass,
    IN OUT PVOID SystemInformation,
    IN ULONG SystemInformationLength,
    OUT PULONG ReturnLength)
{
    (void)SystemInformationClass; (void)SystemInformation;
    (void)SystemInformationLength; (void)ReturnLength;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtQuerySystemTime(
    OUT PLARGE_INTEGER SystemTime)
{
    (void)SystemTime;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtOpenKey(
    OUT PHANDLE KeyHandle,
    IN ACCESS_MASK DesiredAccess,
    IN POBJECT_ATTRIBUTES ObjectAttributes)
{
    (void)KeyHandle; (void)DesiredAccess; (void)ObjectAttributes;
    return STATUS_NOT_IMPLEMENTED;
}

NTSTATUS Sw3NtQueryValueKey(
    IN HANDLE KeyHandle,
    IN PUNICODE_STRING ValueName,
    IN KEY_VALUE_INFORMATION_CLASS KeyValueInformationClass,
    OUT PVOID KeyValueInformation,
    IN ULONG Length,
    OUT PULONG ResultLength)
{
    (void)KeyHandle; (void)ValueName; (void)KeyValueInformationClass;
    (void)KeyValueInformation; (void)Length; (void)ResultLength;
    return STATUS_NOT_IMPLEMENTED;
}
