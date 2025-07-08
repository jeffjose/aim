use crate::core::types::{Device, DeviceId, DeviceState, DeviceProperties};
use crate::types::DeviceDetails;

/// Create a test device with minimal information
pub fn test_device(id: &str) -> Device {
    Device::new(DeviceId::new(id))
        .with_state(DeviceState::Device)
        .with_model("TestModel".to_string())
        .with_product("TestProduct".to_string())
}

/// Create a test device with full information
pub fn test_device_full(id: &str, model: &str, brand: &str) -> Device {
    Device::new(DeviceId::new(id))
        .with_state(DeviceState::Device)
        .with_model(model.to_string())
        .with_product(format!("{}_product", brand.to_lowercase()))
        .with_device(format!("{}_device", model.to_lowercase()))
        .with_transport_id(1)
}

/// Create a collection of test devices
pub fn test_devices() -> Vec<Device> {
    vec![
        test_device_full("emulator-5554", "Emulator", "Google"),
        test_device_full("abc123def456", "Pixel 6", "Google"),
        test_device_full("192.168.1.100:5555", "Galaxy S21", "Samsung"),
    ]
}

/// Create test device details for compatibility with existing code
pub fn test_device_details(id: &str) -> DeviceDetails {
    DeviceDetails {
        adb_id: id.to_string(),
        device_type: "device".to_string(),
        device_id: id.to_string(),
        device_id_short: id.chars().take(8).collect(),
        device_name: format!("test-{}", id),
        brand: Some("TestBrand".to_string()),
        model: Some("TestModel".to_string()),
        usb: Some("1-1".to_string()),
        product: Some("test_product".to_string()),
        device: Some("test_device".to_string()),
        transport_id: Some("1".to_string()),
        additional_props: std::collections::HashMap::new(),
    }
}

/// Common device properties for testing
pub fn test_properties() -> Vec<(String, String)> {
    vec![
        ("ro.product.model".to_string(), "TestModel".to_string()),
        ("ro.product.brand".to_string(), "TestBrand".to_string()),
        ("ro.product.device".to_string(), "test_device".to_string()),
        ("ro.build.version.release".to_string(), "13".to_string()),
        ("ro.build.version.sdk".to_string(), "33".to_string()),
        ("ro.product.cpu.abi".to_string(), "arm64-v8a".to_string()),
        ("persist.sys.timezone".to_string(), "UTC".to_string()),
    ]
}

/// Create device properties struct
pub fn test_device_properties() -> DeviceProperties {
    DeviceProperties {
        brand: Some("TestBrand".to_string()),
        manufacturer: Some("TestManufacturer".to_string()),
        sdk_version: Some("33".to_string()),
        android_version: Some("13".to_string()),
        build_type: Some("user".to_string()),
        additional: test_properties().into_iter().collect(),
    }
}

/// Test file content
pub fn test_file_content() -> Vec<u8> {
    b"This is test file content.\nLine 2\nLine 3\n".to_vec()
}

/// Test binary file content
pub fn test_binary_content() -> Vec<u8> {
    vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] // PNG header
}

/// Create a test command context
pub fn test_context() -> crate::core::context::CommandContext {
    use crate::core::types::OutputFormat;
    
    crate::core::context::CommandContext::new()
        .with_output_format(OutputFormat::Table)
        .with_verbose(false)
}

/// Create test command arguments
pub mod args {
    use crate::core::types::OutputFormat;
    
    pub fn ls_args(output: OutputFormat) -> crate::commands::ls::LsArgs {
        crate::commands::ls::LsArgs {
            output: output.to_string(),
        }
    }
}

/// Test data for different scenarios
pub mod scenarios {
    use super::*;
    
    /// No devices connected
    pub fn no_devices() -> Vec<Device> {
        vec![]
    }
    
    /// Single device connected
    pub fn single_device() -> Vec<Device> {
        vec![test_device("abc123")]
    }
    
    /// Multiple devices connected
    pub fn multiple_devices() -> Vec<Device> {
        test_devices()
    }
    
    /// Device with root access
    pub fn rooted_device() -> DeviceDetails {
        let mut details = test_device_details("root123");
        details.additional_props.insert("service.adb.root".to_string(), "1".to_string());
        details
    }
    
    /// Unauthorized device
    pub fn unauthorized_device() -> Device {
        Device::new(DeviceId::new("unauth123"))
            .with_state(DeviceState::Unauthorized)
    }
    
    /// Offline device
    pub fn offline_device() -> Device {
        Device::new(DeviceId::new("offline123"))
            .with_state(DeviceState::Offline)
    }
}