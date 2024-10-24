use anyhow::Result;
use colored::Colorize;
use log::info;
use std::{net::SocketAddr, sync::Arc};
use tonic::async_trait;

use crate::application::node::Node;
use crate::application::server::ServerHandle;
use crate::application::server::StartRequestHandle;

#[cfg(feature = "wasm-drivers")]
use crate::application::driver::wasm::WasmDriver;

#[cfg(feature = "wasm-drivers")]
mod wasm;

pub const DRIVERS_DIRECTORY: &str = "drivers";
pub const DATA_DIRECTORY: &str = "data";

pub struct Information {
    authors: Vec<String>,
    version: String,
    ready: bool,
}

#[async_trait]
pub trait GenericDriver: Send + Sync {
    fn name(&self) -> &String;
    fn init(&self) -> Result<Information>;
    fn init_node(&self, node: &Node) -> Result<DriverNodeHandle>;
}

#[async_trait]
pub trait GenericNode: Send + Sync {
    /* Prepare */
    fn allocate_addresses(&self, request: &StartRequestHandle) -> Result<Vec<SocketAddr>>;
    fn deallocate_addresses(&self, addresses: Vec<SocketAddr>) -> Result<()>;

    /* Servers */
    fn start_server(&self, server: &ServerHandle) -> Result<()>;
    fn restart_server(&self, server: &ServerHandle) -> Result<()>;
    fn stop_server(&self, server: &ServerHandle) -> Result<()>;
}

pub type DriverHandle = Arc<dyn GenericDriver>;
pub type DriverNodeHandle = Arc<dyn GenericNode>;

pub struct Drivers {
    drivers: Vec<DriverHandle>,
}

impl Drivers {
    pub fn load_all(cloud_identifier: &str) -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();

        #[cfg(feature = "wasm-drivers")]
        WasmDriver::load_all(cloud_identifier, &mut drivers);

        info!("Loaded {}", format!("{} driver(s)", drivers.len()).blue());
        Self { drivers }
    }

    pub fn find_by_name(&self, name: &str) -> Option<Arc<dyn GenericDriver>> {
        self.drivers
            .iter()
            .find(|driver| driver.name().eq_ignore_ascii_case(name))
            .cloned()
    }
}

#[cfg(feature = "wasm-drivers")]
mod source {
    use anyhow::Result;
    use std::fmt::{Display, Formatter};
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct Source {
        pub path: PathBuf,
        pub code: Vec<u8>,
    }

    impl Display for Source {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "{}", self.path.display())
        }
    }

    impl Source {
        pub fn from_file(path: &Path) -> Result<Self> {
            let path = path.to_owned();
            let code = fs::read(&path)?;
            Ok(Source { path, code })
        }
    }
}
