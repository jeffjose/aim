mod commands;

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
    Foo,
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Ls)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command() {
        Commands::Ls => commands::ls::run(&cli.server),
        Commands::Version => commands::version::run(&cli.server),
        Commands::Foo => commands::foo::run(&cli.server),
    };

    Ok(())
}
