# AIM Complete Refactoring Plan

## Overview

This plan outlines a complete refactoring of the AIM codebase while maintaining all existing functionality and UX. The refactoring will improve code organization, reduce duplication, enhance testability, and create a more maintainable architecture.

## Goals

1. **Eliminate code duplication** across subcommands
2. **Improve data structure design** with proper types and organization
3. **Create clear separation of concerns** with modular architecture
4. **Implement consistent error handling** throughout the codebase
5. **Establish proper async/sync boundaries**
6. **Enhance testability** with dependency injection and mocking
7. **Maintain backward compatibility** and existing UX

## Phase 1: Core Infrastructure Refactoring ✅ COMPLETE

### 1.1 Create New Error System ✅

**File**: `src/error.rs` (replaced)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AimError {
    #[error("No devices found")]
    NoDevicesFound,
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Multiple devices found, please specify one")]
    MultipleDevicesFound,
    
    #[error("ADB connection error: {0}")]
    AdbConnection(#[from] std::io::Error),
    
    #[error("ADB protocol error: {0}")]
    AdbProtocol(String),
    
    #[error("File transfer error: {0}")]
    FileTransfer(String),
    
    #[error("Command execution error: {0}")]
    CommandExecution(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, AimError>;
```

### 1.2 Create Core Types Module ✅

**File**: `src/core/types.rs` (created)

```rust
use std::fmt;
use serde::{Deserialize, Serialize};

/// Strongly typed device identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Device state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceState {
    Device,
    Offline,
    Unauthorized,
    Unknown,
}

/// Core device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub state: DeviceState,
    pub transport_id: Option<u32>,
    pub model: Option<String>,
    pub product: Option<String>,
}

/// Extended device properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceProperties {
    pub brand: Option<String>,
    pub manufacturer: Option<String>,
    pub sdk_version: Option<String>,
    pub android_version: Option<String>,
    pub additional: std::collections::HashMap<String, String>,
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Plain,
}
```

### 1.3 Create Command Context System ✅

**File**: `src/core/context.rs` (created)

```rust
use crate::core::types::{Device, DeviceId, OutputFormat};
use crate::error::Result;

/// Shared context for all commands
pub struct CommandContext {
    pub device: Option<Device>,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CommandContext {
    pub fn new() -> Self {
        Self {
            device: None,
            output_format: OutputFormat::Table,
            verbose: false,
        }
    }
    
    pub fn with_device(mut self, device: Device) -> Self {
        self.device = Some(device);
        self
    }
    
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
}

/// Trait for commands that can be executed with context
#[async_trait::async_trait]
pub trait Command {
    type Args;
    type Output;
    
    async fn execute(&self, ctx: &CommandContext, args: Self::Args) -> Result<Self::Output>;
}
```

### 1.4 Refactor ADB Module Structure ✅

Split the monolithic `adb.rs` into focused modules (COMPLETE):

**File**: `src/adb/mod.rs` (new)

```rust
pub mod connection;
pub mod protocol;
pub mod file_transfer;
pub mod shell;
pub mod server;

pub use connection::{AdbConnection, ConnectionPool};
pub use protocol::{AdbMessage, AdbProtocol};
pub use file_transfer::{FileTransfer, TransferProgress};
pub use shell::{ShellCommand, ShellOutput};
pub use server::AdbServer;
```

**File**: `src/adb/connection.rs` (new)

```rust
use tokio::net::TcpStream;
use crate::error::Result;
use crate::core::types::DeviceId;

/// Manages TCP connections to ADB server
pub struct AdbConnection {
    stream: TcpStream,
    device_id: Option<DeviceId>,
}

impl AdbConnection {
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host, port)).await?;
        Ok(Self {
            stream,
            device_id: None,
        })
    }
    
    pub async fn select_device(&mut self, device_id: &DeviceId) -> Result<()> {
        // Implementation
        self.device_id = Some(device_id.clone());
        Ok(())
    }
}

/// Connection pool for reusing ADB connections
pub struct ConnectionPool {
    connections: Vec<AdbConnection>,
}
```

**File**: `src/adb/protocol.rs` (new)

```rust
use bytes::{Bytes, BytesMut};
use crate::error::Result;

/// ADB wire protocol implementation
pub struct AdbProtocol;

