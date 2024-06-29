use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::Result;
use log::{error, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CONFIG_DIRECTORY: &str = "configs";
const CONFIG_FILE: &str = "config.toml";

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";
const DEFAULT_BIND_PORT: u16 = 12892;

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    /* Cloud Identification */
    pub identifier: Option<String>,

    /* Network */
    pub listener: Option<SocketAddr>,
}

impl Config {
    fn load_or_empty() -> Self {
        let path = Path::new(CONFIG_DIRECTORY).join(CONFIG_FILE);
        if !path.exists() {
            return Self::default();
        }
        Self::load_from_file(&path).unwrap_or_else(|error| {
            warn!("Failed to read configuration from file: {}", error);
            Self::default()
        })
    }

    pub fn new_filled() -> Self {
        let mut config = Self::load_or_empty();

        let mut save = false;
        if config.identifier.is_none() {
            config.identifier = Some(Uuid::new_v4().to_string());
            save = true;
        }
        if config.listener.is_none() {
            config.listener = Some(SocketAddr::new(
                DEFAULT_BIND_ADDRESS.parse().unwrap(),
                DEFAULT_BIND_PORT,
            ));
            save = true;
        }
        if save {
            if let Err(error) = config.save_to_file(&Path::new(CONFIG_DIRECTORY).join(CONFIG_FILE))
            {
                error!("Failed to save generated configuration to file: {}", &error);
            }
        }

        // Check config values are overridden by environment variables
        if let Ok(identifier) = std::env::var("INSTANCE_IDENTIFIER") {
            config.identifier = Some(identifier);
        }
        if let Ok(address) = std::env::var("BIND_ADDRESS") {
            if let Ok(address) = address.parse() {
                config.listener.unwrap().set_ip(address);
            } else {
                error!("Failed to parse BIND_ADDRESS environment variable");
            }
        }
        if let Ok(port) = std::env::var("BIND_PORT") {
            if let Ok(port) = port.parse() {
                config.listener.unwrap().set_port(port);
            } else {
                error!("Failed to parse BIND_PORT environment variable");
            }
        }

        config
    }
}

impl SaveToTomlFile for Config {}
impl LoadFromTomlFile for Config {}

pub trait SaveToTomlFile: Serialize {
    fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }
}

pub trait LoadFromTomlFile: DeserializeOwned {
    fn load_from_file(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
