use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use async_trait::async_trait;
use colored::*;

pub struct StopCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct StopArgs {
    /// Package name (supports partial matching)
    pub package: String,
    
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
}

impl StopCommand {
    pub fn new() -> Self {
        Self
    }
    
    async fn find_package(&self, ctx: &CommandContext, partial: &str) -> Result<String> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Get all packages
        let cmd = "pm list packages".to_string();
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        
        // Find matching packages
        let matches: Vec<String> = output.stdout
            .lines()
            .filter_map(|line| {
                if let Some(pkg) = line.strip_prefix("package:") {
                    if pkg.contains(partial) {
                        return Some(pkg.to_string());
                    }
                }
                None
            })
            .collect();
            
        match matches.len() {
            0 => Err(AimError::CommandExecution(format!("No package found matching '{}'", partial))),
            1 => Ok(matches[0].clone()),
            _ => {
                // If there's an exact match, use it
                if let Some(exact) = matches.iter().find(|&m| m == partial) {
                    Ok(exact.clone())
                } else {
                    Err(AimError::AmbiguousDeviceMatch {
                        prefix: partial.to_string(),
                        matches,
                    })
                }
            }
        }
    }
    
    async fn get_app_name(&self, ctx: &CommandContext, package: &str) -> Result<String> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Try to get app label
        let cmd = format!("cmd package resolve-activity --brief {} | tail -n 1", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        if let Ok(output) = shell_cmd.execute(host, port).await {
            let name = output.stdout.trim();
            if !name.is_empty() && !name.contains('/') {
                return Ok(name.to_string());
            }
        }
        
        // Fallback to package name
        Ok(package.to_string())
    }
}

#[async_trait]
impl SubCommand for StopCommand {
    type Args = StopArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Find the full package name
        let package = self.find_package(ctx, &args.package).await?;
        
        // Get app name for display
        let app_name = self.get_app_name(ctx, &package).await?;
        
        println!("Stopping app: {}", app_name.bright_cyan());
        println!("Package: {}", package.bright_cyan());
        
        // Force stop the app
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        let cmd = format!("am force-stop {}", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        shell_cmd.execute(host, port).await?;
        
        // Verify the app was stopped by checking if it's running
        let check_cmd = format!("pidof {}", package);
        let check_shell = crate::adb::shell::ShellCommand::new(check_cmd)
            .with_device(device.id.clone());
        
        if let Ok(output) = check_shell.execute(host, port).await {
            if output.stdout.trim().is_empty() {
                println!("{} App stopped successfully", "✓".green());
            } else {
                // App might still be running in some processes
                println!("{} App force-stopped (some services may restart automatically)", "✓".yellow());
            }
        } else {
            // Assume success if we can't check
            println!("{} App force-stopped", "✓".green());
        }
        
        Ok(())
    }
}