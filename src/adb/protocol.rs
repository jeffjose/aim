use crate::error::{AimError, Result};
use bytes::{Bytes, BytesMut};
use std::convert::TryInto;

// File mode constants
const S_IFMT: u16 = 0o170000; // bit mask for the file type bit field
const S_IFSOCK: u16 = 0o140000; // socket
const S_IFLNK: u16 = 0o120000; // symbolic link
const S_IFREG: u16 = 0o100000; // regular file
const S_IFBLK: u16 = 0o060000; // block device
const S_IFDIR: u16 = 0o040000; // directory
const S_IFCHR: u16 = 0o020000; // character device
const S_IFIFO: u16 = 0o010000; // FIFO

/// ADB wire protocol implementation
pub struct AdbProtocol;

#[derive(Debug, Clone)]
pub struct AdbMessage {
    pub command: String,
    pub arg0: u32,
    pub arg1: u32,
    pub data: Bytes,
}

impl AdbProtocol {
    /// Format a command for the ADB protocol
    pub fn format_command(device_id: Option<&str>, command: &str) -> String {
        if let Some(id) = device_id {
            format!("host-serial:{}:{}", id, command)
        } else {
            format!("host:{}", command)
        }
    }
    
    /// Encode a message for transmission
    pub fn encode_message(msg: &AdbMessage) -> BytesMut {
        let mut buf = BytesMut::new();
        
        // Add command (4 bytes)
        buf.extend_from_slice(msg.command.as_bytes());
        
        // Add arg0 and arg1 (4 bytes each, little-endian)
        buf.extend_from_slice(&msg.arg0.to_le_bytes());
        buf.extend_from_slice(&msg.arg1.to_le_bytes());
        
        // Add data length and magic (4 bytes each)
        let data_len = msg.data.len() as u32;
        buf.extend_from_slice(&data_len.to_le_bytes());
        let magic = !data_len; // Inverted for verification
        buf.extend_from_slice(&magic.to_le_bytes());
        
        // Add data
        buf.extend_from_slice(&msg.data);
        
        buf
    }
    
    /// Decode a message from bytes
    pub fn decode_message(data: &[u8]) -> Result<AdbMessage> {
        if data.len() < 24 {
            return Err(AimError::AdbProtocol("Message too short".into()));
        }
        
        let command = String::from_utf8_lossy(&data[0..4]).to_string();
        let arg0 = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let arg1 = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let data_len = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        let magic = u32::from_le_bytes(data[16..20].try_into().unwrap());
        
        // Verify magic
        if magic != !data_len as u32 {
            return Err(AimError::AdbProtocol("Invalid magic number".into()));
        }
        
        let message_data = if data_len > 0 && data.len() >= 24 + data_len {
            Bytes::copy_from_slice(&data[24..24 + data_len])
        } else {
            Bytes::new()
        };
        
        Ok(AdbMessage {
            command,
            arg0,
            arg1,
            data: message_data,
        })
    }
}

/// File metadata from lstat response
#[derive(Debug, Clone)]
pub struct AdbLstatResponse {
    magic: [u8; 4],
    metadata: FileMetadata,
    timestamps: FileTimestamps,
}

#[derive(Debug, Clone)]
struct FileMetadata {
    unknown1: u32,
    dev_major: u16,
    dev_minor: u16,
    unknown2: u32,
    inode: u32,
    unknown3: u32,
    mode: u16,
    unknown4: u16,
    nlink: u32,
    uid: u32,
    gid: u32,
    size: u32,
    unknown5: u32,
}

#[derive(Debug, Clone)]
struct FileTimestamps {
    atime: FileTimestamp,
    mtime: FileTimestamp,
    ctime: FileTimestamp,
}

#[derive(Debug, Clone)]
struct FileTimestamp {
    seconds: u32,
    nanoseconds: u32,
}

impl AdbLstatResponse {
    /// Parse lstat response from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 72 {
            return Err(AimError::AdbProtocol("Invalid lstat response length".into()));
        }
        
        let magic = &bytes[0..4];
        if magic != b"LST2" && magic != b"DNT2" {
            return Err(AimError::AdbProtocol(
                format!("Invalid magic number: {:?}", String::from_utf8_lossy(magic))
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
    
    // Accessor methods
    pub fn magic(&self) -> &[u8; 4] {
        &self.magic
    }
    
    pub fn device_id(&self) -> u32 {
        ((self.metadata.dev_major as u32) << 8) | (self.metadata.dev_minor as u32)
    }
    
    pub fn mode(&self) -> u16 {
        self.metadata.mode
    }
    
    pub fn size(&self) -> u32 {
        self.metadata.size
    }
    
    pub fn uid(&self) -> u32 {
        self.metadata.uid
    }
    
    pub fn gid(&self) -> u32 {
        self.metadata.gid
    }
    
    pub fn mtime(&self) -> u32 {
        self.timestamps.mtime.seconds
    }
    
    pub fn is_dir(&self) -> bool {
        (self.metadata.mode & S_IFMT) == S_IFDIR
    }
    
    pub fn is_file(&self) -> bool {
        (self.metadata.mode & S_IFMT) == S_IFREG
    }
    
    pub fn is_link(&self) -> bool {
        (self.metadata.mode & S_IFMT) == S_IFLNK
    }
    
    pub fn file_type(&self) -> &'static str {
        match self.metadata.mode & S_IFMT {
            S_IFSOCK => "socket",
            S_IFLNK => "symlink",
            S_IFREG => "file",
            S_IFBLK => "block",
            S_IFDIR => "directory",
            S_IFCHR => "char",
            S_IFIFO => "fifo",
            _ => "unknown",
        }
    }
    
    pub fn permissions(&self) -> String {
        let mode = self.metadata.mode & 0o777;
        format!("{:03o}", mode)
    }
}

/// Sync protocol commands
pub mod sync {
    pub const DATA: &[u8] = b"DATA";
    pub const DONE: &[u8] = b"DONE";
    pub const SEND: &[u8] = b"SEND";
    pub const RECV: &[u8] = b"RECV";
    pub const LIST: &[u8] = b"LIST";
    pub const STAT: &[u8] = b"STAT";
    pub const QUIT: &[u8] = b"QUIT";
}