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

/// Resolve a device ID, checking if it's an alias from config first
///
/// If the provided ID matches a device alias (e.g., "p10"), returns the
/// corresponding device key (e.g., "510") that can be used for partial matching.
pub fn resolve_device_alias(device_id: Option<&str>) -> Option<String> {
    use crate::config::Config;
    use std::path::PathBuf;

    let id = device_id?;

    // Load config and check if this is an alias
    let config_path = dirs::home_dir()
        .map(|p| p.join(".config/aim/config.toml"))
        .unwrap_or_else(|| PathBuf::from(".config/aim/config.toml"));
    let config = Config::load_from_path(&config_path);

    // Check if any device config has this name as an alias
    for (device_key, device_config) in &config.devices {
        if let Some(name) = &device_config.name {
            if name.eq_ignore_ascii_case(id) {
                // Return the device key (partial ID) instead of the alias
                return Some(device_key.clone());
            }
        }
    }

    // Not an alias, return as-is
    Some(id.to_string())
}

/// Helper for device selection in commands - supports aliases and partial IDs
pub async fn get_device(
    device_arg: Option<&str>,
) -> Result<crate::core::types::Device> {
    use crate::device::DeviceManager;

    let device_manager = DeviceManager::new();

    // Resolve alias first
    let resolved_id = resolve_device_alias(device_arg);

    device_manager.get_target_device(resolved_id.as_deref()).await
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

    // Resolve alias first
    let resolved_id = resolve_device_alias(device_arg);

    // If device specified, find it
    if let Some(device_id) = resolved_id.as_deref() {
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