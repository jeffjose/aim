use crate::library::adb;
use crate::types::DeviceDetails;
use std::error::Error;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct DmesgArgs {
    pub device_id: Option<String>,
    pub args: Vec<String>,
}

pub async fn run(
    args: DmesgArgs,
    target_device: &DeviceDetails,
    host: &str,
    port: &str,
) -> Result<(), Box<dyn Error>> {
    let mut command = "dmesg".to_string();
    if !args.args.is_empty() {
        command = format!("{} {}", command, args.args.join(" "));
    }

    execute_command(host, port, &command, Some(target_device)).await
}

async fn execute_command(
    host: &str,
    port: &str,
    command: &str,
    device: Option<&DeviceDetails>,
) -> Result<(), Box<dyn Error>> {
    let adb_id = device.map(|d| d.adb_id.as_str());

    let response = adb::run_shell_command_async(host, port, command, adb_id).await?;

    println!("{}", response);
    Ok(())
}
