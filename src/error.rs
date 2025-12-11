use thiserror::Error;

fn format_device_list(devices: &[String]) -> String {
    devices
        .iter()
        .map(|d| format!("  {}", d))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AimError {
    #[error("No devices found. Is the device connected and authorized?")]
    NoDevicesFound,
    
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Multiple devices found, please specify one")]
    MultipleDevicesFound,
    
    #[error("Multiple devices match prefix '{prefix}'. Matching devices: {}", matches.join(", "))]
    AmbiguousDeviceMatch {
        prefix: String,
        matches: Vec<String>,
    },
    
    #[error("Ambiguous device configuration for '{device_id}': {}", matching_configs.join(", "))]
    AmbiguousConfigMatch {
        device_id: String,
        matching_configs: Vec<String>,
    },
    
    #[error("Multiple devices connected. Specify a device:\n{}", format_device_list(.0))]
    DeviceIdRequired(Vec<String>),
    
    #[error("ADB connection error: {0}")]
    AdbConnection(#[from] std::io::Error),
    
    #[error("ADB protocol error: {0}")]
    AdbProtocol(String),
    
    #[error("File transfer error: {0}")]
    FileTransfer(String),
    
    #[error("Command execution error: {0}")]
    CommandExecution(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Invalid copy operation: {0}")]
    InvalidCopyOperation(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Screenshot error: {0}")]
    Screenshot(String),
    
    #[error("Screen recording error: {0}")]
    ScreenRecord(String),
    
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Shell error: {0}")]
    Shell(String),
    
    #[error("Timeout error: operation timed out after {0} seconds")]
    Timeout(u64),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    
    #[error("UTF-8 string error: {0}")]
    Utf8Str(#[from] std::str::Utf8Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AimError>;

// Compatibility layer for existing AdbError references
pub type AdbError = AimError;

impl From<String> for AimError {
    fn from(s: String) -> Self {
        AimError::Other(s)
    }
}

impl From<&str> for AimError {
    fn from(s: &str) -> Self {
        AimError::Other(s.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AimError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AimError::Other(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AimError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AimError::Other(err.to_string())
    }
}