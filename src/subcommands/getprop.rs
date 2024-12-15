use crate::cli::OutputType;
use crate::library::adb;
use crate::types::DeviceDetails;
use comfy_table::Table;
use std::collections::HashMap;
use std::sync::LazyLock;

pub async fn run(
    host: &str,
    port: &str,
    propname: &str,
    target_device: Option<&DeviceDetails>,
    output_type: OutputType,
) -> Result<(), Box<dyn std::error::Error>> {
    let adb_id = target_device.map(|d| d.adb_id.as_str());
    let result = adb::getprop_async(host, port, propname, adb_id).await?;

    match output_type {
        OutputType::Json => display_json(propname, &result),
        OutputType::Table => display_table(target_device, propname, &result),
        OutputType::Plain => println!("{}", result.trim()),
    }

    Ok(())
}

fn display_json(propname: &str, value: &str) {
    let json = serde_json::json!({
        propname: value.trim()
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}

fn display_table(device: Option<&DeviceDetails>, propname: &str, value: &str) {
    let mut table = Table::new();
    table.set_header(vec!["PROPERTY", "VALUE"]);
    table.load_preset(comfy_table::presets::NOTHING);

    table.add_row(vec![propname, value.trim()]);

    println!("{table}");
}
