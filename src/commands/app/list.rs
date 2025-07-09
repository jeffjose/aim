use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::core::types::OutputFormat;
use crate::error::{AimError, Result};
use crate::output::OutputFormatter;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct ListCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct ListArgs {
    /// Filter packages (supports partial matching)
    #[clap(short, long)]
    pub filter: Option<String>,
    
    /// Show only user-installed apps
    #[clap(short, long)]
    pub user: bool,
    
    /// Show only system apps
    #[clap(short, long)]
    pub system: bool,
    
    /// Show only enabled apps
    #[clap(short, long)]
    pub enabled: bool,
    
    /// Show only disabled apps
    #[clap(short, long)]
    pub disabled: bool,
    
    /// Show detailed information (slower)
    #[clap(long)]
    pub details: bool,
    
    /// Output format
    #[clap(short, long, value_parser = ["table", "json", "plain"], default_value = "plain")]
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub package: String,
    pub name: String,
    pub version: String,
    pub installed_at: String,
    pub size: String,
    pub is_system: bool,
    pub is_enabled: bool,
}

impl ListCommand {
    pub fn new() -> Self {
        Self
    }
    
    async fn get_packages(&self, ctx: &CommandContext, args: &ListArgs) -> Result<Vec<String>> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Build pm list command
        let mut cmd = "pm list packages -f".to_string();
        
        if args.user {
            cmd.push_str(" -3"); // Third party apps only
        } else if args.system {
            cmd.push_str(" -s"); // System apps only
        }
        
        if args.enabled {
            cmd.push_str(" -e"); // Enabled apps only
        } else if args.disabled {
            cmd.push_str(" -d"); // Disabled apps only
        }
        
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        
        // Parse package list
        let packages: Vec<String> = output.stdout
            .lines()
            .filter_map(|line| {
                // Format: package:/path/to/apk=com.example.app
                if let Some(pkg) = line.strip_prefix("package:") {
                    if let Some(eq_pos) = pkg.rfind('=') {
                        let package_name = &pkg[eq_pos + 1..];
                        
                        // Apply filter if provided
                        if let Some(filter) = &args.filter {
                            if package_name.contains(filter) {
                                return Some(package_name.to_string());
                            }
                        } else {
                            return Some(package_name.to_string());
                        }
                    }
                }
                None
            })
            .collect();
            
        Ok(packages)
    }
    
    async fn get_app_details(&self, ctx: &CommandContext, packages: Vec<String>) -> Result<Vec<AppInfo>> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        let mut apps = Vec::new();
        
        for package in packages {
            // Get app info via dumpsys
            let cmd = format!("dumpsys package {} | grep -E 'versionName=|firstInstallTime=|lastUpdateTime=|codePath='", package);
            let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
                .with_device(device.id.clone());
            
            let output = shell_cmd.execute(host, port).await?;
            
            // Parse the output
            let mut version = "Unknown".to_string();
            let mut installed_at = "Unknown".to_string();
            let mut is_system = false;
            
            for line in output.stdout.lines() {
                if let Some(v) = line.strip_prefix("    versionName=") {
                    version = v.to_string();
                } else if line.contains("firstInstallTime=") {
                    // Extract timestamp
                    if let Some(start) = line.find("firstInstallTime=") {
                        let timestamp_str = &line[start + 17..];
                        if let Some(end) = timestamp_str.find(' ') {
                            installed_at = timestamp_str[..end].to_string();
                        }
                    }
                } else if line.contains("codePath=") {
                    is_system = line.contains("/system/") || line.contains("/vendor/");
                }
            }
            
            // Get app label (user-friendly name)
            let label_cmd = format!("cmd package resolve-activity --brief {} | tail -n 1", package);
            let shell_cmd = crate::adb::shell::ShellCommand::new(label_cmd)
                .with_device(device.id.clone());
            
            let label_output = shell_cmd.execute(host, port).await?;
            let name = if label_output.stdout.trim().is_empty() {
                package.clone()
            } else {
                // Try to extract app name from label
                let lines: Vec<&str> = label_output.stdout.lines().collect();
                if let Some(last_line) = lines.last() {
                    if last_line.contains('/') {
                        package.clone()
                    } else {
                        last_line.trim().to_string()
                    }
                } else {
                    package.clone()
                }
            };
            
            apps.push(AppInfo {
                package: package.clone(),
                name,
                version,
                installed_at,
                size: "N/A".to_string(), // Size calculation would be expensive
                is_system,
                is_enabled: true, // Would need additional check
            });
        }
        
        // Sort by package name
        apps.sort_by(|a, b| a.package.cmp(&b.package));
        
        Ok(apps)
    }
}

#[async_trait]
impl SubCommand for ListCommand {
    type Args = ListArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Get list of packages
        let packages = self.get_packages(ctx, &args).await?;
        
        if packages.is_empty() {
            let msg = if args.filter.is_some() {
                "No packages found matching the filter"
            } else {
                "No packages found"
            };
            return Err(AimError::CommandExecution(msg.to_string()));
        }
        
        // Get output format
        let output_format = OutputFormat::from_str(&args.output)
            .ok_or_else(|| AimError::InvalidArgument(format!("Invalid output format: {}", args.output)))?;
        
        // Never print to stdout when outputting JSON (except the JSON itself)
        let is_json = matches!(output_format, OutputFormat::Json);
        
        let formatter = OutputFormatter::new();
        
        // If details flag is not set, just show package names
        if !args.details {
            match output_format {
                OutputFormat::Plain => {
                    for package in packages {
                        println!("{}", package);
                    }
                }
                OutputFormat::Table => {
                    // For table without details, show a simple single-column table
                    let mut table = comfy_table::Table::new();
                    table.set_header(vec!["PACKAGE"]);
                    for package in packages {
                        table.add_row(vec![package]);
                    }
                    println!("{}", table);
                }
                OutputFormat::Json => {
                    // For JSON, return array of package names
                    formatter.json(&packages)?;
                }
            }
        } else {
            // With details flag, fetch and show full information
            if !is_json && !ctx.quiet {
                println!("Fetching app details...");
            }
            
            let apps = self.get_app_details(ctx, packages).await?;
            
            match output_format {
                OutputFormat::Table => {
                    formatter.table(&apps)?;
                }
                OutputFormat::Json => {
                    formatter.json(&apps)?;
                }
                OutputFormat::Plain => {
                    // For plain output with details, show key info
                    for app in apps {
                        println!("{} - {} ({})", 
                            app.package, 
                            app.name,
                            if app.is_system { "system" } else { "user" }
                        );
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl crate::output::TableFormat for AppInfo {
    fn headers() -> Vec<&'static str> {
        vec!["PACKAGE", "NAME", "VERSION", "TYPE", "INSTALLED"]
    }
    
    fn row(&self) -> Vec<String> {
        vec![
            self.package.clone(),
            self.name.clone(),
            self.version.clone(),
            if self.is_system { "System" } else { "User" }.to_string(),
            self.installed_at.clone(),
        ]
    }
}

impl crate::output::PlainFormat for AppInfo {
    fn plain(&self) -> String {
        self.package.clone()
    }
}