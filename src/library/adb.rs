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
use std::thread;
use tokio::task::JoinHandle;

use crate::types::DeviceDetails;

const SYNC_DATA: &[u8] = b"SEND";
const SYNC_DONE: &[u8] = b"DONE";

struct AdbStream {
    stream: TcpStream,
}

impl AdbStream {
    fn new(host: &str, port: &str) -> Result<Self, Box<dyn Error>> {
        debug!("=== Creating new ADB stream ===");

        // Check if server is running, if not start it
        if !check_server_running(host, port) {
            start_adb_server(port)?;

            // Verify server started successfully
            if !check_server_running(host, port) {
                return Err("Failed to start ADB server".into());
            }
        }

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
        let mut stream = TcpStream::connect(address)?;
        debug!("Connection established");

        stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(2)))?;
        debug!("Timeouts set");

        Ok(Self { stream })
    }

    fn send_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        debug!("Sending command: {}", command);
        let request = format!("{:04x}{}", command.len(), command);
        debug!("Formatted request: {:?}", request);
        self.stream.write_all(request.as_bytes())?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = [0; 1024];
        debug!("Waiting for response...");

        match self.stream.read(&mut buffer) {
            Ok(0) => {
                debug!("Server closed the connection");
                Ok(String::new())
            }
            Ok(bytes_read) => {
                let response = str::from_utf8(&buffer[..bytes_read])?.to_string();
                println!("Raw response: {:?}", response);
                Ok(response)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                debug!("Would block");
                Ok(String::new())
            }
            Err(e) => {
                debug!("Error reading from socket: {}", e);
                Err(e.into())
            }
        }
    }

    fn read_okay(&mut self) -> Result<(), Box<dyn Error>> {
        let mut response = [0u8; 4];
        self.stream.read_exact(&mut response)?;
        println!("Response in read_okay: {:?}", response);
        // Check if the response is "OKAY" or [8, 0, 0, 0]
        if &response != b"OKAY"
            && response != [8, 0, 0, 0]
            && response != [9, 0, 0, 0]
            && response != [0, 0, 0, 0]
            && response != [3, 0, 0, 0]
        {
            return Err("Expected OKAY response".into());
        }
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Box<dyn Error>> {
        self.stream.write_all(buf)?;
        Ok(())
    }
}

