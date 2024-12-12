mod commands;

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
    Ls {
        #[clap(long, short = 'l')]
        long: bool,
    },

    Ll,

    /// Gets the server version
    Version,

    /// Send bogus command to adb-server
    Foo,
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Ls { long: false })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    env_logger::Builder::new()
    .filter_level(cli.verbose.log_level_filter())
    .init();

    match cli.command() {
        Commands::Ls { long } => commands::ls::run(&cli.host, &cli.port, long, cli.output),
        Commands::Ll => commands::ls::run(&cli.host, &cli.port, true, cli.output),
        Commands::Version => commands::version::run(&cli.host, &cli.port),
        Commands::Foo => commands::foo::run(&cli.host, &cli.port),
    };

    Ok(())
}
