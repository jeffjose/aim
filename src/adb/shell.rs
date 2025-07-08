use crate::core::types::DeviceId;
use crate::error::{AimError, Result};
use crate::adb::connection::AdbConnection;
use log::*;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Shell command execution
pub struct ShellCommand {
    command: String,
    device_id: Option<DeviceId>,
}

impl ShellCommand {
    /// Create a new shell command
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            device_id: None,
        }
    }
    
    /// Target a specific device
    pub fn with_device(mut self, device_id: DeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }
    
    /// Execute the command and return output
    pub async fn execute(&self, host: &str, port: u16) -> Result<ShellOutput> {
        let mut conn = AdbConnection::connect(host, port).await?;
        
        // Select device if specified
        if let Some(device_id) = &self.device_id {
            conn.select_device(device_id).await?;
        }
        
        // Send shell command
        let shell_cmd = format!("shell:{}", self.command);
        conn.send_command(&shell_cmd)?;
        conn.read_okay()?;
        
        // Read response
        let response = conn.read_response()?;
        let output = Self::clean_response(&response);
        
        Ok(ShellOutput {
            stdout: output,
            stderr: String::new(),
            exit_code: 0,
        })
    }
    
    /// Execute command asynchronously with streaming output
    pub async fn execute_streaming<F>(&self, host: &str, port: u16, mut callback: F) -> Result<()>
    where
        F: FnMut(&str) + Send + 'static,
    {
        let mut conn = AdbConnection::connect(host, port).await?;
        
        // Select device if specified
        if let Some(device_id) = &self.device_id {
            conn.select_device(device_id).await?;
        }
        
        // Send shell command
        let shell_cmd = format!("shell:{}", self.command);
        conn.send_command(&shell_cmd)?;
        conn.read_okay()?;
        
        // Stream output
        let mut buffer = vec![0u8; 4096];
        let stream = conn.stream();
        
        // Convert to async stream
        stream.set_nonblocking(true)?;
        let mut async_stream = TcpStream::from_std(stream.try_clone()?)?;
        
        loop {
            match async_stream.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    callback(&chunk);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(e) => return Err(AimError::Shell(format!("Stream error: {}", e))),
            }
        }
        
        Ok(())
    }
    
    /// Clean shell command response
    fn clean_response(response: &str) -> String {
        // Remove null bytes and trim
        response
            .replace('\0', "")
            .trim()
            .to_string()
    }
}

/// Shell command output
#[derive(Debug, Clone)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl ShellOutput {
    /// Check if command succeeded
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
    
    /// Get combined output
    pub fn output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }
}

/// Helper functions for common shell operations
pub mod helpers {
    use super::*;
    
    /// Get device property
    pub async fn getprop(
        host: &str,
        port: u16,
        device_id: Option<&DeviceId>,
        property: &str,
    ) -> Result<String> {
        let mut cmd = ShellCommand::new(format!("getprop {}", property));
        if let Some(id) = device_id {
            cmd = cmd.with_device(id.clone());
        }
        
        let output = cmd.execute(host, port).await?;
        Ok(output.stdout.trim().to_string())
    }
    
    /// Get all device properties
    pub async fn getprops(
        host: &str,
        port: u16,
        device_id: Option<&DeviceId>,
    ) -> Result<Vec<(String, String)>> {
        let mut cmd = ShellCommand::new("getprop");
        if let Some(id) = device_id {
            cmd = cmd.with_device(id.clone());
        }
        
        let output = cmd.execute(host, port).await?;
        let mut props = Vec::new();
        
        for line in output.stdout.lines() {
            if let Some((key, value)) = parse_property_line(line) {
                props.push((key, value));
            }
        }
        
        Ok(props)
    }
    
    /// Parse a property line from getprop output
    fn parse_property_line(line: &str) -> Option<(String, String)> {
        let line = line.trim();
        if !line.starts_with('[') || !line.contains(']') {
            return None;
        }
        
        let close_bracket = line.find(']')?;
        let key = line[1..close_bracket].to_string();
        
        let value_part = &line[close_bracket + 1..].trim();
        if value_part.starts_with(": [") && value_part.ends_with(']') {
            let value = value_part[3..value_part.len() - 1].to_string();
            Some((key, value))
        } else {
            None
        }
    }
    
    /// Run a command and return exit code
    pub async fn run_with_exit_code(
        host: &str,
        port: u16,
        device_id: Option<&DeviceId>,
        command: &str,
    ) -> Result<i32> {
        // Run command followed by echo $?
        let cmd_with_exit = format!("{}; echo \"EXIT_CODE:$?\"", command);
        let mut cmd = ShellCommand::new(cmd_with_exit);
        if let Some(id) = device_id {
            cmd = cmd.with_device(id.clone());
        }
        
        let output = cmd.execute(host, port).await?;
        
        // Parse exit code from output
        for line in output.stdout.lines().rev() {
            if let Some(code_str) = line.strip_prefix("EXIT_CODE:") {
                if let Ok(code) = code_str.trim().parse::<i32>() {
                    return Ok(code);
                }
            }
        }
        
        Ok(0) // Default to success if we can't parse
    }
}