use crate::core::types::Device;
use crate::output::{TableFormat, PlainFormat};
use crate::types::DeviceDetails;

/// Formatter for device information
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
}

impl PlainFormat for Device {
    fn plain(&self) -> String {
        format!("{}\t{}", self.id, self.state)
    }
}