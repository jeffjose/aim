# aim - Feature Ideas

Ideas for new subcommands and improvements based on common adb pain points. Subcommands are organized by category (noun) with actions as sub-commands.

## Implementation Status

### ✅ Implemented Commands

**Core Commands:**

- `aim ls` - List connected devices
- `aim run` - Run shell commands on device
- `aim copy` - Copy files to/from device (replaces push/pull)
- `aim rename` - Rename device
- `aim server` - Manage ADB server (start/stop/restart/status)
- `aim adb` - Run arbitrary ADB commands
- `aim config` - Display configuration
- `aim dmesg` - Run dmesg command
- `aim perfetto` - Run perfetto trace
- `aim screenrecord` - Record device screen
- `aim getprop` - Get device properties
- `aim screenshot` - Take screenshots

**App Subcommands (Implemented):**

- `aim app list` - List installed apps
- `aim app clear <package>` - Clear app data
- `aim app pull <package>` - Extract APK from device
- `aim app backup <package>` - Backup app data
- `aim app start <package>` - Start app
- `aim app stop <package>` - Force stop app

### ⏳ Not Yet Implemented App Subcommands

- `aim app info <package>` - Show app details
- `aim app permissions <package>` - List app permissions
- `aim app restore <file>` - Restore app backup
- `aim app uninstall <package>` - Uninstall with options

## New Subcommands

| Proposed Command | Description | Equivalent adb | Why it's better |
|-----------------|-------------|----------------|-----------------|
| **app** | **Application Management** | | |
| `aim app list` | List installed apps | `adb shell pm list packages` | Better formatting, shows app names |
| `aim app info <package>` | Show app details | `adb shell dumpsys package com.example` | Parsed output, key info only |
| `aim app clear <package>` | Clear app data | `adb shell pm clear com.example` | Supports partial package names |
| `aim app pull <package>` | Extract APK from device | `adb shell pm path com.example && adb pull <path>` | One step, automatic renaming |
| `aim app backup <package>` | Backup app data | `adb backup -f backup.ab -apk -obb com.example` | Progress bar, compression |
| `aim app restore <file>` | Restore app backup | `adb restore backup.ab` | Validation, progress indication |
| `aim app permissions <package>` | List app permissions | `adb shell dumpsys package com.example \| grep permission` | Formatted, granted/denied status |
| `aim app stop <package>` | Force stop app | `adb shell am force-stop com.example` | Confirmation, process verification |
| `aim app start <package>` | Start app | `adb shell monkey -p com.example 1` | Launches main activity properly |
| `aim app uninstall <package>` | Uninstall with options | `adb uninstall com.example` | --keep-data flag, batch support |
| **device** | **Device Control** | | |
| `aim device info` | Device information | `adb shell getprop` + multiple commands | All device info in one place |
| `aim device battery` | Battery status | `adb shell dumpsys battery` | Parsed output, charging info |
| `aim device storage` | Storage information | `adb shell df -h` | Internal/external separated |
| `aim device reboot [mode]` | Smart reboot | `adb reboot [bootloader\|recovery]` | Confirmation, mode selection |
| `aim device unlock [pin]` | Unlock device | `adb shell input keyevent 82 && ...` | Works with PIN/pattern |
| `aim device airplane [on\|off]` | Toggle airplane mode | `adb shell settings put global airplane_mode_on 1 && ...` | Single command |
| `aim device screenshot` | Take screenshot | `adb exec-out screencap -p > screen.png` | Already implemented |
| **net** | **Network Operations** | | |
| `aim net connect <ip>` | Connect via WiFi | `adb tcpip 5555 && adb connect <ip>:5555` | Handles port automatically |
| `aim net disconnect` | Disconnect WiFi | `adb disconnect` | Shows what was disconnected |
| `aim net forward <local> <remote>` | Port forwarding | `adb forward tcp:8080 tcp:8080` | Better syntax, validation |
| `aim net reverse <remote> <local>` | Reverse forwarding | `adb reverse tcp:8080 tcp:8080` | Better syntax |
| `aim net list` | List port forwards | `adb forward --list` | Shows both forward and reverse |
| `aim net wifi` | Show WiFi info | Complex shell commands | SSID, IP, password if possible |
| **ui** | **UI Automation** | | |
| `aim ui tap <x> <y>` | Tap screen | `adb shell input tap 500 1000` | Save/load positions |
| `aim ui swipe <direction>` | Swipe gesture | `adb shell input swipe x1 y1 x2 y2` | Named directions (up/down/left/right) |
| `aim ui text <string>` | Type text | `adb shell input text "hello"` | Handles special chars, escaping |
| `aim ui key <keycode>` | Send keycode | `adb shell input keyevent KEYCODE_HOME` | Named keys, combos |
| `aim ui screenshot --area` | Screenshot region | Not possible with adb | Select area interactively |
| **system** | **System Information** | | |
| `aim system top` | Process list | `adb shell top -n 1` | Interactive, sortable |
| `aim system ps <filter>` | Process search | `adb shell ps \| grep <filter>` | Better formatting |
| `aim system kill <process>` | Kill process | `adb shell kill <pid>` | By name or PID |
| `aim system date [time]` | Get/set date | `adb shell date` | Timezone aware |
| `aim system settings <get\|set>` | System settings | `adb shell settings get/put global <key> <value>` | Tab completion |
| **activity** | **Activity Management** | | |
| `aim activity current` | Current activity | `adb shell dumpsys activity \| grep mFocusedActivity` | Clean output |
| `aim activity stack` | Activity stack | `adb shell dumpsys activity activities` | Parsed, hierarchical |
| `aim activity start <intent>` | Start activity | `adb shell am start ...` | Intent builder |
| `aim activity broadcast <intent>` | Send broadcast | `adb shell am broadcast ...` | Common broadcasts preset |
| **package** | **Package Management** | | |
| `aim package install <apk>` | Install APK | `adb install app.apk` | Already exists as `aim install` |
| `aim package list [filter]` | List packages | `adb shell pm list packages` | Better than `aim app list` |
| **debug** | **Debug Tools** | | |
| `aim debug layout` | Show layout bounds | `adb shell setprop debug.layout true && ...` | Toggle easily |
| `aim debug overdraw` | Show overdraw | `adb shell setprop debug.hwui.overdraw show && ...` | Visual debugging |
| `aim debug gpu` | GPU profiling | `adb shell setprop debug.hwui.profile true && ...` | Performance debugging |
| `aim debug touches` | Show touches | `adb shell settings put system show_touches 1` | Toggle on/off |

