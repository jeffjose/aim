//! ADB Operations
//!
//! This module provides the core ADB functionality including:
//! - Connection management (AdbStream)
//! - Shell command execution
//! - File transfer (push/pull)
//! - Server management (start/stop/status)
//!
//! Re-exports protocol types from the protocol module.

use super::protocol::format_command;
use indicatif::ProgressBar;
use log::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::str;
use std::sync::Arc;
use tokio::task::JoinHandle;

// Re-export protocol types for backwards compatibility
pub use super::protocol::{AdbLstatResponse, ProgressDisplay};

// =============================================================================
// Constants
// =============================================================================

const SYNC_DATA: &[u8] = b"SEND";
const SYNC_DONE: &[u8] = b"DONE";
const BUFFER_SIZE: usize = 1024;
const CHUNK_SIZE: usize = 64 * 1024;
const SERVER_START_DELAY: std::time::Duration = std::time::Duration::from_secs(1);
const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);


type AdbResult<T> = Result<T, Box<dyn Error>>;

struct AdbStream {
    stream: TcpStream,
}

impl AdbStream {
    fn new(host: &str, port: &str) -> Result<Self, Box<dyn Error>> {
        debug!("=== Creating new ADB stream ===");
        debug!("AdbStream::new() - host: {}, port: {}", host, port);

        debug!("Ensuring server is running...");
        Self::ensure_server_running(host, port)?;
        debug!("Server check complete");
        
        debug!("Establishing connection...");
        let stream = Self::establish_connection(host, port)?;
        debug!("Connection established successfully");

        Ok(Self { stream })
    }

    fn ensure_server_running(host: &str, port: &str) -> Result<(), Box<dyn Error>> {
        debug!("ensure_server_running: checking if server is already running");
        if !check_server_running(host, port) {
            debug!("Server not running, attempting to start it");
            start_adb_server(port)?;

            debug!("Checking if server started successfully");
            if !check_server_running(host, port) {
                debug!("Server failed to start");
                return Err("Failed to start ADB server".into());
            }
            debug!("Server started successfully");
        } else {
            debug!("Server already running");
        }
        Ok(())
    }

    fn establish_connection(host: &str, port: &str) -> Result<TcpStream, Box<dyn Error>> {
        let server_address = format!(
            "{}:{}",
            if host == "localhost" {
                "127.0.0.1"
            } else {
                host
            },
            port
        );
        debug!("Connecting to address: {}", server_address);

        let mut addresses = server_address.to_socket_addrs()?;
        let address = addresses.next().ok_or("Could not resolve address")?;
        debug!("Resolved address: {:?}", address);

        debug!("Establishing TCP connection to {:?}...", address);
        let stream = match TcpStream::connect(address) {
            Ok(s) => {
                debug!("TCP connection established successfully");
                s
            }
            Err(e) => {
                debug!("Failed to establish TCP connection: {}", e);
                return Err(e.into());
            }
        };

        stream.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_write_timeout(Some(DEFAULT_TIMEOUT))?;
        debug!("Timeouts set");

        Ok(stream)
    }

    fn send_command(&mut self, command: &str) -> AdbResult<()> {
        debug!("Sending command: {}", command);
        let request = format!("{:04x}{}", command.len(), command);
        debug!("Formatted request: {:?}", request);
        self.write_all(request.as_bytes())
    }

