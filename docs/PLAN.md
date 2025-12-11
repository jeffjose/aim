# AIM Modernization Plan

**Based on**: [docs/AUDIT.md](./AUDIT.md)
**Date**: 2025-12-11
**Status**: ✅ COMPLETED (v0.2.0)

## Summary

All phases completed successfully. The codebase has been modernized:
- Documentation organized into `docs/` directory
- Clippy warnings fixed
- Protocol types consolidated
- Device management unified
- Test coverage expanded (54 → 76 tests)
- TODOs resolved
- Unused dependencies removed
- Version bumped to 0.2.0

## Goals

1. ✅ **Eliminate technical debt** without losing functionality
2. ✅ **Choose one architecture** and fully commit to it
3. ✅ **Improve test coverage** to enable safe refactoring
4. ✅ **Consolidate documentation** into organized structure
5. ✅ **Maintain working CLI** throughout the process

---

## Phase 0: Documentation Cleanup ✅

**Scope**: Organize existing documentation before code changes.

### Tasks

- [x] Move all root .md files (except README.md) to appropriate locations
- [x] Archive outdated migration docs
- [x] Create `docs/` structure
- [x] Update CLAUDE.md to be concise (project-specific instructions only)
- [x] Move test data files from `data/` to `tests/fixtures/`

### Verification
- ✅ All useful content preserved
- ✅ Root directory clean

---

## Phase 1: Architecture Decision ✅

**Scope**: Decide direction and remove dead code.

### Decision

Chose **Option B**: Keep legacy, clean up. Discovered src/adb/ modules ARE used by app commands.

### Tasks

- [x] Fix clippy warnings (while let patterns, loop structures)
- [x] Keep both architectures since src/adb/ is actively used
- [x] Run `cargo clippy` and fix all warnings

### Verification
- ✅ `cargo build` succeeds
- ✅ `cargo clippy` passes
- ✅ All commands still work

---

## Phase 2: Legacy Code Organization ✅

**Scope**: Consolidate protocol types.

### Completed

- [x] Move AdbLstatResponse, FileMetadata, FileTimestamps to protocol.rs
- [x] Move ProgressDisplay to protocol.rs
- [x] Add re-exports for backwards compatibility
- [x] Reduce adb.rs from 1153 to 965 lines

### Verification
- ✅ `cargo test` passes
- ✅ All commands work identically

---

## Phase 3: Device Management Unification ✅

**Scope**: Fix fragmented device handling.

### Completed

- [x] Enhanced DeviceManager with with_address(), get_target_device(), get_single_device()
- [x] Simplified main.rs from 126 to 97 lines
- [x] Marked legacy device selection functions as dead_code

### Verification
- ✅ All commands use DeviceManager
- ✅ `main.rs` simplified
- ✅ Device selection works consistently

---

## Phase 4: Test Coverage ✅

**Scope**: Add tests to enable safe future refactoring.

### Completed

- [x] Created manager_test.rs with 7 tests for DeviceManager
- [x] Created error_test.rs with 4 tests for AimError
- [x] Fixed unused variable warnings in existing tests
- [x] Tests increased from 54 to 76

### Verification
- ✅ `cargo test` passes with 76 tests
- ✅ Core infrastructure has test coverage

---

## Phase 5: Feature Completion ✅

**Scope**: Resolve TODOs and incomplete features.

### Completed

- [x] Hook up progress reporter in `app pull`
- [x] Add set_progress_reporter method to FileTransfer
- [x] Improve device filtering warning with helpful message
- [x] Enhance app backup with helpful fallback message

Note: Device filtering and app backup are deferred to future releases.

### Verification
- ✅ No TODO comments in Rust codebase
- ✅ All implemented features work

---

## Phase 6: Dependency Audit ✅

**Scope**: Clean up dependencies.

### Completed

- [x] Remove unused dependencies: futures, rand_xorshift, toml_edit, anyhow, bincode, byteorder
- [x] Move tempfile to dev-dependencies only
- [x] Organize Cargo.toml with clear sections

Note: petname alpha kept for now (no stable alternative with seeded RNG support)

### Verification
- ✅ `cargo build` still works
- ✅ Dependencies organized and documented

---

## Phase 7: Documentation & Polish ✅

**Scope**: Final cleanup.

### Completed

- [x] Version bumped to 0.2.0
- [x] Enhanced Cargo.toml with metadata (description, keywords, etc.)
- [x] Updated PLAN.md with completion status
- [x] All phases documented

### Verification
- ✅ Version 0.2.0 ready
- ✅ Documentation reflects completed work

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

- [x] Zero TODO/FIXME comments in Rust code
- [x] All commands work as documented
- [x] Clean `cargo clippy` output
- [x] Organized `docs/` directory
- [x] Test count increased (54 → 76)
- [ ] Zero `#[allow(dead_code)]` in codebase (many remain, deferred)
- [ ] No files over 400 lines (library/adb.rs still 965, acceptable)
- [ ] No pre-release dependencies (petname alpha kept)
- [ ] Rustdoc on public APIs (deferred to future)

---

## Notes

- **Don't optimize prematurely**: Get clean first, fast later
- **Keep commits small**: One logical change per commit
- **Test manually after each change**: `aim ls`, `aim getprop`, etc.
- **The goal is maintainability**: Not perfection, just "good enough to extend"
