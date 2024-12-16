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

        debug!("Loading config from: {:?}", config_path);

        if let Ok(contents) = std::fs::read_to_string(config_path) {
            debug!("Config contents:\n{}", contents);
            let mut config = Config::default();
            
            if let Ok(toml) = contents.parse::<toml::Table>() {
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
                for (key, value) in toml.iter() {
                    debug!("Processing section: {} = {:?}", key, value);
                    if let Some(device_id) = key.strip_prefix("device.") {
                        if let Some(table) = value.as_table() {
                            if let Some(name) = table.get("name").and_then(|v| v.as_str()) {
                                debug!("Adding device config: {} -> {}", device_id, name);
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
            debug!("Final config: {:?}", config);
            config
        } else {
            Config::default()
        }
    }

    pub fn resolve_alias(&self, cmd: &str) -> String {
        self.aliases
            .get(cmd)
            .cloned()
            .unwrap_or_else(|| cmd.to_string())
    }

    pub fn get_device_name(&self, device_id: &str) -> Option<String> {
        self.devices.get(device_id).and_then(|d| d.name.clone())
    }
}
