use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AdbError {
    DeviceNotFound(String),
    // Add other error variants as needed
}

impl fmt::Display for AdbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdbError::DeviceNotFound(id) => write!(f, "No device found matching ID: {}", id),
        }
    }
}

impl Error for AdbError {} 
