use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::core::types::OutputFormat;
use crate::error::{AimError, Result};
use crate::output::OutputFormatter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct InfoCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct InfoArgs {
    /// Package name (supports partial matching)
    pub package: String,
    
    /// Output format
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "plain")]
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAppInfo {
    pub package: String,
    pub label: String,
    pub version_name: String,
    pub version_code: String,
    pub target_sdk: String,
    pub min_sdk: String,
    pub data_dir: String,
    pub apk_path: String,
    pub install_time: String,
    pub update_time: String,
    pub installer: String,
    pub size: AppSize,
    pub permissions: Vec<String>,
    pub activities: Vec<String>,
    pub services: Vec<String>,
    pub is_system: bool,
    pub is_enabled: bool,
    pub is_debuggable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSize {
    pub code_size: String,
    pub data_size: String,
    pub cache_size: String,
    pub total_size: String,
}

impl InfoCommand {
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
    
    async fn get_app_info(&self, ctx: &CommandContext, package: &str) -> Result<DetailedAppInfo> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Get comprehensive package info via dumpsys
        let cmd = format!("dumpsys package {}", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        
        // Parse the dumpsys output
        let mut info = DetailedAppInfo {
            package: package.to_string(),
            label: package.to_string(),
            version_name: "Unknown".to_string(),
            version_code: "Unknown".to_string(),
            target_sdk: "Unknown".to_string(),
            min_sdk: "Unknown".to_string(),
            data_dir: "Unknown".to_string(),
            apk_path: "Unknown".to_string(),
            install_time: "Unknown".to_string(),
            update_time: "Unknown".to_string(),
            installer: "Unknown".to_string(),
            size: AppSize {
                code_size: "0".to_string(),
                data_size: "0".to_string(),
                cache_size: "0".to_string(),
                total_size: "0".to_string(),
            }.format_sizes(),
            permissions: Vec::new(),
            activities: Vec::new(),
            services: Vec::new(),
            is_system: false,
            is_enabled: true,
            is_debuggable: false,
        };
        
        let mut in_permissions = false;
        let mut in_activities = false;
        let mut in_services = false;
        
        for line in output.stdout.lines() {
            let trimmed = line.trim();
            
            // Version info
            if trimmed.starts_with("versionCode=") {
                if let Some(code) = trimmed.split_whitespace().next() {
                    info.version_code = code.strip_prefix("versionCode=").unwrap_or("Unknown").to_string();
                }
            } else if trimmed.starts_with("versionName=") {
                info.version_name = trimmed.strip_prefix("versionName=").unwrap_or("Unknown").to_string();
            }
            
            // SDK versions
            else if trimmed.contains("targetSdk=") {
                if let Some(start) = trimmed.find("targetSdk=") {
                    let sdk_str = &trimmed[start + 10..];
                    if let Some(end) = sdk_str.find(|c: char| !c.is_numeric()) {
                        info.target_sdk = sdk_str[..end].to_string();
                    } else {
                        info.target_sdk = sdk_str.to_string();
                    }
                }
            } else if trimmed.contains("minSdk=") {
                if let Some(start) = trimmed.find("minSdk=") {
                    let sdk_str = &trimmed[start + 7..];
                    if let Some(end) = sdk_str.find(|c: char| !c.is_numeric()) {
                        info.min_sdk = sdk_str[..end].to_string();
                    } else {
                        info.min_sdk = sdk_str.to_string();
                    }
                }
            }
            
            // Paths
            else if trimmed.starts_with("dataDir=") {
                info.data_dir = trimmed.strip_prefix("dataDir=").unwrap_or("Unknown").to_string();
            } else if trimmed.starts_with("codePath=") {
                info.apk_path = trimmed.strip_prefix("codePath=").unwrap_or("Unknown").to_string();
                info.is_system = info.apk_path.contains("/system/") || info.apk_path.contains("/vendor/");
            }
            
            // Timestamps
            else if trimmed.contains("firstInstallTime=") {
                if let Some(start) = trimmed.find("firstInstallTime=") {
                    let time_str = &trimmed[start + 17..];
                    if let Some(end) = time_str.find(' ') {
                        info.install_time = time_str[..end].to_string();
                    }
                }
            } else if trimmed.contains("lastUpdateTime=") {
                if let Some(start) = trimmed.find("lastUpdateTime=") {
                    let time_str = &trimmed[start + 15..];
                    if let Some(end) = time_str.find(' ') {
                        info.update_time = time_str[..end].to_string();
                    }
                }
            }
            
            // Installer
            else if trimmed.starts_with("installerPackageName=") {
                info.installer = trimmed.strip_prefix("installerPackageName=")
                    .unwrap_or("Unknown")
                    .to_string();
            }
            
            // Flags
            else if trimmed.contains("DEBUGGABLE") {
                info.is_debuggable = true;
            } else if trimmed.contains("enabled=false") {
                info.is_enabled = false;
            }
            
            // Permissions section
            else if trimmed == "grantedPermissions:" || trimmed == "requested permissions:" {
                in_permissions = true;
                in_activities = false;
                in_services = false;
            }
            
            // Activities section
            else if trimmed == "Activities:" {
                in_activities = true;
                in_permissions = false;
                in_services = false;
            }
            
            // Services section
            else if trimmed == "Services:" {
                in_services = true;
                in_permissions = false;
                in_activities = false;
            }
            
            // End of sections
            else if trimmed.starts_with("mKeySetMapping:") || 
                    trimmed.starts_with("Packages:") ||
                    trimmed.starts_with("Hidden system packages:") {
                in_permissions = false;
                in_activities = false;
                in_services = false;
            }
            
            // Collect items
            else if in_permissions && trimmed.starts_with("android.permission.") {
                info.permissions.push(trimmed.to_string());
            } else if in_activities && trimmed.contains(package) && trimmed.contains('/') {
                // Extract activity name
                if let Some(activity_start) = trimmed.find(package) {
                    let activity = &trimmed[activity_start..];
                    if let Some(space_pos) = activity.find(' ') {
                        info.activities.push(activity[..space_pos].to_string());
                    } else {
                        info.activities.push(activity.to_string());
                    }
                }
            } else if in_services && trimmed.contains(package) && trimmed.contains('/') {
                // Extract service name
                if let Some(service_start) = trimmed.find(package) {
                    let service = &trimmed[service_start..];
                    if let Some(space_pos) = service.find(' ') {
                        info.services.push(service[..space_pos].to_string());
                    } else {
                        info.services.push(service.to_string());
                    }
                }
            }
        }
        
        // Get app size
        let size_cmd = format!("du -sh {}", info.data_dir);
        let shell_cmd = crate::adb::shell::ShellCommand::new(size_cmd)
            .with_device(device.id.clone());
        
        if let Ok(size_output) = shell_cmd.execute(host, port).await {
            if let Some(size_line) = size_output.stdout.lines().next() {
                if let Some(size) = size_line.split_whitespace().next() {
                    info.size.data_size = size.to_string();
                }
            }
        }
        
        Ok(info)
    }
}

#[async_trait]
impl SubCommand for InfoCommand {
    type Args = InfoArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Find the full package name
        let package = self.find_package(ctx, &args.package).await?;
        
        // Get detailed info
        let info = self.get_app_info(ctx, &package).await?;
        
        // Get output format
        let output_format = OutputFormat::from_str(&args.output)
            .ok_or_else(|| AimError::InvalidArgument(format!("Invalid output format: {}", args.output)))?;
        
        let formatter = OutputFormatter::new();
        
        match output_format {
            OutputFormat::Plain => {
                // Formatted plain text output
                println!("Package: {}", info.package);
                println!("Label: {}", info.label);
                println!("Version: {} ({})", info.version_name, info.version_code);
                println!("SDK: {} (min: {})", info.target_sdk, info.min_sdk);
                println!("Type: {}", if info.is_system { "System" } else { "User" });
                println!("Enabled: {}", if info.is_enabled { "Yes" } else { "No" });
                println!("Debuggable: {}", if info.is_debuggable { "Yes" } else { "No" });
                println!();
                println!("Paths:");
                println!("  APK: {}", info.apk_path);
                println!("  Data: {}", info.data_dir);
                println!();
                println!("Size:");
                println!("  Code: {}", info.size.code_size);
                println!("  Data: {}", info.size.data_size);
                println!("  Cache: {}", info.size.cache_size);
                println!("  Total: {}", info.size.total_size);
                println!();
                println!("Installed: {}", info.install_time);
                println!("Updated: {}", info.update_time);
                println!("Installer: {}", info.installer);
                
                if !info.activities.is_empty() {
                    println!();
                    println!("Activities ({}):", info.activities.len());
                    for activity in &info.activities {
                        println!("  {}", activity);
                    }
                }
                
                if !info.services.is_empty() {
                    println!();
                    println!("Services ({}):", info.services.len());
                    for service in &info.services {
                        println!("  {}", service);
                    }
                }
                
                if !info.permissions.is_empty() {
                    println!();
                    println!("Permissions ({}):", info.permissions.len());
                    for perm in &info.permissions {
                        println!("  {}", perm);
                    }
                }
            }
            OutputFormat::Json => {
                formatter.json(&info)?;
            }
            OutputFormat::Table => {
                // Create key-value pairs for table
                let table_data = vec![
                    ("Package", info.package.clone()),
                    ("Label", info.label.clone()),
                    ("Version", format!("{} ({})", info.version_name, info.version_code)),
                    ("SDK", format!("{} (min: {})", info.target_sdk, info.min_sdk)),
                    ("Type", if info.is_system { "System" } else { "User" }.to_string()),
                    ("Enabled", if info.is_enabled { "Yes" } else { "No" }.to_string()),
                    ("Debuggable", if info.is_debuggable { "Yes" } else { "No" }.to_string()),
                    ("APK Path", info.apk_path.clone()),
                    ("Data Dir", info.data_dir.clone()),
                    ("Data Size", info.size.data_size.clone()),
                    ("Installed", info.install_time.clone()),
                    ("Updated", info.update_time.clone()),
                    ("Installer", info.installer.clone()),
                    ("Activities", info.activities.len().to_string()),
                    ("Services", info.services.len().to_string()),
                    ("Permissions", info.permissions.len().to_string()),
                ];
                
                // Format as property table
                let properties: Vec<crate::output::property::Property> = table_data
                    .into_iter()
                    .map(|(k, v)| crate::output::property::Property::new(k, v))
                    .collect();
                    
                formatter.table(&properties)?;
            }
        }
        
        Ok(())
    }
}

impl AppSize {
    fn format_sizes(self) -> Self {
        // In a real implementation, we'd format byte sizes to human-readable
        // For now, just return as-is
        self
    }
}