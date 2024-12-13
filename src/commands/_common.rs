use log::*;
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::str;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub fn send_and_receive(
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
    let input = if input.len() >= 4 {
        &input[4..] // Slice from the 4th byte to the end
    } else {
        "" // Handle cases where the string is shorter than 4 bytes
    };

    input
        .chars()
        .filter(|&c| c != '\u{0}' && (c.is_ascii_graphic() || c == '\n' || c == ' ' || c == '\t'))
        .collect()
}

pub fn run_command(host: &str, port: &str, command: &str) -> String {
    let formatted_message = format!("shell,v2,TERM=xterm-256color,raw:{}", command);
    let messages = vec!["host:tport:any", formatted_message.as_str()];
    match send_and_receive(&host, &port, messages) {
        Ok(responses) => format_responses(&responses),
        Err(e) => {
            eprintln!("Error: {}", e);
            String::new()
        }
    }
}

pub fn format_responses(responses: &[String]) -> String {
    responses
        .iter()
        .map(|r| r.trim())
        .collect::<Vec<&str>>()
        .join("\n")
}

pub fn get_prop(host: &str, port: &str, propname: &str) -> String {
    let command = format!("getprop {} {}", propname, propname);

    run_command(host, port, command.as_str())
}

pub async fn run_command_async(
    host: &str,
    port: &str,
    command: &str,
) -> Result<String, Box<dyn Error>> {
    let formatted_message = format!("shell,v2,TERM=xterm-256color,raw:{}", command);
    let messages = vec!["host:tport:any", formatted_message.as_str()];
    match send_and_receive(&host, &port, messages) {
        Ok(responses) => Ok(format_responses(&responses)),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}

pub async fn get_prop_async(
    host: &str,
    port: &str,
    propname: &str,
) -> Result<String, Box<dyn Error>> {
    let command = format!("getprop {} {}", propname, propname);
    run_command_async(host, port, command.as_str()).await
}

pub async fn get_props_parallel(
    host: &str,
    port: &str,
    propnames: &[String],
) -> HashMap<String, String> {
    let mut tasks: Vec<JoinHandle<(String, String)>> = Vec::new();
    let host = Arc::new(host.to_string()); // Arc for shared ownership in async tasks
    let port = Arc::new(port.to_string());

    for propname in propnames {
        let host_clone = Arc::clone(&host);
        let port_clone = Arc::clone(&port);
        let propname = propname.to_string();

        tasks.push(tokio::spawn(async move {
            let result = get_prop_async(&host_clone, &port_clone, &propname)
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
