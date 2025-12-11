# AIM Architecture

## Overview

**aim** is a Rust CLI tool providing an improved interface for Android Debug Bridge (ADB). It enhances ADB with better output formatting, intuitive commands, and improved usability.

## Directory Structure

```
src/
├── main.rs              # Entry point
├── cli.rs               # CLI structure (clap)
├── lib.rs               # Library exports
├── error.rs             # Error types (thiserror)
├── config.rs            # Configuration loading
├── utils.rs             # Utility functions
│
├── commands/            # Command implementations
│   ├── mod.rs           # SubCommand trait
│   ├── runner.rs        # Command routing
│   ├── ls.rs            # List devices
│   ├── run.rs           # Run shell commands
│   ├── copy.rs          # File transfer
│   ├── rename.rs        # Device aliases
│   ├── server.rs        # ADB server management
│   ├── adb.rs           # Pass-through to adb
│   ├── config.rs        # Show config
│   ├── dmesg.rs         # Kernel logs
│   ├── perfetto.rs      # Perfetto tracing
│   ├── screenrecord.rs  # Screen recording
│   ├── getprop.rs       # Device properties
│   ├── screenshot.rs    # Screenshots
│   └── app/             # App management
│       ├── list.rs, clear.rs, start.rs, stop.rs, pull.rs, backup.rs
│
├── library/             # ADB implementation (legacy, active)
│   ├── adb.rs           # Main ADB operations
│   ├── protocol.rs      # Wire protocol
│   └── hash.rs          # File hashing
│
├── core/                # Core types
│   ├── types.rs         # Device, DeviceId, OutputFormat
│   └── context.rs       # CommandContext
│
├── device/              # Device management
│   ├── device_info.rs   # Device detection
│   └── manager.rs       # DeviceManager
│
├── output/              # Output formatting
│   ├── mod.rs           # OutputFormatter
│   ├── device.rs        # Device formatting
│   ├── property.rs      # Property formatting
│   └── file.rs          # File formatting
│
├── progress/            # Progress reporting
│   └── mod.rs           # ProgressReporter trait
│
└── testing/             # Test infrastructure
    ├── mocks.rs         # Mock implementations
    └── fixtures.rs      # Test data
```

## Key Patterns

### Command Pattern

All commands implement the `SubCommand` trait:

```rust
#[async_trait]
pub trait SubCommand {
    type Args;
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()>;
}
```

### Error Handling

Uses `thiserror` for comprehensive error types:

```rust
#[derive(Error, Debug)]
pub enum AimError {
    #[error("No devices found")]
    NoDevicesFound,
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    // ...
}
```

### Output Formatting

Unified output with `OutputFormatter`:
- Table format (default for lists)
- JSON format (machine-readable)
- Plain format (simple text)

### Device Selection

Smart matching with partial IDs and aliases:
```bash
aim -d pixel screenshot    # Matches "pixel6-abc123"
aim -d work ls             # Matches device aliased as "work"
```

## Configuration

Location: `~/.config/aim/config.toml`

```toml
[devices]
work = { id = "28291FDH200001", name = "Pixel 6 (work)" }

[screenshot]
output = "~/Pictures/Screenshots"

[screenrecord]
output = "~/Videos"
```

## Adding New Commands

1. Create `src/commands/[command].rs`
2. Implement `SubCommand` trait
3. Define args struct with `clap::Args`
4. Add to `Commands` enum in `src/cli.rs`
5. Add routing in `src/commands/runner.rs`

Example:

```rust
pub struct MyCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct MyCommandArgs {
    /// Device ID
    pub device_id: Option<String>,

    /// Output format
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "table")]
    pub output: String,
}

impl MyCommand {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl SubCommand for MyCommand {
    type Args = MyCommandArgs;

    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Implementation
    }
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| clap | CLI argument parsing |
| tokio | Async runtime |
| serde/serde_json | Serialization |
| comfy-table | Table output |
| colored | Terminal colors |
| indicatif | Progress bars |
| thiserror | Error types |
| async-trait | Async traits |

## Testing

```bash
cargo test              # Run all tests
cargo test commands::   # Test commands module
cargo clippy            # Lint check
cargo fmt               # Format code
```

Tests use the infrastructure in `src/testing/`:
- `mocks.rs` - Mock ADB operations
- `fixtures.rs` - Test device data
