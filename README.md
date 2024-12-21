# aim - ADB Improved

`aim` is a modern, user-friendly command-line interface for Android Debug Bridge (ADB). It enhances the ADB experience with better output formatting, intuitive commands, and improved usability.

## Features

- 🎨 Colored and formatted output for better readability
- 📱 Smart device selection with partial ID matching
- 📊 Multiple output formats (table, json, plain) for easy integration
- 🔍 Improved device information display with detailed properties
- ⚡ Faster common operations with parallel processing
- 🎯 Intuitive command structure and aliases
- 🔄 Interactive modes for continuous operations
- 📝 Detailed logging and debugging capabilities
- 🌈 Customizable configuration and environment settings

## Requirements

- Rust 1.70.0 or higher
- ADB (Android Debug Bridge) installed and in PATH
- USB debugging enabled on Android device

## Installation

From crates.io:

```bash
cargo install aim
```

From source:

```bash
git clone https://github.com/yourusername/aim.git
cd aim
cargo install --path .
```

## Quick Start

1. Connect your Android device via USB
2. Enable USB debugging on your device
3. Run `aim ls` to verify device connection
4. Start using aim commands!

## Basic Usage

### List Connected Devices

```bash
aim ls
```

This will show a nicely formatted table of all connected devices with their details. You can change the output format:

```bash
aim ls -o json    # JSON output
aim ls -o plain   # Plain text output
aim ls -o table   # Table format (default)
```

Example output:

```
DEVICE ID    BRAND     MODEL       STATUS     NAME
abc123      Google    Pixel 6     device     work-phone
def456      Samsung   Galaxy S21  device     personal
```

### Device Properties

```bash
aim getprop                    # Get all properties
aim getprop ro.product.model   # Get specific property
aim getprop ro.product.*       # Get properties by pattern
```

Properties are displayed with colored output for better readability:

```
ro.product.model: Pixel 6
ro.product.brand: Google
ro.product.name: raven
```

### Device Logs

```bash
aim dmesg              # View kernel logs
aim logcat             # View application logs
aim logcat -p ERROR    # Filter by priority
aim logcat -f myapp    # Filter by tag
```

### Screenshots and Screen Recording

```bash
aim screenshot                     # Take a screenshot
aim screenshot -o screen.png       # Save to specific file
aim screenshot -i                  # Interactive mode

aim screenrecord                   # Record screen
aim screenrecord -o video.mp4      # Save to specific file
aim screenrecord -t 10            # Record for 10 seconds
```

### File Operations

```bash
aim push local_file.txt /sdcard/   # Push file to device
aim pull /sdcard/file.txt ./       # Pull file from device
aim push -r local_dir/ /sdcard/    # Push directory recursively
```

## Comparison with ADB

Here's how `aim` commands compare to their `adb` counterparts:

| Operation       | aim                            | adb                                        | Benefits                                                                   |
| --------------- | ------------------------------ | ------------------------------------------ | -------------------------------------------------------------------------- |
| List devices    | `aim ls`                       | `adb devices -l`                           | ✓ Better formatting<br>✓ More device details<br>✓ Multiple output formats  |
| Get property    | `aim getprop ro.product.model` | `adb shell getprop ro.product.model`       | ✓ Colored output<br>✓ Pattern matching<br>✓ Faster retrieval               |
| Take screenshot | `aim screenshot`               | `adb exec-out screencap -p > screen.png`   | ✓ Automatic file handling<br>✓ Interactive mode<br>✓ Custom save locations |
| Record screen   | `aim screenrecord`             | `adb shell screenrecord /sdcard/video.mp4` | ✓ Progress tracking<br>✓ Automatic file transfer<br>✓ Time limit option    |
| View logs       | `aim logcat`                   | `adb logcat`                               | ✓ Better filtering<br>✓ Colored output<br>✓ Priority levels                |
| Push file       | `aim push file.txt /sdcard/`   | `adb push file.txt /sdcard/`               | ✓ Progress bar<br>✓ Recursive transfer<br>✓ Better error handling          |
| Pull file       | `aim pull /sdcard/file.txt ./` | `adb pull /sdcard/file.txt ./`             | ✓ Progress tracking<br>✓ Multiple file support<br>✓ Auto-retry             |

