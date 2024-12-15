use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub alias: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Config::get_config_path();
        debug!("Loading config from: {:?}", config_path);

        if let Ok(content) = fs::read_to_string(&config_path) {
            debug!("Config file contents: {}", content);
            match toml::from_str(&content) {
                Ok(config) => {
                    debug!("Parsed config: {:?}", config);
                    config
                }
                Err(e) => {
                    eprintln!("Error parsing config file: {}", e);
                    Config::default()
                }
            }
        } else {
            debug!("No config file found or unable to read it");
            Config::default()
        }
    }

    fn get_config_path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not find home directory");
        home.join(".aimconfig")
    }

    pub fn resolve_alias(&self, command: &str) -> String {
        debug!("Resolving alias for: {}", command);
        debug!("Available aliases: {:?}", self.alias);
        self.alias
            .get(command)
            .cloned()
            .unwrap_or_else(|| command.to_string())
    }
}
