mod cli;
mod library;
mod subcommands;
mod types;

use cli::{Cli, Commands};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match cli.command() {
        Commands::Ls => {
            subcommands::ls::run(&cli.host, &cli.port, cli.output).await;
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