pub fn send(host: &str, port: &str, messages: Vec<&str>) -> Result<Vec<String>, Box<dyn Error>> {
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
                responses.push(clean_str(&response));
            }
            if response == "OKAY" {
                // lets check if there's more response
                let response = adb.read_response()?;
                if response.is_empty() {
                    break;
                }
                debug!("!! Got more response: {:?}", response);
                responses.push(clean_str(&response));
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

fn clean_str(s: &str) -> String {
    remove_unnecessary_unicode(s)
}

fn remove_unnecessary_unicode(input: &str) -> String {
    let input = input.strip_prefix("OKAY").unwrap_or(input);

    let input = input.get(4..).unwrap_or("");

    input
        .chars()
        .filter(|&c| c != '\u{0}' && (c.is_ascii_graphic() || c.is_whitespace()))
        .collect()
}

// pub fn run_command(host: &str, port: &str, command: &str, adb_id: Option<&str>) -> String {
//     let host_command = match adb_id {
//         Some(id) => format!("host:tport:serial:{}", id),
//         None => "host:tport:any".to_string(),
//     };

//     let formatted_command = format!("shell,v2,TERM=xterm-256color,raw:{}", command);

//     let messages: Vec<&str> = vec![host_command.as_str(), formatted_command.as_str()];

//     match send_and_receive(&host, &port, messages) {
//         Ok(responses) => format_responses(&responses),
//         Err(e) => {
//             eprintln!("Error: {}", e);
//             String::new()
//         }
//     }
// }

pub fn format_responses(responses: &[String]) -> String {
    debug!("incoming responses = {:?}", responses);
    let outgoing_responses = responses
        .iter()
        .map(|r| r.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    debug!("outgoing responses = {:?}", outgoing_responses);

    outgoing_responses
}

pub async fn run_shell_command_async(
    host: &str,
    port: &str,
    command: &str,
    adb_id: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let host_command = match adb_id {
        Some(id) => format!("host:tport:serial:{}", id),
        None => "host:tport:any".to_string(),
    };

    let formatted_command = format!("shell,v2,TERM=xterm-256color,raw:{}", command);

    let messages: Vec<&str> = vec![host_command.as_str(), formatted_command.as_str()];

    match send(&host, &port, messages) {
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

pub async fn run_command_async(
    host: &str,
    port: &str,
    command: &str,
    adb_id: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let host_command = match adb_id {
        Some(id) => format!("host:tport:serial:{}", id),
        None => "host:tport:any".to_string(),
    };

    let messages: Vec<&str> = vec![host_command.as_str(), command];

    match send(&host, &port, messages) {
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
    let command = format!("getprop {}", propname);
    run_shell_command_async(host, port, command.as_str(), adb_id).await
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

    // Send STA2 command to check destination
    println!("Checking destination path...");
    let dst_path_str = dst_path.to_string_lossy();
    adb.write_all(b"STA2")?;
    adb.write_all(&(dst_path_str.len() as u32).to_le_bytes())?;
    adb.write_all(dst_path_str.as_bytes())?;

    // Read STA2 response
    let mut response = [0u8; 4];
    if let Ok(_) = adb.stream.read_exact(&mut response) {
        if &response == b"STA2" {
            let mut stat_bytes = [0u8; 16];
            adb.stream.read_exact(&mut stat_bytes)?;
            let mode = u32::from_le_bytes(stat_bytes[0..4].try_into()?);
            let size = u64::from_le_bytes(stat_bytes[4..12].try_into()?);
            let error = u32::from_le_bytes(stat_bytes[12..16].try_into()?);

            if error == 0 {
                println!("Destination path exists:");
                println!(
                    "  Type: {}",
                    if mode & 0o040000 != 0 {
                        "directory"
                    } else {
                        "file"
                    }
                );
                println!("  Size: {} bytes", size);
            } else {
                println!("Destination path does not exist");
            }
        }
    }

    // Get the filename from src_path
    let filename = src_path
        .file_name()
        .ok_or("Source path must have a filename")?
        .to_string_lossy();

    // Construct the full destination path
    let full_dst_path = if dst_path.to_string_lossy().ends_with('/')
        || fs::metadata(src_path)?.is_file() && !dst_path.file_name().is_some()
    {
        // Append filename if:
        // - dst_path ends with '/' (it's explicitly a directory)
        // - OR src_path is a file AND dst_path doesn't have a filename component
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
    adb.write_all(b"SEND")?;
    adb.write_all(&(path_header.len() as u32).to_le_bytes())?;
    adb.write_all(path_header.as_bytes())?;

    // Read and send file data in chunks
    debug!("Starting file transfer...");
    let mut file = File::open(src_path)?;
    let file_size = fs::metadata(src_path)?.len();
    let mut buffer = [0u8; 64 * 1024]; // 64KB chunks
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
    let avg_speed = (total_bytes as f64 / total_duration.as_secs_f64() / 1024.0 / 1024.0); // MB/s

    // Finish progress bar with final statistics
    pb.finish_with_message(format!(
        "Transfer completed in {:.2}s at {:.2} MB/s average",
        total_duration.as_secs_f64(),
        avg_speed
    ));

    // Send DONE command with timestamp
    debug!("Sending DONE command...");
    adb.write_all(b"DONE")?;
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
            std::thread::sleep(std::time::Duration::from_secs(1));
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
    match send(host, port, vec!["host:kill"]) {
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
    println!("\n=== Starting Pull Operation ===");
    println!("Source: {:?}", src_path);
    println!("Destination: {:?}", dst_path);
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
