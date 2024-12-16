use clap::{Parser, Subcommand};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputType {
    Table,
    Json,
    Plain,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Verbosity level
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Commands>,

    /// ADB server hostname
    #[arg(long, global = true, default_value = "localhost")]
    pub host: String,

    /// ADB server port
    #[arg(long, short = 'p', global = true, default_value = "5037")]
    pub port: String,

    /// Connection timeout in seconds
    #[arg(long, short = 't', global = true, default_value_t = 5)]
    pub timeout: u8,

    /// Output format (table or json)
    #[arg(long, short = 'o', global = true, default_value = "table")]
    pub output: OutputType,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    /// Lists connected devices
    Ls,

    /// Sends a command to a device
    Command {
        /// The command to execute
        command: String,
        /// Optional device ID (can be partial)
        device_id: Option<String>,
    },

    /// Gets a specific property from a device
    Getprop {
        /// Name of the property to get
        propname: String,
        /// Optional device ID (can be partial)
        device_id: Option<String>,
        /// Output format (table, json, plain)
        #[arg(short = 'o', long, default_value = "plain")]
        output: OutputType,
    },

    /// Gets multiple properties from a device
    Getprops {
        /// Names of properties to get
        propnames: Vec<String>,
        /// Optional device ID (can be partial)
        #[arg(last = true)]
        device_id: Option<String>,
    },

    /// Rename a device
    Rename {
        /// Current device ID (can be partial)
        device_id: String,
        /// New name for the device
        new_name: String,
    },
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Ls)
    }
}
