use crate::core::types::{DeviceId, TransferDirection, TransferProgress as Progress};
use crate::error::{AimError, Result};
use crate::adb::connection::AdbConnection;
use crate::adb::protocol::{AdbLstatResponse, sync};
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const CHUNK_SIZE: usize = 64 * 1024;
const SYNC_DATA: &[u8] = sync::DATA;
const SYNC_DONE: &[u8] = sync::DONE;
const SYNC_SEND: &[u8] = sync::SEND;
const SYNC_RECV: &[u8] = sync::RECV;
const SYNC_STAT: &[u8] = sync::STAT;

/// File transfer operations
pub struct FileTransfer {
    conn: AdbConnection,
    progress_bar: Option<ProgressBar>,
}

impl FileTransfer {
    /// Create a new file transfer instance
    pub async fn new(host: &str, port: u16, device_id: Option<&DeviceId>) -> Result<Self> {
        let mut conn = AdbConnection::connect(host, port).await?;
        
        // Select device if specified
        if let Some(id) = device_id {
            conn.select_device(id).await?;
        }
        
        // Enter sync mode
        conn.send_command("sync:")?;
        conn.read_okay()?;
        
        Ok(Self {
            conn,
            progress_bar: None,
        })
    }
    
    /// Enable progress reporting
    pub fn with_progress(mut self, total_size: u64) -> Self {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        self.progress_bar = Some(pb);
        self
    }
    
    /// Push a file to the device
    pub async fn push(&mut self, local_path: &Path, remote_path: &str) -> Result<()> {
        info!("Pushing {} to {}", local_path.display(), remote_path);
        
        // Get file metadata
        let metadata = fs::metadata(local_path)
            .map_err(|e| AimError::FileTransfer(format!("Cannot read local file: {}", e)))?;
            
        if !metadata.is_file() {
            return Err(AimError::FileTransfer("Can only push regular files".into()));
        }
        
        let file_size = metadata.len();
        let permissions = get_permissions(&metadata);
        
        // Send SEND command
        self.send_sync_command(SYNC_SEND, remote_path)?;
        
        // Open file for reading
        let mut file = File::open(local_path)
            .map_err(|e| AimError::FileTransfer(format!("Cannot open file: {}", e)))?;
            
        // Transfer file data
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut bytes_sent = 0u64;
        
        loop {
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| AimError::FileTransfer(format!("Read error: {}", e)))?;
                
            if bytes_read == 0 {
                break;
            }
            
            // Send DATA chunk
            self.send_data_chunk(&buffer[..bytes_read])?;
            bytes_sent += bytes_read as u64;
            
            // Update progress
            if let Some(ref pb) = self.progress_bar {
                pb.set_position(bytes_sent);
            }
        }
        
