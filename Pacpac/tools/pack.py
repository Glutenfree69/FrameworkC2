#!/usr/bin/env python3
"""
pack.py — Packer script pour Pacpac
Pipeline: input_file → LZ4 compress → ChaCha20 encrypt → include/payload.h

Usage: python3 pack.py <input_file>
       python3 pack.py <input_file> --key <hex_key> --nonce <hex_nonce>
"""

import os
import struct
import secrets
import argparse

import lz4.block # type: ignore

# =============================================
# ChaCha20 (RFC 7539) — identique a src/chacha20.c
# =============================================

def _rotl32(v, n):
    return ((v << n) | (v >> (32 - n))) & 0xFFFFFFFF

def _quarter_round(s, a, b, c, d):
    s[a] = (s[a] + s[b]) & 0xFFFFFFFF; s[d] ^= s[a]; s[d] = _rotl32(s[d], 16)
    s[c] = (s[c] + s[d]) & 0xFFFFFFFF; s[b] ^= s[c]; s[b] = _rotl32(s[b], 12)
    s[a] = (s[a] + s[b]) & 0xFFFFFFFF; s[d] ^= s[a]; s[d] = _rotl32(s[d], 8)
    s[c] = (s[c] + s[d]) & 0xFFFFFFFF; s[b] ^= s[c]; s[b] = _rotl32(s[b], 7)

def _chacha20_block(state):
    w = list(state)
    for _ in range(10):
        _quarter_round(w, 0, 4, 8, 12);  _quarter_round(w, 1, 5, 9, 13)
        _quarter_round(w, 2, 6, 10, 14); _quarter_round(w, 3, 7, 11, 15)
        _quarter_round(w, 0, 5, 10, 15); _quarter_round(w, 1, 6, 11, 12)
        _quarter_round(w, 2, 7, 8, 13);  _quarter_round(w, 3, 4, 9, 14)
    out = b""
    for i in range(16):
        out += struct.pack("<I", (w[i] + state[i]) & 0xFFFFFFFF)
    return out

def chacha20_crypt(data, key, nonce, counter=0):
    """ChaCha20 encrypt/decrypt (RFC 7539)."""
    state = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
    for i in range(8):
        state.append(struct.unpack("<I", key[i*4:(i+1)*4])[0])
    state.append(counter)
    for i in range(3):
        state.append(struct.unpack("<I", nonce[i*4:(i+1)*4])[0])

    result = bytearray()
    offset = 0
    while offset < len(data):
        keystream = _chacha20_block(state)
        block_size = min(64, len(data) - offset)
        for i in range(block_size):
            result.append(data[offset + i] ^ keystream[i])
        offset += block_size
        state[12] = (state[12] + 1) & 0xFFFFFFFF
    return bytes(result)

# =============================================
# Formatage du header C
# =============================================

def bytes_to_c_array(data, name, items_per_line=12):
    """Genere un tableau C a partir de bytes."""
    lines = []
    lines.append(f"static const BYTE {name}[] = {{")
    for i in range(0, len(data), items_per_line):
        chunk = data[i:i+items_per_line]
        hex_vals = ", ".join(f"0x{b:02x}" for b in chunk)
        comma = "," if i + items_per_line < len(data) else ""
        lines.append(f"    {hex_vals}{comma}")
    lines.append("};")
    return "\n".join(lines)

def generate_payload_h(encrypted_data, key, nonce, original_size, compressed_size):
    """Genere le contenu de payload.h."""
    header = f"""#ifndef PAYLOAD_H
#define PAYLOAD_H

#include "types.h"

/* =============================================
 * payload.h — Genere par pack.py
 *
 * Pipeline: input -> LZ4 compress -> ChaCha20 encrypt
 * Original size:   {original_size} bytes
 * Compressed size: {compressed_size} bytes
 * Encrypted size:  {len(encrypted_data)} bytes
 * Ratio:           {compressed_size * 100 / original_size:.1f}%
 * ============================================= */

#define PAYLOAD_ORIGINAL_SIZE   {original_size}
#define PAYLOAD_COMPRESSED_SIZE {compressed_size}
#define PAYLOAD_ENCRYPTED_SIZE  {len(encrypted_data)}

{bytes_to_c_array(key, "payload_key")}

{bytes_to_c_array(nonce, "payload_nonce")}

{bytes_to_c_array(encrypted_data, "payload_data")}

#endif /* PAYLOAD_H */
"""
    return header

# =============================================
# Main
# =============================================

def main():
    parser = argparse.ArgumentParser(description="Pacpac packer — LZ4 + ChaCha20")
    parser.add_argument("input", help="Input file to pack (e.g. loader.dll)")
    parser.add_argument("-o", "--output", default="include/payload.h",
                        help="Output header path (default: include/payload.h)")
    parser.add_argument("--key", help="ChaCha20 key as hex (64 hex chars). Random if omitted.")
    parser.add_argument("--nonce", help="ChaCha20 nonce as hex (24 hex chars). Random if omitted.")
    args = parser.parse_args()

    # Read input
    with open(args.input, "rb") as f:
        raw = f.read()
    original_size = len(raw)
    print(f"[*] Input:      {args.input} ({original_size} bytes)")

    # LZ4 compress (raw block, no frame header)
    compressed = lz4.block.compress(raw, store_size=False)
    compressed_size = len(compressed)
    ratio = compressed_size * 100 / original_size if original_size > 0 else 0
    print(f"[*] Compressed: {compressed_size} bytes ({ratio:.1f}%)")

    # ChaCha20 key + nonce
    if args.key:
        key = bytes.fromhex(args.key)
        assert len(key) == 32, "Key must be 32 bytes (64 hex chars)"
    else:
        key = secrets.token_bytes(32)

    if args.nonce:
        nonce = bytes.fromhex(args.nonce)
        assert len(nonce) == 12, "Nonce must be 12 bytes (24 hex chars)"
    else:
        nonce = secrets.token_bytes(12)

    print(f"[*] Key:        {key.hex()}")
    print(f"[*] Nonce:      {nonce.hex()}")

    # Encrypt
    encrypted = chacha20_crypt(compressed, key, nonce, counter=0)
    print(f"[*] Encrypted:  {len(encrypted)} bytes")

    # Verify round-trip
    decrypted = chacha20_crypt(encrypted, key, nonce, counter=0)
    decompressed = lz4.block.decompress(decrypted, uncompressed_size=original_size)
    assert decompressed == raw, "Round-trip verification FAILED!"
    print(f"[+] Round-trip verification OK")

    # Generate payload.h
    payload_h = generate_payload_h(encrypted, key, nonce, original_size, compressed_size)
    os.makedirs(os.path.dirname(args.output) or ".", exist_ok=True)
    with open(args.output, "w") as f:
        f.write(payload_h)
    print(f"[+] Written:    {args.output}")

if __name__ == "__main__":
    main()
