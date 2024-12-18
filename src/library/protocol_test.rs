use protocol::{format_command, ADB_COMMANDS};

use super::*;

#[test]
fn test_basic_commands() {
    assert_eq!(format_command("VERSION", &[]), "host:version");
    assert_eq!(format_command("DEVICES", &[]), "host:devices");
    assert_eq!(format_command("KILL", &[]), "host:kill");
    assert_eq!(format_command("TRACK_DEVICES", &[]), "host:track-devices");
    assert_eq!(format_command("SYNC", &[]), "sync:");
    assert_eq!(format_command("GETPROP", &[]), "shell:getprop");
}

#[test]
fn test_device_selection_commands() {
    assert_eq!(format_command("ANY_DEVICE", &[]), "host:tport:any");
    assert_eq!(
        format_command("SELECT_DEVICE", &["abc123"]),
        "host:tport:serial:abc123"
    );
    assert_eq!(
        format_command("TRANSPORT", &["def456"]),
        "host:transport:def456"
    );
}

#[test]
fn test_shell_commands() {
    assert_eq!(format_command("SHELL", &["ls"]), "shell:ls");
    assert_eq!(
        format_command("SHELL", &["ls -la /data"]),
        "shell:ls -la /data"
    );
    assert_eq!(
        format_command("SHELL_V2", &["top -n 1"]),
        "shell,v2,TERM=xterm-256color,raw:top -n 1"
    );
}

#[test]
fn test_property_commands() {
    assert_eq!(
        format_command("GETPROP_SINGLE", &["ro.product.model"]),
        "shell:getprop ro.product.model"
    );
}

#[test]
fn test_sync_commands() {
    assert_eq!(format_command("PUSH", &["/local/path"]), "sync:/local/path");
    assert_eq!(
        format_command("PULL", &["/device/path"]),
        "sync:/device/path"
    );
}

#[test]
#[should_panic(expected = "Unknown ADB command: INVALID")]
fn test_unknown_command() {
    format_command("INVALID", &[]);
}

#[test]
fn test_commands_with_special_chars() {
    assert_eq!(
        format_command("SHELL", &["echo 'hello world'"]),
        "shell:echo 'hello world'"
    );
    assert_eq!(
        format_command("SHELL", &["cat /data/local/tmp/file\\ with\\ spaces"]),
        "shell:cat /data/local/tmp/file\\ with\\ spaces"
    );
}

#[test]
fn test_command_map_completeness() {
    // Test that all documented commands are present in the map
    let commands = ADB_COMMANDS.keys().collect::<Vec<_>>();
    assert!(commands.contains(&&"ANY_DEVICE"));
    assert!(commands.contains(&&"SELECT_DEVICE"));
    assert!(commands.contains(&&"SHELL"));
    assert!(commands.contains(&&"SHELL_V2"));
    assert!(commands.contains(&&"GETPROP"));
    assert!(commands.contains(&&"GETPROP_SINGLE"));
    assert!(commands.contains(&&"SYNC"));
    assert!(commands.contains(&&"PUSH"));
    assert!(commands.contains(&&"PULL"));
    assert!(commands.contains(&&"VERSION"));
    assert!(commands.contains(&&"DEVICES"));
    assert!(commands.contains(&&"KILL"));
    assert!(commands.contains(&&"TRACK_DEVICES"));
    assert!(commands.contains(&&"TRANSPORT"));
}
