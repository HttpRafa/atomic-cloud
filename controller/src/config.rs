use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use inquire::{required, Text};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::config::auto_complete::SimpleAutoComplete;
use crate::config::validators::{AddressValidator, PortValidator};

mod auto_complete;
mod validators;

const CONFIG_DIRECTORY: &str = "configs";
const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize, Serialize)]
pub(crate) struct Config {
    pub listener: Option<SocketAddr>,
}

impl Config {
    fn new_empty() -> Self {
        Config {
            listener: None,
        }
    }

    fn new() -> Self {
        Self::load_from_file(Path::new(CONFIG_DIRECTORY).join(CONFIG_FILE))
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


        config.save_toml(Path::new(CONFIG_DIRECTORY).join(CONFIG_FILE));
        config
    }
}

impl SaveToml for Config {}
impl LoadFromFile for Config {}

pub trait SaveToml: Serialize {
    fn save_toml(&self, path: PathBuf) {
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create all directories required");
        fs::write(path, toml::to_string(self).expect("Failed to convert configuration to toml")).expect("Failed to write configuration to file");
    }
}

pub trait LoadFromFile: DeserializeOwned {
    fn load_from_file(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(&path)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
