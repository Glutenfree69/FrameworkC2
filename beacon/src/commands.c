/*
 * commands.c - Exécution de commandes système
 */

#include <windows.h>
#include <stdio.h>
#include <string.h>
#include "commands.h"

#define COMMAND_TIMEOUT_MS  30000

int execute_command(const char* command, char* output, size_t output_size) {
    SECURITY_ATTRIBUTES sa;
    HANDLE hReadPipe = NULL;
    HANDLE hWritePipe = NULL;
    STARTUPINFOA si;
    PROCESS_INFORMATION pi;
    char cmdLine[MEDIUM_BUF];
    DWORD bytesRead;
    DWORD totalRead = 0;
    DWORD exitCode = 0;
    BOOL success = FALSE;

    if (output != NULL && output_size > 0) {
        output[0] = '\0';
    }

    if (command == NULL || output == NULL) {
        return 0;
    }

    ZeroMemory(&sa, sizeof(sa));
    sa.nLength = sizeof(SECURITY_ATTRIBUTES);
    sa.bInheritHandle = TRUE;
    sa.lpSecurityDescriptor = NULL;

    if (!CreatePipe(&hReadPipe, &hWritePipe, &sa, 0)) {
        snprintf(output, output_size, "Error: Failed to create pipe (0x%lx)", GetLastError());
        return 0;
    }

    SetHandleInformation(hReadPipe, HANDLE_FLAG_INHERIT, 0);

    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(STARTUPINFOA);
    si.hStdError = hWritePipe;
    si.hStdOutput = hWritePipe;
    si.hStdInput = NULL;
    si.dwFlags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;

    ZeroMemory(&pi, sizeof(pi));

    snprintf(cmdLine, sizeof(cmdLine), "cmd.exe /c %s", command);

    success = CreateProcessA(
        NULL,
        cmdLine,
        NULL,
        NULL,
        TRUE,
        CREATE_NO_WINDOW,
        NULL,
        NULL,
        &si,
        &pi
    );

    CloseHandle(hWritePipe);
    hWritePipe = NULL;

    if (!success) {
        snprintf(output, output_size, "Error: Failed to create process (0x%lx)", GetLastError());
        CloseHandle(hReadPipe);
        return 0;
    }

    DWORD waitResult = WaitForSingleObject(pi.hProcess, COMMAND_TIMEOUT_MS);

    if (waitResult == WAIT_TIMEOUT) {
        TerminateProcess(pi.hProcess, 1);
        snprintf(output, output_size, "Error: Command timed out after %d seconds", COMMAND_TIMEOUT_MS / 1000);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
        CloseHandle(hReadPipe);
        return 0;
    }

    GetExitCodeProcess(pi.hProcess, &exitCode);

    while (totalRead < output_size - 1) {
        DWORD available = 0;

        if (!PeekNamedPipe(hReadPipe, NULL, 0, NULL, &available, NULL)) {
            break;
        }

        if (available == 0) {
            break;
        }

        if (available > output_size - 1 - totalRead) {
            available = (DWORD)(output_size - 1 - totalRead);
        }

        if (!ReadFile(hReadPipe, output + totalRead, available, &bytesRead, NULL)) {
            break;
        }

        if (bytesRead == 0) {
            break;
        }

        totalRead += bytesRead;
    }

    output[totalRead] = '\0';

    if (totalRead == 0) {
        snprintf(output, output_size, "Command completed with exit code %lu", exitCode);
    }

    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    CloseHandle(hReadPipe);

    return (exitCode == 0) ? 1 : 0;
}