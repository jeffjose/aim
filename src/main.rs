mod cli;
mod config;
mod device;
mod error;
mod library;
mod subcommands;
mod types;

use std::path::PathBuf;

use clap::Parser;
use cli::{Cli, Commands};
use device::device_info;
use log::{debug, error};

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
            // Remove the alias argument but keep any additional args
            let additional_args = if args.len() > 2 {
                args.split_off(2)
            } else {
                Vec::new()
            };

            // Remove the alias
            args.remove(1);

            // Split the resolved command and insert all parts
            let resolved_parts: Vec<String> =
                resolved.split_whitespace().map(String::from).collect();
            args.splice(1..1, resolved_parts);

            // Add back any additional arguments
            args.extend(additional_args);
        }
    }

    debug!("{:?}", args);
    Cli::parse_from(args)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_args();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match cli.command() {
        Commands::Server { operation } => {
            if let Err(e) = subcommands::server::run(&cli.host, &cli.port, &operation).await {
                error!("Server operation failed: {}", e);
                std::process::exit(1);
            }
            return Ok(());
        }
        _ => {
            // Get device list before running any commands
            let devices = device_info::get_devices(&cli.host, &cli.port).await;
            debug!("Found {} devices", devices.len());

            // Check if any devices were found (except for the 'ls' command which should work regardless)
            if devices.is_empty() && !matches!(cli.command(), Commands::Ls) {
                return Err(error::AdbError::NoDevicesFound.into());
            }

            match cli.command() {
                Commands::Ls => {
                    subcommands::ls::run(&devices, cli.output).await;
                }
                Commands::Command { command, device_id, filter } => {
                    let target_device = if let Some(ref id) = device_id {
                        Some(device_info::find_target_device(&devices, Some(id))?)
                    } else {
                        None
                    };

                    if let Err(e) = subcommands::command::run(
                        &cli.host,
                        &cli.port,
                        &command,
                        target_device,
                        filter.as_deref(),
                    )
                    .await
                    {
                        error!("{}", e);
                        std::process::exit(1);
                    }
                }
                Commands::Getprop {
                    propnames,
                    device_id,
                    output,
                } => {
                    let target_device =
                        device_info::find_target_device(&devices, device_id.as_ref())?;

                    if let Err(e) = subcommands::getprop::run(
                        &cli.host,
                        &cli.port,
                        &propnames,
                        Some(target_device),
                        output,
                    )
                    .await
                    {
                        error!("{}", e);
                        std::process::exit(1);
                    }
                }
                Commands::Rename {
                    device_id,
                    new_name,
                } => {
                    let target_device =
                        device_info::find_target_device(&devices, Some(&device_id))?;
                    if let Err(e) = subcommands::rename::run(target_device, &new_name).await {
                        error!("Failed to rename device: {}", e);
                        std::process::exit(1);
                    }
                }
                Commands::Copy { src, dst } => {
                    subcommands::copy::run(
                        subcommands::copy::CopyArgs {
                            src: src.into_iter().map(PathBuf::from).collect(),
                            dst: PathBuf::from(dst),
                        },
                        &devices,
                        &cli.host,
                        &cli.port,
                    )
                    .await?
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}
