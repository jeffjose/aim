use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use async_trait::async_trait;
use colored::*;

pub struct UninstallCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct UninstallArgs {
    /// Package name (supports partial matching)
    pub package: String,
    
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
    
    /// Keep app data and cache
    #[clap(short, long)]
    pub keep_data: bool,
    
    /// Skip confirmation prompt
    #[clap(short = 'y', long)]
    pub yes: bool,
}

impl UninstallCommand {
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
    
    async fn get_app_info(&self, ctx: &CommandContext, package: &str) -> Result<(String, bool)> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Try to get app label
        let cmd = format!("cmd package resolve-activity --brief {} | tail -n 1", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let app_name = if let Ok(output) = shell_cmd.execute(host, port).await {
            let name = output.stdout.trim();
            if !name.is_empty() && !name.contains('/') {
                name.to_string()
            } else {
                package.to_string()
            }
        } else {
            package.to_string()
        };
        
        // Check if it's a system app
        let info_cmd = format!("dumpsys package {} | grep codePath", package);
        let info_shell = crate::adb::shell::ShellCommand::new(info_cmd)
            .with_device(device.id.clone());
        
        let is_system = if let Ok(output) = info_shell.execute(host, port).await {
            output.stdout.contains("/system/") || output.stdout.contains("/vendor/")
        } else {
            false
        };
        
        Ok((app_name, is_system))
    }
    
    fn confirm_uninstall(&self, package: &str, app_name: &str, is_system: bool, keep_data: bool) -> Result<bool> {
        use std::io::{self, Write};
        
        println!();
        if is_system {
            println!("{}", "WARNING: This is a system app!".red().bold());
        } else {
            println!("{}", "WARNING: This will uninstall the app!".yellow().bold());
        }
        
        println!("Package: {}", package.bright_cyan());
        println!("App: {}", app_name.bright_cyan());
        
        if keep_data {
            println!("Mode: {} (app data will be preserved)", "Keep data".green());
        } else {
            println!("Mode: {} (all app data will be deleted)", "Full uninstall".red());
        }
        
        println!();
        print!("Are you sure you want to uninstall this app? [y/N] ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}

#[async_trait]
impl SubCommand for UninstallCommand {
    type Args = UninstallArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Find the full package name
        let package = self.find_package(ctx, &args.package).await?;
        
        // Get app info
        let (app_name, is_system) = self.get_app_info(ctx, &package).await?;
        
        // Confirm unless --yes flag is set
        if !args.yes && !self.confirm_uninstall(&package, &app_name, is_system, args.keep_data)? {
            println!("Operation cancelled.");
            return Ok(());
        }
        
        // Uninstall the app
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        let cmd = if args.keep_data {
            format!("pm uninstall -k {}", package)
        } else {
            format!("pm uninstall {}", package)
        };
        
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        println!("Uninstalling {}...", app_name.bright_cyan());
        
        let output = shell_cmd.execute(host, port).await?;
        
        // Check result
        let result = output.stdout.trim();
        if result == "Success" {
            println!("{} App uninstalled successfully", "✓".green());
            if args.keep_data {
                println!("App data has been preserved and can be restored if the app is reinstalled.");
            }
        } else if result.contains("DELETE_FAILED_INTERNAL_ERROR") {
            return Err(AimError::CommandExecution(
                "Failed to uninstall: Internal error. The app might be a system app that cannot be uninstalled.".to_string()
            ));
        } else if result.contains("DELETE_FAILED_DEVICE_POLICY_MANAGER") {
            return Err(AimError::CommandExecution(
                "Failed to uninstall: App is a device administrator. Disable it in Settings > Security > Device administrators first.".to_string()
            ));
        } else {
            return Err(AimError::CommandExecution(format!(
                "Failed to uninstall: {}",
                result
            )));
        }
        
        Ok(())
    }
}