use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::{run_shell_command_async, pull, ProgressDisplay};
use crate::config::Config;
use async_trait::async_trait;
use chrono::Local;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rand::{distr::Alphanumeric, Rng};
use std::path::PathBuf;
use std::time::Duration;

pub struct ScreenshotCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct ScreenshotArgs {
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
    
    /// Output file or directory path
    #[clap(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
    
    /// Interactive mode - take screenshots with spacebar
    #[clap(short = 'i', long = "interactive")]
    pub interactive: bool,
    
    /// Additional arguments to pass to screencap
    #[clap(trailing_var_arg = true)]
    pub args: Vec<String>,
}

impl ScreenshotCommand {
    pub fn new() -> Self {
        Self
    }
    
    async fn take_screenshot(
        &self,
        _ctx: &CommandContext,
        device: &crate::core::types::Device,
        output_path: &PathBuf,
        args: &[String],
    ) -> Result<()> {
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Generate temp file on device
        let random_suffix: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let temp_file = format!("/sdcard/screenshot_{}.png", random_suffix);
        
        // Take screenshot
        let screencap_cmd = if args.is_empty() {
            format!("screencap {}", temp_file)
        } else {
            format!("screencap {} {}", args.join(" "), temp_file)
        };
        
        let device_id = device.id.to_string();
        let port_str = port.to_string();
        
        // Take screenshot
        run_shell_command_async(host, &port_str, &screencap_cmd, Some(&device_id)).await?;
        
        // Pull file
        pull(
            host,
            &port_str,
            Some(&device_id),
            &PathBuf::from(&temp_file),
            &output_path,
            ProgressDisplay::Show,
        ).await?;
        
        // Clean up
        let rm_cmd = format!("rm -f {}", temp_file);
        run_shell_command_async(host, &port_str, &rm_cmd, Some(&device_id)).await?;
        
        println!("Screenshot saved to: {}", output_path.display());
        Ok(())
    }
}

#[async_trait]
impl SubCommand for ScreenshotCommand {
    type Args = ScreenshotArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        
        if args.interactive {
            // Interactive mode
            enable_raw_mode()?;
            println!("Interactive screenshot mode");
            println!("Press SPACE to take a screenshot, 'q' to quit");
            
            let mut counter = 1;
            loop {
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char(' ') => {
                                // Generate filename
                                let timestamp = Local::now().format("%Y%m%d-%H%M%S");
                                let filename = format!(
                                    "aim-screenshot-{}-{}-{:03}.png",
                                    device.id.short_id(), timestamp, counter
                                );
                                
                                let output_path = if let Some(ref dir) = args.output {
                                    dir.join(filename)
                                } else {
                                    PathBuf::from(filename)
                                };
                                
                                println!("\nTaking screenshot...");
                                self.take_screenshot(ctx, device, &output_path, &args.args).await?;
                                counter += 1;
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                }
            }
            
            disable_raw_mode()?;
            println!("\nExiting interactive mode");
        } else {
            // Single screenshot
            let output_path = if let Some(path) = args.output {
                if path.is_dir() || path.as_os_str().to_string_lossy().ends_with('/') {
                    // Generate filename with timestamp
                    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
                    path.join(format!(
                        "aim-screenshot-{}-{}.png",
                        device.id.short_id(), timestamp
                    ))
                } else {
                    path
                }
            } else {
                // Use config or default
                let config = Config::load();
                let base_dir = config
                    .screenshot
                    .and_then(|s| s.get_output_path())
                    .unwrap_or_else(|| PathBuf::from("/tmp"));
                
                let timestamp = Local::now().format("%Y%m%d-%H%M%S");
                base_dir.join(format!(
                    "aim-screenshot-{}-{}.png",
                    device.id.short_id(), timestamp
                ))
            };
            
            self.take_screenshot(ctx, device, &output_path, &args.args).await?
        }
        
        Ok(())
    }
}