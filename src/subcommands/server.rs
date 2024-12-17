use std::error::Error;
use log::*;
use tokio::time::sleep;
use std::time::Duration;

use crate::library::adb::{start_adb_server, kill_server};

pub async fn run(host: &str, port: &str, operation: &crate::cli::ServerOperation) -> Result<(), Box<dyn Error>> {
    match operation {
        crate::cli::ServerOperation::Start => {
            debug!("Starting ADB server...");
            start_adb_server()?;
            println!("ADB server started successfully");
        }
        crate::cli::ServerOperation::Stop => {
            debug!("Stopping ADB server...");
            kill_server(host, port)?;
            println!("ADB server stopped successfully");
        }
        crate::cli::ServerOperation::Restart => {
            debug!("Restarting ADB server...");
            
            // Stop server
            kill_server(host, port)?;
            println!("ADB server stopped successfully");

            // Wait for 1 second
            debug!("Waiting for 1 second before restart...");
            sleep(Duration::from_secs(1)).await;

            // Start server
            debug!("Starting ADB server...");
            start_adb_server()?;
            println!("ADB server restarted successfully");
        }
    }
    Ok(())
} 
