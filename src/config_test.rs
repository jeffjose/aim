use super::*;
use config::{Config, DeviceConfig};
use std::{collections::HashMap, fs, path::PathBuf};
use tempfile::TempDir;

fn create_test_config(dir: &TempDir, contents: &str) -> PathBuf {
    let config_path = dir.path().join(".aimconfig");
    fs::write(&config_path, contents).unwrap();
    config_path
}

#[test]
fn test_empty_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_test_config(&temp_dir, "");

    let config = Config::load_from_path(&config_path);
    assert!(config.aliases.is_empty());
    assert!(config.devices.is_empty());
}

#[test]
fn test_alias_resolution() {
    let config = Config {
        aliases: {
            let mut map = HashMap::new();
            map.insert("ls".to_string(), "shell ls -la".to_string());
            map.insert("clear".to_string(), "shell clear".to_string());
            map
        },
        devices: HashMap::new(),
        screenshot: None,
        screenrecord: None,
    };

    assert_eq!(config.resolve_alias("ls"), "shell ls -la");
    assert_eq!(config.resolve_alias("clear"), "shell clear");
    assert_eq!(config.resolve_alias("unknown"), "unknown");
}

#[test]
fn test_device_name_lookup() {
    let config = Config {
        aliases: HashMap::new(),
        devices: {
            let mut map = HashMap::new();
            map.insert(
                "device1".to_string(),
                DeviceConfig {
                    name: Some("My Phone".to_string()),
                },
            );
            map.insert(
                "device2".to_string(),
                DeviceConfig {
                    name: Some("Tablet".to_string()),
                },
            );
            map
        },
        screenshot: None,
        screenrecord: None,
    };

    assert_eq!(
        config.get_device_name("device1"),
        Some("My Phone".to_string())
    );
    assert_eq!(config.get_device_name("dev"), None); // Ambiguous match
    assert_eq!(config.get_device_name("unknown"), None);
}

#[test]
fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_contents = r#"
[alias]
ls = "shell ls -la"
clear = "shell clear"

[device.abc123]
name = "Test Phone"

[device.def456]
name = "Test Tablet"
"#;
    let config_path = create_test_config(&temp_dir, config_contents);

    let config = Config::load_from_path(&config_path);

    // Check aliases
    assert_eq!(config.aliases.get("ls"), Some(&"shell ls -la".to_string()));
    assert_eq!(
        config.aliases.get("clear"),
        Some(&"shell clear".to_string())
    );

    // Check devices
    assert_eq!(
        config.devices.get("abc123").and_then(|d| d.name.as_ref()),
        Some(&"Test Phone".to_string())
    );
    assert_eq!(
        config.devices.get("def456").and_then(|d| d.name.as_ref()),
        Some(&"Test Tablet".to_string())
    );
}

#[test]
fn test_case_insensitive_device_lookup() {
    let config = Config {
        aliases: HashMap::new(),
        devices: {
            let mut map = HashMap::new();
            map.insert(
                "ABC123".to_string(),
                DeviceConfig {
                    name: Some("Test Device".to_string()),
                },
            );
            map
        },
        screenshot: None,
        screenrecord: None,
    };

    assert_eq!(
        config.get_device_name("abc123"),
        Some("Test Device".to_string())
    );
    assert_eq!(
        config.get_device_name("ABC123"),
        Some("Test Device".to_string())
    );
    assert_eq!(
        config.get_device_name("abc"),
        Some("Test Device".to_string())
    );
    assert_eq!(
        config.get_device_name("ABC"),
        Some("Test Device".to_string())
    );
}

#[test]
fn test_invalid_toml_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_test_config(&temp_dir, "this is not valid toml");

    let config = Config::load_from_path(&config_path);
    assert!(config.aliases.is_empty());
    assert!(config.devices.is_empty());
}

#[test]
fn test_partial_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_contents = r#"
[alias]
ls = "shell ls -la"
invalid_value = 123  # Should be string

[device.abc123]
name = "Test Phone"
"#;
    let config_path = create_test_config(&temp_dir, config_contents);

    let config = Config::load_from_path(&config_path);
    assert_eq!(config.aliases.get("ls"), Some(&"shell ls -la".to_string()));
    assert_eq!(
        config.devices.get("abc123").and_then(|d| d.name.as_ref()),
        Some(&"Test Phone".to_string())
    );
}

#[test]
fn test_device_name_partial_match() {
    let config = Config {
        aliases: HashMap::new(),
        devices: {
            let mut map = HashMap::new();
            map.insert(
                "adevice123".to_string(),
                DeviceConfig {
                    name: Some("Test Device".to_string()),
                },
            );
            map.insert(
                "device456".to_string(),
                DeviceConfig {
                    name: Some("Other Device".to_string()),
                },
            );
            map
        },
        screenshot: None,
        screenrecord: None,
    };

    // Ambiguous partial match should return None
    assert_eq!(config.get_device_name("abc"), None);
    assert_eq!(
        config.get_device_name("device"),
        Some("Other Device".to_string())
    );
    // Unique partial match should work
    assert_eq!(
        config.get_device_name("ad"),
        Some("Test Device".to_string())
    );
}

#[test]
fn test_device_without_name() {
    let config = Config {
        aliases: HashMap::new(),
        devices: {
            let mut map = HashMap::new();
            map.insert("device123".to_string(), DeviceConfig { name: None });
            map
        },
        screenshot: None,
        screenrecord: None,
    };

    assert_eq!(config.get_device_name("device123"), None);
}

#[test]
fn test_empty_device_section() {
    let temp_dir = TempDir::new().unwrap();
    let config_contents = r#"
[device]
# Empty device section
"#;
    let config_path = create_test_config(&temp_dir, config_contents);

    let config = Config::load_from_path(&config_path);
    assert!(config.devices.is_empty());
}

#[test]
fn test_multiple_partial_matches() {
    let config = Config {
        aliases: HashMap::new(),
        devices: {
            let mut map = HashMap::new();
            map.insert(
                "phone1".to_string(),
                DeviceConfig {
                    name: Some("First Phone".to_string()),
                },
            );
            map.insert(
                "phone2".to_string(),
                DeviceConfig {
                    name: Some("Second Phone".to_string()),
                },
            );
            map
        },
        screenshot: None,
        screenrecord: None,
    };

    // Multiple matches should return None
    assert_eq!(config.get_device_name("phone"), None);
    // Exact match should work
    assert_eq!(
        config.get_device_name("phone1"),
        Some("First Phone".to_string())
    );
}
