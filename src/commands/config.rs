use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use colored::*;

pub struct ConfigCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct ConfigArgs {
    /// Show configuration file path only
    #[clap(short = 'p', long = "path")]
    pub path_only: bool,
}

impl ConfigCommand {
    pub fn new() -> Self {
        Self
    }
    
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AimError::Configuration("Could not determine config directory".to_string()))?;
        
        Ok(config_dir.join("aim").join("config.toml"))
    }
}

#[async_trait]
impl SubCommand for ConfigCommand {
    type Args = ConfigArgs;
    
    async fn run(&self, _ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        if args.path_only {
            println!("{}", config_path.display());
            return Ok(());
        }
        
        if !config_path.exists() {
            println!("No config file found at: {}", config_path.display().to_string().bright_cyan());
            println!("Default configuration will be used.");
            return Ok(());
        }
        
        println!("Reading {}\n", config_path.display().to_string().bright_cyan());
        let contents = std::fs::read_to_string(&config_path)?;
        println!("{}", contents);
        
        Ok(())
    }
}