    fn read_response(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = [0; BUFFER_SIZE];
        let mut response = Vec::new();
        debug!("read_response: start - waiting for response...");

        loop {
            debug!("read_response: attempting to read up to {} bytes", BUFFER_SIZE);
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    debug!("read_response: server closed the connection");
                    break;
                }
                Ok(bytes_read) => {
                    debug!("read_response: read {} bytes", bytes_read);
                    debug!("read_response: first 20 bytes: {:?}", &buffer[..bytes_read.min(20)]);
                    response.extend_from_slice(&buffer[..bytes_read]);
                    // If we read less than buffer size, we're probably done
                    if bytes_read < BUFFER_SIZE {
                        debug!("read_response: read less than buffer size, assuming done");
                        break;
                    }
                    debug!("read_response: read full buffer, continuing...");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!("Would block");
                    break;
                }
                Err(e) => {
                    debug!("Error reading from socket: {}", e);
                    return Err(e.into());
                }
            }
        }

        debug!("Response bytes: {:?}", response);
        self.process_response(&response)
    }

    fn process_response(&self, data: &[u8]) -> Result<String, Box<dyn Error>> {
        debug!("Raw bytes length: {}", data.len());
        match str::from_utf8(data) {
            Ok(s) => {
                debug!("UTF-8 response length: {}", s.len());
                Ok(s.to_string())
            }
            Err(_) => {
                let hex = data
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                debug!("Binary response (hex) length: {}", hex.len());
                Ok(hex)
            }
        }
    }

    fn read_okay(&mut self) -> AdbResult<()> {
        let mut response = [0u8; 4];
        self.stream.read_exact(&mut response)?;
        debug!("Response in read_okay: {:?}", response);

        match response {
            [b'O', b'K', b'A', b'Y'] => Ok(()),
            [n, 0, 0, 0] if n != 0 => Ok(()),  // Accept any non-zero first byte
            _ => Err(format!("Expected OKAY response or status code. Got {:?}", response).into())
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Box<dyn Error>> {
        self.stream.write_all(buf)?;
        Ok(())
    }

    fn write_length_prefixed(&mut self, data: &[u8]) -> AdbResult<()> {
        self.write_all(&(data.len() as u32).to_le_bytes())?;
        self.write_all(data)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn stat(&mut self, path: &PathBuf) -> Result<AdbLstatResponse, Box<dyn Error>> {
        let path_str = path.to_string_lossy();
        let path_bytes = path_str.as_bytes();
        let mut command = Vec::with_capacity(4 + 4 + path_bytes.len());
        command.extend_from_slice(b"LST2");
        command.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
        command.extend_from_slice(path_bytes);
        self.write_all(&command)?;

        let mut response_buf = [0u8; 72];
        self.stream.read_exact(&mut response_buf)?;
        AdbLstatResponse::from_bytes(&response_buf)
            .map_err(|e| format!("Failed to parse stat response: {}", e).into())
    }

    fn transfer_file(
        &mut self,
        src_path: &PathBuf,
        dst_path: &str,
        perms: u32,
        progress: ProgressDisplay,
    ) -> Result<(), Box<dyn Error>> {
        // Send SEND command with path and mode
        debug!("Sending SEND command...");
        self.write_all(SYNC_DATA)?;
        let path_header = format!("{},{}", dst_path, perms);
        self.write_length_prefixed(path_header.as_bytes())?;

        // Open and get file info
        let mut file = File::open(src_path)?;
        let file_size = fs::metadata(src_path)?.len();
        let mut buffer = [0u8; CHUNK_SIZE];
        let mut total_bytes = 0;

        // Setup progress bar if enabled
        let pb = match progress {
            ProgressDisplay::Show => Some(ProgressBar::new(file_size)),
            ProgressDisplay::Hide => None,
        };

        if let Some(pb) = &pb {
            pb.set_style(indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) ({eta})")
                .unwrap()
                .progress_chars("#>-"));
        }

        let transfer_start = std::time::Instant::now();
        let mut chunk_start;

        // Transfer file data
        loop {
            chunk_start = std::time::Instant::now();
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            total_bytes += bytes_read;

            self.write_all(b"DATA")?;
            self.write_all(&(bytes_read as u32).to_le_bytes())?;
            self.write_all(&buffer[..bytes_read])?;

            let chunk_duration = chunk_start.elapsed();
            let chunk_speed = bytes_read as f64 / chunk_duration.as_secs_f64() / 1024.0 / 1024.0;
            if let Some(pb) = &pb {
                pb.set_message(format!("{:.2} MB/s", chunk_speed));
                pb.set_position(total_bytes as u64);
            }
        }

        // Send DONE command with file modification time
        debug!("Sending DONE command...");
        self.write_all(SYNC_DONE)?;
        let mtime = fs::metadata(src_path)?.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as u32;
        self.write_all(&mtime.to_le_bytes())?;

        // Show final statistics if progress bar was enabled
        if let Some(pb) = pb {
            let total_duration = transfer_start.elapsed();
            let avg_speed = total_bytes as f64 / total_duration.as_secs_f64() / 1024.0 / 1024.0;
            pb.finish_with_message(format!(
                "Transfer completed in {:.2}s at {:.2} MB/s average",
                total_duration.as_secs_f64(),
                avg_speed
            ));
        }

        Ok(())
    }

    fn transfer_data(
        &mut self,
        dst_path: &PathBuf,
        file_size: u64,
        description: &str,
        progress: ProgressDisplay,
    ) -> Result<(), Box<dyn Error>> {
        // Create parent directory if needed
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Open destination file
        debug!("Creating file: {:?}", dst_path);
        let mut file = File::create(dst_path)?;
        let mut total_bytes = 0;

        // Setup progress bar if enabled
        let pb = match progress {
            ProgressDisplay::Show => Some(ProgressBar::new(file_size)),
            ProgressDisplay::Hide => None,
        };

        if let Some(pb) = &pb {
            pb.set_style(indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) ({eta})")
                .unwrap()
                .progress_chars("#>-"));
        }

        let transfer_start = std::time::Instant::now();
        let mut chunk_start;

        debug!("Transferring {}...", description);
        loop {
            let mut response = [0u8; 4];
            if self.stream.read_exact(&mut response).is_err() {
                break;
            }

            match &response {
                b"DATA" => {
                    // Handle the data transfer
                    let mut len_bytes = [0u8; 4];
                    self.stream.read_exact(&mut len_bytes)?;
                    let len = u32::from_le_bytes(len_bytes) as usize;

                    chunk_start = std::time::Instant::now();
                    let mut buffer = vec![0u8; len];
                    self.stream.read_exact(&mut buffer)?;
                    file.write_all(&buffer)?;
                    total_bytes += len;

                    let chunk_duration = chunk_start.elapsed();
                    let chunk_speed = len as f64 / chunk_duration.as_secs_f64() / 1024.0 / 1024.0;
                    if let Some(pb) = &pb {
                        pb.set_message(format!("{:.2} MB/s", chunk_speed));
                        pb.set_position(total_bytes as u64);
                    }
                }
                b"DNT2" => {
                    let (_name, _entry_stat) = self.read_dnt2_entry()?;
                    
                    // Read next response - could be DATA or another DNT2
                    self.stream.read_exact(&mut response)?;
                    if &response != b"DATA" && &response != b"DNT2" {
                        return Err(format!(
                            "Expected DATA or DNT2 after DNT2, got: {:?}",
                            String::from_utf8_lossy(&response)
                        ).into());
                    }
                    // If it's DNT2, continue loop to process it
                    if &response == b"DNT2" {
                        continue;
                    }
                    // Otherwise it's DATA, fall through to DATA handling
                }
                b"DONE" => {
                    // Flush any remaining response
                    self.stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))?;
                    let mut buffer = [0u8; 1024];
                    while self.stream.read(&mut buffer).is_ok() {}
                    self.stream.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
                    break;
                }
                _ => return Err(format!(
                    "Unexpected response during transfer: {:?}",
                    String::from_utf8_lossy(&response)
                ).into()),
            }
        }

        // Show final statistics if progress bar was enabled
        if let Some(pb) = pb {
            let total_duration = transfer_start.elapsed();
            let avg_speed = total_bytes as f64 / total_duration.as_secs_f64() / 1024.0 / 1024.0;
            pb.finish_with_message(format!(
                "Transfer completed in {:.2}s at {:.2} MB/s average",
                total_duration.as_secs_f64(),
                avg_speed
            ));
        }

        Ok(())
    }

    fn read_dnt2_entry(&mut self) -> Result<(String, AdbLstatResponse), Box<dyn Error>> {
        // Read full metadata block (72 bytes)
        let mut entry_data = [0u8; 72];
        entry_data[0..4].copy_from_slice(b"DNT2");  // Put magic number back
        self.stream.read_exact(&mut entry_data[4..])?;  // Read remaining 68 bytes
        
        // Parse the entry data
        let entry_stat = AdbLstatResponse::from_bytes(&entry_data)?;

        // Read name length and name
        let mut name_len_bytes = [0u8; 4];
        self.stream.read_exact(&mut name_len_bytes)?;
        let name_len = u32::from_le_bytes(name_len_bytes) as usize;
        let mut name_bytes = vec![0u8; name_len];
        self.stream.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes).to_string();

        Ok((name, entry_stat))
    }
}

