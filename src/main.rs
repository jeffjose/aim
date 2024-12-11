use aim::send_and_receive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = "127.0.0.1:5037";
    let message_to_send = "000chost:devices";

    match send_and_receive(server_address, message_to_send) {
        Ok(responses) => {
            println!("All responses received:");
            for response in responses {
                println!("{}", response);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
