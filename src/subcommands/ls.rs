use crate::library::hash::{petname, sha256, sha256_short};
use crate::cli::OutputType;

use crate::library::adb;

use comfy_table::Table;
use log::debug;
use regex::Regex;
use serde_json::{json, Map, Result, Value};
use std::{collections::HashMap, sync::LazyLock};

#[derive(Debug)]
struct TableDetails {
    display_name: String,
}

static HEADERS: LazyLock<HashMap<String, TableDetails>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "adb_id".to_string(),
        TableDetails {
            display_name: "ADB ID".to_string(),
        },
    );
    m.insert(
        "type".to_string(),
        TableDetails {
            display_name: "TYPE".to_string(),
        },
    );
    m.insert(
        "device".to_string(),
        TableDetails {
            display_name: "DEVICE".to_string(),
        },
    );
    m.insert(
        "product".to_string(),
        TableDetails {
            display_name: "PRODUCT".to_string(),
        },
    );
    m.insert(
        "transport_id".to_string(),
        TableDetails {
            display_name: "TRANSPORT ID".to_string(),
        },
    );
    m.insert(
        "ro.product.product.brand".to_string(),
        TableDetails {
            display_name: "BRAND".to_string(),
        },
    );
    m.insert(
        "ro.product.model".to_string(),
        TableDetails {
            display_name: "MODEL".to_string(),
        },
    );
    m.insert(
        "device_id".to_string(),
        TableDetails {
            display_name: "DEVICE ID".to_string(),
        },
    );

    m.insert(
        "device_id_short".to_string(),
        TableDetails {
            display_name: "DEVICE ID".to_string(),
        },
    );

    m.insert(
        "device_name".to_string(),
        TableDetails {
            display_name: "NAME".to_string(),
        },
    );

    m
});

pub async fn run(host: &str, port: &str, output_type: OutputType) {
    let messages = vec!["host:devices-l"];

    let headers_to_display = vec![
        "device_id_short".to_string(),
        "ro.product.product.brand".to_string(),
        "ro.product.model".to_string(),
        "adb_id".to_string(),
        "device_name".to_string(),
    ];
    let device_info = match adb::send_and_receive(&host, &port, messages) {
        Ok(responses) => {
            format(&responses)

            //match output_type {
            //    OutputType::Json => display_json(&json),
            //    OutputType::Table => display_table(&json, &headers_to_display),
            //}
        }
        Err(_e) => format(&Vec::new()),
    };
    let mut adb_ids: Vec<String> = Vec::new();

    if let Value::Array(arr) = &device_info {
        for item in arr {
            if let Value::Object(obj) = item {
                if let Some(adb_id_value) = obj.get("adb_id") {
                    if let Value::String(adb_id_str) = adb_id_value {
                        adb_ids.push(adb_id_str.clone());
                    } else {
                        eprintln!("Warning: 'adb_id' is not a string: {:?}", adb_id_value);
                    }
                }
            }
        }
    } else {
        eprintln!("Warning: JSON is not an array.");
    }

    debug!("adb_ids = {:?}", adb_ids);

    let propnames = vec![
        "ro.product.product.brand".to_string(),
        "ro.product.model".to_string(),
        "ro.boot.qemu.avd_name".to_string(),
    ];

    let mut all_props = HashMap::new();

    for adb_id in adb_ids {
        let mut props =
            adb::get_props_parallel(host, port, &propnames, Some(adb_id.as_str())).await;

        let device_id_input_string = match props.get("ro.boot.qemu.avd_name") {
            Some(value) if value == "" => &adb_id,
            Some(value) => value,
            None => &adb_id,
        };

        let device_ids = vec![
            ("device_id".to_string(), sha256(&device_id_input_string)),
            (
                "device_id_short".to_string(),
                sha256_short(&device_id_input_string).to_string(),
            ),
            ("device_name".to_string(), petname(&device_id_input_string)),
        ];

        props.extend(device_ids);

        all_props.insert(adb_id, props);
    }

    debug!("all_props = {:?}", all_props);

    let merged = merge_json_with_hashmap(&device_info, &all_props);

    match merged {
        Ok(merged) => match output_type {
            OutputType::Json => display_json(&merged),
            OutputType::Table => display_table(&merged, &headers_to_display),
        },
        Err(_e) => eprintln!("error"),
    }
}

