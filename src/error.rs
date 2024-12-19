use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AdbError {
    DeviceNotFound(String),
    NoDevicesFound,
    AmbiguousDeviceMatch {
        prefix: String,
        matches: Vec<String>,
    },
    AmbiguousConfigMatch {
        device_id: String,
        matching_configs: Vec<String>,
    },
    InvalidCopyOperation(String),
    DeviceIdRequired,
    // Add other error variants as needed
}

impl fmt::Display for AdbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdbError::DeviceNotFound(id) => write!(f, "Device not found: {}", id),
            AdbError::NoDevicesFound => write!(
                f,
                "No devices found. Is the device connected and authorized?"
            ),
            AdbError::AmbiguousDeviceMatch { prefix, matches } => {
                writeln!(
                    f,
                    "Multiple devices match prefix '{}'. Matching devices:",
                    prefix
                )?;
                for device in matches {
                    writeln!(f, "  - {}", device)?;
                }
                write!(f, "Please provide a more specific device ID")
            }
            AdbError::AmbiguousConfigMatch {
                device_id,
                matching_configs,
            } => {
                write!(f, "Ambiguous device configuration for '{}':", device_id)?;
                for config in matching_configs {
                    writeln!(f, "  - {}", config)?;
                }
                write!(f, "Please provide a more specific device ID")
            }
            AdbError::InvalidCopyOperation(msg) => write!(f, "Invalid copy operation: {}", msg),
            AdbError::DeviceIdRequired => write!(
                f,
                "Device ID is required when multiple devices are connected"
            ),
        }
    }
}

impl Error for AdbError {}
