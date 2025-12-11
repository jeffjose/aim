# AIM Modernization Plan

**Based on**: [docs/AUDIT.md](./AUDIT.md)
**Date**: 2025-12-11

## Goals

1. **Eliminate technical debt** without losing functionality
2. **Choose one architecture** and fully commit to it
3. **Improve test coverage** to enable safe refactoring
4. **Consolidate documentation** into organized structure
5. **Maintain working CLI** throughout the process

---

## Phase 0: Documentation Cleanup

**Scope**: Organize existing documentation before code changes.

### Tasks

- [ ] Move all root .md files (except README.md) to appropriate locations
- [ ] Archive outdated migration docs
- [ ] Create `docs/` structure:
  ```
  docs/
  ├── AUDIT.md           # This audit
  ├── PLAN.md            # This plan
  ├── ARCHITECTURE.md    # Current architecture (from CLAUDE.md)
  ├── COMMANDS.md        # Command reference (from IDEAS.md)
  └── archive/           # Old docs for reference
      ├── MIGRATION_STATUS.md
      ├── MIGRATION_PLAN.md
      └── refactor-PLAN.md
  ```
- [ ] Update CLAUDE.md to be concise (project-specific instructions only)
- [ ] Move test data files from `data/` to `tests/fixtures/`

### Verification
- All useful content preserved
- Root directory clean (only README.md, CLAUDE.md, Cargo.toml, etc.)

---

## Phase 1: Architecture Decision

**Scope**: Decide direction and remove dead code.

### Decision Point

Two options:

**Option A: Complete New Architecture**
- Migrate all operations from `library/adb.rs` to `adb/*.rs`
- Implement `DeviceManager` properly
- Delete `library/` once migration complete
- **Effort**: High (rewrite ADB layer)
- **Benefit**: Clean, modular codebase

**Option B: Keep Legacy, Remove Dead New Code**
- Delete unused `adb/*.rs` modules
- Keep `library/adb.rs` as the ADB implementation
- Refactor `library/adb.rs` into smaller files within `library/`
- **Effort**: Low (cleanup only)
- **Benefit**: Quick win, functional codebase

**Recommendation**: **Option B** for now. The legacy code works. Clean it up rather than rewrite.

### Tasks (Option B)

- [ ] Delete `src/adb/` directory (unused modules)
- [ ] Delete `src/types.rs` (duplicate of core/types.rs)
- [ ] Update imports to use only `core/types.rs`
- [ ] Remove all `#[allow(dead_code)]` markers by either:
  - Implementing the code
  - Deleting the code
- [ ] Run `cargo clippy` and fix all warnings

### Verification
- `cargo build` succeeds
- `cargo clippy` passes without allow markers
- All commands still work

---

## Phase 2: Legacy Code Organization

**Scope**: Split `library/adb.rs` (1,159 lines) into focused modules.

### Target Structure

```
src/library/
├── mod.rs           # Re-exports
├── connection.rs    # TCP connection to ADB server
├── protocol.rs      # Wire protocol (keep existing)
├── device.rs        # Device listing and selection
├── shell.rs         # Shell command execution
├── file_ops.rs      # Push/pull operations
├── server.rs        # Server start/stop/restart
└── hash.rs          # Keep existing
```

### Tasks

- [ ] Extract connection logic into `connection.rs`
- [ ] Extract device operations into `device.rs`
- [ ] Extract shell execution into `shell.rs`
- [ ] Extract file operations into `file_ops.rs`
- [ ] Extract server management into `server.rs`
- [ ] Update `mod.rs` to re-export everything
- [ ] Update all command imports

### Verification
- `cargo test` passes
- All commands work identically
- No file over 300 lines (soft target)

---

## Phase 3: Device Management Unification

**Scope**: Fix fragmented device handling.

### Current Problems
1. `device/device_info.rs` has device functions
2. `device/manager.rs` is a placeholder
3. `library/adb.rs` has actual implementations
4. `main.rs` has special routing for app commands

### Tasks

- [ ] Implement `DeviceManager` properly (not placeholder)
- [ ] Move device selection logic from `main.rs` into `DeviceManager`
- [ ] Unify app command routing with other commands
- [ ] Single code path for device selection across all commands

