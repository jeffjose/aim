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

#[test]
fn test_find_target_device_with_spaces() {
    let devices = vec![DeviceDetails {
        adb_id: "0123456789ABCDEF".to_string(),
        device_name: "test device with spaces".to_string(),
        ..Default::default()
    }];

    let device_id = "test device with spaces".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().adb_id, "0123456789ABCDEF");
}

#[test]
fn test_find_target_device_with_special_chars() {
    let devices = vec![DeviceDetails {
        adb_id: "0123456789ABCDEF".to_string(),
        device_name: "test-device_123#@".to_string(),
        ..Default::default()
    }];

    let device_id = "test-device_123#@".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().adb_id, "0123456789ABCDEF");
}

#[test]
fn test_extract_device_info_empty_input() {
    let input = "".to_string();
    let result = extract_device_info(input);

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 0);
    } else {
        panic!("Expected empty array result");
    }
}

#[test]
fn test_extract_device_info_malformed_input() {
    let input = "malformed_input_without_tabs".to_string();
    let result = extract_device_info(input);

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 0);
    } else {
        panic!("Expected empty array result");
    }
}

#[test]
fn test_extract_device_info_with_empty_fields() {
    let input = "0123456789ABCDEF    device    product:    model:    device:    transport_id:1";
    let result = extract_device_info(input.to_string());

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 1);
        let device = &devices[0];
        assert_eq!(device["adb_id"], "0123456789ABCDEF");
        assert_eq!(device["type"], "device");
        assert_eq!(device["product"], "");
        assert_eq!(device["model"], "");
        assert_eq!(device["device"], "");
        assert_eq!(device["transport_id"], "1");
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_find_target_device_with_multiple_matches() {
    let devices = vec![
        DeviceDetails {
            adb_id: "0123456789ABCDEF".to_string(),
            device_name: "test-device".to_string(),
            device_id: "TEST123".to_string(),
            ..Default::default()
        },
        DeviceDetails {
            adb_id: "FEDCBA9876543210".to_string(),
            device_name: "test-device-2".to_string(),
            device_id: "TEST456".to_string(),
            ..Default::default()
        },
    ];

    // Should match by exact device_id even if device_name is similar
    let device_id = "TEST123".to_string();
    let result = find_target_device(&devices, Some(&device_id));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().adb_id, "0123456789ABCDEF");
}

#[test]
fn test_extract_device_info_with_whitespace() {
    let input = "0123456789ABCDEF    device    product:sdk   \t  model:Pixel_4  \t  device:generic    transport_id:1";
    let result = extract_device_info(input.to_string());

    if let Value::Array(devices) = result {
        assert_eq!(devices.len(), 1);
        let device = &devices[0];
        assert_eq!(device["adb_id"], "0123456789ABCDEF");
        assert_eq!(device["product"], "sdk");
        assert_eq!(device["model"], "Pixel_4");
    } else {
        panic!("Expected array result");
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{Cli, Commands, OutputType};
    use crate::types::DeviceDetails;
    use clap::Parser;

    fn create_test_devices() -> Vec<DeviceDetails> {
        vec![
            {
                let mut device = DeviceDetails::new("device1".to_string(), "device".to_string());
                device.device_name = "pixel".to_string();
                device.device_id = "PIXEL123".to_string();
                device.device_id_short = "123".to_string();
                device
            },
            {
                let mut device = DeviceDetails::new("device2".to_string(), "device".to_string());
                device.device_name = "samsung".to_string();
                device.device_id = "SAMSUNG456".to_string();
                device.device_id_short = "456".to_string();
                device
            },
        ]
    }

    fn parse_getprop(args: &[&str]) -> Commands {
        let mut all_args = vec!["aim"];
        all_args.extend(args);

        let cli = Cli::parse_from(all_args);
        cli.command()
    }

    #[test]
    fn test_getprop_no_args() {
        let devices = create_test_devices();

        // Single device - should work
        let _single_devices = vec![devices[0].clone()];
        if let Commands::Getprop {
            propnames,
            device_id,
            output,
        } = parse_getprop(&["getprop"])
        {
            assert!(propnames.is_empty());
            assert!(device_id.is_none());
            assert!(matches!(output, OutputType::Plain));
        } else {
            panic!("Expected Getprop command");
        }
    }

    #[test]
    fn test_getprop_single_device() {
        let devices = create_test_devices();
        let _single_devices = vec![devices[0].clone()];

        // Test property names only
        if let Commands::Getprop {
            propnames,
            device_id,
            ..
        } = parse_getprop(&["getprop", "prop1,prop2"])
        {
            assert_eq!(propnames, "prop1,prop2");
            assert!(device_id.is_none());
        }

        // Test device ID only - should get all properties
        if let Commands::Getprop {
            propnames,
            device_id,
            ..
        } = parse_getprop(&["getprop", "123"])
        {
            assert_eq!(propnames, "123");
            assert!(device_id.is_none());
        }
    }

    #[test]
    fn test_getprop_multiple_devices() {
        let devices = create_test_devices();

        // Test properties and device ID
        if let Commands::Getprop {
            propnames,
            device_id,
            ..
        } = parse_getprop(&["getprop", "prop1,prop2", "123"])
        {
            assert_eq!(propnames, "prop1,prop2");
            assert_eq!(device_id, Some("123".to_string()));
        }

        // Test empty properties with device ID
        if let Commands::Getprop {
            propnames,
            device_id,
            ..
        } = parse_getprop(&["getprop", "", "123"])
        {
            assert!(propnames.is_empty());
            assert_eq!(device_id, Some("123".to_string()));
        }
    }

    #[test]
    fn test_getprop_device_matching() {
        let devices = create_test_devices();

        // Test partial device ID matches
        let cases = vec![
            ("123", "device1"),
            ("PIXEL", "device1"),
            ("pixel", "device1"),
            ("456", "device2"),
            ("SAMSUNG", "device2"),
            ("samsung", "device2"),
        ];
        for (input, expected) in cases {
            if let Ok(device) = super::find_target_device(&devices, Some(&input.to_string())) {
                assert_eq!(device.adb_id, expected);
            } else {
                panic!("Failed to match device ID: {}", input);
            }
        }
    }

    #[test]
    #[should_panic(expected = "DeviceIdRequired")]
    fn test_getprop_multiple_devices_no_id() {
        let devices = create_test_devices();

        // Should fail when multiple devices and no ID provided
        if let Commands::Getprop {
            propnames,
            device_id,
            ..
        } = parse_getprop(&["getprop", "prop1,prop2"])
        {
            let _ = super::find_target_device(&devices, device_id.as_ref())
                .expect("Should fail with DeviceIdRequired");
        }
    }
}
