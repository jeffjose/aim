# AIM Commands Reference

## Implemented Commands

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `aim ls` | List connected devices | `aim ls -o json` |
| `aim run <cmd>` | Run shell command on device | `aim run ls /sdcard` |
| `aim copy <src> <dst>` | Copy files to/from device | `aim copy photo.jpg device:/sdcard/` |
| `aim rename <device> <name>` | Set device alias | `aim rename abc123 work-phone` |
| `aim server <op>` | Manage ADB server | `aim server restart` |
| `aim adb <args>` | Pass-through to adb | `aim adb shell` |
| `aim config` | Display configuration | `aim config` |
| `aim dmesg` | View kernel logs | `aim dmesg` |
| `aim perfetto` | Run perfetto trace | `aim perfetto -t 10` |
| `aim screenrecord` | Record device screen | `aim screenrecord -t 30` |
| `aim getprop [props]` | Get device properties | `aim getprop ro.product.model` |
| `aim screenshot` | Take screenshot | `aim screenshot -o photo.png` |

### App Commands

| Command | Description | Example |
|---------|-------------|---------|
| `aim app list` | List installed apps | `aim app list --user` |
| `aim app clear <pkg>` | Clear app data | `aim app clear com.example` |
| `aim app pull <pkg>` | Extract APK | `aim app pull com.spotify` |
| `aim app backup <pkg>` | Backup app data | `aim app backup com.example` |
| `aim app start <pkg>` | Start app | `aim app start com.spotify` |
| `aim app stop <pkg>` | Force stop app | `aim app stop com.example` |

## Command Details

### `aim ls`

List connected devices with formatted output.

```bash
aim ls              # Table format
aim ls -o json      # JSON output
aim ls -o plain     # Plain text
```

Output:
```
DEVICE ID    BRAND     MODEL       STATUS     NAME
abc123      Google    Pixel 6     device     work-phone
def456      Samsung   Galaxy S21  device     personal
```

### `aim getprop`

Get device properties with pattern matching.

```bash
aim getprop                     # All properties
aim getprop ro.product.model    # Single property
aim getprop "ro.product.*"      # Pattern match
aim getprop -o json             # JSON output
```

### `aim screenshot`

Take device screenshot.

```bash
aim screenshot                  # Auto-named file
aim screenshot -o photo.png     # Specific file
aim screenshot -i               # Interactive mode (space to capture)
```

### `aim screenrecord`

Record device screen.

```bash
aim screenrecord                # Default recording
aim screenrecord -t 30          # 30 seconds
aim screenrecord -o video.mp4   # Specific file
```

### `aim copy`

Copy files to/from device.

```bash
# Push to device
aim copy local.txt device:/sdcard/
aim copy -r folder/ device:/sdcard/

# Pull from device
aim copy device:/sdcard/photo.jpg ./
```

### `aim server`

Manage ADB server.

```bash
aim server status    # Check status (default)
aim server start     # Start server
aim server stop      # Stop server
aim server restart   # Restart server
```

### `aim app list`

List installed applications.

```bash
aim app list              # All apps
aim app list --user       # User apps only
aim app list --system     # System apps only
aim app list -o json      # JSON output
```

## Global Options

| Option | Description |
|--------|-------------|
| `-d, --device <ID>` | Target specific device (partial match supported) |
| `-v` | Verbose output (WARN level) |
| `-vv` | More verbose (INFO level) |
| `-vvv` | Debug output (DEBUG level) |

## Output Formats

Most commands support `-o, --output` with values:
- `table` - Formatted table (default)
- `json` - JSON for scripting
- `plain` - Simple text

---

## Future Commands (Ideas)

### Device Control

| Command | Description |
|---------|-------------|
| `aim device info` | All device info in one place |
| `aim device battery` | Battery status |
| `aim device storage` | Storage information |
| `aim device reboot [mode]` | Smart reboot |

### Network Operations

| Command | Description |
|---------|-------------|
| `aim net connect <ip>` | Connect via WiFi ADB |
| `aim net disconnect` | Disconnect WiFi |
| `aim net forward <local> <remote>` | Port forwarding |
| `aim net wifi` | Show WiFi info |

### UI Automation

| Command | Description |
|---------|-------------|
| `aim ui tap <x> <y>` | Tap screen |
| `aim ui swipe <dir>` | Swipe gesture |
| `aim ui text <string>` | Type text |
| `aim ui key <keycode>` | Send keycode |

### Improvements to Existing

| Current | Proposed |
|---------|----------|
| `aim ls` | Add `--watch` for auto-refresh |
| `aim ls` | Add `--wait` to wait for device |
| `aim screenshot` | Add `--delay 5` for delayed capture |
| `aim screenrecord` | Add `--gif` output |
| `aim logcat` | Add `--since 5m` time filter |
| `aim logcat` | Add `--package <app>` filter |
