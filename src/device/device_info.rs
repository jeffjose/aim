use std::collections::HashMap;
use std::sync::LazyLock;

use log::debug;
use regex::Regex;
use serde_json::{json, Value};

use crate::config::Config;
use crate::library::adb;
use crate::library::hash::{petname, sha256, sha256_short};
use crate::{error::AdbError, types::DeviceDetails};

static RE_SHORT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\S+)\s+(\S+)").unwrap());
static RE_FULL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\S+)\s+(\S+)\s+usb:(\S+)\s+product:(\S+)\s+model:(\S+)\s+device:(\S+)\s+transport_id:(\S+)").unwrap()
});
static RE_TRUNCATED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\S+)\s+(\S+)\s+product:([^\s]*)\s+model:([^\s]*)\s+device:([^\s]*)\s+transport_id:(\S+)")
        .unwrap()
});

pub async fn get_devices(host: &str, port: &str) -> Vec<DeviceDetails> {
    let config = Config::load();
    let messages = vec!["host:devices-l"];
    let device_info = match adb::send(host, port, messages, false) {
        Ok(responses) => format_device_list(&responses),
        Err(_e) => format_device_list(&Vec::new()),
    };

    let mut devices: Vec<DeviceDetails> = Vec::new();

    if let Value::Array(arr) = device_info {
        for item in arr {
            if let Some(mut device) = DeviceDetails::from_json(&item) {
                let propnames = vec![
                    "ro.product.product.brand".to_string(),
                    "ro.product.model".to_string(),
                    "ro.boot.qemu.avd_name".to_string(),
                    "service.adb.root".to_string(),
                ];

                let props =
                    adb::getprops_parallel(host, port, &propnames, Some(&device.adb_id)).await;

                let device_id_input = props
                    .get("ro.boot.qemu.avd_name")
                    .filter(|v| !v.is_empty())
                    .unwrap_or(&device.adb_id);

                let mut identifiers = HashMap::new();
                identifiers.insert("device_id".to_string(), sha256(device_id_input));
                let device_id = sha256(device_id_input);
                identifiers.insert(
                    "device_id_short".to_string(),
                    sha256_short(device_id_input).to_string(),
                );

                let device_name = config
                    .get_device_name(&device_id)
                    .unwrap_or_else(|| petname(device_id_input));
                identifiers.insert("device_name".to_string(), device_name);

                let mut all_props = props;
                all_props.extend(identifiers);
                device.update_from_props(all_props);
                devices.push(device);
            }
        }
    }

    debug!("{:?}", devices);
    devices
}

fn format_device_list(responses: &[String]) -> Value {
    extract_device_info(responses.join("\n"))
}

#[derive(Default)]
struct DeviceCapture<'a> {
    adb_id: &'a str,
    type_str: &'a str,
    usb: &'a str,
    product: &'a str,
    model: &'a str,
    device: &'a str,
    transport_id: &'a str,
}

