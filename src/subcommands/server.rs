use std::error::Error;
use std::process::Command;
use log::*;

use crate::library::adb::start_adb_server;

pub async fn run(operation: &crate::cli::ServerOperation) -> Result<(), Box<dyn Error>> {
    match operation {
        crate::cli::ServerOperation::Start => {
            debug!("Starting ADB server...");
            start_adb_server()?;
            println!("ADB server started successfully");
        }
        crate::cli::ServerOperation::Stop => {
            debug!("Stopping ADB server...");
            let output = Command::new("adb")
                .arg("kill-server")
                .output()?;

            if output.status.success() {
                println!("ADB server stopped successfully");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to stop ADB server: {}", error).into());
            }
        }
    }
    Ok(())
} 
