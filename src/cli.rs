use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Commands>,

    /// ADB server hostname
    #[arg(long, global = true, default_value = "localhost")]
    pub host: String,

    /// Output format (table or json)
    #[arg(long, short = 'o', default_value = "table")]
    pub output: OutputType,

    /// ADB server port
    #[arg(long, short = 'p', global = true, default_value = "5037")]
    pub port: String,

    /// Connection timeout in seconds
    #[arg(long, global = true, default_value_t = 5)]
    pub timeout: u8,

    /// Verbosity level
    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    /// Run arbitrary adb commands
    #[command(name = "adb")]
    Adb {
        #[arg(help = "ADB command to run")]
        command: String,

        #[arg(help = "Device ID")]
        device_id: Option<String>,
    },

    /// Application management commands
    App {
        #[command(subcommand)]
        command: crate::commands::app::AppCommands,
    },

    /// Display configuration
    Config,

    /// Copy files to/from device
    Copy {
        /// Source paths in format device_id:path
        #[arg(required = true)]
        src: Vec<String>,
        /// Destination in format device_id:path
        dst: String,
    },

    /// Run dmesg command on the device
    Dmesg {
        /// Device ID to target (required if multiple devices are connected)
        device_id: Option<String>,

        /// Additional arguments to pass to dmesg
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Get device properties
    Getprop {
        /// Comma-separated list of property names to query. If empty, all properties will be shown
        #[arg(default_value = "")]
        propnames: String,

        /// Device ID (required if multiple devices are connected)
        device_id: Option<String>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputType::Plain)]
        output: OutputType,
    },

    /// Lists connected devices
    Ls {
        /// Output format (table, json, or plain)
        #[arg(short = 'o', long, value_enum, default_value_t = OutputType::Table)]
        output: OutputType,
    },

    /// Run perfetto trace
    Perfetto {
        /// Config file path
        #[arg(short = 'f', long = "config", required = true)]
        config: PathBuf,

        /// Optional device ID (can be partial)
        device_id: Option<String>,

        /// Output file location
        #[arg(short = 'o', long = "output", required = true)]
        output: PathBuf,

        /// Time to run trace in seconds (if not specified, runs until 'q' is pressed)
        #[arg(short = 't', long = "time")]
        time: Option<u32>,
    },

    /// Rename a device
    Rename {
        /// Current device ID (can be partial)
        device_id: String,
        /// New name for the device
        new_name: String,
    },

    /// Runs a command on a device
    Run {
        /// The command to execute
        command: String,
        /// Optional device ID (can be partial)
        device_id: Option<String>,
        /// Filter devices by property (format: key=value)
        #[arg(short = 'f', long = "filter", num_args = 1)]
        filters: Vec<String>,
        /// Watch mode - repeat command every second. Optional value specifies duration in seconds
        #[arg(short = 'w', long = "watch", num_args = 0..=1, default_missing_value = "0")]
        watch: Option<u32>,
    },

    /// Record screen
    Screenrecord {
        /// Optional device ID (can be partial)
        device_id: Option<String>,

        /// Output file location (overrides default location)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Additional arguments to pass to screenrecord
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Take a screenshot
    Screenshot {
        /// Additional arguments to pass to screencap
        #[arg(last = true)]
        args: Vec<String>,

        /// Optional device ID (can be partial)
        device_id: Option<String>,

        /// Interactive mode - take screenshots with spacebar
        #[arg(short = 'i', long = "interactive")]
        interactive: bool,

        /// Output file location (overrides default location)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Manage ADB server
    Server {
        /// Server operation to perform (defaults to status)
        #[arg(value_enum, default_value = "status")]
        operation: ServerOperation,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ServerOperation {
    Start,
    Stop,
    Restart,
    Status,
}

impl Cli {
    pub fn command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Ls {
            output: OutputType::Table,
        })
    }
}
