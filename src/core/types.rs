use std::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strongly typed device identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn short_id(&self) -> &str {
        // Return first 8 characters of the ID or the full ID if shorter
        let id = self.as_str();
        if id.len() <= 8 {
            id
        } else {
            &id[..8]
        }
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DeviceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DeviceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Device state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceState {
    Device,
    Offline,
    Unauthorized,
    Unknown,
}

impl DeviceState {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "device" => DeviceState::Device,
            "offline" => DeviceState::Offline,
            "unauthorized" => DeviceState::Unauthorized,
            _ => DeviceState::Unknown,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            DeviceState::Device => "device",
            DeviceState::Offline => "offline",
            DeviceState::Unauthorized => "unauthorized",
            DeviceState::Unknown => "unknown",
        }
    }
}

impl fmt::Display for DeviceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Core device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub state: DeviceState,
    pub transport_id: Option<u32>,
    pub model: Option<String>,
    pub product: Option<String>,
    pub device: Option<String>,
    /// User-defined alias from config
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[allow(dead_code)]
impl Device {
    pub fn new(id: impl Into<DeviceId>) -> Self {
        Self {
            id: id.into(),
            state: DeviceState::Unknown,
            transport_id: None,
            model: None,
            product: None,
            device: None,
            alias: None,
        }
    }
    
    pub fn with_state(mut self, state: DeviceState) -> Self {
        self.state = state;
        self
    }
    
    pub fn with_transport_id(mut self, transport_id: u32) -> Self {
        self.transport_id = Some(transport_id);
        self
    }
    
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    pub fn with_product(mut self, product: impl Into<String>) -> Self {
        self.product = Some(product.into());
        self
    }
    
    pub fn with_device(mut self, device: impl Into<String>) -> Self {
        self.device = Some(device.into());
        self
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// Check if device is available for commands
    pub fn is_available(&self) -> bool {
        self.state == DeviceState::Device
    }
    
    /// Get a display name for the device
    pub fn display_name(&self) -> String {
        if let Some(model) = &self.model {
            format!("{} ({})", model, self.id)
        } else {
            self.id.to_string()
        }
    }
}

/// Extended device properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceProperties {
    pub brand: Option<String>,
    pub manufacturer: Option<String>,
    pub sdk_version: Option<String>,
    pub android_version: Option<String>,
    pub build_type: Option<String>,
    pub additional: HashMap<String, String>,
}

#[allow(dead_code)]
impl DeviceProperties {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_brand(mut self, brand: impl Into<String>) -> Self {
        self.brand = Some(brand.into());
        self
    }
    
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self
    }
    
    pub fn with_sdk_version(mut self, sdk_version: impl Into<String>) -> Self {
        self.sdk_version = Some(sdk_version.into());
        self
    }
    
    pub fn with_android_version(mut self, android_version: impl Into<String>) -> Self {
        self.android_version = Some(android_version.into());
        self
    }
    
    pub fn with_build_type(mut self, build_type: impl Into<String>) -> Self {
        self.build_type = Some(build_type.into());
        self
    }
    
    pub fn add_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional.insert(key.into(), value.into());
        self
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Plain,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(OutputFormat::Table),
            "json" => Some(OutputFormat::Json),
            "plain" => Some(OutputFormat::Plain),
            _ => None,
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Plain => write!(f, "plain"),
        }
    }
}

/// Common command options
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommonOptions {
    pub device: Option<String>,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl Default for CommonOptions {
    fn default() -> Self {
        Self {
            device: None,
            output_format: OutputFormat::Table,
            verbose: false,
        }
    }
}

/// File transfer direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    #[allow(dead_code)]
    Push,
    #[allow(dead_code)]
    Pull,
}

/// Progress information for file transfers
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub file_path: String,
}

#[allow(dead_code)]
impl TransferProgress {
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_transferred as f64 / self.total_bytes as f64) * 100.0
        }
    }
}