use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, Weak},
};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use stored::StoredNode;
use url::Url;

use super::{
    driver::{DriverHandle, DriverNodeHandle, Drivers, GenericDriver},
    server::{Deployment, Resources, StartRequestHandle},
    CreationResult, WeakControllerHandle,
};
use crate::config::{LoadFromTomlFile, SaveToTomlFile};

const NODES_DIRECTORY: &str = "nodes";

pub type NodeHandle = Arc<Node>;
pub type WeakNodeHandle = Weak<Node>;

pub struct Nodes {
    controller: WeakControllerHandle,

    nodes: HashMap<String, NodeHandle>,
}

impl Nodes {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            nodes: HashMap::new(),
        }
    }

    pub fn load_all(controller: WeakControllerHandle, drivers: &Drivers) -> Self {
        info!("Loading nodes...");

        let nodes_directory = Path::new(NODES_DIRECTORY);
        if !nodes_directory.exists() {
            if let Err(error) = fs::create_dir_all(nodes_directory) {
                warn!("{} to create nodes directory: {}", "Failed".red(), &error);
            }
        }

        let mut nodes = Self::new(controller);
        let entries = match fs::read_dir(nodes_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read nodes directory: {}", "Failed".red(), &error);
                return nodes;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read node entry: {}", "Failed".red(), &error);
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

            let node = match StoredNode::load_from_file(&path) {
                Ok(node) => node,
                Err(error) => {
                    error!(
                        "{} to read node {} from file({:?}): {}",
                        "Failed".red(),
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            info!("Loading node {}", &name.blue());
            let node = match Node::try_from(&name, &node, drivers) {
                Some(node) => node,
                None => continue,
            };

            if let Err(error) = nodes.add_node(node) {
                warn!(
                    "{} to load node {} because it was denied by the driver",
                    "Failed".red(),
                    &name.blue()
                );
                warn!(" -> {}", &error);
            }
        }

        info!("Loaded {}", format!("{} node(s)", nodes.nodes.len()).blue());
        nodes
    }

    pub fn get_amount(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_nodes(&self) -> Vec<NodeHandle> {
        self.nodes.values().cloned().collect()
    }

    pub fn find_by_name(&self, name: &str) -> Option<NodeHandle> {
        self.nodes.get(name).cloned()
    }

    pub fn set_node_status(&mut self, node: &NodeHandle, status: LifecycleStatus) -> Result<()> {
        match status {
            LifecycleStatus::Retired => {
                self.retire_node(node);
                info!("Retired node {}", node.name.blue());
            }
            LifecycleStatus::Active => {
                self.activate_node(node);
                info!("Activated node {}", node.name.blue());
            }
        }
        *node.status.write().unwrap() = status;
        node.mark_dirty()?;
        Ok(())
    }

    fn retire_node(&mut self, node: &NodeHandle) {
        let controller = self
            .controller
            .upgrade()
            .expect("The controller is dead while still running code that requires it");
        {
            controller.lock_groups().search_and_remove_node(node);
            controller.get_servers().stop_all_on_node(node);
        }
    }

    fn activate_node(&mut self, _node: &NodeHandle) {}

    pub fn delete_node(&mut self, node: &NodeHandle) -> Result<()> {
        if *node.status.read().expect("Failed to lock status of node") != LifecycleStatus::Retired {
            return Err(anyhow!("Node is not retired"));
        }
        self.retire_node(node); // Just to be sure
        node.delete_file()?;
        self.remove_node(node);

        let ref_count = Arc::strong_count(node);
        if ref_count > 1 {
            warn!(
                "Node {} still has strong references[{}] this chould indicate a memory leak!",
                node.name.blue(),
                format!("{}", ref_count).red()
            );
        }

        info!("Deleted node {}", node.name.blue());
        Ok(())
    }

    pub fn create_node(
        &mut self,
        name: &str,
        driver: Arc<dyn GenericDriver>,
        capabilities: Capabilities,
        controller: RemoteController,
    ) -> Result<CreationResult> {
        if self.nodes.contains_key(name) {
            return Ok(CreationResult::AlreadyExists);
        }

        let stored_node = StoredNode {
            driver: driver.name().to_string(),
            capabilities,
            status: LifecycleStatus::Retired,
            controller,
        };
        let node = Node::from(name, &stored_node, driver);

        match self.add_node(node) {
            Ok(_) => {
                stored_node
                    .save_to_file(&Path::new(NODES_DIRECTORY).join(format!("{}.toml", name)))?;
                info!("Created node {}", name.blue());
                Ok(CreationResult::Created)
            }
            Err(error) => Ok(CreationResult::Denied(error)),
        }
    }

    fn add_node(&mut self, mut node: Node) -> Result<()> {
        match node.init() {
            Ok(_) => {
                self.nodes.insert(node.name.clone(), Arc::new(node));
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn remove_node(&mut self, node: &NodeHandle) {
        self.nodes.remove(&node.name);
    }
}

pub type AllocationHandle = Arc<Allocation>;

pub struct Allocation {
    pub addresses: Vec<SocketAddr>,
    pub resources: Resources,
    pub deployment: Deployment,
}

impl Allocation {
    pub fn primary_address(&self) -> &SocketAddr {
        &self.addresses[0]
    }
}

pub struct Node {
    /* Settings */
    pub name: String,
    pub capabilities: Capabilities,
    pub status: RwLock<LifecycleStatus>,

    /* Controller */
    pub controller: RemoteController,

    /* Driver handles */
    pub driver: DriverHandle,
    inner: Option<DriverNodeHandle>,

    /* Allocations made on this node */
    pub allocations: RwLock<Vec<AllocationHandle>>,
}

impl Node {
    fn from(name: &str, stored_node: &StoredNode, driver: Arc<dyn GenericDriver>) -> Self {
        Self {
            name: name.to_string(),
            capabilities: stored_node.capabilities.clone(),
            status: RwLock::new(stored_node.status.clone()),
            controller: stored_node.controller.clone(),
            driver,
            inner: None,
            allocations: RwLock::new(Vec::new()),
        }
    }

    fn try_from(name: &str, stored_node: &StoredNode, drivers: &Drivers) -> Option<Self> {
        drivers
            .find_by_name(&stored_node.driver)
            .map(|driver| Self::from(name, stored_node, driver))
            .or_else(|| {
                error!(
                    "{} to load node {} because there is no loaded driver with the name {}",
                    "Failed".red(),
                    &name.red(),
                    &stored_node.driver.red()
                );
                None
            })
    }

    pub fn init(&mut self) -> Result<()> {
        match self.driver.init_node(self) {
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
                "Attempted to allocate resources on {} node {}",
                "retired".red(),
                self.name.blue()
            );
            return Err(anyhow!("Can not allocate resources on retired node"));
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
                return Err(anyhow!("Node has reached the memory limit"));
            }
        }

        if let Some(max_allocations) = self.capabilities.max_allocations {
            if allocations.len() + 1 > max_allocations as usize {
                return Err(anyhow!(
                    "Node has reached the maximum amount of allocations"
                ));
            }
        }

        let addresses = self.inner.as_ref().unwrap().allocate_addresses(request)?;
        if addresses.len() < request.resources.addresses as usize {
            return Err(anyhow!(
                "Node did not allocate the required amount of addresses"
            ));
        }

        let allocation = Arc::new(Allocation {
            addresses,
            resources: request.resources.clone(),
            deployment: request.deployment.clone(),
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

    pub fn get_inner(&self) -> &DriverNodeHandle {
        self.inner.as_ref().unwrap()
    }

    pub fn mark_dirty(&self) -> Result<()> {
        self.save_to_file()
    }

    fn delete_file(&self) -> Result<()> {
        let file_path = self.get_file_path();
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        let stored_node = StoredNode {
            driver: self.driver.name().to_string(),
            capabilities: self.capabilities.clone(),
            status: self.status.read().unwrap().clone(),
            controller: self.controller.clone(),
        };
        stored_node.save_to_file(&self.get_file_path())
    }

    fn get_file_path(&self) -> PathBuf {
        Path::new(NODES_DIRECTORY).join(format!("{}.toml", self.name))
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Capabilities {
    pub memory: Option<u32>,
    pub max_allocations: Option<u32>,
    pub sub_node: Option<String>,
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
    use crate::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct StoredNode {
        /* Settings */
        pub driver: String,
        pub capabilities: Capabilities,
        pub status: LifecycleStatus,

        /* Controller */
        pub controller: RemoteController,
    }

    impl LoadFromTomlFile for StoredNode {}
    impl SaveToTomlFile for StoredNode {}
}
