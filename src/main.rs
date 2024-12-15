mod cli;
mod config;
mod device;
mod error;
mod library;
mod subcommands;
mod types;

use clap::Parser;
use cli::{Cli, Commands};
use log::{debug, error};
use types::DeviceDetails;

fn parse_args() -> Cli {
    let config = config::Config::load();

    // Get raw args and check if the first argument is an alias
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let potential_alias = &args[1];
        let resolved = config.resolve_alias(potential_alias);
        debug!("Potential alias: {}", potential_alias);
        debug!("Resolved alias: {}", resolved);
        if resolved != *potential_alias {
            // Remove the alias argument
            args.remove(1);
            // Split the resolved command and insert all parts
            let resolved_parts: Vec<String> =
                resolved.split_whitespace().map(String::from).collect();
            args.splice(1..1, resolved_parts);
        }
    }

    debug!("{:?}", args);
    Cli::parse_from(args)
}

fn find_target_device<'a>(
    devices: &'a [DeviceDetails],
    device_id: Option<&String>,
) -> Result<&'a DeviceDetails, Box<dyn std::error::Error>> {
    match device_id {
        Some(id) => devices
            .iter()
            .find(|d| d.matches_id_prefix(id))
            .ok_or_else(|| error::AdbError::DeviceNotFound(id.clone()).into()),
        None => {
            if devices.len() == 1 {
                Ok(&devices[0])
            } else {
                Err(error::AdbError::DeviceNotFound(
                    "No device ID provided and multiple devices found".to_string(),
                )
                .into())
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_args();

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
        Commands::Getprop {
            propname,
            device_id,
            output,
        } => {
            let target_device = find_target_device(&devices, device_id.as_ref())?;

            if let Err(e) = subcommands::getprop::run(
                &cli.host,
                &cli.port,
                &propname,
                Some(target_device),
                output,
            )
            .await
            {
                error!("{}", e);
                std::process::exit(1);
            }
        }
        Commands::Getprops {
            propnames,
            device_id,
        } => {
            let target_device = find_target_device(&devices, device_id.as_ref())?;

            if let Err(e) =
                subcommands::getprops::run(&cli.host, &cli.port, &propnames, Some(target_device))
                    .await
            {
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
