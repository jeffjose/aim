use crate::subcommands::adb;
use crate::types::DeviceDetails;
use std::error::Error;

pub struct DmesgArgs {
    pub device_id: Option<String>,
    pub args: Vec<String>,
}

pub async fn run(
    args: DmesgArgs,
    target_device: &DeviceDetails,
    _host: &str,
    _port: &str,
) -> Result<(), Box<dyn Error>> {
    let mut command = "shell dmesg".to_string();
    if !args.args.is_empty() {
        command = format!("{} {}", command, args.args.join(" "));
    }

    adb::run(adb::AdbArgs {
        command: &command,
        adb_id: &target_device.adb_id,
    }).await
}
