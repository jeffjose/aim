mod adb;
mod cli;
mod commands;
mod config;
mod core;
mod device;
mod error;
mod library;
mod output;
mod progress;
mod subcommands;
mod types;
mod utils;

#[cfg(test)]
mod testing;

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
            if devices.is_empty() && !matches!(cli.command(), Commands::Ls { output: _ }) {
                return Err(error::AdbError::NoDevicesFound.into());
            }

            match cli.command() {
                Commands::Ls { output } => {
                    subcommands::ls::run(&devices, output).await;
                }
                Commands::Run {
                    command,
                    device_id,
                    filters,
                    watch,
                } => {
                    let target_device = if devices.len() == 1 {
                        devices.first().unwrap()
                    } else {
                        if device_id.is_none() {
                            return Err(error::AdbError::DeviceIdRequired.into());
                        }
                        device_info::find_target_device(&devices, device_id.as_ref())?
                    };

                    if let Err(e) = subcommands::run::run(
                        &cli.host,
                        &cli.port,
                        &command,
                        Some(target_device),
                        if filters.is_empty() {
                            None
                        } else {
                            Some(&filters)
                        },
                        watch,
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
                    let props_vec = if propnames.is_empty() {
                        Vec::new()
                    } else {
                        propnames
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    };

                    // Handle single argument case - try device ID first if only one arg provided
                    let (props, target_device) = if device_id.is_none() && !propnames.is_empty() {
                        // Try to match the propnames as a device ID first
                        if let Ok(matched_device) =
                            device_info::find_target_device(&devices, Some(&propnames))
                        {
                            // If it matches a device, use it as the target and get all properties
                            (&[][..], matched_device)
                        } else if devices.len() == 1 {
                            // If no device match and only one device, treat as property names
                            (&props_vec[..], devices.first().unwrap())
                        } else {
                            // Multiple devices but no valid device ID
                            return Err(error::AdbError::DeviceIdRequired.into());
                        }
                    } else if let Some(device_id) = device_id {
                        // Two arguments provided - second is device ID
                        let target_device =
                            device_info::find_target_device(&devices, Some(&device_id))?;
                        (&props_vec[..], target_device)
                    } else {
                        // No arguments or empty propnames - must have single device
                        if devices.len() > 1 {
                            return Err(error::AdbError::DeviceIdRequired.into());
                        }
                        (&[][..], devices.first().unwrap())
                    };

                    if let Err(e) = subcommands::getprop::run(
                        &cli.host,
                        &cli.port,
                        props,
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
                Commands::Adb { command, device_id } => {
                    let target_device =
                        device_info::find_target_device(&devices, device_id.as_ref())?;

                    subcommands::adb::run(subcommands::adb::AdbArgs {
                        command: &command,
                        adb_id: &target_device.adb_id,
                    })
                    .await?;
                }
                Commands::Config => subcommands::config::run().await?,
                Commands::Perfetto {
                    config,
                    time,
                    output,
                    device_id,
                } => {
                    let target_device =
                        device_info::find_target_device(&devices, device_id.as_ref())?;

                    subcommands::perfetto::run(
                        subcommands::perfetto::PerfettoArgs {
                            config,
                            time,
                            output,
                            device_id,
                        },
                        target_device,
                        &cli.host,
                        &cli.port,
                    )
                    .await?
                }
                Commands::Screenshot {
                    device_id,
                    output,
                    interactive,
                    args,
                } => {
                    let target_device =
                        device_info::find_target_device(&devices, device_id.as_ref())?;

                    subcommands::screenshot::run(
                        subcommands::screenshot::ScreenshotArgs {
                            device_id,
                            output,
                            interactive,
                            args,
                        },
                        target_device,
                        &cli.host,
                        &cli.port,
                    )
                    .await?
                }
                Commands::Screenrecord {
                    device_id,
                    output,
                    args,
                } => {
                    let target_device =
                        device_info::find_target_device(&devices, device_id.as_ref())?;

                    subcommands::screenrecord::run(
                        subcommands::screenrecord::ScreenrecordArgs {
                            device_id,
                            output,
                            args,
                        },
                        target_device,
                        &cli.host,
                        &cli.port,
                    )
                    .await?
                }
                Commands::Dmesg { device_id, args } => {
                    let target_device =
                        device_info::find_target_device(&devices, device_id.as_ref())?;

                    subcommands::dmesg::run(
                        subcommands::dmesg::DmesgArgs { device_id, args },
                        target_device,
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
