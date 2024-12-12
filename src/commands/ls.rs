use crate::OutputType;

use super::_common;
use comfy_table::Table;
use regex::Regex;
use serde_json::{json, Value};

pub fn run(host: &str, port: &str, long: bool, output_type: OutputType) {
    let message = if long {
        "000ehost:devices-l"
    } else {
        "000chost:devices"
    };
    match _common::send_and_receive(&host, &port, message) {
        Ok(responses) => {
            let json = format(&responses);

            match output_type {
                OutputType::Json => display_json(&json),
                OutputType::Table => display_table(&json),
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e)
        }
    }
}

fn format(responses: &[String]) -> Value {
    extract_device_info(responses.join("\n"))
}

fn display_json(json: &Value) {
    println!("{}", serde_json::to_string_pretty(json).unwrap())
}

fn display_table(json: &Value) {
    let mut table = Table::new();
    table.set_header(vec!["serial", "type"]);

    if let Value::Array(arr) = json {
        for item in arr {
            if let Value::Object(obj) = item {
                let serial = obj.get("serial").and_then(Value::as_str).unwrap();
                let r#type = obj.get("type").and_then(Value::as_str).unwrap();

                table.add_row(vec![serial, r#type]);
            }
        }
    }

    println!("{table}");
}

fn extract_device_info(input: String) -> Value {
    let re = Regex::new(r"^(\S+)\s+(\S+)").unwrap();
    let mut devices: Vec<Value> = Vec::new();

    for line in input.lines() {
        if let Some(captures) = re.captures(line) {
            let serial = captures.get(1).map(|m| m.as_str()).unwrap_or_default();
            let type_str = captures.get(2).map(|m| m.as_str()).unwrap_or_default();

            let device_json = json!({
                "serial": serial,
                "type": type_str,
            });
            devices.push(device_json);
        }
    }

    json!(devices)
}
