use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::{AimError, Result};
use crate::library::adb::{push, pull, ProgressDisplay};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct CopyCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct CopyArgs {
    /// Source paths (can include device_id:path format)
    #[clap(required = true)]
    pub src: Vec<String>,
    
    /// Destination path (can include device_id:path format)
    pub dst: String,
}

impl CopyCommand {
    pub fn new() -> Self {
        Self
    }
    
    /// Parse a path that might have device_id:path format
    fn parse_device_path(path: &str) -> (Option<String>, String) {
        if let Some(colon_pos) = path.find(':') {
            let device_part = &path[..colon_pos];
            let path_part = &path[colon_pos + 1..];
            
            // Check if this looks like a device ID (not a Windows drive letter)
            if device_part.len() > 1 && !path.starts_with("C:") && !path.starts_with("D:") {
                return (Some(device_part.to_string()), path_part.to_string());
            }
        }
        (None, path.to_string())
    }
}

#[async_trait]
impl SubCommand for CopyCommand {
    type Args = CopyArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Parse destination
        let (dst_device_id, dst_path) = Self::parse_device_path(&args.dst);
        
        // Handle multiple source files
        for src in &args.src {
            let (src_device_id, src_path) = Self::parse_device_path(src);
            
            match (&src_device_id, &dst_device_id) {
                (Some(_), Some(_)) => {
                    return Err(AimError::InvalidArgument(
                        "Cannot copy between two devices".to_string()
                    ));
                }
                (None, None) => {
                    return Err(AimError::InvalidArgument(
                        "At least one path must specify a device".to_string()
                    ));
                }
                (Some(device_id), None) => {
                    // Pull from device
                    let device = if let Some(dev) = &ctx.device {
                        dev
                    } else {
                        return Err(AimError::DeviceIdRequired);
                    };
                    
                    // Verify device ID matches if specified
                    if !device.id.to_string().contains(device_id) {
                        return Err(AimError::InvalidArgument(
                            format!("Device '{}' not found", device_id)
                        ));
                    }
                    
                    self.pull_file(host, port, &device.id, &src_path, Path::new(&dst_path)).await?;
                }
                (None, Some(device_id)) => {
                    // Push to device
                    let device = if let Some(dev) = &ctx.device {
                        dev
                    } else {
                        return Err(AimError::DeviceIdRequired);
                    };
                    
                    // Verify device ID matches if specified
                    if !device.id.to_string().contains(device_id) {
                        return Err(AimError::InvalidArgument(
                            format!("Device '{}' not found", device_id)
                        ));
                    }
                    
                    self.push_file(host, port, &device.id, Path::new(&src_path), &dst_path).await?;
                }
            }
        }
        
        Ok(())
    }
}

impl CopyCommand {
    async fn pull_file(
        &self,
        host: &str,
        port: u16,
        device_id: &crate::core::types::DeviceId,
        remote_path: &str,
        local_path: &Path,
    ) -> Result<()> {
        let device_id_str = device_id.to_string();
        let port_str = port.to_string();
        
        println!("Pulling {} to {}", remote_path, local_path.display());
        
        pull(
            host,
            &port_str,
            Some(&device_id_str),
            &PathBuf::from(remote_path),
            &local_path.to_path_buf(),
            ProgressDisplay::Show,
        ).await?;
        
        Ok(())
    }
    
    async fn push_file(
        &self,
        host: &str,
        port: u16,
        device_id: &crate::core::types::DeviceId,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<()> {
        let device_id_str = device_id.to_string();
        let port_str = port.to_string();
        
        println!("Pushing {} to {}", local_path.display(), remote_path);
        
        push(
            host,
            &port_str,
            Some(&device_id_str),
            &local_path.to_path_buf(),
            &PathBuf::from(remote_path),
            false,  // has_multiple_sources
            ProgressDisplay::Show,
        ).await?;
        
        Ok(())
    }
}