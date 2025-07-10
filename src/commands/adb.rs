use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use async_trait::async_trait;
use std::process::Command;

pub struct AdbCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct AdbArgs {
    /// ADB command to run
    pub command: String,
    
    /// Device ID
    pub device_id: Option<String>,
}

impl AdbCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for AdbCommand {
    type Args = AdbArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let mut cmd = Command::new("adb");
        
        // Add device selection if specified
        if let Some(device_id) = &args.device_id {
            cmd.arg("-s").arg(device_id);
        } else if let Some(device) = &ctx.device {
            cmd.arg("-s").arg(device.id.to_string());
        }
        
        // Split command into arguments
        let cmd_parts: Vec<&str> = args.command.split_whitespace().collect();
        for part in cmd_parts {
            cmd.arg(part);
        }
        
        // Execute command and inherit stdio
        let status = cmd.status()?;
        
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        
        Ok(())
    }
}