### Verification
- All commands use `DeviceManager`
- `main.rs` has no device-specific logic
- Device selection works the same for all commands

---

## Phase 4: Test Coverage

**Scope**: Add tests to enable safe future refactoring.

### Target: 70% coverage for command logic

### Tasks

- [ ] Add tests for `run` command
- [ ] Add tests for `copy` command
- [ ] Add tests for `rename` command
- [ ] Add tests for `server` command
- [ ] Add tests for `adb` command
- [ ] Add tests for `dmesg` command
- [ ] Add tests for `perfetto` command
- [ ] Add tests for `screenrecord` command
- [ ] Add tests for `getprop` command
- [ ] Add tests for `screenshot` command
- [ ] Add tests for app commands (list, clear, start, stop, pull)
- [ ] Add integration test framework (mock ADB responses)

### Test Strategy
- Unit tests for parsing and formatting logic
- Mock ADB server responses for command tests
- Use existing `testing/` infrastructure

### Verification
- `cargo test` runs all new tests
- Each command has at least basic coverage

---

## Phase 5: Feature Completion

**Scope**: Resolve TODOs and incomplete features.

### Tasks

- [ ] Implement device filtering in `run` command
- [ ] Implement `app backup` functionality
- [ ] Hook up progress reporter in `app pull`
- [ ] Verify config path consistency (use XDG standard)
- [ ] Add config migration from old `~/.aimconfig` location

### Verification
- No TODO comments in codebase
- All advertised features work
- Config works from standard location

---

## Phase 6: Dependency Audit

**Scope**: Clean up dependencies.

### Tasks

- [ ] Replace `petname` alpha with stable version or alternative
- [ ] Audit for unused dependencies:
  - `futures` (tokio may be sufficient)
  - `lazy_static` (use `std::sync::OnceLock`)
  - `byteorder` (bytes crate may suffice)
- [ ] Update all dependencies to latest stable
- [ ] Run `cargo audit` for security issues

### Verification
- No pre-release dependencies
- `cargo build` still works
- No security advisories

---

## Phase 7: Documentation & Polish

**Scope**: Final cleanup.

### Tasks

- [ ] Add rustdoc comments to all public APIs
- [ ] Update README.md with any command changes
- [ ] Generate and review API documentation
- [ ] Create CONTRIBUTING.md with development setup
- [ ] Consider v1.0 release criteria

### Verification
- `cargo doc` generates clean documentation
- README reflects actual capabilities
- New contributors can onboard easily

---

## Execution Order

```
Phase 0 (Documentation)     <- Do first, enables clean commits
    |
Phase 1 (Architecture)      <- Remove dead code, pick direction
    |
Phase 2 (Legacy Split)      <- Modularize without behavior change
    |
Phase 3 (Device Unify)      <- Single device handling path
    |
Phase 4 (Tests)             <- Enable safe future changes
    |
Phase 5 (Features)          <- Complete advertised functionality
    |
Phase 6 (Dependencies)      <- Clean up external deps
    |
Phase 7 (Polish)            <- Documentation and release prep
```

---

## Risk Mitigation

### Each Phase Should:
1. **Start with passing tests** (`cargo test`)
2. **End with passing tests** (`cargo test`)
3. **Not break any command** (manual verification)
4. **Result in a clean commit** (atomic changes)

### Rollback Plan
- Git tags before each phase: `pre-phase-N`
- If phase breaks something, `git reset --hard pre-phase-N`

---

## Success Criteria

After all phases:

- [ ] Zero `#[allow(dead_code)]` in codebase
- [ ] Zero TODO/FIXME comments
- [ ] No files over 400 lines
- [ ] 70%+ test coverage on commands
- [ ] All commands work as documented
- [ ] Clean `cargo clippy` output
- [ ] No pre-release dependencies
- [ ] Organized `docs/` directory
- [ ] Rustdoc on public APIs

---

## Notes

- **Don't optimize prematurely**: Get clean first, fast later
- **Keep commits small**: One logical change per commit
- **Test manually after each change**: `aim ls`, `aim getprop`, etc.
- **The goal is maintainability**: Not perfection, just "good enough to extend"
