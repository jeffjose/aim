# CLAUDE.md

Project-specific instructions for Claude Code.

## Project

**aim** - A better CLI interface for Android Debug Bridge (ADB). Written in Rust.

## Build Commands

```bash
cargo build              # Build
cargo test               # Run tests
cargo clippy             # Lint
cargo run -- ls          # Run during development
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full details.

Key directories:
- `src/commands/` - Command implementations (SubCommand trait)
- `src/library/adb.rs` - ADB operations (legacy, still active)
- `src/core/types.rs` - Core types (Device, DeviceId)

## Adding Commands

1. Create `src/commands/[name].rs`
2. Implement `SubCommand` trait
3. Add to `Commands` enum in `src/cli.rs`
4. Add routing in `src/commands/runner.rs`

## Git Workflow

Commit after each significant change with cohesive, logical chunks.

## Current State

See [docs/AUDIT.md](docs/AUDIT.md) for codebase audit.
See [docs/PLAN.md](docs/PLAN.md) for modernization plan.
