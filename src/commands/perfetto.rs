use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::{push, pull, run_shell_command_async, ProgressDisplay};
use async_trait::async_trait;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use indicatif::ProgressBar;
use rand::{distr::Alphanumeric, Rng};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

pub struct PerfettoCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct PerfettoArgs {
    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
    
    /// Path to perfetto config file
    #[clap(short = 'c', long = "config")]
    pub config: PathBuf,
    
    /// Duration in seconds (if not specified, press 'q' to stop)
    #[clap(short = 't', long = "time")]
    pub time: Option<u32>,
    
    /// Output file path
    #[clap(short = 'o', long = "output", default_value = "trace.perfetto-trace")]
    pub output: PathBuf,
}

impl PerfettoCommand {
    pub fn new() -> Self {
        Self
    }
    
    async fn wait_for_keypress() -> Result<()> {
        enable_raw_mode()?;
        println!("\nPress 'q' to stop trace collection...");
        
        loop {
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
        
        disable_raw_mode()?;
        Ok(())
    }
}

#[async_trait]
impl SubCommand for PerfettoCommand {
    type Args = PerfettoArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Generate random temp file name
        let temp_file = format!(
            "/data/local/tmp/perfetto_config_{}.txt",
            rand::rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect::<String>()
        );
        
        // Step 1: Copy config file to device
        println!("Copying config file to device...");
        let device_id_str = device.id.to_string();
        let port_str = port.to_string();
        
        push(
            host,
            &port_str,
            Some(&device_id_str),
            &args.config,
            &PathBuf::from(&temp_file),
            false,  // has_multiple_sources
            ProgressDisplay::Show,
        ).await?;
        
        // Step 2: Start perfetto trace
        println!("Starting perfetto trace...");
        let perfetto_cmd = format!(
            "cat {} | perfetto --txt -c - -o /data/misc/perfetto-traces/trace",
            temp_file
        );
        
        // Run perfetto in background
        let bg_cmd = format!("{} > /dev/null 2>&1 &", perfetto_cmd);
        run_shell_command_async(host, &port_str, &bg_cmd, Some(&device_id_str)).await?;
        
        // Step 3: Wait for specified duration or user input
        match args.time {
            Some(duration) => {
                // Use progress bar for fixed duration
                let pb = ProgressBar::new(duration as u64);
                pb.set_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}s")
                        .unwrap()
                        .progress_chars("#>-"),
                );
                
                for _ in 0..duration {
                    sleep(Duration::from_secs(1)).await;
                    pb.inc(1);
                }
                pb.finish_with_message("Trace collection complete");
            }
            None => {
                // Wait for user input
                Self::wait_for_keypress().await?;
                println!("\nStopping trace collection...");
            }
        }
        
        // Step 4: Kill perfetto
        run_shell_command_async(host, &port_str, "killall perfetto", Some(&device_id_str)).await?;
        
        // Give perfetto a moment to finish writing
        sleep(Duration::from_secs(1)).await;
        
        // Step 5: Copy trace file back to host
        println!("Copying trace file back to host...");
        pull(
            host,
            &port_str,
            Some(&device_id_str),
            &PathBuf::from("/data/misc/perfetto-traces/trace"),
            &args.output,
            ProgressDisplay::Show,
        ).await?;
        
        // Clean up temp config file
        let rm_cmd = format!("rm -f {}", temp_file);
        run_shell_command_async(host, &port_str, &rm_cmd, Some(&device_id_str)).await?;
        
        println!("\nTrace file saved to: {}", args.output.display());
        Ok(())
    }
}