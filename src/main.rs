mod cli;
mod device;
mod library;
mod subcommands;
mod types;

use clap::Parser;
use cli::{Cli, Commands};
use log::debug;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    // Get device list before running any commands
    let devices = device::device_info::get_devices(&cli.host, &cli.port).await;
    debug!("Found {} devices", devices.len());

    match cli.command() {
        Commands::Ls => {
            subcommands::ls::display_devices(&devices, cli.output);
        }
        Commands::Command { command } => {
            subcommands::command::run(&cli.host, &cli.port, &command);
        }
        Commands::Getprop { propname } => {
            subcommands::getprop::run(&cli.host, &cli.port, &propname);
        }
        Commands::Getprops { propnames } => {
            subcommands::getprops::run(&cli.host, &cli.port, &propnames).await;
        }
        Commands::Inspect { id } => {
            subcommands::command::run(&cli.host, &cli.port, &id);
        }
    }

    Ok(())
}
