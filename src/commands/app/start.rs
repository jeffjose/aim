use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use async_trait::async_trait;
use colored::*;

pub struct StartCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct StartArgs {
    /// Package name (supports partial matching)
    pub package: String,
    
    /// Activity to start (default: main launcher activity)
    #[clap(short, long)]
    pub activity: Option<String>,
}

impl StartCommand {
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
    
    async fn get_launcher_activity(&self, ctx: &CommandContext, package: &str) -> Result<String> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Get the main launcher activity
        let cmd = format!("cmd package resolve-activity --brief {} | tail -n 1", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        let result = output.stdout.trim();
        
        // Check if we got an activity (format: package/activity)
        if result.contains('/') {
            Ok(result.to_string())
        } else {
            // Try alternative method using dumpsys
            let dump_cmd = format!("dumpsys package {} | grep -A1 'android.intent.action.MAIN' | grep '{}'", package, package);
            let dump_shell = crate::adb::shell::ShellCommand::new(dump_cmd)
                .with_device(device.id.clone());
            
            if let Ok(dump_output) = dump_shell.execute(host, port).await {
                if let Some(line) = dump_output.stdout.lines().next() {
                    // Extract activity from line like: "        com.example/.MainActivity filter ..."
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if let Some(activity) = parts.first() {
                        return Ok(activity.to_string());
                    }
                }
            }
            
            Err(AimError::CommandExecution(format!("Could not find launcher activity for {}", package)))
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
impl SubCommand for StartCommand {
    type Args = StartArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Find the full package name
        let package = self.find_package(ctx, &args.package).await?;
        
        // Get app name for display
        let app_name = self.get_app_name(ctx, &package).await?;
        
        println!("Starting app: {}", app_name.bright_cyan());
        println!("Package: {}", package.bright_cyan());
        
        // Determine activity to start
        let activity = if let Some(act) = args.activity {
            // User specified activity
            if act.starts_with('.') {
                // Relative activity name
                format!("{}/{}", package, act)
            } else if act.contains('/') {
                // Full activity name
                act
            } else {
                // Assume it's relative
                format!("{}/.{}", package, act)
            }
        } else {
            // Find launcher activity
            println!("Finding launcher activity...");
            self.get_launcher_activity(ctx, &package).await?
        };
        
        println!("Activity: {}", activity.bright_cyan());
        
        // Start the app
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        let cmd = format!("am start -n {}", activity);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        
        // Check result
        if output.stdout.contains("Error") {
            return Err(AimError::CommandExecution(format!(
                "Failed to start app: {}",
                output.stdout.trim()
            )));
        }
        
        if output.stdout.contains("Warning: Activity not started") {
            println!("{} App was already in foreground", "⚠".yellow());
        } else {
            println!("{} App started successfully", "✓".green());
        }
        
        Ok(())
    }
}