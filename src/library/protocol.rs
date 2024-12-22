use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum AdbCommand {
    // Device selection
    AnyDevice,
    SelectDevice,
    
    // Shell operations
    Shell,
    ShellV2,
    
    // Property operations
    GetProp,
    GetPropSingle,
    
    // Sync operations
    Sync,
    Push,
    Pull,
    
    // Server operations
    Version,
    Devices,
    Kill,
    TrackDevices,
    
    // Transport operations
    Transport,
}

impl AdbCommand {
    fn template(&self) -> &'static str {
        match self {
            // Device selection
            Self::AnyDevice => "host:tport:any",
            Self::SelectDevice => "host:tport:serial:{}",
            
            // Shell operations
            Self::Shell => "shell:{}",
            Self::ShellV2 => "shell,v2,TERM=xterm-256color,raw:{}",
            
            // Property operations
            Self::GetProp => "shell:getprop",
            Self::GetPropSingle => "shell:getprop {}",
            
            // Sync operations
            Self::Sync => "sync:",
            Self::Push => "sync:{}",
            Self::Pull => "sync:{}",
            
            // Server operations
            Self::Version => "host:version",
            Self::Devices => "host:devices",
            Self::Kill => "host:kill",
            Self::TrackDevices => "host:track-devices",
            
            // Transport operations
            Self::Transport => "host:transport:{}",
        }
    }

    pub fn format(&self, args: &[&str]) -> String {
        let template = self.template();
        if args.is_empty() {
            template.to_string()
        } else {
            template.replace("{}", &args.join(" "))
        }
    }
}

lazy_static! {
    static ref COMMAND_MAP: HashMap<&'static str, AdbCommand> = {
        let mut m = HashMap::new();
        
        // Device selection commands
        m.insert("ANY_DEVICE", AdbCommand::AnyDevice);
        m.insert("SELECT_DEVICE", AdbCommand::SelectDevice);
        
        // Shell commands
        m.insert("SHELL", AdbCommand::Shell);
        m.insert("SHELL_V2", AdbCommand::ShellV2);
        
        // Property commands
        m.insert("GETPROP", AdbCommand::GetProp);
        m.insert("GETPROP_SINGLE", AdbCommand::GetPropSingle);
        
        // Sync commands
        m.insert("SYNC", AdbCommand::Sync);
        m.insert("PUSH", AdbCommand::Push);
        m.insert("PULL", AdbCommand::Pull);
        
        // Server commands
        m.insert("VERSION", AdbCommand::Version);
        m.insert("DEVICES", AdbCommand::Devices);
        m.insert("KILL", AdbCommand::Kill);
        m.insert("TRACK_DEVICES", AdbCommand::TrackDevices);
        
        // Transport commands
        m.insert("TRANSPORT", AdbCommand::Transport);
        
        m
    };
}

pub fn format_command(cmd: &str, args: &[&str]) -> String {
    COMMAND_MAP
        .get(cmd.to_uppercase().as_str())
        .map(|command| command.format(args))
        .unwrap_or_else(|| panic!("Unknown ADB command: {}", cmd))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_command_no_args() {
        assert_eq!(format_command("VERSION", &[]), "host:version");
        assert_eq!(format_command("DEVICES", &[]), "host:devices");
        assert_eq!(format_command("KILL", &[]), "host:kill");
        assert_eq!(format_command("SYNC", &[]), "sync:");
    }

    #[test]
    fn test_format_command_with_args() {
        assert_eq!(
            format_command("SHELL", &["ls"]),
            "shell:ls"
        );
        assert_eq!(
            format_command("TRANSPORT", &["device1"]),
            "host:transport:device1"
        );
        assert_eq!(
            format_command("GETPROP_SINGLE", &["ro.product.model"]),
            "shell:getprop ro.product.model"
        );
    }

    #[test]
    #[should_panic(expected = "Unknown ADB command")]
    fn test_format_command_unknown() {
        format_command("INVALID_COMMAND", &[]);
    }
}
