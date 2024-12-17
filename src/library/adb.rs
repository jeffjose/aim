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
            start_adb_server()?;

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
        println!("Sending command: {}", command);
        let request = format!("{:04x}{}", command.len(), command);
        println!("Formatted request: {:?}", request);
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
                debug!("Raw response: {:?}", response);
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
    println!("get_permissions: {:?}", path);
    let metadata = fs::metadata(path)?;
    Ok(metadata.permissions().mode())
}

pub async fn push(
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

    let mut adb = AdbStream::new("127.0.0.1", "5037")?;

    // Send device selection command
    println!("Sending host_command: {}", host_command);
    adb.send_command(&host_command)?;
    adb.read_okay()?;

    // Send sync command
    println!("Sending sync: command");
    adb.send_command("sync:")?;

    // jeffjose | Dec 17, 2024
    // It is unclear why we need this specific combination of read_okay and read_response
    // But this is the only one that worked
    adb.read_okay()?;
    adb.read_response()?;
    adb.read_okay()?;

    // Get file permissions and prepare path header
    let perms = get_permissions(src_path)?;
    let path_header = format!("{},{}", dst_path.to_string_lossy(), perms);
    println!("Path header: {}", path_header);

    // Send SEND command with path and mode
    println!("Sending SEND command...");
    adb.write_all(b"SEND")?;
    adb.write_all(&(path_header.len() as u32).to_le_bytes())?;
    adb.write_all(path_header.as_bytes())?;

    // Read and send file data in chunks
    println!("Starting file transfer...");
    let mut file = File::open(src_path)?;
    let mut buffer = [0u8; 64 * 1024]; // 64KB chunks
    let mut total_bytes = 0;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        total_bytes += bytes_read;

        adb.write_all(b"DATA")?;
        adb.write_all(&(bytes_read as u32).to_le_bytes())?;
        adb.write_all(&buffer[..bytes_read])?;
        println!("Sent {} bytes (total: {} bytes)", bytes_read, total_bytes);
    }

    // Send DONE command with timestamp
    println!("Sending DONE command...");
    adb.write_all(b"DONE")?;
    let timestamp = fs::metadata(src_path)?
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as u32;
    adb.write_all(&timestamp.to_le_bytes())?;

    // Check final response
    println!("Waiting for final response...");
    //adb.read_okay()?;

    println!("Push operation completed successfully!");
    Ok(())
}

fn start_adb_server() -> Result<(), Box<dyn Error>> {
    debug!("Checking if ADB server needs to be started...");

    // Create the command with proper detached settings
    let mut command = Command::new("adb");
    command
        .args(["-L", "tcp:5037", "server", "--reply-fd", "4"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    // On Unix systems, set process group ID to detach completely
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    debug!("Starting ADB server in detached mode...");
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

fn check_server_running(host: &str, port: &str) -> bool {
    debug!("Checking if ADB server is running...");

    if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", host, port)) {
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
