use crate::{config::Config, library::adb, types::DeviceDetails};
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use log::debug;
use rand::Rng;
use std::{error::Error, path::PathBuf};
use tokio::time::sleep;
use std::time::Duration;

pub struct ScreenrecordArgs {
    pub device_id: Option<String>,
    pub output: Option<PathBuf>,
    pub args: Vec<String>,
}

pub async fn run(
    args: ScreenrecordArgs,
    device: &DeviceDetails,
    host: &str,
    port: &str,
) -> Result<(), Box<dyn Error>> {
    let adb_id = Some(&device.adb_id);

    // Generate random suffix for temp file
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let temp_file = format!("/sdcard/screenrecord_{}.mp4", random_suffix);

    // Get output directory
    let base_dir = if let Some(path) = args.output {
        path
    } else {
        let config = Config::load();
        config
            .screenrecord
            .and_then(|s| s.get_output_path())
            .unwrap_or_else(|| PathBuf::from("/tmp"))
    };

    // Generate output filename with timestamp
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let output_path = base_dir.join(format!(
        "aim-screenrecord-{}-{}.mp4",
        device.device_id_short, timestamp
    ));

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
    debug!("Starting screen recording");
    adb::run_shell_command_async(
        host,
        port,
        &screenrecord_cmd,
        adb_id.map(|x| x.as_str()),
    )
    .await?;

    // Wait for 'q' key
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Stop recording
    debug!("Stopping screen recording");
    adb::run_shell_command_async(
        host,
        port,
        "killall screenrecord",
        adb_id.map(|x| x.as_str()),
    )
    .await?;

    // Give it a moment to finish writing
    sleep(Duration::from_secs(1)).await;

    // Copy recording to host
    debug!("Copying recording to host");
    adb::pull(
        host,
        port,
        adb_id.map(|x| x.as_str()),
        &PathBuf::from(&temp_file),
        &output_path,
        adb::ProgressDisplay::Show,
    )
    .await?;

    // Clean up temp file
    debug!("Cleaning up temp file on device");
    adb::run_shell_command_async(
        host,
        port,
        &format!("rm -f {}", &temp_file),
        adb_id.map(|x| x.as_str()),
    )
    .await?;

    disable_raw_mode()?;
    println!("\nRecording saved to: {}", output_path.display());
    Ok(())
} 
