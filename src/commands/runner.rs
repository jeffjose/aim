use crate::cli::{Cli, Commands};
use crate::commands::{
    ls::{LsCommand, LsArgs},
    run::{RunCommand, RunArgs},
    copy::{CopyCommand, CopyArgs},
    rename::{RenameCommand, RenameArgs},
    server::{ServerCommand, ServerArgs},
    adb::{AdbCommand, AdbArgs},
    config::{ConfigCommand, ConfigArgs},
    dmesg::{DmesgCommand, DmesgArgs},
    perfetto::{PerfettoCommand, PerfettoArgs},
    screenrecord::{ScreenrecordCommand, ScreenrecordArgs},
    getprop::{GetpropCommand, GetpropArgs},
    screenshot::{ScreenshotCommand, ScreenshotArgs},
    SubCommand,
};
use crate::core::context::CommandContextBuilder;
use crate::core::types::OutputFormat;
use crate::device::DeviceManager;
use crate::error::{AimError, Result};
use crate::output::OutputFormatter;
use log::debug;

/// Command runner that handles routing and execution
#[allow(dead_code)]
pub struct CommandRunner {
    device_manager: DeviceManager,
    _output_formatter: OutputFormatter,
}

#[allow(dead_code)]
impl CommandRunner {
    /// Create a new command runner
    pub async fn new() -> Result<Self> {
        debug!("CommandRunner::new() called");
        let device_manager = DeviceManager::new();
        let output_formatter = OutputFormatter::new();
        
        debug!("CommandRunner initialized");
        Ok(Self {
            device_manager,
            _output_formatter: output_formatter,
        })
    }
    
    /// Run a command based on CLI arguments
    pub async fn run(&self, cli: Cli) -> Result<()> {
        debug!("CommandRunner::run() called with command: {:?}", cli.command());
        
        // Build context from global options
        let mut context_builder = CommandContextBuilder::new();
        
        // Set output format from global option
        let output_format = match cli.output {
            crate::cli::OutputType::Table => OutputFormat::Table,
            crate::cli::OutputType::Json => OutputFormat::Json,
            crate::cli::OutputType::Plain => OutputFormat::Plain,
        };
        context_builder = context_builder.output_format(output_format);
        
        // Set verbose mode
        let verbose_level = cli.verbose.log_level();
        context_builder = context_builder.verbose(verbose_level.is_some());
        
        let ctx = context_builder.build();
        
        // Route to appropriate command
        match cli.command() {
            Commands::Ls { output } => {
                let cmd = LsCommand::new();
                let output_str = match output {
                    crate::cli::OutputType::Table => "table",
                    crate::cli::OutputType::Json => "json",
                    crate::cli::OutputType::Plain => "plain",
                };
                let args = LsArgs {
                    output: output_str.to_string(),
                };
                cmd.run(&ctx, args).await?;
            }
            Commands::Run { command, device_id, filters, watch } => {
                let cmd = RunCommand::new();
                let args = RunArgs {
                    command,
                    device_id,
                    filters,
                    watch,
                };
                cmd.run(&ctx, args).await?;
            }
            Commands::Copy { src, dst } => {
                let cmd = CopyCommand::new();
                let args = CopyArgs { src, dst };
                cmd.run(&ctx, args).await?;
            }
            Commands::Rename { device_id, new_name } => {
                let cmd = RenameCommand::new();
                let args = RenameArgs { device_id, new_name };
                cmd.run(&ctx, args).await?;
            }
            Commands::Server { operation } => {
                let cmd = ServerCommand::new();
                let args = ServerArgs { operation };
                cmd.run(&ctx, args).await?;
            }
            Commands::Adb { command, device_id } => {
                let cmd = AdbCommand::new();
                let args = AdbArgs { command, device_id };
                cmd.run(&ctx, args).await?;
            }
            Commands::Config => {
                let cmd = ConfigCommand::new();
                let args = ConfigArgs { path_only: false };
                cmd.run(&ctx, args).await?;
            }
            Commands::Dmesg { device_id, args: dmesg_args } => {
                let cmd = DmesgCommand::new();
                let args = DmesgArgs { device_id, args: dmesg_args };
                cmd.run(&ctx, args).await?;
            }
            Commands::Perfetto { config, device_id, output, time } => {
                let cmd = PerfettoCommand::new();
                let args = PerfettoArgs { device_id, config, time, output };
                cmd.run(&ctx, args).await?;
            }
            Commands::Screenrecord { device_id, output, args: sr_args } => {
                let cmd = ScreenrecordCommand::new();
                let args = ScreenrecordArgs { device_id, output, args: sr_args };
                cmd.run(&ctx, args).await?;
            }
            Commands::Getprop { propnames, device_id, output } => {
                let cmd = GetpropCommand::new();
                let args = GetpropArgs { propnames, device_id, output };
                cmd.run(&ctx, args).await?;
            }
            Commands::Screenshot { args: ss_args, device_id, interactive, output } => {
                let cmd = ScreenshotCommand::new();
                let args = ScreenshotArgs { device_id, interactive, output, args: ss_args };
                cmd.run(&ctx, args).await?;
            }
            Commands::App { .. } => {
                // App commands are still handled by the old implementation
                return Err(AimError::Other("App commands not yet migrated to new runner".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Check if any devices are available
    pub async fn check_devices(&self) -> Result<bool> {
        let devices = self.device_manager.list_devices().await?;
        Ok(!devices.is_empty())
    }
}

/// Helper to get the default ADB host and port
#[allow(dead_code)]
pub fn get_adb_connection_params() -> (&'static str, u16) {
    let host = std::env::var("ADB_SERVER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("ADB_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5037);
    
    // Return static string for host
    if host == "localhost" {
        ("localhost", port)
    } else {
        // In a real implementation, we'd handle this better
        ("localhost", port)
    }
}