use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::core::types::OutputFormat;
use crate::error::{AimError, Result};
use crate::output::OutputFormatter;
use async_trait::async_trait;
use colored::*;
use serde::{Deserialize, Serialize};

pub struct PermissionsCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct PermissionsArgs {
    /// Package name (supports partial matching)
    pub package: String,
    
    /// Show only granted permissions
    #[clap(short, long)]
    pub granted: bool,
    
    /// Show only denied permissions
    #[clap(short, long)]
    pub denied: bool,
    
    /// Output format
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "table")]
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
    pub granted: bool,
    pub flags: Vec<String>,
    pub description: String,
}

impl PermissionsCommand {
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
    
    async fn get_permissions(&self, ctx: &CommandContext, package: &str) -> Result<Vec<Permission>> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Get permissions via dumpsys
        let cmd = format!("dumpsys package {}", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        
        let mut permissions = Vec::new();
        let mut in_requested = false;
        let mut in_install = false;
        let mut in_runtime = false;
        
        // Also get granted runtime permissions
        let mut granted_runtime = std::collections::HashSet::new();
        
        for line in output.stdout.lines() {
            let trimmed = line.trim();
            
            // Track sections
            if trimmed == "requested permissions:" {
                in_requested = true;
                in_install = false;
                in_runtime = false;
            } else if trimmed == "install permissions:" {
                in_install = true;
                in_requested = false;
                in_runtime = false;
            } else if trimmed.contains("runtime permissions:") {
                in_runtime = true;
                in_requested = false;
                in_install = false;
            } else if trimmed.starts_with("User ") || 
                      trimmed.starts_with("android.permission") && !in_requested && !in_install && !in_runtime {
                // End of permissions sections
                in_requested = false;
                in_install = false;
                in_runtime = false;
            }
            
            // Parse permissions
            if in_requested && trimmed.starts_with("android.permission.") {
                permissions.push(Permission {
                    name: trimmed.to_string(),
                    granted: false, // Will update later
                    flags: vec!["requested".to_string()],
                    description: self.get_permission_description(trimmed),
                });
            } else if in_install && trimmed.starts_with("android.permission.") {
                // Install permissions are always granted
                let perm_name = trimmed.split(':').next().unwrap_or(trimmed);
                if let Some(perm) = permissions.iter_mut().find(|p| p.name == perm_name) {
                    perm.granted = true;
                    perm.flags.push("install".to_string());
                } else {
                    permissions.push(Permission {
                        name: perm_name.to_string(),
                        granted: true,
                        flags: vec!["install".to_string()],
                        description: self.get_permission_description(perm_name),
                    });
                }
            } else if in_runtime && trimmed.contains("android.permission.") {
                // Runtime permissions show granted status
                if let Some(perm_start) = trimmed.find("android.permission.") {
                    let perm_part = &trimmed[perm_start..];
                    if let Some(colon_pos) = perm_part.find(':') {
                        let perm_name = &perm_part[..colon_pos];
                        if trimmed.contains("granted=true") {
                            granted_runtime.insert(perm_name.to_string());
                        }
                    }
                }
            }
        }
        
        // Update runtime permission status
        for perm in &mut permissions {
            if granted_runtime.contains(&perm.name) {
                perm.granted = true;
                perm.flags.push("runtime".to_string());
            } else if !perm.flags.contains(&"install".to_string()) {
                perm.flags.push("runtime".to_string());
            }
        }
        
        // Sort by name
        permissions.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(permissions)
    }
    
