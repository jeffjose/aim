# AIM Codebase Audit

**Date**: 2025-12-11
**Auditor**: Claude
**Codebase Stats**: 8,860 lines of Rust across 56 files

## Executive Summary

**aim** is a Rust CLI tool providing an improved interface for Android Debug Bridge (ADB). The project has undergone partial refactoring from a legacy architecture to a modular design, but the migration is incomplete, leaving significant technical debt.

**Overall Assessment: MODERATE TECHNICAL DEBT** - Good foundations but needs cleanup.

---

## 1. Project Structure

```
src/
├── adb/              # NEW: Modular ADB implementation (6 files) - MOSTLY UNUSED
│   ├── connection.rs # Connection management (placeholder)
│   ├── file_transfer.rs
│   ├── protocol.rs   # Wire protocol
│   ├── server.rs     # Server lifecycle
│   └── shell.rs      # Command execution
├── commands/         # NEW: Command implementations (14 files)
│   ├── app/          # App management subcommands (6 files)
│   │   ├── backup.rs, clear.rs, list.rs, pull.rs, start.rs, stop.rs
│   ├── adb.rs, config.rs, copy.rs, dmesg.rs, getprop.rs
│   ├── ls.rs, perfetto.rs, rename.rs, run.rs, runner.rs
│   ├── screenrecord.rs, screenshot.rs, server.rs
│   └── mod.rs        # SubCommand trait
├── core/             # NEW: Core types (3 files)
│   ├── types.rs      # Device, DeviceId, OutputFormat
│   └── context.rs    # CommandContext
├── device/           # Device management (3 files)
│   ├── device_info.rs
│   └── manager.rs    # PLACEHOLDER IMPLEMENTATION
├── library/          # LEGACY: Old ADB implementation (4 files)
│   ├── adb.rs        # 1,159 lines - MONOLITHIC, STILL IN USE
│   ├── hash.rs, protocol.rs
│   └── mod.rs
├── output/           # Output formatting (4 files)
├── progress/         # Progress reporting (1 file)
├── testing/          # Test infrastructure (3 files)
├── cli.rs            # CLI structure with clap
├── config.rs         # Configuration loading
├── error.rs          # Error types with thiserror
├── types.rs          # LEGACY: Old type definitions - DUPLICATE
├── lib.rs, main.rs, utils.rs
```

---

## 2. Architectural Issues

### 2.1 Dual Architecture (Critical)

The codebase maintains **two parallel implementations**:

| Aspect | Legacy (`src/library/`) | New (`src/adb/`) |
|--------|------------------------|------------------|
| Status | **ACTIVE** - All commands use this | **DORMANT** - Mostly `#[allow(dead_code)]` |
| Size | 1,159 lines (adb.rs) | Split across 6 files |
| Quality | Monolithic, mixed concerns | Well-structured but incomplete |

**Problem**: Commands were migrated to new `SubCommand` trait but still call legacy `library::adb` functions. The new `adb/` modules are scaffolded but unused.

### 2.2 Duplicate Type Systems

| File | Purpose | Status |
|------|---------|--------|
| `src/types.rs` | Legacy types (`DeviceDetails`) | Still referenced |
| `src/core/types.rs` | New types (`Device`, `DeviceId`) | Partially adopted |

**Problem**: Two type systems coexist, causing confusion about which to use.

### 2.3 Device Management Fragmentation

Device handling is scattered across:
- `src/device/device_info.rs` - Device detection functions
- `src/device/manager.rs` - Placeholder `DeviceManager` (calls legacy functions)
- `src/library/adb.rs` - Actual device operations
- `src/main.rs` - Special routing for app commands (lines 69-111)

---

## 3. Technical Debt Inventory

### 3.1 Dead Code (89 occurrences)

Files with `#[allow(dead_code)]` attributes:

| File | Count | Notes |
|------|-------|-------|
| `src/adb/protocol.rs` | 17 | Entire module unused |
| `src/adb/file_transfer.rs` | 12 | Entire module unused |
| `src/core/types.rs` | 8 | Some types unused |
| `src/library/adb.rs` | 7 | Legacy code |
| `src/progress/mod.rs` | 6 | Progress variants unused |
| `src/commands/mod.rs` | 6 | Helper functions unused |
| `src/adb/connection.rs` | 6 | Entire module unused |
| Other files | 27 | Various |

