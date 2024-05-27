use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use colored::Colorize;
use log::{error, info, warn};
use crate::driver::{Driver, DRIVERS_DIRECTORY, Information};
use crate::node::Node;

const PELICAN_DIRECTORY: &str = "pelican";

pub struct PelicanDriver {
    pub name: String,
}

impl Driver for PelicanDriver {
    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn init(&self) -> Result<Information, Box<dyn Error>> {
        todo!()
    }

    fn init_node(&self, node: &Node) -> Result<bool, Box<dyn Error>> {
        todo!()
    }

    fn stop_server(&self, server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn start_server(&self, server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

impl PelicanDriver {
    pub fn load_drivers(drivers: &mut Vec<Arc<dyn Driver>>) {
        let old_loaded = drivers.len();

        let drivers_directory = Path::new(DRIVERS_DIRECTORY).join(PELICAN_DIRECTORY);
        if !drivers_directory.exists() {
            fs::create_dir_all(&drivers_directory).unwrap_or_else(|error| warn!("{} to create pelican drivers directory: {}", "Failed".red(), error));
        }

        if old_loaded == drivers.len() {
            warn!("The Pelican driver feature is enabled, but no instances of the Pelican driver were created.");
        }
    }
}