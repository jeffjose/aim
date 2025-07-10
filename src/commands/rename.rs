use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use colored::*;

pub struct RenameCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct RenameArgs {
    /// Current device ID (can be partial)
    pub device_id: String,
    
    /// New name for the device
    pub new_name: String,
}

impl RenameCommand {
    pub fn new() -> Self {
        Self
    }
    
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AimError::Configuration("Could not determine config directory".to_string()))?;
        
        let aim_config_dir = config_dir.join("aim");
        if !aim_config_dir.exists() {
            fs::create_dir_all(&aim_config_dir)?;
        }
        
        Ok(aim_config_dir.join("config.toml"))
    }
}

#[async_trait]
impl SubCommand for RenameCommand {
    type Args = RenameArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        
        // Verify the device ID matches
        if !device.id.to_string().contains(&args.device_id) {
            return Err(AimError::InvalidArgument(
                format!("Device '{}' not found", args.device_id)
            ));
        }
        
        let config_path = Self::get_config_path()?;
        
        // Read existing config
        let mut config_content = if config_path.exists() {
            fs::read_to_string(&config_path)?
        } else {
            String::new()
        };
        
        // Create device section if it doesn't exist
        let device_section = format!("[device.{}]", device.id);
        let name_entry = format!("name = \"{}\"", args.new_name);
        
        if config_content.contains(&device_section) {
            // Update existing entry
            let lines: Vec<String> = config_content.lines().map(String::from).collect();
            let mut new_lines = Vec::new();
            let mut in_device_section = false;
            let mut name_updated = false;
            
            for line in lines {
                if line.trim() == device_section {
                    in_device_section = true;
                    new_lines.push(line);
                } else if in_device_section && line.trim().starts_with("name =") {
                    new_lines.push(name_entry.clone());
                    name_updated = true;
                } else if in_device_section && line.trim().starts_with('[') {
                    if !name_updated {
                        new_lines.push(name_entry.clone());
                    }
                    in_device_section = false;
                    new_lines.push(line);
                } else {
                    new_lines.push(line);
                }
            }
            
            if in_device_section && !name_updated {
                new_lines.push(name_entry);
            }
            
            config_content = new_lines.join("\n");
        } else {
            // Add new section
            if !config_content.is_empty() && !config_content.ends_with('\n') {
                config_content.push('\n');
            }
            config_content.push_str(&device_section);
            config_content.push('\n');
            config_content.push_str(&name_entry);
            config_content.push('\n');
        }
        
        // Write config
        fs::write(&config_path, config_content)?;
        
        println!("Device {} renamed to '{}'", 
            device.id.to_string().bright_cyan(),
            args.new_name.bright_green()
        );
        
        Ok(())
    }
}