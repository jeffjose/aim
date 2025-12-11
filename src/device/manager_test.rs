//! Tests for DeviceManager

#[cfg(test)]
mod tests {
    use crate::core::types::DeviceState;
    use crate::device::DeviceManager;

    #[test]
    fn test_device_manager_default() {
        let _manager = DeviceManager::default();
        // Should create without panicking
        assert!(true, "DeviceManager::default() works");
    }

    #[test]
    fn test_device_manager_new() {
        let _manager = DeviceManager::new();
        // Should create without panicking
        assert!(true, "DeviceManager::new() works");
    }

    #[test]
    fn test_device_manager_with_address() {
        let _manager = DeviceManager::with_address("192.168.1.100", "5555");
        // Should create without panicking
        assert!(true, "DeviceManager::with_address() works");
    }

    #[test]
    fn test_details_to_device_conversion() {
        use crate::types::DeviceDetails;
        use std::collections::HashMap;

        let details = DeviceDetails {
            adb_id: "abc123".to_string(),
            device_type: "device".to_string(),
            device_id: "full_id".to_string(),
            device_id_short: "abc".to_string(),
            device_name: "test-phone".to_string(),
            brand: Some("Google".to_string()),
            model: Some("Pixel 6".to_string()),
            usb: None,
            product: Some("oriole".to_string()),
            device: Some("oriole".to_string()),
            transport_id: Some("1".to_string()),
            additional_props: HashMap::new(),
        };

        let device = DeviceManager::details_to_device(details);

        assert_eq!(device.id.as_str(), "abc123");
        assert!(matches!(device.state, DeviceState::Device));
        assert_eq!(device.model, Some("Pixel 6".to_string()));
        assert_eq!(device.product, Some("oriole".to_string()));
    }

    #[test]
    fn test_details_to_device_offline() {
        use crate::types::DeviceDetails;
        use std::collections::HashMap;

        let details = DeviceDetails {
            adb_id: "offline123".to_string(),
            device_type: "offline".to_string(),
            device_id: String::new(),
            device_id_short: String::new(),
            device_name: String::new(),
            brand: None,
            model: None,
            usb: None,
            product: None,
            device: None,
            transport_id: None,
            additional_props: HashMap::new(),
        };

        let device = DeviceManager::details_to_device(details);

        assert_eq!(device.id.as_str(), "offline123");
        assert!(matches!(device.state, DeviceState::Offline));
    }

    #[test]
    fn test_details_to_device_unauthorized() {
        use crate::types::DeviceDetails;
        use std::collections::HashMap;

        let details = DeviceDetails {
            adb_id: "unauth123".to_string(),
            device_type: "unauthorized".to_string(),
            device_id: String::new(),
            device_id_short: String::new(),
            device_name: String::new(),
            brand: None,
            model: None,
            usb: None,
            product: None,
            device: None,
            transport_id: None,
            additional_props: HashMap::new(),
        };

        let device = DeviceManager::details_to_device(details);

        assert_eq!(device.id.as_str(), "unauth123");
        assert!(matches!(device.state, DeviceState::Unauthorized));
    }

    #[test]
    fn test_details_to_device_unknown_state() {
        use crate::types::DeviceDetails;
        use std::collections::HashMap;

        let details = DeviceDetails {
            adb_id: "mystery123".to_string(),
            device_type: "something_weird".to_string(),
            device_id: String::new(),
            device_id_short: String::new(),
            device_name: String::new(),
            brand: None,
            model: None,
            usb: None,
            product: None,
            device: None,
            transport_id: None,
            additional_props: HashMap::new(),
        };

        let device = DeviceManager::details_to_device(details);

        assert!(matches!(device.state, DeviceState::Unknown));
    }
}
