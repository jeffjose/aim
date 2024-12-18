use crate::{cli::OutputType, library::adb, types::DeviceDetails};
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
    let results = adb::getprops_parallel(host, port, propnames, adb_id).await;

    match output {
        OutputType::Plain => {
            // For single property, just print value
            if propnames.len() == 1 {
                if let Some(value) = results.get(&propnames[0]) {
                    println!("{}", value.trim());
                }
            } else {
                // For multiple properties, print property=value format
                for propname in propnames {
                    if let Some(value) = results.get(propname) {
                        println!("{}={}", propname, value.trim());
                    }
                }
            }
        }
        OutputType::Json => {
            let json = json!({
                "device": device.map(|d| &d.adb_id),
                "properties": results
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputType::Table => {
            let mut table = Table::new();
            table.set_header(vec!["Property", "Value"]);
            table.load_preset(comfy_table::presets::NOTHING);

            for propname in propnames {
                if let Some(value) = results.get(propname) {
                    table.add_row(vec![propname, value.trim()]);
                }
            }

            println!("{table}");
        }
    }

    Ok(())
}
