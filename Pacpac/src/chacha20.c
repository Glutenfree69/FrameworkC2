#include "../include/types.h"
#include "../include/chacha20.h"

/* =============================================
 * chacha20.c — ChaCha20 stream cipher (RFC 7539)
 *
 * Implementation minimale no-CRT.
 * Ref: https://datatracker.ietf.org/doc/html/rfc7539
 * ============================================= */

#define ROTL32(v, n) (((v) << (n)) | ((v) >> (32 - (n))))

/* Quarter round — operation de base de ChaCha20 */
static void quarter_round(DWORD* a, DWORD* b, DWORD* c, DWORD* d) {
    *a += *b; *d ^= *a; *d = ROTL32(*d, 16);
    *c += *d; *b ^= *c; *b = ROTL32(*b, 12);
    *a += *b; *d ^= *a; *d = ROTL32(*d, 8);
    *c += *d; *b ^= *c; *b = ROTL32(*b, 7);
}

/* Charge un DWORD depuis un pointeur en little-endian */
static DWORD load32_le(const BYTE* p) {
    return (DWORD)p[0]        | ((DWORD)p[1] << 8) |
           ((DWORD)p[2] << 16) | ((DWORD)p[3] << 24);
}

/* Genere 64 octets de keystream a partir du state */
static void chacha20_block(DWORD state[16], BYTE out[64]) {
    DWORD working[16];
    int i;

    for (i = 0; i < 16; i++)
        working[i] = state[i];

    /* 20 rounds = 10 double rounds */
    for (i = 0; i < 10; i++) {
        /* Column rounds */
        quarter_round(&working[0], &working[4], &working[8],  &working[12]);
        quarter_round(&working[1], &working[5], &working[9],  &working[13]);
        quarter_round(&working[2], &working[6], &working[10], &working[14]);
        quarter_round(&working[3], &working[7], &working[11], &working[15]);
        /* Diagonal rounds */
        quarter_round(&working[0], &working[5], &working[10], &working[15]);
        quarter_round(&working[1], &working[6], &working[11], &working[12]);
        quarter_round(&working[2], &working[7], &working[8],  &working[13]);
        quarter_round(&working[3], &working[4], &working[9],  &working[14]);
    }

    /* Ajouter le state original (feedforward) */
    for (i = 0; i < 16; i++)
        working[i] += state[i];

    /* Serialiser en bytes little-endian */
    for (i = 0; i < 16; i++) {
        out[i * 4 + 0] = (BYTE)(working[i]);
        out[i * 4 + 1] = (BYTE)(working[i] >> 8);
        out[i * 4 + 2] = (BYTE)(working[i] >> 16);
        out[i * 4 + 3] = (BYTE)(working[i] >> 24);
    }
}

/* =============================================
 * chacha20_crypt — Chiffre/dechiffre des donnees
 *
 * State layout (4x4 matrix de DWORD) :
 *   "expa"  "nd 3"  "2-by"  "te k"    <- constantes
 *    key0    key1    key2    key3      <- cle (mots 0-3)
 *    key4    key5    key6    key7      <- cle (mots 4-7)
 *   counter nonce0  nonce1  nonce2    <- counter + nonce
 * ============================================= */
void chacha20_crypt(BYTE* data, DWORD data_len,
                    const BYTE key[32], const BYTE nonce[12],
                    DWORD counter) {
    DWORD state[16];
    BYTE keystream[64];
    DWORD i, block_size;

    /* Constantes "expand 32-byte k" */
    state[0] = 0x61707865;
    state[1] = 0x3320646e;
    state[2] = 0x79622d32;
    state[3] = 0x6b206574;

    /* Cle : 8 mots de 32 bits */
    for (i = 0; i < 8; i++)
        state[4 + i] = load32_le(key + i * 4);

    /* Counter */
    state[12] = counter;

    /* Nonce : 3 mots de 32 bits */
    state[13] = load32_le(nonce);
    state[14] = load32_le(nonce + 4);
    state[15] = load32_le(nonce + 8);

    /* Generer le keystream et XOR avec les donnees */
    while (data_len > 0) {
        chacha20_block(state, keystream);

        block_size = (data_len < 64) ? data_len : 64;
        for (i = 0; i < block_size; i++)
            data[i] ^= keystream[i];

        data     += block_size;
        data_len -= block_size;
        state[12]++;  /* incrementer le counter */
    }
}
