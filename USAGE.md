# aim - ADB Improved

`aim` is a modern, user-friendly command-line interface for Android Debug Bridge (ADB). It enhances the ADB experience with better output formatting, intuitive commands, and improved usability.

## Features

- 🎨 Colored and formatted output
- 📱 Smart device selection
- 📊 Multiple output formats (table, json, plain)
- 🔍 Improved device information display
- ⚡ Faster common operations
- 🎯 Intuitive command structure

## Installation

```bash
cargo install aim
```

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

### Device Properties

```bash
aim getprop                    # Get all properties
aim getprop ro.product.model   # Get specific property
```

### Device Logs

```bash
aim dmesg              # View kernel logs
aim logcat             # View application logs
aim logcat -p ERROR    # Filter by priority
```

### Screenshots and Screen Recording

```bash
aim screenshot                     # Take a screenshot
aim screenshot -o screen.png       # Save to specific file
aim screenshot -i                  # Interactive mode

aim screenrecord                   # Record screen
aim screenrecord -o video.mp4      # Save to specific file
```

### File Operations

```bash
aim push local_file.txt /sdcard/   # Push file to device
aim pull /sdcard/file.txt ./       # Pull file from device
```

## Comparison with ADB

Here's how `aim` commands compare to their `adb` counterparts:

| Operation       | aim                            | adb                                        | Notes                                            |
| --------------- | ------------------------------ | ------------------------------------------ | ------------------------------------------------ |
| List devices    | `aim ls`                       | `adb devices -l`                           | aim provides more details and better formatting  |
| Get property    | `aim getprop ro.product.model` | `adb shell getprop ro.product.model`       | aim has colored output and better formatting     |
| Take screenshot | `aim screenshot`               | `adb exec-out screencap -p > screen.png`   | aim handles file saving automatically            |
| Record screen   | `aim screenrecord`             | `adb shell screenrecord /sdcard/video.mp4` | aim manages temporary files and transfer         |
| View logs       | `aim logcat`                   | `adb logcat`                               | aim provides better filtering and colored output |
| Push file       | `aim push file.txt /sdcard/`   | `adb push file.txt /sdcard/`               | aim shows progress bar                           |
| Pull file       | `aim pull /sdcard/file.txt ./` | `adb pull /sdcard/file.txt ./`             | aim shows progress bar                           |

## Advanced Features

### Device Selection

When multiple devices are connected, you can specify the target device in several ways:

```bash
aim -d 1234 ls                     # By device ID
aim -d pixel ls                    # By device name
aim -d 192.168.1.100:5555 ls      # By network address
```

### Output Formatting

Most commands support multiple output formats:

```bash
aim ls -o json          # JSON output
aim getprop -o plain    # Plain text
aim dmesg -o table      # Table format
```

### Filtering and Search

```bash
aim logcat -p ERROR                 # Filter by priority
aim getprop ro.product             # Filter properties by prefix
aim dmesg -f "USB"                 # Filter kernel logs
```

### Interactive Mode

Some commands support interactive mode for continuous operation:

```bash
aim screenshot -i    # Take screenshots with spacebar
aim logcat -i       # Interactive log viewer
```

## Environment Variables

- `AIM_DEFAULT_OUTPUT`: Set default output format (json, plain, table)
- `AIM_CONFIG`: Path to custom config file
- `AIM_LOG_LEVEL`: Set log level (error, warn, info, debug)

## Configuration

Create `~/.config/aim/config.toml` to customize behavior:

```toml
[devices]
pixel = { name = "Pixel 6" }
tablet = { name = "Galaxy Tab" }

[screenshot]
output = "~/Pictures/Screenshots"

[screenrecord]
output = "~/Videos/Screenrecords"
```

## Tips and Tricks

1. Use tab completion for commands and options
2. Set aliases for frequently used commands
3. Use `-v` flag for verbose output when debugging issues
4. Combine with pipes for advanced filtering: `aim logcat | grep "ERROR"`

## Common Issues

### Device Not Found

- Ensure USB debugging is enabled
- Check USB cable connection
- Verify device is authorized for debugging

### Permission Denied

- Check ADB user permissions
- Verify device authorization status
- Run `aim server restart` if needed

## Contributing

Contributions are welcome! Please check our [Contributing Guidelines](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details
