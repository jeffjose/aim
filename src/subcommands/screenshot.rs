use crate::{config::Config, library::adb, types::DeviceDetails};
use chrono::Local;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use log::debug;
use rand::Rng;
use std::error::Error;
use std::path::PathBuf;

pub struct ScreenshotArgs {
    pub device_id: Option<String>,
    pub output: Option<PathBuf>,
    pub interactive: bool,
}

async fn take_single_screenshot(
    device: &DeviceDetails,
    base_dir: PathBuf,
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
    let temp_file = format!("/tmp/screenshot_{}.png", random_suffix);

    // Generate timestamp for output file
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let output_path = base_dir.join(format!(
        "aim-screenshot-{}-{}.png",
        device.device_id_short, timestamp
    ));

    debug!("Taking screenshot");
    adb::run_shell_command_async(
        host,
        port,
        &format!("screencap -p 2> /dev/null > {}", &temp_file),
        adb_id.map(|x| x.as_str()),
    )
    .await?;

    debug!("Copying screenshot to host");
    adb::pull(
        host,
        port,
        adb_id.map(|x| x.as_str()),
        &PathBuf::from(&temp_file),
        &output_path,
        adb::ProgressDisplay::Hide,
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

    println!("Screenshot saved to: {}", output_path.display());
    Ok(())
}

pub async fn run(
    args: ScreenshotArgs,
    device: &DeviceDetails,
    host: &str,
    port: &str,
) -> Result<(), Box<dyn Error>> {
    // Get output directory
    let base_dir = if let Some(path) = args.output {
        path
    } else {
        let config = Config::load();
        config
            .screenshot
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"))
    };

    if !args.interactive {
        // Single screenshot mode
        return take_single_screenshot(device, base_dir, host, port).await;
    }

    // Interactive mode
    println!("Interactive screenshot mode");
    println!("Press SPACE to take a screenshot");
    println!("Press Ctrl+C to exit");
    enable_raw_mode()?;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(' ') => {
                        if let Err(e) = take_single_screenshot(device, base_dir.clone(), host, port).await {
                            eprintln!("Error taking screenshot: {}", e);
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    println!("\nInteractive mode ended");
    Ok(())
}
