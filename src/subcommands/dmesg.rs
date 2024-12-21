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
    let mut command_vec = vec!["shell".to_string()];
    command_vec.extend(command.split_whitespace().map(String::from));
    
    let stream = adb::run_command_stream(host, port, adb_id, &command_vec).await?;
    let mut reader = BufReader::new(stream).lines();

    while let Some(line) = reader.next_line().await? {
        println!("{}", line);
    }
    
    Ok(())
}
