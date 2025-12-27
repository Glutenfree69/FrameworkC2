/*
 * sysinfo.h - Récupération des informations système
 */

#ifndef SYSINFO_H
#define SYSINFO_H

#include "types.h"

int get_system_info(AgentInfo* info);
int generate_uuid(char* buffer, size_t size);

#endif