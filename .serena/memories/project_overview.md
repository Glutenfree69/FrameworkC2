# FrameworkC2 - Project Overview

## Purpose
Educational C2 (Command & Control) framework in Rust demonstrating:
- PE parsing and reflective loading
- Indirect syscalls for API evasion
- Discord-based C2 communication
- AMSI/ETW bypass techniques

## Tech Stack
- **Language**: Rust (with some C for test DLLs)
- **Target**: Windows x64 only
- **Cross-compilation**: From macOS/Linux using `x86_64-pc-windows-gnu`
- **Dependencies**: winapi, reqwest (blocking), serde, image

## Project Structure
- `common/` - Shared library (c2_common): PE loader, syscalls, obfuscation
- `beacon_rust/` - Beacon DLL: Discord client, commands (shell, screenshot, loaddll)
- `loader/` - Initial loader EXE: Injects beacon into explorer.exe
- `test_dll/` - Test DLLs for PE loader validation
- `tools/` - Helper scripts (xor_encrypt.py)

## Key Commands
```bash
# Build everything
cargo build --release --target x86_64-pc-windows-gnu

# Build specific package
cargo build --release --target x86_64-pc-windows-gnu --package beacon

# Encrypt payload
python3 tools/xor_encrypt.py input.dll output.dll.enc 41
```

## Important Technical Notes

### PE Loader (!loaddll command)
- DLLs loaded via `!loaddll` **MUST be compiled without CRT**
- Use: `x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o out.dll in.c -lkernel32`
- Reason: MinGW CRT doesn't support double initialization (beacon already has CRT)
- The loader registers exception tables via `RtlAddFunctionTable` for x64 SEH support
- Duplicate DLL detection via DJB2 hash prevents loading same DLL twice

### Code Style
- Rust 2021 edition
- Extensive comments in French
- `#![allow(dead_code)]` used liberally
- Error handling via custom `BeaconError` enum with `thiserror`

### Security Considerations
- Bot token embedded in config.toml (compile-time)
- XOR encryption key configurable in config.toml
- AMSI/ETW bypass via hardware breakpoints (VEH)
