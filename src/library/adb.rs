use super::protocol::format_command;
use indicatif::ProgressBar;
use log::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{self};
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

const SYNC_DATA: &[u8] = b"SEND";
const SYNC_DONE: &[u8] = b"DONE";
const BUFFER_SIZE: usize = 1024;
const CHUNK_SIZE: usize = 64 * 1024;
const SERVER_START_DELAY: std::time::Duration = std::time::Duration::from_secs(1);
const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
const RESPONSE_OKAY: &[u8] = b"OKAY";
const S_IFMT: u16 = 0o170000; // bit mask for the file type bit field
const S_IFSOCK: u16 = 0o140000; // socket
const S_IFLNK: u16 = 0o120000; // symbolic link
const S_IFREG: u16 = 0o100000; // regular file
const S_IFBLK: u16 = 0o060000; // block device
const S_IFDIR: u16 = 0o040000; // directory
const S_IFCHR: u16 = 0o020000; // character device
const S_IFIFO: u16 = 0o010000; // FIFO

type AdbResult<T> = Result<T, Box<dyn Error>>;

struct AdbStream {
    stream: TcpStream,
}

impl AdbStream {
    fn new(host: &str, port: &str) -> Result<Self, Box<dyn Error>> {
        debug!("=== Creating new ADB stream ===");

        Self::ensure_server_running(host, port)?;
        let stream = Self::establish_connection(host, port)?;

        Ok(Self { stream })
    }

    fn ensure_server_running(host: &str, port: &str) -> Result<(), Box<dyn Error>> {
        if !check_server_running(host, port) {
            start_adb_server(port)?;

            if !check_server_running(host, port) {
                return Err("Failed to start ADB server".into());
            }
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

        debug!("Establishing connection...");
        let stream = TcpStream::connect(address)?;
        debug!("Connection established");

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
                    return Err(e.into());
                }
            }
        }

        println!("Response: {:?}", response);
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

        // Define valid responses
        const VALID_RESPONSES: &[&[u8]] = &[
            RESPONSE_OKAY,
            &[0, 0, 0, 0],
            &[1, 0, 0, 0],
            &[2, 0, 0, 0],
            &[3, 0, 0, 0],
            &[4, 0, 0, 0],
            &[5, 0, 0, 0],
            &[6, 0, 0, 0],
            &[7, 0, 0, 0],
            &[8, 0, 0, 0],
            &[9, 0, 0, 0],
        ];

        if !VALID_RESPONSES.contains(&&response[..]) {
            return Err(format!("Expected OKAY response.  Got {:?}", response).into());
        }
        Ok(())
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
}

