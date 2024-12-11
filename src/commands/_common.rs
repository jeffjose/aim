use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::str;

pub fn send_and_receive(
    host: &str,
    port: &str,
    message: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

    println!("{:?}", addresses);

    let mut stream = TcpStream::connect(address)?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(2)))?;

    println!("Sending: {}", message);
    stream.write_all(message.as_bytes())?;

    let mut responses = Vec::new();
    loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(0) => {
                //println!("Server closed the connection.");
                break;
            }
            Ok(bytes_read) => {
                let response = str::from_utf8(&buffer[..bytes_read])?;
                responses.push(response.to_string()); // Store response
                                                      //println!("Received: {}", response);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                return Err(e.into()); // Return the error
            }
        }
    }

    Ok(responses)
}
