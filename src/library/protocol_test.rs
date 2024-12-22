use super::protocol::{format_command, AdbCommand};

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
    // Test that all enum variants are mapped
    for command in [
        AdbCommand::AnyDevice,
        AdbCommand::SelectDevice,
        AdbCommand::Shell,
        AdbCommand::ShellV2,
        AdbCommand::GetProp,
        AdbCommand::GetPropSingle,
        AdbCommand::Sync,
        AdbCommand::Push,
        AdbCommand::Pull,
        AdbCommand::Version,
        AdbCommand::Devices,
        AdbCommand::Kill,
        AdbCommand::TrackDevices,
        AdbCommand::Transport,
    ] {
        // Verify each command can format without panicking
        command.format(&[]);
    }
}

#[test]
fn test_shell_commands_with_complex_arguments() {
    assert_eq!(
        format_command("SHELL", &["pm list packages -f"]),
        "shell:pm list packages -f"
    );
    assert_eq!(
        format_command("SHELL", &["dumpsys battery | grep level"]),
        "shell:dumpsys battery | grep level"
    );
    assert_eq!(
        format_command("SHELL", &["am start -n com.example.app/.MainActivity"]),
        "shell:am start -n com.example.app/.MainActivity"
    );
}

#[test]
fn test_shell_v2_commands_with_options() {
    assert_eq!(
        format_command("SHELL_V2", &["logcat -v time"]),
        "shell,v2,TERM=xterm-256color,raw:logcat -v time"
    );
    assert_eq!(
        format_command("SHELL_V2", &["screencap -p"]),
        "shell,v2,TERM=xterm-256color,raw:screencap -p"
    );
}

#[test]
fn test_getprop_commands_with_patterns() {
    assert_eq!(
        format_command("GETPROP_SINGLE", &["ro.build.*"]),
        "shell:getprop ro.build.*"
    );
    assert_eq!(
        format_command("GETPROP_SINGLE", &["persist.sys.usb.config"]),
        "shell:getprop persist.sys.usb.config"
    );
}

#[test]
fn test_device_selection_with_special_chars() {
    assert_eq!(
        format_command("SELECT_DEVICE", &["emulator-5554"]),
        "host:tport:serial:emulator-5554"
    );
    assert_eq!(
        format_command("SELECT_DEVICE", &["192.168.1.100:5555"]),
        "host:tport:serial:192.168.1.100:5555"
    );
}

#[test]
fn test_sync_commands_with_paths() {
    assert_eq!(
        format_command("PUSH", &["/path/with spaces/file.txt"]),
        "sync:/path/with spaces/file.txt"
    );
    assert_eq!(
        format_command("PULL", &["/data/app/com.example.app-1.apk"]),
        "sync:/data/app/com.example.app-1.apk"
    );
}

#[test]
fn test_commands_with_empty_arguments() {
    assert_eq!(format_command("SHELL", &[""]), "shell:");
    assert_eq!(format_command("GETPROP_SINGLE", &[""]), "shell:getprop ");
}

#[test]
fn test_commands_with_unicode() {
    assert_eq!(
        format_command("SHELL", &["echo 'Hello 世界'"]),
        "shell:echo 'Hello 世界'"
    );
    assert_eq!(
        format_command("PUSH", &["/path/to/文件.txt"]),
        "sync:/path/to/文件.txt"
    );
}

#[test]
fn test_commands_with_multiple_arguments() {
    assert_eq!(
        format_command("SHELL", &["input", "text", "'hello world'"]),
        "shell:input text 'hello world'"
    );
    assert_eq!(
        format_command("SHELL", &["am", "force-stop", "com.example.app"]),
        "shell:am force-stop com.example.app"
    );
}

#[test]
fn test_transport_command_variations() {
    assert_eq!(
        format_command("TRANSPORT", &["usb:123456"]),
        "host:transport:usb:123456"
    );
    assert_eq!(
        format_command("TRANSPORT", &["local"]),
        "host:transport:local"
    );
}

#[test]
fn test_command_case_sensitivity() {
    assert_eq!(format_command("VERSION", &[]), "host:version");
    assert_eq!(format_command("version", &[]), "host:version");
    assert_eq!(format_command("DEVICES", &[]), "host:devices");
    assert_eq!(format_command("devices", &[]), "host:devices");
}

#[test]
fn test_adb_command_direct_format() {
    assert_eq!(AdbCommand::Version.format(&[]), "host:version");
    assert_eq!(AdbCommand::Shell.format(&["ls"]), "shell:ls");
    assert_eq!(
        AdbCommand::Transport.format(&["device1"]),
        "host:transport:device1"
    );
}
