/*
 * commands.h - Exécution de commandes système
 */

#ifndef COMMANDS_H
#define COMMANDS_H

#include "types.h"

int execute_command(const char* command, char* output, size_t output_size);

#endif