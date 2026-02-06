# Code Style & Conventions

## C Style
- Pure C (C99/C11), no C++ features
- No CRT includes — everything from scratch
- Custom Windows typedefs in `types.h` (DWORD, PVOID, etc.)
- Function prefix by module: `peb_*`, `my_*`, `chacha20_*`
- Snake_case for functions, UPPER_CASE for constants/macros
- Comments in French or English (project is educational, French-speaking author)
- Block comments with `/* */`, single-line with `/* */` as well

## Naming
- Hash constants: `HASH_FUNCTIONNAME` (e.g., `HASH_GETPROCADDRESS`)
- API function typedefs: `fn_FunctionName` (e.g., `fn_ExitProcess`)
- Struct for resolved APIs: `API_TABLE`

## No-CRT Conventions
- Entry point: `Entry()` (not main)
- Utility functions: `my_memcpy`, `my_memset`, `my_strlen`, `my_strcmp`
- Inline assembly with GCC syntax for PEB access: `__asm__ volatile`
- All API resolution via DJB2 hash — no plaintext API names at runtime

## Build Flags
- MinGW: `-Os -nostdlib -fno-stack-protector -fno-ident -Wl,-eEntry`
- MSVC: `/O1 /GS- /NODEFAULTLIB /ENTRY:Entry`
