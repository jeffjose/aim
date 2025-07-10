use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::run_shell_command_async;
use async_trait::async_trait;

pub struct DmesgCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct DmesgArgs {
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
    
    /// Additional arguments to pass to dmesg
    #[clap(trailing_var_arg = true)]
    pub args: Vec<String>,
}

impl DmesgCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for DmesgCommand {
    type Args = DmesgArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Build dmesg command with any additional arguments
        let mut command = "dmesg".to_string();
        if !args.args.is_empty() {
            command = format!("{} {}", command, args.args.join(" "));
        }
        
        // Execute the command
        let device_id = device.id.to_string();
        let port_str = port.to_string();
        let output = run_shell_command_async(host, &port_str, &command, Some(&device_id)).await?;
        
        // Print output directly (dmesg output is typically line-based)
        print!("{}", output);
        
        Ok(())
    }
}