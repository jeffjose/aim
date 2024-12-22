use std::collections::HashMap;

use log::debug;
use serde_json::{json, Value};

use crate::config::Config;
use crate::library::adb;
use crate::library::hash::{petname, sha256, sha256_short};
use crate::{error::AdbError, types::DeviceDetails};

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

pub fn extract_device_info(input: String) -> Value {
    let devices = input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| parse_device_line(line))
        .collect();

    Value::Array(devices)
}

fn parse_device_line(line: &str) -> Option<Value> {
    let (adb_id, remainder) = line.split_once(char::is_whitespace)?;
    let remainder = remainder.trim();
    let (device_type, properties) = remainder.split_once(char::is_whitespace).unwrap_or((remainder, ""));

    let mut device = json!({
        "adb_id": adb_id,
        "type": device_type,
        "usb": "",
        "product": "",
        "model": "",
        "device": "",
        "transport_id": "",
    });

    if !properties.is_empty() {
        for prop in properties.split_whitespace() {
            if let Some((key, value)) = prop.split_once(':') {
                match key {
                    "usb" => device["usb"] = Value::String(value.to_string()),
                    "product" => device["product"] = Value::String(value.to_string()),
                    "model" => device["model"] = Value::String(value.to_string()),
                    "device" => device["device"] = Value::String(value.to_string()),
                    "transport_id" => device["transport_id"] = Value::String(value.to_string()),
                    _ => {} // Ignore unknown properties
                }
            }
        }
    }

    Some(device)
}

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
