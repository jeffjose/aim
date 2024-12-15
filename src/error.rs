use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AdbError {
    DeviceNotFound(String),
    NoDevicesFound,
    // Add other error variants as needed
}

impl fmt::Display for AdbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdbError::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            AdbError::NoDevicesFound => write!(f, "No devices found. Is the device connected and authorized?"),
        }
    }
}

impl Error for AdbError {} 
