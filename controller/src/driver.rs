use std::error::Error;
use std::sync::Arc;
use colored::Colorize;
use log::info;
use serde::Deserialize;
use crate::node::Node;

#[cfg(feature = "pelican-drivers")]
use crate::driver::pelican::PelicanDriver;
#[cfg(feature = "lua-drivers")]
use crate::driver::lua::LuaDriver;

#[cfg(feature = "pelican-drivers")]
mod pelican;
#[cfg(feature = "lua-drivers")]
mod lua;

const DRIVERS_DIRECTORY: &str = "drivers";

pub trait Driver {
    fn name(&self) -> String;
    fn init(&self) -> Result<Information, Box<dyn Error>>;
    fn init_node(&self, node: &Node) -> Result<bool, Box<dyn Error>>;

    fn stop_server(&self, server: &str) -> Result<(), Box<dyn Error>>;
    fn start_server(&self, server: &str) -> Result<(), Box<dyn Error>>;
}

pub struct Drivers {
    drivers: Vec<Arc<dyn Driver>>,
}

impl Drivers {
    pub fn load_all() -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();

        #[cfg(feature = "pelican-drivers")]
        PelicanDriver::load_drivers(&mut drivers);

        #[cfg(feature = "lua-drivers")]
        LuaDriver::load_drivers(&mut drivers);

        info!("Loaded {}", format!("{} driver(s)", drivers.len()).blue());
        Drivers { drivers }
    }
    pub fn find_by_name(&self, name: &String) -> Option<Arc<dyn Driver>> {
        for driver in &self.drivers {
            if driver.name().eq_ignore_ascii_case(&name) {
                return Some(Arc::clone(driver));
            }
        }
        None
    }
}

#[derive(Deserialize)]
pub struct Information {
    author: String,
    version: String,
}

#[cfg(feature = "lua-drivers")]
mod source {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct Source {
        pub path: PathBuf,
        pub code: String,
    }

    impl Source {
        pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
            let path = path.to_owned();
            let code = fs::read_to_string(&path)?;
            Ok(Source { path, code })
        }
    }
}