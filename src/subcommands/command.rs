use crate::library::adb;
use crate::types::DeviceDetails;
use crossterm::{
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use indicatif::ProgressBar;
use std::error::Error;
use std::io::stdout;
use std::time::Duration;
use tokio::time::sleep;

pub struct CommandArgs<'a> {
    pub command: &'a str,
    pub device: Option<&'a DeviceDetails>,
    pub filters: Option<&'a [String]>,
    pub watch: bool,
    pub watch_time: Option<u32>,
}

pub async fn run(
    host: &str,
    port: &str,
    command: &str,
    device: Option<&DeviceDetails>,
    filters: Option<&[String]>,
    watch: bool,
    watch_time: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    if !watch {
        // Regular single execution
        return execute_command(host, port, command, device, filters).await;
    }

    // Watch mode
    let pb = watch_time.map(|t| {
        let pb = ProgressBar::new(t as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}s",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    });

    let mut iteration = 0;
    loop {
        // Clear screen and move cursor to top
        stdout().execute(Clear(ClearType::All))?;
        println!("Watch iteration: {}", iteration);
        println!("Press Ctrl+C to stop\n");

        // Execute command
        execute_command(host, port, command, device, filters).await?;

        // Check if we should continue
        if let Some(time) = watch_time {
            if iteration >= time {
                break;
            }
            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        iteration += 1;
        sleep(Duration::from_secs(1)).await;
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Watch completed");
    }

    Ok(())
}

async fn execute_command(
    host: &str,
    port: &str,
    command: &str,
    device: Option<&DeviceDetails>,
    filters: Option<&[String]>,
) -> Result<(), Box<dyn Error>> {
    let adb_id = device.map(|d| d.adb_id.as_str());
    let response = adb::run_shell_command_async(host, port, command, adb_id).await?;
    println!("{}", response);
    Ok(())
}
