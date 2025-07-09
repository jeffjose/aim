# aim - Feature Ideas

Ideas for new subcommands and improvements based on common adb pain points.

## New Subcommands

| Proposed Command | Description | Equivalent adb | Why it's better |
|-----------------|-------------|----------------|-----------------|
| `aim wifi <IP>` | Connect to device over WiFi | `adb tcpip 5555 && adb connect <IP>:5555` | Single command, handles port automatically |
| `aim backup <app>` | Backup app data | `adb backup -f backup.ab -apk -obb com.example` | Simpler syntax, automatic file naming |
| `aim restore backup.ab` | Restore app data | `adb restore backup.ab` | Progress indication, validation |
| `aim apk <package>` | Extract APK from device | `adb shell pm path com.example && adb pull <path>` | Finds and pulls APK in one step |
| `aim clear <app>` | Clear app data | `adb shell pm clear com.example` | Shorter, supports partial package names |
| `aim permissions <app>` | List app permissions | `adb shell dumpsys package com.example | grep permission` | Formatted output, granted/denied status |
| `aim battery` | Show battery status | `adb shell dumpsys battery` | Parsed output, percentage prominently shown |
| `aim storage` | Show storage info | `adb shell df -h` | Better formatting, shows internal/SD separately |
| `aim top` | Show running processes | `adb shell top -n 1` | Interactive mode, sortable columns |
| `aim activity` | Show current activity | `adb shell dumpsys activity | grep mFocusedActivity` | Clean output, just the activity name |
| `aim monkey <package>` | Run monkey test | `adb shell monkey -p com.example -v 500` | Progress bar, crash detection |
| `aim sideload <file>` | Sideload OTA/zip | `adb sideload update.zip` | Resume support, verification |
| `aim reboot [mode]` | Smart reboot | `adb reboot [bootloader|recovery|download]` | Confirms device, shows mode options |
| `aim forward <local> <remote>` | Port forwarding | `adb forward tcp:8080 tcp:8080` | Lists active forwards, easier syntax |
| `aim reverse <remote> <local>` | Reverse port forward | `adb reverse tcp:8080 tcp:8080` | Lists active reverses |
| `aim dumpsys <service>` | Pretty dumpsys | `adb shell dumpsys <service>` | Formatted output, common services list |
| `aim settings get/set` | Manage settings | `adb shell settings get/put global <key> <value>` | Tab completion, validation |
| `aim input <text>` | Send text input | `adb shell input text "Hello World"` | Handles special chars, clipboard support |
| `aim tap <x> <y>` | Tap coordinates | `adb shell input tap 500 1000` | Visual guide mode, save locations |
| `aim swipe` | Swipe gestures | `adb shell input swipe x1 y1 x2 y2 duration` | Preset gestures (up/down/left/right) |
| `aim unlock` | Unlock device | `adb shell input keyevent 82 && adb shell input swipe ...` | Works with PIN/pattern/swipe |
| `aim airplane [on|off]` | Toggle airplane mode | `adb shell settings put global airplane_mode_on 1 && adb shell am broadcast ...` | Single command |
| `aim wifi-password` | Show WiFi password | `adb shell su -c 'cat /data/misc/wifi/wpa_supplicant.conf'` | Works on non-root via backup |
| `aim deeplink <url>` | Open deeplink | `adb shell am start -a android.intent.action.VIEW -d "url"` | URL validation, encoding |
| `aim stress-test` | UI stress test | Multiple adb commands | Combines monkey + screenshots + logs |

## Improvements to Existing Subcommands

