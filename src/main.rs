use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::str;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = "127.0.0.1:5037"; // Replace with your server address

    // Resolve the server address
    let mut addresses = server_address.to_socket_addrs()?;
    let address = addresses.next().ok_or("Could not resolve address")?;

    // Connect to the server
    let mut stream = TcpStream::connect(address)?;

    // Set a timeout for reads (Optional but recommended)
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;

    let message = "000chost:devices";
    println!("Sending: {}", message);

    // Send the message as bytes
    stream.write_all(message.as_bytes())?;

    // Receive multiple responses (until the connection closes or an error occurs)
    loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Server closed the connection.");
                break; // Connection closed by the server
            }
            Ok(bytes_read) => {
                let response = str::from_utf8(&buffer[..bytes_read])?;
                println!("Received: {}", response);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Non-blocking read, no data available yet.
                continue; // Or handle it differently if needed
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break; // Exit loop on error
            }
        }
    }

    Ok(())
}
