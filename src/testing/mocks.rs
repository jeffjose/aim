use async_trait::async_trait;
use crate::core::types::{Device, DeviceId, DeviceState};
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

/// Trait for ADB operations that can be mocked
#[async_trait]
pub trait AdbOperations: Send + Sync {
    async fn list_devices(&self) -> Result<Vec<Device>>;
    async fn execute_command(&self, device: &DeviceId, cmd: &str) -> Result<String>;
    async fn push_file(&self, device: &DeviceId, local: &Path, remote: &str) -> Result<()>;
    async fn pull_file(&self, device: &DeviceId, remote: &str, local: &Path) -> Result<()>;
    async fn get_property(&self, device: &DeviceId, property: &str) -> Result<String>;
    async fn get_properties(&self, device: &DeviceId) -> Result<Vec<(String, String)>>;
}

/// Mock implementation of ADB operations for testing
#[cfg(test)]
pub struct MockAdb {
    pub devices: Vec<Device>,
    pub command_responses: HashMap<String, String>,
    pub properties: HashMap<String, String>,
    pub files: HashMap<String, Vec<u8>>,
    pub error_on_next_call: Option<crate::error::AimError>,
}

#[cfg(test)]
impl MockAdb {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            command_responses: HashMap::new(),
            properties: HashMap::new(),
            files: HashMap::new(),
            error_on_next_call: None,
        }
    }
    
    pub fn with_devices(mut self, devices: Vec<Device>) -> Self {
        self.devices = devices;
        self
    }
    
    pub fn with_command_response(mut self, cmd: &str, response: &str) -> Self {
        self.command_responses.insert(cmd.to_string(), response.to_string());
        self
    }
    
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn with_file(mut self, path: &str, content: Vec<u8>) -> Self {
        self.files.insert(path.to_string(), content);
        self
    }
    
    pub fn fail_next_call(mut self, error: crate::error::AimError) -> Self {
        self.error_on_next_call = Some(error);
        self
    }
    
    fn check_error(&mut self) -> Result<()> {
        if let Some(error) = self.error_on_next_call.take() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[async_trait]
impl AdbOperations for MockAdb {
    async fn list_devices(&self) -> Result<Vec<Device>> {
        Ok(self.devices.clone())
    }
    
    async fn execute_command(&self, _device: &DeviceId, cmd: &str) -> Result<String> {
        Ok(self.command_responses.get(cmd)
            .cloned()
            .unwrap_or_else(|| format!("Mock response for: {}", cmd)))
    }
    
    async fn push_file(&self, _device: &DeviceId, _local: &Path, _remote: &str) -> Result<()> {
        Ok(())
    }
    
    async fn pull_file(&self, _device: &DeviceId, remote: &str, local: &Path) -> Result<()> {
        if let Some(content) = self.files.get(remote) {
            std::fs::write(local, content)?;
        }
        Ok(())
    }
    
    async fn get_property(&self, _device: &DeviceId, property: &str) -> Result<String> {
        Ok(self.properties.get(property)
            .cloned()
            .unwrap_or_default())
    }
    
    async fn get_properties(&self, _device: &DeviceId) -> Result<Vec<(String, String)>> {
        Ok(self.properties.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
}

/// Mock device manager for testing
#[cfg(test)]
pub struct MockDeviceManager {
    mock_adb: MockAdb,
}

#[cfg(test)]
impl MockDeviceManager {
    pub fn new(mock_adb: MockAdb) -> Self {
        Self { mock_adb }
    }
    
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        self.mock_adb.list_devices().await
    }
    
    pub async fn find_device(&self, partial_id: &str) -> Result<Device> {
        let devices = self.list_devices().await?;
        
        let matches: Vec<_> = devices.iter()
            .filter(|d| d.id.as_str().contains(partial_id))
            .collect();
            
        match matches.len() {
            0 => Err(crate::error::AimError::DeviceNotFound(partial_id.to_string())),
            1 => Ok(matches[0].clone()),
            _ => Err(crate::error::AimError::MultipleDevicesFound),
        }
    }
}

/// Mock progress reporter for testing
#[cfg(test)]
pub struct MockProgressReporter {
    pub started: bool,
    pub current: u64,
    pub finished: bool,
    pub messages: Vec<String>,
}

#[cfg(test)]
impl MockProgressReporter {
    pub fn new() -> Self {
        Self {
            started: false,
            current: 0,
            finished: false,
            messages: Vec::new(),
        }
    }
}

#[cfg(test)]
impl crate::progress::ProgressReporter for MockProgressReporter {
    fn start(&self, _total: u64) {
        // In a real mock, we'd use RefCell or Mutex here
    }
    
    fn update(&self, _current: u64) {
        // In a real mock, we'd use RefCell or Mutex here
    }
    
    fn finish(&self) {
        // In a real mock, we'd use RefCell or Mutex here
    }
    
    fn set_message(&self, _msg: &str) {
        // In a real mock, we'd use RefCell or Mutex here
    }
    
    fn inc(&self, _delta: u64) {
        // In a real mock, we'd use RefCell or Mutex here
    }
}

/// Builder for creating test scenarios
#[cfg(test)]
pub struct TestScenario {
    devices: Vec<Device>,
    properties: HashMap<String, String>,
    command_responses: HashMap<String, String>,
}

#[cfg(test)]
impl TestScenario {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            properties: HashMap::new(),
            command_responses: HashMap::new(),
        }
    }
    
    pub fn with_device(mut self, id: &str) -> Self {
        let device = Device::new(DeviceId::new(id))
            .with_state(DeviceState::Device);
        self.devices.push(device);
        self
    }
    
    pub fn with_device_full(mut self, device: Device) -> Self {
        self.devices.push(device);
        self
    }
    
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn with_command(mut self, cmd: &str, response: &str) -> Self {
        self.command_responses.insert(cmd.to_string(), response.to_string());
        self
    }
    
    pub fn build_mock_adb(self) -> MockAdb {
        MockAdb {
            devices: self.devices,
            command_responses: self.command_responses,
            properties: self.properties,
            files: HashMap::new(),
            error_on_next_call: None,
        }
    }
}