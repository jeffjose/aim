use crate::types::DeviceDetails;
use log::debug;
use std::fs;
use std::path::PathBuf;

pub async fn run(device: &DeviceDetails, new_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    debug!("Config path: {:?}", config_path);

    // Read existing config
    let mut lines = if let Ok(content) = fs::read_to_string(&config_path) {
        content.lines().map(String::from).collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let device_id = &device.device_id_short;
    let section = format!("[device.{}]", device_id);
    let name_entry = format!("name = \"{}\"", new_name);

    // Find if section already exists
    let mut section_start = None;
    let mut section_end = None;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == section {
            section_start = Some(i);
        } else if section_start.is_some() && line.trim().starts_with('[') {
            section_end = Some(i);
            break;
        }
    }

    match (section_start, section_end) {
        (Some(start), Some(end)) => {
            // Update existing section
            lines.splice(start..end, vec![section.clone(), name_entry.clone()]);
        }
        (Some(start), None) => {
            // Update section at the end of file
            lines.splice(start.., vec![section.clone(), name_entry.clone()]);
        }
        (None, _) => {
            // Add new section at the end
            if !lines.is_empty() {
                lines.push(String::new());  // Add blank line before new section
            }
            lines.push(section);
            lines.push(name_entry);
        }
    }

    // Write back to file
    let config_dir = config_path.parent().ok_or("Invalid config path")?;
    fs::create_dir_all(config_dir)?;

    let config_string = lines.join("\n");
    debug!("Writing config:\n{}", config_string);

    fs::write(&config_path, config_string)?;

    println!("Device '{}' renamed to '{}'", device_id, new_name);
    Ok(())
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home.join(".aimconfig"))
}
