use crate::library::adb;
use crate::types::DeviceDetails;

pub async fn run(host: &str, port: &str, propnames: &[String], target_device: Option<&DeviceDetails>) -> Result<(), Box<dyn std::error::Error>> {
    let adb_id = target_device.map(|d| d.adb_id.as_str());
    let result = adb::get_props_parallel(host, port, propnames, adb_id).await;
    
    if let Some(device) = target_device {
        println!("Device: {} ({})", device.device_name, device.device_id_short);
    }
    println!("{:?}", result);

    Ok(())
}
