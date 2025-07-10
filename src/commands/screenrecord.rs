use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::{run_shell_command_async, pull, ProgressDisplay};
use crate::config::Config;
use async_trait::async_trait;
use chrono::Local;
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use rand::{distr::Alphanumeric, Rng};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct ScreenrecordCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct ScreenrecordArgs {
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
    
    /// Output file or directory path
    #[clap(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
    
    /// Additional arguments to pass to screenrecord
    #[clap(trailing_var_arg = true)]
    pub args: Vec<String>,
}

impl ScreenrecordCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for ScreenrecordCommand {
    type Args = ScreenrecordArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Generate random suffix for temp file
        let random_suffix: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let temp_file = format!("/sdcard/screenrecord_{}.mp4", random_suffix);
        
        // Get output directory or file
        let output_path = if let Some(path) = args.output {
            if path.is_dir() || path.as_os_str().to_string_lossy().ends_with('/') {
                // Generate filename with timestamp
                let timestamp = Local::now().format("%Y%m%d-%H%M%S");
                path.join(format!(
                    "aim-screenrecord-{}-{}.mp4",
                    device.id.short_id(), timestamp
                ))
            } else {
                path
            }
        } else {
            // Use config or default
            let config = Config::load();
            let base_dir = config
                .screenrecord
                .and_then(|s| s.get_output_path())
                .unwrap_or_else(|| PathBuf::from("/tmp"));
            
            let timestamp = Local::now().format("%Y%m%d-%H%M%S");
            base_dir.join(format!(
                "aim-screenrecord-{}-{}.mp4",
                device.id.short_id(), timestamp
            ))
        };
        
        // Build screenrecord command with additional args
        let screenrecord_cmd = if args.args.is_empty() {
            format!("screenrecord {} > /dev/null 2>&1 &", &temp_file)
        } else {
            format!(
                "screenrecord {} {} > /dev/null 2>&1 &",
                args.args.join(" "),
                &temp_file
            )
        };
        
        println!("Recording screen. Press 'q' to stop...");
        enable_raw_mode()?;
        
        // Start recording
        let device_id_str = device.id.to_string();
        let port_str = port.to_string();
        run_shell_command_async(host, &port_str, &screenrecord_cmd, Some(&device_id_str)).await?;
        
        let start_time = Instant::now();
        
        // Wait for 'q' key
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
            
            // Update elapsed time display
            let elapsed = start_time.elapsed();
            let hours = elapsed.as_secs() / 3600;
            let minutes = (elapsed.as_secs() % 3600) / 60;
            let seconds = elapsed.as_secs() % 60;
            
            stdout()
                .execute(Clear(ClearType::CurrentLine))?
                .execute(MoveToColumn(0))?;
            print!(
                "\rRecording... [{:02}:{:02}:{:02}]",
                hours, minutes, seconds
            );
            stdout().flush()?;
        }
        
        // Stop recording
        run_shell_command_async(host, &port_str, "killall -s 2 screenrecord", Some(&device_id_str)).await?;
        
        // Give it a moment to finish writing
        sleep(Duration::from_secs(1)).await;
        
        // Copy recording to host
        println!("\nCopying recording to host...");
        pull(
            host,
            &port_str,
            Some(&device_id_str),
            &PathBuf::from(&temp_file),
            &output_path,
            ProgressDisplay::Show,
        ).await?;
        
        // Clean up temp file
        let rm_cmd = format!("rm -f {}", &temp_file);
        run_shell_command_async(host, &port_str, &rm_cmd, Some(&device_id_str)).await?;
        
        let total_elapsed = start_time.elapsed();
        let hours = total_elapsed.as_secs() / 3600;
        let minutes = (total_elapsed.as_secs() % 3600) / 60;
        let seconds = total_elapsed.as_secs() % 60;
        
        disable_raw_mode()?;
        println!("\nRecording saved to: {}", output_path.display());
        println!(
            "Total recording time: {:02}:{:02}:{:02}",
            hours, minutes, seconds
        );
        Ok(())
    }
}