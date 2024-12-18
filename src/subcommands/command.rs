use crate::{library::adb, types::DeviceDetails};
use log::debug;
use std::error::Error;

pub async fn run(
    host: &str,
    port: &str,
    command: &str,
    device: Option<&DeviceDetails>,
    filters: Option<&[String]>,
) -> Result<(), Box<dyn Error>> {
    match (device, filters) {
        (Some(d), None) => {
            // Single device specified, no filter
            run_on_device(host, port, command, d).await
        }
        (Some(d), Some(f)) if !f.is_empty() => {
            // Single device specified with filters - verify device matches filters
            let host_command = format!("host:tport:serial:{}", d.adb_id);
            let getprop_command = "shell:getprop";
            let messages = vec![host_command.as_str(), getprop_command];

            if let Ok(output) = adb::send(host, port, messages) {
                let props_output = output.into_iter().next().unwrap_or_default();
                let device_props: Vec<(&str, &str)> = props_output
                    .lines()
                    .filter_map(parse_getprop_line)
                    .collect();

                let parsed_filters: Vec<(&str, &str)> = f
                    .iter()
                    .map(|f| parse_filter(f))
                    .collect::<Result<Vec<_>, _>>()?;

                if parsed_filters.iter().all(|(filter_key, filter_value)| {
                    device_props.iter().any(|(prop_key, prop_value)| {
                        prop_key == filter_key && prop_value.trim() == filter_value.trim()
                    })
                }) {
                    run_on_device(host, port, command, d).await
                } else {
                    Err("Specified device does not match all filters".into())
                }
            } else {
                Err("Failed to get device properties".into())
            }
        }
        (None, Some(filters)) if !filters.is_empty() => {
            // No device specified but filters present - run on all matching devices
            run_on_filtered_devices(host, port, command, filters).await
        }
        (_, _) => {
            // No device or empty filters specified - check number of devices
            let devices = crate::device::device_info::get_devices(host, port).await;
            match devices.len() {
                0 => Err("No devices found".into()),
                1 => {
                    debug!("Running command on single available device");
                    run_on_device(host, port, command, &devices[0]).await
                }
                _ => {
                    Err("Multiple devices found. Please specify a device ID or use --filter".into())
                }
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
    filters: &[String],
) -> Result<(), Box<dyn Error>> {
    // Parse all filters
    let parsed_filters: Vec<(&str, &str)> = filters
        .iter()
        .map(|f| parse_filter(f))
        .collect::<Result<Vec<_>, _>>()?;

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

            // Parse all properties once
            let device_props: Vec<(&str, &str)> = props_output
                .lines()
                .filter_map(parse_getprop_line)
                .collect();

            // Check if all filters match
            let all_filters_match = parsed_filters.iter().all(|(filter_key, filter_value)| {
                device_props.iter().any(|(prop_key, prop_value)| {
                    prop_key == filter_key && prop_value.trim() == filter_value.trim()
                })
            });

            if all_filters_match {
                matched = true;
                println!(
                    "Running command on device {} ({}):",
                    device.adb_id, device.device_name
                );
                if let Err(e) = run_on_device(host, port, command, &device).await {
                    eprintln!("Error on device {}: {}", device.adb_id, e);
                }
            }
        }
    }

    if !matched {
        println!("No devices matched all filters:");
        for (key, value) in parsed_filters {
            println!("  {}={}", key, value);
        }
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