| Current | Proposed Improvement | Current adb | Why it's better |
|---------|---------------------|-------------|-----------------|
| `aim ls` | `aim ls --watch` | `watch adb devices` | Built-in watch mode |
| `aim ls` | `aim ls --wait` | `adb wait-for-device` | Waits and shows when device appears |
| `aim screenshot` | `aim screenshot --area x,y,w,h` | `adb shell screencap -p | convert ...` | Crop without external tools |
| `aim screenshot` | `aim screenshot --window <app>` | Complex shell commands | Captures specific app window |
| `aim screenrecord` | `aim screenrecord --gif` | Record + ffmpeg convert | Direct GIF output |
| `aim screenrecord` | `aim screenrecord --audio` | Not possible with adb | Records with internal audio |
| `aim pull` | `aim pull --newer` | Manual timestamp checking | Only pulls files newer than local |
| `aim push` | `aim push --preserve-time` | `adb push` + `touch` commands | Maintains modification times |
| `aim shell` | `aim shell --root` | `adb root && adb shell` | Automatic root escalation |
| `aim shell` | `aim shell --user <id>` | `adb shell run-as com.example` | Run as specific user/app |
| `aim logcat` | `aim logcat --grep <pattern>` | `adb logcat | grep pattern` | Built-in grep with colors |
| `aim logcat` | `aim logcat --since 5m` | Complex filtering | Time-based filtering |
| `aim logcat` | `aim logcat --save <file>` | `adb logcat > file` | Rotation, compression |
| `aim install` | `aim install --replace` | `adb install -r app.apk` | Better error messages |
| `aim install` | `aim install *.apk` | Loop with adb install | Batch install with progress |
| `aim uninstall` | `aim uninstall --keep-data` | `adb shell pm uninstall -k` | Clearer flag name |
| `aim getprop` | `aim getprop --export` | Manual formatting | Export as shell variables |
| `aim getprop` | `aim getprop --diff <device>` | Manual comparison | Compare properties between devices |
| `aim dmesg` | `aim dmesg --errors` | `adb shell dmesg | grep -E "error|fail"` | Pre-filtered error view |
| `aim server` | `aim server --port 5038` | `export ADB_SERVER_PORT=5038` | Direct port specification |

## Usage Examples

### New Commands

```bash
# Connect over WiFi in one command
aim wifi 192.168.1.100

# Extract APK from device
aim apk com.spotify.music
# Output: Pulled com.spotify.music.apk (45.2 MB)

# Show current activity
aim activity
# Output: com.example.app/.MainActivity

# Smart reboot with confirmation
aim reboot recovery
# Output: Reboot Pixel 6 (abc123) to recovery mode? [y/N]

# Port forwarding with list
aim forward 8080:8080
aim forward --list
# Output: 
# Local    Remote   Device
# tcp:8080 tcp:8080 abc123

# Send text with special characters
aim input "Hello & goodbye!"

# Unlock device
aim unlock --pin 1234
```

### Improved Commands

```bash
# Watch for devices
aim ls --watch

# Screenshot with area
aim screenshot --area 100,200,300,400

# Record as GIF
aim screenrecord --gif --time 5

# Pull only newer files
aim pull --newer /sdcard/DCIM/ ./photos/

# Logcat with time filter
aim logcat --since 10m --grep "Error"

# Batch install
aim install *.apk
# Output: 
# Installing app1.apk... ✓
# Installing app2.apk... ✓
# Completed: 2/2

# Compare properties between devices  
aim getprop --diff pixel6,galaxy
```

## Implementation Priority

High priority (most common pain points):
1. `aim wifi` - WiFi connection is very common
2. `aim apk` - Extracting APKs is frequently needed
3. `aim activity` - Debugging current screen
4. `aim clear` - Clearing app data
5. Improvements to `aim screenshot` (area, window)
6. `aim logcat --since` - Time-based filtering

Medium priority:
1. `aim battery` / `aim storage` - System info
2. `aim forward` / `aim reverse` - Better port forwarding
3. `aim input` / `aim tap` / `aim swipe` - UI automation
4. Batch operations for install/uninstall

Low priority:
1. `aim stress-test` - Advanced testing
2. `aim deeplink` - Developer focused
3. GIF recording - Nice to have