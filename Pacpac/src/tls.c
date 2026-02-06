#include "../include/types.h"
#include "../include/pe.h"

/* =============================================
 * tls.c — Section TLS statique pour le stub
 *
 * Reproduit l'approche de mingw-w64/crt/tlssup.c
 * en C pur, compatible -nostdlib.
 *
 * Le linker MinGW cherche le symbole _tls_used
 * pour peupler IMAGE_DATA_DIRECTORY[9] (TLS)
 * dans le PE header. C'est un mecanisme du linker,
 * pas du CRT.
 *
 * Windows assigne _tls_index au demarrage du
 * processus. On reutilise cet index pour la DLL
 * chargee par le manual mapper (index stealing).
 * ============================================= */

#define SECTION(x) __attribute__((section(x)))

SECTION(".tls")     char *_tls_start = NULL;
SECTION(".tls$ZZZ") char *_tls_end   = NULL;

__attribute__((used))
ULONG _tls_index = 0;  /* Ecrit par Windows au demarrage */

__attribute__((used))
SECTION(".rdata$T")
const IMAGE_TLS_DIRECTORY64 _tls_used = {
    (ULONGLONG) &_tls_start,
    (ULONGLONG) &_tls_end,
    (ULONGLONG) &_tls_index,
    (ULONGLONG) 0,    /* Pas de callbacks pour le stub */
    0, 0
};
