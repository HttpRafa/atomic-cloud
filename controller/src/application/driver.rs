use anyhow::Result;
use simplelog::info;
use std::sync::Arc;
use tonic::async_trait;

use crate::application::cloudlet::Cloudlet;
use crate::application::unit::StartRequestHandle;
use crate::application::unit::UnitHandle;

#[cfg(feature = "wasm-drivers")]
use crate::application::driver::wasm::WasmDriver;

use super::cloudlet::HostAndPort;

#[cfg(feature = "wasm-drivers")]
mod wasm;

pub struct Information {
    authors: Vec<String>,
    version: String,
    ready: bool,
}

#[async_trait]
pub trait GenericDriver: Send + Sync {
    fn name(&self) -> &String;
    fn init(&self) -> Result<Information>;
    fn init_cloudlet(&self, cloudlet: &Cloudlet) -> Result<DriverCloudletHandle>;
}

#[async_trait]
pub trait GenericCloudlet: Send + Sync {
    /* Prepare */
    fn allocate_addresses(&self, request: &StartRequestHandle) -> Result<Vec<HostAndPort>>;
    fn deallocate_addresses(&self, addresses: Vec<HostAndPort>) -> Result<()>;

    /* Unitss */
    fn start_unit(&self, unit: &UnitHandle) -> Result<()>;
    fn restart_unit(&self, unit: &UnitHandle) -> Result<()>;
    fn stop_unit(&self, unit: &UnitHandle) -> Result<()>;
}

pub type DriverHandle = Arc<dyn GenericDriver>;
pub type DriverCloudletHandle = Arc<dyn GenericCloudlet>;

pub struct Drivers {
    drivers: Vec<DriverHandle>,
}

impl Drivers {
    pub fn load_all(cloud_identifier: &str) -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();

        #[cfg(feature = "wasm-drivers")]
        WasmDriver::load_all(cloud_identifier, &mut drivers);

        info!("Loaded <blue>{} driver(s)</>", drivers.len());
        Self { drivers }
    }

    pub fn find_by_name(&self, name: &str) -> Option<Arc<dyn GenericDriver>> {
        self.drivers
            .iter()
            .find(|driver| driver.name().eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn get_drivers(&self) -> Vec<DriverHandle> {
        self.drivers.clone()
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
