use crate::OutputType;

use super::lib;
use comfy_table::Table;
use log::debug;
use regex::Regex;
use serde_json::{json, Value};

pub async fn run(host: &str, port: &str, long: bool, output_type: OutputType) {
    let headers_to_display;
    let messages: Vec<&str>;

    if long {
        messages = vec!["host:devices-l"];
        headers_to_display = vec![
            "device_id".to_string(),
            "type".to_string(),
            "device".to_string(),
            "product".to_string(),
            "transport_id".to_string(),
        ];
    } else {
        messages = vec!["host:devices"];
        headers_to_display = vec!["device_id".to_string(), "type".to_string()];
    }
    let json = match lib::send_and_receive(&host, &port, messages) {
        Ok(responses) => {
            format(&responses)

            //match output_type {
            //    OutputType::Json => display_json(&json),
            //    OutputType::Table => display_table(&json, &headers_to_display),
            //}
        }
        Err(e) => format(&Vec::new()),
    };
    let mut device_ids: Vec<String> = Vec::new();

    if let Value::Array(arr) = json {
        for item in arr {
            if let Value::Object(obj) = item {
                if let Some(device_id_value) = obj.get("device_id") {
                    if let Value::String(device_id_str) = device_id_value {
                        device_ids.push(device_id_str.clone());
                    } else {
                        eprintln!(
                            "Warning: 'device_id' is not a string: {:?}",
                            device_id_value
                        );
                    }
                }
            }
        }
    } else {
        eprintln!("Warning: JSON is not an array.");
    }

    println!("device_ids = {:?}", device_ids);

    let propnames = vec!["ro.boot.qemu.avd_name".to_string()];
    let props = lib::get_props_parallel(host, port, &propnames).await;
    println!("props = {:?}", props);
}

fn format(responses: &[String]) -> Value {
    extract_device_info(responses.join("\n"))
}

fn display_json(json: &Value) {
    println!("{}", serde_json::to_string_pretty(json).unwrap())
}

fn display_table(json: &Value, headers_to_display: &Vec<String>) {
    let mut table = Table::new();
    table.set_header(headers_to_display.iter().map(|s| s.to_uppercase()));

    table.load_preset(comfy_table::presets::NOTHING);

    if let Value::Array(arr) = json {
        for item in arr {
            if let Value::Object(obj) = item {
                let mut values: Vec<&str> = Vec::new();

                for header in headers_to_display {
                    let value = obj.get(header).and_then(Value::as_str).unwrap();
                    values.push(value)
                }

                table.add_row(comfy_table::Row::from(values));
            }
        }
    }

    println!("{table}");
}

fn extract_device_info(input: String) -> Value {
    // 00d14B141FDCH0001U         device
    let re_short = Regex::new(r"^(\S+)\s+(\S+)").unwrap();

    // 00d14B141FDCH0001U         device usb:1-9 product:blazer model:Blazer device:blazer transport_id:1
    let re_full = Regex::new(r"^(\S+)\s+(\S+)\s+usb:(\S+)\s+product:(\S+)\s+model:(\S+)\s+device:(\S+)\s+transport_id:(\S+)").unwrap();

    // emulator-5554          device product:sdk_gphone64_x86_64 model:sdk_gphone64_x86_64 device:emu64xa transport_id:3
    let re_truncated = Regex::new(
        r"^(\S+)\s+(\S+)\s+product:(\S+)\s+model:(\S+)\s+device:(\S+)\s+transport_id:(\S+)",
    )
    .unwrap();

    let mut devices: Vec<Value> = Vec::new();

    for line in input.lines() {
        let (
            mut device_id,
            mut type_str,
            mut usb,
            mut product,
            mut model,
            mut device,
            mut transport_id,
        ): (&str, &str, &str, &str, &str, &str, &str) = ("", "", "", "", "", "", "");
        if let Some(captures) = re_full.captures(line) {
            device_id = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            type_str = captures.get(2).map(|m| m.as_str()).unwrap_or_default();
            usb = captures.get(3).map(|m| m.as_str()).unwrap_or_default();
            product = captures.get(4).map(|m| m.as_str()).unwrap_or_default();
            model = captures.get(5).map(|m| m.as_str()).unwrap_or_default();
            device = captures.get(6).map(|m| m.as_str()).unwrap_or_default();
            transport_id = captures.get(7).map(|m| m.as_str()).unwrap_or_default();
        } else if let Some(captures) = re_truncated.captures(line) {
            device_id = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            type_str = captures.get(2).map(|m| m.as_str()).unwrap_or_default();
            product = captures.get(3).map(|m| m.as_str()).unwrap_or_default();
            model = captures.get(4).map(|m| m.as_str()).unwrap_or_default();
            device = captures.get(5).map(|m| m.as_str()).unwrap_or_default();
            transport_id = captures.get(6).map(|m| m.as_str()).unwrap_or_default();
        }
        // This needs to come last, because this will always match
        else if let Some(captures) = re_short.captures(line) {
            device_id = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            type_str = captures.get(2).map(|m| m.as_str()).unwrap_or_default();
        }

        let device_json = json!({
            "device_id": device_id,
            "type": type_str,
            "usb": usb,
            "product": product,
            "model": model,
            "device": device,
            "transport_id": transport_id,
        });

        devices.push(device_json);
    }

    debug!("{:?}", devices);
    json!(devices)
}
