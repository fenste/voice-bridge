# TeamSpeak 3 ↔ Discord Voice Bridge

A real-time audio bridge between TeamSpeak 3 and Discord voice channels using Rust.

**Status:** Migrated to modern Poise framework with automated builds for multiple platforms.

## About This Fork

This is a modernized fork of the original [voice-bridge](https://github.com/0xpr03/voice-bridge) by [0xpr03](https://github.com/0xpr03).

**Major improvements in this fork:**
- Migrated from deprecated StandardFramework to modern Poise
- Modern Discord slash commands
- Volume control commands
- Ephemeral command responses (no channel spam)
- Graceful shutdown handling
- Cleaned up all deprecation warnings
- Cross-platform support

**All credit for the original architecture and core functionality goes to the original author.**

## Features
- Bidirectional audio relay between TeamSpeak and Discord
- Volume control with `/volume` commands
- Modern slash commands with ephemeral responses
- Audio queue management
- Graceful shutdown handling
- Automated multi-platform builds via GitHub Actions
- Works on Windows, Linux, and Raspberry Pi

## Quick Start

**TL;DR - What you need:**

| Platform | Pre-built Binary | Build from Source |
|----------|-----------------|-------------------|
| **Windows** | FFmpeg, VC++ Redist | + Rust, Visual Studio Build Tools |
| **Linux** | FFmpeg | + pkg-config, libopus-dev, libssl-dev, build-essential |
| **Raspberry Pi** | FFmpeg | + Same as Linux (not recommended - very slow) |

**Note:** Pre-built binaries have OpenSSL and Opus **statically compiled in**. You only need pkg-config, libopus-dev, and libssl-dev if you're building from source on Linux.

---

## Installation

### Windows

#### Prerequisites

Before running the bot, you need to install:

**Required:**
1. **FFmpeg** - For audio processing
   - Download from [https://www.gyan.dev/ffmpeg/builds/](https://www.gyan.dev/ffmpeg/builds/)
   - Get "ffmpeg-release-essentials.zip"
   - Extract to `C:\ffmpeg\`
   - Add `C:\ffmpeg\bin\` to your System PATH:
     1. Press `Win + X` → System → Advanced system settings
     2. Click "Environment Variables"
     3. Under "System variables", find "Path" and click "Edit"
     4. Click "New" and add: `C:\ffmpeg\bin`
     5. Click OK on all windows
   - Verify: Open a new Command Prompt and run `ffmpeg -version`

**If you get DLL errors:**
2. **Visual C++ Redistributable**
   - Download: [https://aka.ms/vs/17/release/vc_redist.x64.exe](https://aka.ms/vs/17/release/vc_redist.x64.exe)
   - Install with default settings

#### Option 1: Download Pre-built Binary (Recommended)

1. Go to [GitHub Actions](https://github.com/fenste/voice-bridge/actions)
2. Click the latest successful workflow run (green checkmark ✅)
3. Scroll down to **Artifacts**
4. Download `voice_bridge-windows-x64`
5. Extract the ZIP file to a folder (e.g., `C:\voice-bridge\`)
6. You should have `voice_bridge.exe`

#### Option 2: Build from Source

**Prerequisites:**
1. **Rust** - Install from [https://rustup.rs/](https://rustup.rs/)
   - Use default settings (MSVC toolchain)
   
2. **Visual Studio Build Tools** - Required for compiling
   - Download: [https://visualstudio.microsoft.com/downloads/](https://visualstudio.microsoft.com/downloads/)
   - Scroll to "Tools for Visual Studio"
   - Download "Build Tools for Visual Studio 2022"
   - During installation, select "Desktop development with C++"
   - This provides: MSVC compiler, Windows SDK, and C++ build tools

3. **FFmpeg** - Same as runtime requirement (see above)

**Note:** Unlike Linux, you DON'T need to manually install Opus or OpenSSL development libraries on Windows. Cargo handles these automatically during the build process through the `audiopus` and `openssl` crates (with the `vendored` feature enabled in our Cargo.toml).
```powershell
# Clone the repository
git clone https://github.com/fenste/voice-bridge
cd voice-bridge

# Build (this will download and compile dependencies automatically)
cargo build --release

# Binary will be at: target\release\voice_bridge.exe
```

**Build time:** First build takes 10-20 minutes as it compiles all dependencies.

#### Configuration (Windows)

1. Copy `credentials.example.toml` to `.credentials.toml` in the same folder as `voice_bridge.exe`
2. Edit `.credentials.toml` with Notepad:
```toml
discord_token = "YOUR_DISCORD_BOT_TOKEN"
teamspeak_server = "your.server.address:9987"
teamspeak_identity = "YOUR_TS_IDENTITY_STRING"

# Optional: Auto-join specific TeamSpeak channel
teamspeak_channel_name = "Your Channel Name"
# OR use channel ID:
# teamspeak_channel_id = 5

# Optional settings
teamspeak_server_password = "server_password"
teamspeak_channel_password = "channel_password"
teamspeak_name = "VoiceBridge Bot"
verbose = 0  # 0-3, higher = more logs
volume = 1.0  # Default volume (0.0-2.0)
```

#### Running on Windows

**Option A: Double-click** `voice_bridge.exe`

**Option B: Run from PowerShell/CMD:**
```powershell
cd C:\path\to\voice-bridge
.\voice_bridge.exe
```

**To stop:** Press `Ctrl+C` in the window

#### Windows Logging

To enable debug logging on Windows:

**PowerShell:**
```powershell
$env:RUST_LOG="info"
.\voice_bridge.exe
```

**Command Prompt:**
```cmd
set RUST_LOG=info
voice_bridge.exe
```

#### Windows-Specific Notes

- **Antivirus Warning:** Some antivirus software may flag the executable. This is a false positive - you can safely add an exception.
- **Firewall:** Windows may ask for firewall permission. Click "Allow" for both private and public networks.
- **Audio Devices:** Make sure Windows hasn't muted the application in Volume Mixer (right-click speaker icon → Open Volume mixer).

---

### Linux (x86_64)

#### Prerequisites

**Runtime Dependencies (Required):**
```bash
# Debian/Ubuntu
sudo apt update
sudo apt install ffmpeg

# Fedora/RHEL
sudo dnf install ffmpeg

# Arch Linux
sudo pacman -S ffmpeg
```

**Note:** OpenSSL and Opus are statically compiled into the pre-built binary. You don't need to install them separately if using the pre-built binary.

#### Option 1: Download Pre-built Binary (Recommended)

1. Go to [GitHub Actions](https://github.com/fenste/voice-bridge/actions)
2. Click the latest successful workflow run
3. Download `voice_bridge-linux-x64`
4. Extract and make executable:
```bash
unzip voice_bridge-linux-x64.zip
chmod +x voice_bridge
```

#### Option 2: Build from Source

**Build Dependencies (only needed for compilation):**
```bash
# Debian/Ubuntu
sudo apt update
sudo apt install pkg-config libopus-dev libssl-dev build-essential

# Fedora/RHEL
sudo dnf install pkg-config opus-devel openssl-devel gcc

# Arch Linux
sudo pacman -S pkg-config opus openssl base-devel
```

**Build:**
```bash
git clone https://github.com/fenste/voice-bridge
cd voice-bridge
cargo build --release
# Binary at: target/release/voice_bridge
```

#### Configuration (Linux)

Create `.credentials.toml` in the same directory as the binary:
```bash
cp credentials.example.toml .credentials.toml
nano .credentials.toml  # or use your preferred editor
```
```toml
discord_token = "YOUR_DISCORD_BOT_TOKEN"
teamspeak_server = "your.server.address:9987"
teamspeak_identity = "YOUR_TS_IDENTITY_STRING"

# Optional: Auto-join specific TeamSpeak channel
teamspeak_channel_name = "Your Channel Name"
# OR use channel ID:
# teamspeak_channel_id = 5

# Optional settings
teamspeak_server_password = "server_password"
teamspeak_channel_password = "channel_password"
teamspeak_name = "VoiceBridge Bot"
verbose = 0  # 0-3, higher = more logs
volume = 1.0  # Default volume (0.0-2.0)
```

#### Running on Linux
```bash
./voice_bridge
```

**To stop:** Press `Ctrl+C`

#### Running as a Service (systemd)

Create `/etc/systemd/system/voice-bridge.service`:
```ini
[Unit]
Description=TeamSpeak Discord Voice Bridge
After=network.target

[Service]
Type=simple
User=fenste
WorkingDirectory=/path/to/voice-bridge
ExecStart=/path/to/voice-bridge/voice_bridge
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable voice-bridge
sudo systemctl start voice-bridge

# Check status
sudo systemctl status voice-bridge

# View logs
journalctl -u voice-bridge -f
```

---

### Raspberry Pi (ARM64)

#### Prerequisites

**Runtime Dependencies (Required):**
```bash
# FFmpeg for audio processing
sudo apt update
sudo apt install ffmpeg
```

**Note:** The pre-built binary has OpenSSL and Opus statically compiled in. No need to install additional libraries!

#### Option 1: Download Pre-built Binary (Recommended)

Perfect for Raspberry Pi 4 with 64-bit OS!

1. Go to [GitHub Actions](https://github.com/fenste/voice-bridge/actions)
2. Click the latest successful workflow run
3. Download `voice_bridge-linux-arm64`
4. On your Pi:
```bash
unzip voice_bridge-linux-arm64.zip
chmod +x voice_bridge
```

#### Option 2: Build from Source (Not Recommended - Very Slow!)

⚠️ Building on a Raspberry Pi can take 30-60 minutes. **Use the pre-built binary instead!**

If you really must build from source:

**Build Dependencies:**
```bash
sudo apt update
sudo apt install pkg-config libopus-dev libssl-dev build-essential

# Clone and build
git clone https://github.com/fenste/voice-bridge
cd voice-bridge
cargo build --release
```

#### Configuration (Raspberry Pi)

Same as Linux - create `.credentials.toml`:
```bash
cp credentials.example.toml .credentials.toml
nano .credentials.toml
```
```toml
discord_token = "YOUR_DISCORD_BOT_TOKEN"
teamspeak_server = "your.server.address:9987"
teamspeak_identity = "YOUR_TS_IDENTITY_STRING"

# Optional: Auto-join specific TeamSpeak channel
teamspeak_channel_name = "Your Channel Name"

# Optional settings
teamspeak_name = "VoiceBridge Bot"
verbose = 0
volume = 1.0
```

#### Running on Raspberry Pi
```bash
./voice_bridge
```

**To stop:** Press `Ctrl+C`

**Running on boot:** Use the systemd service setup from the Linux section above.

---

## Usage

### Starting the Bot

**Windows:** Double-click `voice_bridge.exe` or run from PowerShell/CMD

**Linux/Raspberry Pi:** `./voice_bridge`

The bot will:
1. Connect to your Discord server
2. Connect to your TeamSpeak server
3. Auto-join the configured TeamSpeak channel (if specified)
4. Wait for Discord `/join` command

### Discord Commands

All commands respond only to you (ephemeral messages):

- `/join <channel>` - Join a Discord voice channel
- `/leave` - Leave the Discord voice channel
- `/volume <0.0-2.0>` - Set output volume (1.0 = normal, 2.0 = double)
- `/volume_check` - Check current volume level
- `/mute` / `/unmute` - Mute/unmute bot microphone
- `/deafen` / `/undeafen` - Deafen/undeafen bot
- `/reset_audio` - Reset audio queues (if audio gets stuck)
- `/ping` - Test bot responsiveness

### Stopping the Bot

**All Platforms:** Press `Ctrl+C` for graceful shutdown

The bot will:
1. Disconnect from all Discord voice channels
2. Disconnect from TeamSpeak
3. Exit cleanly

---

## Debugging

### Enable Backtrace

For detailed error traces:

**Linux/Raspberry Pi:**
```bash
RUST_BACKTRACE=1 ./voice_bridge
```

**Windows PowerShell:**
```powershell
$env:RUST_BACKTRACE="1"
.\voice_bridge.exe
```

**Windows CMD:**
```cmd
set RUST_BACKTRACE=1
voice_bridge.exe
```

### Control Logging Level

Set the `RUST_LOG` environment variable:

**Linux/Raspberry Pi:**
```bash
# Show only errors (quietest)
RUST_LOG=error ./voice_bridge

# Show info, warnings, and errors (recommended)
RUST_LOG=info ./voice_bridge

# Show debug information
RUST_LOG=debug ./voice_bridge

# Custom per-crate logging
RUST_LOG="error,voice_bridge=info" ./voice_bridge
```

**Windows PowerShell:**
```powershell
$env:RUST_LOG="info"
.\voice_bridge.exe
```

**Windows CMD:**
```cmd
set RUST_LOG=info
voice_bridge.exe
```

### Common Issues

**"Out of order command packet" warnings (TeamSpeak):**

These are harmless UDP packet ordering issues. Suppress with:

**Linux/Pi:** `RUST_LOG="error,tsclientlib=error" ./voice_bridge`

**Windows PowerShell:** 
```powershell
$env:RUST_LOG="error,tsclientlib=error"
.\voice_bridge.exe
```

**Audio not playing:**
- Ensure bot has "Connect" and "Speak" permissions in Discord
- Check that you're in the same voice channel as the bot
- Try `/reset_audio` to clear stuck queues
- On Windows, check Windows sound settings aren't blocking the app

**Windows: "VCRUNTIME140.dll is missing":**
Install [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)

**Windows: "ffmpeg not found":**
- Make sure FFmpeg is installed and in your PATH
- Open a new Command Prompt and verify with `ffmpeg -version`
- If it doesn't work, restart your computer after adding to PATH

**Windows: Build fails with "linker not found":**
Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with "Desktop development with C++"

**Windows: Antivirus blocks the executable:**
Add an exception for `voice_bridge.exe` in your antivirus settings

**Raspberry Pi: "Illegal instruction":**
Make sure you downloaded the ARM64 binary and are running 64-bit Raspberry Pi OS

---

## Development

### Build Optimization

The default release build is heavily optimized using:
- Native target-cpu instructions
- Link-Time Optimization (LTO)
- Vendored OpenSSL for portability

You can disable LTO in `Cargo.toml` under `[profile.release]` to reduce build time. Target-cpu flags can be disabled in `.cargo/config.toml`.

### Cross-Compilation

GitHub Actions automatically builds for all platforms. To build locally:

**For Raspberry Pi from Linux:**
```bash
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

**For Windows from Linux:**
```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## License

voice_bridge is distributed under the terms of the AGPL license (Version 3.0).

Libraries specified by Cargo.toml and code annotated otherwise are copyright by their respective authors.

See LICENSE-AGPL for details.

---

## Credits

**Original Project:** [voice-bridge](https://github.com/0xpr03/voice-bridge) by [0xpr03](https://github.com/0xpr03)

### Libraries & Dependencies
- [tsclientlib](https://github.com/ReSpeak/tsclientlib) - TeamSpeak 3 client library
- [Serenity](https://github.com/serenity-rs/serenity) - Discord API library
- [Songbird](https://github.com/serenity-rs/songbird) - Discord voice client
- [Poise](https://github.com/serenity-rs/poise) - Discord slash command framework