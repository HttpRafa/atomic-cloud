use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    sync::{Arc, RwLock, Weak},
};

use anyhow::{anyhow, Result};
use colored::Colorize;
use common::config::{LoadFromTomlFile, SaveToTomlFile};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use stored::StoredCloudlet;
use url::Url;

use super::{
    driver::{DriverCloudletHandle, DriverHandle, Drivers, GenericDriver},
    unit::{Resources, Spec, StartRequestHandle},
    CreationResult, WeakControllerHandle,
};
use crate::storage::Storage;

pub type CloudletHandle = Arc<Cloudlet>;
pub type WeakCloudletHandle = Weak<Cloudlet>;

pub struct Cloudlets {
    controller: WeakControllerHandle,

    cloudlets: HashMap<String, CloudletHandle>,
}

impl Cloudlets {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            cloudlets: HashMap::new(),
        }
    }

    /// This will try to load all the cloudletss stored as toml files from the cloudlets directory
    ///
    /// Any compilcations will be logged and the cloudlet will be skipped
    pub fn load_all(controller: WeakControllerHandle, drivers: &Drivers) -> Self {
        info!("Loading cloudlets...");

        let mut cloudlets = Self::new(controller);
        let cloudlets_directory = Storage::get_cloudlets_folder();
        if !cloudlets_directory.exists() {
            if let Err(error) = fs::create_dir_all(&cloudlets_directory) {
                warn!(
                    "{} to create cloudlets directory: {}",
                    "Failed".red(),
                    &error
                );
                return cloudlets;
            }
        }

        let entries = match fs::read_dir(&cloudlets_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read cloudlets directory: {}", "Failed".red(), &error);
                return cloudlets;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read cloudlet entry: {}", "Failed".red(), &error);
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let name = match path.file_stem() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let cloudlet = match StoredCloudlet::load_from_file(&path) {
                Ok(cloudlet) => cloudlet,
                Err(error) => {
                    error!(
                        "{} to read cloudlet {} from file({:?}): {}",
                        "Failed".red(),
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            info!("Loading cloudlet {}", &name.blue());
            let cloudlet = match Cloudlet::try_from(&name, &cloudlet, drivers) {
                Some(cloudlet) => cloudlet,
                None => continue,
            };

            if let Err(error) = cloudlets.add_cloudlet(cloudlet) {
                warn!(
                    "{} to load cloudlet {} because it was denied by the driver",
                    "Failed".red(),
                    &name.blue()
                );
                warn!(" -> {}", &error);
            }
        }

        info!(
            "Loaded {}",
            format!("{} cloudlet(s)", cloudlets.cloudlets.len()).blue()
        );
        cloudlets
    }

    pub fn get_amount(&self) -> usize {
        self.cloudlets.len()
    }

    pub fn get_cloudlets(&self) -> Vec<CloudletHandle> {
        self.cloudlets.values().cloned().collect()
    }

    pub fn find_by_name(&self, name: &str) -> Option<CloudletHandle> {
        self.cloudlets.get(name).cloned()
    }

    /// This can be used to retire or activate a cloudlet
    ///
    /// Retiring a cloudlet will remove it from the deployments that use it and stop all units on it
    pub fn set_cloudlet_status(
        &mut self,
        cloudlet: &CloudletHandle,
        status: LifecycleStatus,
    ) -> Result<()> {
        match status {
            LifecycleStatus::Retired => {
                self.retire_cloudlet(cloudlet);
                info!("Retired cloudlet {}", cloudlet.name.blue());
            }
            LifecycleStatus::Active => {
                self.activate_cloudlet(cloudlet);
                info!("Activated cloudlet {}", cloudlet.name.blue());
            }
        }
        *cloudlet.status.write().unwrap() = status;
        cloudlet.mark_dirty()?;
        Ok(())
    }

    /// This should only be called from set_cloudlet_status and delete_cloudlet
    fn retire_cloudlet(&mut self, cloudlet: &CloudletHandle) {
        let controller = self
            .controller
            .upgrade()
            .expect("The controller is dead while still running code that requires it");
        {
            controller
                .lock_deployments()
                .search_and_remove_cloudlet(cloudlet);
            controller.get_units().stop_all_on_cloudlet(cloudlet);
        }
    }

    /// This should only be called from set_cloudlet_status
    fn activate_cloudlet(&mut self, _cloudlet: &CloudletHandle) {}

    pub fn delete_cloudlet(&mut self, cloudlet: &CloudletHandle) -> Result<()> {
        if *cloudlet
            .status
            .read()
            .expect("Failed to lock status of cloudlet")
            != LifecycleStatus::Retired
        {
            return Err(anyhow!("Cloudlet is not retired"));
        }
        self.retire_cloudlet(cloudlet); // Just to be sure
        cloudlet.delete_file()?;
        self.remove_cloudlet(cloudlet);

        let ref_count = Arc::strong_count(cloudlet);
        if ref_count > 1 {
            warn!(
                "Cloudlet {} still has strong references[{}] this chould indicate a memory leak!",
                cloudlet.name.blue(),
                format!("{}", ref_count).red()
            );
        }

        info!("Deleted cloudlet {}", cloudlet.name.blue());
        Ok(())
    }

    pub fn create_cloudlet(
        &mut self,
        name: &str,
        driver: Arc<dyn GenericDriver>,
        capabilities: Capabilities,
        controller: RemoteController,
    ) -> Result<CreationResult> {
        if self.cloudlets.contains_key(name) {
            return Ok(CreationResult::AlreadyExists);
        }

        let stored_cloudlet = StoredCloudlet {
            driver: driver.name().to_string(),
            capabilities,
            status: LifecycleStatus::Retired,
            controller,
        };
        let cloudlet = Cloudlet::from(name, &stored_cloudlet, driver);

        match self.add_cloudlet(cloudlet) {
            Ok(_) => {
                stored_cloudlet.save_to_file(&Storage::get_cloudlet_file(name))?;
                info!("Created cloudlet {}", name.blue());
                Ok(CreationResult::Created)
            }
            Err(error) => Ok(CreationResult::Denied(error)),
        }
    }

    fn add_cloudlet(&mut self, mut cloudlet: Cloudlet) -> Result<()> {
        match cloudlet.init() {
            Ok(_) => {
                self.cloudlets
                    .insert(cloudlet.name.clone(), Arc::new(cloudlet));
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn remove_cloudlet(&mut self, cloudlet: &CloudletHandle) {
        self.cloudlets.remove(&cloudlet.name);
    }
}

pub type AllocationHandle = Arc<Allocation>;

pub struct Allocation {
    pub addresses: Vec<SocketAddr>,
    pub resources: Resources,
    pub spec: Spec,
}

impl Allocation {
    pub fn primary_address(&self) -> &SocketAddr {
        &self.addresses[0]
    }
}

pub struct Cloudlet {
    /* Settings */
    pub name: String,
    pub capabilities: Capabilities,
    pub status: RwLock<LifecycleStatus>,

    /* Controller */
    pub controller: RemoteController,

    /* Driver handles */
    pub driver: DriverHandle,
    inner: Option<DriverCloudletHandle>,

    /* Allocations made on this cloudlet */
    pub allocations: RwLock<Vec<AllocationHandle>>,
}

impl Cloudlet {
    fn from(name: &str, stored_cloudlet: &StoredCloudlet, driver: Arc<dyn GenericDriver>) -> Self {
        Self {
            name: name.to_string(),
            capabilities: stored_cloudlet.capabilities.clone(),
            status: RwLock::new(stored_cloudlet.status.clone()),
            controller: stored_cloudlet.controller.clone(),
            driver,
            inner: None,
            allocations: RwLock::new(Vec::new()),
        }
    }

    fn try_from(name: &str, stored_cloudlet: &StoredCloudlet, drivers: &Drivers) -> Option<Self> {
        drivers
            .find_by_name(&stored_cloudlet.driver)
            .map(|driver| Self::from(name, stored_cloudlet, driver))
            .or_else(|| {
                error!(
                    "{} to load cloudlet {} because there is no loaded driver with the name {}",
                    "Failed".red(),
                    &name.red(),
                    &stored_cloudlet.driver.red()
                );
                None
            })
    }

    pub fn init(&mut self) -> Result<()> {
        match self.driver.init_cloudlet(self) {
            Ok(value) => {
                self.inner = Some(value);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn allocate(&self, request: &StartRequestHandle) -> Result<AllocationHandle> {
        if *self.status.read().unwrap() == LifecycleStatus::Retired {
            warn!(
                "Attempted to allocate resources on {} cloudlet {}",
                "retired".red(),
                self.name.blue()
            );
            return Err(anyhow!("Can not allocate resources on retired cloudlet"));
        }

        let mut allocations = self
            .allocations
            .write()
            .expect("Failed to lock allocations");

        if let Some(max_memory) = self.capabilities.memory {
            let used_memory: u32 = allocations
                .iter()
                .map(|allocation| allocation.resources.memory)
                .sum();
            if used_memory > max_memory {
                return Err(anyhow!("Cloudlet has reached the memory limit"));
            }
        }

        if let Some(max_allocations) = self.capabilities.max_allocations {
            if allocations.len() + 1 > max_allocations as usize {
                return Err(anyhow!(
                    "Cloudlet has reached the maximum amount of allocations"
                ));
            }
        }

        let addresses = self.inner.as_ref().unwrap().allocate_addresses(request)?;
        if addresses.len() < request.resources.addresses as usize {
            return Err(anyhow!(
                "Cloudlet did not allocate the required amount of addresses"
            ));
        }

        let allocation = Arc::new(Allocation {
            addresses,
            resources: request.resources.clone(),
            spec: request.spec.clone(),
        });
        allocations.push(allocation.clone());
        Ok(allocation)
    }

    pub fn deallocate(&self, allocation: &AllocationHandle) {
        if let Err(error) = self
            .inner
            .as_ref()
            .unwrap()
            .deallocate_addresses(allocation.addresses.clone())
        {
            error!("{} to deallocate addresses: {}", "Failed".red(), &error);
        }
        self.allocations
            .write()
            .expect("Failed to lock allocations")
            .retain(|alloc| !Arc::ptr_eq(alloc, allocation));
    }

    pub fn get_inner(&self) -> &DriverCloudletHandle {
        self.inner.as_ref().unwrap()
    }

    pub fn mark_dirty(&self) -> Result<()> {
        self.save_to_file()
    }

    fn delete_file(&self) -> Result<()> {
        let file_path = Storage::get_cloudlet_file(&self.name);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        let stored_cloudlet = StoredCloudlet {
            driver: self.driver.name().to_string(),
            capabilities: self.capabilities.clone(),
            status: self.status.read().unwrap().clone(),
            controller: self.controller.clone(),
        };
        stored_cloudlet.save_to_file(&Storage::get_cloudlet_file(&self.name))
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Capabilities {
    pub memory: Option<u32>,
    pub max_allocations: Option<u32>,
    pub child: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub enum LifecycleStatus {
    #[serde(rename = "retired")]
    #[default]
    Retired,
    #[serde(rename = "active")]
    Active,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoteController {
    pub address: Url,
}

mod stored {
    use super::{Capabilities, LifecycleStatus, RemoteController};
    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct StoredCloudlet {
        /* Settings */
        pub driver: String,
        pub capabilities: Capabilities,
        pub status: LifecycleStatus,

        /* Controller */
        pub controller: RemoteController,
    }

    impl LoadFromTomlFile for StoredCloudlet {}
    impl SaveToTomlFile for StoredCloudlet {}
}
