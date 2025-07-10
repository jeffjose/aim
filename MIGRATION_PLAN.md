# Command Migration Plan

## Goal
Migrate all commands from `src/subcommands/` to `src/commands/` while keeping the existing CLI interface unchanged.

## Migration Status

### ✅ Completed
- [x] app/* - All app subcommands (list, info, clear, pull, etc.)
- [x] ls - List devices command

### 🔄 In Progress (Stubs Created)
- [ ] getprop - Get device properties
- [ ] screenshot - Take screenshot

### ⏳ Pending
- [ ] adb - Run arbitrary adb commands
- [ ] config - Display configuration  
- [ ] copy - Copy files to/from device
- [ ] dmesg - Run dmesg command
- [ ] perfetto - Run perfetto trace
- [ ] rename - Rename a device
- [ ] run - Run a command on device
- [ ] screenrecord - Record screen
- [ ] server - Manage ADB server

## Migration Steps for Each Command

1. Create new file in `src/commands/[command].rs`
2. Implement the `SubCommand` trait
3. Define Args struct with all parameters (alphabetically ordered)
4. Add device_id support where appropriate
5. Migrate functionality from old implementation
6. Update `src/commands/mod.rs` to export the command
7. Update main.rs to use the new command implementation
8. Test the command works correctly
9. Remove old implementation from subcommands/

## Command Structure Template

```rust
use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use async_trait::async_trait;

pub struct CommandNameCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct CommandNameArgs {
    // Positional arguments first
    
    // Optional arguments in alphabetical order
    
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
}

impl CommandNameCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for CommandNameCommand {
    type Args = CommandNameArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Implementation
    }
}
```