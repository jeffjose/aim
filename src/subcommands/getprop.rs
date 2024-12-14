use crate::library::adb;
use crate::types::DeviceDetails;

pub fn run(host: &str, port: &str, propname: &str, target_device: Option<&DeviceDetails>) -> Result<(), Box<dyn std::error::Error>> {
    let adb_id = target_device.map(|d| d.adb_id.as_str());
    let result = adb::run_command(host, port, &format!("getprop {}", propname), adb_id);
    
    if let Some(device) = target_device {
        println!("Device: {} ({})", device.device_name, device.device_id_short);
    }
    println!("{}", result);

    Ok(())
}
