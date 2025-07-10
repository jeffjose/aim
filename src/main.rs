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
mod types;
mod utils;

#[cfg(test)]
mod testing;

use clap::Parser;
use cli::{Cli, Commands};
use device::device_info;
use log::debug;

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

            // Append any additional args after the resolved command
            args.extend(additional_args);
        }
    }

    debug!("Final args: {:?}", args);
    Cli::parse_from(args)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_args();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    debug!("Starting aim with command: {:?}", cli.command());
    
    // Use CommandRunner for all non-app commands
    match &cli.command() {
        Commands::App { command } => {
            // App commands still use the old routing for now
            let devices = device_info::get_devices(&cli.host, &cli.port).await;
            debug!("Found {} devices", devices.len());

            if devices.is_empty() {
                return Err(error::AdbError::NoDevicesFound.into());
            }
            
            // For app commands, we need to create a CommandContext
            use crate::core::context::CommandContext;
            use crate::core::types::{Device, DeviceId, DeviceState};
            
            // Get device_id from the app subcommand
            let device_id_arg = command.device_id();
            
            // Find target device based on device_id argument
            let device = if !devices.is_empty() {
                let target_device = if let Some(device_id) = device_id_arg {
                    // User specified a device ID
                    device_info::find_target_device(&devices, Some(&device_id.to_string()))?
                } else if devices.len() == 1 {
                    // Only one device connected, use it
                    &devices[0]
                } else {
                    // Multiple devices but no device ID specified
                    return Err(error::AdbError::DeviceIdRequired.into());
                };
                
                Some(Device::new(DeviceId::new(&target_device.adb_id))
                    .with_state(DeviceState::Device)
                    .with_model(target_device.model.clone().unwrap_or_default())
                    .with_product(target_device.product.clone().unwrap_or_default())
                    .with_device(target_device.device.clone().unwrap_or_default()))
            } else {
                // No devices connected
                None
            };
            
            let ctx = CommandContext::new()
                .with_device(device.unwrap_or_else(|| Device::new(DeviceId::new(""))));
            
            crate::commands::app::run(&ctx, command.clone()).await?
        }
        _ => {
            // Use CommandRunner for all other commands
            debug!("Using CommandRunner for command");
            use crate::commands::runner::CommandRunner;
            
            debug!("Creating CommandRunner...");
            let runner = CommandRunner::new().await?;
            debug!("Running command through CommandRunner...");
            runner.run(cli).await?;
        }
    }

    Ok(())
}