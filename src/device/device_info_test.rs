use crate::device::device_info::{extract_device_info, find_target_device};
use crate::types::DeviceDetails;
use serde_json::Value;

#[test]
fn test_basic_device_format() {
    let input = "0123456789ABCDEF    device";
    let result = extract_device_info(input.to_string());

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 1);
        let device = &devices[0];
        assert_eq!(device["adb_id"], "0123456789ABCDEF");
        assert_eq!(device["type"], "device");
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_full_device_format() {
    let input = "0123456789ABCDEF    device    usb:1-2    product:sdk    model:Pixel_4    device:generic    transport_id:1";
    let result = extract_device_info(input.to_string());

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 1);
        let device = &devices[0];
        assert_eq!(device["adb_id"], "0123456789ABCDEF");
        assert_eq!(device["type"], "device");
        assert_eq!(device["usb"], "1-2");
        assert_eq!(device["product"], "sdk");
        assert_eq!(device["model"], "Pixel_4");
        assert_eq!(device["device"], "generic");
        assert_eq!(device["transport_id"], "1");
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_truncated_device_format() {
    let input = "0123456789ABCDEF    device    product:sdk    model:Pixel_4    device:generic    transport_id:1";
    let result = extract_device_info(input.to_string());

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 1);
        let device = &devices[0];
        assert_eq!(device["adb_id"], "0123456789ABCDEF");
        assert_eq!(device["type"], "device");
        assert_eq!(device["product"], "sdk");
        assert_eq!(device["model"], "Pixel_4");
        assert_eq!(device["device"], "generic");
        assert_eq!(device["transport_id"], "1");
        assert_eq!(device["usb"], "");
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_multiple_devices() {
    let input = "\
0123456789ABCDEF    device    product:sdk    model:Pixel_4    device:generic    transport_id:1
FEDCBA9876543210    offline    product:sdk2    model:Pixel_5    device:generic    transport_id:2";

    let result = extract_device_info(input.to_string());

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 2);

        assert_eq!(devices[0]["adb_id"], "0123456789ABCDEF");
        assert_eq!(devices[0]["type"], "device");
        assert_eq!(devices[0]["model"], "Pixel_4");

        assert_eq!(devices[1]["adb_id"], "FEDCBA9876543210");
        assert_eq!(devices[1]["type"], "offline");
        assert_eq!(devices[1]["model"], "Pixel_5");
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_different_device_states() {
    let states = [
        "device",
        "offline",
        "bootloader",
        "recovery",
        "unauthorized",
        "sideload",
    ];

    for state in states {
        let input = format!("0123456789ABCDEF    {}", state);
        let result = extract_device_info(input);

        if let Value::Array(devices) = result {
            assert_eq!(devices.len(), 1);
            assert_eq!(devices[0]["adb_id"], "0123456789ABCDEF");
            assert_eq!(devices[0]["type"], state);
        } else {
            panic!("Expected array result");
        }
    }
}

#[test]
fn test_find_target_device_single_device_no_id() {
    let devices = vec![DeviceDetails {
        adb_id: "0123456789ABCDEF".to_string(),
        ..Default::default()
    }];

    let result = find_target_device(&devices, None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().adb_id, "0123456789ABCDEF");
}

#[test]
fn test_find_target_device_multiple_devices_no_id() {
    let devices = vec![
        DeviceDetails {
            adb_id: "0123456789ABCDEF".to_string(),
            ..Default::default()
        },
        DeviceDetails {
            adb_id: "FEDCBA9876543210".to_string(),
            ..Default::default()
        },
    ];

    let result = find_target_device(&devices, None);
    assert!(result.is_err());
}

#[test]
fn test_find_target_device_exact_match() {
    let devices = vec![
        DeviceDetails {
            //adb_id: "0123456789ABCDEF".to_string(),
            device_id: "device_id_0123456789ABCDEF".to_string(),
            ..Default::default()
        },
        DeviceDetails {
            //adb_id: "FEDCBA9876543210".to_string(),
            device_id: "device_id_FEDCBA9876543210".to_string(),
            ..Default::default()
        },
    ];

    let device_id = "device_id_0123456789ABCDEF".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().device_id, device_id);
}

#[test]
fn test_find_target_device_prefix_match() {
    let devices = vec![
        DeviceDetails {
            device_id: "0123456789ABCDEF".to_string(),
            ..Default::default()
        },
        DeviceDetails {
            device_id: "FEDCBA9876543210".to_string(),
            ..Default::default()
        },
    ];

    let device_id = "0123".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().device_id, "0123456789ABCDEF");
}

#[test]
fn test_find_target_device_no_match() {
    let devices = vec![DeviceDetails {
        adb_id: "0123456789ABCDEF".to_string(),
        ..Default::default()
    }];

    let device_id = "XXXX".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_err());
}

#[test]
fn test_find_target_device_ambiguous_prefix() {
    let devices = vec![
        DeviceDetails {
            adb_id: "0123456789ABCDEF".to_string(),
            ..Default::default()
        },
        DeviceDetails {
            adb_id: "0123ABCDEF012345".to_string(),
            ..Default::default()
        },
    ];

    let device_id = "0123".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_err());
}

#[test]
fn test_find_target_device_empty_list() {
    let devices: Vec<DeviceDetails> = vec![];
    let device_id = "0123".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_err());
}

#[test]
fn test_find_target_device_empty_list_no_id() {
    let devices: Vec<DeviceDetails> = vec![];
    let result = find_target_device(&devices, None);
    assert!(result.is_err());
}

#[test]
fn test_find_target_device_case_sensitive() {
    let devices = vec![DeviceDetails {
        adb_id: "0123456789ABCDEF".to_string(),
        ..Default::default()
    }];

    let device_id = "0123456789abcdef".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_err());
}

#[test]
fn test_find_target_device_with_device_name() {
    let devices = vec![DeviceDetails {
        adb_id: "0123456789ABCDEF".to_string(),
        device_name: "test-device".to_string(),
        ..Default::default()
    }];

    let device_id = "test-device".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().adb_id, "0123456789ABCDEF");
}