## Improvements to Existing Subcommands

| Current | Proposed Improvement | Current behavior | Why it's better |
|---------|---------------------|------------------|-----------------|
| `aim ls` | `aim ls --watch` | Static list | Auto-refresh when devices change |
| `aim ls` | `aim ls --wait` | Fails if no device | Waits for device to appear |
| `aim ls` | `aim ls --json` | Table output | Machine readable |
| `aim screenshot` | `aim screenshot --area x,y,w,h` | Full screen only | Crop without external tools |
| `aim screenshot` | `aim screenshot --window <app>` | Full screen only | Capture specific app |
| `aim screenshot` | `aim screenshot --delay 5` | Immediate | Delayed capture |
| `aim screenrecord` | `aim screenrecord --gif` | MP4 only | Direct GIF output |
| `aim screenrecord` | `aim screenrecord --resolution 720p` | Default resolution | Common resolutions |
| `aim pull` | `aim pull --newer` | Pulls all files | Only newer than local |
| `aim pull` | `aim pull --pattern "*.jpg"` | All files | Glob patterns |
| `aim push` | `aim push --preserve` | Loses timestamps | Maintains times/permissions |
| `aim push` | `aim push --verify` | No verification | MD5 check after transfer |
| `aim shell` | `aim shell --root` | Regular shell | Auto root escalation |
| `aim shell` | `aim shell --script file.sh` | Interactive only | Run script files |
| `aim logcat` | `aim logcat --since 5m` | All logs | Time-based filtering |
| `aim logcat` | `aim logcat --package <app>` | All apps | Filter by package |
| `aim logcat` | `aim logcat --level ERROR` | All levels | Cleaner than `-p` |
| `aim getprop` | `aim getprop --export` | Text output | Shell variable format |
| `aim getprop` | `aim getprop --diff <device>` | Single device | Compare devices |
| `aim server` | `aim server --auto-restart` | Manual restart | Auto-restart on failure |

## Usage Examples

### Category-based Commands

```bash
# App management
aim app list                          # List all apps
aim app list --user                   # List user apps only  
aim app info com.spotify              # Show app details
aim app clear com.spotify             # Clear app data
aim app pull com.spotify              # Extract APK
aim app permissions com.spotify       # Show permissions

# Device management  
aim device info                       # Complete device info
aim device battery                    # Battery status
aim device reboot recovery            # Reboot to recovery
aim device unlock 1234                # Unlock with PIN

# Network operations
aim net connect 192.168.1.100         # Connect over WiFi
aim net forward 8080:8080             # Port forwarding
aim net list                          # List all forwards

# UI automation
aim ui tap 500 1000                   # Tap at coordinates
aim ui swipe up                       # Swipe up
aim ui text "Hello World"             # Type text
aim ui key home                       # Press home key

# System information
aim system top                        # Interactive process viewer
aim system settings get secure android_id  # Get setting
aim system kill com.example           # Kill by package name

# Activity management
aim activity current                  # Show current activity
aim activity start com.example/.MainActivity  # Start activity
```

### Improved Existing Commands

```bash
# Watch for devices
aim ls --watch

# Wait for device
aim ls --wait

# Screenshot with delay
aim screenshot --delay 5

# Record as GIF
aim screenrecord --gif --time 5

# Pull only newer files
aim pull --newer /sdcard/DCIM/ ./

# Logcat with time and package filter
aim logcat --since 10m --package com.example --level ERROR
```

## Implementation Priority

High priority (most common use cases):

1. `aim app` subcommands - App management is very common
2. `aim net connect` - WiFi debugging is essential  
3. `aim ui` basic commands - Automation basics
4. `aim logcat` improvements - Time filtering is critical
5. `aim device info/battery` - Quick device status

Medium priority:

1. `aim activity` - Developer focused
2. `aim system` - Power user features
3. Screenshot/screenrecord improvements
4. File transfer improvements

Low priority:

1. `aim debug` - Specialized debugging
2. Advanced UI automation features
3. GIF recording
