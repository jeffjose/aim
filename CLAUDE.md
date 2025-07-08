# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**aim** is a modern CLI tool written in Rust that serves as an improved interface for Android Debug Bridge (ADB). It enhances the ADB experience with better output formatting, intuitive commands, and improved usability.

## Key Commands

### Build and Development
```bash
cargo build              # Build the project
cargo build --release    # Build optimized release version
cargo run -- [args]      # Run aim with arguments during development
cargo install --path .   # Install aim locally
```

### Testing and Quality
```bash
cargo test              # Run all tests
cargo test [module]     # Run tests for specific module
cargo clippy            # Run Clippy linter for code quality
cargo fmt               # Format code using rustfmt
cargo check             # Quick compilation check without building
```

### Common Development Patterns
```bash
# Test a single subcommand implementation
cargo test subcommands::ls

# Run aim during development
cargo run -- ls
cargo run -- getprop ro.product.model
cargo run -- screenshot -o test.png

# Check for compilation errors quickly
cargo check
```

## High-Level Architecture

The codebase follows a clean modular architecture designed for extensibility:

### Core Structure
- **`src/main.rs`**: Entry point that handles command routing and global options
- **`src/cli.rs`**: CLI structure using clap, defines all subcommands and their arguments
- **`src/config.rs`**: Configuration management for user preferences and device aliases
- **`src/error.rs`**: Custom error types for comprehensive error handling
- **`src/types.rs`**: Common type definitions used across the codebase
- **`src/utils.rs`**: Utility functions for common operations

### Key Modules

1. **Device Management (`src/device/`)**
   - `device_info.rs`: Core device detection, enumeration, and information retrieval
   - Handles device selection logic including partial ID matching
   - Manages device state and properties

2. **ADB Protocol (`src/library/`)**
   - `adb.rs`: Low-level ADB communication implementation
   - `protocol.rs`: ADB wire protocol implementation
   - `hash.rs`: Hashing utilities for file transfers
   - Implements async communication with ADB server

3. **Subcommands (`src/subcommands/`)**
   - Each subcommand is implemented as a separate module
   - Common pattern: parse args → select device → execute command → format output
   - All subcommands support multiple output formats (table, json, plain)

### Architectural Patterns

1. **Async Architecture**: Built on Tokio for efficient async I/O operations
2. **Error Handling**: Uses anyhow for error propagation with custom error types
3. **Output Formatting**: Centralized formatting logic supporting multiple formats
4. **Device Selection**: Smart device selection with partial matching and aliases
5. **Configuration**: TOML-based config at `~/.config/aim/config.toml`

### Adding New Subcommands

To add a new subcommand:
1. Create a new file in `src/subcommands/`
2. Define the command structure in `src/cli.rs`
3. Implement the command logic following existing patterns
4. Add the command to the match statement in `src/main.rs`
5. Write tests in a corresponding `*_test.rs` file

### Testing Strategy

- Unit tests are placed in `*_test.rs` files alongside source files
- Use `#[test]` for sync tests and `#[tokio::test]` for async tests
- Tests should cover normal operation, edge cases, and error conditions
- Use `tempfile` crate for file system operations in tests

### Key Dependencies

- **clap**: CLI argument parsing with derive macros
- **tokio**: Async runtime for concurrent operations
- **serde/serde_json**: JSON serialization and output
- **comfy-table**: Table formatting for readable output
- **colored/colored_json**: Colored terminal output
- **indicatif**: Progress bars for long operations