    fn get_permission_description(&self, permission: &str) -> String {
        // Common permission descriptions
        match permission {
            "android.permission.INTERNET" => "Full network access",
            "android.permission.CAMERA" => "Take pictures and videos",
            "android.permission.RECORD_AUDIO" => "Record audio",
            "android.permission.ACCESS_FINE_LOCATION" => "Precise location (GPS)",
            "android.permission.ACCESS_COARSE_LOCATION" => "Approximate location",
            "android.permission.READ_CONTACTS" => "Read your contacts",
            "android.permission.WRITE_CONTACTS" => "Modify your contacts",
            "android.permission.READ_CALENDAR" => "Read calendar events",
            "android.permission.WRITE_CALENDAR" => "Add or modify calendar events",
            "android.permission.READ_EXTERNAL_STORAGE" => "Read storage contents",
            "android.permission.WRITE_EXTERNAL_STORAGE" => "Modify or delete storage contents",
            "android.permission.READ_PHONE_STATE" => "Read phone status and identity",
            "android.permission.CALL_PHONE" => "Directly call phone numbers",
            "android.permission.READ_SMS" => "Read text messages",
            "android.permission.SEND_SMS" => "Send SMS messages",
            "android.permission.RECEIVE_SMS" => "Receive text messages",
            "android.permission.VIBRATE" => "Control vibration",
            "android.permission.WAKE_LOCK" => "Prevent phone from sleeping",
            "android.permission.ACCESS_WIFI_STATE" => "View Wi-Fi connections",
            "android.permission.CHANGE_WIFI_STATE" => "Connect and disconnect from Wi-Fi",
            "android.permission.BLUETOOTH" => "Pair with Bluetooth devices",
            "android.permission.BLUETOOTH_ADMIN" => "Access Bluetooth settings",
            "android.permission.NFC" => "Control Near Field Communication",
            "android.permission.USE_FINGERPRINT" => "Use fingerprint hardware",
            "android.permission.USE_BIOMETRIC" => "Use biometric hardware",
            "android.permission.BODY_SENSORS" => "Access body sensors",
            _ => "",
        }.to_string()
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
impl SubCommand for PermissionsCommand {
    type Args = PermissionsArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Find the full package name
        let package = self.find_package(ctx, &args.package).await?;
        
        // Get app name for display
        let app_name = self.get_app_name(ctx, &package).await?;
        
        // Get permissions
        let mut permissions = self.get_permissions(ctx, &package).await?;
        
        // Filter if requested
        if args.granted {
            permissions.retain(|p| p.granted);
        } else if args.denied {
            permissions.retain(|p| !p.granted);
        }
        
        if permissions.is_empty() {
            let msg = if args.granted {
                "No granted permissions found"
            } else if args.denied {
                "No denied permissions found"
            } else {
                "No permissions found"
            };
            
            // For JSON output, return empty array
            let output_format = OutputFormat::from_str(&args.output)
                .ok_or_else(|| AimError::InvalidArgument(format!("Invalid output format: {}", args.output)))?;
            
            if output_format == OutputFormat::Json {
                let formatter = OutputFormatter::new();
                formatter.json(&permissions)?;
            } else {
                println!("{}", msg);
            }
            return Ok(());
        }
        
        // Get output format
        let output_format = OutputFormat::from_str(&args.output)
            .ok_or_else(|| AimError::InvalidArgument(format!("Invalid output format: {}", args.output)))?;
        
        let formatter = OutputFormatter::new();
        
        match output_format {
            OutputFormat::Plain => {
                if !ctx.quiet {
                    println!("Getting permissions for: {}", app_name.bright_cyan());
                    println!("Package: {}", package.bright_cyan());
                    println!();
                }
                
                let granted_count = permissions.iter().filter(|p| p.granted).count();
                let denied_count = permissions.len() - granted_count;
                
                println!("Total permissions: {}", permissions.len());
                println!("Granted: {} | Denied: {}", 
                    granted_count.to_string().green(),
                    denied_count.to_string().red()
                );
                println!();
                
                for perm in &permissions {
                    let status = if perm.granted { 
                        "GRANTED".green() 
                    } else { 
                        "DENIED".red() 
                    };
                    let flags = perm.flags.join(", ");
                    
                    println!("{} [{}] {}", 
                        status,
                        flags.bright_black(),
                        perm.name.bright_white()
                    );
                    
                    if !perm.description.is_empty() {
                        println!("  {}", perm.description.bright_black());
                    }
                }
            }
            OutputFormat::Table => {
                formatter.table(&permissions)?;
            }
            OutputFormat::Json => {
                formatter.json(&permissions)?;
            }
        }
        
        Ok(())
    }
}

impl crate::output::TableFormat for Permission {
    fn headers() -> Vec<&'static str> {
        vec!["PERMISSION", "STATUS", "TYPE", "DESCRIPTION"]
    }
    
    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            if self.granted { "Granted" } else { "Denied" }.to_string(),
            self.flags.join(", "),
            self.description.clone(),
        ]
    }
}

impl crate::output::PlainFormat for Permission {
    fn plain(&self) -> String {
        format!("{}: {}", self.name, if self.granted { "granted" } else { "denied" })
    }
}