### 3.2 Incomplete Features (TODOs)

```rust
// src/commands/run.rs:44
// TODO: Implement device filtering

// src/commands/app/backup.rs:45
// TODO: Implement app backup functionality

// src/commands/app/pull.rs:143
// TODO: Hook up progress reporter
```

### 3.3 Large Monolithic File

`src/library/adb.rs` at **1,159 lines** mixes:
- Connection management
- Protocol handling
- File transfer
- Shell execution
- Server management
- Device listing

---

## 4. Testing Status

### Current Test Coverage: ~15-20%

| Test File | Purpose | Coverage |
|-----------|---------|----------|
| `config_test.rs` | Config loading | Partial |
| `device_info_test.rs` | Device parsing | Partial |
| `hash_test.rs` | File hashing | Good |
| `protocol_test.rs` | ADB protocol | Good |
| `ls_test.rs` | ls command | Example only |

### Missing Tests

- 11 of 12 commands have no tests
- New `adb/` modules have no tests
- Output formatters untested
- Progress reporters untested
- Integration tests absent

---

## 5. Dependencies Analysis

### Production Dependencies (40+)

**Well-chosen core dependencies**:
- `clap` 4.5.40 - CLI parsing
- `tokio` 1.46.1 - Async runtime
- `serde` 1.0.219 - Serialization
- `thiserror` 2.0 - Error handling
- `indicatif` 0.18.0 - Progress bars

**Concern**:
- `petname` 3.0.0-alpha.2 - **Pre-release dependency**

### Unused Dependencies (Potential)

Need verification:
- `futures` - May be unused with tokio
- `lazy_static` - Could use `std::sync::OnceLock`
- `byteorder` - bytes crate may suffice

---

## 6. Code Quality Patterns

### Good Patterns

1. **Command Pattern**: `SubCommand` trait provides consistent interface
2. **Error Handling**: `thiserror`-based errors with automatic conversions
3. **Type Safety**: `DeviceId` newtype pattern, enum states
4. **Async Design**: Proper tokio usage
5. **Output Formatting**: Unified `OutputFormatter` with traits

### Bad Patterns

1. **Placeholder Implementations**: `DeviceManager` just wraps legacy code
2. **Mixed Async/Sync**: Some functions could be fully async
3. **Magic Numbers**: Hardcoded buffer sizes, timeouts
4. **Inconsistent Device Selection**: App commands routed differently

---

## 7. Documentation Inventory

### Root-level Markdown Files

| File | Lines | Purpose | Action |
|------|-------|---------|--------|
| `README.md` | 153 | User documentation | Keep, update |
| `CLAUDE.md` | ~200 | Development guide | Move to docs/ |
| `MIGRATION_STATUS.md` | 78 | Migration tracking | Outdated - update or archive |
| `MIGRATION_PLAN.md` | 73 | Migration roadmap | Outdated - archive |
| `AUDIT.md` | 139 | Command testing checklist | Outdated - archive |
| `IDEAS.md` | 205 | Feature backlog | Review and prioritize |
| `plans/refactor-PLAN.md` | 740 | Detailed refactor plan | Outdated - archive |

### Data Files

| File | Purpose | Action |
|------|---------|--------|
| `data/lst2.md` | ADB `stat` response examples | Move to tests or archive |
| `data/sta2.md` | More stat examples | Move to tests or archive |

**Recommendation**: Consolidate all development docs into `docs/` directory.

---

## 8. Command Status

All 12 non-app commands migrated to `SubCommand` trait:

| Command | Status | Uses Legacy ADB | Has Tests |
|---------|--------|-----------------|-----------|
| `ls` | Working | Yes | Yes (example) |
| `run` | Working | Yes | No |
| `copy` | Working | Yes | No |
| `rename` | Working | Yes | No |
| `server` | Working | Yes | No |
| `adb` | Working | Yes | No |
| `config` | Working | N/A | No |
| `dmesg` | Working | Yes | No |
| `perfetto` | Working | Yes | No |
| `screenrecord` | Working | Yes | No |
| `getprop` | Working | Yes | No |
| `screenshot` | Working | Yes | No |

### App Subcommands

