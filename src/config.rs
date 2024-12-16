use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
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
    pub fn load() -> Self {
        let config_path = dirs::home_dir()
            .map(|mut path| {
                path.push(".aimconfig");
                path
            })
            .unwrap_or_else(|| PathBuf::from(".aimconfig"));

        if let Ok(contents) = std::fs::read_to_string(config_path) {
            let mut config: Config = toml::from_str(&contents).unwrap_or_default();
            
            // Parse device sections from [device.*] format
            if let Ok(toml) = contents.parse::<toml::Table>() {
                for (key, value) in toml.iter() {
                    if let Some(device_id) = key.strip_prefix("device.") {
                        if let Some(table) = value.as_table() {
                            if let Some(name) = table.get("name").and_then(|v| v.as_str()) {
                                config.devices.insert(
                                    device_id.to_string(),
                                    DeviceConfig {
                                        name: Some(name.to_string()),
                                    },
                                );
                            }
                        }
                    }
                }
            }
            config
        } else {
            Config::default()
        }
    }

    pub fn resolve_alias(&self, cmd: &str) -> String {
        self.aliases.get(cmd).cloned().unwrap_or_else(|| cmd.to_string())
    }

    pub fn get_device_name(&self, device_id: &str) -> Option<String> {
        self.devices.get(device_id).and_then(|d| d.name.clone())
    }
}
