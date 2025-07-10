use std::collections::HashMap;
use log::{debug, info};
use serde_json::{json, Value};

use crate::config::Config;
use crate::library::{adb, hash::{petname, sha256, sha256_short}};
use crate::{error::AdbError, types::DeviceDetails};

const DEVICE_PROPERTIES: [&str; 4] = [
    "ro.product.product.brand",
    "ro.product.model",
    "ro.boot.qemu.avd_name",
    "service.adb.root",
];

pub async fn get_devices(host: &str, port: &str) -> Vec<DeviceDetails> {
    debug!("get_devices called with host={}, port={}", host, port);
    let config = Config::load();
    debug!("Getting device list from ADB...");
    let device_info = get_device_list_from_adb(host, port);
    debug!("Device info: {:?}", device_info);
    let mut devices = Vec::new();

    if let Value::Array(arr) = device_info {
        for item in arr {
            if let Some(device) = process_device(host, port, item, &config).await {
                devices.push(device);
            }
        }
    }

    debug!("{:?}", devices);
    devices
}

fn get_device_list_from_adb(host: &str, port: &str) -> Value {
    debug!("get_device_list_from_adb: sending host:devices-l");
    let messages = vec!["host:devices-l"];
    match adb::send(host, port, messages, false) {
        Ok(responses) => {
            debug!("Got responses: {:?}", responses);
            format_device_list(&responses)
        }
        Err(e) => {
            debug!("Error getting device list: {}", e);
            Value::Array(vec![])
        }
    }
}

async fn process_device(host: &str, port: &str, item: Value, config: &Config) -> Option<DeviceDetails> {
    let mut device = DeviceDetails::from_json(&item)?;
    
    // Log device state if not normal
    if device.device_type != "device" {
        info!("Device {} is {}", device.adb_id, device.device_type);
    }
    
    let propnames: Vec<String> = DEVICE_PROPERTIES
        .iter()
        .map(|&s| s.to_string())
        .collect();

    let props = adb::getprops_parallel(host, port, &propnames, Some(&device.adb_id)).await;
    debug!("Props for device {}: {:?}", device.adb_id, props);
    let identifiers = create_device_identifiers(&props, &device.adb_id, config);
    
    let mut all_props = props;
    all_props.extend(identifiers);
    device.update_from_props(all_props);
    
    Some(device)
}

fn create_device_identifiers(props: &HashMap<String, String>, adb_id: &str, config: &Config) -> HashMap<String, String> {
    let device_id_input = props
        .get("ro.boot.qemu.avd_name")
        .filter(|v| !v.is_empty())
        .map(String::as_str)
        .unwrap_or(adb_id);

    let device_id = sha256(device_id_input);
    let mut identifiers = HashMap::new();
    
    identifiers.insert("device_id".to_string(), device_id.clone());
    identifiers.insert("device_id_short".to_string(), sha256_short(device_id_input).to_string());
    identifiers.insert(
        "device_name".to_string(),
        config.get_device_name(&device_id).unwrap_or_else(|| petname(device_id_input))
    );
    
    identifiers
}

fn format_device_list(responses: &[String]) -> Value {
    extract_device_info(responses.join("\n"))
}

pub fn extract_device_info(input: String) -> Value {
    let devices = input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_device_line)
        .collect();

    Value::Array(devices)
}

fn parse_device_line(line: &str) -> Option<Value> {
    let (adb_id, remainder) = line.split_once(char::is_whitespace)?;
    let remainder = remainder.trim();
    let (device_type, properties) = remainder
        .split_once(char::is_whitespace)
        .unwrap_or((remainder, ""));

    let mut device = create_base_device(adb_id, device_type);
    parse_device_properties(&mut device, properties);
    Some(device)
}

fn create_base_device(adb_id: &str, device_type: &str) -> Value {
    json!({
        "adb_id": adb_id,
        "type": device_type,
        "usb": "",
        "product": "",
        "model": "",
        "device": "",
        "transport_id": "",
    })
}

fn parse_device_properties(device: &mut Value, properties: &str) {
    if properties.is_empty() {
        return;
    }

    for prop in properties.split_whitespace() {
        if let Some((key, value)) = prop.split_once(':') {
            if let Some(field) = device.get_mut(key) {
                *field = Value::String(value.to_string());
            }
        }
    }
}

pub fn find_target_device<'a>(
    devices: &'a [DeviceDetails],
    device_id: Option<&String>,
) -> Result<&'a DeviceDetails, Box<dyn std::error::Error>> {
    match device_id {
        Some(id) => find_device_by_id(devices, id),
        None => find_single_device(devices),
    }
}

fn find_device_by_id<'a>(
    devices: &'a [DeviceDetails],
    id: &str,
) -> Result<&'a DeviceDetails, Box<dyn std::error::Error>> {
    let matching_devices: Vec<&DeviceDetails> = devices
        .iter()
        .filter(|d| d.matches_id_prefix(id))
        .collect();

    match matching_devices.len() {
        0 => Err(AdbError::DeviceNotFound(id.to_string()).into()),
        1 => Ok(matching_devices[0]),
        _ => Err(AdbError::AmbiguousDeviceMatch {
            prefix: id.to_string(),
            matches: matching_devices.iter().map(|d| d.adb_id.clone()).collect(),
        }.into()),
    }
}

fn find_single_device<'a>(
    devices: &'a [DeviceDetails],
) -> Result<&'a DeviceDetails, Box<dyn std::error::Error>> {
    match devices.len() {
        1 => Ok(&devices[0]),
        _ => Err(AdbError::DeviceIdRequired.into()),
    }
}
