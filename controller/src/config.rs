use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::process::exit;

use inquire::{required, Text};
use log::error;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::config::auto_complete::SimpleAutoComplete;
use crate::config::validators::{AddressValidator, PortValidator};

mod auto_complete;
mod validators;

const CONFIG_DIRECTORY: &str = "configs";
const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub listener: Option<SocketAddr>,
}

impl Config {
    fn new_empty() -> Self {
        Config {
            listener: None,
        }
    }

    fn new() -> Self {
        Self::load_from_file(&Path::new(CONFIG_DIRECTORY).join(CONFIG_FILE))
            .unwrap_or_else(|_| Self::new_empty())
    }

    pub(crate) fn new_filled() -> Config {
        let mut config = Self::new();

        while config.listener.is_none() {
            let address = Text::new("Which address should the TcpListener listen to?")
                .with_autocomplete(SimpleAutoComplete::new(vec!["0.0.0.0"]))
                .with_validator(AddressValidator::new())
                .with_validator(required!())
                .prompt();

            let port = Text::new("On which port should the TcpListener listen?")
                .with_autocomplete(SimpleAutoComplete::new(vec!["51067"]))
                .with_validator(PortValidator::new())
                .with_validator(required!())
                .prompt();

            if let (Ok(address), Ok(port)) = (address, port) {
                config.listener = Some(SocketAddr::new(address.parse().unwrap(), port.parse().unwrap()));
            }
        }


        config.save_toml(&Path::new(CONFIG_DIRECTORY).join(CONFIG_FILE)).unwrap_or_else(|error| {
            error!("Failed to save generated configuration to file: {}", error);
            exit(1);
        });
        config
    }
}

impl SaveToTomlFile for Config {}
impl LoadFromTomlFile for Config {}

pub trait SaveToTomlFile: Serialize {
    fn save_toml(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }
}

pub trait LoadFromTomlFile: DeserializeOwned {
    fn load_from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(&path)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
