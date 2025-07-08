use crate::cli::{Cli, Commands};
use crate::commands::{ls::{LsCommand, LsArgs}, SubCommand};
use crate::core::context::{CommandContext, CommandContextBuilder};
use crate::core::types::OutputFormat;
use crate::device::DeviceManager;
use crate::error::{AimError, Result};
use crate::output::OutputFormatter;

/// Command runner that handles routing and execution
pub struct CommandRunner {
    device_manager: DeviceManager,
    output_formatter: OutputFormatter,
}

impl CommandRunner {
    /// Create a new command runner
    pub async fn new() -> Result<Self> {
        let device_manager = DeviceManager::new();
        let output_formatter = OutputFormatter::new();
        
        Ok(Self {
            device_manager,
            output_formatter,
        })
    }
    
    /// Run a command based on CLI arguments
    pub async fn run(&self, cli: Cli) -> Result<()> {
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
                let cmd = LsCommand::new(self.device_manager.clone());
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
            // Other commands would be handled here
            _ => {
                // For now, fall back to the old implementation
                return Err(AimError::Other("Command not yet refactored".to_string()));
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