use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use colored::Colorize;
use log::{error, info, warn};
use minreq::URL;
use serde::{Deserialize, Serialize};

use crate::{AUTHORS, VERSION};
use crate::config::{LoadFromTomlFile, SaveToTomlFile};
use crate::driver::{Driver, DRIVERS_DIRECTORY, Information};
use crate::node::Node;

const PELICAN_DIRECTORY: &str = "pelican";

#[derive(Serialize, Deserialize)]
pub(self) struct PelicanConfig {
    pub endpoint: URL,
}

pub struct PelicanDriver {
    pub name: String,
    pub config: PelicanConfig,
}

impl Driver for PelicanDriver {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn init(&self) -> Result<Information, Box<dyn Error>> {
        Ok(Information {
            author: AUTHORS.join(", "),
            version: VERSION.to_string(),
        })
    }

    fn init_node(&self, _node: &Node) -> Result<bool, Box<dyn Error>> {
        todo!()
    }

    fn stop_server(&self, _server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn start_server(&self, _server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

impl PelicanDriver {
    fn new(name: String, config: PelicanConfig) -> Self {
        Self { name, config }
    }

    pub fn load_drivers(drivers: &mut Vec<Arc<dyn Driver>>) {
        let old_loaded = drivers.len();

        let drivers_directory = Path::new(DRIVERS_DIRECTORY).join(PELICAN_DIRECTORY);
        if !drivers_directory.exists() {
            fs::create_dir_all(&drivers_directory).unwrap_or_else(|error| {
                warn!(
                    "{} to create pelican drivers directory: {}",
                    "Failed".red(),
                    &error
                )
            });
        }

        let entries = match fs::read_dir(&drivers_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read pelican driver directory: {}", "Failed".red(), &error);
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read pelican driver entry: {}", "Failed".red(), &error);
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() {
                warn!(
                    "The driver directory should only contain TOML files, please remove {:?}",
                    &entry.file_name()
                );
                continue;
            }

            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let config = match PelicanConfig::load_from_file(&path) {
                Ok(config) => config,
                Err(error) => {
                    error!(
                        "{} to read config for pelican driver instance from file({:?}): {}",
                        "Failed".red(),
                        &path,
                        &error
                    );
                    continue;
                }
            };

            let driver = PelicanDriver::new(name, config);
            match driver.init() {
                Ok(_) => {
                    info!(
                        "Created pelican driver instance {} pointing to {}",
                        &driver.name.blue(),
                        &driver.config.endpoint.blue()
                    );
                    drivers.push(Arc::new(driver));
                }
                Err(error) => {
                    error!(
                        "{} to create pelican driver instance {}: {}",
                        "Failed".red(),
                        &driver.name,
                        &error
                    );
                }
            }
        }

        if old_loaded == drivers.len() {
            warn!("The Pelican driver feature is enabled, but no instances of the Pelican driver were created.");
        }
    }
}

impl SaveToTomlFile for PelicanConfig {}
impl LoadFromTomlFile for PelicanConfig {}