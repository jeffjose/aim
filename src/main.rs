mod cli;
mod device;
mod error;
mod library;
mod subcommands;
mod types;

use clap::Parser;
use cli::{Cli, Commands};
use log::{debug, error};

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
        Commands::Getprop { propname, device_id } => {
            let target_device = match device_id {
                // If device_id is provided, do partial match
                Some(id) => {
                    devices.iter()
                        .find(|d| d.matches_id_prefix(&id))
                        .ok_or_else(|| error::AdbError::DeviceNotFound(id.clone()))?
                }
                // If no device_id provided, check if there's exactly one device
                None => {
                    if devices.len() == 1 {
                        &devices[0]
                    } else {
                        return Err(error::AdbError::DeviceNotFound("No device ID provided and multiple devices found".to_string()).into());
                    }
                }
            };

            if let Err(e) = subcommands::getprop::run(&cli.host, &cli.port, &propname, Some(target_device)).await {
                error!("{}", e);
                std::process::exit(1);
            }
        }
        Commands::Getprops { propnames, device_id } => {
            let target_device = match device_id {
                // If device_id is provided, do partial match
                Some(id) => {
                    devices.iter()
                        .find(|d| d.matches_id_prefix(&id))
                        .ok_or_else(|| error::AdbError::DeviceNotFound(id.clone()))?
                }
                // If no device_id provided, check if there's exactly one device
                None => {
                    if devices.len() == 1 {
                        &devices[0]
                    } else {
                        return Err(error::AdbError::DeviceNotFound("No device ID provided and multiple devices found".to_string()).into());
                    }
                }
            };

            if let Err(e) = subcommands::getprops::run(&cli.host, &cli.port, &propnames, Some(target_device)).await {
                error!("{}", e);
                std::process::exit(1);
            }
        }
        Commands::Inspect { id } => {
            subcommands::command::run(&cli.host, &cli.port, &id);
        }
    }

    Ok(())
}
