#ifndef CHACHA20_H
#define CHACHA20_H

#include "types.h"

/* =============================================
 * chacha20.h — ChaCha20 stream cipher (RFC 7539)
 *
 * key:     32 bytes (256-bit)
 * nonce:   12 bytes (96-bit)
 * counter: bloc initial (typiquement 0)
 *
 * XOR-based : encrypt == decrypt
 * ============================================= */

void chacha20_crypt(BYTE* data, DWORD data_len,
                    const BYTE key[32], const BYTE nonce[12],
                    DWORD counter);

#endif /* CHACHA20_H */
