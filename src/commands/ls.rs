use super::SubCommand;
use crate::core::context::CommandContext;
use crate::core::types::OutputFormat;
use crate::device::DeviceManager;
use crate::error::Result;
use crate::output::OutputFormatter;
use async_trait::async_trait;

pub struct LsCommand {
    device_manager: DeviceManager,
}

#[derive(Debug, clap::Args)]
pub struct LsArgs {
    /// Output format
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "table")]
    pub output: String,
}

#[allow(dead_code)]
impl LsCommand {
    pub fn new(device_manager: DeviceManager) -> Self {
        Self { device_manager }
    }
}

#[async_trait]
impl SubCommand for LsCommand {
    type Args = LsArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Get devices from device manager
        let devices = self.device_manager.list_devices().await?;
        
        // Convert to DeviceDetails for compatibility
        let mut device_details = Vec::new();
        for device in devices {
            let details = self.device_manager.get_device_details(device).await?;
            device_details.push(details);
        }
        
        // Format output based on context
        let output_format = OutputFormat::from_str(&args.output)
            .unwrap_or(ctx.output_format);
            
        // Use the unified output formatter
        let formatter = OutputFormatter::new()
            .with_quiet(ctx.quiet);
            
        match output_format {
            OutputFormat::Table => formatter.table(&device_details)?,
            OutputFormat::Json => formatter.json(&device_details)?,
            OutputFormat::Plain => formatter.plain(&device_details)?,
        }
        
        Ok(())
    }
}

