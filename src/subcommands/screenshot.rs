use crate::{config::Config, library::adb, types::DeviceDetails};
use log::debug;
use rand::Rng;
use std::error::Error;
use std::path::PathBuf;

pub struct ScreenshotArgs {
    pub device_id: Option<String>,
    pub output: Option<PathBuf>,
}

pub async fn run(
    args: ScreenshotArgs,
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
    let temp_file = format!("/tmp/screenshot_{}.png", random_suffix);

    // Generate output filename
    let output_path = if let Some(path) = args.output {
        path
    } else {
        // Check config for screenshot directory
        let config = Config::load();
        let base_dir = config
            .screenshot
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));

        // Generate random suffix for output file
        let random_suffix: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        base_dir.join(format!(
            "aim-{}-{}.png",
            device.device_id_short, random_suffix
        ))
    };

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
    adb::run_shell_command_async(
        host,
        port,
        &format!("rm {}", &temp_file),
        adb_id.map(|x| x.as_str()),
    )
    .await?;

    println!("Screenshot saved to: {}", output_path.display());
    Ok(())
}
