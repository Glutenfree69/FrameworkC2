#include "../include/types.h"

/* =============================================
 * utils.c — Fonctions utilitaires no-CRT
 * Remplace memcpy, memset, strlen de la libc
 * ============================================= */

void my_memcpy(void* dst, const void* src, SIZE_T size) {
    BYTE* d = (BYTE*)dst;
    const BYTE* s = (const BYTE*)src;
    while (size--)
        *d++ = *s++;
}

void my_memset(void* dst, int val, SIZE_T size) {
    BYTE* d = (BYTE*)dst;
    while (size--)
        *d++ = (BYTE)val;
}

SIZE_T my_strlen(const char* str) {
    const char* s = str;
    while (*s) s++;
    return (SIZE_T)(s - str);
}

int my_strcmp(const char* s1, const char* s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(const unsigned char*)s1 - *(const unsigned char*)s2;
}
