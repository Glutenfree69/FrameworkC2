#!/usr/bin/env python3
"""
XOR Encryption Tool for C2 Framework
=====================================

Encrypts DLL payloads for the !loaddll command.

Usage:
    python3 xor_encrypt.py <input_file> <output_file> [key]
    python3 xor_encrypt.py beacon.dll beacon.dll.enc
    python3 xor_encrypt.py mimikatz.dll mimikatz.dll.enc deadbeef

Default key: 41 (single byte, same as beacon config default)
"""

import sys
import os

def xor_encrypt(data: bytes, key: bytes) -> bytes:
    """XOR encrypt/decrypt data with a key (symmetric)"""
    return bytes([b ^ key[i % len(key)] for i, b in enumerate(data)])

def hex_to_bytes(hex_str: str) -> bytes:
    """Convert hex string to bytes"""
    return bytes.fromhex(hex_str)

def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    key_hex = sys.argv[3] if len(sys.argv) > 3 else "41"
    
    # Parse key
    try:
        key = hex_to_bytes(key_hex)
    except ValueError:
        print(f"[!] Invalid hex key: {key_hex}")
        sys.exit(1)
    
    # Read input
    if not os.path.exists(input_file):
        print(f"[!] File not found: {input_file}")
        sys.exit(1)
    
    with open(input_file, "rb") as f:
        data = f.read()
    
    print(f"[*] Input:  {input_file} ({len(data)} bytes)")
    print(f"[*] Key:    {key_hex} ({len(key)} bytes)")
    
    # Encrypt
    encrypted = xor_encrypt(data, key)
    
    # Write output
    with open(output_file, "wb") as f:
        f.write(encrypted)
    
    print(f"[+] Output: {output_file} ({len(encrypted)} bytes)")
    print(f"[+] Done!")
    
    # Verify (decrypt and check MZ header)
    decrypted = xor_encrypt(encrypted, key)
    if decrypted[:2] == b"MZ":
        print(f"[+] Verification: OK (MZ header intact after round-trip)")
    else:
        print(f"[!] Warning: Decrypted file doesn't have MZ header")

if __name__ == "__main__":
    main()
