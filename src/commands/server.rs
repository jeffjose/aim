use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::{start_adb_server, kill_server, check_server_status};
use crate::cli::ServerOperation;
use async_trait::async_trait;
use colored::*;

pub struct ServerCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct ServerArgs {
    /// Server operation to perform
    pub operation: ServerOperation,
}

impl ServerCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for ServerCommand {
    type Args = ServerArgs;
    
    async fn run(&self, _ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        let port_str = port.to_string();
        
        match args.operation {
            ServerOperation::Start => {
                println!("Starting ADB server...");
                start_adb_server(&port_str)?;
                println!("{} ADB server started", "✓".green());
            }
            ServerOperation::Stop => {
                println!("Stopping ADB server...");
                kill_server(host, &port_str)?;
                println!("{} ADB server stopped", "✓".green());
            }
            ServerOperation::Restart => {
                println!("Restarting ADB server...");
                // First stop if running
                if check_server_status(host, &port_str) {
                    kill_server(host, &port_str)?;
                    // Wait a bit for server to stop
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
                // Then start
                start_adb_server(&port_str)?;
                println!("{} ADB server restarted", "✓".green());
            }
            ServerOperation::Status => {
                if check_server_status(host, &port_str) {
                    println!("{} ADB server is running on {}:{}", 
                        "●".green(), 
                        host, 
                        port
                    );
                } else {
                    println!("{} ADB server is not running", "●".red());
                }
            }
        }
        
        Ok(())
    }
}