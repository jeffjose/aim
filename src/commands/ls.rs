use super::_common;
use json_to_table::json_to_table;
use regex::Regex;
use serde_json::{json, Value};

pub fn run(host: &str, port: &str, long: bool) {
    let message = if long {
        "000ehost:devices-l"
    } else {
        "000chost:devices"
    };
    match _common::send_and_receive(&host, &port, message) {
        Ok(responses) => {
            let json = format(&responses);

            display(&json);
            println!("{}", json_to_table(&json).to_string())
        }
        Err(e) => {
            eprintln!("Error: {}", e)
        }
    }
}

fn format(responses: &[String]) -> Value {
    extract_device_info(responses.join("\n"))
}

fn display(json: &Value) {
    println!("{}", serde_json::to_string_pretty(json).unwrap())
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
