use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::core::types::OutputFormat;
use crate::device::DeviceManager;
use crate::error::Result;
use crate::output::OutputFormatter;
use async_trait::async_trait;
use log::{debug, info};

pub struct LsCommand {
    device_manager: DeviceManager,
}

#[derive(Debug, Clone, clap::Args)]
pub struct LsArgs {
    /// Output format
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "table")]
    pub output: String,
}

impl LsCommand {
    pub fn new() -> Self {
        Self { 
            device_manager: DeviceManager::new()
        }
    }
}

#[async_trait]
impl SubCommand for LsCommand {
    type Args = LsArgs;
    
    async fn run(&self, _ctx: &CommandContext, args: Self::Args) -> Result<()> {
        debug!("LsCommand::run() called with args: {:?}", args);
        
        // Get list of devices
        debug!("Listing devices...");
        let devices = self.device_manager.list_devices().await?;
        debug!("Found {} devices", devices.len());
        
        if devices.is_empty() {
            info!("No devices found");
        } else {
            info!("Found {} device(s)", devices.len());
        }
        
        // Parse output format
        let output_format = OutputFormat::from_str(&args.output)
            .unwrap_or(OutputFormat::Table);
        
        // Create formatter
        let formatter = OutputFormatter::new();
        
        // Format and display
        match output_format {
            OutputFormat::Table => formatter.table(&devices)?,
            OutputFormat::Json => formatter.json(&devices)?,
            OutputFormat::Plain => formatter.plain(&devices)?,
        }
        
        Ok(())
    }
}

