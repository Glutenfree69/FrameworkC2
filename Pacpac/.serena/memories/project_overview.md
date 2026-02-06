# Pacpac — Packer C pour Loader Rust

## Purpose
Educational C packer/stub for a Rust-based C2 framework. The stub decrypts and memory-maps a Rust loader DLL
without it ever touching disk, reducing AV/EDR static analysis exposure.

## Tech Stack
- **Language**: Pure C, no-CRT (no libc, no standard library)
- **Target**: Windows x64 only
- **Build**: MinGW cross-compilation from macOS (`x86_64-w64-mingw32-gcc`) or MSVC on Windows
- **Crypto**: ChaCha20 (RFC 7539) from scratch
- **Packer script**: Python (`pack.py`, to be created)

## Architecture
```
stub.exe (tiny, ~4 KB) → decrypts payload → manual maps loader.dll in-process → calls Run()
```

## Key Design Decisions
- No CRT: custom entry point `Entry()`, own memcpy/memset
- Empty IAT: all APIs resolved via PEB walking + DJB2 hashing
- No syscalls in stub (only in the Rust loader for cross-process injection)
- No ETW/AMSI patching in stub (handled by Rust beacon)
- TLS support required (Rust binaries use TLS callbacks)

## Project Structure
```
include/    - Headers (types.h, peb.h, pe.h, hash.h, chacha20.h, loader.h)
src/        - Source (entry.c, peb.c, utils.c, chacha20.c, pe_loader.c)
payload/    - Generated payload.h (by pack.py)
Makefile    - MinGW cross-compilation
build.bat   - MSVC compilation
pack.py     - Python packer script (to be created)
```
