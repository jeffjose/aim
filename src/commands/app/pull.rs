use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use crate::progress::{ProgressFactory, ProgressReporter};
use async_trait::async_trait;
use colored::*;
use std::path::{Path, PathBuf};

pub struct PullCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct PullArgs {
    /// Package name (supports partial matching)
    pub package: String,
    
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
    
    /// Output directory (default: current directory)
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    
    /// Include split APKs (for app bundles)
    #[clap(short, long)]
    pub splits: bool,
}

impl PullCommand {
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
    
    async fn get_apk_paths(&self, ctx: &CommandContext, package: &str) -> Result<Vec<String>> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Get APK paths
        let cmd = format!("pm path {}", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let output = shell_cmd.execute(host, port).await?;
        
        let paths: Vec<String> = output.stdout
            .lines()
            .filter_map(|line| {
                line.strip_prefix("package:").map(|p| p.to_string())
            })
            .collect();
            
        if paths.is_empty() {
            return Err(AimError::CommandExecution(format!("No APK paths found for package '{}'", package)));
        }
        
        Ok(paths)
    }
    
    async fn get_app_info(&self, ctx: &CommandContext, package: &str) -> Result<(String, String)> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Get version info
        let cmd = format!("dumpsys package {} | grep versionName", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(cmd)
            .with_device(device.id.clone());
        
        let mut version = "Unknown".to_string();
        if let Ok(output) = shell_cmd.execute(host, port).await {
            if let Some(line) = output.stdout.lines().next() {
                if let Some(v) = line.trim().strip_prefix("versionName=") {
                    version = v.to_string();
                }
            }
        }
        
        // Try to get app name
        let label_cmd = format!("cmd package resolve-activity --brief {} | tail -n 1", package);
        let shell_cmd = crate::adb::shell::ShellCommand::new(label_cmd)
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
        
        Ok((app_name, version))
    }
    
    async fn pull_file(&self, ctx: &CommandContext, remote_path: &str, local_path: &Path, progress: Box<dyn ProgressReporter>) -> Result<()> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();

        // Use the new FileTransfer API
        let mut file_transfer = crate::adb::file_transfer::FileTransfer::new(host, port, Some(&device.id)).await?;

        // Hook up progress reporter
        file_transfer.set_progress_reporter(progress);

        file_transfer.pull(remote_path, local_path).await
    }
}

#[async_trait]
impl SubCommand for PullCommand {
    type Args = PullArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        // Find the full package name
        let package = self.find_package(ctx, &args.package).await?;
        
        println!("Finding APK for package: {}", package.bright_cyan());
        
        // Get APK paths
        let apk_paths = self.get_apk_paths(ctx, &package).await?;
        
        // Get app info for naming
        let (app_name, version) = self.get_app_info(ctx, &package).await?;
        
        println!("Found {} APK{}", apk_paths.len(), if apk_paths.len() > 1 { "s" } else { "" });
        println!("App: {}", app_name.bright_cyan());
        println!("Version: {}", version.bright_cyan());
        println!();
        
        // Determine output directory
        let output_dir = args.output.unwrap_or_else(|| PathBuf::from("."));
        
        // Create output directory if needed
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }
        
        // Pull each APK
        let progress_factory = ProgressFactory::new(true);
        
        for (idx, apk_path) in apk_paths.iter().enumerate() {
            let filename = if apk_paths.len() == 1 {
                // Single APK - use clean name
                format!("{}_v{}.apk", package, version.replace(' ', "_"))
            } else if apk_path.contains("split_") {
                // Split APK - extract split name
                if let Some(split_start) = apk_path.rfind("split_") {
                    let split_part = &apk_path[split_start..];
                    if let Some(dot_pos) = split_part.find('.') {
                        format!("{}_{}_v{}.apk", package, &split_part[6..dot_pos], version.replace(' ', "_"))
                    } else {
                        format!("{}_split_{}_v{}.apk", package, idx, version.replace(' ', "_"))
                    }
                } else {
                    format!("{}_split_{}_v{}.apk", package, idx, version.replace(' ', "_"))
                }
            } else {
                // Base APK or unknown
                if idx == 0 {
                    format!("{}_v{}.apk", package, version.replace(' ', "_"))
                } else {
                    format!("{}_part{}_v{}.apk", package, idx, version.replace(' ', "_"))
                }
            };
            
            let local_path = output_dir.join(&filename);
            
            println!("Pulling: {}", apk_path.bright_yellow());
            println!("To: {}", local_path.display());
            
            // Create progress bar for this file
            let progress = progress_factory.file_transfer(&filename, 0);
            progress.start(0);

            // Pull the file
            self.pull_file(ctx, apk_path, &local_path, progress).await?;
            
            // Get file size
            if let Ok(metadata) = std::fs::metadata(&local_path) {
                let size = metadata.len();
                let size_mb = size as f64 / 1_048_576.0;
                println!("{} Pulled {} ({:.1} MB)", "✓".green(), filename, size_mb);
            } else {
                println!("{} Pulled {}", "✓".green(), filename);
            }
            println!();
        }
        
        if !args.splits && apk_paths.len() > 1 {
            println!("{}", "Note: This app uses split APKs (App Bundle).".yellow());
            println!("{}", "Use --splits flag to pull all split APKs.".yellow());
        }
        
        println!("{} APK extraction complete!", "✓".green().bold());
        println!("Location: {}", output_dir.display());
        
        Ok(())
    }
}