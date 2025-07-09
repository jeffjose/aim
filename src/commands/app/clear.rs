use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use async_trait::async_trait;
use colored::*;

pub struct ClearCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct ClearArgs {
    /// Package name (supports partial matching)
    pub package: String,
    
    /// Skip confirmation prompt
    #[clap(short = 'y', long)]
    pub yes: bool,
}

impl ClearCommand {
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
    
    fn confirm_clear(&self, package: &str, app_name: &str) -> Result<bool> {
        use std::io::{self, Write};
        
        println!();
        println!("{}", "WARNING: This will clear all app data!".yellow().bold());
        println!("Package: {}", package.bright_cyan());
        println!("App: {}", app_name.bright_cyan());
        println!();
        print!("Are you sure you want to clear all data for this app? [y/N] ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}

#[async_trait]
impl SubCommand for ClearCommand {
    type Args = ClearArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Find the full package name
        let package = self.find_package(ctx, &args.package).await?;
        
        // Get app name for confirmation
        let app_name = self.get_app_name(ctx, &package).await?;
        
        // Confirm unless --yes flag is set
        if !args.yes && !self.confirm_clear(&package, &app_name)? {
            println!("Operation cancelled.");
            return Ok(());
        }
        
        // Clear app data
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        let cmd = format!("pm clear {}", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        
        // Check result
        if output.stdout.trim() == "Success" {
            println!("{} App data cleared successfully", "✓".green());
            println!("Package: {}", package.bright_cyan());
            println!("App: {}", app_name.bright_cyan());
        } else {
            return Err(AimError::CommandExecution(format!(
                "Failed to clear app data: {}",
                output.stdout.trim()
            )));
        }
        
        Ok(())
    }
}