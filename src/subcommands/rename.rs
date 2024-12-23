use crate::types::DeviceDetails;
use crate::error::AdbError;
use log::debug;
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
pub fn set_test_config_path(path: PathBuf) -> PathBuf {
    path
}

pub async fn run(device: &DeviceDetails, new_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(test)]
    let config_path = set_test_config_path(get_config_path()?);
    #[cfg(not(test))]
    let config_path = get_config_path()?;
    
    debug!("Config path: {:?}", config_path);

    // Read existing config
    let mut lines = if let Ok(content) = fs::read_to_string(&config_path) {
        content.lines().map(String::from).collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // Find all matching device sections
    let mut matching_sections = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("[device.") && line.trim().ends_with("]") {
            let section_id = line.trim()
                .trim_start_matches("[device.")
                .trim_end_matches("]");
            
            // Check if either ID is a prefix of the other
            if section_id.starts_with(&device.device_id_short) || 
               device.device_id_short.starts_with(section_id) {
                matching_sections.push((i, section_id.to_string()));
            }
        }
    }

    // Handle multiple matches
    if matching_sections.len() > 1 {
        let sections = matching_sections.iter()
            .map(|(_, id)| format!("device.{}", id))
            .collect();
        return Err(Box::new(AdbError::AmbiguousConfigMatch {
            device_id: device.device_id_short.clone(),
            matching_configs: sections,
        }));
    }

    // Use the matched section ID if found, otherwise use the device ID
    let section_id = matching_sections.first()
        .map(|(_, id)| id.clone())
        .unwrap_or_else(|| device.device_id_short.clone());

    let section = format!("[device.{}]", section_id);
    let name_entry = format!("name = \"{}\"", new_name);

    // Find section boundaries
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
            if !lines.is_empty() && !lines.last().unwrap().is_empty() {
                lines.push(String::new());  // Add blank line before new section
            }
            lines.push(section);
            lines.push(name_entry);
            lines.push(String::new());  // Add blank line at the end
        }
    }

    // Write back to file
    let config_dir = config_path.parent().ok_or("Invalid config path")?;
    fs::create_dir_all(config_dir)?;

    let config_string = lines.join("\n") + "\n";  // Ensure trailing newline
    debug!("Writing config:\n{}", config_string);

    fs::write(&config_path, config_string)?;

    println!("Device '{}' renamed to '{}'", section_id, new_name);
    Ok(())
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    Ok(home.join(".aimconfig"))
}
