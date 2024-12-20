use dirs::home_dir;
use std::error::Error;
use std::path::PathBuf;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let config_path = home_dir()
        .ok_or("Could not determine home directory")?
        .join(".aimconfig");

    if !config_path.exists() {
        println!("No config file found at: {:?}", config_path);
        println!("Default configuration will be used.");
        return Ok(());
    }

    println!("Reading {}\n", config_path.display());
    let contents = std::fs::read_to_string(&config_path)?;
    println!("{}", contents);

    Ok(())
}
