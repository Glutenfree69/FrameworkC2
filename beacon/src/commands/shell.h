#ifndef SHELL_H
#define SHELL_H

#include "../types.h"

// Command execution with syscalls
int execute_command(const char* command, char* output, size_t output_size);

#endif // SHELL_H
