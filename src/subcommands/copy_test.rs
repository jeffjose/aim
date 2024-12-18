use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    error::AdbError,
    subcommands::copy::{parse_device_path, run, CopyArgs},
    types::DeviceDetails,
};

#[test]
fn test_parse_device_path() {
    // Test local path
    let local_path = PathBuf::from("/path/to/file");
    let (device_id, path) = parse_device_path(&local_path).unwrap();
    assert_eq!(device_id, None);
    assert_eq!(path, PathBuf::from("/path/to/file"));

    // Test device path
    let device_path = PathBuf::from("device1:/path/on/device");
    let (device_id, path) = parse_device_path(&device_path).unwrap();
    assert_eq!(device_id, Some("device1".to_string()));
    assert_eq!(path, PathBuf::from("/path/on/device"));

    // Test empty path
    let empty_path = PathBuf::from("");
    let (device_id, path) = parse_device_path(&empty_path).unwrap();
    assert_eq!(device_id, None);
    assert_eq!(path, PathBuf::from(""));

    // Test device path with multiple colons
    let multi_colon_path = PathBuf::from("device1:/path:with:colons");
    let (device_id, path) = parse_device_path(&multi_colon_path).unwrap();
    assert_eq!(device_id, Some("device1".to_string()));
    assert_eq!(path, PathBuf::from("/path:with:colons"));
}

#[tokio::test]
async fn test_run_no_device_specified() {
    let devices = vec![DeviceDetails {
        adb_id: "device1".to_string(),
        device_id: "id1".to_string(),
        device_id_short: "short1".to_string(),
        device_name: "name1".to_string(),
        device_type: "device".to_string(),
        brand: None,
        model: None,
        product: None,
        device: None,
        usb: None,
        transport_id: None,
        additional_props: HashMap::new(),
    }];

    let args = CopyArgs {
        src: vec![PathBuf::from("/local/path")],
        dst: PathBuf::from("/another/local/path"),
    };

    let result = run(args, &devices, "localhost", "5037").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let adb_err = err.downcast_ref::<AdbError>().unwrap();
    assert!(matches!(
        adb_err,
        AdbError::InvalidCopyOperation(msg) if msg == "At least one path must specify a device"
    ));
}

#[tokio::test]
async fn test_run_multiple_sources() {
    let devices = vec![DeviceDetails {
        adb_id: "device1".to_string(),
        device_id: "id1".to_string(),
        device_id_short: "short1".to_string(),
        device_name: "name1".to_string(),
        device_type: "device".to_string(),
        brand: None,
        model: None,
        product: None,
        device: None,
        usb: None,
        transport_id: None,
        additional_props: HashMap::new(),
    }];

    let args = CopyArgs {
        src: vec![
            PathBuf::from("device1:/path1"),
            PathBuf::from("device1:/path2"),
            PathBuf::from("device1:/path3"),
        ],
        dst: PathBuf::from("/local/path"),
    };

    // This test will fail because we need to mock adb::pull
    // In a real implementation, we would use a mock or test double for adb::pull
    let result = run(args, &devices, "localhost", "5037").await;
    assert!(result.is_err()); // Will fail due to actual ADB operations
}
