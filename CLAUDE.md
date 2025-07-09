# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**aim** is a modern CLI tool written in Rust that serves as an improved interface for Android Debug Bridge (ADB). It enhances the ADB experience with better output formatting, intuitive commands, and improved usability.

## Recent Refactoring (2025)

The codebase has undergone a major refactoring to improve maintainability, type safety, and testability:

- **New error system** using `thiserror` for better error handling
- **Strongly-typed core types** for devices, IDs, and output formats
- **Modular ADB implementation** split into focused modules
- **Unified output formatting** and progress reporting systems
- **Testing infrastructure** with mocks and fixtures
- **Command pattern** for consistent subcommand implementation

## Key Commands

### Build and Development
```bash
cargo build              # Build the project
cargo build --release    # Build optimized release version
cargo run -- [args]      # Run aim with arguments during development
cargo install --path .   # Install aim locally
cargo fix               # Fix common warnings automatically
```

### Testing and Quality
```bash
cargo test              # Run all tests
cargo test [module]     # Run tests for specific module
cargo test --lib        # Run library tests only
cargo clippy            # Run Clippy linter for code quality
cargo fmt               # Format code using rustfmt
cargo check             # Quick compilation check without building
```

### Common Development Patterns
```bash
# Test a single subcommand implementation
cargo test commands::ls_test

# Run aim during development
cargo run -- ls
cargo run -- getprop ro.product.model
cargo run -- screenshot -o test.png

# Check for compilation errors quickly
cargo check

# Fix warnings
cargo fix --lib -p aim
cargo fix --bin aim
```

## High-Level Architecture (Updated)

The codebase follows a clean modular architecture designed for extensibility:

### Core Structure
- **`src/main.rs`**: Entry point with minimal logic
- **`src/cli.rs`**: CLI structure using clap, defines all subcommands and their arguments
- **`src/error.rs`**: Comprehensive error types using `thiserror`
- **`src/types.rs`**: Legacy type definitions (being migrated to core::types)
- **`src/utils.rs`**: Utility functions for common operations

### New Core Modules

1. **Core Types (`src/core/`)**
   - `types.rs`: Strongly-typed `Device`, `DeviceId`, `OutputFormat`, etc.
   - `context.rs`: `CommandContext` for shared command state

2. **ADB Implementation (`src/adb/`)**
   - `connection.rs`: TCP connection management with connection pooling
   - `protocol.rs`: Wire protocol implementation and data structures
   - `file_transfer.rs`: Push/pull operations with progress
   - `shell.rs`: Command execution and output streaming
   - `server.rs`: ADB server lifecycle management

3. **Commands (`src/commands/`)**
   - `mod.rs`: Base `SubCommand` trait and common functionality
   - `runner.rs`: Command routing and execution
   - Individual command implementations (e.g., `ls.rs`)
   - Test modules for each command

4. **Output System (`src/output/`)**
   - `mod.rs`: Unified `OutputFormatter` for all output types
   - `device.rs`: Device-specific formatting
   - `property.rs`: Property formatting with color support
   - `file.rs`: File listing and transfer formatting

5. **Progress System (`src/progress/`)**
   - Trait-based progress reporting
   - Support for file transfers, commands, and custom progress

6. **Testing Infrastructure (`src/testing/`)**
   - `mocks.rs`: Mock implementations for testing
   - `fixtures.rs`: Common test data and scenarios

7. **Device Management (`src/device/`)**
   - `device_info.rs`: Device detection and information
   - `manager.rs`: Unified device management (placeholder)

### Architectural Patterns

1. **Async Architecture**: Built on Tokio for efficient async I/O operations
2. **Error Handling**: Comprehensive error types with `thiserror`, automatic conversions
3. **Output Formatting**: Unified output system with trait-based formatting
4. **Device Selection**: Smart device selection with partial matching and aliases
5. **Configuration**: TOML-based config at `~/.config/aim/config.toml`
6. **Dependency Injection**: Mock-friendly design for testing
7. **Command Pattern**: Consistent interface for all subcommands

### Adding New Subcommands (New Pattern)

To add a new subcommand using the refactored architecture:
1. Create a new file in `src/commands/[command].rs`
2. Implement the `SubCommand` trait for your command
3. Define args struct with `clap::Args` derive
4. Add command to `src/cli.rs` Commands enum
5. Add routing in `src/commands/runner.rs`
6. Write tests in `src/commands/[command]_test.rs`

Example:
```rust
use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;

pub struct MyCommand {
    // dependencies
}

#[derive(Debug, clap::Args)]
pub struct MyCommandArgs {
    // command-specific arguments
    
    /// Output format (for commands that produce data)
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "table")]
    pub output: String,
}

#[async_trait]
impl SubCommand for MyCommand {
    type Args = MyCommandArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // implementation
    }
}
```

#### Output Format Guidelines

For commands that produce queryable data (not action commands):
- Always include `-o, --output` flag with options: `table`, `json`, `plain`
- Use consistent field name: `output: String`
- Convert using: `OutputFormat::from_str(&args.output)`
- Use `OutputFormatter` for consistent formatting
- Default to `"table"` for lists, `"plain"` for single items

Action commands (start, stop, clear, etc.) should NOT have output format options.

### Testing Strategy

- Tests live in separate `*_test.rs` files in the same module
- Use the testing infrastructure in `src/testing/`
- Mock external dependencies using traits
- Test fixtures available for common scenarios
- Run with `cargo test --lib` for faster testing

### Key Dependencies

- **clap**: CLI argument parsing with derive macros
- **tokio**: Async runtime for concurrent operations
- **serde/serde_json**: JSON serialization and output
- **comfy-table**: Table formatting for readable output
- **colored/colored_json**: Colored terminal output
- **indicatif**: Progress bars for long operations
- **thiserror**: Error type derivation
- **async-trait**: Async traits for commands
- **bytes**: Efficient byte buffer handling

### Migration Status

Currently migrating from the old architecture to the new one:
- ✅ Core infrastructure (error, types, context)
- ✅ ADB modules split and refactored
- ✅ Output and progress systems
- ✅ Testing infrastructure
- ✅ Command runner and base trait
- 🔄 `ls` command migrated as example
- ⏳ Other subcommands pending migration

The old subcommands in `src/subcommands/` still work but will be gradually migrated to `src/commands/`.