pub fn send(
    host: &str,
    port: &str,
    messages: Vec<&str>,
    no_clean_response: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    debug!("=== Starting send operation ===");
    debug!("Messages to send: {:?}", messages);

    let mut adb = AdbStream::new(host, port)?;
    let mut responses = Vec::new();

    for (i, message) in messages.iter().enumerate() {
        debug!("\n--- Sending message {} of {} ---", i + 1, messages.len());
        debug!("Message: {:?}", message);
        debug!("--------------------------------");
        adb.send_command(message)?;

        loop {
            let response = adb.read_response()?;
            if response.is_empty() {
                continue;
            }
            if response != "OKAY" {
                responses.push(if no_clean_response {
                    clean_str(response.as_str())
                } else {
                    remove_unnecessary_unicode(&response)
                });
            }
            if response == "OKAY" {
                // lets check if there's more response
                let response = adb.read_response()?;
                if response.is_empty() {
                    break;
                }
                debug!("!! Got more response: {:?}", response);
                responses.push(if no_clean_response {
                    clean_str(response.as_str())
                } else {
                    remove_unnecessary_unicode(&response)
                });
            }
            debug!("Got response: {:?}", response);
            break;
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
        Ok(responses) => Ok(format_responses(&responses)),
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
            .unwrap();
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
) -> Result<(), Box<dyn Error>> {
    debug!("Starting push operation:");
    debug!("Source path: {:?}", src_path);
    debug!("Destination path: {:?}", dst_path);

    let host_command = match adb_id {
        Some(id) => format!("host:tport:serial:{}", id),
        None => "host:tport:any".to_string(),
    };
    debug!("Using host command: {}", host_command);

    let mut adb = AdbStream::new(host, port)?;

    // Send device selection command
    debug!("Sending host_command: {}", host_command);
    adb.send_command(&host_command)?;
    adb.read_okay()?;

    // Send sync command
    debug!("Sending sync: command");
    adb.send_command("sync:")?;
    adb.read_okay()?;
    adb.read_response()?;
    adb.read_okay()?;

    // Send STA2 command and parse response
    println!("Checking destination path... {:?}", dst_path);
    let dst_path_str = dst_path.to_string_lossy();
    let dst_path_bytes = dst_path_str.as_bytes();
    let mut command = Vec::with_capacity(4 + 4 + dst_path_bytes.len());
    command.extend_from_slice(b"LST2");
    command.extend_from_slice(&(dst_path_bytes.len() as u32).to_le_bytes());
    command.extend_from_slice(dst_path_bytes);
    adb.write_all(&command)?;

    // Read and parse STA2 response
    let mut response_buf = [0u8; 72]; // STA2 response is 72 bytes
    adb.stream.read_exact(&mut response_buf)?;

    println!("{:?}", response_buf);
    let lstat_response = AdbLstatResponse::from_bytes(&response_buf)?;
    println!("LST2 response: {:?}", lstat_response);

    let is_directory = lstat_response.file_type() == "Directory";
    println!("Is directory: {}", is_directory);
    println!("File type: {}", lstat_response.file_type());

    // Get the filename from src_path
    let filename = src_path
        .file_name()
        .ok_or("Source path must have a filename")?
        .to_string_lossy();

    // Construct the full destination path
    let full_dst_path = if is_directory || dst_path.to_string_lossy().ends_with('/') {
        dst_path.join(&*filename)
    } else {
        dst_path.clone()
    };

    debug!("Full destination path: {:?}", full_dst_path);

    // Get file permissions and prepare path header
    let perms = get_permissions(src_path)?;
    let path_header = format!("{},{}", full_dst_path.to_string_lossy(), perms);
    debug!("Path header: {}", path_header);

    // Send SEND command with path and mode
    debug!("Sending SEND command...");
    adb.write_all(SYNC_DATA)?;
    adb.write_length_prefixed(path_header.as_bytes())?;

    // Read and send file data in chunks
    debug!("Starting file transfer...");
    let mut file = File::open(src_path)?;
    let file_size = fs::metadata(src_path)?.len();
    let mut buffer = [0u8; CHUNK_SIZE]; // 64KB chunks
    let mut total_bytes = 0;

    // Setup progress bar
    let pb = ProgressBar::new(file_size);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let transfer_start = std::time::Instant::now();
    let mut chunk_start;

    loop {
        chunk_start = std::time::Instant::now();
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        total_bytes += bytes_read;

        adb.write_all(b"DATA")?;
        adb.write_all(&(bytes_read as u32).to_le_bytes())?;
        adb.write_all(&buffer[..bytes_read])?;

        // Calculate chunk transfer speed
        let chunk_duration = chunk_start.elapsed();
        let chunk_speed = bytes_read as f64 / chunk_duration.as_secs_f64() / 1024.0 / 1024.0; // MB/s

        // Update progress bar with transfer speed
        pb.set_message(format!("{:.2} MB/s", chunk_speed));
        pb.set_position(total_bytes as u64);
    }

    // Calculate total transfer statistics
    let total_duration = transfer_start.elapsed();
    let avg_speed = total_bytes as f64 / total_duration.as_secs_f64() / 1024.0 / 1024.0; // MB/s

    // Finish progress bar with final statistics
    pb.finish_with_message(format!(
        "Transfer completed in {:.2}s at {:.2} MB/s average",
        total_duration.as_secs_f64(),
        avg_speed
    ));

    // Send DONE command with timestamp
    debug!("Sending DONE command...");
    adb.write_all(SYNC_DONE)?;
    let timestamp = fs::metadata(src_path)?
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as u32;
    adb.write_all(&timestamp.to_le_bytes())?;

    // Check final response
    debug!("Waiting for final response...");
    //adb.read_okay()?;

    debug!("Push operation completed successfully!");
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

    // Construct the full destination path
    let full_dst_path = if dst_path.to_string_lossy().ends_with('/') || dst_path.is_dir() {
        // Append filename if:
        // - dst_path ends with '/' (it's explicitly a directory)
        // - OR dst_path is an existing directory
        dst_path.join(&*filename)
    } else {
        dst_path.clone()
    };

    debug!("Full destination path: {:?}", full_dst_path);

    let host_command = match adb_id {
        Some(id) => format!("host:tport:serial:{}", id),
        None => "host:tport:any".to_string(),
    };
    debug!("Using host command: {}", host_command);

    let mut adb = AdbStream::new(host, port)?;

    // Send device selection command
    println!("\n[1/4] Connecting to device...");
    debug!("Sending host_command: {}", host_command);
    adb.send_command(&host_command)?;
    adb.read_okay()?;

    // Send sync command
    println!("[2/4] Initializing sync...");
    debug!("Sending sync: command");
    adb.send_command("sync:")?;
    adb.read_okay()?;
    adb.read_response()?;
    adb.read_okay()?;

    // Send RECV command with path
    println!("[3/4] Starting file transfer...");
    debug!("Sending RECV command...");
    adb.write_all(b"RECV")?;
    let path_bytes = src_path.to_string_lossy();
    let path_bytes = path_bytes.as_bytes();
    adb.write_all(&(path_bytes.len() as u32).to_le_bytes())?;
    adb.write_all(path_bytes)?;

    // Create destination directory if it doesn't exist
    if let Some(parent) = full_dst_path.parent() {
        println!("Creating directory: {:?}", parent);
        debug!("Creating destination directory: {:?}", parent);
        fs::create_dir_all(parent)?;
    }

    // Open destination file
    println!("[4/4] Creating file: {:?}", full_dst_path);
    let mut file = File::create(&full_dst_path)?;
    let mut total_bytes = 0;

    // Setup progress bar (we don't know total size yet)
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec})")
            .unwrap(),
    );

    let transfer_start = std::time::Instant::now();
    let mut chunk_start;

    println!("\nTransferring data...");
    loop {
        // Read data header
        let mut response = [0u8; 4];
        if adb.stream.read_exact(&mut response).is_err() {
            break;
        }

        match &response {
            b"DATA" => {
                // Read data length
                let mut len_bytes = [0u8; 4];
                adb.stream.read_exact(&mut len_bytes)?;
                let len = u32::from_le_bytes(len_bytes) as usize;

                // Read and write data
                chunk_start = std::time::Instant::now();
                let mut buffer = vec![0u8; len];
                adb.stream.read_exact(&mut buffer)?;
                file.write_all(&buffer)?;
                total_bytes += len;

                // Calculate chunk transfer speed
                let chunk_duration = chunk_start.elapsed();
                let chunk_speed = len as f64 / chunk_duration.as_secs_f64() / 1024.0 / 1024.0; // MB/s

                // Update progress
                pb.set_message(format!("{:.2} MB/s", chunk_speed));
                pb.set_position(total_bytes as u64);
            }
            b"DONE" => {
                println!("\nTransfer complete!");
                break;
            }
            _ => return Err("Unexpected response during file transfer".into()),
        }
    }

    // Calculate total transfer statistics
    let total_duration = transfer_start.elapsed();
    let avg_speed = total_bytes as f64 / total_duration.as_secs_f64() / 1024.0 / 1024.0; // MB/s

    // Finish progress bar with final statistics
    pb.finish_with_message(format!(
        "Transfer completed: {} bytes in {:.2}s at {:.2} MB/s average",
        total_bytes,
        total_duration.as_secs_f64(),
        avg_speed
    ));

    println!("\n=== Pull Operation Completed Successfully ===");
    println!("Total bytes: {}", total_bytes);
    println!("Duration: {:.2}s", total_duration.as_secs_f64());
    println!("Average speed: {:.2} MB/s", avg_speed);
    debug!("Pull operation completed successfully!");
    Ok(())
}

#[derive(Debug)]
pub struct AdbLstatResponse {
    magic: [u8; 4],
    metadata: FileMetadata,
    timestamps: FileTimestamps,
}

#[derive(Debug)]
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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 72 {
            return Err("Invalid byte array length");
        }

        if &bytes[0..4] != b"LST2" {
            return Err("Invalid magic number");
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
