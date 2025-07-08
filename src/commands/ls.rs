use super::{SubCommand, format_json_output};
use crate::core::context::CommandContext;
use crate::core::types::{Device, DeviceProperties, OutputFormat};
use crate::device::DeviceManager;
use crate::error::Result;
use crate::types::DeviceDetails;
use async_trait::async_trait;
use comfy_table::Table;
use serde::{Deserialize, Serialize};

pub struct LsCommand {
    device_manager: DeviceManager,
}

#[derive(Debug, clap::Args)]
pub struct LsArgs {
    /// Output format
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "table")]
    pub output: String,
}

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
            
        match output_format {
            OutputFormat::Table => self.format_table(&device_details)?,
            OutputFormat::Json => format_json_output(&device_details)?,
            OutputFormat::Plain => self.format_plain(&device_details)?,
        }
        
        Ok(())
    }
}

impl LsCommand {
    fn format_table(&self, devices: &[DeviceDetails]) -> Result<()> {
        let mut table = Table::new();
        
        // Set headers
        table.set_header(vec![
            "DEVICE ID",
            "BRAND", 
            "MODEL",
            "STATUS",
            "ADB ID",
            "NAME"
        ]);
        
        table.load_preset(comfy_table::presets::NOTHING);
        
        for device in devices {
            let status = if device.additional_props.get("service.adb.root") == Some(&"1".to_string()) {
                "root"
            } else {
                ""
            };
            
            table.add_row(vec![
                device.device_id_short.clone(),
                device.brand.clone().unwrap_or_default(),
                device.model.clone().unwrap_or_default(),
                status.to_string(),
                device.adb_id.clone(),
                device.device_name.clone(),
            ]);
        }
        
        println!("{}", table);
        Ok(())
    }
    
    fn format_plain(&self, devices: &[DeviceDetails]) -> Result<()> {
        for device in devices {
            println!("{}", device.adb_id);
        }
        Ok(())
    }
}

