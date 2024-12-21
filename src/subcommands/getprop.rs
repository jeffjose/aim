use crate::{cli::OutputType, library::adb, types::DeviceDetails};
use colored_json::ToColoredJson;
use comfy_table::Table;
use log::debug;
use serde_json::json;
use std::collections::HashMap;

pub async fn run(
    host: &str,
    port: &str,
    propnames: &[String],
    device: Option<&DeviceDetails>,
    output: OutputType,
) -> Result<(), Box<dyn std::error::Error>> {
    let adb_id = device.map(|d| d.adb_id.as_str());

    let results = if propnames.is_empty() {
        // If no properties specified, get all properties
        debug!("No properties specified, getting all properties");
        let host_command = match adb_id {
            Some(id) => format!("host:tport:serial:{}", id),
            None => "host:tport:any".to_string(),
        };

        let command = "shell:getprop";
        let messages = vec![host_command.as_str(), command];

        let output = adb::send(host, port, messages, false)?
            .into_iter()
            .next()
            .unwrap_or_default();

        // Parse the output into a HashMap
        let mut props = HashMap::new();
        for line in output.lines() {
            if let Some((key, value)) = parse_getprop_line(line) {
                props.insert(key.to_string(), value.to_string());
            }
        }
        props
    } else {
        adb::getprops_parallel(host, port, propnames, adb_id).await
    };

    match output {
        OutputType::Plain => {
            // For single property, just print value
            if propnames.len() == 1 {
                if let Some(value) = results.get(&propnames[0]) {
                    println!("{}", value.trim());
                }
            } else {
                // For multiple or all properties, print property=value format
                let mut sorted_props: Vec<_> = results.iter().collect();
                sorted_props.sort_by(|a, b| a.0.cmp(b.0));

                for (propname, value) in sorted_props {
                    println!("{}={}", propname, value.trim());
                }
            }
        }
        OutputType::Json => {
            let json_str = serde_json::to_string_pretty(&results)?;
            println!("{}", json_str.to_colored_json_auto()?);
        }
        OutputType::Table => {
            let mut table = Table::new();
            table.set_header(vec!["Property", "Value"]);
            table.load_preset(comfy_table::presets::NOTHING);

            let mut sorted_props: Vec<_> = results.iter().collect();
            sorted_props.sort_by(|a, b| a.0.cmp(b.0));

            for (propname, value) in sorted_props {
                table.add_row(vec![propname, value.trim()]);
            }

            println!("{table}");
        }
    }

    Ok(())
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
