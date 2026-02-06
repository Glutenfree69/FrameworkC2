#ifndef LZ4_H
#define LZ4_H

#include "types.h"

/* =============================================
 * lz4.h — LZ4 block decompressor (no frame)
 *
 * Compatible avec lz4.block.compress(store_size=False)
 * Retourne le nombre d'octets decompresses, ou -1 si erreur
 * ============================================= */

int lz4_decompress(const BYTE* src, DWORD src_len,
                   BYTE* dst, DWORD dst_cap);

#endif /* LZ4_H */
