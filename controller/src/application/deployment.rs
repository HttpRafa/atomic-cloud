use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock, Weak,
    },
    time::Instant,
};

use anyhow::{anyhow, Result};
use common::config::{LoadFromTomlFile, SaveToTomlFile};
use serde::{Deserialize, Serialize};
use shared::StoredDeployment;
use simplelog::{debug, error, info, warn};

use crate::storage::Storage;

use super::{
    cloudlet::{CloudletHandle, Cloudlets, LifecycleStatus, WeakCloudletHandle},
    unit::{DeploymentRef, Resources, Spec, StartRequest, StartRequestHandle, UnitHandle, Units},
    CreationResult, WeakControllerHandle,
};

pub type DeploymentHandle = Arc<Deployment>;
pub type WeakDeploymentHandle = Weak<Deployment>;

pub struct Deployments {
    controller: WeakControllerHandle,

    deployments: HashMap<String, DeploymentHandle>,
}

impl Deployments {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            deployments: HashMap::new(),
        }
    }

    pub fn load_all(controller: WeakControllerHandle, cloudlets: &Cloudlets) -> Self {
        info!("Loading deployments...");

        let mut deployments = Self::new(controller);
        let deployments_directory = Storage::get_deployments_folder();
        if !deployments_directory.exists() {
            if let Err(error) = fs::create_dir_all(&deployments_directory) {
                warn!(
                    "<red>Failed</> to create deployments directory: <red>{}</>",
                    &error
                );
                return deployments;
            }
        }

        let entries = match fs::read_dir(&deployments_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!(
                    "<red>Failed</> to read deployments directory: <red>{}</>",
                    &error
                );
                return deployments;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("<red>Failed</> to read deployment entry: <red>{}</>", &error);
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

            let deployment = match StoredDeployment::load_from_file(&path) {
                Ok(deployment) => deployment,
                Err(error) => {
                    error!(
                        "<red>Failed</> to read deployment <blue>{}</> from file(<blue>{:?}</>): <red>{}</>",
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            let deployment = match Deployment::try_from(&name, &deployment, cloudlets) {
                Some(deployment) => deployment,
                None => continue,
            };

            deployments.add_deployment(deployment);
            info!("Loaded deployment <blue>{}</>", &name);
        }

        info!(
            "Loaded <blue>{} deployment(s)</>",
            deployments.deployments.len()
        );
        deployments
    }

    pub fn get_amount(&self) -> usize {
        self.deployments.len()
    }

    pub fn get_deployments(&self) -> &HashMap<String, DeploymentHandle> {
        &self.deployments
    }

    pub fn tick(&self, units: &Units) {
        for deployment in self.deployments.values() {
            deployment.tick(&self.controller, units);
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<DeploymentHandle> {
        self.deployments.get(name).cloned()
    }

    pub fn set_deployment_status(
        &mut self,
        deployment: &DeploymentHandle,
        status: LifecycleStatus,
    ) -> Result<()> {
        match status {
            LifecycleStatus::Retired => {
                self.retire_deployment(deployment);
                info!("<red>Retired</> deployment <blue>{}</>", deployment.name);
            }
            LifecycleStatus::Active => {
                self.activate_deployment(deployment);
                info!("<green>Activated</> deployment <blue>{}</>", deployment.name);
            }
        }
        *deployment.status.write().unwrap() = status;
        deployment.mark_dirty()?;
        Ok(())
    }

    fn retire_deployment(&mut self, deployment: &DeploymentHandle) {
        let controller = self
            .controller
            .upgrade()
            .expect("The controller is dead while still running code that requires it");
        {
            let unit_manager = controller.get_units();
            let mut units = deployment.units.write().unwrap();
            for unit in units.iter() {
                if let AssociatedUnit::Active(unit) = unit {
                    unit_manager.checked_unit_stop(unit);
                } else if let AssociatedUnit::Queueing(request) = unit {
                    request.canceled.store(true, Ordering::Relaxed);
                }
            }
            units.clear();
        }
    }

    fn activate_deployment(&mut self, _deployment: &DeploymentHandle) {}

    pub fn delete_deployment(&mut self, deployment: &DeploymentHandle) -> Result<()> {
        if *deployment
            .status
            .read()
            .expect("Failed to lock status of deployment")
            != LifecycleStatus::Retired
        {
            return Err(anyhow!("Deployment is not retired"));
        }
        self.retire_deployment(deployment); // Make sure all units are stopped
        deployment.delete_file()?;
        self.remove_deployment(deployment);

        let ref_count = Arc::strong_count(deployment);
        if ref_count > 1 {
            warn!(
                "Deployment <blue>{}</> still has strong references[<red>{}</>] this chould indicate a memory leak!",
                deployment.name,
                ref_count
            );
        }

        info!("<red>Deleted</> deployment <blue>{}</>", deployment.name);
        Ok(())
    }

    pub fn create_deployment(
        &mut self,
        name: &str,
        cloudlet_handles: Vec<CloudletHandle>,
        constraints: StartConstraints,
        scaling: ScalingPolicy,
        resources: Resources,
        spec: Spec,
    ) -> Result<CreationResult> {
        if cloudlet_handles.is_empty() {
            return Ok(CreationResult::Denied(anyhow!("No cloudlet provided")));
        }

        if self.deployments.contains_key(name) {
            return Ok(CreationResult::AlreadyExists);
        }

        let cloudlets: Vec<String> = cloudlet_handles
            .iter()
            .map(|cloudlet| cloudlet.name.clone())
            .collect();

        let stored_deployment = StoredDeployment {
            status: LifecycleStatus::Retired,
            cloudlets,
            constraints,
            scaling,
            resources,
            spec,
        };
        let deployment = Deployment::from(
            name,
            &stored_deployment,
            cloudlet_handles.iter().map(Arc::downgrade).collect(),
        );

        self.add_deployment(deployment);
        stored_deployment.save_to_file(&Storage::get_deployment_file(name))?;
        info!("<green>Created</> deployment <blue>{}</>", name);
        Ok(CreationResult::Created)
    }

    pub fn search_and_remove_cloudlet(&self, cloudlet: &CloudletHandle) {
        for deployment in self.deployments.values() {
            deployment
                .cloudlets
                .write()
                .expect("Failed to lock cloudlets list of deployment")
                .retain(|handle| {
                    if let Some(strong_cloudlet) = handle.upgrade() {
                        return !Arc::ptr_eq(&strong_cloudlet, cloudlet);
                    }
                    false
                });
            deployment
                .mark_dirty()
                .expect("Failed to mark deployment as dirty");
        }
    }

    fn add_deployment(&mut self, deployment: DeploymentHandle) {
        self.deployments
            .insert(deployment.name.to_string(), deployment);
    }

    fn remove_deployment(&mut self, deployment: &DeploymentHandle) {
        self.deployments.remove(&deployment.name);
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct StartConstraints {
    pub minimum: u32,
    pub maximum: u32,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ScalingPolicy {
    pub enabled: bool,
    pub max_players: u32,
    pub start_threshold: f32,
    pub stop_empty_units: bool,
}

pub enum AssociatedUnit {
    Queueing(StartRequestHandle),
    Active(UnitHandle),
}

pub struct Deployment {
    handle: WeakDeploymentHandle,

    /* Settings */
    pub name: String,
    pub status: RwLock<LifecycleStatus>,

    /* Where? */
    pub cloudlets: RwLock<Vec<WeakCloudletHandle>>,
    pub constraints: StartConstraints,
    pub scaling: ScalingPolicy,

    /* How? */
    pub resources: Resources,
    pub spec: Spec,

    /* What do i need to know? */
    id_allocator: RwLock<IdAllocator>,
    units: RwLock<Vec<AssociatedUnit>>,
}

impl Deployment {
    fn from(
        name: &str,
        stored_deployment: &StoredDeployment,
        cloudlets: Vec<WeakCloudletHandle>,
    ) -> DeploymentHandle {
        Arc::new_cyclic(|handle| Self {
            handle: handle.clone(),
            name: name.to_string(),
            status: RwLock::new(stored_deployment.status.clone()),
            cloudlets: RwLock::new(cloudlets),
            constraints: stored_deployment.constraints,
            scaling: stored_deployment.scaling,
            resources: stored_deployment.resources.clone(),
            spec: stored_deployment.spec.clone(),
            id_allocator: RwLock::new(IdAllocator::new()),
            units: RwLock::new(Vec::new()),
        })
    }

    fn try_from(
        name: &str,
        stored_deployment: &StoredDeployment,
        cloudlets: &Cloudlets,
    ) -> Option<DeploymentHandle> {
        let cloudlet_handles: Vec<WeakCloudletHandle> = stored_deployment
            .cloudlets
            .iter()
            .filter_map(|name| {
                cloudlets
                    .find_by_name(name)
                    .map(|handle| Arc::downgrade(&handle))
            })
            .collect();
        if cloudlet_handles.is_empty() {
            return None;
        }
        Some(Self::from(name, stored_deployment, cloudlet_handles))
    }

    fn tick(&self, controller: &WeakControllerHandle, units: &Units) {
        if *self.status.read().unwrap() == LifecycleStatus::Retired {
            // Do not tick this deployment because it is retired
            return;
        }

        let mut deployment_units = self.units.write().expect("Failed to lock units");
        let mut id_allocator = self
            .id_allocator
            .write()
            .expect("Failed to lock id allocator");
        let mut target_unit_count = self.constraints.minimum;

        // Apply scaling policy
        if self.scaling.enabled {
            for unit in deployment_units.iter() {
                if let AssociatedUnit::Active(unit) = unit {
                    let player_ratio =
                        unit.get_user_count() as f32 / self.scaling.max_players as f32;
                    if player_ratio >= self.scaling.start_threshold {
                        target_unit_count += 1; // Unit has reached the threshold, start a new one
                    }
                }
            }

            if self.scaling.stop_empty_units && deployment_units.len() as u32 > target_unit_count {
                let mut amount_to_stop = deployment_units.len() as u32 - target_unit_count;

                // We have more units than needed
                // Check if a unit is empty and stop it after the configured timeout
                if let Some(controller) = controller.upgrade() {
                    for unit in deployment_units.iter() {
                        if let AssociatedUnit::Active(unit) = unit {
                            let mut stop_flag =
                                unit.flags.stop.write().expect("Failed to lock stop flag");
                            if unit.get_user_count() == 0 {
                                if let Some(stop_time) = stop_flag.as_ref() {
                                    if &Instant::now() > stop_time && amount_to_stop > 0 {
                                        debug!(
                                            "Unit <blue>{}</> is empty and reached the timeout, <red>stopping</> it...",
                                            unit.name
                                        );
                                        controller.get_units().checked_unit_stop(unit);
                                        amount_to_stop -= 1;
                                    }
                                } else {
                                    debug!(
                                        "Unit <blue>{}</> is empty, starting stop timer...",
                                        unit.name
                                    );
                                    stop_flag.replace(
                                        Instant::now()
                                            + controller.configuration.timings.empty_unit.unwrap(),
                                    );
                                }
                            } else if stop_flag.is_some() {
                                debug!(
                                    "Unit <blue>{}</> is no longer empty, clearing stop timer...",
                                    unit.name
                                );
                                stop_flag.take();
                            }
                        }
                    }
                }
            }
        }

        // Check if we need to start more units
        for requested in 0..(target_unit_count as usize).saturating_sub(deployment_units.len()) {
            if (deployment_units.len() + requested) >= target_unit_count as usize {
                break;
            }

            let unit_id = id_allocator.get_id();
            let request = units.queue_unit(StartRequest {
                canceled: AtomicBool::new(false),
                when: None,
                name: format!("{}-{}", self.name, unit_id),
                cloudlets: self.cloudlets.read().unwrap().clone(),
                deployment: Some(DeploymentRef {
                    unit_id,
                    deployment: self.handle.clone(),
                }),
                resources: self.resources.clone(),
                spec: self.spec.clone(),
                priority: self.constraints.priority,
            });

            // Add queueing unit to deployment
            deployment_units.push(AssociatedUnit::Queueing(request));
        }
    }

    pub fn set_unit_active(&self, unit: UnitHandle, request: &StartRequestHandle) {
        let mut units = self.units.write().expect("Failed to lock units");
        units.retain(|queued_unit| {
            if let AssociatedUnit::Queueing(start_request) = queued_unit {
                return !Arc::ptr_eq(start_request, request);
            }
            true
        });
        units.push(AssociatedUnit::Active(unit));
    }

    pub fn remove_unit(&self, unit: &UnitHandle) {
        self.units
            .write()
            .expect("Failed to lock units")
            .retain(|handle| {
                if let AssociatedUnit::Active(s) = handle {
                    return !Arc::ptr_eq(s, unit);
                }
                true
            });
        self.id_allocator
            .write()
            .expect("Failed to lock id allocator")
            .release_id(unit.deployment.as_ref().unwrap().unit_id);
    }

    pub fn get_free_unit(&self) -> Option<UnitHandle> {
        let units = self.units.read().expect("Failed to lock units");
        for unit in units.iter() {
            if let AssociatedUnit::Active(unit) = unit {
                return Some(unit.clone());
            }
        }
        None
    }

    pub fn mark_dirty(&self) -> Result<()> {
        self.save_to_file()
    }

    fn delete_file(&self) -> Result<()> {
        let file_path = Storage::get_deployment_file(&self.name);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        let stored_deployment = StoredDeployment {
            status: self.status.read().unwrap().clone(),
            cloudlets: self
                .cloudlets
                .read()
                .unwrap()
                .iter()
                .map(|cloudlet| cloudlet.upgrade().unwrap().name.clone())
                .collect(),
            constraints: self.constraints,
            scaling: self.scaling,
            resources: self.resources.clone(),
            spec: self.spec.clone(),
        };
        stored_deployment.save_to_file(&Storage::get_deployment_file(&self.name))
    }
}

struct IdAllocator {
    next_id: usize,
    available_ids: BTreeSet<usize>,
    active_ids: HashSet<usize>,
}

impl IdAllocator {
    fn new() -> Self {
        Self {
            next_id: 1,
            available_ids: BTreeSet::new(),
            active_ids: HashSet::new(),
        }
    }

    fn get_id(&mut self) -> usize {
        if let Some(&id) = self.available_ids.iter().next() {
            self.available_ids.remove(&id);
            self.active_ids.insert(id);
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.active_ids.insert(id);
            id
        }
    }

    fn release_id(&mut self, id: usize) {
        if self.active_ids.remove(&id) {
            self.available_ids.insert(id);
        }
    }
}

mod shared {
    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    use crate::application::{
        cloudlet::LifecycleStatus,
        unit::{Resources, Spec},
    };

    use super::{ScalingPolicy, StartConstraints};

    #[derive(Serialize, Deserialize)]
    pub struct StoredDeployment {
        /* Settings */
        pub status: LifecycleStatus,

        /* Where? */
        pub cloudlets: Vec<String>,
        pub constraints: StartConstraints,
        pub scaling: ScalingPolicy,

        /* How? */
        pub resources: Resources,
        pub spec: Spec,
    }

    impl LoadFromTomlFile for StoredDeployment {}
    impl SaveToTomlFile for StoredDeployment {}
}
