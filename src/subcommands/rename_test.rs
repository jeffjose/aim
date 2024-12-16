use crate::subcommands::rename;
use crate::types::DeviceDetails;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_device(id: &str) -> DeviceDetails {
    DeviceDetails {
        adb_id: id.to_string(),
        device_type: "test".to_string(),
        device_id: id.to_string(),
        device_id_short: id.to_string(),
        device_name: format!("test-{}", id),
        brand: None,
        model: None,
        usb: None,
        product: None,
        device: None,
        transport_id: None,
        additional_props: Default::default(),
    }
}

fn setup_test_config(dir: &TempDir, content: &str) -> PathBuf {
    let config_path = dir.path().join(".aimconfig");
    fs::write(&config_path, content).unwrap();
    config_path
}

#[tokio::test]
async fn test_rename_new_device() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = setup_test_config(&temp_dir, "");

    let device = create_test_device("abc123");
    let result = rename::run(&device, "test_device").await;

    assert!(result.is_ok());
    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("[device.abc123]"));
    assert!(content.contains("name = \"test_device\""));
}

#[tokio::test]
async fn test_rename_existing_device() {
    let temp_dir = TempDir::new().unwrap();
    let config = r#"
[device.abc123]
name = "old_name"
"#;
    let config_path = setup_test_config(&temp_dir, config);

    let device = create_test_device("abc123");
    let result = rename::run(&device, "new_name").await;

    assert!(result.is_ok());
    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("[device.abc123]"));
    assert!(content.contains("name = \"new_name\""));
    assert!(!content.contains("old_name"));
}

#[tokio::test]
async fn test_rename_partial_match() {
    let temp_dir = TempDir::new().unwrap();
    let config = r#"
[device.abc123]
name = "old_name"
"#;
    let config_path = setup_test_config(&temp_dir, config);

    let device = create_test_device("abc");
    let result = rename::run(&device, "new_name").await;

    assert!(result.is_ok());
    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("[device.abc123]")); // Should use existing section ID
    assert!(content.contains("name = \"new_name\""));
}

#[tokio::test]
async fn test_rename_ambiguous_match() {
    let temp_dir = TempDir::new().unwrap();
    let config = r#"
[device.abc123]
name = "device1"

[device.abc456]
name = "device2"
"#;
    let config_path = setup_test_config(&temp_dir, config);

    let device = create_test_device("abc");
    let result = rename::run(&device, "new_name").await;

    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("Multiple sections match"));
        assert!(error_msg.contains("device.abc123"));
        assert!(error_msg.contains("device.abc456"));
    }
}

#[tokio::test]
async fn test_rename_preserves_other_sections() {
    let temp_dir = TempDir::new().unwrap();
    let config = r#"[alias]
lsj = "ls -o json"

[device.abc123]
name = "old_name"

[device.def456]
name = "other_device"
"#;
    let config_path = setup_test_config(&temp_dir, config);

    let device = create_test_device("abc123");
    let result = rename::run(&device, "new_name").await;

    assert!(result.is_ok());
    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("[alias]"));
    assert!(content.contains("lsj = \"ls -o json\""));
    assert!(content.contains("[device.def456]"));
    assert!(content.contains("name = \"other_device\""));
    assert!(content.contains("[device.abc123]"));
    assert!(content.contains("name = \"new_name\""));
}
