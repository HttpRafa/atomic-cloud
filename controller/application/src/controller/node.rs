use std::{fs, net::SocketAddr, path::Path, sync::{Arc, Mutex, Weak}};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use stored::StoredNode;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};
use super::{driver::{DriverHandle, DriverNodeHandle, Drivers, GenericDriver}, server::{Deployment, Resources}, CreationResult};

const NODES_DIRECTORY: &str = "nodes";

pub type NodeHandle = Arc<Node>;
pub type WeakNodeHandle = Weak<Node>;

pub struct Nodes {
    nodes: Vec<NodeHandle>,
}

impl Nodes {
    pub fn load_all(drivers: &Drivers) -> Self {
        info!("Loading nodes...");

        let nodes_directory = Path::new(NODES_DIRECTORY);
        if !nodes_directory.exists() {
            fs::create_dir_all(nodes_directory).unwrap_or_else(|error| {
                warn!("{} to create nodes directory: {}", "Failed".red(), &error)
            });
        }

        let mut nodes = Self { nodes: Vec::new() };
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

            let name = path.file_stem().unwrap().to_string_lossy().to_string();
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

            match nodes.add_node(node) {
                Ok(_) => {},
                Err(error) => {
                    warn!("{} to load node {} because it was denied by the driver", "Failed".red(), &name.blue());
                    warn!(" -> {}", &error);
                }
            }
        }

        info!("Loaded {}", format!("{} node(s)", nodes.nodes.len()).blue());
        nodes
    }

    pub fn get_amount(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_nodes(&self) -> &Vec<NodeHandle> {
        &self.nodes
    }

    pub fn find_by_name(&self, name: &str) -> Option<WeakNodeHandle> {
        for node in &self.nodes {
            if node.name.eq_ignore_ascii_case(name) {
                return Some(Arc::downgrade(node));
            }
        }
        None
    }

    pub fn create_node(&mut self, name: &str, driver: Arc<dyn GenericDriver>, capabilities: Capabilities) -> Result<CreationResult> {
        for node in &self.nodes {
            if node.name == name {
                return Ok(CreationResult::AlreadyExists);
            }
        }

        let stored_node = StoredNode { driver: driver.name().to_string(), capabilities };
        let node = Node::from(name, &stored_node, driver);

        match self.add_node(node) {
            Ok(_) => {
                stored_node.save_to_file(&Path::new(NODES_DIRECTORY).join(format!("{}.toml", name)))?;
                info!("Created node {}", name.blue());
                Ok(CreationResult::Created)
            }
            Err(error) => Ok(CreationResult::Denied(error)),
        }
    }

    fn add_node(&mut self, mut node: Node) -> Result<()> {
        match node.init() {
            Ok(_) => {
                self.nodes.push(Arc::new(node));
                Ok(())
            }
            Err(error) => Err(error),
        }
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
    pub name: String,
    pub capabilities: Capabilities,

    /* Driver handles */
    pub driver: DriverHandle,
    inner: Option<DriverNodeHandle>,

    /* Allocations made on this node */
    pub allocations: Mutex<Vec<AllocationHandle>>,
}

impl Node {
    fn from(name: &str, stored_node: &StoredNode, driver: Arc<dyn GenericDriver>) -> Self {
        Self {
            name: name.to_string(),
            capabilities: stored_node.capabilities.clone(),
            driver,
            inner: None,
            allocations: Mutex::new(Vec::new()),
        }
    }

    fn try_from(name: &str, stored_node: &StoredNode, drivers: &Drivers) -> Option<Self> {
        drivers.find_by_name(&stored_node.driver).map(|driver| Self::from(name, stored_node, driver)).or_else(|| {
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
            },
            Err(error) => Err(error),
        }
    }

    pub fn allocate(&self, resources: &Resources, deployment: Deployment) -> Result<AllocationHandle> {
        let mut allocations = self.allocations.lock().expect("Failed to lock allocations");
        if let Some(max_memory) = self.capabilities.memory {
            let used_memory: u32 = allocations.iter().map(|allocation| allocation.resources.memory).sum();
            if used_memory > max_memory {
                return Err(anyhow!("Node has reached the memory limit"));
            }
        }
        if let Some(max_allocations) = self.capabilities.max_allocations {
            if allocations.len() + 1 > max_allocations as usize {
                return Err(anyhow!("Node has reached the maximum amount of allocations"));
            }
        }

        let addresses = self.inner.as_ref().unwrap().allocate_addresses(resources.addresses)?;
        if addresses.len() < resources.addresses as usize {
            return Err(anyhow!("Node did not allocate the required amount of addresses"));
        }

        let allocation = Arc::new(Allocation {
            addresses,
            resources: resources.clone(),
            deployment,
        });
        allocations.push(allocation.clone());
        Ok(allocation)
    }

    pub fn deallocate(&self, allocation: &AllocationHandle) {
        self.inner.as_ref().unwrap().deallocate_addresses(allocation.addresses.clone()).unwrap_or_else(|error| {
            error!("{} to deallocate addresses: {}", "Failed".red(), &error);
        });
        self.allocations.lock().expect("Failed to lock allocations").retain(|alloc| !Arc::ptr_eq(alloc, allocation));
    }

    pub fn get_inner(&self) -> &DriverNodeHandle {
        self.inner.as_ref().unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Capabilities {
    pub memory: Option<u32>,
    pub max_allocations: Option<u32>,
    pub sub_node: Option<String>,
}

mod stored {
    use serde::{Deserialize, Serialize};
    use crate::config::{LoadFromTomlFile, SaveToTomlFile};

    use super::Capabilities;

    #[derive(Serialize, Deserialize)]
    pub struct StoredNode {
        pub driver: String,
        pub capabilities: Capabilities,
    }

    impl LoadFromTomlFile for StoredNode {}
    impl SaveToTomlFile for StoredNode {}
}