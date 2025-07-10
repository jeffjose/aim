use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::run_shell_command_async;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

pub struct RunCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct RunArgs {
    /// The command to execute
    pub command: String,
    
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
    
    /// Filter devices by property (format: key=value)
    #[clap(short = 'f', long = "filter", num_args = 1)]
    pub filters: Vec<String>,
    
    /// Watch mode - repeat command every second. Optional value specifies duration in seconds
    #[clap(short = 'w', long = "watch", num_args = 0..=1, default_missing_value = "0")]
    pub watch: Option<u32>,
}

impl RunCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for RunCommand {
    type Args = RunArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Apply filters if any
        if !args.filters.is_empty() {
            // TODO: Implement device filtering
            eprintln!("Warning: Device filters not yet implemented in new architecture");
        }
        
        if let Some(duration) = args.watch {
            // Watch mode
            let interval = if duration == 0 { 1 } else { duration };
            println!("Executing: {}", args.command);
            println!("Press Ctrl+C to stop\n");
            
            loop {
                self.execute_command(host, port, &device.id, &args.command).await?;
                
                // Clear screen for next iteration
                print!("\x1B[2J\x1B[H");
                println!("Executing: {} (every {}s)", args.command, interval);
                println!("Press Ctrl+C to stop\n");
                
                sleep(Duration::from_secs(interval as u64)).await;
            }
        } else {
            // Single execution
            self.execute_command(host, port, &device.id, &args.command).await
        }
    }
}

impl RunCommand {
    async fn execute_command(
        &self,
        host: &str,
        port: u16,
        device_id: &crate::core::types::DeviceId,
        command: &str,
    ) -> Result<()> {
        let device_id_str = device_id.to_string();
        let port_str = port.to_string();
        
        let output = run_shell_command_async(host, &port_str, command, Some(&device_id_str)).await?;
        
        // Print output
        if !output.is_empty() {
            print!("{}", output);
            if !output.ends_with('\n') {
                println!();
            }
        }
        
        Ok(())
    }
}