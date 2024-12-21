use std::process::{Command, Stdio};
use std::error::Error;

pub struct AdbArgs<'a> {
    pub command: &'a str,
    pub adb_id: &'a str,
}

pub async fn run(args: AdbArgs<'_>) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("adb");
    
    // Add the device selector flag and ID
    command.arg("-s");
    command.arg(args.adb_id);
    
    // Split and pass the command string as arguments
    command.args(args.command.split_whitespace());

    // Set up for interactive use
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Spawn and wait for the command
    let mut child = command.spawn()?;
    let status = child.wait()?;

    if !status.success() {
        return Err(format!(
            "adb command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ).into());
    }

    Ok(())
} 
