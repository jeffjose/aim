use aim::send_and_receive;
use clap::{Parser, Subcommand}; // Import from the 'aim' module

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Server address (e.g., 127.0.0.1:8080)
    #[arg(short, long, default_value = "127.0.0.1:5037")]
    server: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Lists devices
    Ls { name: Option<String> },
    /// Gets the server version
    Version,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let message_to_send = match cli.command {
        Some(Commands::Ls { name }) => "000chost:devices",
        Some(Commands::Version) => "000chost:version",
        None => "000chost:devices",
    };

    match send_and_receive(&cli.server, message_to_send) {
        //call the function
        Ok(responses) => {
            println!("All responses received:");
            for response in responses {
                println!("{}", response);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
