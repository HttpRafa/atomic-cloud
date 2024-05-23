mod auto_complete;
mod validators;

use std::fs;
use std::fs::read_dir;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::exit;
use inquire::{required, Select, Text};
use log::error;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::config::auto_complete::SimpleAutoComplete;
use crate::config::validators::{AddressValidator, PortValidator};

const CONFIG_DIRECTORY: &str = "configs";
const CONFIG_FILE: &str = "config.toml";
const DRIVER_DIRECTORY: &str = "drivers";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub driver: Option<String>,
    pub listener: Option<SocketAddr>,
}

impl Config {
    fn new_empty() -> Self {
        Config {
            driver: None,
            listener: None,
        }
    }

    fn new() -> Self {
        Self::load_from_file(Path::new(CONFIG_DIRECTORY).join(CONFIG_FILE))
            .unwrap_or_else(|_| Self::new_empty())
    }

    pub fn new_filled() -> Config {
        let mut config = Self::new();

        if config.driver.is_none() {
            let path = Path::new(DRIVER_DIRECTORY);
            if !path.exists() {
                error!("No drivers are installed please install some");
                exit(1);
            }

            let drivers = read_dir(path)
                .expect("Failed to read driver directory")
                .filter_map(Result::ok)
                .map(|e| {
                    e.path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned()
                })
                .collect::<Vec<_>>();
            if drivers.is_empty() {
                error!("No drivers are installed please install some");
                exit(1);
            }

            let driver = Select::new("What backend driver do you want to use?", drivers)
                .prompt()
                .expect("Failed to read user input");
            config.driver = Some(driver);
        }

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
