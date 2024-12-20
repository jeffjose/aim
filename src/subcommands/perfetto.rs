use crate::library::adb;
use crate::types::DeviceDetails;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use indicatif::ProgressBar;
use log::debug;
use rand::Rng;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

pub struct PerfettoArgs {
    pub config: PathBuf,
    pub time: Option<u32>,
    pub output: PathBuf,
    pub device_id: Option<String>,
}

async fn wait_for_keypress() -> Result<(), Box<dyn Error>> {
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

pub async fn run(
    args: PerfettoArgs,
    device: &DeviceDetails,
    host: &str,
    port: &str,
) -> Result<(), Box<dyn Error>> {
    debug!("Starting perfetto trace collection");
    let adb_id = Some(&device.adb_id);

    // Generate random temp file name
    let temp_file = format!(
        "/tmp/perfetto_config_{}.txt",
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>()
    );
    debug!("Using temp config file: {}", temp_file);

    // Step 1: Copy config file to device
    debug!("Copying config file to device");
    adb::push(
        host,
        port,
        adb_id.map(|x| x.as_str()),
        &args.config,
        &PathBuf::from(&temp_file),
        false,
        adb::ProgressDisplay::Hide,
    )
    .await?;

    // Step 2: Start perfetto trace
    debug!("Starting perfetto trace");
    let perfetto_cmd = format!(
        "cat {} | perfetto --txt -c - -o /data/misc/perfetto-traces/trace",
        temp_file
    );

    // Run perfetto in background
    adb::run_shell_command_async(
        host,
        port,
        &format!("{} > /dev/null 2>&1 &", perfetto_cmd),
        adb_id.map(|x| x.as_str()),
    )
    .await?;

    // Step 3: Wait for specified duration or user input
    match args.time {
        Some(duration) => {
            // Use progress bar for fixed duration
            debug!("Waiting for {} seconds", duration);
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
            debug!("Waiting for user input to stop trace collection");
            wait_for_keypress().await?;
            println!("\nStopping trace collection...");
        }
    }

    // Step 4: Kill perfetto
    debug!("Stopping perfetto");
    adb::run_shell_command_async(host, port, "killall perfetto", adb_id.map(|x| x.as_str()))
        .await?;

    // Give perfetto a moment to finish writing
    sleep(Duration::from_secs(1)).await;

    // Step 5: Copy trace file back to host
    debug!("Copying trace file back to host");
    adb::pull(
        host,
        port,
        adb_id.map(|x| x.as_str()),
        &PathBuf::from("/data/misc/perfetto-traces/trace"),
        &args.output,
        adb::ProgressDisplay::Hide,
    )
    .await?;

    // Clean up temp config file
    debug!("Cleaning up temp config file");
    adb::run_shell_command_async(
        host,
        port,
        &format!("rm {}", temp_file),
        adb_id.map(|x| x.as_str()),
    )
    .await?;

    debug!("Perfetto trace collection completed");
    Ok(())
}
