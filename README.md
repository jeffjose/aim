# aim

A better interface for `adb`. Same functionality, better UX.

## Why?

Because `adb` output is hard to read and commands are inconsistent. `aim` makes common tasks easier without changing how ADB works.

## Install

```bash
cargo install --path .
```

Requirements: Rust 1.70+, ADB in PATH

## Command Comparison

| Task | aim | adb |
|------|-----|-----|
| List devices | `aim ls` | `adb devices -l` |
| Get property | `aim getprop ro.product.model` | `adb shell getprop ro.product.model` |
| Get all properties | `aim getprop` | `adb shell getprop` |
| Pattern match props | `aim getprop "ro.product.*"` | `adb shell getprop \| grep ro.product` |
| Screenshot | `aim screenshot` | `adb exec-out screencap -p > screen.png` |
| Screenshot (specific file) | `aim screenshot -o photo.png` | `adb exec-out screencap -p > photo.png` |
| Screen record | `aim screenrecord` | `adb shell screenrecord /sdcard/video.mp4 && adb pull /sdcard/video.mp4` |
| Screen record (30s) | `aim screenrecord -t 30` | `adb shell screenrecord --time-limit 30 /sdcard/video.mp4 && adb pull /sdcard/video.mp4` |
| Push file | `aim push file.txt /sdcard/` | `adb push file.txt /sdcard/` |
| Pull file | `aim pull /sdcard/file.txt ./` | `adb pull /sdcard/file.txt ./` |
| Push directory | `aim push -r folder/ /sdcard/` | `adb push folder/ /sdcard/` |
| Interactive shell | `aim shell` | `adb shell` |
| Run command | `aim shell ls /sdcard` | `adb shell ls /sdcard` |
| View kernel logs | `aim dmesg` | `adb shell dmesg` |
| View app logs | `aim logcat` | `adb logcat` |
| Filter logs by priority | `aim logcat -p ERROR` | `adb logcat *:E` |
| Clear logs | `aim logcat -c` | `adb logcat -c` |
| Restart ADB server | `aim server restart` | `adb kill-server && adb start-server` |
| Check server status | `aim server status` | `adb start-server` |
| Install APK | `aim install app.apk` | `adb install app.apk` |
| Uninstall app | `aim uninstall com.example` | `adb uninstall com.example` |
| List packages | `aim shell pm list packages` | `adb shell pm list packages` |
| Device info | `aim ls -v` | `adb devices -l` |
| Run with specific device | `aim -d pixel screenshot` | `adb -s <full-id> exec-out screencap -p > screen.png` |

## Examples

### List devices

```bash
# Instead of: adb devices -l
aim ls

# Output:
DEVICE ID    BRAND     MODEL       STATUS     NAME
abc123      Google    Pixel 6     device     work-phone
def456      Samsung   Galaxy S21  device     personal
```

### Get properties

```bash
# Instead of: adb shell getprop ro.product.model
aim getprop ro.product.model

# Get all properties matching a pattern
aim getprop "ro.product.*"
```

### Screenshots

```bash
# Instead of: adb exec-out screencap -p > screen.png
aim screenshot

# Interactive mode - press space to capture
aim screenshot -i
```

### Screen recording

```bash
# Instead of: adb shell screenrecord /sdcard/video.mp4 && adb pull /sdcard/video.mp4
aim screenrecord

# Record for 30 seconds
aim screenrecord -t 30
```

### File operations

```bash
# Progress bars for file transfers
aim push largefile.zip /sdcard/
aim pull /sdcard/DCIM/ ./photos/
```

## Device selection

When multiple devices are connected:

```bash
# Partial ID matching
aim -d pixel screenshot    # Matches "pixel6-serial123"
aim -d work screenshot     # Matches device aliased as "work"
```

## Configuration

`~/.config/aim/config.toml`:

```toml
[devices]
work = { id = "28291FDH200001", name = "Pixel 6 (work)" }
personal = { id = "R5CR10ZYZAB", name = "Galaxy S21" }

[screenshot]
output = "~/Pictures/Screenshots"

[screenrecord]
output = "~/Videos"
```

## All commands

- `aim ls` - List devices (with better formatting)
- `aim getprop [pattern]` - Get device properties  
- `aim screenshot` - Take a screenshot
- `aim screenrecord` - Record screen
- `aim push/pull` - Transfer files with progress bars
- `aim dmesg` - View kernel logs
- `aim logcat` - View app logs (with filtering)
- `aim shell [cmd]` - Run shell commands
- `aim server start/stop/restart` - Manage ADB server
- `aim rename <device> <alias>` - Create device aliases

## Output formats

Most commands support `-o json` or `-o plain`:

```bash
aim ls -o json
aim getprop -o json
```

## Contributing

Standard Rust project. Run tests with `cargo test`.

## License

MIT
