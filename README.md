# aim (Adb IMproved)

A modern command-line interface for ADB (Android Debug Bridge) that provides a more user-friendly and efficient way to interact with Android devices.

## Features

- 🔍 Smart device selection with partial ID matching
- 📋 Clean, formatted output in table, JSON, or plain text
- 🏷️ Memorable device names generated automatically
- ⚡ Parallel property fetching for faster operations
- 🔧 Customizable aliases for frequently used commands
- 💻 Cross-platform support (Linux, macOS, Windows)

## Installation

```bash
cargo install aim
```

## Quick Start

List all connected devices:

```bash
aim
```

Get a property from a device:

```bash
aim getprop ro.product.model
```

## Supported ADB Commands

The following ADB commands are supported by `aim`:

- [x] `devices` - List connected devices
- [ ] `shell` - Run shell commands on device
- [ ] `install` - Install an APK
- [ ] `uninstall` - Remove an app
- [.] `push` - Copy files to device
- [.] `pull` - Copy files from device
- [ ] `logcat` - View device logs
- [.] `getprop` - Get device properties
- [ ] `reboot` - Restart device
- [ ] `backup` - Backup device data (planned)
- [ ] `restore` - Restore device backup (planned)
- [ ] `sideload` - Install OTA update (planned)

## Configuration

Create `~/.aimconfig` (or `%APPDATA%\aim\config.toml` on Windows):

```toml
[alias]
lsj = "ls --output json"
brand = "getprop ro.product.brand"
```

## License

MIT