pub fn send(
    host: &str,
    port: &str,
    messages: Vec<&str>,
    no_clean_response: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    debug!("=== Starting send operation ===");
    debug!("Host: {}, Port: {}", host, port);
    debug!("Messages to send: {:?}", messages);
    debug!("no_clean_response: {}", no_clean_response);

    debug!("Creating AdbStream...");
    let mut adb = AdbStream::new(host, port)?;
    debug!("AdbStream created successfully");
    let mut responses = Vec::new();

    for (i, message) in messages.iter().enumerate() {
        debug!("\n--- Sending message {} of {} ---", i + 1, messages.len());
        debug!("Message: {:?}", message);
        debug!("--------------------------------");
        debug!("About to call send_command...");
        adb.send_command(message)?;
        debug!("send_command completed successfully");

        let response = adb.read_response()?;
        if !response.is_empty() {
            if response != "OKAY" {
                responses.push(if no_clean_response {
                    clean_str(response.as_str())
                } else {
                    remove_unnecessary_unicode(&response)
                });
            } else {
                // response == "OKAY", check if there's more response
                let response = adb.read_response()?;
                if !response.is_empty() {
                    debug!("!! Got more response: {:?}", response);
                    responses.push(if no_clean_response {
                        clean_str(response.as_str())
                    } else {
                        remove_unnecessary_unicode(&response)
                    });
                }
            }
            debug!("Got response: {:?}", response);
        }
    }

    debug!("Cleaning up responses...");
    responses.retain(|s| !s.is_empty());
    debug!("Final responses: {:?}", responses);
    debug!("=== Send operation completed ===");

    Ok(responses)
}

