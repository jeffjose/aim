use std::process::{Command, Stdio};
use std::error::Error;

pub struct AdbArgs {
    pub args: Vec<String>,
}

pub async fn run(args: AdbArgs) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("adb");
    
    // Pass all arguments including flags directly to adb
    command.args(&args.args);

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
