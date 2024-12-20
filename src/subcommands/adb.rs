use std::process::Command;
use std::error::Error;

pub struct AdbArgs {
    pub args: Vec<String>,
}

pub async fn run(args: AdbArgs) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("adb");
    
    // Pass all arguments including flags directly to adb
    command.args(&args.args);

    let output = command.output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        return Err(format!(
            "adb command failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        ).into());
    }

    Ok(())
} 
