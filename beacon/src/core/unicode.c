#include "unicode.h"
#include <string.h>

// Convertir UNICODE_STRING → char* (UTF-16 → ASCII)
void unicode_to_ansi(UNICODE_STRING* unicode, char* ansi, size_t max_len) {
    if (unicode == NULL || ansi == NULL || max_len == 0) {
        return;
    }

    size_t len = unicode->Length / sizeof(WCHAR);
    if (len >= max_len) {
        len = max_len - 1;
    }

    for (size_t i = 0; i < len; i++) {
        ansi[i] = (char)unicode->Buffer[i];
    }
    ansi[len] = '\0';
}

// Convertir char* → UNICODE_STRING (ASCII → UTF-16)
void ansi_to_unicode(const char* ansi, UNICODE_STRING* unicode, WCHAR* buffer, size_t buffer_size) {
    if (ansi == NULL || unicode == NULL || buffer == NULL) {
        return;
    }

    size_t len = strlen(ansi);
    if (len >= buffer_size) {
        len = buffer_size - 1;
    }

    for (size_t i = 0; i < len; i++) {
        buffer[i] = (WCHAR)ansi[i];
    }
    buffer[len] = L'\0';

    unicode->Buffer = buffer;
    unicode->Length = (USHORT)(len * sizeof(WCHAR));
    unicode->MaximumLength = (USHORT)(buffer_size * sizeof(WCHAR));
}
