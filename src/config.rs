use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    #[serde(default)]
    pub devices: HashMap<String, DeviceConfig>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DeviceConfig {
    pub name: Option<String>,
}

impl Config {
    pub fn load_from_path(config_path: &PathBuf) -> Self {
        debug!("Loading config from: {:?}", config_path);

        match std::fs::read_to_string(config_path) {
            Ok(contents) => {
                debug!("Config contents:\n{}", contents);
                let mut config = Config::default();
                
                match contents.parse::<toml::Table>() {
                    Ok(toml) => {
                        // Parse alias section
                        if let Some(aliases) = toml.get("alias").and_then(|v| v.as_table()) {
                            debug!("Processing alias section: {:?}", aliases);
                            for (key, value) in aliases {
                                if let Some(cmd) = value.as_str() {
                                    debug!("Adding alias: {} -> {}", key, cmd);
                                    config.aliases.insert(key.clone(), cmd.to_string());
                                }
                            }
                        }

                        // Parse device sections
                        if let Some(device_section) = toml.get("device").and_then(|v| v.as_table()) {
                            debug!("Processing device section: {:?}", device_section);
                            for (device_id, value) in device_section {
                                debug!("Processing device: {} = {:?}", device_id, value);
                                if let Some(table) = value.as_table() {
                                    debug!("Processing device config: {:?}", table);
                                    let device_config = DeviceConfig {
                                        name: table.get("name").and_then(|v| v.as_str()).map(String::from),
                                    };
                                    debug!("Adding device config: {} -> {:?}", device_id, device_config);
                                    config.devices.insert(device_id.to_string(), device_config);
                                }
                            }
                        }
                        debug!("Final config: {:?}", config);
                        config
                    }
                    Err(e) => {
                        eprintln!("Error parsing config file {}: {}", config_path.display(), e);
                        Config::default()
                    }
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("Error reading config file {}: {}", config_path.display(), e);
                }
                Config::default()
            }
        }
    }

    pub fn load() -> Self {
        let config_path = dirs::home_dir()
            .map(|mut path| {
                path.push(".aimconfig");
                path
            })
            .unwrap_or_else(|| PathBuf::from(".aimconfig"));

        Self::load_from_path(&config_path)
    }

    pub fn resolve_alias(&self, cmd: &str) -> String {
        self.aliases
            .get(cmd)
            .cloned()
            .unwrap_or_else(|| cmd.to_string())
    }

    pub fn get_device_name(&self, device_id: &str) -> Option<String> {
        let matches: Vec<(&String, &DeviceConfig)> = self.devices
            .iter()
            .filter(|(id, _)| {
                let id = id.to_lowercase();
                let device_id = device_id.to_lowercase();
                id.starts_with(&device_id) || device_id.starts_with(&id)
            })
            .collect();

        match matches.len() {
            0 => None,
            1 => matches[0].1.name.clone(),
            _ => {
                let matching_sections: Vec<String> = matches
                    .iter()
                    .map(|(id, _)| format!("device.{}", id))
                    .collect();
                println!(
                    "Warning: Multiple config sections match device '{}': {}",
                    device_id,
                    matching_sections.join(", ")
                );
                None
            }
        }
    }
}
