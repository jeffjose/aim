use crate::{library::adb, types::DeviceDetails};
use log::debug;
use std::error::Error;

pub async fn run(
    host: &str,
    port: &str,
    command: &str,
    device: Option<&DeviceDetails>,
    filter: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    match (device, filter) {
        (Some(d), None) => {
            // Single device specified, no filter
            run_on_device(host, port, command, d).await
        }
        (None, Some(filter)) => {
            // No device specified but filter present - run on all matching devices
            run_on_filtered_devices(host, port, command, filter).await
        }
        (Some(_), Some(_)) => {
            Err("Cannot specify both device ID and filter".into())
        }
        (None, None) => {
            // No device or filter specified - check number of devices
            let devices = crate::device::device_info::get_devices(host, port).await;
            match devices.len() {
                0 => Err("No devices found".into()),
                1 => {
                    debug!("Running command on single available device");
                    run_on_device(host, port, command, &devices[0]).await
                }
                _ => Err("Multiple devices found. Please specify a device ID or use --filter".into())
            }
        }
    }
}

async fn run_on_device(
    host: &str,
    port: &str,
    command: &str,
    device: &DeviceDetails,
) -> Result<(), Box<dyn Error>> {
    let output = adb::run_shell_command_async(host, port, command, Some(&device.adb_id)).await?;
    println!("{}", output);
    Ok(())
}

async fn run_on_filtered_devices(
    host: &str,
    port: &str,
    command: &str,
    filter: &str,
) -> Result<(), Box<dyn Error>> {
    // Parse filter in format key=value
    let (key, value) = parse_filter(filter)?;
    
    // Get all devices
    let devices = crate::device::device_info::get_devices(host, port).await;
    let mut matched = false;

    // Check each device's properties
    for device in devices {
        // Get all properties for the device
        let host_command = format!("host:tport:serial:{}", device.adb_id);
        let getprop_command = "shell:getprop";
        let messages = vec![host_command.as_str(), getprop_command];
        
        if let Ok(output) = adb::send(host, port, messages) {
            let props_output = output.into_iter().next().unwrap_or_default();
            
            // Parse properties and check for match
            for line in props_output.lines() {
                if let Some((prop_key, prop_value)) = parse_getprop_line(line) {
                    if prop_key == key && prop_value.trim() == value.trim() {
                        matched = true;
                        println!("Running command on device {} ({}):", device.adb_id, device.device_name);
                        if let Err(e) = run_on_device(host, port, command, &device).await {
                            eprintln!("Error on device {}: {}", device.adb_id, e);
                        }
                    }
                }
            }
        }
    }

    if !matched {
        println!("No devices matched the filter {}={}", key, value);
    }

    Ok(())
}

fn parse_filter(filter: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let parts: Vec<&str> = filter.split('=').collect();
    if parts.len() != 2 {
        return Err("Filter must be in format key=value".into());
    }
    Ok((parts[0], parts[1]))
}

fn parse_getprop_line(line: &str) -> Option<(&str, &str)> {
    // Format is typically: [prop.name]: [value]
    let line = line.trim();
    if line.starts_with('[') {
        let parts: Vec<&str> = line.split("]: [").collect();
        if parts.len() == 2 {
            let key = parts[0].trim_start_matches('[');
            let value = parts[1].trim_end_matches(']');
            Some((key, value))
        } else {
            None
        }
    } else {
        None
    }
}
