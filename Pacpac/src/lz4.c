#include "../include/types.h"
#include "../include/lz4.h"

/* =============================================
 * lz4.c — LZ4 block decompressor (no-CRT)
 *
 * Format LZ4 block :
 *   [token] [lit_len_ext*] [literals] [offset] [match_len_ext*]
 *
 * Token : high nibble = literal length, low nibble = match length
 * Si nibble == 15, lire des octets supplementaires (sommes)
 * Match length += 4 (minimum match)
 * Offset : 2 bytes little-endian (backward ref dans output)
 * La derniere sequence n'a pas de match
 * ============================================= */

int lz4_decompress(const BYTE* src, DWORD src_len,
                   BYTE* dst, DWORD dst_cap) {
    const BYTE* src_end   = src + src_len;
    const BYTE* dst_start = dst;
    const BYTE* dst_end   = dst + dst_cap;

    while (src < src_end) {
        /* Token */
        BYTE token = *src++;
        DWORD lit_len   = token >> 4;
        DWORD match_len = token & 0x0F;

        /* Longueur des literaux etendue */
        if (lit_len == 15) {
            BYTE extra;
            do {
                if (src >= src_end) return -1;
                extra = *src++;
                lit_len += extra;
            } while (extra == 255);
        }

        /* Copier les literaux */
        if (src + lit_len > src_end)  return -1;
        if (dst + lit_len > dst_end)  return -1;
        {
            DWORD i;
            for (i = 0; i < lit_len; i++)
                dst[i] = src[i];
        }
        src += lit_len;
        dst += lit_len;

        /* Derniere sequence : pas de match */
        if (src >= src_end)
            break;

        /* Offset (2 bytes little-endian) */
        if (src + 2 > src_end) return -1;
        {
            DWORD offset = (DWORD)src[0] | ((DWORD)src[1] << 8);
            BYTE* match_src;

            src += 2;
            if (offset == 0) return -1;

            /* Longueur du match etendue */
            if (match_len == 15) {
                BYTE extra;
                do {
                    if (src >= src_end) return -1;
                    extra = *src++;
                    match_len += extra;
                } while (extra == 255);
            }
            match_len += 4;  /* minimum match = 4 */

            /* Copier le match (peut chevaucher — byte par byte) */
            match_src = dst - offset;
            if (match_src < dst_start) return -1;
            if (dst + match_len > dst_end) return -1;
            {
                DWORD i;
                for (i = 0; i < match_len; i++)
                    dst[i] = match_src[i];
            }
            dst += match_len;
        }
    }

    return (int)(dst - dst_start);
}
