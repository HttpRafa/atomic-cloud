use anyhow::Result;
use simplelog::error;
use simplelog::info;
use std::sync::Arc;
use tonic::async_trait;

use crate::application::cloudlet::Cloudlet;
use crate::application::unit::StartRequestHandle;
use crate::application::unit::UnitHandle;

#[cfg(feature = "wasm-drivers")]
use crate::application::driver::wasm::WasmDriver;

use super::cloudlet::HostAndPort;

mod process;

#[cfg(feature = "wasm-drivers")]
mod wasm;

pub struct Information {
    authors: Vec<String>,
    version: String,
    ready: bool,
}

pub type BoxedDriver = Box<dyn GenericDriver>;
pub type BoxedCloudlet = Box<dyn GenericCloudlet>;

#[async_trait]
pub trait GenericDriver: Send + Sync {
    fn name(&self) -> &str;
    async fn init(&self) -> Result<Information>;
    async fn init_cloudlet(&self, cloudlet: &Cloudlet) -> Result<BoxedCloudlet>;

    /* Cleanup */
    async fn cleanup(&self) -> Result<()>;

    /* Ticking */
    async fn tick(&self) -> Result<()>;
}

#[async_trait]
pub trait GenericCloudlet: Send + Sync {
    /* Ticking */
    async fn tick(&self) -> Result<()>;

    /* Prepare */
    async fn allocate_addresses(&self, request: &StartRequestHandle) -> Result<Vec<HostAndPort>>;
    async fn deallocate_addresses(&self, addresses: Vec<HostAndPort>) -> Result<()>;

    /* Unitss */
    async fn start_unit(&self, unit: &UnitHandle) -> Result<()>;
    async fn restart_unit(&self, unit: &UnitHandle) -> Result<()>;
    async fn stop_unit(&self, unit: &UnitHandle) -> Result<()>;
}

pub struct Drivers {
    drivers: Vec<BoxedDriver>,
}

impl Drivers {
    pub async fn load_all(cloud_identifier: &str) -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();

        #[cfg(feature = "wasm-drivers")]
        WasmDriver::load_all(cloud_identifier, &mut drivers).await;

        info!("Loaded <blue>{} driver(s)</>", drivers.len());
        Self { drivers }
    }

    pub async fn cleanup(&self) {
        for driver in &self.drivers {
            if let Err(error) = driver.cleanup().await {
                error!(
                    "Failed to dispose resources of driver <red>{}</>: <red>{}</>",
                    driver.name(),
                    error
                );
            }
        }
    }

    pub async fn tick(&self) {
        for driver in &self.drivers {
            if let Err(error) = driver.tick().await {
                error!(
                    "Failed to tick driver <red>{}</>: <red>{}</>",
                    driver.name(),
                    error
                );
            }
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Box<dyn GenericDriver>> {
        self.drivers
            .iter()
            .find(|driver| driver.name().eq_ignore_ascii_case(name))
    }

    pub fn get_drivers(&self) -> &Vec<Box<dyn GenericDriver>> {
        &self.drivers
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