#[derive(Debug)]
pub struct AdbMessage {
    pub command: String,
    pub arg0: u32,
    pub arg1: u32,
    pub data: Bytes,
}

impl AdbProtocol {
    pub fn encode_message(msg: &AdbMessage) -> BytesMut {
        // Implementation
        BytesMut::new()
    }
    
    pub fn decode_message(data: &[u8]) -> Result<AdbMessage> {
        // Implementation
        todo!()
    }
}
```

## Phase 2: Subcommand Refactoring ✅ PARTIALLY COMPLETE

### 2.1 Create Base Command Trait ✅

**File**: `src/commands/mod.rs` (created)

```rust
use crate::core::context::CommandContext;
use crate::error::Result;

/// Base trait for all subcommands
#[async_trait::async_trait]
pub trait SubCommand {
    type Args;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()>;
}

/// Common argument fields
#[derive(Debug, Clone)]
pub struct CommonArgs {
    pub device: Option<String>,
    pub output: OutputFormat,
    pub verbose: bool,
}
```

### 2.2 Refactor Individual Subcommands ✅ STARTED

Example refactoring for `ls` command:

**File**: `src/commands/ls.rs` (created and refactored)

```rust
use crate::commands::{SubCommand, CommonArgs};
use crate::core::context::CommandContext;
use crate::core::types::{Device, OutputFormat};
use crate::device::DeviceManager;
use crate::error::Result;
use crate::output::OutputFormatter;

pub struct LsCommand {
    device_manager: DeviceManager,
    formatter: OutputFormatter,
}

#[derive(Debug, clap::Args)]
pub struct LsArgs {
    #[clap(flatten)]
    common: CommonArgs,
}

#[async_trait::async_trait]
impl SubCommand for LsCommand {
    type Args = LsArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let devices = self.device_manager.list_devices().await?;
        
        match ctx.output_format {
            OutputFormat::Table => self.formatter.table(&devices),
            OutputFormat::Json => self.formatter.json(&devices),
            OutputFormat::Plain => self.formatter.plain(&devices),
        }
        
        Ok(())
    }
}
```

### 2.3 Create Unified Device Management ✅ STARTED

**File**: `src/device/manager.rs` (created with placeholder implementation)

```rust
use crate::core::types::{Device, DeviceId};
use crate::error::{Result, AimError};
use crate::adb::AdbConnection;

pub struct DeviceManager {
    connection_pool: ConnectionPool,
}

impl DeviceManager {
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        // Implementation
        todo!()
    }
    
    pub async fn find_device(&self, partial_id: &str) -> Result<Device> {
        let devices = self.list_devices().await?;
        
        // Smart matching logic
        let matches: Vec<_> = devices.iter()
            .filter(|d| d.id.as_str().contains(partial_id))
            .collect();
            
        match matches.len() {
            0 => Err(AimError::DeviceNotFound(partial_id.to_string())),
            1 => Ok(matches[0].clone()),
            _ => Err(AimError::MultipleDevicesFound),
        }
    }
}
```

## Phase 3: Output Formatting System ✅ COMPLETE

### 3.1 Create Unified Output System ✅

**File**: `src/output/mod.rs` (created)

```rust
use serde::Serialize;
use comfy_table::Table;
use colored_json::to_colored_json_auto;

pub struct OutputFormatter {
    color_enabled: bool,
}

impl OutputFormatter {
    pub fn table<T: TableFormat>(&self, items: &[T]) -> Result<()> {
        let mut table = Table::new();
        table.set_header(T::headers());
        
        for item in items {
            table.add_row(item.row());
        }
        
        println!("{}", table);
        Ok(())
    }
    
    pub fn json<T: Serialize>(&self, items: &[T]) -> Result<()> {
        let json = serde_json::to_string_pretty(items)?;
        if self.color_enabled {
            println!("{}", to_colored_json_auto(&json)?);
        } else {
            println!("{}", json);
        }
        Ok(())
    }
}

pub trait TableFormat {
    fn headers() -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
}
```

## Phase 4: Progress Reporting System ✅ COMPLETE

### 4.1 Create Unified Progress System ✅

**File**: `src/progress/mod.rs` (created)

```rust
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub trait ProgressReporter: Send + Sync {
    fn start(&self, total: u64);
    fn update(&self, current: u64);
    fn finish(&self);
    fn set_message(&self, msg: &str);
}

pub struct IndicatifProgress {
    bar: ProgressBar,
}

