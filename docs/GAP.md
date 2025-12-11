# AIM Feature Gaps

**Date**: 2025-12-11
**Version**: 0.2.0

This document tracks the gaps between the README documentation and actual CLI functionality.

## README vs Reality

### Commands documented but not implemented as shown

| README Documents | Actual Status | Notes |
|------------------|---------------|-------|
| `aim push file.txt /sdcard/` | Missing | Use `aim copy file.txt device:/sdcard/` |
| `aim pull /sdcard/file.txt ./` | Missing | Use `aim copy device:/sdcard/file.txt ./` |
| `aim shell` | Missing | Use `aim run <command>` |
| `aim shell ls /sdcard` | Different | Use `aim run "ls /sdcard"` |
| `aim install app.apk` | Missing | Use `aim adb install app.apk` |
| `aim uninstall com.example` | Missing | Use `aim adb uninstall com.example` |
| `aim logcat` | Missing | Use `aim adb logcat` |
| `aim logcat -p ERROR` | Missing | Use `aim adb logcat *:E` |
| `aim logcat -c` | Missing | Use `aim adb logcat -c` |
| `aim -d pixel screenshot` | Different | Device ID is positional: `aim screenshot pixel` |

### Commands implemented but not documented in README

| Command | Description |
|---------|-------------|
| `aim app list` | List installed applications |
| `aim app start <package>` | Start an application |
| `aim app stop <package>` | Force stop an application |
| `aim app clear <package>` | Clear app data |
| `aim app pull <package>` | Pull APK from device |
| `aim app backup <package>` | Backup app data (not yet implemented) |
| `aim copy <src> <dst>` | Copy files to/from device |
| `aim run <command>` | Run shell command on device |
| `aim perfetto` | Run perfetto trace |
| `aim config` | Display configuration |
| `aim adb <args>` | Run arbitrary adb commands |

## Priority Fixes

### High Priority (UX consistency with README)

1. **Add `aim push` / `aim pull` aliases**
   - Users expect these from README
   - Should alias to `aim copy` functionality
   - `aim push src dst` → `aim copy src device:dst`
   - `aim pull device:src dst` → `aim copy device:src dst`

2. **Add `aim shell` command**
   - Interactive shell when no args
   - Run command when args provided
   - Should alias/wrap `aim run`

3. **Add `aim install` / `aim uninstall` commands**
   - Common operations deserve first-class support
   - Currently requires `aim adb install/uninstall`

4. **Add `aim logcat` command**
   - Very common operation
   - Should support `-p` for priority filtering
   - Should support `-c` for clearing

### Medium Priority

5. **Standardize device selection**
   - README shows `-d <device>` flag
   - Reality uses positional arguments
   - Pick one approach and update both code and docs

6. **Update README**
   - Document actual command structure
   - Add `aim app` subcommands
   - Add `aim copy` examples
   - Fix device selection examples

### Low Priority

7. **Add `aim push -r` recursive flag**
   - README mentions it but copy may handle this differently

## Implementation Notes

### Option A: Add Alias Commands
Add thin wrapper commands that call existing functionality:
- `push.rs` → wraps `copy` with device:dst format
- `pull.rs` → wraps `copy` with device:src format
- `shell.rs` → wraps `run` or provides interactive mode
- `install.rs` → wraps `adb install`
- `logcat.rs` → wraps `adb logcat` with nicer flags

### Option B: Update README
Rewrite README to match actual CLI:
- Change examples to use `aim copy`, `aim run`
- Document `aim app` subcommands
- Update device selection syntax

### Recommended: Hybrid Approach
1. Add high-priority aliases (push, pull, shell, install, logcat)
2. Update README for remaining gaps
3. Ensures backward compatibility with user expectations

## Tracking

- [ ] Add `aim push` alias
- [ ] Add `aim pull` alias
- [ ] Add `aim shell` command
- [ ] Add `aim install` command
- [ ] Add `aim uninstall` command
- [ ] Add `aim logcat` command
- [ ] Standardize device selection
- [ ] Update README to match reality
