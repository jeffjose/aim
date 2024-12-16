use crate::library::adb;
use crate::types::DeviceDetails;

pub async fn run(
    host: &str,
    port: &str,
    command: &str,
    target_device: Option<&DeviceDetails>,
) -> Result<(), Box<dyn std::error::Error>> {
    let adb_id = target_device.map(|d| d.adb_id.as_str());

    let result = adb::run_command_async(host, port, command, adb_id).await?;
    println!("{}", result);

    Ok(())
}
