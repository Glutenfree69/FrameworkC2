#ifndef UNICODE_H
#define UNICODE_H

#include <windows.h>
#include "../types.h"

// Conversions UNICODE_STRING <-> char*
void unicode_to_ansi(UNICODE_STRING* unicode, char* ansi, size_t max_len);
void ansi_to_unicode(const char* ansi, UNICODE_STRING* unicode, WCHAR* buffer, size_t buffer_size);

#endif // UNICODE_H
