use crate::core::types::DeviceId;
use crate::error::{AimError, Result};
use log::*;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use tokio::sync::Mutex;

// Constants
const BUFFER_SIZE: usize = 1024;
const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
const RESPONSE_OKAY: &[u8] = b"OKAY";

/// Manages TCP connections to ADB server
pub struct AdbConnection {
    stream: TcpStream,
    device_id: Option<DeviceId>,
}

impl AdbConnection {
    /// Create a new connection to the ADB server
    pub fn new(host: &str, port: u16) -> Result<Self> {
        debug!("=== Creating new ADB connection ===");
        
        let stream = Self::establish_connection(host, port)?;
        
        Ok(Self {
            stream,
            device_id: None,
        })
    }
    
    /// Connect to ADB server with automatic server startup
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        use crate::adb::server::AdbServer;
        
        // Check if server is running
        if !AdbServer::is_running(host, port).await {
            AdbServer::start(port).await?;
            
            // Verify server started
            if !AdbServer::is_running(host, port).await {
                return Err(AimError::Server("Failed to start ADB server".into()));
            }
        }
        
        Self::new(host, port)
    }
    
    fn establish_connection(host: &str, port: u16) -> Result<TcpStream> {
        let server_address = format!(
            "{}:{}",
            if host == "localhost" { "127.0.0.1" } else { host },
            port
        );
        debug!("Connecting to address: {}", server_address);
        
        let mut addresses = server_address
            .to_socket_addrs()
            .map_err(|e| AimError::AdbConnection(e))?;
            
        let address = addresses
            .next()
            .ok_or_else(|| AimError::AdbConnection(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Could not resolve address")
            ))?;
            
        debug!("Resolved address: {:?}", address);
        
        let stream = TcpStream::connect(address)?;
        debug!("Connection established");
        
        stream.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_write_timeout(Some(DEFAULT_TIMEOUT))?;
        debug!("Timeouts set");
        
        Ok(stream)
    }
    
    /// Select a specific device for this connection
    pub async fn select_device(&mut self, device_id: &DeviceId) -> Result<()> {
        let command = format!("host:transport:{}", device_id.as_str());
        self.send_command(&command)?;
        self.read_okay()?;
        self.device_id = Some(device_id.clone());
        Ok(())
    }
    
    /// Send a command to the ADB server
    pub fn send_command(&mut self, command: &str) -> Result<()> {
        debug!("Sending command: {}", command);
        let request = format!("{:04x}{}", command.len(), command);
        debug!("Formatted request: {:?}", request);
        self.write_all(request.as_bytes())
    }
    
    /// Read response from the ADB server
    pub fn read_response(&mut self) -> Result<String> {
        let mut buffer = [0; BUFFER_SIZE];
        let mut response = Vec::new();
        debug!("Waiting for response...");
        
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    debug!("Server closed the connection");
                    break;
                }
                Ok(bytes_read) => {
                    response.extend_from_slice(&buffer[..bytes_read]);
                    // If we read less than buffer size, we're probably done
                    if bytes_read < BUFFER_SIZE {
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!("Would block");
                    break;
                }
                Err(e) => {
                    debug!("Error reading from socket: {}", e);
                    return Err(AimError::AdbConnection(e));
                }
            }
        }
        
        info!("Response: {:?}", response);
        self.process_response(&response)
    }
    
    fn process_response(&self, data: &[u8]) -> Result<String> {
        debug!("Raw bytes length: {}", data.len());
        match std::str::from_utf8(data) {
            Ok(s) => {
                debug!("UTF-8 response length: {}", s.len());
                Ok(s.to_string())
            }
            Err(_) => {
                // For binary data, convert to hex string
                let hex = data
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                debug!("Binary response (hex) length: {}", hex.len());
                Ok(hex)
            }
        }
    }
    
    /// Read and verify OKAY response
    pub fn read_okay(&mut self) -> Result<()> {
        let mut response = [0u8; 4];
        self.stream.read_exact(&mut response)?;
        debug!("Response in read_okay: {:?}", response);
        
        match response {
            [b'O', b'K', b'A', b'Y'] => Ok(()),
            [b'F', b'A', b'I', b'L'] => {
                // Read failure message
                let mut len_bytes = [0u8; 4];
                self.stream.read_exact(&mut len_bytes)?;
                let len = u32::from_be_bytes(len_bytes) as usize;
                let mut msg = vec![0u8; len];
                self.stream.read_exact(&mut msg)?;
                let error_msg = String::from_utf8_lossy(&msg);
                Err(AimError::AdbProtocol(format!("Command failed: {}", error_msg)))
            }
            _ => Err(AimError::AdbProtocol(
                format!("Expected OKAY response. Got {:?}", response)
            ))
        }
    }
    
    /// Write all bytes to the connection
    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.stream.write_all(buf)?;
        Ok(())
    }
    
    /// Read exact number of bytes
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.stream.read_exact(buf)?;
        Ok(())
    }
    
    /// Get the underlying stream (for advanced operations)
    pub fn stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }
    
    /// Check if a device is selected
    pub fn has_device(&self) -> bool {
        self.device_id.is_some()
    }
    
    /// Get the selected device ID
    pub fn device_id(&self) -> Option<&DeviceId> {
        self.device_id.as_ref()
    }
}

/// Connection pool for reusing ADB connections
pub struct ConnectionPool {
    connections: Arc<Mutex<Vec<AdbConnection>>>,
    host: String,
    port: u16,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            host: host.into(),
            port,
        }
    }
    
    /// Get a connection from the pool or create a new one
    pub async fn get(&self) -> Result<AdbConnection> {
        let mut pool = self.connections.lock().await;
        
        if let Some(conn) = pool.pop() {
            Ok(conn)
        } else {
            AdbConnection::connect(&self.host, self.port).await
        }
    }
    
    /// Return a connection to the pool
    pub async fn return_connection(&self, conn: AdbConnection) {
        let mut pool = self.connections.lock().await;
        pool.push(conn);
    }
    
    /// Clear all connections from the pool
    pub async fn clear(&self) {
        let mut pool = self.connections.lock().await;
        pool.clear();
    }
}