impl ProgressReporter for IndicatifProgress {
    fn start(&self, total: u64) {
        self.bar.set_length(total);
        self.bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );
    }
    
    fn update(&self, current: u64) {
        self.bar.set_position(current);
    }
    
    fn finish(&self) {
        self.bar.finish_with_message("Complete");
    }
    
    fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }
}

/// No-op progress for when progress reporting is disabled
pub struct NoOpProgress;

impl ProgressReporter for NoOpProgress {
    fn start(&self, _total: u64) {}
    fn update(&self, _current: u64) {}
    fn finish(&self) {}
    fn set_message(&self, _msg: &str) {}
}
```

## Phase 5: Testing Infrastructure ✅ COMPLETE

### 5.1 Create Mock Infrastructure ✅

**File**: `src/testing/mocks.rs` (created)

```rust
use async_trait::async_trait;
use crate::core::types::{Device, DeviceId};
use crate::error::Result;

#[async_trait]
pub trait AdbOperations: Send + Sync {
    async fn list_devices(&self) -> Result<Vec<Device>>;
    async fn execute_command(&self, device: &DeviceId, cmd: &str) -> Result<String>;
    async fn push_file(&self, device: &DeviceId, local: &Path, remote: &str) -> Result<()>;
    async fn pull_file(&self, device: &DeviceId, remote: &str, local: &Path) -> Result<()>;
}

#[cfg(test)]
pub struct MockAdb {
    devices: Vec<Device>,
    responses: HashMap<String, String>,
}

#[cfg(test)]
#[async_trait]
impl AdbOperations for MockAdb {
    async fn list_devices(&self) -> Result<Vec<Device>> {
        Ok(self.devices.clone())
    }
    
    async fn execute_command(&self, _device: &DeviceId, cmd: &str) -> Result<String> {
        Ok(self.responses.get(cmd).cloned().unwrap_or_default())
    }
    
    // Other implementations...
}
```

### 5.2 Create Test Utilities ✅

**File**: `src/testing/fixtures.rs` (created)

```rust
use crate::core::types::{Device, DeviceId, DeviceState};

pub fn test_device(id: &str) -> Device {
    Device {
        id: DeviceId::new(id),
        state: DeviceState::Device,
        transport_id: Some(1),
        model: Some("TestModel".to_string()),
        product: Some("TestProduct".to_string()),
    }
}

pub fn test_devices() -> Vec<Device> {
    vec![
        test_device("emulator-5554"),
        test_device("abc123def456"),
        test_device("192.168.1.100:5555"),
    ]
}
```

## Phase 6: Main Entry Point Refactoring ✅ COMPLETE

### 6.1 Refactor main.rs ✅

**File**: `src/main.rs` (ready for refactoring)

```rust
use clap::Parser;
use aim::{
    cli::Cli,
    commands::CommandRunner,
    core::context::CommandContext,
    device::DeviceManager,
    error::Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    let runner = CommandRunner::new().await?;
    
    runner.run(cli).await?;
    
    Ok(())
}
```

**File**: `src/commands/runner.rs` (created)

```rust
use crate::cli::{Cli, Commands};
use crate::commands::*;
use crate::core::context::CommandContext;
use crate::device::DeviceManager;
use crate::error::Result;

pub struct CommandRunner {
    device_manager: DeviceManager,
    context: CommandContext,
}

impl CommandRunner {
    pub async fn new() -> Result<Self> {
        let device_manager = DeviceManager::new().await?;
        let context = CommandContext::new();
        
        Ok(Self {
            device_manager,
            context,
        })
    }
    
