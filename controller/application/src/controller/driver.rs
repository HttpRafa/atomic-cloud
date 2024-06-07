use std::sync::Arc;
use anyhow::Result;
use colored::Colorize;
use log::info;
use tonic::async_trait;

use crate::controller::node::Node;

#[cfg(feature = "wasm-drivers")]
use crate::controller::driver::wasm::WasmDriver;

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
    async fn init(&self) -> Result<Information>;
    async fn init_node(&self, node: &Node) -> Result<Result<DriverNodeHandle, String>>;
    async fn stop_server(&self, server: &str) -> Result<()>;
    async fn start_server(&self, server: &str) -> Result<()>;
}

#[async_trait]
pub trait GenericNode: Send + Sync {
    async fn allocate_ports(&self, amount: u32) -> Result<Result<Vec<u32>, String>>;
}

pub type DriverHandle = Arc<dyn GenericDriver>;
pub type DriverNodeHandle = Arc<dyn GenericNode>;

pub struct Drivers {
    drivers: Vec<DriverHandle>,
}

impl Drivers {
    pub async fn load_all() -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();

        #[cfg(feature = "wasm-drivers")]
        WasmDriver::load_all(&mut drivers).await;

        info!("Loaded {}", format!("{} driver(s)", drivers.len()).blue());
        Self { drivers }
    }

    pub fn find_by_name(&self, name: &str) -> Option<Arc<dyn GenericDriver>> {
        self.drivers.iter()
            .find(|driver| driver.name().eq_ignore_ascii_case(name))
            .cloned()
    }
}

#[cfg(feature = "wasm-drivers")]
mod source {
    use std::fs;
    use std::path::{Path, PathBuf};
    use anyhow::Result;

    pub struct Source {
        pub path: PathBuf,
        pub code: Vec<u8>,
    }

    impl Source {
        pub fn from_file(path: &Path) -> Result<Self> {
            let path = path.to_owned();
            let code = fs::read(&path)?;
            Ok(Source { path, code })
        }
    }
}