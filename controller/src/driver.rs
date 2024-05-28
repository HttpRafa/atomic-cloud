use std::error::Error;
use std::sync::{Arc, Mutex};
use colored::Colorize;
use log::info;
use serde::Deserialize;

use crate::node::Node;

#[cfg(feature = "wasm-drivers")]
use crate::driver::wasm::WasmDriver;

#[cfg(feature = "wasm-drivers")]
mod wasm;

const DRIVERS_DIRECTORY: &str = "drivers";

pub trait GenericDriver {
    fn name(&self) -> String;
    fn init(&self) -> Result<Information, Box<dyn Error>>;
    fn init_node(&self, node: &Node) -> Result<bool, Box<dyn Error>>;
    fn stop_server(&self, server: &str) -> Result<(), Box<dyn Error>>;
    fn start_server(&self, server: &str) -> Result<(), Box<dyn Error>>;
}

pub struct Drivers {
    drivers: Vec<Arc<dyn GenericDriver>>,
}

impl Drivers {
    pub fn load_all() -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();

        #[cfg(feature = "wasm-drivers")]
        WasmDriver::load_all(&mut drivers);

        info!("Loaded {}", format!("{} driver(s)", drivers.len()).blue());
        Self { drivers }
    }

    pub fn find_by_name(&self, name: &str) -> Option<Arc<dyn GenericDriver>> {
        self.drivers.iter()
            .find(|driver| driver.name().eq_ignore_ascii_case(name))
            .map(Arc::clone)
    }
}

pub struct Information {
    author: String,
    version: String,
}

#[cfg(feature = "wasm-drivers")]
mod source {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct Source {
        pub path: PathBuf,
        pub code: Vec<u8>,
    }

    impl Source {
        pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
            let path = path.to_owned();
            let code = fs::read(&path)?;
            Ok(Source { path, code })
        }
    }
}