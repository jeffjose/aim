use crate::cli::OutputType;
use crate::types::DeviceDetails;
use comfy_table::Table;
use std::collections::HashMap;
use std::sync::LazyLock;

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

    m.insert(
        "adb_status".to_string(),
        TableDetails {
            display_name: "STATUS".to_string(),
        },
    );

    m
});

pub async fn run(devices: &[DeviceDetails], output_type: OutputType) {
    display_devices(devices, output_type);
}

pub fn display_devices(devices: &[DeviceDetails], output_type: OutputType) {
    let headers_to_display = vec![
        "device_id_short".to_string(),
        "ro.product.product.brand".to_string(),
        "ro.product.model".to_string(),
        "adb_status".to_string(),
        "adb_id".to_string(),
        "device_name".to_string(),
    ];

    match output_type {
        OutputType::Json => display_json(devices),
        OutputType::Table => display_table(devices, &headers_to_display),
        OutputType::Plain => display_plain(devices),
    }
}

fn display_plain(devices: &[DeviceDetails]) {
    println!("{:?}", devices);
}

fn display_json(devices: &[DeviceDetails]) {
    println!("{}", serde_json::to_string_pretty(devices).unwrap());
}

fn display_table(devices: &[DeviceDetails], headers: &[String]) {
    let mut table = Table::new();

    let header_names: Vec<String> = headers
        .iter()
        .filter_map(|key| HEADERS.get(key).map(|details| details.display_name.clone()))
        .collect();

    table.set_header(&header_names);
    table.load_preset(comfy_table::presets::NOTHING);

    for device in devices {
        let mut values: Vec<String> = Vec::new();
        for header in headers {
            let value = match header.as_str() {
                "device_id_short" => device.device_id_short.clone(),
                "ro.product.product.brand" => device.brand.clone().unwrap_or_default(),
                "ro.product.model" => device.model.clone().unwrap_or_default(),
                "adb_status" => {
                    if device.additional_props.get("service.adb.root") == Some(&"1".to_string()) {
                        "root".to_string()
                    } else {
                        "".to_string()
                    }
                },
                "adb_id" => device.adb_id.clone(),
                "device_name" => device.device_name.clone(),
                _ => device
                    .additional_props
                    .get(header)
                    .cloned()
                    .unwrap_or_default(),
            };
            values.push(value);
        }
        table.add_row(values);
    }

    println!("{table}");
}
