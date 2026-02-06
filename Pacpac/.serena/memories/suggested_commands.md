# Suggested Commands

## Build (MinGW cross-compilation from macOS)
```bash
make              # Build stub.exe
make clean        # Clean build artifacts
```

## Build (MSVC on Windows)
```cmd
build.bat         # From x64 Native Tools Command Prompt
```

## Analysis
```bash
x86_64-w64-mingw32-objdump -h stub.exe          # Show PE sections
x86_64-w64-mingw32-objdump -x stub.exe           # Full PE dump (imports, exports, etc.)
x86_64-w64-mingw32-objdump -d stub.exe            # Disassemble
x86_64-w64-mingw32-strings stub.exe               # Check for leaked strings
```

## System Utils (Darwin/macOS)
```bash
ls -la            # List files
find . -name "*.c"  # Find files
grep -rn "pattern" src/  # Search in source
python3 script.py  # Run Python scripts
```
