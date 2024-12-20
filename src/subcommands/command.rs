use crate::library::adb;
use crate::types::DeviceDetails;
use crossterm::{
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use indicatif::ProgressBar;
use std::error::Error;
use std::io::stdout;
use std::time::Duration;
use tokio::time::sleep;

pub struct CommandArgs<'a> {
    pub command: &'a str,
    pub device: Option<&'a DeviceDetails>,
    pub filters: Option<&'a [String]>,
    pub watch: bool,
    pub watch_time: Option<u32>,
}

pub async fn run(
    host: &str,
    port: &str,
    command: &str,
    device: Option<&DeviceDetails>,
    filters: Option<&[String]>,
    watch: bool,
    watch_time: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    // Check device requirements
    let target_device = match device {
        Some(d) => d,
        None => {
            // If no device specified, check number of devices
            let devices = crate::device::device_info::get_devices(host, port).await;
            match devices.len() {
                0 => return Err("No devices found".into()),
                1 => &devices.into_iter().next().unwrap(),
                _ => return Err("Multiple devices found. Please specify a device ID".into()),
            }
        }
    };

    if !watch {
        // Regular single execution
        return execute_command(host, port, command, Some(target_device), filters).await;
    }

    // Watch mode
    let mut iteration = 1;
    println!("Press Ctrl+C to stop\n");

    loop {
        // Execute command
        execute_command(host, port, command, Some(target_device), filters).await?;

        // Check if we should continue
        if let Some(time) = watch_time {
            if iteration >= time {
                break;
            }
        }

        iteration += 1;
        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn execute_command(
    host: &str,
    port: &str,
    command: &str,
    device: Option<&DeviceDetails>,
    filters: Option<&[String]>,
) -> Result<(), Box<dyn Error>> {
    let adb_id = device.map(|d| d.adb_id.as_str());
    let response = adb::run_shell_command_async(host, port, command, adb_id).await?;
    println!("{}", response);
    Ok(())
}
