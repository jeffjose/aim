use crate::core::types::{Device, DeviceId, DeviceState};
use crate::error::{AimError, Result};
use crate::types::DeviceDetails;
use log::debug;

/// Unified device management
///
/// Provides consistent device discovery and selection across all commands.
#[derive(Clone)]
pub struct DeviceManager {
    host: String,
    port: String,
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceManager {
    /// Create a new DeviceManager with default localhost:5037
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: "5037".to_string(),
        }
    }

    /// Create a DeviceManager with custom host and port
    pub fn with_address(host: impl Into<String>, port: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: port.into(),
        }
    }

    /// List all connected devices (fast - uses only adb devices -l data)
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        use super::device_info;

        debug!("DeviceManager::list_devices() - {}:{}", self.host, self.port);
        let device_details = device_info::get_devices_fast(&self.host, &self.port).await;
        debug!("Found {} devices", device_details.len());

        Ok(device_details.into_iter().map(Self::details_to_device).collect())
    }

    /// List all connected devices with full details
    #[allow(dead_code)]
    pub async fn list_device_details(&self) -> Result<Vec<DeviceDetails>> {
        use super::device_info;

        debug!("DeviceManager::list_device_details() - {}:{}", self.host, self.port);
        let device_details = device_info::get_devices(&self.host, &self.port).await;
        Ok(device_details)
    }

    /// Find a device by partial ID match
    pub async fn find_device(&self, partial_id: &str) -> Result<Device> {
        let devices = self.list_devices().await?;

        // Smart matching - check if ID contains the search string
        let matches: Vec<_> = devices
            .iter()
            .filter(|d| {
                d.id.as_str().to_lowercase().contains(&partial_id.to_lowercase())
            })
            .collect();

        match matches.len() {
            0 => Err(AimError::DeviceNotFound(partial_id.to_string())),
            1 => Ok(matches[0].clone()),
            _ => Err(AimError::MultipleDevicesFound),
        }
    }

    /// Find device details by partial ID match
    #[allow(dead_code)]
    pub async fn find_device_details(&self, partial_id: &str) -> Result<DeviceDetails> {
        let devices = self.list_device_details().await?;

        let matches: Vec<_> = devices
            .iter()
            .filter(|d| d.matches_id_prefix(partial_id))
            .collect();

        match matches.len() {
            0 => Err(AimError::DeviceNotFound(partial_id.to_string())),
            1 => Ok(matches[0].clone()),
            _ => Err(AimError::MultipleDevicesFound),
        }
    }

    /// Get a single device, or error if none or multiple
    pub async fn get_single_device(&self) -> Result<Device> {
        let devices = self.list_devices().await?;

        match devices.len() {
            0 => Err(AimError::NoDevicesFound),
            1 => Ok(devices.into_iter().next().unwrap()),
            _ => Err(AimError::DeviceIdRequired),
        }
    }

    /// Get target device - uses device_id if provided, otherwise requires single device
    pub async fn get_target_device(&self, device_id: Option<&str>) -> Result<Device> {
        match device_id {
            Some(id) => self.find_device(id).await,
            None => self.get_single_device().await,
        }
    }

    /// Get target device details - uses device_id if provided, otherwise requires single device
    #[allow(dead_code)]
    pub async fn get_target_device_details(&self, device_id: Option<&str>) -> Result<DeviceDetails> {
        match device_id {
            Some(id) => self.find_device_details(id).await,
            None => {
                let devices = self.list_device_details().await?;
                match devices.len() {
                    0 => Err(AimError::NoDevicesFound),
                    1 => Ok(devices.into_iter().next().unwrap()),
                    _ => Err(AimError::DeviceIdRequired)
                }
            }
        }
    }

    /// Convert DeviceDetails to Device
    pub(crate) fn details_to_device(d: DeviceDetails) -> Device {
        let state = match d.device_type.as_str() {
            "device" => DeviceState::Device,
            "offline" => DeviceState::Offline,
            "unauthorized" => DeviceState::Unauthorized,
            _ => DeviceState::Unknown,
        };

        Device::new(DeviceId::new(d.adb_id))
            .with_state(state)
            .with_model(d.model.unwrap_or_default())
            .with_product(d.product.unwrap_or_default())
            .with_device(d.device.unwrap_or_default())
    }
}