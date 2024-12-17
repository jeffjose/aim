use log::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::str;
use std::sync::Arc;
use tokio::task::JoinHandle;
use std::os::unix::fs::PermissionsExt;

use crate::types::DeviceDetails;

const SYNC_DATA: &[u8] = b"SEND";
const SYNC_DONE: &[u8] = b"DONE";

pub fn send(
    host: &str,
    port: &str,
    messages: Vec<&str>,
) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
    let server_address = format!(
        "{}:{}",
        if host == "localhost" {
            "127.0.0.1"
        } else {
            host
        },
        port
    );

    let mut addresses = server_address.to_socket_addrs()?;

    let address = addresses.next().ok_or("Could not resolve address")?;

    let mut stream = TcpStream::connect(address)?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(2)))?;

    let mut responses = Vec::new();

    for (i, message) in messages.iter().enumerate() {
        info!("   [SEND-{}] {}", i, message);

        let request = format!("{:04x}{}", &message.len(), &message);

        stream.write_all(request.as_bytes())?;

        loop {
            let mut buffer = [0; 1024];
            match stream.read(&mut buffer) {
                Ok(0) => {
                    //println!("Server closed the connection.");
                    break;
                }
                Ok(bytes_read) => {
                    let response = str::from_utf8(&buffer[..bytes_read])?;
                    info!("[RECEIVE-{}] {:?}", i, response);
                    if response != "OKAY" {
                        responses.push(clean_str(response));
                        break;
                    }
                    //println!("Received: {}", response);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    eprintln!("Error reading from socket: {}", e);
                    return Err(e.into()); // Return the error
                }
            }
        }

        info!("[RECEIVE]: {:?}", responses.retain(|s| !s.is_empty()));
    }
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
    let metadata = fs::metadata(path)?;
    Ok(metadata.permissions().mode())
}

fn unix_perms_to_custom_format(perms: u32) -> u32 {
    let mut result = 0;
    let octal_perms = format!("{:o}", perms);

    for (i, digit) in octal_perms.chars().rev().enumerate() {
        let digit_value = digit.to_digit(10).unwrap();
        result += digit_value * 10u32.pow(i as u32);
    }

    result
}

fn get_custom_permissions(path: &PathBuf) -> std::io::Result<u32> {
    let perms = get_permissions(path)?;
    Ok(unix_perms_to_custom_format(perms))
}

pub async fn push(
    adb_id: Option<&str>,
    src_path: &PathBuf,
    dst_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // First select the device

    let host_command = match adb_id {
        Some(id) => {
            format!("host:tport:serial:{}", id)
        }
        None => "host:tport:any".to_string(),
    };

    let path_header = match get_custom_permissions(dst_path) {
        Ok(perms) => format!("{},{},", dst_path.to_string_lossy(), perms),
        Err(e) => {
            // Handle the error appropriately, e.g., log it or return an error
            eprintln!("Error getting permissions: {}", e);
            return Err(Box::new(e)); // Or handle the error differently
        }
    };

    let messages = [format!("sync:")];

    // Send initial commands
    send(
        "127.0.0.1",
        "5037",
        messages.iter().map(|s| s.as_str()).collect(),
    )?;

    // Read and send file data
    let mut file = File::open(src_path)?;
    let mut buffer = [0u8; 64 * 1024]; // 64KB chunks

    let mut stream = TcpStream::connect("127.0.0.1:5037")?;

    let _ = stream.write_all(format!("SEND{}{}", path_header.len(), path_header).as_bytes());

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        stream.write_all(&(bytes_read as u32).to_le_bytes())?;
        stream.write_all(&buffer[..bytes_read])?;
    }

    // Send DONE command
    stream.write_all(SYNC_DONE)?;
    stream.write_all(&(0u32).to_le_bytes())?;

    // Check final response
    let mut response = [0u8; 4];
    stream.read_exact(&mut response)?;
    if &response != b"OKAY" {
        return Err("Final sync failed".into());
    }

    Ok(())
}