#[allow(dead_code)]
fn remove_002b(input: &str) -> String {
    input.replace("\u{002B}", "") // Use Unicode escape for "+"
}

// Or, if you specifically want to remove the literal "002b"
#[allow(dead_code)]
fn remove_literal_002b(input: &str) -> String {
    input.replace("002b", "")
}

fn clean_str(input: &str) -> String {
    input
        .replace("OKAY", "")
        .chars()
        .filter(|&c| c != '\u{0}' && (c.is_ascii_graphic() || c.is_whitespace()))
        .collect()
}

fn remove_unnecessary_unicode(input: &str) -> String {
    let input = input.strip_prefix("OKAY").unwrap_or(input);

    let input = input.get(4..).unwrap_or("");

    clean_str(input)
}

pub fn format_responses(responses: &[String]) -> String {
    debug!("before formatting responses = {:?}", responses);
    let outgoing_responses = responses
        .iter()
        .map(|r| r.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    debug!("after formatting responses = {:?}", outgoing_responses);

    outgoing_responses
}

pub async fn run_shell_command_async(
    host: &str,
    port: &str,
    command: &str,
    adb_id: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let host_command = match adb_id {
        Some(id) => format_command("SELECT_DEVICE", &[id]),
        None => format_command("ANY_DEVICE", &[]),
    };

    let formatted_command = format_command("SHELL_V2", &[command]);
    let messages = vec![host_command.as_str(), formatted_command.as_str()];

    match send(host, port, messages, false) {
        Ok(responses) => {
            debug!("{:?}", responses);
            Ok(format_responses(&responses))
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}

#[allow(dead_code)]
pub async fn run_command_async(
    host: &str,
    port: &str,
    command: &str,
    adb_id: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let host_command = match adb_id {
        Some(id) => format_command("SELECT_DEVICE", &[id]),
        None => format_command("ANY_DEVICE", &[]),
    };

    let messages: Vec<&str> = vec![host_command.as_str(), command];

    match send(host, port, messages, false) {
        Ok(responses) => {
            debug!("{:?}", responses);
            Ok(format_responses(&responses))
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}

pub async fn getprop_async(
    host: &str,
    port: &str,
    propname: &str,
    adb_id: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let host_command = match adb_id {
        Some(id) => format_command("SELECT_DEVICE", &[id]),
        None => format_command("ANY_DEVICE", &[]),
    };

    let getprop_command = format_command("GETPROP_SINGLE", &[propname]);
    let messages = vec![host_command.as_str(), getprop_command.as_str()];

    match send(host, port, messages, true) {
        Ok(responses) => {
            let formatted = format_responses(&responses);
            // Check if the response is an error from ADB
            if formatted.starts_with("FAIL") {
                // Return empty string for unauthorized devices
                Ok(String::new())
            } else {
                Ok(formatted)
            }
        },
        Err(e) => Err(e),
    }
}

pub async fn getprops_parallel(
    host: &str,
    port: &str,
    propnames: &[String],
    adb_id: Option<&str>,
) -> HashMap<String, String> {
    let mut tasks: Vec<JoinHandle<(String, String)>> = Vec::new();
    let host = Arc::new(host.to_string()); // Arc for shared ownership in async tasks
    let port = Arc::new(port.to_string());
    let adb_id = adb_id.map(|id| Arc::new(id.to_string()));

    for propname in propnames {
        let host_clone = Arc::clone(&host);
        let port_clone = Arc::clone(&port);
        let propname = propname.to_string();
        let adb_id_clone = adb_id.clone();

        tasks.push(tokio::spawn(async move {
            let result = getprop_async(
                &host_clone,
                &port_clone,
                &propname,
                adb_id_clone.as_ref().map(|arc| arc.as_str()),
            )
            .await
            .unwrap_or_default();
            (propname, result)
        }));
    }

    let mut results = HashMap::new();
    for task in tasks {
        let (propname, result) = task.await.unwrap();
        results.insert(propname, result);
    }

    results
}

fn get_permissions(path: &PathBuf) -> std::io::Result<u32> {
    debug!("get_permissions: {:?}", path);
    let metadata = fs::metadata(path)?;
    Ok(metadata.permissions().mode())
}

pub async fn push(
    host: &str,
    port: &str,
    adb_id: Option<&str>,
    src_path: &PathBuf,
    dst_path: &PathBuf,
    has_multiple_sources: bool,
    progress: ProgressDisplay,
) -> Result<(), Box<dyn Error>> {
    debug!("Starting push operation:");
    debug!("Source path: {:?}", src_path);
    debug!("Destination path: {:?}", dst_path);
    debug!("Has multiple sources: {}", has_multiple_sources);

    // Initialize connection
    let mut adb = AdbStream::new(host, port)?;
    let host_command = match adb_id {
        Some(id) => format!("host:tport:serial:{}", id),
        None => "host:tport:any".to_string(),
    };

    // Setup connection
    adb.send_command(&host_command)?;
    adb.read_okay()?;
    adb.send_command("sync:")?;
    adb.read_okay()?;
    adb.read_response()?;
    adb.read_okay()?;

    // If source is a directory, collect all files first
    let files_to_transfer = if src_path.is_dir() {
        let mut files = Vec::new();
        let src_base = src_path.parent().unwrap_or(src_path);
        for entry in walkdir::WalkDir::new(src_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                files.push((
                    entry.path().to_path_buf(),
                    dst_path.join(entry.path().strip_prefix(src_base)?),
                ));
            }
        }
        files
    } else {
        // For single file, construct destination path with filename
        let filename = src_path.file_name()
            .ok_or("Source file must have a name")?;
        let dst_file = if dst_path.to_string_lossy().ends_with('/') || dst_path.is_dir() {
            dst_path.join(filename)
        } else {
            dst_path.clone()
        };
        vec![(src_path.clone(), dst_file)]
    };

    // Transfer each file
    for (src_file, dst_file) in files_to_transfer {
        // Get permissions and transfer file
        let perms = get_permissions(&src_file)?;
        adb.transfer_file(&src_file, &dst_file.to_string_lossy(), perms, progress)?;
    }

    Ok(())
}

pub fn start_adb_server(port: &str) -> Result<(), Box<dyn Error>> {
    debug!("Checking if ADB server needs to be started...");

    // Create the command with proper detached settings
    let mut command = Command::new("adb");
    command
        .args(["-L", &format!("tcp:{}", port), "server", "--reply-fd", "4"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    // On Unix systems, set process group ID to detach completely
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    debug!("Starting ADB server in detached mode on port {}...", port);
    match command.spawn() {
        Ok(_) => {
            debug!("ADB server process spawned successfully");
            // Give the server a moment to initialize
            std::thread::sleep(SERVER_START_DELAY);
            Ok(())
        }
        Err(e) => {
            debug!("Failed to start ADB server: {}", e);
            Err(e.into())
        }
    }
}

pub fn check_server_status(host: &str, port: &str) -> bool {
    debug!("Checking if ADB server is running...");

    // Try to connect directly without using AdbStream to avoid recursion
    if let Ok(mut stream) = TcpStream::connect(format!(
        "{}:{}",
        if host == "localhost" {
            "127.0.0.1"
        } else {
            host
        },
        port
    )) {
        debug!("Connected to ADB port, checking server response...");

        // Format the version command according to ADB protocol
        let request = "000chost:version";
        if let Ok(_) = stream.write_all(request.as_bytes()) {
            let mut response = [0u8; 4];
            if let Ok(_) = stream.read_exact(&mut response) {
                let is_running = &response == b"OKAY";
                debug!(
                    "ADB server status: {}",
                    if is_running { "running" } else { "not running" }
                );
                return is_running;
            }
        }
    }

    debug!("ADB server is not running");
    false
}

// Update the existing check_server_running to use the new function
fn check_server_running(host: &str, port: &str) -> bool {
    check_server_status(host, port)
}

pub fn kill_server(host: &str, port: &str) -> Result<(), Box<dyn Error>> {
    debug!("Sending kill command to ADB server...");
    match send(host, port, vec!["host:kill"], false) {
        Ok(_) => {
            debug!("Kill command sent successfully");
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("Connection refused") {
                debug!("Server is already stopped");
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

pub async fn pull(
    host: &str,
    port: &str,
    adb_id: Option<&str>,
    src_path: &PathBuf,
    dst_path: &PathBuf,
    progress: ProgressDisplay,
) -> Result<(), Box<dyn Error>> {
    debug!("\n=== Starting Pull Operation ===");
    debug!("Source: {:?}", src_path);
    debug!("Destination: {:?}", dst_path);
    debug!("Starting pull operation:");
    debug!("Source path: {:?}", src_path);
    debug!("Destination path: {:?}", dst_path);

    // Get the filename from src_path
    let filename = src_path
        .file_name()
        .ok_or("Source path must have a filename")?
        .to_string_lossy();
    debug!("Filename: {}", filename);

    // Construct the full destination path
    let full_dst_path = if dst_path.to_string_lossy().ends_with('/') || dst_path.is_dir() {
        debug!("Destination is a directory, appending filename");
        dst_path.join(&*filename)
    } else {
        dst_path.clone()
    };

    debug!("Full destination path: {:?}", full_dst_path);
    debug!("Full destination path: {:?}", full_dst_path);

    let host_command = match adb_id {
        Some(id) => format!("host:tport:serial:{}", id),
        None => "host:tport:any".to_string(),
    };
    debug!("Using host command: {}", host_command);
    debug!("Using device: {}", host_command);

    let mut adb = AdbStream::new(host, port)?;

    // Send device selection command
    debug!("\n[1/4] Connecting to device...");
    debug!("Sending host_command: {}", host_command);
    adb.send_command(&host_command)?;
    adb.read_okay()?;

    // Send sync command
    debug!("[2/4] Initializing sync...");
    debug!("Sending sync: command");
    adb.send_command("sync:")?;
    adb.read_okay()?;
    adb.read_response()?;
    adb.read_okay()?;

    // Send LST2 command to get file size
    debug!("[3/4] Checking source path...");
    let src_path_str = src_path.to_string_lossy();
    let src_path_bytes = src_path_str.as_bytes();
    let mut command = Vec::with_capacity(4 + 4 + src_path_bytes.len());
    command.extend_from_slice(b"LST2");
    command.extend_from_slice(&(src_path_bytes.len() as u32).to_le_bytes());
    command.extend_from_slice(src_path_bytes);
    adb.write_all(&command)?;

    // Read and parse LST2 response
    let mut response_buf = [0u8; 72];
    adb.stream.read_exact(&mut response_buf)?;
    let lstat_response = AdbLstatResponse::from_bytes(&response_buf)?;
    let file_size = lstat_response.size() as u64;
    debug!("Source type: {}", lstat_response.file_type());
    debug!("Source size: {} bytes", file_size);

    // If source is a directory, list its contents with LIS2
    let files_to_transfer = if lstat_response.file_type() == "Directory" {
        debug!("\n=== Pulling Directory ===");
        debug!("Listing directory contents...");
        let mut command = Vec::with_capacity(4 + 4 + src_path_bytes.len());
        command.extend_from_slice(b"LIS2");
        command.extend_from_slice(&(src_path_bytes.len() as u32).to_le_bytes());
        command.extend_from_slice(src_path_bytes);
        adb.write_all(&command)?;

        // Get the source directory name
        let src_dir_name = src_path.file_name()
            .ok_or("Source directory must have a name")?
            .to_string_lossy();
        
        // Create destination directory including the source directory name
        let dst_dir = dst_path.join(&*src_dir_name);
        fs::create_dir_all(&dst_dir)?;

        // Collect all files to transfer
        let mut files = Vec::new();
        loop {
            let mut response = [0u8; 4];
            if adb.stream.read_exact(&mut response).is_err() {
                break;
            }

            match &response {
                b"DNT2" => {
                    let (name, entry_stat) = adb.read_dnt2_entry()?;
                    // Preserve relative path by joining with src_path first, then getting relative component
                    let full_src_path = src_path.join(&name);
                    let relative_path = full_src_path.strip_prefix(src_path)?;
                    let full_dst_path = dst_dir.join(relative_path);
                    
                    files.push((
                        full_src_path,
                        full_dst_path,
                        entry_stat.size() as u64,
                    ));
                }
                b"DONE" => {
                    debug!("Directory listing complete");
                    // Flush any remaining response
                    adb.stream.set_read_timeout(Some(std::time::Duration::from_millis(100)))?;
                    let mut buffer = [0u8; 1024];
                    while adb.stream.read(&mut buffer).is_ok() {}
                    adb.stream.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
                    break;
                }
                _ => {
                    return Err(format!(
                        "Unexpected response during directory listing: {:?}",
                        String::from_utf8_lossy(&response)
                    ).into())
                }
            }
        }
        files
    } else {
        vec![(src_path.clone(), full_dst_path.clone(), file_size)]
    };

    // Transfer all files
    for (src_file, dst_file, file_size) in files_to_transfer {
        // Send RCV2 command with path
        debug!("\n[4/4] Starting file transfer...");
        let file_path_str = src_file.to_string_lossy();
        let file_path_bytes = file_path_str.as_bytes();
        let mut command = Vec::with_capacity(4 + 4 + file_path_bytes.len() + 8);
        command.extend_from_slice(b"RCV2");
        command.extend_from_slice(&(file_path_bytes.len() as u32).to_le_bytes());
        command.extend_from_slice(file_path_bytes);
        command.extend_from_slice(b"RCV2");
        command.extend_from_slice(&[0, 0, 0, 0]);
        adb.write_all(&command)?;

        // Transfer the file using shared function
        adb.transfer_data(
            &dst_file,
            file_size,
            "file",
            progress,
        )?;
    }

    debug!("\n=== Pull Operation Completed Successfully ===");
    Ok(())
}

// AdbLstatResponse and ProgressDisplay are now in protocol.rs
// Re-exported at the top of this file for backwards compatibility
