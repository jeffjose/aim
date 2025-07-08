use crate::core::types::{Device, DeviceId};
use crate::error::{Result, AimError};

/// Unified device management
#[derive(Clone)]
pub struct DeviceManager;

impl DeviceManager {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        // Placeholder implementation using existing device_info
        use super::device_info;
        use crate::core::types::DeviceState;
        
        // For now, use default host and port
        let device_details = device_info::get_devices("localhost", "5037").await;
        
        Ok(device_details.into_iter().map(|d| {
            Device::new(DeviceId::new(d.adb_id))
                .with_state(DeviceState::Device)
                .with_model(d.model.unwrap_or_default())
                .with_product(d.product.unwrap_or_default())
                .with_device(d.device.unwrap_or_default())
        }).collect())
    }
    
    pub async fn find_device(&self, partial_id: &str) -> Result<Device> {
        let devices = self.list_devices().await?;
        
        // Smart matching logic
        let matches: Vec<_> = devices.iter()
            .filter(|d| d.id.as_str().contains(partial_id))
            .collect();
            
        match matches.len() {
            0 => Err(AimError::DeviceNotFound(partial_id.to_string())),
            1 => Ok(matches[0].clone()),
            _ => Err(AimError::MultipleDevicesFound),
        }
    }
    
    pub async fn get_device_details(&self, device: Device) -> Result<crate::types::DeviceDetails> {
        use super::device_info;
        
        // For now, get all devices and find the matching one
        let all_devices = device_info::get_devices("localhost", "5037").await;
        
        all_devices.into_iter()
            .find(|d| d.adb_id == device.id.as_str())
            .ok_or_else(|| AimError::DeviceNotFound(device.id.to_string()))
    }
}