## Advanced Features

### Device Selection

When multiple devices are connected, you can specify the target device in several ways:

```bash
aim -d 1234 ls                     # By device ID (full or partial)
aim -d pixel ls                    # By device name
aim -d 192.168.1.100:5555 ls      # By network address
aim -d work-phone ls              # By custom alias
```

### Output Formatting

Most commands support multiple output formats:

```bash
aim ls -o json          # JSON output (colorized)
aim getprop -o plain    # Plain text (with colors)
aim dmesg -o table      # Table format
```

### Filtering and Search

```bash
aim logcat -p ERROR                 # Filter by priority (ERROR, WARN, INFO)
aim getprop ro.product             # Filter properties by prefix
aim dmesg -f "USB"                 # Filter kernel logs
aim logcat -t "1h"                 # Show last hour of logs
```

### Interactive Mode

Some commands support interactive mode for continuous operation:

```bash
aim screenshot -i    # Take screenshots with spacebar
aim logcat -i       # Interactive log viewer with filtering
aim shell -i        # Interactive shell session
```

## Environment Variables

- `AIM_DEFAULT_OUTPUT`: Set default output format (json, plain, table)
- `AIM_CONFIG`: Path to custom config file
- `AIM_LOG_LEVEL`: Set log level (error, warn, info, debug)
- `AIM_SCREENSHOT_DIR`: Default directory for screenshots
- `AIM_RECORD_DIR`: Default directory for screen recordings
- `AIM_DEFAULT_DEVICE`: Default device to use
- `AIM_COLOR_MODE`: Control color output (auto, always, never)

## Configuration

Create `~/.config/aim/config.toml` to customize behavior:

```toml
# Device aliases and settings
[devices]
pixel = { name = "Pixel 6", id = "abc123" }
tablet = { name = "Galaxy Tab", id = "def456" }

# Screenshot settings
[screenshot]
output = "~/Pictures/Screenshots"
format = "png"
quality = 100

# Screen recording settings
[screenrecord]
output = "~/Videos/Screenrecords"
bitrate = "4M"
size = "720p"

# Command aliases
[aliases]
props = "getprop ro.product"
logs = "logcat -p ERROR"
cap = "screenshot -i"
```

## Tips and Tricks

1. Use tab completion for commands and options
2. Set aliases for frequently used commands
3. Use `-v` flag for verbose output when debugging issues
4. Combine with pipes for advanced filtering: `aim logcat | grep "ERROR"`
5. Use pattern matching in property searches: `aim getprop "ro.product.*"`
6. Set up device aliases for quick access
7. Use interactive mode for repeated operations
8. Configure default directories for screenshots and recordings

## Common Issues

### Device Not Found

- Ensure USB debugging is enabled in Developer Options
- Check USB cable connection and try a different cable
- Verify device is authorized for debugging
- Try restarting ADB server: `aim server restart`
- Check device shows up in `adb devices`

### Permission Denied

- Check ADB user permissions
- Verify device authorization status
- Run `aim server restart` if needed
- Try running with sudo (Linux/macOS)
- Check SELinux settings on device

### Performance Issues

- Use filtered output when possible
- Avoid pulling large files over USB 2.0
- Consider using a USB 3.0 cable
- Use network connection for large file transfers
- Enable compression for file transfers

## Contributing

Contributions are welcome! Please check our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Clone your fork
3. Install development dependencies
4. Create a new branch
5. Make your changes
6. Run tests: `cargo test`
7. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details

## Acknowledgments

- Android Debug Bridge (ADB) team
- Rust community
- All contributors to this project
