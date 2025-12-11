# aim Command Audit

This document tracks the status of all `aim` commands, their testing status, and any issues or refactoring needs identified.

## Audit Status Legend

- ✅ **Tested & Working** - Command works as expected
- ⚠️  **Needs Attention** - Works but has issues or needs improvements
- ❌ **Broken** - Command is not working
- 🔄 **Not Tested** - Haven't tested yet
- 🚧 **In Progress** - Currently being refactored

## Global Options

- `-v` : WARN level logging (minimal output)
- `-vv` : INFO level logging (useful status messages)
- `-vvv` : DEBUG level logging (detailed trace information)

## Core Commands

| Command | Status | Notes | Refactoring Needed |
|---------|--------|-------|-------------------|
| `aim adb <command>` | 🔄 | | Pass-through to adb |
| `aim config` | 🔄 | | Check config file location |
| `aim copy <src> <dst>` | 🔄 | | Verify device:path parsing |
| `aim dmesg` | 🔄 | | Test with args |
| `aim getprop [props]` | 🔄 | | Test single/multiple/all props |
| `aim ls` | ✅ | Fixed unauthorized device handling, fixed hang issue | - |
| `aim perfetto` | 🔄 | | Complex command, needs thorough testing |
| `aim rename <device_id> <name>` | 🔄 | | Check config persistence |
| `aim run <command>` | 🔄 | | Check watch mode implementation |
| `aim screenrecord` | 🔄 | | Test output locations, formats |
| `aim screenshot` | 🔄 | | Test interactive mode |
| `aim server [operation]` | 🔄 | Now defaults to status | Verify all operations work |

## App Subcommands

| Command | Status | Notes | Refactoring Needed |
|---------|--------|-------|-------------------|
| `aim app list` | 🔄 | | Check filtering options |
| `aim app clear <package>` | 🔄 | | Verify package name matching |
| `aim app pull <package>` | 🔄 | | Test APK extraction |
| `aim app backup <package>` | 🔄 | | Test backup format |
| `aim app start <package>` | 🔄 | | Verify activity launch |
| `aim app stop <package>` | 🔄 | | Check force-stop behavior |

## Known Issues to Address

### 1. Device Selection

- [ ] Partial device ID matching consistency
- [ ] Better error messages when device not found
- [ ] Handle multiple matches gracefully

### 2. Error Handling

- [x] Unauthorized devices now handled gracefully
- [x] Fixed infinite loop when ADB returns empty response (hang with no parameters)
- [ ] Network errors (ADB server not running)
- [ ] Permission errors
- [ ] File not found errors

### 3. Output Formatting

- [ ] Consistent use of OutputFormat across all commands
- [ ] JSON output for all commands that display data
- [ ] Progress bars for long operations

### 4. Configuration

- [ ] Verify config file location (~/.config/aim/config.toml)
- [ ] Document all config options
- [ ] Migration from old .aimconfig if exists

### 5. Code Quality

- [ ] Remove dead code (e.g., ProgressDisplay::Hide warning)
- [ ] Consistent error types across modules
- [ ] Better separation between library and command code
- [ ] Complete migration from old architecture

## Testing Checklist

### Basic Functionality

- [ ] Single device scenarios
- [ ] Multiple device scenarios
- [ ] No device scenarios
- [ ] Unauthorized device handling
- [ ] Offline device handling

### Each Command Should Be Tested For

- [ ] Basic operation
- [ ] All command-line options
- [ ] Error cases
- [ ] Output formats (table/json/plain where applicable)
- [ ] Device selection (partial matching)
- [ ] Help text accuracy

## Refactoring Opportunities

### 1. Consistent Command Structure

- All commands should follow the SubCommand trait pattern
- Consistent Args struct naming and organization
- Proper use of CommandContext

### 2. Library Consolidation

- `src/library/adb.rs` has mixed concerns
- Consider splitting into focused modules
- Better async/await usage

### 3. Output System

- Centralize all output formatting
- Consistent progress reporting
- Better error display

### 4. Testing

- Add unit tests for each command
- Integration tests for device operations
- Mock ADB responses for testing

## Next Steps

1. Test each command systematically
2. Update this document with findings
3. Create issues for each problem found
4. Prioritize fixes based on user impact
5. Consider a v2.0 release after cleanup

---

*Last Updated: 2025-07-10*
*Tester: Claude*