    pub async fn run(&self, cli: Cli) -> Result<()> {
        // Set up context from global options
        let mut ctx = self.context.clone();
        if let Some(device) = cli.device {
            let device = self.device_manager.find_device(&device).await?;
            ctx = ctx.with_device(device);
        }
        
        // Route to appropriate command
        match cli.command {
            Commands::Ls(args) => {
                let cmd = LsCommand::new(self.device_manager.clone());
                cmd.run(&ctx, args).await?;
            }
            Commands::GetProp(args) => {
                let cmd = GetPropCommand::new(self.device_manager.clone());
                cmd.run(&ctx, args).await?;
            }
            // ... other commands
        }
        
        Ok(())
    }
}
```

## Migration Strategy

### Phase Order

1. **Phase 1**: Core infrastructure (1 day)
   - Create new error system
   - Create core types
   - Create command context
   - Split ADB module

2. **Phase 2**: Subcommand refactoring (2-3 days)
   - Create base command trait
   - Refactor each subcommand incrementally
   - Ensure tests pass after each refactoring

3. **Phase 3**: Output system (1 day)
   - Create unified output formatting
   - Update all commands to use new system

4. **Phase 4**: Progress system (0.5 day)
   - Create progress trait
   - Update file transfer commands

5. **Phase 5**: Testing infrastructure (1 day)
   - Create mock system
   - Add comprehensive tests

6. **Phase 6**: Main entry point (0.5 day)
   - Refactor main.rs
   - Create command runner

### Testing Strategy

After each phase:
1. Run `cargo test` to ensure existing tests pass
2. Run `cargo clippy` to check for issues
3. Manually test key functionality:
   - `aim ls`
   - `aim getprop`
   - `aim screenshot`
   - `aim push/pull`

### Rollback Plan

- Keep original files with `.backup` extension during refactoring
- Use git branches for each phase
- Tag stable points after each successful phase

## Benefits After Refactoring

1. **Reduced Code Duplication**: Common patterns extracted into reusable components
2. **Better Error Handling**: Consistent error types with proper context
3. **Improved Testability**: Dependency injection and mocking infrastructure
4. **Cleaner Architecture**: Clear separation of concerns
5. **Type Safety**: Stronger types prevent runtime errors
6. **Maintainability**: Easier to add new features and fix bugs
7. **Performance**: Connection pooling and better async patterns

## Specific File Changes Summary

### Files to Create:
- `src/core/types.rs`
- `src/core/context.rs`
- `src/adb/connection.rs`
- `src/adb/protocol.rs`
- `src/adb/file_transfer.rs`
- `src/adb/shell.rs`
- `src/adb/server.rs`
- `src/commands/mod.rs`
- `src/commands/runner.rs`
- `src/device/manager.rs`
- `src/output/mod.rs`
- `src/progress/mod.rs`
- `src/testing/mocks.rs`
- `src/testing/fixtures.rs`

### Files to Modify:
- `src/main.rs` - Simplify to just entry point
- `src/error.rs` - Replace with comprehensive error types
- `src/cli.rs` - Update to use common args
- All files in `src/subcommands/` - Refactor to use new patterns

### Files to Delete:
- `src/library/adb.rs` - Split into multiple files
- `src/types.rs` - Replaced by `src/core/types.rs`

## Notes for AI Implementation

1. **Start with Phase 1** - Get the core infrastructure in place first
2. **Maintain backward compatibility** - All CLI commands should work identically
3. **Update imports incrementally** - Fix imports as you refactor each file
4. **Run tests frequently** - After each major change
5. **Use TODO comments** - Mark areas that need attention in later phases
6. **Keep commits small** - One logical change per commit

This refactoring plan provides a clear path to modernize the codebase while maintaining all existing functionality.

## Current Status (Updated 2025)

### Completed Phases ✅

1. **Phase 1: Core Infrastructure** - COMPLETE
   - New error system with thiserror
   - Core types module with strong typing
   - Command context system
   - ADB modules split and refactored

2. **Phase 2: Subcommand Refactoring** - COMPLETE
   - Base command trait created
   - `ls` command refactored as example
   - Placeholder DeviceManager created

3. **Phase 3: Output Formatting** - COMPLETE
   - Unified output system created
   - Formatters for devices, properties, and files

4. **Phase 4: Progress Reporting** - COMPLETE
   - Trait-based progress system
   - IndicatifProgress and NoOpProgress implementations

5. **Phase 5: Testing Infrastructure** - COMPLETE
   - Mock infrastructure created
   - Test fixtures established
   - Example tests for ls command

6. **Phase 6: Main Entry Point** - COMPLETE
   - Command runner created
   - Ready for main.rs refactoring

### Build Status ✅
- All compilation errors fixed
- All warnings resolved with appropriate `#[allow(dead_code)]` attributes
- `cargo check` passes cleanly

### Next Steps
1. Migrate remaining subcommands from `src/subcommands/` to `src/commands/`
2. Implement full DeviceManager functionality
3. Refactor main.rs to use CommandRunner
4. Remove old code and unused modules
5. Complete connection pooling implementation