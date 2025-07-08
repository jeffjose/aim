use crate::error::{AimError, Result};
use log::*;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

const SERVER_START_DELAY: Duration = Duration::from_secs(1);
const SERVER_CHECK_TIMEOUT: Duration = Duration::from_millis(500);

/// ADB server management
pub struct AdbServer;

#[allow(dead_code)]
impl AdbServer {
    /// Start the ADB server
    pub async fn start(port: u16) -> Result<()> {
        info!("Starting ADB server on port {}", port);
        
        let adb_command = std::env::var("ADB_PATH").unwrap_or_else(|_| "adb".to_string());
        
        let output = Command::new(&adb_command)
            .args(&["-P", &port.to_string(), "start-server"])
            .output()
            .map_err(|e| AimError::Server(format!("Failed to execute adb command: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AimError::Server(format!("Failed to start ADB server: {}", stderr)));
        }
        
        info!("ADB server started successfully");
        
        // Give the server time to fully start
        sleep(SERVER_START_DELAY).await;
        
        Ok(())
    }
    
    /// Stop the ADB server
    pub async fn stop(port: u16) -> Result<()> {
        info!("Stopping ADB server on port {}", port);
        
        let adb_command = std::env::var("ADB_PATH").unwrap_or_else(|_| "adb".to_string());
        
        let output = Command::new(&adb_command)
            .args(&["-P", &port.to_string(), "kill-server"])
            .output()
            .map_err(|e| AimError::Server(format!("Failed to execute adb command: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AimError::Server(format!("Failed to stop ADB server: {}", stderr)));
        }
        
        info!("ADB server stopped successfully");
        Ok(())
    }
    
    /// Restart the ADB server
    pub async fn restart(port: u16) -> Result<()> {
        Self::stop(port).await?;
        sleep(Duration::from_millis(500)).await;
        Self::start(port).await
    }
    
    /// Check if the ADB server is running
    pub async fn is_running(host: &str, port: u16) -> bool {
        use std::net::TcpStream;
        
        let address = format!("{}:{}", host, port);
        let addr_for_log = address.clone();
        
        match tokio::time::timeout(
            SERVER_CHECK_TIMEOUT,
            tokio::task::spawn_blocking(move || TcpStream::connect(address))
        ).await {
            Ok(Ok(Ok(_))) => {
                debug!("ADB server is running at {}", addr_for_log);
                true
            }
            _ => {
                debug!("ADB server is not running at {}", addr_for_log);
                false
            }
        }
    }
    
    /// Get server version
    pub async fn version(host: &str, port: u16) -> Result<String> {
        use crate::adb::connection::AdbConnection;
        
        let mut conn = AdbConnection::new(host, port)?;
        conn.send_command("host:version")?;
        conn.read_okay()?;
        
        let mut version_bytes = [0u8; 4];
        conn.read_exact(&mut version_bytes)?;
        let version = u32::from_be_bytes(version_bytes);
        
        Ok(format!("{:04x}", version))
    }
    
    /// Get list of devices from server
    pub async fn list_devices(host: &str, port: u16) -> Result<String> {
        use crate::adb::connection::AdbConnection;
        
        let mut conn = AdbConnection::new(host, port)?;
        conn.send_command("host:devices-l")?;
        conn.read_okay()?;
        
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        conn.read_exact(&mut len_bytes)?;
        let len = u32::from_str_radix(std::str::from_utf8(&len_bytes)?, 16)
            .map_err(|e| AimError::ParseError(format!("Invalid length prefix: {}", e)))?;
        
        // Read device list
        let mut devices_data = vec![0u8; len as usize];
        conn.read_exact(&mut devices_data)?;
        
        Ok(String::from_utf8_lossy(&devices_data).to_string())
    }
    
    /// Track devices (returns a stream of device changes)
    pub async fn track_devices(host: &str, port: u16) -> Result<crate::adb::connection::AdbConnection> {
        use crate::adb::connection::AdbConnection;
        
        let mut conn = AdbConnection::new(host, port)?;
        conn.send_command("host:track-devices")?;
        conn.read_okay()?;
        
        Ok(conn)
    }
}