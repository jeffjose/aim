mod formatters;

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

#[derive(Subcommand, Clone)]
enum Commands {
    /// Lists devices
    Ls,
    /// Gets the server version
    Version,
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Ls)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let message_to_send = match cli.command() {
        Commands::Ls => "000chost:devices",
        Commands::Version => "000chost:version",
    };

    match send_and_receive(&cli.server, message_to_send) {
        Ok(responses) => {
            let formatted_output = match cli.command() {
                Commands::Ls => formatters::ls::format(&responses),
                Commands::Version => formatters::version::format(&responses),
            };
            println!("{}", formatted_output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
