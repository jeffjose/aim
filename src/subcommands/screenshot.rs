use crate::{config::Config, library::adb, types::DeviceDetails};
use chrono::Local;
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use log::debug;
use rand::Rng;
use std::{error::Error, io::stdout, path::PathBuf};

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
    interactive: bool,
    count: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    let adb_id = Some(&device.adb_id);

    // Generate random suffix for temp file
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let temp_file = format!("/tmp/screenshot_{}.png", random_suffix);

    // Generate timestamp for output file with microseconds for interactive mode
    let timestamp = if interactive {
        Local::now().format("%Y%m%d-%H%M%S-%6f")
    } else {
        Local::now().format("%Y%m%d-%H%M%S")
    };
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

    if interactive {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        stdout()
            .execute(Clear(ClearType::CurrentLine))?
            .execute(MoveToColumn(0))?;
        println!(
            "[{}] Screenshot #{} saved to: {}",
            timestamp,
            count.unwrap_or(1),
            output_path.display()
        );
    } else {
        println!("Screenshot saved to: {}", output_path.display());
    }
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
            .and_then(|s| s.output)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"))
    };

    if !args.interactive {
        return take_single_screenshot(device, base_dir, host, port, false, None).await;
    }

    // Interactive mode
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("[{}] Interactive screenshot mode", timestamp);
    println!(
        "[{}] Screenshots will be saved to: {}",
        timestamp,
        base_dir.display()
    );
    println!("[{}] Press SPACE to take a screenshot", timestamp);
    println!("[{}] Press Ctrl+C to exit\n", timestamp);
    enable_raw_mode()?;

    let mut screenshot_count = 0;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(' ') => {
                        screenshot_count += 1;
                        if let Err(e) = take_single_screenshot(
                            device,
                            base_dir.clone(),
                            host,
                            port,
                            true,
                            Some(screenshot_count),
                        )
                        .await
                        {
                            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                            stdout()
                                .execute(Clear(ClearType::CurrentLine))?
                                .execute(MoveToColumn(0))?;
                            eprintln!("[{}] Error taking screenshot: {}", timestamp, e);
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
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    stdout()
        .execute(Clear(ClearType::CurrentLine))?
        .execute(MoveToColumn(0))?;
    println!(
        "[{}] Interactive mode ended. {} screenshots taken.",
        timestamp, screenshot_count
    );
    Ok(())
}
