use super::*;
use dirs::home_dir;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

fn setup_test_config(dir: &TempDir, contents: &str) -> PathBuf {
    let config_path = dir.path().join(".aimconfig");
    fs::write(&config_path, contents).expect("Failed to write test config");
    config_path
}

#[tokio::test]
async fn test_config_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".aimconfig");

    // Override home directory for test
    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
    assert!(!config_path.exists());
}

#[tokio::test]
async fn test_config_file_empty() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_config(&temp_dir, "");

    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_file_valid_content() {
    let temp_dir = TempDir::new().unwrap();
    let config_content = r#"
{
    "device_names": {
        "abc123": "my-test-device"
    }
}
"#;
    setup_test_config(&temp_dir, config_content);

    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_file_with_unicode() {
    let temp_dir = TempDir::new().unwrap();
    let config_content = r#"
{
    "device_names": {
        "abc123": "测试设备"
    }
}
"#;
    setup_test_config(&temp_dir, config_content);

    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let config_path = setup_test_config(&temp_dir, "test content");

    // Make file read-only
    let mut perms = fs::metadata(&config_path).unwrap().permissions();
    perms.set_mode(0o400);
    fs::set_permissions(&config_path, perms).unwrap();

    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_file_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".aimconfig");
    fs::create_dir(&config_path).unwrap();

    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_home_dir_not_found() {
    // Set HOME to a non-existent directory
    std::env::set_var("HOME", "/path/that/does/not/exist");

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_file_large_content() {
    let temp_dir = TempDir::new().unwrap();
    let large_content = "x".repeat(1024 * 1024); // 1MB of content
    setup_test_config(&temp_dir, &large_content);

    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
}

#[cfg(unix)]
#[tokio::test]
async fn test_config_file_symlink() {
    use std::os::unix::fs::symlink;

    let temp_dir = TempDir::new().unwrap();
    let real_config = temp_dir.path().join("real_config");
    let config_path = temp_dir.path().join(".aimconfig");

    fs::write(&real_config, "test content").unwrap();
    symlink(&real_config, &config_path).unwrap();

    std::env::set_var("HOME", temp_dir.path());

    let result = crate::subcommands::config::run().await;
    assert!(result.is_ok());
}

#[test]
fn test_config_path_construction() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    let home = home_dir().unwrap();
    let config_path = home.join(".aimconfig");

    assert_eq!(config_path.file_name().unwrap(), ".aimconfig");
}
