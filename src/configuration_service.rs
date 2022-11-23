use std::fs;

use serde::{Serialize, Deserialize};

pub struct ConfigurationService {
    configuration: Configuration,
    config_file: String
}

impl ConfigurationService {
    pub fn new() -> ConfigurationService {
        let config_dir = dirs::config_dir().unwrap();
        let config_file = config_dir.as_path().join("notion_backup.conf");

        let configuration: Configuration;

        match fs::read_to_string(&config_file) {
            Ok(content) => {
                configuration = serde_json::from_str(&content).unwrap_or(Configuration::empty());
            }
            Err(_) => {
                configuration = Configuration::empty();
            }
        }

        ConfigurationService {
            configuration,
            config_file: String::from(config_file.to_str().unwrap())
        }
    }

    pub fn get_token(&self) -> Option<&String> {
        let token = &self.configuration.token;

        if token.is_empty() {
            return None
        };

        Some(token)
    }

    pub fn set_token(&mut self, token: &str) -> &String {
        self.configuration.token = String::from(token);
        self.write_config();

        &self.configuration.token
    }

    fn write_config(&self) {
        match serde_json::to_string(&self.configuration) {
            Ok(json) => {
                match fs::write(&self.config_file, json) {
                    Ok(_) => (),
                    Err(_) => ()
                }
            }
            Err(_) => ()
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Configuration {
    token: String
}

impl Configuration {
    fn empty() -> Configuration {
        Configuration {
            token: String::new()
        }
    }
}