| Command | Status | Notes |
|---------|--------|-------|
| `app list` | Working | |
| `app clear` | Working | |
| `app pull` | Working | Missing progress hookup |
| `app backup` | **NOT IMPLEMENTED** | TODO in code |
| `app start` | Working | |
| `app stop` | Working | |

---

## 9. Configuration

### Config Location
- **Expected**: `~/.config/aim/config.toml` (per XDG spec)
- **Code**: Uses `~/.aimconfig` in some places
- **Issue**: Inconsistency between documentation and implementation

### Config Format (TOML)
```toml
[devices]
work = { id = "...", name = "..." }

[screenshot]
output = "~/Pictures/Screenshots"
```

---

## 10. Key Files to Address

### Priority 1: Remove or Integrate

| File | Size | Issue | Action |
|------|------|-------|--------|
| `src/library/adb.rs` | 1,159 lines | Monolithic legacy code | Split and migrate |
| `src/types.rs` | 122 lines | Duplicate of core/types.rs | Remove |
| `src/device/manager.rs` | 69 lines | Placeholder | Implement properly |

### Priority 2: Clean Up

| File | Issue | Action |
|------|-------|--------|
| `src/adb/*.rs` | 89 dead_code markers | Either use or remove |
| `src/main.rs` | Special app command routing | Unify with CommandRunner |

### Priority 3: Documentation

| File | Issue | Action |
|------|-------|--------|
| `MIGRATION_*.md` | Outdated | Archive to docs/archive/ |
| `AUDIT.md` (root) | Outdated | Archive, replaced by this |
| `plans/` | Contains old plan | Archive |

---

## 11. Build Status

```bash
cargo build     # Success
cargo clippy    # Passes (with allow attributes)
cargo test      # 5 test files pass
```

No compilation errors. Dead code warnings suppressed with `#[allow(dead_code)]`.

---

## 12. Summary of Issues

### Critical
1. **Dual architecture** - New modules unused, legacy monolith active
2. **Duplicate type systems** - Confusing which to use

### High
3. **89 dead_code markers** - Indicates incomplete migration
4. **~85% commands untested** - Risk of regressions
5. **Placeholder DeviceManager** - Not properly implemented
6. **3 TODOs** - Incomplete features

### Medium
7. **Scattered documentation** - Multiple outdated .md files
8. **Alpha dependency** - petname is pre-release
9. **Config path inconsistency** - XDG vs legacy location
10. **Large file** - library/adb.rs at 1,159 lines

### Low
11. **Missing rustdoc comments** - Public API undocumented
12. **Magic numbers** - Hardcoded values throughout

---

## 13. Recommendations

### Immediate (Before Any New Features)

1. **Decide architecture direction**: Either complete new `adb/` migration or remove it
2. **Remove duplicate types**: Keep only `core/types.rs`
3. **Consolidate documentation**: Move all .md to `docs/`, archive outdated

### Short-term (Next Iteration)

4. **Add tests for all commands** - Prevent regressions
5. **Implement DeviceManager properly** - Remove placeholder
6. **Resolve TODOs** - Finish backup, device filtering, progress hookup

### Medium-term

7. **Split library/adb.rs** - If keeping legacy approach, modularize it
8. **Audit dependencies** - Remove unused, update pre-release
9. **Standardize config path** - Use XDG consistently

---

## Appendix A: File Sizes

```
1159 src/library/adb.rs          <- MONOLITHIC, needs splitting
 271 src/core/types.rs
 235 src/progress/mod.rs
 215 src/adb/protocol.rs
 196 src/cli.rs
 175 src/config.rs
 166 src/commands/runner.rs
 158 src/output/mod.rs
 126 src/main.rs
 122 src/types.rs                 <- DUPLICATE
 115 src/error.rs
 ...
```

## Appendix B: Module Dependencies

```
main.rs
  ├── cli.rs (clap definitions)
  ├── commands/runner.rs
  │     └── commands/*.rs (SubCommand implementations)
  │           └── library/adb.rs (LEGACY - actual ADB operations)
  └── device/device_info.rs
        └── library/adb.rs

adb/*.rs         <- NEW, UNUSED
core/types.rs    <- NEW, partially used
output/*.rs      <- NEW, used
progress/*.rs    <- NEW, used
testing/*.rs     <- NEW, basic use
```
