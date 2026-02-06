# Suggested Commands for FrameworkC2

## Build Commands

```bash
# Build all packages for Windows x64
cargo build --release --target x86_64-pc-windows-gnu

# Build beacon only
cargo build --release --target x86_64-pc-windows-gnu --package beacon

# Build loader only
cargo build --release --target x86_64-pc-windows-gnu --package loader

# Check compilation without building
cargo check --target x86_64-pc-windows-gnu
```

## Test DLL Compilation

```bash
# Compile test DLL WITHOUT CRT (required for PE loader!)
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test.dll test.c -lkernel32 -luser32

# Minimal DLL (no imports)
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o minimal.dll minimal.c

# Compile all test DLLs
cd test_dll
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_reloc.dll test_reloc.c -lkernel32 -luser32
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_nocrt.dll test_nocrt.c -lkernel32 -luser32
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o test_minimal.dll test_minimal.c
```

## Payload Encryption

```bash
# XOR encrypt with key 0x41 (default)
python3 tools/xor_encrypt.py input.dll output.dll.enc 41

# Encrypt beacon for loader
python3 tools/xor_encrypt.py target/x86_64-pc-windows-gnu/release/beacon.dll beacon.dll.enc 41
```

## Analysis Commands

```bash
# Check DLL imports
x86_64-w64-mingw32-objdump -p file.dll | grep -A 50 "The Import Tables"

# Check DLL exports
x86_64-w64-mingw32-objdump -p file.dll | grep -A 20 "The Export Tables"

# Check entry point and flags
x86_64-w64-mingw32-objdump -f file.dll

# Disassemble entry point
x86_64-w64-mingw32-objdump -d file.dll | grep -A 30 "entry_address:"

# Check if DLL has relocations
x86_64-w64-mingw32-objdump -x file.dll | grep -E "ImageBase|BASERELOC"
```

## Pacpac Build Commands

```bash
# Build Pacpac stub (MinGW cross-compilation from macOS)
cd Pacpac && make

# Clean Pacpac build
cd Pacpac && make clean

# Pacpac PE analysis
x86_64-w64-mingw32-objdump -h Pacpac/stub.exe        # Show PE sections
x86_64-w64-mingw32-objdump -x Pacpac/stub.exe        # Full PE dump (imports, exports)
x86_64-w64-mingw32-strings Pacpac/stub.exe            # Check for leaked strings
```

## Git Commands

```bash
git status
git add .
git commit -m "message"
git push
```