fn format(responses: &[String]) -> Value {
    extract_device_info(responses.join("\n"))
}

fn display_json(json: &Value) {
    println!("{}", serde_json::to_string_pretty(json).unwrap());
}

#[allow(dead_code)]
fn display_table(json: &Value, headers_to_display: &Vec<String>) {
    let mut table = Table::new();

    let headers: Vec<String> = headers_to_display
        .iter()
        .filter_map(|key| HEADERS.get(key).map(|details| details.display_name.clone()))
        .collect();

    table.set_header(headers);

    table.load_preset(comfy_table::presets::NOTHING);

    if let Value::Array(arr) = json {
        for item in arr {
            if let Value::Object(obj) = item {
                let mut values: Vec<&str> = Vec::new();

                for header in headers_to_display {
                    let value = obj.get(header).and_then(Value::as_str).unwrap_or_default();
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
            mut adb_id,
            mut type_str,
            mut usb,
            mut product,
            mut model,
            mut device,
            mut transport_id,
        ): (&str, &str, &str, &str, &str, &str, &str) = ("", "", "", "", "", "", "");
        if let Some(captures) = re_full.captures(line) {
            adb_id = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            type_str = captures.get(2).map(|m| m.as_str()).unwrap_or_default();
            usb = captures.get(3).map(|m| m.as_str()).unwrap_or_default();
            product = captures.get(4).map(|m| m.as_str()).unwrap_or_default();
            model = captures.get(5).map(|m| m.as_str()).unwrap_or_default();
            device = captures.get(6).map(|m| m.as_str()).unwrap_or_default();
            transport_id = captures.get(7).map(|m| m.as_str()).unwrap_or_default();
        } else if let Some(captures) = re_truncated.captures(line) {
            adb_id = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            type_str = captures.get(2).map(|m| m.as_str()).unwrap_or_default();
            product = captures.get(3).map(|m| m.as_str()).unwrap_or_default();
            model = captures.get(4).map(|m| m.as_str()).unwrap_or_default();
            device = captures.get(5).map(|m| m.as_str()).unwrap_or_default();
            transport_id = captures.get(6).map(|m| m.as_str()).unwrap_or_default();
        }
        // This needs to come last, because this will always match
        else if let Some(captures) = re_short.captures(line) {
            adb_id = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            type_str = captures.get(2).map(|m| m.as_str()).unwrap_or_default();
        }

        let device_json = json!({
            "adb_id": adb_id,
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

fn merge_json_with_hashmap(
    list: &Value,
    map: &HashMap<String, HashMap<String, String>>,
) -> Result<Value> {
    let mut merged_list = Vec::new();

    if let Value::Array(list_arr) = list {
        for list_item in list_arr {
            if let Value::Object(list_obj) = list_item {
                if let Some(adb_id_value) = list_obj.get("adb_id") {
                    if let Value::String(adb_id) = adb_id_value {
                        if let Some(map_props) = map.get(adb_id) {
                            let mut merged_obj = Map::new();

                            for (k, v) in list_obj.iter() {
                                merged_obj.insert(k.clone(), v.clone());
                            }

                            for (k, v) in map_props.iter() {
                                merged_obj.insert(k.clone(), Value::String(v.clone()));
                                // Convert String to Value::String
                            }

                            merged_list.push(Value::Object(merged_obj));
                        } else {
                            // If adb_id is not found in the map, keep the original list item.
                            merged_list.push(list_item.clone());
                            eprintln!("Warning: adb_id {} not found in map", adb_id);
                        }
                    }
                } else {
                    merged_list.push(list_item.clone());
                    eprintln!("Warning: list item does not contain adb_id");
                }
            }
        }
    }
    Ok(Value::Array(merged_list))
}
