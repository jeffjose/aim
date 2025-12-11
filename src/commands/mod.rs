use crate::core::context::CommandContext;
use crate::core::types::OutputFormat;
use crate::error::Result;
use async_trait::async_trait;

/// Base trait for all subcommands
#[async_trait]
#[allow(dead_code)]
pub trait SubCommand {
    type Args;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()>;
}

/// Common argument fields shared by most commands
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommonArgs {
    pub device: Option<String>,
    pub output: OutputFormat,
    pub verbose: bool,
}

#[allow(dead_code)]
impl CommonArgs {
    /// Convert string output format to enum
    pub fn parse_output_format(s: &str) -> OutputFormat {
        OutputFormat::from_str(s).unwrap_or(OutputFormat::Table)
    }
}

/// Helper trait for commands that produce formatted output
#[async_trait]
#[allow(dead_code)]
pub trait OutputCommand {
    type Output: serde::Serialize;
    
    async fn execute(&self, ctx: &CommandContext) -> Result<Self::Output>;
    
    async fn run_with_output(&self, ctx: &CommandContext) -> Result<()> {
        let output = self.execute(ctx).await?;
        
        match ctx.output_format {
            OutputFormat::Table => self.format_table(&output),
            OutputFormat::Json => self.format_json(&output),
            OutputFormat::Plain => self.format_plain(&output),
        }
    }
    
    fn format_table(&self, output: &Self::Output) -> Result<()>;
    fn format_json(&self, output: &Self::Output) -> Result<()>;
    fn format_plain(&self, output: &Self::Output) -> Result<()>;
}

/// Default JSON formatting implementation
#[allow(dead_code)]
pub fn format_json_output<T: serde::Serialize>(output: &T) -> Result<()> {
    // Use the existing utility function
    crate::utils::print_colored_json(output)?;
    Ok(())
}

/// Helper for device selection in commands
#[allow(dead_code)]
pub async fn select_device(
    ctx: &CommandContext,
    device_arg: Option<&str>,
) -> Result<Option<crate::core::types::Device>> {
    use crate::device::DeviceManager;
    
    // If context already has a device, use it
    if let Some(device) = &ctx.device {
        return Ok(Some(device.clone()));
    }
    
    // Get list of devices
    let device_manager = DeviceManager::new();
    let devices = device_manager.list_devices().await?;
    
    if devices.is_empty() {
        return Err(crate::error::AimError::NoDevicesFound);
    }
    
    // If device specified, find it
    if let Some(device_id) = device_arg {
        let device = device_manager.find_device(device_id).await?;
        Ok(Some(device))
    } else if devices.len() == 1 {
        // Single device, use it automatically
        Ok(Some(devices.into_iter().next().unwrap()))
    } else {
        // Multiple devices, require selection
        Err(crate::error::AimError::DeviceIdRequired)
    }
}

/// Module re-exports
pub mod app;
pub mod runner;

// Individual command modules
pub mod ls;
pub mod getprop;
pub mod screenshot;
pub mod run;
pub mod copy;
pub mod rename;
pub mod server;
pub mod adb;
pub mod config;
pub mod dmesg;
pub mod perfetto;
pub mod screenrecord;

// New commands (matching README expectations)
pub mod push;
pub mod pull;
pub mod shell;

// Tests for commands are in individual *_test.rs files
// Currently: config_test.rs, device_info_test.rs, hash_test.rs, protocol_test.rs

// Re-export command implementations
// pub use getprop::GetPropCommand;
// pub use screenshot::ScreenshotCommand;
// pub use screenrecord::ScreenRecordCommand;
// pub use copy::{PushCommand, PullCommand};
// pub use dmesg::DmesgCommand;
// pub use perfetto::PerfettoCommand;
// pub use server::ServerCommand;
// pub use shell::ShellCommand;
// pub use logcat::LogcatCommand;
// pub use run::RunCommand;
// pub use rename::RenameCommand;
// pub use config::ConfigCommand;