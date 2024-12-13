mod subcommands;
mod library;

use clap::{Parser, Subcommand};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputType {
    Table,
    Json,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Hostname
    #[arg(long, global = true, default_value = "localhost")]
    host: String,

    /// Port
    #[arg(long, short = 'p', global = true, default_value = "5037")]
    port: String,

    /// Timeout in seconds
    #[arg(long, short = 't', global = true, default_value_t = 5)]
    timeout: u8,

    /// Output format
    #[arg(long, short = 'o', global = true, default_value = "table")]
    output: OutputType,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Lists devices
    // Ls {
    //     #[clap(long, short = 'l')]
    //     long: bool,
    // },
    Ls,

    //Ll,
    /// Gets the server version
    Version,

    /// Send bogus command to adb-server
    Foo,

    /// Get model name
    Model,

    /// Send a command
    Command {
        command: String,
    },

    /// Get a prop
    Getprop {
        propname: String,
    },

    Getprops {
        propnames: Vec<String>,
    },
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Ls)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match cli.command() {
        Commands::Ls => subcommands::ls::run(&cli.host, &cli.port, cli.output).await,
        Commands::Version => subcommands::version::run(&cli.host, &cli.port),
        Commands::Foo => subcommands::foo::run(&cli.host, &cli.port),
        Commands::Model => subcommands::model::run(&cli.host, &cli.port),
        Commands::Command { command } => subcommands::command::run(&cli.host, &cli.port, &command),
        Commands::Getprop { propname } => subcommands::getprop::run(&cli.host, &cli.port, &propname),
        Commands::Getprops { propnames } => {
            subcommands::getprops::run(&cli.host, &cli.port, &propnames).await
        }
    };

    Ok(())
}
