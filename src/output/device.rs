use crate::core::types::{Device, DeviceState};
use crate::output::{TableFormat, PlainFormat};
use crate::types::DeviceDetails;
use comfy_table::{Cell, Color};

/// Formatter for device information
#[allow(dead_code)]
pub struct DeviceFormatter;

impl TableFormat for DeviceDetails {
    fn headers() -> Vec<&'static str> {
        vec![
            "DEVICE ID",
            "BRAND",
            "MODEL",
            "STATUS",
            "ADB ID",
            "NAME"
        ]
    }

    fn row(&self) -> Vec<String> {
        let status = if self.additional_props.get("service.adb.root") == Some(&"1".to_string()) {
            "root"
        } else {
            "device"
        };

        vec![
            self.device_id_short.clone(),
            self.brand.clone().unwrap_or_default(),
            self.model.clone().unwrap_or_default(),
            status.to_string(),
            self.adb_id.clone(),
            self.device_name.clone(),
        ]
    }

    fn colored_row(&self) -> Vec<Cell> {
        let is_root = self.additional_props.get("service.adb.root") == Some(&"1".to_string());
        let status = if is_root { "root" } else { "device" };
        let status_color = if is_root { Color::Yellow } else { Color::Green };

        vec![
            Cell::new(self.device_id_short.clone()),
            Cell::new(self.brand.clone().unwrap_or_default()),
            Cell::new(self.model.clone().unwrap_or_default()),
            Cell::new(status).fg(status_color),
            Cell::new(self.adb_id.clone()),
            Cell::new(self.device_name.clone()),
        ]
    }
}

impl PlainFormat for DeviceDetails {
    fn plain(&self) -> String {
        self.adb_id.clone()
    }
}

impl TableFormat for Device {
    fn headers() -> Vec<&'static str> {
        vec![
            "DEVICE ID",
            "STATE",
            "MODEL",
            "PRODUCT"
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.state.to_string(),
            self.model.clone().unwrap_or_default(),
            self.product.clone().unwrap_or_default(),
        ]
    }

    fn colored_row(&self) -> Vec<Cell> {
        let state_color = match self.state {
            DeviceState::Device => Color::Green,
            DeviceState::Offline => Color::Red,
            DeviceState::Unauthorized => Color::Yellow,
            DeviceState::Unknown => Color::DarkGrey,
        };

        vec![
            Cell::new(self.id.to_string()),
            Cell::new(self.state.to_string()).fg(state_color),
            Cell::new(self.model.clone().unwrap_or_default()),
            Cell::new(self.product.clone().unwrap_or_default()),
        ]
    }
}

impl PlainFormat for Device {
    fn plain(&self) -> String {
        format!("{}\t{}", self.id, self.state)
    }
}