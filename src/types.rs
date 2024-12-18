use std::collections::HashMap;

use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeviceDetails {
    // Basic device identifiers
    pub adb_id: String,
    pub device_type: String,
    pub device_id: String,
    pub device_id_short: String,
    pub device_name: String,

    // Device properties from getprop
    pub brand: Option<String>,
    pub model: Option<String>,

    // Connection details
    pub usb: Option<String>,
    pub product: Option<String>,
    pub device: Option<String>,
    pub transport_id: Option<String>,

    // Additional properties
    #[serde(flatten)]
    pub additional_props: HashMap<String, String>,
}

impl DeviceDetails {
    pub fn new(adb_id: String, device_type: String) -> Self {
        DeviceDetails {
            adb_id,
            device_type,
            device_id: String::new(),
            device_id_short: String::new(),
            device_name: String::new(),
            brand: None,
            model: None,
            usb: None,
            product: None,
            device: None,
            transport_id: None,
            additional_props: HashMap::new(),
        }
    }

    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        let obj = value.as_object()?;

        let mut device = DeviceDetails::new(
            obj.get("adb_id")?.as_str()?.to_string(),
            obj.get("type")?.as_str()?.to_string(),
        );

        // Set optional fields
        if let Some(usb) = obj.get("usb").and_then(|v| v.as_str()) {
            device.usb = Some(usb.to_string());
        }
        if let Some(product) = obj.get("product").and_then(|v| v.as_str()) {
            device.product = Some(product.to_string());
        }
        if let Some(device_val) = obj.get("device").and_then(|v| v.as_str()) {
            device.device = Some(device_val.to_string());
        }
        if let Some(transport_id) = obj.get("transport_id").and_then(|v| v.as_str()) {
            device.transport_id = Some(transport_id.to_string());
        }

        Some(device)
    }

    pub fn update_from_props(&mut self, props: HashMap<String, String>) {
        // Update known fields
        if let Some(brand) = props.get("ro.product.product.brand") {
            self.brand = Some(brand.clone());
        }
        if let Some(model) = props.get("ro.product.model") {
            self.model = Some(model.clone());
        }
        if let Some(device_id) = props.get("device_id") {
            self.device_id = device_id.clone();
        }
        if let Some(device_id_short) = props.get("device_id_short") {
            self.device_id_short = device_id_short.clone();
        }
        if let Some(device_name) = props.get("device_name") {
            self.device_name = device_name.clone();
        }

        // Store remaining properties
        for (key, value) in props {
            if ![
                "ro.product.product.brand",
                "ro.product.model",
                "device_id",
                "device_id_short",
                "device_name",
            ]
            .contains(&key.as_str())
            {
                self.additional_props.insert(key, value);
            }
        }
    }

    pub fn matches_id_prefix(&self, id_prefix: &str) -> bool {
        let id_prefix = id_prefix.to_lowercase();
        let matched = self.device_id.to_lowercase().starts_with(&id_prefix)
            || self.device_id_short.to_lowercase().starts_with(&id_prefix)
            || self.device_name.to_lowercase().eq(&id_prefix);

        info!(
            "Checking [{}] {} {} ({}) - {}",
            matched, self.device_id_short, self.device_name, self.adb_id, id_prefix
        );

        matched
    }
}
