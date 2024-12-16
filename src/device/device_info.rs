use std::collections::HashMap;
use std::sync::LazyLock;

use log::debug;
use regex::Regex;
use serde_json::{json, Value};

use crate::config::Config;
use crate::error::AdbError;
use crate::library::adb;
use crate::library::hash::{petname, sha256, sha256_short};
use crate::types::DeviceDetails;

static RE_SHORT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\S+)\s+(\S+)").unwrap());
static RE_FULL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\S+)\s+(\S+)\s+usb:(\S+)\s+product:(\S+)\s+model:(\S+)\s+device:(\S+)\s+transport_id:(\S+)").unwrap()
});
static RE_TRUNCATED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\S+)\s+(\S+)\s+product:(\S+)\s+model:(\S+)\s+device:(\S+)\s+transport_id:(\S+)")
        .unwrap()
});

pub async fn get_devices(host: &str, port: &str) -> Vec<DeviceDetails> {
    let config = Config::load();
    let messages = vec!["host:devices-l"];
    let device_info = match adb::send_and_receive(host, port, messages) {
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

fn extract_device_info(input: String) -> Value {
    let mut devices: Vec<Value> = Vec::new();

    for line in input.lines() {
        let device_info = match (
            RE_FULL.captures(line),
            RE_TRUNCATED.captures(line),
            RE_SHORT.captures(line),
        ) {
            (Some(captures), _, _) => DeviceCapture {
                adb_id: captures.get(1).map_or("", |m| m.as_str()),
                type_str: captures.get(2).map_or("", |m| m.as_str()),
                usb: captures.get(3).map_or("", |m| m.as_str()),
                product: captures.get(4).map_or("", |m| m.as_str()),
                model: captures.get(5).map_or("", |m| m.as_str()),
                device: captures.get(6).map_or("", |m| m.as_str()),
                transport_id: captures.get(7).map_or("", |m| m.as_str()),
            },
            (_, Some(captures), _) => DeviceCapture {
                adb_id: captures.get(1).map_or("", |m| m.as_str()),
                type_str: captures.get(2).map_or("", |m| m.as_str()),
                product: captures.get(3).map_or("", |m| m.as_str()),
                model: captures.get(4).map_or("", |m| m.as_str()),
                device: captures.get(5).map_or("", |m| m.as_str()),
                transport_id: captures.get(6).map_or("", |m| m.as_str()),
                ..Default::default()
            },
            (_, _, Some(captures)) => DeviceCapture {
                adb_id: captures.get(1).map_or("", |m| m.as_str()),
                type_str: captures.get(2).map_or("", |m| m.as_str()),
                ..Default::default()
            },
            _ => DeviceCapture::default(),
        };

        let device_json = json!({
            "adb_id": device_info.adb_id,
            "type": device_info.type_str,
            "usb": device_info.usb,
            "product": device_info.product,
            "model": device_info.model,
            "device": device_info.device,
            "transport_id": device_info.transport_id,
        });

        devices.push(device_json);
    }

    debug!("{:?}", devices);
    json!(devices)
}
