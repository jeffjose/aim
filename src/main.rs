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
            // App commands use DeviceManager for device selection
            use crate::core::context::CommandContext;
            use crate::device::DeviceManager;

            let device_manager = DeviceManager::with_address(&cli.host, &cli.port);
            let device_id_arg = command.device_id();

            // Get target device using DeviceManager
            let device = device_manager
                .get_target_device(device_id_arg.as_deref())
                .await?;

            let ctx = CommandContext::new().with_device(device);

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