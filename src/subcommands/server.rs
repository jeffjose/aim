use std::error::Error;
use log::*;

use crate::library::adb::{start_adb_server, send};

pub async fn run(host: &str, port: &str, operation: &crate::cli::ServerOperation) -> Result<(), Box<dyn Error>> {
    match operation {
        crate::cli::ServerOperation::Start => {
            debug!("Starting ADB server...");
            start_adb_server()?;
            println!("ADB server started successfully");
        }
        crate::cli::ServerOperation::Stop => {
            debug!("Stopping ADB server...");
            
            // Send kill command using the existing ADB protocol implementation
            match send(host, port, vec!["host:kill"]) {
                Ok(_) => println!("ADB server stopped successfully"),
                Err(e) => {
                    if e.to_string().contains("Connection refused") {
                        println!("ADB server is not running");
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }
    Ok(())
} 