        // Send DONE
        self.send_done(metadata.modified()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32)
            .unwrap_or(0))?;
            
        // Read response
        self.read_sync_response()?;
        
        // Finish progress
        if let Some(ref pb) = self.progress_bar {
            pb.finish_with_message("Complete");
        }
        
        info!("Successfully pushed {}", local_path.display());
        Ok(())
    }
    
    /// Pull a file from the device
    pub async fn pull(&mut self, remote_path: &str, local_path: &Path) -> Result<()> {
        info!("Pulling {} to {}", remote_path, local_path.display());
        
        // Get remote file info
        let stat = self.stat(remote_path).await?;
        if !stat.is_file() {
            return Err(AimError::FileTransfer("Can only pull regular files".into()));
        }
        
        let file_size = stat.size() as u64;
        
        // Send RECV command
        self.send_sync_command(SYNC_RECV, remote_path)?;
        
        // Create local file
        let mut file = File::create(local_path)
            .map_err(|e| AimError::FileTransfer(format!("Cannot create file: {}", e)))?;
            
        // Receive file data
        let mut bytes_received = 0u64;
        
        loop {
            let (cmd, data) = self.read_sync_packet()?;
            
            match &cmd {
                b"DATA" => {
                    file.write_all(&data)
                        .map_err(|e| AimError::FileTransfer(format!("Write error: {}", e)))?;
                    bytes_received += data.len() as u64;
                    
                    // Update progress
                    if let Some(ref pb) = self.progress_bar {
                        pb.set_position(bytes_received);
                    }
                }
                b"DONE" => {
                    break;
                }
                b"FAIL" => {
                    let error_msg = String::from_utf8_lossy(&data);
                    return Err(AimError::FileTransfer(format!("Pull failed: {}", error_msg)));
                }
                _ => {
                    return Err(AimError::FileTransfer(format!("Unexpected response: {:?}", cmd)));
                }
            }
        }
        
        // Set file permissions if on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = (stat.mode() & 0o777) as u32;
            fs::set_permissions(local_path, fs::Permissions::from_mode(mode))?;
        }
        
        // Finish progress
        if let Some(ref pb) = self.progress_bar {
            pb.finish_with_message("Complete");
        }
        
        info!("Successfully pulled {}", remote_path);
        Ok(())
    }
    
    /// Get file statistics
    pub async fn stat(&mut self, remote_path: &str) -> Result<AdbLstatResponse> {
        self.send_sync_command(SYNC_STAT, remote_path)?;
        
        let mut response = [0u8; 72];
        self.conn.read_exact(&mut response)?;
        
        AdbLstatResponse::from_bytes(&response)
    }
    
    /// Send a sync command
    fn send_sync_command(&mut self, command: &[u8], path: &str) -> Result<()> {
        debug!("Sending sync command: {:?} {}", command, path);
        
        // Write command (4 bytes)
        self.conn.write_all(command)?;
        
        // Write path length (4 bytes, little-endian)
        let path_bytes = path.as_bytes();
        let len = path_bytes.len() as u32;
        self.conn.write_all(&len.to_le_bytes())?;
        
        // Write path
        self.conn.write_all(path_bytes)?;
        
        Ok(())
    }
    
    /// Send a data chunk
    fn send_data_chunk(&mut self, data: &[u8]) -> Result<()> {
        // Write DATA command
        self.conn.write_all(SYNC_DATA)?;
        
        // Write data length
        let len = data.len() as u32;
        self.conn.write_all(&len.to_le_bytes())?;
        
        // Write data
        self.conn.write_all(data)?;
        
        Ok(())
    }
    
    /// Send DONE command
    fn send_done(&mut self, mtime: u32) -> Result<()> {
        // Write DONE command
        self.conn.write_all(SYNC_DONE)?;
        
        // Write mtime
        self.conn.write_all(&mtime.to_le_bytes())?;
        
        Ok(())
    }
    
    /// Read sync response
    fn read_sync_response(&mut self) -> Result<()> {
        let mut response = [0u8; 8];
        self.conn.read_exact(&mut response)?;
        
        let cmd = &response[0..4];
        let len = u32::from_le_bytes(response[4..8].try_into().unwrap());
        
        match cmd {
            b"OKAY" => Ok(()),
            b"FAIL" => {
                let mut error_msg = vec![0u8; len as usize];
                self.conn.read_exact(&mut error_msg)?;
                let msg = String::from_utf8_lossy(&error_msg);
                Err(AimError::FileTransfer(format!("Transfer failed: {}", msg)))
            }
            _ => Err(AimError::FileTransfer(format!("Unexpected response: {:?}", cmd)))
        }
    }
    
    /// Read a sync packet
    fn read_sync_packet(&mut self) -> Result<([u8; 4], Vec<u8>)> {
        let mut header = [0u8; 8];
        self.conn.read_exact(&mut header)?;
        
        let cmd = [header[0], header[1], header[2], header[3]];
        let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        
        let mut data = vec![0u8; len];
        if len > 0 {
            self.conn.read_exact(&mut data)?;
        }
        
        Ok((cmd, data))
    }
}

/// Progress tracking for file transfers
pub struct TransferProgress {
    direction: TransferDirection,
    progress: Progress,
    callback: Option<Box<dyn Fn(&Progress) + Send>>,
}

impl TransferProgress {
    /// Create a new progress tracker
    pub fn new(direction: TransferDirection, file_path: String, total_bytes: u64) -> Self {
        Self {
            direction,
            progress: Progress {
                bytes_transferred: 0,
                total_bytes,
                file_path,
            },
            callback: None,
        }
    }
    
    /// Set progress callback
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Progress) + Send + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }
    
    /// Update progress
    pub fn update(&mut self, bytes: u64) {
        self.progress.bytes_transferred = bytes;
        if let Some(ref callback) = self.callback {
            callback(&self.progress);
        }
    }
    
    /// Get current progress
    pub fn progress(&self) -> &Progress {
        &self.progress
    }
}

/// Get Unix permissions from metadata
#[cfg(unix)]
fn get_permissions(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn get_permissions(_metadata: &fs::Metadata) -> u32 {
    0o644 // Default permissions for non-Unix systems
}