pub fn extract_device_info(input: String) -> Value {
    println!("Processing input: {:?}", input);
    println!(
        "Input length: {} characters, {} lines",
        input.len(),
        input.lines().count()
    );

    let mut devices = Vec::new();

    for line in input.lines() {
        println!("\nProcessing line: {:?}", line);
        println!("Line length: {} characters", line.len());

        // Skip empty lines
        if line.trim().is_empty() {
            println!("Skipping empty line");
            continue;
        }

        println!("Attempting regex matches...");
        let full_match = RE_FULL.captures(line);
        let truncated_match = RE_TRUNCATED.captures(line);
        let short_match = RE_SHORT.captures(line);

        println!(
            "Match results - Full: {}, Truncated: {}, Short: {}",
            full_match.is_some(),
            truncated_match.is_some(),
            short_match.is_some()
        );

        let device_info = match (full_match, truncated_match, short_match) {
            (Some(captures), _, _) => {
                println!("Using full match pattern");
                DeviceCapture {
                    adb_id: captures.get(1).map_or("", |m| m.as_str()),
                    type_str: captures.get(2).map_or("", |m| m.as_str()),
                    usb: captures.get(3).map_or("", |m| m.as_str()),
                    product: captures.get(4).map_or("", |m| m.as_str()),
                    model: captures.get(5).map_or("", |m| m.as_str()),
                    device: captures.get(6).map_or("", |m| m.as_str()),
                    transport_id: captures.get(7).map_or("", |m| m.as_str()),
                }
            }
            (_, Some(captures), _) => {
                println!("Using truncated match pattern");
                DeviceCapture {
                    adb_id: captures.get(1).map_or("", |m| m.as_str()),
                    type_str: captures.get(2).map_or("", |m| m.as_str()),
                    product: captures.get(3).map_or("", |m| m.as_str()),
                    model: captures.get(4).map_or("", |m| m.as_str()),
                    device: captures.get(5).map_or("", |m| m.as_str()),
                    transport_id: captures.get(6).map_or("", |m| m.as_str()),
                    ..Default::default()
                }
            }
            (_, _, Some(captures)) => {
                println!("Using short match pattern");
                DeviceCapture {
                    adb_id: captures.get(1).map_or("", |m| m.as_str()),
                    type_str: captures.get(2).map_or("", |m| m.as_str()),
                    ..Default::default()
                }
            }
            _ => {
                println!("Failed to parse line, no regex patterns matched");
                println!("Line format did not match any expected patterns");
                continue;
            }
        };

        println!("Extracted fields:");
        println!("  adb_id: {:?}", device_info.adb_id);
        println!("  type: {:?}", device_info.type_str);
        println!("  usb: {:?}", device_info.usb);
        println!("  product: {:?}", device_info.product);
        println!("  model: {:?}", device_info.model);
        println!("  device: {:?}", device_info.device);
        println!("  transport_id: {:?}", device_info.transport_id);

        // Skip if we didn't get at least an adb_id and type
        if device_info.adb_id.is_empty() || device_info.type_str.is_empty() {
            println!("Skipping device with empty adb_id or type");
            println!(
                "adb_id empty: {}, type empty: {}",
                device_info.adb_id.is_empty(),
                device_info.type_str.is_empty()
            );
            continue;
        }

        let device_json = json!({
            "adb_id": device_info.adb_id,
            "type": device_info.type_str,
            "usb": device_info.usb,
            "product": device_info.product,
            "model": device_info.model,
            "device": device_info.device,
            "transport_id": device_info.transport_id,
        });

        println!("Created device JSON: {}", device_json);
        devices.push(device_json);
        println!("Current device count: {}", devices.len());
    }

    println!("\nFinal results:");
    println!("Total devices processed: {}", devices.len());
    println!("Final devices array: {:#}", Value::Array(devices.clone()));
    Value::Array(devices)
}

/// Find a target device from a list of devices based on an optional device ID.
/// If no device ID is provided and there is exactly one device, that device is returned.
/// If no device ID is provided and there are multiple devices, an error is returned.
/// If a device ID is provided, it will match against device IDs that start with the provided prefix.
pub fn find_target_device<'a>(
    devices: &'a [DeviceDetails],
    device_id: Option<&String>,
) -> Result<&'a DeviceDetails, Box<dyn std::error::Error>> {
    match device_id {
        Some(id) => {
            let matching_devices: Vec<&DeviceDetails> =
                devices.iter().filter(|d| d.matches_id_prefix(id)).collect();

            match matching_devices.len() {
                0 => Err(AdbError::DeviceNotFound(id.clone()).into()),
                1 => Ok(matching_devices[0]),
                _ => {
                    let matching_ids: Vec<String> =
                        matching_devices.iter().map(|d| d.adb_id.clone()).collect();
                    Err(AdbError::AmbiguousDeviceMatch {
                        prefix: id.clone(),
                        matches: matching_ids,
                    }
                    .into())
                }
            }
        }
        None => {
            if devices.len() == 1 {
                Ok(&devices[0])
            } else {
                Err(AdbError::DeviceIdRequired.into())
            }
        }
    }
}
