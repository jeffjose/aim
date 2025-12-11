use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

// =============================================================================
// File type constants (from stat.h)
// =============================================================================

const S_IFMT: u16 = 0o170000;   // bit mask for the file type bit field
const S_IFSOCK: u16 = 0o140000; // socket
const S_IFLNK: u16 = 0o120000;  // symbolic link
const S_IFREG: u16 = 0o100000;  // regular file
const S_IFBLK: u16 = 0o060000;  // block device
const S_IFDIR: u16 = 0o040000;  // directory
const S_IFCHR: u16 = 0o020000;  // character device
const S_IFIFO: u16 = 0o010000;  // FIFO

// =============================================================================
// ADB Lstat Response (LST2/DNT2 protocol)
// =============================================================================

/// Response from ADB lstat/directory listing commands
#[derive(Debug)]
pub struct AdbLstatResponse {
    magic: [u8; 4],
    metadata: FileMetadata,
    timestamps: FileTimestamps,
}

#[derive(Debug)]
struct FileMetadata {
    #[allow(dead_code)]
    unknown1: u32,
    dev_major: u16,
    dev_minor: u16,
    #[allow(dead_code)]
    unknown2: u32,
    #[allow(dead_code)]
    inode: u32,
    #[allow(dead_code)]
    unknown3: u32,
    mode: u16,
    #[allow(dead_code)]
    unknown4: u16,
    #[allow(dead_code)]
    nlink: u32,
    #[allow(dead_code)]
    uid: u32,
    #[allow(dead_code)]
    gid: u32,
    size: u32,
    #[allow(dead_code)]
    unknown5: u32,
}

#[derive(Debug)]
struct FileTimestamps {
    atime: FileTimestamp,
    mtime: FileTimestamp,
    ctime: FileTimestamp,
}

#[derive(Debug)]
struct FileTimestamp {
    seconds: u32,
    nanoseconds: u32,
}

impl AdbLstatResponse {
    /// Parse lstat response from 72-byte buffer
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 72 {
            return Err("Invalid byte array length".to_string());
        }

        let magic = &bytes[0..4];
        if magic != b"LST2" && magic != b"DNT2" {
            return Err(format!(
                "Invalid magic number: {:?} (expected LST2 or DNT2)",
                String::from_utf8_lossy(magic)
            ));
        }

        let metadata = FileMetadata {
            unknown1: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            dev_major: u16::from_le_bytes(bytes[8..10].try_into().unwrap()),
            dev_minor: u16::from_le_bytes(bytes[10..12].try_into().unwrap()),
            unknown2: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
            inode: u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            unknown3: u32::from_le_bytes(bytes[20..24].try_into().unwrap()),
            mode: u16::from_le_bytes(bytes[24..26].try_into().unwrap()),
            unknown4: u16::from_le_bytes(bytes[26..28].try_into().unwrap()),
            nlink: u32::from_le_bytes(bytes[28..32].try_into().unwrap()),
            uid: u32::from_le_bytes(bytes[32..36].try_into().unwrap()),
            gid: u32::from_le_bytes(bytes[36..40].try_into().unwrap()),
            size: u32::from_le_bytes(bytes[40..44].try_into().unwrap()),
            unknown5: u32::from_le_bytes(bytes[44..48].try_into().unwrap()),
        };

        let timestamps = FileTimestamps {
            atime: FileTimestamp {
                seconds: u32::from_le_bytes(bytes[48..52].try_into().unwrap()),
                nanoseconds: u32::from_le_bytes(bytes[52..56].try_into().unwrap()),
            },
            mtime: FileTimestamp {
                seconds: u32::from_le_bytes(bytes[56..60].try_into().unwrap()),
                nanoseconds: u32::from_le_bytes(bytes[60..64].try_into().unwrap()),
            },
            ctime: FileTimestamp {
                seconds: u32::from_le_bytes(bytes[64..68].try_into().unwrap()),
                nanoseconds: u32::from_le_bytes(bytes[68..72].try_into().unwrap()),
            },
        };

        Ok(Self {
            magic: bytes[0..4].try_into().unwrap(),
            metadata,
            timestamps,
        })
    }

    pub fn magic(&self) -> &[u8; 4] {
        &self.magic
    }

    pub fn device_id(&self) -> u32 {
        ((self.metadata.dev_major as u32) << 8) | (self.metadata.dev_minor as u32)
    }

    #[allow(dead_code)]
    pub fn mode(&self) -> u16 {
        self.metadata.mode
    }

    pub fn size(&self) -> u32 {
        self.metadata.size
    }

    pub fn file_type(&self) -> &'static str {
        match self.metadata.mode & S_IFMT {
            S_IFIFO => "Named pipe (fifo)",
            S_IFCHR => "Character device",
            S_IFDIR => "Directory",
            S_IFBLK => "Block device",
            S_IFREG => "Regular file",
            S_IFLNK => "Symbolic link",
            S_IFSOCK => "Socket",
            _ => "Unknown",
        }
    }

    #[allow(dead_code)]
    pub fn is_directory(&self) -> bool {
        (self.metadata.mode & S_IFMT) == S_IFDIR
    }

    pub fn permissions_string(&self) -> String {
        let mode = self.metadata.mode;
        let owner = Self::permission_triplet_string(mode >> 6);
        let group = Self::permission_triplet_string(mode >> 3);
        let others = Self::permission_triplet_string(mode);

        let file_type = match mode & S_IFMT {
            S_IFIFO => "p",
            S_IFCHR => "c",
            S_IFDIR => "d",
            S_IFBLK => "b",
            S_IFREG => "-",
            S_IFLNK => "l",
            S_IFSOCK => "s",
            _ => "?",
        };

        format!("{}{}{}{}", file_type, owner, group, others)
    }

    fn permission_triplet_string(mode: u16) -> String {
        let mut triplet = String::with_capacity(3);
        triplet.push(if (mode & 4) != 0 { 'r' } else { '-' });
        triplet.push(if (mode & 2) != 0 { 'w' } else { '-' });
        triplet.push(match (mode & 1, mode & 0o7000) {
            (0, 0) => '-',
            (1, 0) => 'x',
            (0, 0o4000) => 'S',
            (1, 0o4000) => 's',
            (0, 0o2000) => 'S',
            (1, 0o2000) => 's',
            (0, 0o1000) => 'T',
            (1, 0o1000) => 't',
            _ => '?',
        });
        triplet
    }
}

impl fmt::Display for AdbLstatResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AdbLstatResponse {{\n\
             \tmagic: {:?}\n\
             \tdevice_id: {}\n\
             \tfile_type: {}\n\
             \tpermissions: {}\n\
             \tsize: {}\n\
             \tatime: {}.{:09}\n\
             \tmtime: {}.{:09}\n\
             \tctime: {}.{:09}\n\
             }}",
            self.magic(),
            self.device_id(),
            self.file_type(),
            self.permissions_string(),
            self.size(),
            self.timestamps.atime.seconds,
            self.timestamps.atime.nanoseconds,
            self.timestamps.mtime.seconds,
            self.timestamps.mtime.nanoseconds,
            self.timestamps.ctime.seconds,
            self.timestamps.ctime.nanoseconds,
        )
    }
}

// =============================================================================
// Progress Display
// =============================================================================

#[derive(Clone, Copy, Debug)]
pub enum ProgressDisplay {
    Show,
    #[allow(dead_code)]
    Hide,
}

impl Default for ProgressDisplay {
    fn default() -> Self {
        ProgressDisplay::Show
    }
}

// =============================================================================
// ADB Commands
// =============================================================================

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
