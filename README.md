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

## Subcommands Reference

### `ls` - List Devices

Lists all connected Android devices with their details.

```bash
aim ls [options]
```

Options:

- `-o, --output <format>` Output format [default: table]
  - `table`: Formatted table output
  - `json`: JSON output (colorized)
  - `plain`: Plain text output
- `-v, --verbose` Enable verbose output

Examples:

```bash
aim ls                  # Default table output
aim ls -o json         # JSON output
aim ls -o plain        # Plain text output
aim ls -v              # Verbose output with more details
```

### `getprop` - Get Device Properties

Query device properties with optional filtering.

```bash
aim getprop [pattern] [options]
```

Options:

- `-o, --output <format>` Output format [default: plain]
  - `plain`: Key-value pairs with colored output
  - `json`: JSON output (colorized)
  - `table`: Formatted table
- `-d, --device <id>` Target specific device
- `-v, --verbose` Show verbose output

Examples:

```bash
aim getprop                     # List all properties
aim getprop ro.product.model    # Get specific property
aim getprop "ro.product.*"      # Pattern matching
aim getprop -o json            # JSON output
aim getprop -d pixel6          # Query specific device
```

### `dmesg` - Device Kernel Logs

View and filter kernel logs from the device.

```bash
aim dmesg [options]
```

Options:

- `-f, --filter <pattern>` Filter log entries
- `-o, --output <format>` Output format [default: plain]
- `-d, --device <id>` Target specific device
- `-c, --clear` Clear the log after reading
- `-w, --follow` Wait for new messages
- `-t, --time` Show timestamps

Examples:

```bash
aim dmesg                  # Show all kernel logs
aim dmesg -f "USB"        # Filter USB-related logs
aim dmesg -w              # Watch for new logs
aim dmesg -t              # Show with timestamps
aim dmesg -o json         # JSON output
```

### `logcat` - Application Logs

View and filter application logs.

```bash
aim logcat [options]
```

Options:

- `-p, --priority <level>` Filter by priority (V,D,I,W,E,F)
- `-f, --filter <pattern>` Filter by content
- `-t, --tag <tag>` Filter by tag
- `-n, --lines <count>` Number of lines to show
- `-w, --follow` Watch for new logs
- `-c, --clear` Clear the log buffer
- `-o, --output <format>` Output format

Examples:

```bash
aim logcat                    # Show all logs
aim logcat -p ERROR          # Show only errors
aim logcat -t "MyApp"        # Filter by tag
aim logcat -f "Exception"    # Filter by content
aim logcat -n 100            # Show last 100 lines
aim logcat -w               # Watch mode
```

### `screenshot` - Take Screenshots

Capture device screen.

```bash
aim screenshot [options]
```

Options:

- `-o, --output <file>` Output file path
- `-i, --interactive` Interactive mode
- `-d, --device <id>` Target specific device
- `-q, --quality <num>` JPEG quality (1-100)
- `-f, --format <type>` Output format (png/jpg)

Examples:

```bash
aim screenshot                      # Save with timestamp
aim screenshot -o screen.png        # Specific filename
aim screenshot -i                   # Interactive mode
aim screenshot -q 90 -f jpg        # JPEG with quality
```

### `screenrecord` - Record Screen

Record device screen.

```bash
aim screenrecord [options]
```

Options:

- `-o, --output <file>` Output file path
- `-t, --time <seconds>` Recording duration
- `-s, --size <WxH>` Video size (e.g., 1280x720)
- `-b, --bitrate <rate>` Video bitrate (e.g., 4M)
- `-r, --rotate` Rotate 90 degrees

Examples:

```bash
aim screenrecord                    # Default recording
aim screenrecord -t 30             # Record for 30 seconds
aim screenrecord -s 1280x720       # Set resolution
aim screenrecord -b 8M             # Set bitrate
aim screenrecord -o video.mp4      # Custom filename
```

### `push` - Copy Files to Device

Upload files or directories to device.

```bash
aim push [options] <source> <destination>
```

Options:

- `-r, --recursive` Copy directories recursively
- `-p, --progress` Show progress bar
- `-s, --sync` Sync mode (only copy newer files)
- `-d, --device <id>` Target specific device

Examples:

```bash
aim push file.txt /sdcard/           # Push single file
aim push -r local/ /sdcard/remote/   # Push directory
aim push -s backup/ /sdcard/backup/  # Sync directory
aim push -p large_file.zip /sdcard/  # Show progress
```

### `pull` - Copy Files from Device

Download files or directories from device.

```bash
aim pull [options] <source> <destination>
```

Options:

- `-r, --recursive` Copy directories recursively
- `-p, --progress` Show progress bar
- `-s, --sync` Sync mode (only copy newer files)
- `-d, --device <id>` Target specific device

Examples:

```bash
aim pull /sdcard/file.txt ./           # Pull single file
aim pull -r /sdcard/DCIM/ ./photos/    # Pull directory
aim pull -s /sdcard/backup/ ./backup/  # Sync directory
aim pull -p /sdcard/large.zip ./       # Show progress
```

### `shell` - Interactive Shell

Start an interactive shell session on the device.

```bash
aim shell [options] [command]
```

Options:

- `-d, --device <id>` Target specific device
- `-t, --tty` Allocate a TTY
- `-x, --exit` Exit after command execution

Examples:

```bash
aim shell                    # Interactive shell
aim shell ls /sdcard        # Run single command
aim shell -t                # With TTY allocation
aim shell -x "pm list packages"  # Run and exit
```

### `server` - ADB Server Control

Manage the ADB server.

```bash
aim server [command]
```

Commands:

- `start` Start the server
- `stop` Stop the server
- `restart` Restart the server
- `status` Show server status

Examples:

```bash
aim server start     # Start ADB server
aim server stop      # Stop ADB server
aim server restart   # Restart ADB server
aim server status    # Check server status
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
