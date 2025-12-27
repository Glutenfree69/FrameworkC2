/*
 * json.h - Construction et parsing de JSON
 */

#ifndef JSON_H
#define JSON_H

#include "../types.h"

int json_build_checkin(const AgentInfo* info, char* output, size_t output_size);
int json_build_result(const TaskResult* result, char* output, size_t output_size);
int json_parse_task(const char* json, TaskResponse* task);
int json_get_string(const char* json, const char* key, char* value, size_t value_size);
int json_get_int(const char* json, const char* key, int* value);

#endif